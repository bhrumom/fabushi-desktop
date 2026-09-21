# Post-1.2.74 Agent Architecture Cutover — Specification

Status: active  
Owner: Fabushi desktop architecture  
Last updated: 2026-09-21  
Related project: `projects/grok-fabu-parity`  
Related task / PR: PR #14, user task `[Fabushi:76d588cf-3096-4cea-8685-2ca518f3c7c6]`

## 1. Context / problem

Fabushi Desktop must finish the architecture rebuild that absorbs the strongest design boundaries from Grok Bot and OpenBot while retaining Fabushi's Electron desktop shell, Mahayana Rust Agent runtime, local-first persistence, and control of the user's installed computer.

The historical renderer/messaging shell mixed Agent product UI, compatibility messaging, persistence, polling, runtime coordination, computer control, cloud synchronization, and presentation state. That coupling caused duplicate/stuck turn state, section projection races, unnecessary renderer wakeups, fragile Bot/Contact distinctions, and difficult verification.

The current continuation branch is `refactor/post-1.2.74-architecture-cutover-20260921` / PR #14. It is a continuation of the architecture merged by PR #9 and must not be considered complete merely because prior PRs or branches claimed parity.

## 2. Goal

Finish the architecture cutover completely before starting a new acceptance CI cycle, then verify the exact completed source in GitHub Actions, merge only if all required gates pass, and publish a signed/notarized macOS installer from the exact merged release source.

## 3. Non-goals / out of scope

- Do not replace Electron with Tauri.
- Do not import OpenBot's Docker/Postgres/one-Bot-one-container deployment model.
- Do not copy reconstructed Grok source wholesale.
- Do not move Mahayana Agent execution to a cloud-only service.
- Do not make Contacts, Telegram, Mini Apps, Payments, Calls, or Settings owners of Agent runtime state.
- Do not publish an installer from an unmerged PR head.
- Windows/Linux packaging is not a gate for this cutover unless an existing release workflow makes it mandatory.

## 4. Requirements

- **R1 — Sole Agent product root.** `DesktopApp` must mount `AgentRootShell` as the sole authenticated desktop product root. The old `messaging-shell-v2.tsx` and `legacy-messaging-shell.tsx` must be deleted. Compatibility domains must live behind explicit feature adapters and must not own Agent runtime state.
- **R2 — Typed identity boundaries.** Agent/Bot, Contact, Telegram and Mini App identities must remain explicit domain types/projections. The Agent root must not construct Telegram/contact peers by string inference.
- **R3 — Rust-owned conversation execution.** Mahayana Rust must own one serial execution lane per conversation, LogicalTurn/ExecutionRun identity, the canonical turn lifecycle, queueing, cancellation/recovery semantics, and provider-session ownership. Renderer state must be projection only.
- **R4 — Rust-owned durable Agent state.** RuntimeStore/SQLite must own Agent conversation/runtime/workspace durable state. Renderer `localStorage` may contain cosmetic/window-local values and one-time migration reads only, not authoritative Agent/turn/provider/permission/sidebar/runtime state.
- **R5 — Capability governance.** Privileged Computer, Browser/Remote Computer, filesystem, MCP, Mini App, Connector, shell and Agent-to-Agent actions must enter the Rust `CapabilityBroker`, produce policy/audit decisions, and then reach domain executors. Existing domain policy remains an additional enforcement layer.
- **R6 — Computer control lease and human takeover.** Physical desktop input must be single-controller via a `ComputerControlLease`. Agent waiting for login/2FA/password/OTP/sensitive confirmation or user judgement must project `waiting-user`; user takeover and release must be explicit lifecycle transitions.
- **R7 — Agent collaboration.** Agent-to-Agent work must use durable handoff/group/broadcast semantics with bounded fan-out/depth rather than synchronous prompt recursion. `ask_user` is a first-class waiting path.
- **R8 — Event-driven/power architecture.** UI must not rely on permanent 2s account sync, 5s sidebar sync, renderer-owned sub-second Remote Computer polling, or disabled Electron background throttling. Renderer should receive pushed typed runtime events and be throttleable while hidden; background services may remain alive outside the renderer.
- **R9 — Low-power avatar.** Desktop Agent/Bot avatar paths must use `FabAvatar` with explicit simplified state. Legacy `BotMark` / `FabushiAvatarRuntime`, per-avatar `requestAnimationFrame`, and DOM `MutationObserver` business-state inference are prohibited.
- **R10 — Fabushi design system.** Product UI must use shared Fabushi tokens/primitives for common buttons, menus, popovers, dialogs, tooltips, selects, badges, avatars, spinners, inputs/surfaces/switches rather than reintroducing large feature-local inline styling systems.
- **R11 — Architecture regression checks.** Static architecture checks must fail on reintroduction of deleted legacy shells, old avatar/runtime patterns, renderer ownership of Agent durable/runtime state, privileged-path CapabilityBroker bypasses, or merge-ref-only acceptance evidence.
- **R12 — Verification order.** Product-affecting refactor code is completed first. Only then may a new exact-head acceptance run be used as the completion gate.
- **R13 — Exact-source CI.** Required PR workflows must explicitly verify the exact `pull_request.head.sha`, not only GitHub's synthetic merge ref. Required scopes are renderer/typecheck/build + focused desktop parity, Rust desktop runtime/architecture, and packaged OpenBot/Grok/Mahayana acceptance required by the branch.
- **R14 — Merge gate.** PR #14 must not merge until all exact-head required gates are green and evidence belongs to the same accepted head.
- **R15 — Release gate.** After architecture merge, create the required monotonic version bump (at least 1.2.75), verify the release source as required by repository workflow, then publish the signed/notarized macOS installer from the exact merged release SHA. Do not reuse 1.2.74 as the new architecture-complete public release.

