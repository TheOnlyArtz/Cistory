## Why

The clipboard history currently supports text only, which drops copied images and breaks expected parity with modern clipboard workflows. Adding image support now closes a major usability gap while preserving the existing local-first architecture.

## What Changes

- Add clipboard ingestion support for image payloads in addition to text payloads.
- Persist image clipboard entries by writing image files into an application-owned temporary directory and storing file paths in SQLite.
- Extend domain and storage contracts so entries can represent image content consistently across ingestion, persistence, and retrieval.
- Update picker previews to render image thumbnails for image entries using fixed-size preview containers that preserve layout stability.
- Keep selection/copyback behavior predictable for image entries, including graceful handling for missing temp files.

## Capabilities

### New Capabilities
- `image-clipboard-history`: Capture, persist, and present image clipboard entries using temp-file storage with fixed-size picker previews.

### Modified Capabilities
- `phase-04-clipboard-ingestion`: Extend normalized ingestion behavior to include image clipboard content and image-specific safeguards.
- `phase-03-sqlite-storage`: Extend schema and mapping behavior to persist image entry paths and associated metadata needed for retrieval.
- `phase-08-picker-experience`: Extend picker visual behavior to render fixed-size image previews without breaking list layout.

## Impact

- Affected backend crates: `src-tauri/crates/domain`, `src-tauri/crates/clipboard`, `src-tauri/crates/storage`, and runtime wiring in `src-tauri/src/lib.rs`.
- Affected frontend surface: `src/main.ts` and `src/style.css` for entry rendering and thumbnail layout constraints.
- SQLite migration required for new image-path persistence fields and image-aware entry mapping.
- Filesystem usage increases through temp image writes and cleanup logic under the local temporary directory.
