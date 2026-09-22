# Grok Bot 0.18 One-to-One Rust Structural Port and Product Parity Recovery — Specification

Status: active  
Owner: Fabushi desktop / Agent runtime  
Last updated: 2026-09-22  
Related project: `projects/grok-fabu-parity`  
Related task / issue / PR: 2026-09-22 packaged-app regression report; spec PR created from canonical main `20644cf5aa2f5777cc350ece32edeb0cb90f88d7`

## 1. Context / problem

The current signed Fabushi desktop package can present a user turn immediately and then remain indefinitely at “正在思考” without producing assistant text. The same installed build can complete narrow deterministic probe prompts while a normal Agent request stalls, so “a probe returned the requested marker” is not sufficient evidence that the product chat path is healthy.

The 2026-09-22 report also identifies user-visible parity gaps versus `b-nnett/grok-bot-0.18-reconstructed`:

- ordinary question/answer latency and streaming do not feel like Grok Bot;
- a normal task such as “创建一个打地鼠的小程序” can stay in thinking state with no answer;
- Plugins / connectors / MCP are not exposed as a complete product surface;
- the creation affordance does not match the user’s Grok Bot reference behavior;
- the packaged app is shown by macOS as using significant energy;
- existing parity/refactor documents and synthetic acceptance have not prevented these packaged regressions.

This specification is based on the following exact source baselines:

- Fabushi desktop: `bhrumom/fabushi-desktop@20644cf5aa2f5777cc350ece32edeb0cb90f88d7` (desktop 1.2.75 canonical main at investigation time).
- Grok Bot reference: `b-nnett/grok-bot-0.18-reconstructed@a9f633e09d49a85829b8236331b9e21f7e612634`.

The Grok repository is a reconstruction. Its readable TypeScript/Node source is substantial, but its provenance documentation states that the complete original frontend source is not present and a retained compiled renderer is part of the fidelity baseline. Therefore exact UI behavior must be verified from observable reference captures, not inferred from partial source alone.

### 1.1 Verified root-cause findings

The investigation found concrete implementation differences that explain the reported behavior.

#### A. Fabushi paints “正在思考” before the Host/provider has actually started

`desktop/src/agent-workspace/use-agent-workspace-runtime.ts` begins a local turn before awaiting host acceptance.  
`desktop/src/agent-workspace/agent-runtime-coordinator.ts` then appends a synthetic assistant `operation.started` entry labelled “正在思考”, using the renderer request id as a provisional operation id.

The real Host later returns/adopts a different operation id. The coordinator contains repair/fallback logic for events that arrive before adoption. This creates an avoidable correlation race, especially with two concurrent Agents. The screenshot state is therefore not proof that model inference has started; it can be only the renderer’s optimistic projection.

#### B. The Rust turn can remain Thinking while provider inference waits

`third_party/mahayana/mahayana-rs/mahayana-runtime/src/lib.rs` moves a turn through Preparing/Thinking and then awaits the provider. It emits terminal state only after the provider returns or fails. There is no first-token/stall deadline around the provider wait in this turn owner.

`desktop/electron/host-process.cjs` has a 120-second request timeout for request/response IPC, but chat acceptance is asynchronous. That timeout does not bound an already-accepted provider turn.

#### C. Provider streaming is inconsistent and there is no Grok-style first-token watchdog

`third_party/mahayana/mahayana-rs/mahayana-model/src/responses.rs`:

- uses streaming for the Responses wire API;
- deliberately sends `stream: false` for Chat Completions;
- does not request streaming for Anthropic Messages;
- performs the HTTP call with `ureq` without an explicit first-token/read/overall deadline in this layer.

By contrast, Grok’s `source/host/runner/stream-attempt.ts` and `source/host/runner/transient-stream-error.ts` explicitly implement a first-token stall watchdog, bounded safe retry, transient/capacity classification, backoff, cancellation, and resume-from-checkpoint semantics. The reconstructed reference’s default first-token stall deadline is 150 seconds and overload retry is bounded; importantly, the runner has an explicit terminal path instead of permitting an unbounded silent “thinking” state.

#### D. Fabushi’s normal Agent task path is heavier than its conversational fast path

`third_party/mahayana/mahayana-rs/mahayana-native-engine/src/lib.rs` allows up to 16 model turns and supplies the full tool schema for ordinary Agent tasks. It has a special lightweight conversational fast path that removes tools and reduces reasoning/output budget, but a task such as creating an application is intentionally routed through the full Agent path. That is valid for capability, but it makes correct streaming, progress, cancellation, timeout, and tool lifecycle essential.

#### E. Connector primitives exist, but the product surface is not Grok-equivalent

Fabushi already has `mcp.*` contracts and `desktop/src/agent-workspace/use-agent-mcp-controller.ts`, but the renderer controller primarily lists servers and projects them into @mention references.

The sidebar’s “Plugins” action in `desktop/src/agent-workspace/agent-root-shell.tsx` currently sets the surface to `miniapps`. That renders `desktop/src/features/miniapps/miniapp-compatibility-adapter.tsx`, which is a Mini Apps marketplace/install/WebMCP surface. It is not equivalent to Grok’s full Plugins/MCP surface.

Grok’s reconstructed plugin stack includes desktop MCP manager/runtime/OAuth, host MCP services, routed MCP bridge, account management, catalog/install/update/remove flows, server authentication, tool listing/toggling, and plugin/private-skill synchronization. The UI source `frontend/src/recovered/features/plugins/overlay/desktop-surface.tsx` exposes this product surface.

#### F. The current “+ / New” behavior cannot be specified from partial source alone

In the reconstructed Grok 0.18 source, the sidebar `+ New` invokes `onNewChat` and the composer’s plus/attachment affordance is for attachments. The user’s current reference behavior includes a create-selection flow. Because the reconstructed repository is explicitly incomplete on the frontend, the observable reference capture is normative when it conflicts with partial source.

#### G. Fabushi’s energy problem is a lifecycle/process problem, not evidence that Rust is inherently inefficient

The renderer already uses `backgroundThrottling: true`, and the current low-power avatar does not run a permanent JavaScript animation frame loop.

However, production `desktop/electron/main.cjs` enables background persistence by default, intercepts window close and hides instead of quitting, starts the Mahayana Host at app startup, starts the remote-device supervisor at startup, and intentionally keeps the Host/account-bound computer presence alive while the window is hidden.

`desktop/electron/remote-device-agent-supervisor.cjs` can spawn a long-lived Node/Electron device-agent helper connected to the official WebSocket gateway and retries a crashed helper every five seconds. Runtime events are also fanned out to every BrowserWindow rather than delivered only to interested subscribers.

