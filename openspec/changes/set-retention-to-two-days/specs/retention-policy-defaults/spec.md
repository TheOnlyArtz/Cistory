## ADDED Requirements

### Requirement: Default retention period is two days
The system SHALL use a default clipboard-history retention period of 2 days (48 hours) when the user has not configured a custom retention value.

#### Scenario: Default retention shown in settings
- **WHEN** a user opens retention settings without any saved custom retention preference
- **THEN** the displayed retention default is 2 days

#### Scenario: Cleanup uses default retention without override
- **WHEN** cleanup evaluates clipboard entries for a user with no custom retention setting
- **THEN** entries older than 48 hours are eligible for pruning

### Requirement: Custom retention overrides default
The system MUST apply a user-configured retention value instead of the default two-day retention period whenever a valid custom value exists.

#### Scenario: Cleanup honors custom retention
- **WHEN** cleanup runs for a user who configured a custom retention period
- **THEN** pruning eligibility is calculated using the custom retention value, not the two-day default

#### Scenario: UI reflects custom value
- **WHEN** a user with a saved custom retention value opens retention settings
- **THEN** the saved custom value is shown as active rather than the default
