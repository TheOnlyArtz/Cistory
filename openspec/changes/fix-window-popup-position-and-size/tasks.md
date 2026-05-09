## 1. Window Constraints

- [x] 1.1 Update `src-tauri/tauri.conf.json` so the picker window is non-resizable and its configured height does not exceed `400px`
- [x] 1.2 Remove or update any documentation that still describes the picker as opening in the top-right corner

## 2. Runtime Positioning

- [x] 2.1 Replace the hard-coded picker position in `src-tauri/src/lib.rs` with monitor-aware bottom-right placement logic
- [x] 2.2 Ensure the picker position is refreshed when the window is shown so repeated opens stay anchored in the bottom-right corner

## 3. Verification

- [x] 3.1 Verify the Rust/Tauri code compiles after the window behavior changes
- [ ] 3.2 Manually confirm the picker opens bottom-right, cannot be resized, and never exceeds `400px` in height
