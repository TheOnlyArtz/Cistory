## 1. Domain and Contract Updates

- [x] 1.1 Extend clipboard domain content typing to represent image entries and any required image path metadata.
- [x] 1.2 Update command/serialization contracts between Tauri backend and UI for mixed text/image entry payloads.

## 2. Storage and Migration

- [x] 2.1 Add a SQLite migration that introduces image-path persistence fields required by image history entries.
- [x] 2.2 Update storage read/write mapping and deduplication logic for text and image entry variants.
- [x] 2.3 Add/adjust storage tests for migration safety and image-entry persistence across reopen.

## 3. Clipboard Ingestion and Temp File Handling

- [x] 3.1 Extend clipboard ingestion to detect supported image payloads and normalize them into history candidates.
- [x] 3.2 Implement temp-directory image file writing and path assignment for image entries.
- [x] 3.3 Ensure self-copy suppression and error handling continue to work with image copyback flows.

## 4. Picker Preview Experience

- [x] 4.1 Update picker rendering to show image thumbnails for image entries and text previews for text entries.
- [x] 4.2 Add fixed-size thumbnail styling so image previews cannot break card/list layout.
- [x] 4.3 Add fallback rendering for missing image files while preserving keyboard selection behavior.

## 5. Validation and Hardening

- [x] 5.1 Add integration coverage for mixed text/image history behavior (capture, list, select).
- [x] 5.2 Validate pruning/retention interactions with image temp files and path references.
- [x] 5.3 Run full project verification checks and document any image-support caveats.
