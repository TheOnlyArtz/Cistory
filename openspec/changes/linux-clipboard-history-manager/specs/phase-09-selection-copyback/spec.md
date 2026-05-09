## ADDED Requirements

### Requirement: Phase 09 restores selected history to clipboard
Selecting an item in the picker SHALL copy the selected payload back to the system clipboard and finalize interaction predictably.

#### Scenario: Selection updates clipboard content
- **WHEN** a user confirms a history item
- **THEN** the selected content MUST become the active clipboard value

#### Scenario: Picker closes after successful selection
- **WHEN** copyback completes
- **THEN** the picker MUST close or hide according to interaction policy