These always-live processes/network services are credible sources of idle energy use. The exact contribution of each process must be measured; source inspection alone cannot assign a watt/CPU percentage.

### 1.2 Source-of-truth order

For this recovery:

1. the latest explicit user requirement and reference captures;
2. this specification;
3. observable behavior of the exact Grok reference build/source baseline;
4. existing Fabushi parity/refactor documents.

Any older `IMPLEMENTED` or `COMPLETE` label in `projects/grok-fabu-parity/PARITY.md` is informational only until it is revalidated against this specification on a packaged build.

## 2. Goal

The target is no longer “borrow Grok Bot’s design principles while keeping a different Fabushi desktop architecture.” The target is an **explicit one-to-one Rust structural port of Grok Bot 0.18** from the exact reference baseline `b-nnett/grok-bot-0.18-reconstructed@a9f633e09d49a85829b8236331b9e21f7e612634`.

The implementation must proceed **module by module and file by file** across the Grok Bot code tree:

- every Grok Bot source-code file must have a tracked Fabushi Rust counterpart, or an explicit evidence-backed `not-applicable` classification when the reference file is generated data, a platform-only artifact, or non-executable evidence;
- the **relative source folder architecture must mirror Grok Bot**, including the same major process/domain boundaries and the same nested feature/module organization;
- Grok Bot TypeScript/Node application logic must be translated into Rust with equivalent contracts, state machines, failure behavior, process ownership, persistence, streaming, retries, MCP/plugin behavior, and observable product semantics;
- Fabushi code, folders, services, compatibility layers, and product surfaces that **do not have a Grok Bot counterpart must be removed from the canonical desktop implementation** rather than kept “just in case”;
- the final shipped desktop application must not have a second parallel Fabushi architecture beside the Grok-shaped Rust port;
- Fabushi branding, service endpoints, signing identity, and account credentials may differ through narrow configuration/adapters, but those differences must not introduce a separate product architecture;
- ordinary chat, Agents, creation flow, Plugins/connectors/MCP, power behavior, process lifecycle, error recovery, and UI interaction must match the approved Grok reference behavior.

The desired end state is therefore: **Grok Bot 0.18’s code architecture and product behavior, reimplemented in Rust under Fabushi identity, with no unexplained extra Fabushi desktop subsystems.**

## 3. Non-goals / out of scope

- Keeping the current Fabushi desktop folder layout merely because it already exists.
- Keeping `desktop/src`, `desktop/electron`, `frontend/apps/web`, `third_party/mahayana`, compatibility adapters, Mini Apps, Telegram/messaging, payments, calls, or other Fabushi-only desktop subsystems after cutover **unless an exact Grok Bot counterpart is documented in the port manifest**.
- Reinterpreting “same architecture” as only matching high-level concepts. Folder ownership, module boundaries, dependency direction, process boundaries, runtime contracts, and lifecycle ownership must be mirrored and verified.
- Rewriting Grok behavior into a new “cleaner” architecture when that changes ownership or observable semantics. Improvements may be proposed only after one-to-one parity is complete and separately specified.
- Treating existing Fabushi features as automatically grandfathered. If Grok Bot does not have a counterpart, the default action is removal from the desktop product/code path.
- Treating a deterministic “reply exactly X” probe as proof of ordinary user-chat correctness.
- Declaring parity from unit tests, mocks, source layout, or compile success without signed packaged-app evidence.
- Copying unlicensed upstream source text, comments, compiled code, assets, or branding. “Translate file by file” in this spec means a **Rust semantic port/reimplementation with a traceable one-to-one mapping**. Because the Grok reconstruction provenance explicitly says no upstream source-code license is implied, any direct source-level reuse requires an independent rights review before redistribution.
- Expanding this desktop port into iOS/Android work.

## 4. Requirements

### Chat / turn lifecycle

- **CHAT-001 — Accepted-state correctness.** The renderer may optimistically paint the user bubble, but it must not represent the assistant as “thinking” until a canonical Host operation has been accepted and an actual execution state has started.
- **CHAT-002 — Stable identifiers.** `clientNonce`, renderer/request id, conversation id, Agent id, operation id, turn id, and run id must be distinct typed concepts. A request id must never be temporarily reused as an operation id.
- **CHAT-003 — Durable send journal.** Submission must follow the Grok pattern: local optimistic journal -> queued/offline state -> durable Host acceptance -> run lifecycle. Duplicate client nonces must coalesce safely; reconnect must flush only valid queued submissions.
- **CHAT-004 — One canonical operation owner.** All deltas, tool events, final message, completion, failure, retry, cancellation, and waiting-user state for one turn must carry the canonical operation identity. “Only pending peer” inference must not be required for correctness.
- **CHAT-005 — First-output watchdog.** Every provider attempt must have an explicit first-output deadline, reset/disarm rules, cancellation behavior, and bounded retry. The policy may be configurable, but an accepted operation must never stay silently active forever.
- **CHAT-006 — Bounded safe retry.** Transient network/provider/capacity failures may retry only when no user-visible stream output has been produced or a resumable checkpoint exists. Retry count/backoff/server Retry-After must be bounded and observable.
- **CHAT-007 — Real streaming.** Responses, OpenAI-compatible Chat Completions, and Anthropic Messages routes must use streaming where the upstream supports it. Provider adapters must normalize streaming events into one typed event model.
- **CHAT-008 — Terminal invariant.** Every accepted operation must reach exactly one terminal outcome: completed, failed, cancelled, or explicit waiting-user. Any terminal provider error must remove the spinning/thinking UI and render a useful recoverable error.
- **CHAT-009 — Concurrency isolation.** Two or more Agents may run concurrently without event misrouting, transcript contamination, draft loss, or one Agent’s lifecycle adopting another Agent’s event.
- **CHAT-010 — Reconnect/restart recovery.** Renderer reload, coordinator restart, Host restart, process crash, network reconnect, and app restart must recover from the durable journal/transcript without replaying accepted turns or losing terminal state.
- **CHAT-011 — Stop semantics.** Stop must cancel the canonical running operation and any provider attempt, settle UI state, and leave the transcript in a deterministic state.
- **CHAT-012 — Human-scale progress.** Long Agent tasks must surface typed progress/tool/activity transitions. “正在思考” alone for an extended interval is not an acceptable progress model.
- **CHAT-013 — Conversation fast lane.** Simple conversational turns must avoid unnecessary tool-schema/context overhead while preserving the same lifecycle guarantees. Classification must be tested against Chinese and English prompts and must not be required for correctness.

