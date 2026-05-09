## Context

The picker window is configured in two places today: `src-tauri/tauri.conf.json` defines the default shell window properties, and `src-tauri/src/lib.rs` creates the picker window and immediately moves it to a fixed top-right coordinate with `set_position(1480.0, 32.0)`. That leaves the popup resizable, taller than requested, and tied to a hard-coded position instead of the user-visible requirement.

## Goals / Non-Goals

**Goals:**
- Make the picker open in the bottom-right corner every time it is shown.
- Prevent manual resizing of the picker window.
- Cap the picker height at `400px` without changing the tray, hotkey, or clipboard flows.
- Keep the implementation small and centered on the existing picker window creation/show path.

**Non-Goals:**
- Redesign the picker UI contents or scrolling behavior.
- Add multi-window management beyond the existing single picker window.
- Change unrelated window properties such as transparency, decorations, or always-on-top behavior.

## Decisions

### Keep static window constraints in Tauri config
The baseline size contract should remain in `src-tauri/tauri.conf.json` because that is already the source of truth for default window properties. The window definition will be updated to disable resizing and to use a height no greater than `400`.

Alternative considered: leaving all sizing rules in Rust only. This was rejected because it duplicates configuration that already exists in Tauri config and makes default window behavior harder to inspect.

### Reposition the picker at runtime using monitor-aware logic
The hard-coded top-right coordinate in `ensure_picker_window` should be replaced with runtime positioning logic that computes the bottom-right corner from the relevant monitor's visible bounds and the picker's current size. This keeps the popup aligned even when screen size or scale differs from the developer's machine.

Alternative considered: replacing the hard-coded coordinates with a different fixed bottom-right constant. This was rejected because it would still fail on different monitor sizes and display arrangements.

### Reapply positioning when the picker is shown
The picker should be positioned immediately before or during `show_picker`, not only at initial creation time, so repeated opens keep the window anchored correctly after display changes or prior window movement.

Alternative considered: positioning only once during startup. This was rejected because the user can change monitor layout between opens and the app already keeps the picker window alive in the background.

## Risks / Trade-offs

- [Monitor APIs may return no active monitor in edge cases] → Fall back to the primary or available monitor and keep the existing show flow intact.
- [Bottom-right placement can vary with window-manager decorations or scale] → Use the window's actual runtime size and visible monitor bounds rather than hard-coded offsets.
- [Reducing height to `400px` exposes UI overflow] → Rely on the existing picker content scrolling behavior and verify the visible layout manually after the change.
