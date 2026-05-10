## Context

The application currently defaults clipboard-history retention to one week. This default influences both how long entries are considered valid and when cleanup removes old data. The requested change narrows that default window to two days while preserving current behavior for users who explicitly set a custom retention value.

## Goals / Non-Goals

**Goals:**
- Update the system default retention from 7 days to 2 days in a single source of truth.
- Ensure pruning/cleanup logic consistently uses the updated default when no user override exists.
- Keep existing user-configured retention behavior unchanged.
- Align user-facing text and settings defaults with the new policy.

**Non-Goals:**
- Introducing new retention policy options or custom scheduling.
- Migrating or rewriting historical data beyond normal cleanup behavior.
- Changing storage backend or clipboard capture flow.

## Decisions

- Set the default retention value in the existing retention/config constant rather than adding a new configuration pathway.
  - Rationale: minimizes surface area and ensures all consumers inheriting the default update together.
  - Alternative considered: overriding retention only inside cleanup jobs. Rejected because UI defaults and backend behavior could diverge.
- Preserve precedence rules: explicit user retention setting overrides the default.
  - Rationale: avoids breaking user intent and keeps behavior backward compatible for customized installs.
  - Alternative considered: force-reseting all users to two days. Rejected as a breaking behavioral change.
- Update retention-related copy/tests in the same change.
  - Rationale: keeps docs/tests coherent with runtime behavior and prevents regressions.

## Risks / Trade-offs

- Reduced retention may remove entries users expected to remain longer by default -> Mitigation: ensure settings clearly indicate current retention value and preserve custom overrides.
- Missed references to the old seven-day assumption could cause inconsistent behavior -> Mitigation: update tests and scan retention-related constants/messages in the implementation phase.
- Time-based tests may become flaky if they assume broad windows -> Mitigation: use deterministic timestamps in tests and assert explicit 48-hour boundaries.
