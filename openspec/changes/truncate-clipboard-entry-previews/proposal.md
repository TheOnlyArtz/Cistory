## Why

Clipboard entries can currently render at full length in the picker, which makes long items hard to scan and can visually crowd out neighboring entries. A small, explicit preview limit is needed now so the history list stays readable while still signaling that more text exists.

## What Changes

- Add a clipboard entry preview requirement that limits rendered list previews to 256 characters.
- Require truncated previews to end with an ellipsis so users can distinguish shortened text from complete short entries.
- Preserve full clipboard entry content for filtering, selection, storage, and copy-back behavior; only the visible preview changes.

## Capabilities

### New Capabilities
- `clipboard-entry-previews`: Define how clipboard history entries are summarized in the picker list, including truncation length and overflow signaling.

### Modified Capabilities
- None.

## Impact

- Affects picker rendering in `src/main.ts`, where clipboard entry text is shown in the history list.
- Affects the new OpenSpec capability file under `openspec/changes/truncate-clipboard-entry-previews/specs/clipboard-entry-previews/spec.md`.
- Does not change persistence, clipboard ingestion, or clipboard selection semantics.
