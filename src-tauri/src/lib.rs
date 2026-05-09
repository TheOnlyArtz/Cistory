use anyhow::Context;
use chrono::Utc;
use clipboard::{ArboardBackend, ClipboardService, ClipboardSnapshot};
use domain::{EntryQuery, Settings};
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager};
use hotkey::{plan_registration, HotkeyBackendKind, HotkeyBinding};
use parking_lot::Mutex;
use std::{
    fs,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use storage::SqliteStore;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::WebviewWindowBuilder,
    AppHandle, Emitter, LogicalPosition, Manager, RunEvent, State, WebviewUrl,
};
use tauri_plugin_autostart::{Builder as AutostartBuilder, ManagerExt as AutostartExt};
use tracing::{error, warn};

const PICKER_LABEL: &str = "picker";
const HISTORY_EVENT: &str = "history-updated";
const PICKER_OPENED_EVENT: &str = "picker-opened";
const PICKER_WIDTH: f64 = 420.0;
const PICKER_HEIGHT: f64 = 400.0;
const TRAY_ICON_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
    0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
    0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78,
    0x9c, 0x63, 0xf8, 0xcf, 0xc0, 0xf0, 0x1f, 0x00, 0x05, 0x00, 0x01, 0xff, 0x89, 0x99,
    0x3d, 0x1d, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