### Latency / streaming quality

- **PERF-001 — Local submit paint.** The submitted user bubble must paint within 250 ms p95 after the send gesture on the reference test machine.
- **PERF-002 — Acceptance visibility.** Canonical accepted/running state must become visible within 500 ms p95 after Host acceptance.
- **PERF-003 — TTFA/first output.** Under a healthy live provider and stable network, simple Q&A first assistant output should be <= 3 s p50 and <= 8 s p95, and must not regress by more than 10% versus the same provider/model exercised through the Grok reference control path when a direct comparison is possible.
- **PERF-004 — No hidden 180-second success criterion.** A test may use a larger outer watchdog to collect diagnostics, but user-facing latency acceptance must be scored from first-output and completion timing. “Completed sometime within 180 seconds” is not a responsiveness pass.
- **PERF-005 — Stream cadence.** Delta batching may coalesce renderer work, but visible stream updates must remain smooth and must not introduce a second artificial multi-second buffering layer.

### Plugins / connectors / MCP

- **CONN-001 — First-class Plugins surface.** “Plugins” must open a dedicated Plugins/Connectors/MCP product surface, not the Mini Apps compatibility surface.
- **CONN-002 — Catalog and installed views.** Users must be able to browse/search available integrations and inspect installed/enabled integrations.
- **CONN-003 — Install/update/remove.** Supported integrations must provide product-visible install, update/reinstall, disable, and remove flows with explicit status/error state.
- **CONN-004 — OAuth/auth.** OAuth/authentication must support start, callback/completion, failure, re-authentication, and disconnect. Credentials/tokens must never be exposed in renderer logs.
- **CONN-005 — Accounts.** Where an integration supports multiple accounts, the product must expose add/select/rename/remove account behavior comparable to the reference.
- **CONN-006 — Server/tool inventory.** MCP servers/connectors must expose connection state, available tools, tool count, and per-tool enable/disable when supported.
- **CONN-007 — Composer integration.** Enabled connectors/MCP references must be available from the composer through the reference-equivalent interaction, including mention/tool selection where applicable.
- **CONN-008 — Runtime execution.** Selecting a connector must affect the actual Agent tool set and tool execution path; a UI-only catalog is not completion.
- **CONN-009 — Remove non-reference Mini Apps surface.** The current Fabushi Mini Apps/Marketplace/WebMCP desktop surface must be removed unless the frozen Grok baseline contains an explicit counterpart. It must not survive as a separate compatibility/product architecture.
- **CONN-010 — Error and offline behavior.** Connector auth expiry, unavailable server, tool failure, lost network, and partial catalog failure must be recoverable without breaking ordinary chat.

### New / create interaction and visible parity

- **UI-001 — Reference capture before implementation.** Capture the target Grok interaction for sidebar New, all visible plus buttons, creation chooser, composer, Plugins, settings, Agent list, transcript, loading/thinking, errors, and connector flows at the exact reference build used for acceptance.
- **UI-002 — Observable reference wins over incomplete recovered source.** If a reference capture conflicts with the reconstructed 0.18 partial frontend source, the capture is normative for user-visible behavior and the discrepancy must be recorded.
- **UI-003 — Creation parity.** The Fabushi “New/+” flow must expose the same choices, ordering, keyboard/mouse behavior, result objects, focus behavior, and post-create navigation as the approved reference capture.
- **UI-004 — No dead affordances.** Every visible control in the parity surface must be connected to a real product action or explicitly disabled with an explanation. Placeholder controls are forbidden in a released package.
- **UI-005 — State fidelity.** idle, queued, preparing, thinking, streaming, tool-running, waiting-user, retrying, completed, failed, cancelled, offline, and reconnecting states must be distinguishable and driven by canonical runtime state.
- **UI-006 — Reference screenshots/video.** Pixel/layout parity is assessed against captured reference at 1671x937 dark mode/reduced-motion where applicable, with documented intentional Fabushi branding/product differences.

### Architecture / one-to-one Rust port

- **ARCH-001 — Frozen Grok source baseline.** All structural comparisons and port decisions use `b-nnett/grok-bot-0.18-reconstructed@a9f633e09d49a85829b8236331b9e21f7e612634`. A baseline change requires a spec update and a new complete mapping.
- **ARCH-002 — Complete source inventory.** Generate and check in a machine-readable port manifest covering every source-bearing Grok path. Each row records: Grok path, Grok blob SHA, kind, target Fabushi path, Rust module/crate, status, behavior/test evidence, and removal/replacement notes. No source file may be silently skipped.
- **ARCH-003 — Folder-tree parity.** The Fabushi canonical source tree must mirror Grok Bot’s relative folder hierarchy for application code. Major reference roots such as `frontend/`, `source/electron-main/`, `source/electron-preload/`, `source/host/`, `source/node-agent-coordinator/`, `source/shared/`, and `source/packages/` keep the same relative domain/subfolder structure. The historical name `node-agent-coordinator` is retained for structural parity even though its implementation is Rust.
- **ARCH-004 — File-by-file Rust counterpart.** Each executable `.ts/.tsx/.js/.mjs/.cjs` Grok module must map to a Rust implementation at the corresponding logical path/domain. The file stem and subfolder should remain the same where practical; when Rust module naming makes this impractical, the port manifest must record the exact 1:1 mapping.
- **ARCH-005 — Rust owns application logic.** Chat/runtime, submission journal, coordinator, host/runner, persistence, retry/deadline logic, Plugins/MCP, account/runtime orchestration, settings state machines, local execution orchestration, process lifecycle policy, and equivalent frontend state machines are implemented in Rust.
- **ARCH-006 — Minimal platform glue exception.** JavaScript/TypeScript may remain only where the platform itself requires it (for example Electron bootstrap/preload or generated WASM loader glue). Such files must be listed in an explicit allowlist, contain no independent product/business state machine, and delegate immediately to Rust-owned contracts. Any non-allowlisted JS/TS application logic is a release blocker.
- **ARCH-007 — Frontend Rust translation.** Recovered Grok frontend application logic must also be translated, not left as a separate Fabushi React architecture. The preferred target is Rust/WASM or another Rust-owned renderer state layer with only minimal browser/Electron glue. CSS/HTML/static assets may remain non-Rust where they are declarative resources.
- **ARCH-008 — Process-boundary parity.** Preserve the Grok responsibilities and communication topology between renderer/frontend, electron-main boundary, preload bridge, coordinator, host, runner, shared contracts, MCP/plugins, and local/native execution. Changing language to Rust must not collapse those responsibilities into a new monolith.
- **ARCH-009 — Dependency-direction parity.** The dependency graph must follow the Grok tree’s domain direction. Cross-folder imports/RPC dependencies that have no reference counterpart require an explicit spec exception; “convenient” Fabushi-only cross-coupling is forbidden.
- **ARCH-010 — State-ownership parity.** Submission, run identity, transcript/checkpoint, retry/cancel, connector state, account state, process lifecycle, and renderer projection ownership must follow the corresponding Grok module ownership. Renderer-side inference/repair used only by the old Fabushi architecture must be removed.
- **ARCH-011 — Remove non-Grok code.** Any shipping Fabushi module without a reference counterpart must be deleted after required data migration. Git history is the archive; a parallel “legacy” implementation is not allowed to remain enabled or compiled into the production desktop package.
- **ARCH-012 — Remove non-Grok product surfaces.** Mini Apps, Telegram/messaging compatibility UI, payments, calls, legacy compatibility shells, duplicate Agent runtimes, and other extra desktop surfaces must be removed unless the port manifest identifies an exact Grok Bot reference counterpart. A Fabushi-only feature cannot remain merely because it existed in an earlier release.
- **ARCH-013 — No shadow architecture.** After cutover there must be exactly one canonical implementation for each reference subsystem. Adapters may exist only during migration and must have a removal task and fail the final architecture gate if still on a shipping path.
- **ARCH-014 — Automated structural gate.** CI must compare the frozen Grok source inventory against the Fabushi Rust port manifest and canonical tree. Completion requires zero unmapped executable reference files and zero unauthorized extra shipping code modules.
- **ARCH-015 — Semantic port, not literal-copy dependency.** The Rust implementation must preserve behavior/contracts proven by reference code/artifacts while respecting provenance and licensing. If direct source reuse is not licensed, reimplement semantics from inspectable evidence instead of copying source text.
- **ARCH-016 — Migration safety.** User data needed by the retained Grok-equivalent product must be migrated before legacy trees are deleted. Data belonging only to removed Fabushi-only features may be exported/backed up, but the feature implementation itself does not remain in the shipping architecture.

