# Grok Bot 0.18 Runtime, Product, Connector, and Power Parity Recovery — Specification

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

Deliver a Fabushi desktop Agent experience that matches the Grok Bot reference in observable interaction quality and control-plane architecture for chat, streaming, Agent lifecycle, creation, Plugins/connectors/MCP, and failure recovery, while retaining Fabushi-specific identity, account system, local-computer control, Mini Apps, and Mahayana capabilities.

The finished product must:

- answer ordinary questions reliably and promptly;
- stream visible output/progress rather than sitting in a synthetic thinking state;
- terminate or recover every accepted turn;
- expose Plugins/connectors/MCP as first-class product functionality;
- reproduce the reference creation flow from captured evidence;
- materially reduce idle/background energy consumption;
- use the same architectural pattern as the reference where doing so is necessary for parity: TypeScript/Node for the desktop orchestration/control plane and Rust only behind narrow native/capability boundaries.

## 3. Non-goals / out of scope

- Removing Rust from the product merely because a TypeScript implementation exists. Rust remains appropriate for native OS integration, local-computer control, security-sensitive/native capabilities, and compute-heavy code.
- Copying proprietary compiled code, assets, branding, or other material without a license/right to do so. Parity means behavior, architecture, interaction, and measurable quality unless a source asset is licensed for reuse.
- Replacing Fabushi account, messaging, Mini Apps, payments, or computer-control product identity with Grok branding.
- Treating a deterministic “reply exactly X” probe as proof of ordinary user-chat correctness.
- Declaring parity from unit tests or renderer mocks without signed packaged-app evidence.
- Expanding this desktop recovery into iOS/Android work.

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
- **CONN-009 — Mini Apps remain distinct.** Mini Apps/Marketplace/WebMCP remain supported but must not be mislabeled or used as the implementation of the Plugins/Connector surface.
- **CONN-010 — Error and offline behavior.** Connector auth expiry, unavailable server, tool failure, lost network, and partial catalog failure must be recoverable without breaking ordinary chat.

### New / create interaction and visible parity

- **UI-001 — Reference capture before implementation.** Capture the target Grok interaction for sidebar New, all visible plus buttons, creation chooser, composer, Plugins, settings, Agent list, transcript, loading/thinking, errors, and connector flows at the exact reference build used for acceptance.
- **UI-002 — Observable reference wins over incomplete recovered source.** If a reference capture conflicts with the reconstructed 0.18 partial frontend source, the capture is normative for user-visible behavior and the discrepancy must be recorded.
- **UI-003 — Creation parity.** The Fabushi “New/+” flow must expose the same choices, ordering, keyboard/mouse behavior, result objects, focus behavior, and post-create navigation as the approved reference capture.
- **UI-004 — No dead affordances.** Every visible control in the parity surface must be connected to a real product action or explicitly disabled with an explanation. Placeholder controls are forbidden in a released package.
- **UI-005 — State fidelity.** idle, queued, preparing, thinking, streaming, tool-running, waiting-user, retrying, completed, failed, cancelled, offline, and reconnecting states must be distinguishable and driven by canonical runtime state.
- **UI-006 — Reference screenshots/video.** Pixel/layout parity is assessed against captured reference at 1671x937 dark mode/reduced-motion where applicable, with documented intentional Fabushi branding/product differences.

### Architecture

- **ARCH-001 — TypeScript/Node desktop control plane.** To minimize semantic drift from Grok, the canonical desktop orchestration plane for submission journaling, coordinator RPC, conversation/run lifecycle, provider-attempt policy, transcript projection, Plugins/MCP orchestration, and desktop state machines must be TypeScript/Node.
- **ARCH-002 — React/TypeScript renderer.** User-visible Agent/Plugins/creation surfaces remain React/TypeScript and consume typed events/state; renderer code must not own durable run truth.
- **ARCH-003 — Narrow Rust boundary.** Rust may own native OS integration, local-computer control, sandbox/native execution, security-sensitive/native helpers, and compute-heavy capabilities. Rust must not be the only owner of desktop-visible first-token timeout/retry semantics or require the renderer to infer lifecycle from Rust-internal state.
- **ARCH-004 — Single coordinator transport.** Renderer <-> preload <-> coordinator uses a single typed MessagePort/RPC channel with explicit ready/disconnect/reconnect semantics and pending-request rejection on disconnect.
- **ARCH-005 — Host runner boundary.** Provider attempts and tool-run orchestration use a dedicated Host/runner layer, separate from Electron window lifecycle and from React components.
- **ARCH-006 — Durable transcript boundary.** Canonical transcript/run state must have one persistence owner. Renderer localStorage may cache UI preferences/drafts but must not become run-state authority.
- **ARCH-007 — No compatibility shell ownership inversion.** Mini Apps, contacts, Telegram, payments, and calls may remain compatibility/features, but they must not own or wrap the canonical Agent runtime.
- **ARCH-008 — Migration safety.** The TypeScript control-plane cutover must be incremental behind explicit adapters/feature gates until packaged acceptance is green; no “big bang” rewrite may discard recoverability.

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

