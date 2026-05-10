import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./style.css";

type ClipboardEntry = {
  id: number;
  content: string;
  content_hash: string;
  content_kind: "text" | "image";
  image_path: string | null;
  captured_at: string;
  source_app: string | null;
  pinned: boolean;
};

type AppSettings = {
  hotkey_binding: string;
  autostart_enabled: boolean;
  retention_days: number;
};

type ViewMode = "history" | "settings";

type SettingsTone = "info" | "success" | "error";

const entryList = document.querySelector<HTMLUListElement>("#entry-list")!;
const statusMessage = document.querySelector<HTMLSpanElement>("#status-message")!;
const statusBar = document.querySelector<HTMLElement>(".status-bar")!;
const historyView = document.querySelector<HTMLElement>("#history-view")!;
const settingsView = document.querySelector<HTMLElement>("#settings-view")!;
const viewToggleButton = document.querySelector<HTMLButtonElement>("#view-toggle-button")!;
const settingsForm = document.querySelector<HTMLFormElement>("#settings-form")!;
const hotkeyInput = document.querySelector<HTMLInputElement>("#hotkey-input")!;
const recordHotkeyButton = document.querySelector<HTMLButtonElement>("#record-hotkey-button")!;
const applyHotkeyButton = document.querySelector<HTMLButtonElement>("#apply-hotkey-button")!;
const autostartToggle = document.querySelector<HTMLInputElement>("#autostart-toggle")!;
const retentionDefaultDays = document.querySelector<HTMLSpanElement>("#retention-default-days")!;
const settingsStatus = document.querySelector<HTMLParagraphElement>("#settings-status")!;

let entries: ClipboardEntry[] = [];
let filteredEntries: ClipboardEntry[] = [];
let selectedIndex = 0;
let currentView: ViewMode = "history";
let isRecordingHotkey = false;
let isSavingHotkey = false;
let isSavingAutostart = false;
let settingsLoaded = false;
let lastSavedHotkey = "";
const recordingModifiers = new Set<string>();
let pendingRecordedKey: string | null = null;
let pendingPlainBindingTimer: number | null = null;
const ENTRY_PREVIEW_LIMIT = 256;

const formatEntryPreview = (content: string) => {
  if (content.length <= ENTRY_PREVIEW_LIMIT) {
    return content;
  }

  return `${content.slice(0, ENTRY_PREVIEW_LIMIT)}...`;
};

const buildImagePreviewSources = (imagePath: string): string[] => {
  const trimmed = imagePath.trim();
  if (!trimmed) {
    return [];
  }

  const directSources = new Set<string>();
  if (trimmed.startsWith("asset:") || trimmed.startsWith("http://asset.localhost/")) {
    directSources.add(trimmed);
  }

  let normalizedPath = trimmed;
  if (trimmed.startsWith("file://")) {
    try {
      normalizedPath = decodeURIComponent(new URL(trimmed).pathname);
    } catch (_error) {
      normalizedPath = trimmed.slice("file://".length);
    }
  }

  normalizedPath = normalizedPath.replace(/\\/g, "/");

  const convertedSources = new Set<string>();
  const pushConverted = (candidate: string) => {
    if (!candidate) {
      return;
    }

    try {
      convertedSources.add(convertFileSrc(candidate));
    } catch (error) {
      console.warn("Failed to convert image preview path", { candidate, error });
    }
  };

  pushConverted(normalizedPath);

  if (/^[A-Za-z]:\//.test(normalizedPath)) {
    pushConverted(`/${normalizedPath}`);
  } else if (/^\/[A-Za-z]:\//.test(normalizedPath)) {
    pushConverted(normalizedPath.slice(1));
  }

  return [...directSources, ...convertedSources];
};

const setSettingsStatus = (message: string, tone: SettingsTone = "info") => {
  settingsStatus.textContent = message;
  settingsStatus.dataset.tone = tone;
};

