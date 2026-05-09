## Context

Clipboard history currently stores and renders text-only entries. The ingestion layer normalizes text snapshots, the storage layer persists text payloads in SQLite, and the picker renders text previews. Supporting images introduces cross-cutting changes across ingestion, domain contracts, storage schema, filesystem handling, and UI rendering.

The feature must remain local-first and align with existing Linux behavior constraints. Image payloads should be written to a temporary directory, while the database stores durable references to those files so history can render and select entries consistently. Because temporary files can disappear independently of database records, the system also needs resilient handling for missing files.

## Goals / Non-Goals

**Goals:**
- Capture image clipboard content and normalize it as first-class history entries.
- Persist image entry references in SQLite while storing image binary files in an application-owned temp directory.
- Render image previews in fixed-size picker containers so cards remain layout-safe.
- Preserve predictable copyback behavior and avoid recursive ingestion loops.
- Maintain compatibility with existing text history behavior.

**Non-Goals:**
- Cloud sync, remote asset storage, or outbound image transfer.
- Advanced image editing, annotation, or transformation features.
- Backfilling historical text entries into image-aware metadata beyond required migration defaults.
- Reworking picker information architecture beyond the required image-preview additions.

## Decisions

1. **Store image binaries in temp files, not in SQLite blobs.**
   - **Decision:** Write image data to an app-specific directory under the system temp root and store the absolute path in SQLite.
   - **Rationale:** Keeps the database smaller, reduces migration complexity for large payloads, and matches the requested storage model.
   - **Alternatives considered:**
     - SQLite BLOB columns: simplifies single-file persistence but increases DB size and I/O pressure.
     - Base64 in text columns: high overhead and poor query/storage ergonomics.

2. **Introduce explicit image path field in schema/domain mapping.**
   - **Decision:** Keep text content semantics intact and add a dedicated image-path field for image entries.
   - **Rationale:** Avoids overloading existing text search behavior and enables clear type-aware rendering.
   - **Alternatives considered:**
     - Reusing `content` to store file paths: lower migration cost but ambiguous semantics and brittle filtering behavior.

3. **Expand content-kind contract from text-only to typed text/image.**
   - **Decision:** Extend `ContentKind` and associated serialization/mapping paths to include image entries.
   - **Rationale:** Preserves strict type boundaries between ingestion, storage, and UI rendering.
   - **Alternatives considered:**
     - Inferring kind from path prefixes or MIME-only metadata: implicit and error-prone.

4. **Use fixed-size thumbnail containers in picker previews.**
   - **Decision:** Render image previews inside a constrained frame with deterministic width/height and object-fit behavior.
   - **Rationale:** Prevents large images from breaking list flow and keeps keyboard navigation visually stable.
   - **Alternatives considered:**
     - Natural-size rendering: higher fidelity but layout instability.
     - Dynamic-height cards: degrades list scanning and selection predictability.

5. **Treat missing temp files as non-fatal degraded entries.**
   - **Decision:** If image path no longer exists, show fallback preview state and keep app running.
   - **Rationale:** Temp directories can be cleaned externally; runtime resilience is required.
   - **Alternatives considered:**
     - Hard-fail on missing files: unacceptable reliability impact.
     - Silent row deletion: risks surprising data loss.

## Risks / Trade-offs

- **[Temp files may be deleted by OS cleanup]** -> Mitigation: guard rendering and copyback paths with existence checks; surface graceful fallback UI and diagnostics.
- **[Orphaned image files can accumulate]** -> Mitigation: tie lifecycle cleanup to retention pruning and optional startup sweep.
- **[Schema migration can create mixed old/new rows]** -> Mitigation: additive migration with safe defaults and backward-compatible mapping.
- **[Image writes may increase I/O and latency]** -> Mitigation: hash-based dedupe and bounded payload handling before persistence.
- **[Search semantics become type-dependent]** -> Mitigation: keep text filtering on text payloads and avoid path-string leakage into user-facing search behavior.

## Migration Plan

1. Add additive SQLite migration for image-aware fields and indexes required for retrieval.
2. Extend domain serialization and storage mapping to read/write both text and image entry variants.
3. Add ingestion support for image snapshots, temp-file persistence, and image-entry upsert flow.
4. Update picker rendering/CSS to show fixed-size image thumbnails with fallback states.
5. Validate copyback and pruning behavior for both text and image entries; ensure image-path cleanup for pruned entries.

Rollback strategy: if deployment issues emerge, disable image ingestion path while retaining additive schema compatibility so existing text flow remains operational.

## Open Questions

- Should image copyback restore native image clipboard data only, or also support text fallback when files are missing?
- What file naming strategy should be canonical (`hash.ext` vs timestamped names) for dedupe and debugging?
- Should startup perform a lightweight reconciliation pass to prune stale image-path records?