### 6.1 Target desktop data flow

```text
React/TypeScript Renderer
  -> SubmissionJournal.enqueue(clientNonce, agentId, conversationId, payload)
       -> optimistic user message only
  -> Coordinator RPC (MessagePort)
       -> durable acceptance assigns operationId
       -> canonical state event: queued/preparing
  -> TypeScript/Node Host Runner
       -> attempt owner
       -> first-output watchdog
       -> transient/capacity classification
       -> bounded retry / resume checkpoint
       -> streaming provider adapter
       -> typed tool/MCP execution
       -> durable transcript/checkpoint persistence
  -> canonical typed event stream
       -> {agentId, conversationId, operationId, turnId, runId, sequence}
  -> subscribed Renderer
       -> coalesced paint only
       -> no inference/correlation guesses
  -> optional Rust native services
       -> computer control / native OS / sandbox / security capability
       -> explicit typed RPC, not desktop run-state ownership
```

The first visible assistant state after submit is a canonical accepted/preparing state. “Thinking” is only shown after the real run reports it. First output, retry, waiting-user, terminal state, and errors are all generated by the Host runner.

### 6.2 Target connector state

Plugins/Connectors/MCP is a dedicated first-class surface with catalog, installed list, auth/account management, server/tool inventory and control, and composer/runtime integration. Mini Apps remains a separate surface.

### 6.3 Target power state

The default idle profile runs only the processes needed to keep the visible app responsive. Remote-control/background presence is opt-in/demand-driven. Closing/hiding the app enters a measured low-power profile. Long-lived helpers have lifecycle ownership and bounded restart policy.

## 7. Architecture and ownership boundaries

### Renderer — React/TypeScript

Owns:

- visual state and interaction;
- draft text, selection, ephemeral menu/dialog state;
- rendering normalized transcript/run events;
- presentation batching.

Must not own:

- durable send acceptance;
- operation identity allocation;
- provider retries/timeouts;
- canonical transcript/run state;
- connector credentials.

### Coordinator — TypeScript/Node

Owns:

- renderer MessagePort lifecycle;
- Agent/conversation registry;
- submission journal and client-nonce deduplication;
- durable acceptance routing;
- reconnect/resync protocol;
- event subscription/scoping.

### Host / runner — TypeScript/Node

Owns:

- provider selection and attempt lifecycle;
- first-output/overall deadlines;
- safe retry and checkpoint resume;
- tool/MCP invocation orchestration;
- transcript/checkpoint persistence;
- canonical lifecycle events;
- cancellation and terminal settlement.

### Electron main

Owns:

- process/window lifecycle;
- secure preload/IPC bootstrap;
- native menu/tray/update integration;
- starting/stopping coordinator/Host/native services according to explicit lifecycle policy.

Must not synthesize chat results or become transcript authority.

### Rust services

Allowed ownership:

- local-computer control and native input/capture;
- platform/native security adapters;
- sandbox/native execution primitives;
- high-performance/native-only capabilities.

The desktop Agent control plane must not require renderer-side provisional operation ids or special-case event repair because a Rust actor owns opaque lifecycle state.

### Plugins/MCP

A single TypeScript product service owns catalog/install/auth/account/server/tool state. Renderer receives normalized projections. Raw credentials remain in secure main/Host storage.

## 8. Interfaces / contracts / schemas / data flow

### 8.1 Submission contract

```ts
type SubmitTurn = {
  clientNonce: string;
  requestId: string;
  agentId: string;
  conversationId: string;
  text: string;
  attachments: AttachmentRef[];
  references: AgentOrConnectorRef[];
};

type TurnAccepted = {
  requestId: string;
  clientNonce: string;
  operationId: string;
  turnId: string;
  runId: string;
  acceptedAtMs: number;
};
```

`clientNonce` is the idempotency key. `requestId` scopes one RPC. `operationId` is created only by the canonical runtime owner.

### 8.2 Lifecycle envelope

Every runtime event used for a user-visible turn must include:

```ts
type TurnEnvelope = {
  agentId: string;
  conversationId: string;
  operationId: string;
  turnId: string;
  runId: string;
  sequence: number;
  occurredAtMs: number;
};
```

Event kinds include accepted/queued, preparing, thinking, first-output, text delta, reasoning/progress, tool started/completed/failed, retrying, waiting-user, completed, failed, cancelled.

The renderer must reject stale generation/sequence events deterministically and must never guess the target Agent from “only pending peer”.

### 8.3 Provider attempt contract