const clearPendingRecordedKey = () => {
  pendingRecordedKey = null;
  if (pendingPlainBindingTimer !== null) {
    window.clearTimeout(pendingPlainBindingTimer);
    pendingPlainBindingTimer = null;
  }
};

const updateControlInteractivity = () => {
  const controlsDisabled = isSavingHotkey || isSavingAutostart;
  hotkeyInput.disabled = controlsDisabled;
  applyHotkeyButton.disabled = controlsDisabled || hotkeyInput.value.trim().length === 0;
  autostartToggle.disabled = controlsDisabled;
  recordHotkeyButton.disabled = isSavingHotkey || isSavingAutostart;
};

const setRecordingState = (recording: boolean) => {
  isRecordingHotkey = recording;
  if (!recording) {
    recordingModifiers.clear();
    clearPendingRecordedKey();
  }
  recordHotkeyButton.classList.toggle("is-recording", recording);
  recordHotkeyButton.textContent = recording ? "Press keys..." : "Record";
  void invoke("set_recording_hotkey", { recording }).catch((error) => {
    setSettingsStatus(`Recording mode update failed: ${String(error)}`, "error");
  });
  if (recording) {
    setSettingsStatus("Recording shortcut. Press key combination or Escape to cancel.");
  }
};

const joinBinding = (key: string, modifiers: Set<string>) => {
  const ordered: string[] = [];
  for (const modifier of ["Ctrl", "Shift", "Alt", "Super"]) {
    if (modifiers.has(modifier)) {
      ordered.push(modifier);
    }
  }
  ordered.push(key);
  return ordered.join("+");
};

const setView = (view: ViewMode) => {
  currentView = view;
  const inHistory = view === "history";

  historyView.hidden = !inHistory;
  settingsView.hidden = inHistory;
  statusBar.hidden = !inHistory;

  viewToggleButton.textContent = inHistory ? "⚙" : "←";
  viewToggleButton.setAttribute("aria-label", inHistory ? "Open settings" : "Back to history");
  viewToggleButton.title = inHistory ? "Open settings" : "Back to history";

  if (inHistory) {
    setRecordingState(false);
    entryList.focus();
  }
};

const loadSettings = async () => {
  setSettingsStatus("Loading settings...");
  const settings = await invoke<AppSettings>("load_settings");
  hotkeyInput.value = settings.hotkey_binding;
  autostartToggle.checked = settings.autostart_enabled;
  retentionDefaultDays.textContent = String(settings.retention_days);
  lastSavedHotkey = settings.hotkey_binding;
  settingsLoaded = true;
  setSettingsStatus("Settings loaded.");
  updateControlInteractivity();
};

const normalizeSingleKey = (key: string): string | null => {
  if (key.length === 1 && /^[a-z0-9]$/i.test(key)) {
    return key.toUpperCase();
  }

  if (/^F\d{1,2}$/i.test(key)) {
    return key.toUpperCase();
  }

  switch (key) {
    case " ":
      return "Space";
    case "Enter":
      return "Enter";
    case "Escape":
      return "Escape";
    case ".":
      return "Period";
    case ",":
      return "Comma";
    case "-":
      return "Minus";
    default:
      return null;
  }
};

const normalizeKeyFromCode = (code: string): string | null => {
  const letterMatch = /^Key([A-Z])$/.exec(code);
  if (letterMatch) {
    return letterMatch[1];
  }

  const digitMatch = /^Digit([0-9])$/.exec(code);
  if (digitMatch) {
    return digitMatch[1];
  }

  const functionMatch = /^F([0-9]{1,2})$/.exec(code);
  if (functionMatch) {
    return `F${functionMatch[1]}`;
  }

  switch (code) {
    case "Space":
      return "Space";
    case "Enter":
      return "Enter";
    case "Escape":
      return "Escape";
    case "Period":
      return "Period";
    case "Comma":
      return "Comma";
    case "Minus":
      return "Minus";
    default:
      return null;
  }
};