## 5. Current state

Verified on 2026-09-21 before implementation:
- canonical `main` is at least `ba0eb705665a7d938f19ea334208cc941d7c23b7`, which adds repository-wide Spec-first governance;
- PR #7 is closed/unmerged at `4771410a79ae750ce5be6337cce09f0d42c629bf` and is not the active implementation branch;
- PR #9 merged the first Agent-first Rust runtime architecture into main at `a6d6a56fdc836135e16c126a559ed51ce94e4de1`;
- active continuation PR #14 is `refactor/post-1.2.74-architecture-cutover-20260921`;
- at inspected PR #14 head `13bbcd2ece31a74fc9252822db2a422641deefaa`, `legacy-messaging-shell.tsx` is absent, `DesktopApp` mounts `AgentRootShell`, and Rust contains `ConversationActor` and `CapabilityBroker`;
- desktop version remains `1.2.74`;
- `projects/grok-fabu-parity/PARITY.md` is the detailed parity inventory but explicitly is not, by itself, a completion claim.

## 6. Target state

The desktop product is Agent-first: React owns presentation, Electron owns native composition/background services, and Mahayana Rust owns execution, lifecycle, policy, durable state and privileged capability governance. Compatibility features are isolated adapters. Hidden windows can throttle renderer work without stopping required background services. A single logical turn may have multiple execution generations without duplicate user-visible answers.

## 7. Architecture and ownership boundaries

- **React Renderer:** presentation/projection/controllers only; no durable Agent source of truth, no direct privileged execution, no permanent synchronization polling.
- **Electron Main:** trusted native composition, lifecycle, secure bridge, updater, packaging and background service hosting; does not become the Agent business-state authority.
- **Mahayana Rust Runtime:** Agent/Conversation actors, LogicalTurn/ExecutionRun lifecycle, RuntimeStore, CapabilityBroker, collaboration, ask-user/handoff semantics, ComputerControlLease and audit.
- **Compatibility adapters:** Contacts/Telegram/MiniApps/Payments/Calls/Settings only; may translate legacy/product events but may not import/render/own Agent runtime root/controllers.

## 8. Interfaces / contracts / schemas / data flow

Canonical execution flow:

`User intent -> persisted LogicalTurn -> ConversationActor queue -> ExecutionRun(generation) -> Rust TurnState events -> Electron typed push -> renderer projection`.

Canonical capability flow:

`CapabilityRequest{actor,agent_id,conversation_id,run_id,capability,target,intent} -> CapabilityBroker -> allow/deny/needs-user + AuditLog -> domain executor`.

Canonical computer control flow:

`Agent requests control -> CapabilityBroker -> ComputerControlLease -> execute/observe/wait -> optional human takeover -> explicit release/resume`.

## 9. Constraints and non-functional requirements

- Local-first and restart-safe.
- No weakening of security/approval gates to make tests pass.
- Hidden renderer must be throttleable.
- No high-frequency idle polling where a push/revision event can express the change.
- Per-Agent/conversation isolation must prevent cross-talk.
- Release evidence must be tied to exact immutable SHAs.
- User secrets/passwords/OTP must not be captured into Agent prompts or release evidence.

