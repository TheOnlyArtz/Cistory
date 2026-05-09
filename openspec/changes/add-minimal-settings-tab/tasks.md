## 1. Picker Layout and Navigation

- [x] 1.1 Add a header area with a top-right gear control in `index.html` and corresponding minimal styles in `src/style.css`.
- [x] 1.2 Implement frontend view state to switch between History and Settings in `src/main.ts` without creating a new window.
- [x] 1.3 Add a settings back control that returns from Settings to History while preserving current picker session.

## 2. Settings Data Loading and Rendering

- [x] 2.1 Add typed frontend settings model and load current values via `load_settings` when entering Settings.
- [x] 2.2 Render hotkey binding input and autostart toggle controls populated from loaded settings values.
- [x] 2.3 Add inline status messaging for loading, save success, and error states in the settings view.

## 3. Hotkey Configuration Flow

- [x] 3.1 Implement manual hotkey update submission wired to `set_hotkey` with optimistic UI guards.
- [x] 3.2 Implement explicit Record mode that captures the next valid key combination and exits recording on completion.
- [x] 3.3 Ensure invalid hotkey submissions/recordings surface inline errors and retain the last persisted valid binding.

## 4. Autostart Toggle Flow

- [x] 4.1 Wire the autostart toggle to `set_autostart` for immediate persistence.
- [x] 4.2 Reconcile toggle state with command responses and roll back UI state on command failure.
- [x] 4.3 Verify autostart state remains consistent after reopening settings/picker.

## 5. Interaction and Regression Verification

- [x] 5.1 Ensure picker keyboard shortcuts do not interfere with hotkey recording mode, including Escape cancel behavior.
- [x] 5.2 Verify history behaviors (selection, entry activation, hide picker) remain unchanged when not in settings mode.
- [ ] 5.3 Run project checks/tests and perform manual smoke validation for settings navigation, hotkey update, and autostart toggle.
