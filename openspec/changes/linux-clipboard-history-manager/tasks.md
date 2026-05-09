## 1. Phase 00 - Product Constraints

- [x] 1.1 Finalize scope, non-goals, and measurable SLO targets.
- [x] 1.2 Document platform risk assumptions and acceptance matrix for Linux environments.

## 2. Phase 01 - Workspace Foundation

- [x] 2.1 Define Rust workspace crate boundaries and Tauri shell responsibilities.
- [x] 2.2 Validate build and test bootstrap for all foundational modules.

## 3. Phase 02 - Domain Contracts

- [x] 3.1 Define clipboard domain entities, invariants, and typed contracts.
- [x] 3.2 Validate domain-level tests independent of storage and UI.

## 4. Phase 03 - SQLite Storage

- [x] 4.1 Define schema, migrations, indexes, and transaction guarantees.
- [x] 4.2 Validate migration safety and storage integrity under restart conditions.

## 5. Phase 04 - Clipboard Ingestion

- [x] 5.1 Define watcher behavior, normalization pipeline, and loop-prevention rules.
- [x] 5.2 Validate ingestion stability under rapid copy events.

## 6. Phase 05 - History Policy

- [x] 6.1 Define dedupe rules, retention limits, and pruning triggers.
- [x] 6.2 Validate deterministic behavior for duplicate and eviction scenarios.

## 7. Phase 06 - Global Hotkeys

- [x] 7.1 Define default and custom hotkey registration semantics.
- [x] 7.2 Validate collision handling and runtime rebinding behavior.

## 8. Phase 07 - Background Runtime

- [x] 8.1 Define tray-first lifecycle and hidden-window orchestration.
- [x] 8.2 Validate background persistence and window show/hide stability.

## 9. Phase 08 - Picker Experience

- [x] 9.1 Define keyboard-first picker interaction and filtering behavior.
- [ ] 9.2 Validate picker latency and large-history responsiveness targets.

## 10. Phase 09 - Selection Copyback

- [x] 10.1 Define selection-to-clipboard copyback flow and close behavior.
- [x] 10.2 Validate copyback correctness and recursion guard behavior.

## 11. Phase 10 - Settings Management

- [x] 11.1 Define settings schema, validation, and persistence boundaries.
- [x] 11.2 Validate live settings application and restart persistence.

## 12. Phase 11 - Autostart Resilience

- [x] 12.1 Define autostart and single-instance behavior by environment.
- [ ] 12.2 Validate crash recovery and reboot-start reliability.

## 13. Phase 12 - Error Observability

- [x] 13.1 Define typed error taxonomy and boundary-mapping behavior.
- [x] 13.2 Validate graceful degradation paths for partial subsystem failures.

## 14. Phase 13 - Local-first Security

- [x] 14.1 Define no-network guarantees and sensitive-content handling rules.
- [x] 14.2 Validate local data protection and permission constraints.

## 15. Phase 14 - Quality Gates

- [x] 15.1 Define cross-platform QA matrix and stress-test requirements.
- [ ] 15.2 Validate performance budgets and reliability SLO conformance.

## 16. Phase 15 - Packaging Release

- [x] 16.1 Define package artifacts, upgrade requirements, and rollback expectations.
- [ ] 16.2 Validate install, upgrade, and release acceptance checklist.

## 17. Governance

- [x] 17.1 Require one major-step git commit per phase completion with verification notes.
- [x] 17.2 Track phase acceptance evidence before moving to archive.