## 10. Failure modes and edge cases

- stale runtime generation after Host restart;
- retry run for the same LogicalTurn;
- simultaneous conversations on different Agents;
- same-conversation overlapping sends;
- stale cloud/sidebar revision conflict;
- user takeover during an active tool/computer run;
- capability denied or requiring user approval;
- renderer hidden/backgrounded;
- release workflow running on a SHA different from the accepted CI SHA;
- PR head moving after successful CI.

## 11. Implementation strategy

1. Inspect the live PR #14 head against R1-R11.
2. Repair any missing architecture/code/test boundary before new acceptance CI.
3. Keep architecture guards aligned with the final boundaries.
4. Once code is complete, run fresh GitHub Actions on the exact final PR head.
5. Review every requirement/AC against code + Actions evidence.
6. Merge PR #14 only if exact-head gates pass.
7. From the resulting canonical main, perform the monotonic release version bump, verify the release source as required, and run the signed/notarized macOS release workflow.
8. Publish only if release acceptance succeeds; otherwise report the concrete blocker without substituting older evidence.

## 12. Verification / test strategy

- Static source inspection and architecture checker for R1-R11.
- Renderer typecheck/build + focused Electron parity E2E for Agent shell, transcript/composer, Sections and multi-Agent isolation.
- Rust unit/integration tests for ConversationActor lifecycle, RuntimeStore, CapabilityBroker, ComputerControlLease and collaboration semantics.
- Exact-head CI assertion that `git rev-parse HEAD` equals `pull_request.head.sha`.
- Packaged macOS acceptance with `OBF_REAL_ACCEPTANCE=1`, including direct handoff, broadcast, two-Agent isolation, lifecycle, screenshots/video/trace/logs as configured by the workflow.
- Release workflow must use the exact merged release SHA and produce signed/notarized artifacts.

## 13. Acceptance criteria / Definition of Done

- **AC-1:** deleted legacy shells remain absent and `AgentRootShell` is the sole product root.
- **AC-2:** compatibility adapters have no Agent runtime ownership.
- **AC-3:** ConversationActor/LogicalTurn/ExecutionRun lifecycle and durable state are Rust-owned and covered by tests.
- **AC-4:** privileged capability paths are brokered/audited, including computer and Agent collaboration.
- **AC-5:** no prohibited high-frequency renderer sync/remote polling or `backgroundThrottling: false` remains.
- **AC-6:** old avatar runtime/DOM inference/per-instance rAF patterns are absent from desktop source.
- **AC-7:** architecture guard rejects regression of AC-1 through AC-6.
- **AC-8:** all required exact-head PR Actions pass on one immutable final PR head.
- **AC-9:** spec compliance review records every R/AC as passed, blocked, or not-applicable with evidence.
- **AC-10:** PR #14 merges only after AC-8.
- **AC-11:** new release version is strictly greater than 1.2.74.
- **AC-12:** signed/notarized macOS installer is produced/published from the exact merged release SHA after its required gates pass.

## 14. Release / migration / rollback

Do not publish from PR #14 directly. Merge the accepted architecture first. Then bump the desktop release version to at least 1.2.75 on canonical main through the repository's normal protected integration path. Release must be tied to the resulting immutable merge SHA. If packaged acceptance, signing, notarization, or exact-SHA verification fails, stop publication and keep the previous public version as rollback.

## 15. Observability / evidence

Retain:
- PR head SHA and merge SHA;
- workflow run/job IDs and conclusions;
- exact-head assertion output;
- packaged-app acceptance artifacts;
- video/screenshots/trace/logs/digests produced by the workflow;
- release/tag/artifact identifiers and installer checksums when available.

## 16. References / provenance

- Latest explicit user requirement in task `[Fabushi:76d588cf-3096-4cea-8685-2ca518f3c7c6]`.
- `AGENTS.md` and `docs/specs/spec-first-ai-development.md` on canonical main.
- `projects/grok-fabu-parity/PARITY.md`.
- PR #9 architecture merge; PR #14 active continuation.
- Reference design intent: Grok Bot desktop organization + OpenBot capability governance + Mahayana Rust execution.

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1-R15 | pending | Complete after final implementation/verification. |
| AC-1-AC-12 | pending | Complete after final implementation/verification. |
