## ADDED Requirements

### Requirement: Phase 02 defines domain contracts and invariants
Core clipboard entities, typed contracts, and business invariants SHALL be defined independent of storage and UI transport layers.

#### Scenario: Domain invariants are testable
- **WHEN** Phase 02 artifacts are produced
- **THEN** domain rules MUST be represented as deterministic tests

#### Scenario: Domain remains infrastructure-agnostic
- **WHEN** domain APIs are reviewed
- **THEN** domain modules MUST not depend directly on Tauri, SQL drivers, or UI frameworks
