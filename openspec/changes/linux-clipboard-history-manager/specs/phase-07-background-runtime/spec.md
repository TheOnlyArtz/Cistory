## ADDED Requirements

### Requirement: Phase 07 defines always-running background lifecycle
The application SHALL run tray-first, remain active in background mode, and manage picker window visibility without terminating the process.

#### Scenario: Closing picker does not exit app
- **WHEN** the picker loses focus or receives close intent
- **THEN** the picker MUST hide while the background process remains active

#### Scenario: Window positioning is deterministic
- **WHEN** picker is shown
- **THEN** it MUST appear at a defined top-right anchored position respecting monitor and DPI context
