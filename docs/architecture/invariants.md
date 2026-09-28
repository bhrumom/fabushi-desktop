# Architecture invariants

Status: normative  
Last reviewed: 2026-09-22

These rules are repository-wide defaults. A change may alter one only through an explicit Spec + accepted/superseding ADR + migration plan when transition is required.

- **ARCH-001 — Renderer isolation:** Renderer code must not directly own or invoke Agent Runner lifecycle.
- **ARCH-002 — Runtime authority:** Canonical Agent/session/turn execution state belongs to the runtime/Coordinator boundary, not UI presentation state.
- **ARCH-003 — Electron shell boundary:** Electron main owns desktop/OS concerns and routing, not canonical Agent lifecycle truth.
- **ARCH-004 — Thin preload:** Preload is a narrow typed bridge; business/runtime orchestration does not accumulate there.
- **ARCH-005 — Coordinator/Host/Runner separation:** Coordinator, Host, and Runner remain independent responsibility boundaries even if implemented in the same language/workspace.
- **ARCH-006 — Typed contracts:** Cross-boundary communication uses explicit request/reply/event/snapshot contracts. Hidden direct coupling is prohibited.
- **ARCH-007 — Terminal settlement:** Every accepted operation must settle as completed, failed, or cancelled. Crashes/disconnects must not leave indefinite busy state.
- **ARCH-008 — Reconnect/resync:** Transport loss must have an explicit reconnect/resynchronization path when the underlying operation may survive renderer/transport loss.
- **ARCH-009 — No UI inference of runtime truth:** DOM observers, animation state, local timers, naming prefixes, or incidental UI structure must not become the authoritative source for runtime state or identity.
- **ARCH-010 — Failure isolation:** Host/Runner/tool failure must be detectable and must not require the renderer to infer recovery.
- **ARCH-011 — Privileged capability boundary:** Local exec, computer control, MCP/connectors, OAuth, filesystem/network-sensitive actions, and equivalent privileged behavior go through explicit capability/security controls.
- **ARCH-012 — Durable ownership changes require migration:** Moving state/process/protocol ownership requires an explicit migration path; compatibility adapters may be transitional but must not silently become the permanent root architecture.
- **ARCH-013 — Dependency direction is intentional:** Lower runtime layers do not depend on React/Electron presentation concepts. UI depends on stable contracts, not concrete Runner internals.
- **ARCH-014 — Exact-source verification:** Acceptance evidence for a revision must be produced from that exact revision; older green evidence cannot certify a newer head.
- **ARCH-015 — Architecture is enforceable:** Invariants that can be statically checked should be guarded by architecture tests/CI, not documentation alone.
- **ARCH-016 — Docs close with code:** Intentional architecture changes are incomplete until Spec, ADR, canonical architecture docs, migration state, and implementation agree.

When an invariant appears incompatible with a legitimate new requirement, do not bypass it. Propose the change in the task Spec and record the decision in a new ADR.