The runner exposes an attempt abstraction equivalent in responsibility to Grok’s `createStreamAttempt`:

- cancelable context;
- first-output deadline;
- stream-output-produced flag;
- resumable checkpoint;
- bounded retry policy;
- server-paced retry support;
- explicit retry/exhausted outcome;
- final-state persistence.

### 8.4 Connector contract

The TypeScript Plugins service exposes typed operations for:

- catalog search/list;
- installed list;
- install/update/remove/enable/disable;
- auth start/callback/status/disconnect;
- accounts list/add/rename/remove/select;
- MCP server status/refresh;
- tools list/enable/disable;
- reference/tool selection for a turn.

Mini Apps contracts remain separate.

## 9. Constraints and non-functional requirements

### Performance / latency

Performance is measured from the user gesture, not from an internal Rust state transition. TTFA and time-to-terminal are stored as test artifacts. Slow providers may exceed normal targets, but they must produce explicit retry/progress/error state instead of an indefinite spinner.

### Power / CPU / memory

Power gates are measured on the whole process tree. Disabling a feature must stop the processes/network loops dedicated to that feature. Background services must be demand-driven and must have idle/suspend semantics.

### Security / privacy

- preserve Electron context isolation, sandboxing, web security, and no renderer Node integration;
- OAuth/tokens remain outside renderer state and logs;
- Mini App/WebMCP iframe security boundaries are unchanged unless separately specified;
- connector tool execution remains policy/approval controlled.

### Compatibility

During migration, existing Fabushi accounts, Agent identities, conversations, Mini Apps, settings, and computer-control configuration must remain readable. Any storage/schema change requires an explicit forward migration and rollback-safe compatibility plan.

### Reliability

No accepted turn may be orphaned by renderer reload, Host restart, provider timeout, OAuth expiry, or transient network failure.

### Provenance

Reference code/behavior may be studied and independently implemented. No unlicensed binary/assets may be copied into Fabushi solely to achieve parity.

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

### Phase 0 — Evidence and instrumentation

Before behavior changes, add stage timing and process-tree instrumentation, reproduce the reported normal prompts on the exact 1.2.75 package, and capture the approved Grok UI/create/plugin reference. Preserve the failing lifecycle/log trace.

### Phase 1 — Stop indefinite thinking

Introduce a canonical provider-attempt watchdog/retry policy and terminal invariant. Remove the renderer’s synthetic “thinking” semantics: optimistic user paint remains, but assistant state is driven by canonical accepted/runtime events.

Enable actual streaming for provider routes that currently buffer full responses. Ensure all provider errors settle the operation.

### Phase 2 — Canonical ID and send-journal cutover

Port the Grok-style submission journal and MessagePort coordinator semantics into the TypeScript/Node control plane. Separate `clientNonce`, request id, and operation id. Remove “only pending peer” correlation as a correctness mechanism.

### Phase 3 — TypeScript Host/runner cutover

Move provider-attempt lifecycle, retry/deadline, transcript/checkpoint orchestration, and desktop-visible Agent run-state ownership into the TypeScript/Node Host/runner. Keep Rust behind typed capability RPCs. Migrate incrementally with compatibility adapters until package acceptance proves equivalence.

### Phase 4 — Plugins/connectors/MCP parity

Build the dedicated Plugins surface and wire catalog, auth/accounts, server/tool state, composer selection, and actual Agent execution. Keep Mini Apps in their own navigation/surface.

### Phase 5 — New/create and visual parity

Implement the approved captured New/+ creation behavior and close remaining transcript/sidebar/composer/loading/error interaction gaps. Do not guess from incomplete recovered source.

### Phase 6 — Power lifecycle

Make remote-device/background services demand-driven, introduce low-power hidden-window profile, scope runtime subscriptions, and add bounded helper restart. Profile before/after each process to avoid moving the cost rather than removing it.

### Phase 7 — Packaged acceptance and release

Only after all implementation phases are complete, run static/unit/contract/integration tests and then signed packaged acceptance through GitHub Actions. Fix failures at the exact candidate HEAD. Merge only a fully evidenced candidate, then verify the canonical-main release artifact separately.

## 12. Verification / test strategy

### Static / architecture

- architecture checker rejects renderer-owned durable operation state;
- rejects request id used as provisional operation id;
- rejects Plugins navigation to Mini Apps;
- rejects direct connector credentials in renderer;
- enforces allowed TypeScript/Rust dependency boundaries.

### Unit / contract

- submission journal duplicate/offline/reconnect/stale-generation behavior;
- operation correlation and sequence fencing;
- first-output watchdog;
- retry eligibility after zero/partial output;
- server Retry-After/backoff;
- cancellation during every phase;
- streaming parsers for Responses, Chat Completions, Anthropic;
- connector auth/account/tool state machines;
- helper lifecycle/circuit breaker.

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

