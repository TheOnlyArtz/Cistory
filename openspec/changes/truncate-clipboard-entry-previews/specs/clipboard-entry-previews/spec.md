## ADDED Requirements

### Requirement: Picker entry previews are capped at 256 characters
The system SHALL render at most 256 characters of clipboard entry content in the picker list preview.

#### Scenario: Entry content is within the preview limit
- **WHEN** a clipboard entry contains 256 characters or fewer
- **THEN** the picker SHALL render the full entry content without truncation

#### Scenario: Entry content exceeds the preview limit
- **WHEN** a clipboard entry contains more than 256 characters
- **THEN** the picker SHALL render only the first 256 characters of the entry content

### Requirement: Truncated picker previews signal overflow with ellipsis
The system SHALL append an ellipsis to a picker entry preview when clipboard content is truncated for display.

#### Scenario: Truncated preview is shown
- **WHEN** a clipboard entry exceeds 256 characters and is shortened for the picker list
- **THEN** the rendered preview SHALL end with an ellipsis to indicate overflow

### Requirement: Preview formatting does not alter entry behavior
The system SHALL preserve full clipboard entry content for non-preview behavior even when the picker shows a truncated preview.

#### Scenario: Search uses full content
- **WHEN** a user searches for text that appears after the first 256 characters of a stored clipboard entry
- **THEN** that entry SHALL remain discoverable in picker filtering results

#### Scenario: Selection uses full content
- **WHEN** a user selects a clipboard entry whose picker preview was truncated
- **THEN** the system SHALL restore the full stored clipboard content rather than the truncated preview text
