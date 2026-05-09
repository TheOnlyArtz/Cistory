## Why

Linux lacks a polished, always-available clipboard history experience comparable to Windows `Win+V`, creating daily friction for power users. A phased, spec-first plan is needed now so implementation quality, reliability, safety, and maintainability are built in from day one.

## What Changes

- Define a full implementation roadmap as individually testable capabilities, one spec per phase.
- Specify quality gates, error-handling expectations, and local-first guarantees across all phases.
- Standardize phase completion criteria so each step can be reviewed, validated, and committed as a major milestone.
- Capture background-runtime behavior, clipboard ingestion, hotkey UX, persistence, packaging, and release readiness as explicit requirements.

## Capabilities

### New Capabilities
- `phase-00-product-constraints`: Define scope, non-goals, SLOs, and platform risk framing.
- `phase-01-workspace-foundation`: Establish modular Rust/Tauri workspace boundaries.
- `phase-02-domain-contracts`: Define domain entities, invariants, and repository/service contracts.
- `phase-03-sqlite-storage`: Define SQLite schema, migration, and persistence guarantees.
- `phase-04-clipboard-ingestion`: Define clipboard watcher pipeline and normalization behavior.
- `phase-05-history-policy`: Define deduplication and retention policies for clipboard history.
- `phase-06-global-hotkeys`: Define global shortcut behavior and conflict handling.
- `phase-07-background-runtime`: Define tray/background lifecycle and picker-window orchestration.
- `phase-08-picker-experience`: Define keyboard-first picker UX and performance expectations.
- `phase-09-selection-copyback`: Define selection flow that restores chosen entry to clipboard.
- `phase-10-settings-management`: Define user settings persistence and live application of config.
- `phase-11-autostart-resilience`: Define startup, single-instance, and crash-recovery behavior.
- `phase-12-error-observability`: Define typed error handling and diagnostics boundaries.
- `phase-13-local-first-security`: Define privacy, no-network behavior, and local data protections.
- `phase-14-quality-gates`: Define test matrix, stress checks, and performance budgets.
- `phase-15-packaging-release`: Define packaging, upgrade safety, and release acceptance criteria.

### Modified Capabilities
- None.

## Impact

- Affects OpenSpec artifacts for planning and governance of the entire clipboard manager project.
- Establishes requirements that future Rust crates, Tauri shell, UI components, CI pipelines, and release workflows must satisfy.
- Introduces explicit expectation for major-step git commits tied to phase completion.
