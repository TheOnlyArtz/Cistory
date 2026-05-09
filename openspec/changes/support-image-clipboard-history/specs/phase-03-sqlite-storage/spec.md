## MODIFIED Requirements

### Requirement: Phase 03 establishes durable SQLite persistence
The system SHALL define a versioned SQLite schema with migrations, indexing strategy, and transactional guarantees for clipboard history storage, including persistence for image entry file paths and related retrieval metadata.

#### Scenario: Migration behavior is controlled
- **WHEN** schema versions change
- **THEN** forward migration steps MUST be defined and validated for repeatable execution

#### Scenario: Persistence is resilient to restarts
- **WHEN** the application restarts after writes
- **THEN** committed clipboard entries MUST remain queryable without data corruption

#### Scenario: Image entry paths persist durably
- **WHEN** an image clipboard entry is committed
- **THEN** its stored file path metadata MUST remain queryable across application restarts
