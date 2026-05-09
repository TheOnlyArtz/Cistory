## ADDED Requirements

### Requirement: Picker shall provide in-window settings navigation
The picker SHALL expose a top-right gear icon control that switches the current picker content from History to Settings within the same window session.

#### Scenario: Open settings from history
- **WHEN** the user activates the gear icon while viewing clipboard history
- **THEN** the picker displays the Settings view in the same window

#### Scenario: Return to history from settings
- **WHEN** the user activates the settings back control
- **THEN** the picker displays the History view without opening a new window

### Requirement: Picker shall load and display current settings
The settings view SHALL load and present the current persisted hotkey binding and autostart state when the user enters settings.

#### Scenario: Populate settings values
- **WHEN** the user navigates to the Settings view
- **THEN** the hotkey input and autostart toggle reflect the latest values returned by settings storage

### Requirement: Picker shall support hotkey update by entry and recording
The settings view SHALL allow the user to update the global hotkey binding either by entering a binding value or by recording a keyboard combination, and SHALL persist valid changes through the existing settings command path.

#### Scenario: Save valid typed binding
- **WHEN** the user submits a valid hotkey binding string in Settings
- **THEN** the picker persists the binding through the hotkey settings command and shows inline success feedback

#### Scenario: Save recorded binding
- **WHEN** the user enters recording mode and presses a valid key combination
- **THEN** the picker persists the captured binding through the hotkey settings command and exits recording mode

#### Scenario: Handle invalid binding
- **WHEN** the user submits or records an invalid hotkey binding
- **THEN** the picker shows inline error feedback and keeps the previously saved valid binding active

### Requirement: Picker shall support autostart toggle
The settings view SHALL allow enabling and disabling autostart using the existing autostart settings command and SHALL keep UI state consistent with persisted result.

#### Scenario: Enable autostart
- **WHEN** the user turns autostart on in Settings
- **THEN** the picker persists the enabled state through the autostart settings command and shows inline success feedback

#### Scenario: Disable autostart
- **WHEN** the user turns autostart off in Settings
- **THEN** the picker persists the disabled state through the autostart settings command and shows inline success feedback

#### Scenario: Autostart command failure
- **WHEN** autostart persistence fails while toggling
- **THEN** the picker restores the previous toggle state and shows inline error feedback
