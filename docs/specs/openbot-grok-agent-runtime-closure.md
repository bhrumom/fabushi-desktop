# OpenBot + Grok Agent Runtime Closure — Specification

Status: active  
Owner: AI-assisted desktop refactor  
Last updated: 2026-09-22  
Related project: projects/grok-fabu-parity  
Related task / issue / PR: user task Fabushi:5fc4d16a-4172-4c99-89d5-08f24b7c435a; predecessor PR #9

## 1. Context / problem

Fabushi Desktop has already merged the first Agent-first Rust runtime refactor through PR #9, but the live `main` still contains a 4,987-line `desktop/src/adapters/legacy-messaging/legacy-messaging-shell.tsx` that directly imports/renders Agent product surfaces and owns large amounts of Agent-adjacent orchestration. That means the architecture is not yet at the intended boundary even though ConversationActor, RuntimeStore, CapabilityBroker, ComputerControlLease, push events and the low-power avatar exist.

The current task is the closure pass: keep the good boundaries already merged, remove the remaining compatibility-shell ownership leaks, finish the product-shell split, then verify only through GitHub Actions and release only from an accepted exact source revision.

Verified task-start repository facts:
- canonical repository: `bhrumom/fabushi-desktop`
- task-start latest `main`: `ba0eb705665a7d938f19ea334208cc941d7c23b7`
- PR #7 is closed/unmerged; its head `4771410a79ae750ce5be6337cce09f0d42c629bf` is historical, not the implementation base
- PR #9 merged the Agent-first Rust runtime architecture as `a6d6a56fdc836135e16c126a559ed51ce94e4de1`
- repository-wide Spec-first policy is active through PR #15
- desktop version at task start is `1.2.74`

## 2. Goal

Complete the OpenBot/Grok-inspired Fabushi desktop refactor so that:
- Grok-style Agent desktop organization and UI composition are owned by the Agent product layer;
- OpenBot-style capability governance remains centralized in Mahayana Rust;
- Mahayana Rust remains the source of truth for Agent execution, turn/run lifecycle, persistence and local-computer control;
- compatibility messaging features remain available without owning or rendering the Agent product shell;
- high-frequency Renderer polling and independent per-avatar animation loops do not return;
- CI is run only after the refactor code is complete; merge and installer publication happen only after the exact-head gates are green.

## 3. Non-goals / out of scope

- Do not copy recovered Grok Bot source wholesale.
- Do not move Fabushi to Tauri, Docker, one-container-per-Agent, PostgreSQL or a cloud-first architecture.
- Do not remove Contacts, Telegram, Mini Apps, Payments, Calls or other compatibility features solely to make architecture checks pass.
- Do not weaken tests, timeouts, assertions, policy checks or acceptance criteria to obtain green CI.
- Do not run local build/test as a substitute for GitHub Actions acceptance.
- Do not change the local-first product requirement that Agents control the user-installed Fabushi computer.

## 4. Requirements

### R1 — Product shell ownership
`AgentRootShell` / Agent-owned composition must own the visible desktop Agent layout: Agent roster/sidebar, header, transcript, composer, context/computer surfaces and Agent controllers.

The compatibility layer must not import or render:
- `AgentSidebar`
- `AgentHeader`
- `AgentNetwork`
- `AgentWorkspace`
- `AgentCommandPalette`

### R2 — Compatibility isolation
Contacts, Telegram, Mini Apps, Payments, Calls, Settings and legacy messaging behavior must be behind explicit compatibility/feature adapters. The 4,987-line legacy shell may not remain the owner of both Agent and compatibility product logic.

The end state must either delete `legacy-messaging-shell.tsx` or reduce it to a small compatibility composition boundary whose code size and responsibilities are enforced by the architecture check. Renaming an oversized god component is not completion.

### R3 — Rust runtime ownership
Keep ConversationActor, LogicalTurn/ExecutionRun state, RuntimeStore SQLite, CapabilityBroker, ask_user/handoff intents and ComputerControlLease Rust-owned. Renderer code may project state, not become the source of truth.

### R4 — Turn/run model
A logical user turn and an execution run are distinct durable entities. Retries/recovery create additional run generations without duplicating the logical assistant answer.

Allowed lifecycle semantics remain:
`accepted -> queued -> preparing -> thinking -> tool-running -> streaming -> waiting-user -> completed|failed|cancelled`, with `recovering` for restart recovery.

### R5 — Persistence
Agent identity, sections, conversation drafts, turn/run state, provider state, permissions, runtime state and cloud revisions may not use renderer `localStorage` as the source of truth. Legacy reads are migration-only and must be explicitly bounded.

### R6 — Event/power architecture
- hidden Renderer windows must use Electron background throttling;
- normal Agent/runtime state must be event-driven;
- no 2-second account sync loop, 5-second sidebar layout polling, 650ms/1.5s Renderer Remote Computer polling or 500ms Host event receive loop may be reintroduced;
- background Host/presence may remain alive in Main/Rust;
- update checks may remain bounded foreground/background service timers because they are not Agent/runtime polling.

