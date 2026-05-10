## Why

Clipboard history entries are currently retained for one week, which keeps data longer than needed for many users and increases local storage/privacy exposure. Reducing default retention to two days better matches short-term clipboard workflows while minimizing retained sensitive content.

## What Changes

- Change the default retention window from 7 days to 2 days.
- Ensure cleanup/pruning logic uses the updated default when no custom retention is configured.
- Update user-facing wording and configuration defaults that reference one-week retention.

## Capabilities

### New Capabilities
- `retention-policy-defaults`: Defines and exposes the default clipboard-history retention policy used when users have not customized settings.

### Modified Capabilities
- None.

## Impact

- Affects clipboard history retention configuration and cleanup behavior.
- May reduce persisted history size and improve privacy by default.
- Requires updates in settings/default constants and any retention-related copy or tests.
