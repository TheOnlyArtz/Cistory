## Context

The current picker renders clipboard entry text directly into the list item content area in `src/main.ts`. Long clipboard entries can dominate the list visually even though the rest of the application still needs the full underlying entry content for search, persistence, and copy-back behavior.

This change is intentionally narrow: it only standardizes how entry text is presented in the picker list. The current architecture already separates the stored entry data from the rendered DOM text node, so the preview rule can be implemented without changing storage or backend contracts.

## Goals / Non-Goals

**Goals:**
- Keep clipboard history rows readable by capping visible preview text at 256 characters.
- Make truncation explicit with an ellipsis when overflow occurs.
- Preserve full entry content everywhere outside the rendered preview string.

**Non-Goals:**
- Changing clipboard storage, filtering, or selection logic.
- Adding hover expansion, tooltips, or a detail pane for full entry text.
- Introducing configuration for preview length in this change.

## Decisions

- Apply truncation at render time in the picker UI.
  - Rationale: the requirement only affects presentation, and render-time truncation avoids mutating the source entry data used elsewhere.
  - Alternative considered: truncating entries when they are stored or fetched; rejected because it would leak a UI concern into domain behavior and could break search/copy fidelity.

- Use a fixed limit of 256 characters with an appended ellipsis only when the original content exceeds the limit.
  - Rationale: the requirement is explicit, deterministic, and easy to verify.
  - Alternative considered: CSS-only visual clipping; rejected because visual clipping does not guarantee a 256-character preview and may not clearly signal truncation.

- Keep the implementation local to the existing picker rendering flow, with a small helper only if needed for readability.
  - Rationale: this is a single-view behavior change, so the smallest correct change is preferable.
  - Alternative considered: introducing a broader formatting utility module; rejected because the behavior is not reused elsewhere today.

## Risks / Trade-offs

- [Character-count truncation may cut text at an awkward boundary] -> Accept for now because the requirement is a fixed preview cap, and it keeps implementation simple and predictable.
- [Future UI surfaces may need a different preview policy] -> Contain the behavior to the picker rendering path so later surfaces can opt into separate rules.
- [An ellipsis character choice could differ from plain three dots] -> Follow the product requirement during implementation and use one consistent representation everywhere.

## Migration Plan

1. Update the picker rendering path to convert full entry content into a preview string before assigning `textContent`.
2. Verify entries at, below, and above 256 characters render correctly while filtering and selection still use full content.
3. Ship without data migration or rollback steps because the change is UI-only and does not alter persisted state.

## Open Questions

- None.