### Power / process lifecycle

- **POWER-001 — Measure by process.** Capture CPU, memory, wakeups/timer activity where available, and network activity separately for Electron main, renderer, coordinator/Host, Rust sidecars, remote-device helper, and any browser/control helper.
- **POWER-002 — Demand-driven remote helper.** The remote-device Agent/WebSocket helper must not run unless remote computer presence/control is enabled and an authenticated session requires it.
- **POWER-003 — Idle suspension.** Provider/Agent runner work must quiesce after active turns settle. No retry, polling, animation, or event loop may continue at high frequency in idle state.
- **POWER-004 — Hidden-window policy.** Closing/hiding the main window must transition to an explicit low-power background profile. Only user-enabled background capabilities may remain active.
- **POWER-005 — Scoped event delivery.** Runtime events must be delivered only to subscribed/eligible renderer targets; blind fanout to every BrowserWindow is prohibited unless the event is truly global.
- **POWER-006 — Crash-loop backoff.** Long-lived helper restart loops must use bounded exponential backoff/circuit breaking; a five-second infinite respawn loop is not permitted.
- **POWER-007 — Renderer animation.** No permanent JavaScript animation loop may run solely for avatar/thinking decoration. Reduced-motion and hidden-window states must stop decorative animation work.
- **POWER-008 — Idle budget.** On the designated Apple Silicon/macOS reference machine after a five-minute warmup, 10-minute visible-idle process-tree average CPU must be <= 2.0%, hidden-idle average CPU with no enabled background computer-control feature must be <= 0.5%, and no remote-device helper process may exist in that disabled state. A comparable baseline must also be stored for regression tracking.
- **POWER-009 — macOS product check.** After 15 minutes of idle packaged-app use with background computer control disabled, Fabushi must not appear in macOS’s “Using Significant Energy” list during the acceptance capture. This is supplemental to numeric process metrics, not a replacement for them.

### Observability / release truth

- **OBS-001 — Stage timestamps.** Record localSubmitAt, acceptedAt, attemptStartedAt, firstOutputAt, firstTextAt, retryAt, toolStart/finish, terminalAt with operation/conversation/Agent correlation.
- **OBS-002 — Redaction.** Provider prompts/content may be omitted/redacted from telemetry; tokens, cookies, OAuth codes, passwords, and secrets must never be logged.
- **OBS-003 — Stuck-turn diagnostic.** A timeout/failure record must say which stage stalled and include retry count/provider route without exposing credentials.
- **OBS-004 — Packaged evidence.** Acceptance evidence must bind to exact source SHA and signed/notarized packaged executable, and include lifecycle trace, timings, screenshots/video, process metrics, connector trace, and logs.
- **OBS-005 — Existing parity statuses are revalidated.** No prior parity row may be cited as completion unless its corresponding requirement is re-run against the packaged candidate.

## 5. Current state

### 5.1 Current Fabushi data flow

```text
React renderer
  -> beginLocalTurn(requestId)
       -> optimistic user bubble
       -> synthetic assistant "正在思考" / provisional operationId=requestId
  -> coordinatorClient.send(requestId, Agent/conversation)
  -> preload / Electron IPC
  -> Electron MahayanaHostProcess (stdio JSON)
  -> Rust app-host/runtime
       -> accepted/queued
       -> spawned turn actor: preparing -> thinking
       -> provider.send_message(...).await
  -> model adapter
       -> Responses API: SSE stream
       -> Chat Completions: non-streaming JSON
       -> Anthropic Messages: non-streaming JSON
  -> Rust runtime events
  -> Electron broadcasts runtime event to all BrowserWindows
  -> renderer adopts real operationId / repairs early event correlation
  -> transcript projection
```

The weak points are the provisional operation identity, optimistic assistant state, asynchronous provider wait without a first-output attempt watchdog, non-streaming alternate provider routes, broadcast fanout, and lifecycle split across React/TypeScript/Electron/Rust.

### 5.2 Current connector flow

Fabushi has backend/runtime MCP contracts and an Agent MCP list/reference controller, but the main sidebar “Plugins” action opens the Mini Apps surface. Mini Apps can browse/install/open marketplace apps, but this does not expose the complete MCP/plugin auth/account/tool-management flow.

### 5.3 Current power lifecycle

