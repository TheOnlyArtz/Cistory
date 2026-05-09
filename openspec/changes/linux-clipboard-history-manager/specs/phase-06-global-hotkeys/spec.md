## ADDED Requirements

### Requirement: Phase 06 provides global hotkey invocation
The application SHALL support a global shortcut with a default `Win+V` binding and allow validated user customization.

#### Scenario: Default hotkey opens picker
- **WHEN** the user presses the default binding
- **THEN** the picker window MUST open from background mode without requiring manual app focus

#### Scenario: Binding conflicts are managed
- **WHEN** a requested shortcut cannot be registered
- **THEN** the system MUST report the conflict and preserve a working fallback behavior
