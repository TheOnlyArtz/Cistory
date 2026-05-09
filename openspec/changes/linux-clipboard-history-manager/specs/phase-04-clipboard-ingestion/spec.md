## ADDED Requirements

### Requirement: Phase 04 defines clipboard ingestion pipeline
The system SHALL listen for clipboard changes in background mode, normalize supported content types, and safely enqueue entries for persistence.

#### Scenario: New clipboard content is captured
- **WHEN** the user copies supported content
- **THEN** a normalized history candidate MUST be generated for downstream persistence

#### Scenario: Ingestion avoids self-loop behavior
- **WHEN** copyback actions originate from the application itself
- **THEN** ingestion logic MUST prevent unbounded recursive history insertion