struct AppState {
    store: Arc<SqliteStore>,
    clipboard: Arc<ClipboardService<ArboardBackend>>,
    image_cache_dir: PathBuf,
    active_hotkey: Mutex<String>,
    hotkey_manager: Mutex<Option<GlobalHotKeyManager>>,
    registered_hotkey: Mutex<Option<HotKey>>,
    diagnostics: Mutex<Vec<String>>,
    hotkey_listener_started: AtomicBool,
    quit_requested: AtomicBool,
    suppress_blur_hide: AtomicBool,
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error(transparent)]
    Storage(#[from] storage::StorageError),
    #[error(transparent)]
    Clipboard(#[from] clipboard::ClipboardError),
    #[error(transparent)]
    Domain(#[from] domain::DomainError),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Hotkey(#[from] hotkey::HotkeyError),
    #[error(transparent)]
    GlobalHotKey(#[from] global_hotkey::Error),
    #[error(transparent)]
    Autostart(#[from] tauri_plugin_autostart::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    PngEncoding(#[from] png::EncodingError),
    #[error(transparent)]
    PngDecoding(#[from] png::DecodingError),
    #[error("image entry {0} is missing a local image path")]
    MissingImagePath(i64),
    #[error("application state is unavailable")]
    MissingAppState,
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command]
fn list_entries(
    state: State<'_, AppState>,
    query: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<domain::ClipboardEntry>, AppError> {
    state.store.list_entries(&EntryQuery {
        query,
        limit: limit.unwrap_or(100),
    }).map_err(AppError::from)
}

#[tauri::command]
fn select_entry(app: AppHandle, state: State<'_, AppState>, entry_id: i64) -> Result<(), AppError> {
    let entry = state.store.get_entry(entry_id)?;
    match entry.content_kind {
        domain::ContentKind::Text => {
            state.clipboard.write_text(entry.content)?;
        }
        domain::ContentKind::Image => {
            let image_path = entry
                .image_path
                .clone()
                .ok_or(AppError::MissingImagePath(entry.id))?;
            let path = PathBuf::from(image_path);
            if !path.exists() {
                record_diagnostic(
                    &app,
                    format!("image file missing for entry {}: {}", entry.id, path.display()),
                );
            } else {
                let (width, height, bytes) = load_png_from_path(&path)?;
                state.clipboard.write_image(width, height, bytes)?;
            }
        }
    }
    if let Some(window) = app.get_webview_window(PICKER_LABEL) {
        let _ = window.hide();
    }
    Ok(())
}

#[tauri::command]
fn toggle_pin(state: State<'_, AppState>, entry_id: i64) -> Result<(), AppError> {
    state.store.toggle_pin(entry_id)?;
    Ok(())
}

#[tauri::command]
fn load_settings(state: State<'_, AppState>) -> Result<Settings, AppError> {
    state.store.load_settings().map_err(AppError::from)
}

#[tauri::command]
fn list_diagnostics(state: State<'_, AppState>) -> Vec<String> {
    state.diagnostics.lock().clone()
}

#[tauri::command]
fn set_autostart(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<Settings, AppError> {
    if enabled {
        app.autolaunch().enable()?;
    } else {
        app.autolaunch().disable()?;
    }

    let settings = Settings {
        autostart_enabled: enabled,
        ..state.store.load_settings()?
    }
    .validate()?;

    state.store.save_settings(&settings).map_err(AppError::from)
}

#[tauri::command]
fn set_hotkey(
    app: AppHandle,
    state: State<'_, AppState>,
    binding: String,
) -> Result<Settings, AppError> {
    let binding = HotkeyBinding::parse(binding)?;
    register_hotkey(&app, &binding)?;

    {
        let mut active_hotkey = state.active_hotkey.lock();
        *active_hotkey = binding.value.clone();
    }

    let settings = Settings {
        hotkey_binding: binding.value,
        ..state.store.load_settings()?
    }
    .validate()?;

    state.store.save_settings(&settings).map_err(AppError::from)
}

#[tauri::command]
fn hide_picker(app: AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window(PICKER_LABEL) {
        window.hide()?;
    }
    Ok(())
}

#[tauri::command]
fn set_recording_hotkey(state: State<'_, AppState>, recording: bool) {
    state.suppress_blur_hide.store(recording, Ordering::Relaxed);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(AutostartBuilder::new().app_name("Cistory").build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_picker(&app);
        }))
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .context("failed to resolve app data directory")?;
            std::fs::create_dir_all(&data_dir)?;
            let database_path = data_dir.join("history.sqlite3");

            let store = Arc::new(SqliteStore::open(&database_path)?);
            tighten_local_permissions(&database_path)?;
            let mut settings = store.load_settings()?;
            let clipboard = Arc::new(ClipboardService::new(ArboardBackend::new()?));
            let image_cache_dir = data_dir.join("images");
            fs::create_dir_all(&image_cache_dir)?;

            app.manage(AppState {
                store: Arc::clone(&store),
                clipboard: Arc::clone(&clipboard),
                image_cache_dir,
                active_hotkey: Mutex::new(settings.hotkey_binding.clone()),
                hotkey_manager: Mutex::new(None),
                registered_hotkey: Mutex::new(None),
                diagnostics: Mutex::new(Vec::new()),
                hotkey_listener_started: AtomicBool::new(false),
                quit_requested: AtomicBool::new(false),
                suppress_blur_hide: AtomicBool::new(false),
            });

            settings.autostart_enabled = app.autolaunch().is_enabled()?;
            let settings = store.save_settings(&settings)?;

            ensure_picker_window(app.handle())?;
            setup_tray(app.handle())?;
            spawn_hotkey_listener(app.handle());
            if let Err(error) = register_hotkey(app.handle(), &HotkeyBinding::parse(settings.hotkey_binding)?) {
                warn!(%error, "hotkey registration failed during setup");
                record_diagnostic(app.handle(), format!("hotkey registration failed during setup: {error}"));
            }
            spawn_clipboard_loop(app.handle(), store, clipboard);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_entries,
            select_entry,
            toggle_pin,
            load_settings,
            list_diagnostics,
            set_autostart,
            set_hotkey,
            hide_picker,
            set_recording_hotkey,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Tauri application")
        .run(|app_handle, event| match event {
            RunEvent::ExitRequested { api, .. } => {
                if let Some(state) = app_handle.try_state::<AppState>() {
                    if !state.quit_requested.load(Ordering::Relaxed) {
                        api.prevent_exit();
                        if let Err(error) = hide_picker(app_handle.clone()) {
                            warn!(%error, "failed to hide picker during exit request");
                        }
                    }
                }
            }
            RunEvent::WindowEvent { label, event, .. } if label == PICKER_LABEL => {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        if let Err(error) = hide_picker(app_handle.clone()) {
                            warn!(%error, "failed to hide picker after close request");
                        }
                    }
                    tauri::WindowEvent::Focused(false) => {
                        let suppress_blur_hide = app_handle
                            .try_state::<AppState>()
                            .is_some_and(|state| state.suppress_blur_hide.load(Ordering::Relaxed));

                        if !suppress_blur_hide {
                            if let Err(error) = hide_picker(app_handle.clone()) {
                                warn!(%error, "failed to hide picker after blur");
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        });
}

fn ensure_picker_window(app: &AppHandle) -> Result<(), AppError> {
    if app.get_webview_window(PICKER_LABEL).is_some() {
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, PICKER_LABEL, WebviewUrl::App("index.html".into()))
        .title("Cistory")
        .visible(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .inner_size(PICKER_WIDTH, PICKER_HEIGHT)
        .build()?;

    let _ = position_picker_window(&window);
    Ok(())
}

fn position_picker_window(window: &tauri::WebviewWindow) -> Result<(), AppError> {
    let Some(monitor) = window.current_monitor()?.or(window.primary_monitor()?) else {
        return Ok(());
    };

    let work_area = monitor.work_area();
    let window_size = window.outer_size()?;
    let scale_factor = monitor.scale_factor();
    let width = f64::from(window_size.width) / scale_factor;
    let height = f64::from(window_size.height) / scale_factor;
    let x = f64::from(work_area.position.x) + f64::from(work_area.size.width) - width;
    let y = f64::from(work_area.position.y) + f64::from(work_area.size.height) - height;

    window.set_position(LogicalPosition::new(x, y))?;
    Ok(())
}

fn setup_tray(app: &AppHandle) -> Result<(), AppError> {
    let show_item = MenuItem::with_id(app, "show", "Show Picker", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
    let image = Image::from_bytes(TRAY_ICON_PNG)?;

    TrayIconBuilder::new()
        .icon(image)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                let _ = show_picker(app);
            }
            "quit" => {
                if let Some(state) = app.try_state::<AppState>() {
                    state.quit_requested.store(true, Ordering::Relaxed);
                }
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_picker(&tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

fn show_picker(app: &AppHandle) -> Result<(), AppError> {
    ensure_picker_window(app)?;
    if let Some(window) = app.get_webview_window(PICKER_LABEL) {
        position_picker_window(&window)?;
        window.show()?;
        let _ = window.unminimize();
        let _ = window.set_focus();
        focus_picker_window(app);
        app.emit(PICKER_OPENED_EVENT, ())?;
    }
    Ok(())
}

fn focus_picker_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(PICKER_LABEL) {
        if let Err(error) = window.set_focus() {
            warn!(%error, "failed to focus picker window");
        }
    }

    let app_handle = app.clone();
    thread::spawn(move || {
        for delay in [Duration::from_millis(40), Duration::from_millis(80)] {
            thread::sleep(delay);

            let app_for_main = app_handle.clone();
            if let Err(error) = app_handle.run_on_main_thread(move || {
                let Some(window) = app_for_main.get_webview_window(PICKER_LABEL) else {
                    return;
                };

                let is_visible = window.is_visible().unwrap_or(false);
                let is_focused = window.is_focused().unwrap_or(false);
                if is_visible && !is_focused {
                    if let Err(error) = window.set_focus() {
                        warn!(%error, "failed to refocus picker window");
                    }
                }
            }) {
                warn!(%error, "failed to dispatch picker refocus on main thread");
                break;
            }
        }
    });
}

fn spawn_hotkey_listener(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };

    if state.hotkey_listener_started.swap(true, Ordering::Relaxed) {
        return;
    }

    let app_handle = app.clone();
    thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            let Some(state) = app_handle.try_state::<AppState>() else {
                continue;
            };

            let active_id = state
                .registered_hotkey
                .lock()
                .as_ref()
                .map(|hotkey| hotkey.id);

            if active_id.is_some_and(|id| id == event.id) {
                let app_for_main = app_handle.clone();
                if let Err(error) = app_handle.run_on_main_thread(move || {
                    if let Err(error) = show_picker(&app_for_main) {
                        warn!(%error, "failed to show picker from hotkey event");
                        record_diagnostic(
                            &app_for_main,
                            format!("failed to show picker from hotkey event: {error}"),
                        );
                    }
                }) {
                    warn!(%error, "failed to dispatch hotkey event to main thread");
                    record_diagnostic(
                        &app_handle,
                        format!("failed to dispatch hotkey event to main thread: {error}"),
                    );
                }
            }
        }
    });
}

fn register_hotkey(app: &AppHandle, binding: &HotkeyBinding) -> Result<(), AppError> {
    let session_type = std::env::var("XDG_SESSION_TYPE").ok();
    let registration = plan_registration(binding.clone(), session_type.as_deref())?;

    match registration.backend {
        HotkeyBackendKind::X11 => register_x11_hotkey(app, binding),
        HotkeyBackendKind::Portal => {
            warn!(binding = %binding.value, "Wayland portal hotkey registration is deferred; binding is stored for future portal setup");
            Ok(())
        }
        HotkeyBackendKind::Unsupported => Ok(()),
    }
}

fn register_x11_hotkey(app: &AppHandle, binding: &HotkeyBinding) -> Result<(), AppError> {
    let state = app
        .try_state::<AppState>()
        .expect("application state is registered before hotkey setup");
    let hotkey = binding.to_global_hotkey()?;
    let mut manager_slot = state.hotkey_manager.lock();
    let manager = if let Some(manager) = manager_slot.as_mut() {
        manager
    } else {
        manager_slot.insert(GlobalHotKeyManager::new()?)
    };

    if let Some(previous_hotkey) = state.registered_hotkey.lock().take() {
        manager.unregister(previous_hotkey)?;
    }

    manager.register(hotkey.clone())?;
    *state.registered_hotkey.lock() = Some(hotkey);

    Ok(())
}

fn spawn_clipboard_loop(
    app: &AppHandle,
    store: Arc<SqliteStore>,
    clipboard: Arc<ClipboardService<ArboardBackend>>,
) {
    let app_handle = app.clone();

    thread::spawn(move || loop {
        if let Err(error) = poll_clipboard_once(&app_handle, &store, &clipboard) {
            error!(%error, "clipboard polling failed");
            record_diagnostic(&app_handle, format!("clipboard polling failed: {error}"));
        }

        thread::sleep(Duration::from_millis(500));
    });
}

fn record_diagnostic(app: &AppHandle, message: String) {
    if let Some(state) = app.try_state::<AppState>() {
        let mut diagnostics = state.diagnostics.lock();
        diagnostics.push(message.clone());
        if diagnostics.len() > 50 {
            diagnostics.remove(0);
        }
    }

    let _ = app.emit("diagnostic-recorded", message);
}

#[cfg(unix)]
fn tighten_local_permissions(path: &std::path::Path) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;

    if !path.exists() {
        return Ok(());
    }

    let permissions = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(path, permissions)
}

#[cfg(not(unix))]
fn tighten_local_permissions(_path: &std::path::Path) -> Result<(), std::io::Error> {
    Ok(())
}

fn poll_clipboard_once(
    app: &AppHandle,
    store: &SqliteStore,
    clipboard: &ClipboardService<ArboardBackend>,
) -> Result<(), AppError> {
    let Some(snapshot) = clipboard.snapshot()? else {
        return Ok(());
    };

    let entry = match snapshot {
        ClipboardSnapshot::Text { .. } => ClipboardService::<ArboardBackend>::to_entry(snapshot, None)?,
        ClipboardSnapshot::Image {
            bytes,
            width,
            height,
            content_hash,
        } => {
            let state = app
                .try_state::<AppState>()
                .ok_or(AppError::MissingAppState)?;
            let image_path = persist_image_snapshot(&state.image_cache_dir, &content_hash, width, height, &bytes)?;
            domain::NewClipboardEntry::new_image(content_hash, image_path, None, Utc::now())?
        }
    };

    store.upsert_entry(&entry)?;

    let retention_days = store.load_settings()?.retention_days;
    let (_, stale_image_paths) = store.prune_expired_entries_with_paths(retention_days)?;
    cleanup_stale_image_paths(stale_image_paths);
    app.emit(HISTORY_EVENT, Utc::now().to_rfc3339())?;
    Ok(())
}

fn persist_image_snapshot(
    image_cache_dir: &Path,
    content_hash: &str,
    width: usize,
    height: usize,
    bytes: &[u8],
) -> Result<String, AppError> {
    fs::create_dir_all(image_cache_dir)?;
    let image_path = image_cache_dir.join(format!("{content_hash}.png"));
    if !image_path.exists() {
        save_png_to_path(&image_path, width, height, bytes)?;
        let _ = tighten_local_permissions(&image_path);
    }

    Ok(image_path.to_string_lossy().to_string())
}

fn save_png_to_path(path: &Path, width: usize, height: usize, rgba: &[u8]) -> Result<(), AppError> {
    let file = fs::File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder.write_header()?;
    png_writer.write_image_data(rgba)?;
    Ok(())
}

fn load_png_from_path(path: &Path) -> Result<(usize, usize, Vec<u8>), AppError> {
    let file = fs::File::open(path)?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info()?;
    let output_buffer_size = reader.output_buffer_size().ok_or_else(|| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "png decoder did not provide output buffer size",
        ))
    })?;
    let mut buffer = vec![0; output_buffer_size];
    let output = reader.next_frame(&mut buffer)?;
    let data = &buffer[..output.buffer_size()];

    let rgba = match output.color_type {
        png::ColorType::Rgba => data.to_vec(),
        png::ColorType::Rgb => data
            .chunks_exact(3)
            .flat_map(|chunk| [chunk[0], chunk[1], chunk[2], 255])
            .collect(),
        png::ColorType::Grayscale => data
            .iter()
            .flat_map(|value| [*value, *value, *value, 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => data
            .chunks_exact(2)
            .flat_map(|chunk| [chunk[0], chunk[0], chunk[0], chunk[1]])
            .collect(),
        png::ColorType::Indexed => {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "indexed-color PNG is not supported for clipboard copyback",
            )))
        }
    };

    Ok((output.width as usize, output.height as usize, rgba))
}

fn cleanup_stale_image_paths(paths: Vec<String>) {
    for path in paths {
        if let Err(error) = fs::remove_file(&path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                warn!(%error, %path, "failed to remove stale image path");
            }
        }
    }
}

#[allow(dead_code)]
fn default_database_path() -> PathBuf {
    PathBuf::from("history.sqlite3")
}

#[cfg(test)]
mod tests {
    use super::tighten_local_permissions;
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    #[cfg(unix)]
    #[test]
    fn tightens_database_permissions_on_unix() {
        use std::os::unix::fs::PermissionsExt;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let file_path = std::env::temp_dir().join(format!("cistory-permissions-{unique}.sqlite3"));
        fs::write(&file_path, b"test").expect("write temp database");

        tighten_local_permissions(&file_path).expect("tighten permissions");

        let mode = fs::metadata(&file_path).expect("metadata").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        let _ = fs::remove_file(file_path);
    }
}