const isModifierKey = (key: string, code?: string) => {
  return (
    key === "Shift" ||
    key === "Control" ||
    key === "Alt" ||
    key === "Meta" ||
    key === "Super" ||
    key === "OS" ||
    key === "Super_L" ||
    key === "Super_R" ||
    code === "MetaLeft" ||
    code === "MetaRight"
  );
};

const modifierTokenFromKey = (key: string, code?: string): string | null => {
  switch (key) {
    case "Control":
      return "Ctrl";
    case "Shift":
      return "Shift";
    case "Alt":
      return "Alt";
    case "Meta":
    case "Super":
    case "OS":
    case "Super_L":
    case "Super_R":
      return "Super";
    default:
      if (code === "MetaLeft" || code === "MetaRight") {
        return "Super";
      }
      return null;
  }
};

const bindingFromKeyboardEvent = (event: KeyboardEvent): string | null => {
  const parts: string[] = [];
  const modifiers = new Set<string>(recordingModifiers);

  if (event.ctrlKey) {
    modifiers.add("Ctrl");
  }
  if (event.shiftKey) {
    modifiers.add("Shift");
  }
  if (event.altKey) {
    modifiers.add("Alt");
  }
  if (
    event.metaKey ||
    event.getModifierState("Meta") ||
    event.getModifierState("Super") ||
    event.getModifierState("OS")
  ) {
    modifiers.add("Super");
  }

  if (isModifierKey(event.key, event.code)) {
    const token = modifierTokenFromKey(event.key, event.code);
    if (token) {
      modifiers.add(token);
    }
    return null;
  }

  const key = normalizeSingleKey(event.key) ?? normalizeKeyFromCode(event.code);
  if (!key) {
    return null;
  }

  for (const modifier of ["Ctrl", "Shift", "Alt", "Super"]) {
    if (modifiers.has(modifier)) {
      parts.push(modifier);
    }
  }

  parts.push(key);
  return parts.join("+");
};

const applyHotkeyBinding = async (binding: string) => {
  if (isSavingHotkey) {
    return;
  }

  const candidate = binding.trim();
  if (!candidate) {
    hotkeyInput.value = lastSavedHotkey;
    setSettingsStatus("Hotkey binding cannot be empty.", "error");
    updateControlInteractivity();
    return;
  }

  isSavingHotkey = true;
  const previous = lastSavedHotkey;
  hotkeyInput.value = candidate;
  setSettingsStatus("Saving hotkey...");
  updateControlInteractivity();

  try {
    const updated = await invoke<AppSettings>("set_hotkey", { binding: candidate });
    lastSavedHotkey = updated.hotkey_binding;
    hotkeyInput.value = updated.hotkey_binding;
    setSettingsStatus("Hotkey saved.", "success");
  } catch (error) {
    hotkeyInput.value = previous;
    setSettingsStatus(`Hotkey update failed: ${String(error)}`, "error");
  } finally {
    isSavingHotkey = false;
    updateControlInteractivity();
  }
};

const finishRecordingWithBinding = async (binding: string) => {
  clearPendingRecordedKey();
  setRecordingState(false);
  await applyHotkeyBinding(binding);
};

const hasAnyModifierState = (event: KeyboardEvent) => {
  return (
    event.ctrlKey ||
    event.shiftKey ||
    event.altKey ||
    event.metaKey ||
    event.getModifierState("Meta") ||
    event.getModifierState("Super") ||
    event.getModifierState("OS")
  );
};

