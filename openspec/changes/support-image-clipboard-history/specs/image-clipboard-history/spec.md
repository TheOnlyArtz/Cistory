## ADDED Requirements

### Requirement: Image clipboard entries are persisted as temp-file references
The system SHALL persist image clipboard history by writing image binary payloads to an application-owned temporary directory and storing their filesystem paths in the history database.

#### Scenario: Image clipboard capture writes temp file and path
- **WHEN** a supported image payload is copied to the clipboard
- **THEN** the system MUST write an image file under the configured temporary directory and store an entry that references the written file path

#### Scenario: Missing image file degrades gracefully
- **WHEN** an image history entry references a file path that no longer exists
- **THEN** the picker MUST render a non-crashing fallback preview state and keep the entry selectable

### Requirement: Image previews are constrained to fixed-size thumbnail frames
The picker SHALL render image history entries using fixed-size preview frames so image dimensions do not alter card sizing or break list layout.

#### Scenario: Large image does not change list card dimensions
- **WHEN** a copied image exceeds thumbnail frame dimensions
- **THEN** the preview MUST be rendered inside the fixed frame with deterministic sizing behavior

#### Scenario: Mixed text and image entries remain layout-stable
- **WHEN** text and image entries are shown in the same history list
- **THEN** row alignment and keyboard navigation targets MUST remain visually stable across entry types
