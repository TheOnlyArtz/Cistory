## MODIFIED Requirements

### Requirement: Phase 08 defines keyboard-first picker UX
The picker SHALL provide low-latency, keyboard-first selection with searchable history and clear visual hierarchy, including fixed-size image previews for image entries.

#### Scenario: Keyboard navigation selects entries
- **WHEN** the picker is visible
- **THEN** users MUST be able to navigate results and confirm selection without using a mouse

#### Scenario: Large history remains responsive
- **WHEN** history size is high
- **THEN** filtering and navigation MUST remain within defined interaction latency targets

#### Scenario: Image previews preserve layout stability
- **WHEN** image entries are rendered in history previews
- **THEN** the picker MUST constrain previews to fixed-size frames so list layout remains stable
