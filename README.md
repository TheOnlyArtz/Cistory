# Cistory

Cistory is a Linux-first clipboard history manager built with Tauri 2, Rust, and a lightweight TypeScript UI. It runs in the background, captures clipboard updates, stores them locally in SQLite, and gives you a fast keyboard-driven picker inspired by `Win+V`.

## Disclaimer

This project is 100% vibe-coded in a couple of hours. It is functional and useful, but still early-stage software: expect rough edges, validate behavior on your desktop setup, and use with appropriate caution around sensitive clipboard content.

## What It Does

- Runs as a single-instance background app with a tray icon.
- Polls clipboard content and stores history locally.
- Supports text entries and image entries.
- Opens a picker window with a global hotkey.
- Lets you choose entries to copy them back to the clipboard.
- Includes settings for hotkey recording and autostart.

## Tech Stack

- Desktop shell: Tauri 2
- Backend/runtime: Rust
- Frontend UI: TypeScript + Vite
- Persistence: SQLite (`rusqlite`)
- Clipboard integration: `arboard`
- Global hotkeys: `global-hotkey`

## Repository Structure

- `src/` - picker UI (TypeScript/CSS)
- `src-tauri/src/` - Tauri shell, commands, tray, window lifecycle
- `src-tauri/crates/domain/` - core models and validation rules
- `src-tauri/crates/storage/` - SQLite schema and persistence logic
- `src-tauri/crates/clipboard/` - clipboard snapshot and conversion logic
- `src-tauri/crates/hotkey/` - hotkey parsing/planning logic
- `docs/` - governance, security notes, QA/release checklists
- `openspec/` - change/spec management artifacts

## Prerequisites

### System Packages (Ubuntu/Debian)

```bash
sudo apt-get install -y \
  pkg-config \
  libdbus-1-dev \
  libglib2.0-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev
```

### Tooling

- Node.js 20+
- npm 10+
- Rust toolchain (stable)

## Getting Started

```bash
npm install
npm run tauri:dev
```

The app launches in development mode and runs as a desktop app with the picker window hidden by default until triggered.

## Scripts

- `npm run dev` - run Vite frontend on `127.0.0.1:1420`
- `npm run build` - compile TypeScript and bundle frontend assets
- `npm run tauri:dev` - run the full desktop app in dev mode
- `npm run tauri:build` - create production desktop bundles

## Usage Notes

- Use tray left-click (or configured hotkey) to open the picker.
- Arrow keys navigate history; `Enter` copies selected entry.
- `Esc` hides the picker.
- In settings, use the Record button to capture a new hotkey.

## Local-First and Security

- Clipboard data is stored only on the local machine.
- There is no telemetry and no cloud sync.
- On Unix systems, the app attempts to set the database file to `0600` permissions.
- Sensitive-content filtering is not yet implemented.

## Scope and Limits (Current)

- Target desktop environments: GNOME and KDE on X11/Wayland.
- Wayland global shortcut behavior may depend on portal support and compositor behavior.
- Not all Linux compositors are considered first-class yet.
- Encryption-at-rest is not included in this version.

## Verification

Recommended checks before release:

```bash
npm run build
cd src-tauri && cargo check --workspace && cargo test --workspace
```

Manual validation checklist and acceptance evidence are under `docs/`.