Production starts the Host and remote-device supervisor at app ready. The main window is hidden on close and background persistence remains active. The remote-device supervisor can spawn a separate helper process and maintain a WebSocket presence. This lifecycle remains active independently of whether the user is interacting with chat.

### 5.4 Current packaged acceptance gap

`desktop/e2e/openbot-packaged-acceptance.spec.ts` does exercise a real packaged candidate, real lifecycle, multi-Agent isolation, and marker-bearing prompts. However it permits up to 180 seconds for a canonical completed assistant turn and focuses on completion/correlation rather than first-output latency, ordinary semantic chat, provider stall recovery, connector parity, or idle power. A green run therefore does not prove the user-reported scenarios are healthy.

## 6. Target state

### 6.1 Canonical repository shape

The shipping source tree follows the Grok Bot 0.18 source tree instead of the current Fabushi desktop layout. The exact mapping is generated from the frozen reference commit and checked by CI.

Representative shape:

```text
frontend/
  ... same Grok frontend domain/subfolder structure ...
source/
  electron-main/
    ... same Grok electron-main domain/subfolder structure, Rust-owned logic ...
  electron-preload/
    ... same Grok preload structure, minimal platform glue + Rust-owned contracts ...
  host/
    ... same Grok host/runner/extensions/services structure in Rust ...
  node-agent-coordinator/
    ... same Grok coordinator structure in Rust ...
  shared/
    ... same Grok shared-contract structure in Rust ...
  packages/
    ... same Grok package/domain structure in Rust ...
```

The directory name `node-agent-coordinator` is intentionally retained for structural parity; it does not imply the final implementation remains Node.

Legacy Fabushi roots such as `desktop/src`, `desktop/electron`, `frontend/apps/web`, and `third_party/mahayana` are migration sources only. They must not remain as parallel canonical architectures after the one-to-one port is complete.

### 6.2 Target runtime flow

```text
Rust-owned frontend state / renderer projection
  -> Rust submission journal
  -> Rust coordinator at Grok-equivalent source/node-agent-coordinator boundary
  -> Rust host/runner at Grok-equivalent source/host boundary
       -> provider attempt owner
       -> first-output watchdog
       -> bounded retry / checkpoint resume
       -> streaming provider adapter
       -> tools / MCP / plugins
       -> durable transcript/checkpoint
  -> canonical typed event stream
  -> Rust-owned frontend projection
```

Electron/browser-required JavaScript is a thin transport/bootstrap layer only and cannot own run state.

### 6.3 Target product surface

The visible desktop product contains the Grok-equivalent surfaces and flows proven by the reference: Agent roster/conversation, create/new flows, Plugins/connectors/MCP, settings/account, local execution/computer capabilities where the reference has them, and the corresponding overlays/dialogs.

Fabushi-only surfaces with no reference counterpart are removed rather than hidden behind compatibility menus.

### 6.4 Target power/process behavior

Process lifetime, background services, reconnect policy, coordinator/host startup, helper lifecycle, and event fanout follow the Grok-equivalent ownership model, implemented in Rust. No old Fabushi background supervisor may remain active unless it maps to an explicit reference subsystem.

## 7. Architecture and ownership boundaries

This section is defined by **reference folder ownership**, not by the current Fabushi implementation.

### `frontend/` — Rust-owned frontend/application state

Owns the user-visible state/projection corresponding to the Grok frontend tree:

- conversation/Agent UI state;
- create/new flows;
- Plugins/connectors UI state;
- composer/drafts/selection;
- transcript projection;
- settings/account presentation;
- approved reference interactions.

Business/run truth remains outside the renderer. Any required JS/WASM loader is glue only.

### `source/electron-main/` — Rust-owned desktop-main semantics

Owns the Grok-equivalent desktop-main responsibilities:

- window/process lifecycle policy;
- application menu/update/deep-link behavior;
- account/native adapters;
- main-side MCP/OAuth bridge;
- starting/stopping coordinator, host, and native helpers;
- secure transport to preload/renderer.

A tiny Electron bootstrap may remain JavaScript only as an allowlisted platform adapter.

### `source/electron-preload/` — minimal bridge

Owns no business state. It exposes the smallest safe typed bridge required by Electron and delegates to Rust-owned IPC/RPC contracts.

### `source/node-agent-coordinator/` — Rust coordinator

Owns the Grok-equivalent:

- renderer connection lifecycle;
- request/RPC multiplexing;
- Agent/conversation registry;
- submission journal coordination;
- reconnect/resync;
- routed MCP bridge;
- event subscription/scoping.

### `source/host/` — Rust host/runner

Owns the Grok-equivalent:

- provider selection and streaming attempt lifecycle;
- first-output/overall deadlines;
- safe retry/backoff/checkpoint resume;
- transcript/checkpoint persistence;
- tools and MCP invocation;
- canonical lifecycle events;
- cancellation and terminal settlement;
- Host extensions/services.

### `source/shared/`

Contains Rust shared schemas/contracts corresponding one-to-one to Grok shared modules. It must not become a dumping ground for Fabushi-only abstractions.

### `source/packages/`

Contains Rust translations of Grok package modules with the same package/domain segmentation. Each package must have an explicit manifest mapping to its reference source path(s).

### Native/platform capability implementations

Native OS/computer-control code may be Rust, but it must be placed behind the Grok-equivalent module boundary. A separate Fabushi-specific native architecture outside the mirrored tree is not allowed without an explicit reference mapping.

### Removed legacy ownership

After the final cutover:

- `desktop/src/**` is not a canonical product runtime;
- `desktop/electron/**` is not a parallel Electron architecture;
- `frontend/apps/web/**` is not a parallel desktop runtime;
- `third_party/mahayana/**` is not a parallel Agent engine;
- compatibility adapters for Fabushi-only surfaces are deleted if they have no Grok counterpart.

Git history provides rollback provenance; shipping duplicate architectures do not.

## 8. Interfaces / contracts / schemas / data flow

### 8.1 Submission contract

The Rust port keeps the Grok-equivalent separation between client nonce/request identity and canonical operation identity.

```rust
struct SubmitTurn {
    client_nonce: String,
    request_id: String,
    agent_id: String,
    conversation_id: String,
    text: String,
    attachments: Vec<AttachmentRef>,
    references: Vec<AgentOrConnectorRef>,
}

struct TurnAccepted {
    request_id: String,
    client_nonce: String,
    operation_id: String,
    turn_id: String,
    run_id: String,
    accepted_at_ms: u64,
}
```

`client_nonce` is the idempotency key. `request_id` scopes one RPC. `operation_id` is created only by the canonical runtime owner.