### R7 — Typed identity boundary
Agent, Contact, Telegram and MiniApp Bot identities must remain distinct at the type/domain boundary. Agent product code must not reconstruct Telegram/contact peers via string-prefix heuristics.

### R8 — Design system and avatar
Agent-visible product UI must use Fabushi primitives/tokens rather than new ad-hoc styling. `FabAvatar` remains the Agent avatar boundary:
- static roster avatars: zero animation loop;
- ambient/active animation: shared/low-frequency CSS or a single shared clock if a clock is required;
- no per-avatar `requestAnimationFrame`;
- no DOM/MutationObserver inference of unread/runtime state;
- avatar state is explicit and limited to user-meaningful lifecycle states.

### R9 — Capability governance
Computer, Browser, Filesystem, MCP, Mini App, Connector, Shell and Agent-to-Agent privileged actions must enter the Rust `CapabilityBroker` before domain executors. Decision/audit data must retain actor, agent, conversation, run, capability, target/intent and allow/deny/needs-user outcome.

### R10 — Computer control lease / takeover
Physical desktop control is single-controller. Agent AI and remote paths must honor `ComputerControlLease`; user takeover must suspend/override Agent control with auditable lifecycle and allow return-to-Agent.

### R11 — Agent collaboration
Agent-to-Agent work uses durable handoff/broadcast semantics with bounded depth/fan-out, not synchronous nested prompt recursion. `ask_user` is a first-class waiting-user path.

### R12 — UI layout
Desktop Agent product uses the three-area structure:
- Agent roster/navigation
- Conversation (header/transcript/composer)
- Context/Computer surface

Compatibility feature surfaces may overlay or replace the center/right content when explicitly selected, but they may not become the root Agent shell.

### R13 — Architecture enforcement
`desktop/scripts/check-agent-workspace-boundary.mjs` must fail CI if:
- compatibility code imports/renders Agent product shell components;
- an oversized compatibility god component returns;
- Renderer runtime polling, forbidden localStorage writes, direct privileged bypasses, per-avatar loops, or obsolete shell files return.

### R14 — Verification sequence
Implementation sequence is:
1. complete all code/spec/test changes on one refactor branch;
2. open/update one PR;
3. run GitHub Actions on the exact PR head;
4. fix failures on the branch and repeat until all required gates are green;
5. perform spec compliance review against the final exact head;
6. merge through the protected branch workflow;
7. package/publish installer(s) only from the exact merged source using repository release workflow(s).

No installer publication from an unaccepted refactor head.

## 5. Current state

Already present on `main`:
- `AgentRootShell`, `AgentWorkspace`, Agent transcript/composer/controller modules;
- Rust `ConversationActor`, `RuntimeStore`, `CapabilityBroker`, `ComputerControlLease`;
- `backgroundThrottling: true`;
- Main-owned remote device supervisor;
- Host push-event path;
- low-power `FabAvatar`;
- architecture checker;
- version 1.2.74.

Known remaining contradiction at task start:
- `desktop/src/adapters/legacy-messaging/legacy-messaging-shell.tsx` is 4,987 lines / ~254k chars, imports Agent product components, contains Agent controller wiring, compatibility feature rendering, local persistence migration code, settings/calls/payments/miniapps and many native calls.
- `AgentRootShell` is currently only a thin wrapper around children rather than the actual owner of the complete Agent product composition.
- `projects/grok-fabu-parity/PARITY.md` still labels transcript projection and Plugins/Computer as IN PROGRESS and contains stale pre-merge wording.

## 6. Target state

The root application composes an Agent product surface and explicit compatibility feature adapters. Agent state/controller ownership is outside compatibility files. Compatibility feature modules are independently testable and cannot silently regain Agent runtime ownership.

Rust remains authoritative for Agent execution and privileged capability governance. Electron Main remains the native/background composition layer. Renderer is a projection/UI client.

## 7. Architecture and ownership boundaries

```text
DesktopApp
  -> AgentRootShell
      -> AgentProductSurface
          -> AgentSidebar
          -> AgentWorkspace
              -> AgentHeader
              -> AgentTranscript
              -> AgentComposer
          -> AgentNetwork / CommandPalette / Context / Computer overlays
      -> CompatibilityRouter
          -> Contacts adapter
          -> Telegram adapter
          -> MiniApp adapter
          -> Payments adapter
          -> Calls adapter
          -> Settings adapter

React Renderer
  -> typed desktop bridge
Electron Main
  -> native/background services
Mahayana Rust Runtime
  -> Conversation Actors
  -> RuntimeStore
  -> CapabilityBroker
  -> ComputerControlLease
  -> sync/presence/persistence
```

Prohibited dependency direction:
- compatibility adapters -> Agent shell implementation components
- UI view components -> direct privileged native executor bypass
- avatar -> parent DOM/business-state inference
- renderer -> authoritative Agent run/persistence state

## 8. Interfaces / contracts / schemas / data flow

Existing Rust wire/state contracts remain compatible unless a required ownership move exposes a missing explicit contract.

Key data/control flow:
```text
User intent
  -> Agent product controller
  -> AgentCoordinatorClient / typed bridge
  -> Electron Main
  -> Mahayana Runtime command
  -> ConversationActor
  -> CapabilityBroker (when privileged)
  -> executor
  -> Rust event
  -> Main push
  -> Renderer projection
```

