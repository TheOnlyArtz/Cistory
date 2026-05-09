## ADDED Requirements

### Requirement: Phase 05 defines deduplication and retention policy
The history store SHALL apply deterministic duplicate handling and time-based retention rules to preserve relevance and control growth.

#### Scenario: Duplicate entries are normalized consistently
- **WHEN** clipboard content repeats within policy thresholds
- **THEN** deduplication MUST follow defined hash and timing rules

#### Scenario: One-week retention is enforced
- **WHEN** an entry age exceeds seven days from creation
- **THEN** the entry MUST be eligible for pruning under retention policy

#### Scenario: v1 does not enforce entry-count caps
- **WHEN** retention policy is evaluated in v1
- **THEN** pruning MUST be driven by time-window rules rather than a fixed maximum entry count
