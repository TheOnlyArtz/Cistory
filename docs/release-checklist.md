# Release Checklist

## Build

- Install Linux system dependencies.
- Run `npm run build`.
- Run `cargo check --workspace` from `src-tauri/`.
- Run `cargo test --workspace` from `src-tauri/`.

## Runtime Validation

- Validate picker lifecycle, tray interactions, clipboard ingestion, and copyback behavior.
- Validate autostart toggle and single-instance behavior.
- Validate hotkey behavior on X11 and confirm graceful fallback messaging on unsupported Wayland setups.

## Packaging Readiness

- Confirm Tauri bundle targets and icon assets are present.
- Confirm fresh install path and upgrade path preserve settings and history.
- Record any platform-specific caveats in release notes.
