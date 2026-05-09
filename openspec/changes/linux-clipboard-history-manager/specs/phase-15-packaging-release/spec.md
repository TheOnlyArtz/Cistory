## ADDED Requirements

### Requirement: Phase 15 defines packaging and upgrade readiness
Release artifacts SHALL include installable packages, upgrade-safe migration behavior, and rollback guidance for supported Linux targets.

#### Scenario: Fresh install is validated
- **WHEN** a new user installs a release package
- **THEN** the application MUST launch and provide background-ready baseline behavior

#### Scenario: Upgrade path is validated
- **WHEN** an existing user upgrades from a prior supported version
- **THEN** settings and history persistence MUST remain intact under migration rules
