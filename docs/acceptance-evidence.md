# Acceptance Evidence

## Automated Checks Completed

- `npm run build`
- `cargo check --workspace` from `src-tauri/`
- `cargo test --workspace` from `src-tauri/`

## Automated Coverage Notes

- Domain invariants are covered by unit tests in `src-tauri/crates/domain`.
- SQLite migration, persistence, deduplication, and retention behaviors are covered by unit tests in `src-tauri/crates/storage`.
- Clipboard recursion and rapid-change behavior are covered by unit tests in `src-tauri/crates/clipboard`.
- Hotkey binding parsing and backend planning are covered by unit tests in `src-tauri/crates/hotkey`.
- Local database file permissions are covered by a Unix-only test in `src-tauri/src/lib.rs`.

## Manual Validation Still Required

- GNOME X11 tray, hotkey, and picker lifecycle validation.
- GNOME Wayland portal fallback behavior and picker lifecycle validation.
- KDE X11 tray, hotkey, and picker lifecycle validation.
- KDE Wayland portal fallback behavior and picker lifecycle validation.
- Reboot/autostart validation in a real desktop session.
- Packaging, fresh-install, and upgrade-path validation.
- Performance-budget measurement against the target SLOs.
