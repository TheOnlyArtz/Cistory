## 1. Update Retention Defaults

- [x] 1.1 Locate the shared retention default constant/config and change it from 7 days to 2 days (48 hours).
- [x] 1.2 Confirm default-resolution logic still prioritizes user-configured retention over the new default.

## 2. Align Cleanup and Settings Behavior

- [x] 2.1 Update cleanup/pruning code paths to ensure they consume the shared default when no user override exists.
- [x] 2.2 Update settings/UI default display text and labels to reflect a 2-day default retention period.

## 3. Validate and Guard Against Regressions

- [x] 3.1 Update or add automated tests for default 48-hour pruning and custom-retention override behavior.
- [x] 3.2 Run relevant test suites and fix any retention-related failures caused by legacy seven-day assumptions.