Compatibility messages use their own typed identity/event adapters and do not backfeed Agent state.

## 9. Constraints and non-functional requirements

- local-first
- maintain existing compatibility feature behavior
- no new high-frequency idle wakeups
- no renderer-global Agent runtime pointer/state
- no destructive account-switch migration
- no privilege bypass
- exact-source CI evidence
- protected-branch integration
- release artifact provenance tied to merged SHA

## 10. Failure modes and edge cases

Must preserve:
- runtime restart while a turn is active;
- failed run followed by retry generation;
- account switch during draft/sync state;
- cloud/sidebar revision conflict;
- two Agents active concurrently;
- waiting-user approval/takeover;
- foreground/background window transitions;
- compatibility feature events while Agent workspace is active;
- stale/unscoped legacy Remote Computer events;
- release workflow failure after merge (must not rewrite accepted source just to package).

## 11. Implementation strategy

1. Move Agent-visible composition and controller wiring out of the legacy compatibility shell into Agent-owned product modules.
2. Split compatibility feature state/rendering by domain. Reuse the existing compatibility adapter contracts and avoid behavior rewrites where unnecessary.
3. Shrink/delete the legacy shell and strengthen the architecture checker with ownership and size/forbidden-import rules.
4. Close remaining transcript/computer projection notes in the parity spec only when code evidence supports closure.
5. Add/adjust focused E2E/static tests for shell ownership and retained compatibility.
6. Only after implementation is complete, run required GitHub Actions on the exact head.
7. Merge only when required checks pass; release only from merged exact source.

## 12. Verification / test strategy

Required GitHub Actions evidence:
- Desktop Chat Parity workflow on exact PR head;
- Rust Desktop Runtime workflow on exact PR head;
- architecture checker in CI;
- renderer typecheck/build in CI;
- focused Electron Agent E2E including shell ownership, sections, two-Agent isolation and canonical transcript;
- retained compatibility smoke coverage for Contacts/Telegram/MiniApp/Payments/Calls/Settings as applicable;
- packaging workflow from exact merged SHA.

Static acceptance checks include:
- no obsolete `messaging-shell-v2.tsx`;
- compatibility files do not import Agent product shell components;
- no forbidden Renderer runtime polling;
- no per-avatar animation loop/MutationObserver state inference;
- no direct CapabilityBroker bypass on covered privileged paths.

## 13. Acceptance criteria / Definition of Done

- AC-1: Agent product composition is owned outside legacy compatibility files.
- AC-2: legacy compatibility shell is deleted or reduced to a small compatibility-only composition module with no Agent shell/component imports.
- AC-3: compatibility domains remain available through explicit adapters.
- AC-4: architecture checker enforces AC-1/AC-2 and power/runtime boundaries.
- AC-5: Rust ConversationActor/Turn/Run/RuntimeStore/CapabilityBroker/ComputerControlLease ownership remains intact.
- AC-6: no prohibited high-frequency Renderer/Host polling or per-avatar loops are present.
- AC-7: exact-head Desktop Chat Parity CI is green.
- AC-8: exact-head Rust Desktop Runtime CI is green.
- AC-9: final spec compliance record maps every requirement/AC to evidence or an explicit blocker.
- AC-10: refactor PR is merged only after AC-7/AC-8 and protected-branch requirements pass.
- AC-11: installer package is built/published from the exact merged source; artifact/run/release provenance is recorded.

## 14. Release / migration / rollback

Migration:
- preserve current persisted Rust RuntimeStore and existing compatibility data;
- any remaining localStorage reads are migration-only and must not delete user data until authoritative write succeeds.

Release:
- bump desktop version only after implementation has stabilized and before final exact-head CI.
- merge through the protected PR path.
- run the repository installer workflow from the exact merged SHA.

Rollback:
- if post-merge packaging fails, fix packaging/release infrastructure without pretending the accepted source artifact exists.
- if runtime regression is found before merge, keep PR open and fix on the same branch.

## 15. Observability / evidence

Retain:
- PR head SHA;
- GitHub Actions run IDs/job IDs;
- uploaded Playwright artifacts and digests when produced;
- architecture checker output;
- merged SHA;
- packaging run ID;
- installer artifact/release URL and checksum/signing/notarization evidence when the workflow provides them.

## 16. References / provenance

- Latest explicit user requirement: task `Fabushi:5fc4d16a-4172-4c99-89d5-08f24b7c435a`, 2026-09-22.
- `AGENTS.md` and `docs/specs/spec-first-ai-development.md`.
- `projects/grok-fabu-parity/PARITY.md`.
- `docs/fabu-agent-architecture-parity.md`.
- predecessor PR #9 and merged commit `a6d6a56fdc836135e16c126a559ed51ce94e4de1`.
- reference design principles: recovered Grok/Fabu Agent desktop organization + OpenBot capability governance, without source-code copying.

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| R1-R14 | pending | implementation not yet complete |
| AC-1-AC-11 | pending | implementation/CI/release pending |