### 8.2 Lifecycle envelope

Every runtime event used for a user-visible turn must carry the canonical correlation fields:

```rust
struct TurnEnvelope {
    agent_id: String,
    conversation_id: String,
    operation_id: String,
    turn_id: String,
    run_id: String,
    sequence: u64,
    occurred_at_ms: u64,
}
```

Event kinds include accepted/queued, preparing, thinking, first-output, text delta, reasoning/progress, tool started/completed/failed, retrying, waiting-user, completed, failed, and cancelled.

The frontend must reject stale generation/sequence events deterministically and must never guess the target Agent from “only pending peer”.

### 8.3 Provider attempt contract

The Rust translation of the Grok runner exposes the same responsibilities as Grok’s `createStreamAttempt` / retry modules:

- cancelable context;
- first-output deadline;
- stream-output-produced flag;
- resumable checkpoint;
- bounded retry policy;
- server-paced retry support;
- explicit retry/exhausted outcome;
- final-state persistence.

The port manifest links each Rust runner module to its exact Grok source file(s).

### 8.4 Plugins/MCP contract

The Rust Plugins/MCP subsystem exposes typed operations equivalent to the Grok reference for:

- catalog search/list;
- installed list;
- install/update/remove/enable/disable;
- auth start/callback/status/disconnect;
- accounts list/add/rename/remove/select;
- MCP server status/refresh;
- tools list/enable/disable;
- reference/tool selection and actual tool execution for a turn.

No separate Mini Apps contract remains unless the frozen Grok baseline contains an equivalent subsystem.

### 8.5 Port-manifest contract

A machine-readable manifest is mandatory. Minimum fields:

```text
reference_path
reference_blob_sha
reference_kind
target_path
target_crate_or_module
status
behavioral_evidence
test_evidence
notes
```

Allowed final statuses are `ported`, `not-applicable-noncode`, and `removed-extra`. `pending` or `compatibility-only` blocks completion.

## 9. Constraints and non-functional requirements

### Performance / latency

Performance is measured from the user gesture. Rust translation is not accepted merely because it compiles or uses less CPU; it must reproduce Grok-equivalent responsiveness, streaming cadence, cancellation, and failure recovery.

### Power / CPU / memory

Power gates are measured on the whole process tree. The Rust port must not preserve old Fabushi always-on helpers simply because they already exist. Background processes/services remain only when the Grok architecture has a corresponding responsibility and the feature is active.

### Structural fidelity

- source-bearing folder hierarchy mirrors the frozen Grok tree;
- every executable reference module is mapped;
- unauthorized extra shipping modules are forbidden;
- old Fabushi trees cannot remain as a parallel implementation;
- the Rust translation may change syntax/language, but not silently change module responsibility.

### Security / privacy

- preserve Electron/browser sandbox and isolation requirements;
- OAuth/tokens remain outside renderer-visible state and logs;
- connector tool execution remains policy/approval controlled;
- Rust FFI/WASM/platform boundaries must be memory-safe and narrowly exposed;
- unavoidable JS/preload glue must expose only allowlisted methods.

### Compatibility and deletion

Compatibility exists only to migrate data into the new Grok-shaped Rust architecture. It is not a reason to keep old product features.

Before removing a Fabushi-only subsystem:

1. identify whether any user data must be exported or migrated;
2. provide a deterministic migration/backup path where needed;
3. remove the code, navigation, background service, storage writer, tests, and package dependencies for that subsystem;
4. verify no shipping binary imports or starts it.

### Reliability

No accepted turn may be orphaned by renderer reload, coordinator/Host restart, provider timeout, OAuth expiry, or transient network failure. The corresponding recovery behavior must map back to the reference Grok module/state machine.

### Provenance / rights

The Grok reconstruction’s provenance explicitly states that no upstream source-code license is implied. The one-to-one requirement is therefore a **traceable semantic Rust port**, not permission to reproduce unlicensed source text or binaries. Public redistribution of any directly reused material requires a separate rights review.

## 10. Failure modes and edge cases

The implementation and tests must cover:

- provider accepts TCP/HTTP but never sends first token;
- provider disconnects before first token;
- provider disconnects after partial output;
- rate limit / capacity / Retry-After;
- malformed SSE/JSON;
- OAuth expired/revoked;
- MCP server unavailable during a turn;
- Host/coordinator crashes while a turn is queued;
- Host/coordinator crashes after acceptance but before first output;
- renderer reload during stream;
- duplicate submit after reconnect;
- two concurrent Agents with events arriving out of order;
- user stops during retry, tool execution, or waiting-user;
- app hides/closes while idle and while a turn is active;
- remote-device helper crashes repeatedly;
- background computer-control disabled while helper is connected;
- account logout/switch while connector/provider/remote helper is active;
- reference-capture behavior differs from reconstructed source.

## 11. Implementation strategy

### Phase 0 — Freeze and enumerate Grok 0.18

Generate the complete source inventory from `a9f633e09d49a85829b8236331b9e21f7e612634`. Establish the port manifest before implementation. Every source-bearing Grok file must have a row; every current Fabushi shipping module must be classified as mapped-to-Grok or extra-to-remove.

### Phase 1 — Create the mirrored Rust tree

Create the Grok-shaped canonical folders first:

- `frontend/**`
- `source/electron-main/**`
- `source/electron-preload/**`
- `source/host/**`
- `source/node-agent-coordinator/**`
- `source/shared/**`
- `source/packages/**`

Populate compileable Rust module/crate skeletons following the exact reference subdivision. Do not invent a new Fabushi hierarchy.

### Phase 2 — Translate shared contracts and coordinator semantics

Port Grok shared schemas, RPC/event contracts, submission journal, coordinator client/server behavior, reconnect/resync, request rejection, routing, and event subscription logic into Rust, file by file.

No current Fabushi provisional-operation-id or “only pending peer” repair logic may survive unless a Grok counterpart exists.

### Phase 3 — Translate Host / runner / transcript / provider stack

Port the Grok `source/host/**` tree module by module, including:

- send pipeline;
- stream attempt;
- first-token stall policy;
- transient/provider error classification;
- retry/backoff/checkpoint behavior;
- transcript persistence;
- runner lifecycle;
- provider streaming;
- tool execution;
- waiting-user/cancel/terminal state.

This phase replaces the old Mahayana desktop Agent engine as canonical runtime ownership.

### Phase 4 — Translate Plugins/MCP and desktop-main/preload

Port Grok MCP/plugin/account/OAuth/catalog/runtime modules and the corresponding Electron-main/preload contracts into Rust. Keep only minimal platform-required JS glue.