const renderEntries = () => {
  entryList.innerHTML = "";

  if (filteredEntries.length === 0) {
    const emptyItem = document.createElement("li");
    emptyItem.className = "entry-card empty-state";
    emptyItem.textContent = "Clipboard history is empty.";
    entryList.append(emptyItem);
    return;
  }

  filteredEntries.forEach((entry, index) => {
    const item = document.createElement("li");
    item.className = `entry-card${index === selectedIndex ? " is-selected" : ""}`;
    item.dataset.entryId = String(entry.id);

    const meta = document.createElement("div");
    meta.className = "entry-meta";
    meta.innerHTML = `<span>${entry.pinned ? "Pinned" : "Recent"}</span><span>${new Date(entry.captured_at).toLocaleString()}</span>`;

    let content: HTMLElement;
    if (entry.content_kind === "image") {
      const imageFrame = document.createElement("div");
      imageFrame.className = "entry-image-frame";

      if (entry.image_path) {
        const image = document.createElement("img");
        image.className = "entry-image";
        image.alt = "Clipboard image preview";

        const previewSources = buildImagePreviewSources(entry.image_path);
        let sourceIndex = 0;

        const showFallback = () => {
          imageFrame.innerHTML = "";
          const fallback = document.createElement("span");
          fallback.className = "entry-image-missing";
          fallback.textContent = "Image unavailable";
          imageFrame.append(fallback);
        };

        image.addEventListener("error", () => {
          sourceIndex += 1;
          if (sourceIndex < previewSources.length) {
            image.src = previewSources[sourceIndex];
            return;
          }

          console.warn("Image preview failed to load", {
            imagePath: entry.image_path,
            attemptedSources: previewSources,
            currentSrc: image.currentSrc,
          });
          showFallback();
        });

        if (previewSources.length === 0) {
          showFallback();
        } else {
          image.src = previewSources[sourceIndex];
          imageFrame.append(image);
        }
      } else {
        const fallback = document.createElement("span");
        fallback.className = "entry-image-missing";
        fallback.textContent = "Image unavailable";
        imageFrame.append(fallback);
      }

      content = imageFrame;
    } else {
      const textContent = document.createElement("p");
      textContent.className = "entry-content";
      textContent.textContent = formatEntryPreview(entry.content);
      content = textContent;
    }

    item.append(meta, content);
    item.addEventListener("click", async () => {
      selectedIndex = index;
      renderEntries();
      await selectCurrentEntry();
    });

    entryList.append(item);
  });

  const selected = entryList.querySelector<HTMLElement>(".is-selected");
  selected?.scrollIntoView({ block: "nearest" });
};

const refreshEntries = async () => {
  statusMessage.textContent = "Loading history…";
  entries = await invoke<ClipboardEntry[]>("list_entries", { query: null, limit: 100 });
  filteredEntries = [...entries];
  selectedIndex = 0;
  renderEntries();
  statusMessage.textContent = `${filteredEntries.length} item(s)`;
};

const selectCurrentEntry = async () => {
  const current = filteredEntries[selectedIndex];
  if (!current) {
    return;
  }

  await invoke("select_entry", { entryId: current.id });
  statusMessage.textContent = "Copied to clipboard.";
};

