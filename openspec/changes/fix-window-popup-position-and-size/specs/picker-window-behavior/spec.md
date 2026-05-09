## ADDED Requirements

### Requirement: Picker opens anchored to the bottom-right corner
The system SHALL position the picker window at the bottom-right corner of the relevant desktop monitor whenever the picker is opened.

#### Scenario: Hotkey opens picker from the bottom-right corner
- **WHEN** the user triggers the picker with the configured hotkey
- **THEN** the picker window opens aligned to the bottom-right edge of the monitor used for the picker

#### Scenario: Tray click reopens picker in the correct corner
- **WHEN** the user opens the picker from the tray after a previous open
- **THEN** the picker window is shown in the bottom-right corner instead of reusing a stale top-right placement

### Requirement: Picker window is not user-resizable
The picker window SHALL not allow manual resizing by the user.

#### Scenario: Window chrome does not offer resize behavior
- **WHEN** the picker window is visible
- **THEN** the desktop window manager treats the picker as non-resizable

### Requirement: Picker height is capped for compact popup behavior
The picker window SHALL use a height no greater than `400px` when shown.

#### Scenario: Picker opens within the height limit
- **WHEN** the picker window is created or reopened
- **THEN** its visible height does not exceed `400px`