### Phase 5 — Translate frontend/product behavior

Port the recovered Grok frontend tree and state machines into Rust-owned renderer logic, preserving the same subfolder organization and approved observed behavior. Where reconstructed frontend source is incomplete, use the immutable reference artifact/captures as the behavioral source and document the gap in the manifest.

### Phase 6 — Delete everything without a Grok counterpart

After each mapped subsystem has a passing Rust replacement, remove the old Fabushi implementation. Before final acceptance, delete all unmatched shipping code and product surfaces, including any Mini Apps, messaging/Telegram, payments, calls, legacy compatibility shells, duplicate Agent runtime, or background service that lacks a reference mapping.

No compatibility adapter may remain in the final shipping path simply to preserve the previous architecture.

### Phase 7 — Structural, performance, power, and behavioral convergence

Run the tree-parity checker, module manifest checker, JS/TS logic allowlist checker, latency suite, live chat suite, connector suite, process-tree power suite, and side-by-side UI reference capture.

### Phase 8 — GitHub Actions, packaged acceptance, merge, release

Only after the **entire one-to-one Rust port and removal pass is complete** should the release validation run through GitHub Actions. Fix any failure at the exact candidate HEAD. Merge only after all structural/behavioral/power gates pass, then independently verify the canonical-main signed release and published assets.

## 12. Verification / test strategy

### Structural / port-completeness gates

CI must fail if any of the following is true:

- a source-bearing Grok file has no port-manifest row;
- an executable Grok module is not `ported`;
- a shipping Fabushi code module has no Grok counterpart and is not explicitly allowlisted as platform bootstrap/configuration;
- the canonical folder hierarchy diverges from the frozen Grok tree without an approved spec exception;
- old `desktop/src`, `desktop/electron`, `frontend/apps/web`, `third_party/mahayana`, or another legacy tree is still compiled/imported as a parallel runtime after final cutover;
- non-allowlisted JS/TS application logic remains;
- a temporary compatibility adapter remains on the production code path.

### Rust compile / dependency gates

- all mirrored Rust crates/modules compile on supported desktop platforms;
- circular/cross-domain dependencies not present in the Grok architecture are rejected;
- Rust module ownership is checked against the manifest;
- required WASM/native/Electron glue is generated or allowlisted and contains no independent business state.

### Behavioral unit / contract gates

For each mapped Grok module, tests cover the corresponding observable semantics. At minimum:

- submission journal duplicate/offline/reconnect/stale-generation behavior;
- operation correlation and sequence fencing;
- first-output watchdog;
- retry eligibility after zero/partial output;
- server Retry-After/backoff;
- cancellation during every phase;
- streaming parsers for supported providers;
- connector auth/account/tool state machines;
- coordinator disconnect/reconnect;
- helper lifecycle/circuit breaker;
- transcript/checkpoint recovery.

### Simulated integration faults

Use a controllable provider test server to produce:

- immediate streaming;
- delayed first token;
- no first token;
- disconnect before first token;
- partial stream then disconnect;
- 429/503 + Retry-After;
- malformed stream;
- successful retry with checkpoint.

Assertions cover both visible UI behavior and canonical persisted state.

### Packaged live-provider acceptance

The signed package must exercise ordinary prompts, not only marker probes:

1. simple Chinese Q&A;
2. simple English Q&A;
3. “创建一个打地鼠的小程序” or an equivalent real tool-capable task **only if the approved Grok reference supports the same creation capability**;
4. two concurrent Agent turns;
5. stop/cancel then new turn;
6. transient provider/network recovery;
7. renderer reload/reopen during a turn;
8. authenticated connector/MCP catalog -> select -> tool execution;
9. the approved reference New/+ creation journey.

Marker prompts may remain diagnostics but cannot be the only proof.

### Removal acceptance

For every current Fabushi subsystem classified `extra-to-remove`, acceptance must prove:

- source removed;
- navigation/control removed;
- background process/service removed;
- package dependency removed when no longer used;
- storage migration/export completed if required;
- packaged app contains no runtime entrypoint for it.

### Latency acceptance

Record at least 20 simple live turns and report p50/p95 local-paint, acceptance, first-output, first-text, and completion. Reaching a 180-second outer watchdog is a latency failure even if the turn eventually completes.

### Power acceptance

On the packaged candidate:

- record the full process tree before interaction;
- warm for five minutes;
- sample visible idle for ten minutes;
- sample hidden idle for ten minutes;
- verify no unmatched Fabushi-only helper/service remains alive;
- attach CPU/memory/process/network samples and an Activity Monitor/battery-menu capture.

## 13. Acceptance criteria / Definition of Done

- **AC-01:** The exact packaged candidate answers all required ordinary live-provider prompts with canonical completed/failed terminal state; none remains indefinitely at “正在思考”.
- **AC-02:** No renderer-generated synthetic assistant “thinking” event exists before canonical runtime acceptance.
- **AC-03:** Concurrent Agent acceptance/stream/final events remain isolated without fallback target guessing.
- **AC-04:** A provider that emits no first output triggers the Grok-equivalent watchdog and bounded retry/terminal failure.
- **AC-05:** Supported providers stream partial output with Grok-equivalent attempt lifecycle.
- **AC-06:** Latency artifacts meet PERF-001 through PERF-005 and show p50/p95, not only eventual completion.
- **AC-07:** Plugins/connectors/MCP reproduce the mapped Grok catalog/auth/account/server/tool behavior in the packaged app.
- **AC-08:** The approved New/+ reference journey is reproduced and verified by side-by-side video/screenshots.
- **AC-09:** The complete frozen Grok source inventory has a port-manifest row; there are zero silently skipped executable source files.
- **AC-10:** Every executable Grok application module is represented by a Rust counterpart or an explicitly justified non-code/platform exception; final manifest has no `pending` or `compatibility-only` rows.
- **AC-11:** The canonical Fabushi source folder hierarchy mirrors the Grok source hierarchy and passes the automated tree-parity gate.
- **AC-12:** There are zero unauthorized extra shipping Fabushi code modules/surfaces without a Grok counterpart.
- **AC-13:** Legacy Fabushi parallel runtimes/compatibility shells are deleted from the production path; there is exactly one canonical implementation per reference subsystem.
- **AC-14:** Non-allowlisted JS/TS application logic is absent. Remaining JS/TS is only platform-required bootstrap/generated glue with no product state machine.
- **AC-15:** Rust process/module boundaries reproduce Grok’s renderer/main/preload/coordinator/host/shared/packages ownership rather than collapsing into a different monolith.
- **AC-16:** With inactive background features, the packaged app satisfies POWER-008 and the macOS supplemental energy check in POWER-009.
- **AC-17:** No credentials/secrets appear in frontend state, logs, traces, or evidence bundles.
- **AC-18:** Required retained user data survives migration; data for removed Fabushi-only features has an explicit export/backup decision.
- **AC-19:** Exact-HEAD GitHub Actions tests, signed packaged acceptance, merge SHA, and canonical-main release are all separately recorded. A green PR without canonical-main release evidence is not release completion.
- **AC-20:** A final diff report lists every Grok reference source file and its Rust target, plus every deleted Fabushi-only source path, so reviewers can verify the port was actually performed one by one.

