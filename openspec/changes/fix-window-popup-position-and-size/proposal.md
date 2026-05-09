## Why

The picker window does not match the intended desktop behavior: it is currently created as a resizable `420x720` window and explicitly positioned near the top-right corner. Tightening the placement and sizing rules now prevents UI drift and gives implementation work a clear contract for consistent popup behavior.

## What Changes

- Define picker window behavior as a tracked capability in OpenSpec.
- Require the picker window to open in the bottom-right corner of the active screen instead of the top-right corner.
- Require the picker window to be non-resizable.
- Require the picker window height to stay at or below `400px` while preserving the existing picker-style popup behavior.

## Capabilities

### New Capabilities
- `picker-window-behavior`: Defines how the desktop picker window is sized, constrained, and positioned when the user opens it.

### Modified Capabilities

## Impact

- Affected code: `src-tauri/src/lib.rs`, `src-tauri/tauri.conf.json`
- Affected docs: `README.md` window-position description and any related validation notes
- Affected verification: desktop manual validation for picker open/show behavior across supported Linux sessions