document.addEventListener("keydown", async (event) => {
  if (currentView === "settings" && isRecordingHotkey) {
    if (event.repeat) {
      event.preventDefault();
      return;
    }

    event.preventDefault();

    if (event.key === "Escape") {
      setRecordingState(false);
      setSettingsStatus("Hotkey recording canceled.");
      return;
    }

    if (isModifierKey(event.key, event.code)) {
      const token = modifierTokenFromKey(event.key, event.code);
      if (token) {
        recordingModifiers.add(token);
        if (pendingRecordedKey) {
          void finishRecordingWithBinding(joinBinding(pendingRecordedKey, recordingModifiers));
          return;
        }
        setSettingsStatus(`Modifier captured: ${token}. Press a key to complete.`);
      }
      return;
    }

    const key = normalizeSingleKey(event.key) ?? normalizeKeyFromCode(event.code);
    if (!key) {
      setSettingsStatus("Invalid shortcut. Use a key with optional modifiers.", "error");
      return;
    }

    const binding = bindingFromKeyboardEvent(event);
    if (!binding) {
      setSettingsStatus("Invalid shortcut. Use a key with optional modifiers.", "error");
      return;
    }

    if (!hasAnyModifierState(event) && recordingModifiers.size === 0) {
      clearPendingRecordedKey();
      pendingRecordedKey = key;
      setSettingsStatus(`Captured ${key}. Waiting briefly for modifier state...`);
      pendingPlainBindingTimer = window.setTimeout(() => {
        if (!pendingRecordedKey) {
          return;
        }
        void finishRecordingWithBinding(pendingRecordedKey);
      }, 200);
      return;
    }

    await finishRecordingWithBinding(binding);
    return;
  }

  if (currentView !== "history") {
    if (event.key === "Escape") {
      event.preventDefault();
      await invoke("hide_picker");
    }
    return;
  }

  if (event.key === "ArrowDown") {
    event.preventDefault();
    selectedIndex = Math.min(selectedIndex + 1, Math.max(filteredEntries.length - 1, 0));
    renderEntries();
  }

  if (event.key === "ArrowUp") {
    event.preventDefault();
    selectedIndex = Math.max(selectedIndex - 1, 0);
    renderEntries();
  }

  if (event.key === "Enter") {
    event.preventDefault();
    await selectCurrentEntry();
  }

  if (event.key === "Escape") {
    event.preventDefault();
    await invoke("hide_picker");
  }
});

document.addEventListener("keyup", (event) => {
  if (currentView !== "settings" || !isRecordingHotkey || !pendingRecordedKey) {
    return;
  }

  if (!isModifierKey(event.key, event.code)) {
    return;
  }

  const token = modifierTokenFromKey(event.key, event.code);
  if (!token) {
    return;
  }

  recordingModifiers.add(token);
  void finishRecordingWithBinding(joinBinding(pendingRecordedKey, recordingModifiers));
});

viewToggleButton.addEventListener("click", async () => {
  if (currentView === "history") {
    setView("settings");
    settingsLoaded = false;
    try {
      await loadSettings();
      hotkeyInput.focus();
    } catch (error) {
      setSettingsStatus(`Failed to load settings: ${String(error)}`, "error");
    }
    return;
  }

  setView("history");
});

hotkeyInput.addEventListener("input", () => {
  updateControlInteractivity();
});

settingsForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  await applyHotkeyBinding(hotkeyInput.value);
});

recordHotkeyButton.addEventListener("click", () => {
  if (!settingsLoaded || isSavingHotkey) {
    return;
  }

  setRecordingState(!isRecordingHotkey);
});

autostartToggle.addEventListener("change", async () => {
  if (!settingsLoaded || isSavingAutostart || isSavingHotkey) {
    return;
  }

  const previous = !autostartToggle.checked;
  const enabled = autostartToggle.checked;
  isSavingAutostart = true;
  setSettingsStatus("Saving autostart...");
  updateControlInteractivity();

  try {
    const updated = await invoke<AppSettings>("set_autostart", { enabled });
    autostartToggle.checked = updated.autostart_enabled;
    setSettingsStatus("Autostart saved.", "success");
  } catch (error) {
    autostartToggle.checked = previous;
    setSettingsStatus(`Autostart update failed: ${String(error)}`, "error");
  } finally {
    isSavingAutostart = false;
    updateControlInteractivity();
  }
});

listen("history-updated", async () => {
  await refreshEntries();
});

listen("picker-opened", async () => {
  setView("history");
  await refreshEntries();
  settingsLoaded = false;
  hotkeyInput.value = "";
  autostartToggle.checked = false;
  setSettingsStatus("");
  updateControlInteractivity();
  entryList.focus();
});

void refreshEntries().catch((error) => {
  statusMessage.textContent = String(error);
});

setView("history");
updateControlInteractivity();