## 14. Release / migration / rollback

Implementation must be delivered from the canonical main containing this spec.

Migration order:

1. freeze reference inventory and create the one-to-one port manifest;
2. create the mirrored Rust folder/crate skeleton;
3. translate shared contracts and coordinator;
4. translate Host/runner/transcript/provider stack;
5. translate Plugins/MCP/electron-main/preload;
6. translate frontend/product state;
7. migrate/export required retained user data;
8. delete every Fabushi-only/unmapped shipping subsystem and old parallel runtime tree;
9. run structural parity, behavioral, performance, power, signed packaged acceptance;
10. merge;
11. verify canonical-main release and published assets.

Rollback must operate at a release boundary. Git history and tagged artifacts provide code rollback; the production binary must not ship both old and new architectures just to make rollback easier.

No release is allowed while:

- the port manifest is incomplete;
- any executable Grok module remains unported;
- unauthorized extra Fabushi shipping modules remain;
- a legacy compatibility runtime is still required for ordinary operation;
- the folder-tree parity gate fails.

Release gate order remains: implementation complete -> GitHub Actions tests -> exact-HEAD signed packaged acceptance -> merge -> canonical-main release workflow -> published artifact verification.

## 15. Observability / evidence

Required evidence bundle:

- exact source SHA and package version;
- process tree and binary paths;
- stage-timing JSON for every acceptance turn;
- typed lifecycle trace with canonical ids;
- provider route/retry/stall diagnostics with secrets redacted;
- connector catalog/auth/tool trace;
- side-by-side Grok/Fabushi reference captures;
- 1671x937 dark/reduced-motion screenshots/video where applicable;
- visible-idle and hidden-idle process metrics;
- macOS energy supplemental capture;
- GitHub Actions run/job ids and artifact checksums.

The evidence must make it possible to answer “where did this turn spend time?” without reading private model chain-of-thought or exposing credentials.

## 16. References / provenance

### Fabushi exact baseline

- `bhrumom/fabushi-desktop@20644cf5aa2f5777cc350ece32edeb0cb90f88d7`
- `AGENTS.md`
- `docs/specs/SPEC_TEMPLATE.md`
- `projects/grok-fabu-parity/PARITY.md`
- `projects/desktop-chat-streaming-parity/SOURCE_OF_TRUTH.md`
- `docs/specs/post-1.2.74-agent-architecture-cutover.md`
- `desktop/src/agent-workspace/use-agent-workspace-runtime.ts`
- `desktop/src/agent-workspace/agent-runtime-coordinator.ts`
- `desktop/src/agent-workspace/coordinator-client.ts`
- `desktop/src/agent-workspace/use-agent-mcp-controller.ts`
- `desktop/src/agent-workspace/agent-root-shell.tsx`
- `desktop/src/agent-workspace/agent-composer.tsx`
- `desktop/src/features/miniapps/miniapp-compatibility-adapter.tsx`
- `desktop/electron/main.cjs`
- `desktop/electron/host-process.cjs`
- `desktop/electron/remote-device-agent-supervisor.cjs`
- `third_party/mahayana/mahayana-rs/mahayana-runtime/src/lib.rs`
- `third_party/mahayana/mahayana-rs/mahayana-model/src/responses.rs`
- `third_party/mahayana/mahayana-rs/mahayana-native-engine/src/lib.rs`
- `desktop/e2e/openbot-packaged-acceptance.spec.ts`

### Grok Bot exact reference baseline

- `b-nnett/grok-bot-0.18-reconstructed@a9f633e09d49a85829b8236331b9e21f7e612634`
- `README.md`
- `PROVENANCE.md`
- `frontend/src/production/ProductionRenderer.tsx`
- `frontend/src/production/coordinator-client.ts`
- `frontend/src/recovered/features/conversation/workspace/sidebar.tsx`
- `frontend/src/recovered/features/conversation/workspace/composer.tsx`
- `frontend/src/recovered/features/conversation/workspace/submission.ts`
- `frontend/src/recovered/features/plugins/overlay/desktop-surface.tsx`
- `source/host/extensions/transcript/send-pipeline.ts`
- `source/host/runner/stream-attempt.ts`
- `source/host/runner/transient-stream-error.ts`
- `source/electron-main/mcp/desktop-mcp-manager.ts`
- `source/electron-main/mcp/mcp-runtime.ts`
- `source/electron-main/mcp/mcp-oauth-loopback-provider.ts`
- `source/node-agent-coordinator/routed-mcp-bridge.ts` (or the corresponding routed MCP bridge in the exact tree)

### User evidence

2026-09-22 screenshots show:

- a normal Mahayana turn “创建一个打地鼠的小程序” remaining at “正在思考”;
- deterministic PR7/BENCH marker turns completing in another Agent while a later turn again remains in thinking;
- macOS reporting the packaged Fabushi process under significant energy use.

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| CHAT-001..013 | blocked | Implementation not started under this spec. |
| PERF-001..005 | blocked | Requires exact packaged live-provider timing evidence. |
| CONN-001..010 | blocked | Current Plugins action routes to Mini Apps; full parity not implemented. |
| UI-001..006 | blocked | Reference capture set not yet attached. |
| ARCH-001..016 | blocked | Current Fabushi tree is not yet a one-to-one Rust mirror of the frozen Grok source tree; manifest, translation, deletion, and structural gates remain incomplete. |
| POWER-001..009 | blocked | Process-level measurement and demand-driven lifecycle work not yet completed. |
| OBS-001..005 | blocked | Required parity evidence bundle not yet produced. |
| AC-01..15 | blocked | This document defines the recovery gate; no implementation completion is claimed. |

Allowed statuses: `passed`, `blocked`, `not-applicable`.

Implementation must update this compliance table with exact commit/workflow/artifact evidence before any claim that Grok parity is complete.
