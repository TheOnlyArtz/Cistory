## Why

The picker currently exposes clipboard history only, forcing users to edit configuration outside the main interaction flow. Adding an in-window settings surface now enables fast self-service configuration for the two most critical behaviors: global hotkey and autostart.

## What Changes

- Add a minimal Settings view accessible from a top-right gear button in the picker window.
- Add view navigation between History and Settings inside the same picker window.
- Add hotkey configuration UI with manual entry and keyboard recording flow.
- Add autostart enable/disable UI backed by existing backend command support.
- Add inline success and error feedback for settings operations without opening modal dialogs.

## Capabilities

### New Capabilities
- `picker-settings`: In-window settings navigation and controls for hotkey binding and autostart.

### Modified Capabilities
- None.

## Impact

- Affected frontend files include picker markup, styles, and interaction logic (`index.html`, `src/main.ts`, `src/style.css`).
- Uses existing backend commands (`load_settings`, `set_hotkey`, `set_autostart`) with no new API surface required.
- No new runtime dependencies are required.
