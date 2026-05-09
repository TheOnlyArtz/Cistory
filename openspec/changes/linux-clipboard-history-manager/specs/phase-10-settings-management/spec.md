## ADDED Requirements

### Requirement: Phase 10 defines persistent settings management
The system SHALL provide validated settings for hotkey, retention behavior, startup preferences, and privacy-related controls with durable local persistence.

#### Scenario: Settings survive restart
- **WHEN** the user changes configuration
- **THEN** updated values MUST persist across application restarts

#### Scenario: Invalid settings are rejected safely
- **WHEN** unsupported configuration values are submitted
- **THEN** the system MUST reject invalid input with actionable feedback and preserve last known valid settings
