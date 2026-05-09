# Security Notes

## Local Data Handling

- Clipboard entries are stored locally in SQLite.
- No outbound network communication is implemented.
- Encrypted-at-rest storage is deferred from v1 by design.

## File Permissions

- On Unix systems, the app attempts to apply `0600` permissions to the SQLite database file.
- Users remain responsible for local machine trust and filesystem-level account security.

## Sensitive Content

- The current implementation stores copied text without app-based filtering.
- Sensitive-application exclusion remains a future enhancement and should be validated before broader release.
