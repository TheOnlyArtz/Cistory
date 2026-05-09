## ADDED Requirements

### Requirement: Phase 13 guarantees local-first privacy posture
Clipboard data SHALL remain local by default, and the application MUST avoid third-party communication unless explicitly introduced by a future opt-in capability.

#### Scenario: Network egress is absent by default
- **WHEN** the application is operating under v1 scope
- **THEN** no outbound third-party synchronization or telemetry communication MUST occur

#### Scenario: Local data protections are defined
- **WHEN** persistence paths are configured
- **THEN** storage and settings files MUST use least-privilege local access constraints

#### Scenario: Encrypted-at-rest is deferred in v1
- **WHEN** v1 security controls are finalized
- **THEN** encrypted-at-rest storage MUST NOT be required for release acceptance
