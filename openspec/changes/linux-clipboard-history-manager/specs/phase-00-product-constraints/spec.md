## ADDED Requirements

### Requirement: Phase 00 defines measurable constraints
The project SHALL define v1 scope, explicit non-goals, performance and reliability SLOs, and Linux target assumptions before implementation begins.

#### Scenario: Scope baseline is approved
- **WHEN** the team enters Phase 00
- **THEN** a documented baseline MUST exist with in-scope features, out-of-scope features, and measurable success criteria

#### Scenario: Platform risk framing is explicit
- **WHEN** Linux environment support is planned
- **THEN** the phase output MUST identify supported desktop environments and known X11/Wayland risk areas

#### Scenario: v1 target matrix is fixed
- **WHEN** v1 scope is baselined
- **THEN** mandatory acceptance targets MUST include GNOME and KDE on both X11 and Wayland sessions
