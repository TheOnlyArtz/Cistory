## 1. Picker Preview Rendering

- [x] 1.1 Update the picker entry rendering path in `src/main.ts` to cap visible clipboard previews at 256 characters.
- [x] 1.2 Append an ellipsis only when entry content exceeds the 256-character preview limit.

## 2. Behavior Verification

- [x] 2.1 Verify entries at and below 256 characters render unchanged in the picker list.
- [x] 2.2 Verify entries above 256 characters show a truncated preview while search and selection still use full clipboard content.
