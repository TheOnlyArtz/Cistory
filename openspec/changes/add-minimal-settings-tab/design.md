## Context

The picker currently renders a single clipboard-history view and already integrates with backend commands for settings persistence (`load_settings`, `set_hotkey`, `set_autostart`). The change introduces a minimal in-window settings surface without adding a second window or new backend endpoints. The existing picker window is fixed-size and auto-hides on close/blur, so the design must remain compact and resilient to transient focus changes.

## Goals / Non-Goals

**Goals:**
- Add a lightweight settings view reachable from a top-right gear button.
- Keep history and settings within one picker window and one frontend bundle.
- Support two settings only: hotkey binding (including keyboard recording) and autostart toggle.
- Reuse existing backend commands and validation paths for persistence and error handling.
- Provide inline operation feedback (saved/error) with minimal interaction friction.

**Non-Goals:**
- No new settings categories beyond hotkey and autostart.
- No redesign of tray behavior, window lifecycle, or backend hotkey architecture.
- No new backend command contracts, schema migrations, or additional dependencies.
- No multi-step settings wizard, modal-based configuration, or advanced key-sequence editing.

## Decisions

- Use in-place view switching instead of a new window.
  - Rationale: Minimizes complexity, preserves current picker invocation flow, and avoids additional Tauri window state.
  - Alternative considered: dedicated settings window launched from tray/picker; rejected for added lifecycle and focus complexity.

- Apply settings immediately on user action instead of introducing an explicit Save button.
  - Rationale: Fits minimal UX, reduces clicks, and aligns with command-level persistence already implemented.
  - Alternative considered: draft changes with explicit Save/Cancel; rejected to keep UI and state management small.

- Implement hotkey recording as explicit capture mode triggered by a Record button.
  - Rationale: Prevents accidental capture during normal navigation and makes keyboard ownership clear.
  - Alternative considered: always-on key capture in the hotkey field; rejected due to conflict risk with existing picker shortcuts.

- Keep backend as source of truth for valid binding and final saved state.
  - Rationale: Existing parser/validator and registration logic already enforce platform constraints.
  - Alternative considered: full frontend hotkey parsing before submit; rejected as duplicate logic and drift risk.

## Risks / Trade-offs

- [Hotkey capture conflicts with picker key handlers] -> Suspend or bypass list-navigation shortcuts while recording and treat Escape as cancel-recording first.
- [Platform-dependent hotkey activation behavior (e.g., Wayland deferred registration)] -> Surface inline status that distinguishes "saved" from activation limitations when command errors/diagnostics indicate issues.
- [Fixed window dimensions constrain settings layout] -> Use compact single-column controls and concise labels to avoid clipping without resizing the window.
- [Immediate apply can persist accidental toggle/binding] -> Provide clear inline feedback and allow quick correction via the same controls.