Assertions must cover UI state and canonical persisted state, not just backend return values.

### Packaged live-provider acceptance

The signed package must exercise ordinary prompts, not only marker probes. Minimum scenarios:

1. simple Chinese Q&A;
2. simple English Q&A;
3. “创建一个打地鼠的小程序” or an equivalent real tool-capable task;
4. two concurrent Agent turns;
5. stop/cancel then new turn;
6. transient provider/network recovery;
7. renderer reload/reopen during a turn;
8. at least one authenticated connector/MCP catalog -> select -> tool execution journey;
9. creation flow from the approved reference capture.

Marker prompts such as `PR7_A_OK`, `BENCH_OK`, `FABUSHI-*-ONLY-*`, or `CANDIDATE-LIFECYCLE-OK` may remain diagnostic cases but cannot be the only chat proof.

### Latency acceptance

Record at least 20 simple live turns and report p50/p95 local-paint, acceptance, first-output, first-text, and completion. A turn that reaches the outer 180-second test watchdog is a latency failure even if it eventually completes.

### Power acceptance

On the packaged candidate:

- record process tree before interaction;
- warm for five minutes;
- sample visible idle for ten minutes;
- sample hidden idle for ten minutes with remote computer background capability disabled;
- repeat with the background capability explicitly enabled to quantify the feature cost;
- attach CPU/memory/process/network samples and an Activity Monitor / battery menu capture.

## 13. Acceptance criteria / Definition of Done

- **AC-01:** The exact packaged candidate answers all required ordinary live-provider prompts with canonical completed/failed terminal state; none remains indefinitely at “正在思考”.
- **AC-02:** No renderer-generated synthetic assistant “thinking” event exists before canonical Host acceptance.
- **AC-03:** Concurrent Agent acceptance/stream/final events remain isolated without fallback target guessing.
- **AC-04:** A provider that emits no first output triggers the configured watchdog and bounded retry/terminal failure; the UI leaves spinning state.
- **AC-05:** Supported providers stream partial output; Chat Completions/Anthropic are not intentionally buffered when upstream streaming is available.
- **AC-06:** Latency artifacts meet PERF-001 through PERF-005 and show p50/p95, not only eventual completion.
- **AC-07:** “Plugins” opens a dedicated connector/plugin surface and a real auth/account/server/tool journey completes in the packaged app.
- **AC-08:** Mini Apps remain independently accessible and are not presented as the Plugins implementation.
- **AC-09:** The approved New/+ reference journey is reproduced and verified by side-by-side video/screenshots.
- **AC-10:** The canonical desktop Agent orchestration/control plane conforms to ARCH-001 through ARCH-008.
- **AC-11:** With background computer control disabled, the remote-device helper is absent and the packaged app satisfies POWER-008; the macOS supplemental energy check in POWER-009 also passes.
- **AC-12:** Hiding/closing the window enters a measurable low-power profile and does not keep unnecessary provider/Agent activity alive.
- **AC-13:** No credentials/secrets appear in renderer state, logs, traces, or evidence bundles.
- **AC-14:** Existing user account/Agent/conversation/Mini App data survives migration and rollback testing.
- **AC-15:** Exact-HEAD GitHub Actions tests, signed packaged acceptance, merge SHA, and canonical-main release are all separately recorded. A green PR without canonical-main release evidence is not release completion.

## 14. Release / migration / rollback

Implementation must be delivered from a single recovery branch (or a documented sequence of small protected PRs) based on the canonical main that contains this spec.

Migration order:

1. instrumentation and watchdog/stream correctness;
2. typed ID/submission-journal compatibility layer;
3. TypeScript coordinator/Host ownership cutover;
4. Plugins/connectors surface;
5. creation/visual parity;
6. power lifecycle;
7. removal of obsolete Rust/renderer lifecycle compatibility paths only after packaged acceptance.

Rollback must preserve the previous persistence schema or provide a backward reader. A rollback must not strand accepted turns, corrupt Agent transcripts, or lose connector account metadata.

Release gate order is: implementation complete -> GitHub Actions tests -> exact-HEAD signed packaged acceptance -> merge -> canonical-main release workflow -> published artifact verification.

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
| ARCH-001..008 | blocked | Current canonical user-visible run lifecycle is split across TypeScript/Electron/Rust and requires cutover. |
| POWER-001..009 | blocked | Process-level measurement and demand-driven lifecycle work not yet completed. |
| OBS-001..005 | blocked | Required parity evidence bundle not yet produced. |
| AC-01..15 | blocked | This document defines the recovery gate; no implementation completion is claimed. |

Allowed statuses: `passed`, `blocked`, `not-applicable`.

Implementation must update this compliance table with exact commit/workflow/artifact evidence before any claim that Grok parity is complete.
