## Context

This change structures delivery of a Linux-first clipboard history manager using Rust and Tauri as a sequence of independently verifiable phases. The primary user promise is Windows-like `Win+V` behavior with low latency, background reliability, and strict local-first data handling.

The project is early-stage, so requirements must prevent architectural drift and premature feature expansion. Linux desktop variance (X11 vs Wayland, distro-specific startup behavior, window-manager differences) is a major source of implementation risk.

## Goals / Non-Goals

**Goals:**
- Define one clear specification per implementation phase so planning and execution stay auditable.
- Encode quality gates, reliability expectations, and safe-Rust error boundaries before coding.
- Make major-step git commits part of delivery discipline.
- Keep architecture modular: domain logic separate from transport/UI/infrastructure details.

**Non-Goals:**
- Implementing application code in this change.
- Designing advanced v2 features (cloud sync, collaboration, cross-device transport).
- Supporting every Linux desktop environment at equal depth in v1.

## Decisions

- Use phase-oriented capabilities (`phase-00` through `phase-15`) instead of one broad spec.
  - Rationale: improves traceability, review quality, and rollback clarity.
  - Alternative considered: single monolithic spec file; rejected due to poor maintainability.

- Each phase spec includes explicit scenarios that can become acceptance tests.
  - Rationale: keeps planning testable and objective.
  - Alternative considered: narrative-only phase docs; rejected because completion becomes subjective.

- Cross-cutting concerns (safe error handling, local-first, commit cadence, background runtime) are repeated where relevant across phase specs.
  - Rationale: prevents critical concerns from being treated as optional.
  - Alternative considered: one standalone cross-cutting spec; rejected because phase owners may miss it.

- v1 acceptance targets include GNOME and KDE across both X11 and Wayland sessions.
  - Rationale: broad practical Linux coverage for the intended desktop-user audience.
  - Alternative considered: limiting to one display protocol first; rejected because it would delay confidence in the core user promise.

- History policy uses unlimited entry count with a default retention window of one week.
  - Rationale: keeps UX unconstrained during active usage while bounding storage with time-based pruning.
  - Alternative considered: fixed maximum row count; rejected because it can evict useful recent context unpredictably.

- Encrypted-at-rest storage is deferred from v1.
  - Rationale: v1 assumes local machine trust and prioritizes reliability and UX parity.
  - Alternative considered: shipping encryption in v1; rejected to avoid key-management complexity during initial delivery.

## Risks / Trade-offs

- [Too many specs increase overhead] -> Use concise requirement language and keep each spec focused on phase acceptance criteria.
- [Linux platform variance causes rework] -> Front-load platform constraints and risk spikes in `phase-00` and validate again in `phase-14`.
- [Commit cadence may encourage small noisy commits] -> Define major-step commits tied to phase exits, not every micro-change.
- [Performance goals may conflict with maintainability] -> Treat latency SLOs as measurable budgets and document exceptions explicitly.

## Migration Plan

1. Approve this change as the planning baseline.
2. During implementation, execute phases in order and treat each phase spec as a DoD contract.
3. At each phase completion, create one major-step commit with evidence of gate completion.
4. After all phases are complete and validated, archive the change and sync specs into main `openspec/specs`.

## Open Questions

- None currently. Previous open questions have been resolved for v1 scope.
