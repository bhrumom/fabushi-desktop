# Grok Bot 0.18 Architecture-Equivalent Rebuild and Product Parity Recovery — Specification

Status: active  
Owner: Fabushi desktop / Agent runtime  
Last updated: 2026-09-23  
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

The target is to rebuild Fabushi desktop so that its **overall architecture, process boundaries, module ownership, protocols, runtime state machines, failure semantics, folder/domain structure, and observable product behavior are equivalent to Grok Bot 0.18** at the frozen reference baseline `b-nnett/grok-bot-0.18-reconstructed@a9f633e09d49a85829b8236331b9e21f7e612634`.

The implementation must proceed module by module across the Grok Bot code tree, but **language parity is not a requirement and one-to-one physical file copying is not a requirement**. The architectural role and observable product effect of each Grok module are normative; the implementation language and exact target-file granularity are selected by technical fit.

The canonical migration rule is: **per-source-file audit/disposition + per-product-responsibility desktop implementation**. Every frozen Grok source file must be accounted for in the architecture manifest, but the Fabushi target tree does not need the same file count. One Grok file may split across multiple Fabushi files, and multiple Grok files may converge into one implementation module, provided no Grok architectural boundary or product responsibility is collapsed or lost.

Required principles:

- every source-bearing Grok module must be individually audited and dispositioned, while every product-relevant responsibility must have a real Fabushi desktop implementation, an evidenced existing equivalent, or an explicit evidence-backed platform/non-code `not-applicable` classification;
- the **relative source/domain folder architecture must mirror Grok Bot**, preserving the same major boundaries such as `frontend/`, `source/electron-main/`, `source/electron-preload/`, `source/node-agent-coordinator/`, `source/host/`, `source/shared/`, and `source/packages/`;
- Mahayana may and should implement the **Grok Node Agent Coordinator architectural role** where Rust is a strong fit. The fact that Grok names the folder `node-agent-coordinator` does not require Node as the implementation language;
- the coordinator/host split must remain real even if both are implemented in Rust: Coordinator, Host, Runner, renderer bridge, MCP routing, local execution, persistence, and native capabilities must not be collapsed into one opaque monolith;
- UI and Electron-native boundaries should use TypeScript/React where that is the best fit for Electron/DOM APIs; runtime, coordinator, Host/Runner, provider streaming, persistence, native execution, and computer-control paths should prefer Rust where it improves correctness, performance, resource use, and maintainability;
- a module may use another language when the reference/platform ecosystem makes that clearly superior, but the decision must be recorded in the architecture manifest and may not alter the Grok-equivalent responsibility or contract;
- Fabushi code, services, compatibility layers, product surfaces, or background processes that have no Grok counterpart must be removed from the canonical desktop implementation unless this spec explicitly approves a Fabushi-specific extension boundary;
- the final shipped desktop application must not retain a second parallel legacy Fabushi runtime beside the Grok-shaped architecture;
- Fabushi branding, service endpoints, signing identity, account implementation details, and approved native capabilities may differ through narrow adapters, but those differences must not create a different desktop orchestration architecture;
- ordinary chat, Agents, creation flow, Plugins/connectors/MCP, process lifecycle, retry/recovery, power behavior, and UI interaction must match the approved Grok reference behavior;
- no-op mirror files, placeholder counterparts, or manifest-only status changes are not parity evidence; every completed mapping must demonstrate production wiring and the corresponding desktop product effect.

The desired end state is therefore: **Grok Bot 0.18’s architecture and product behavior under Fabushi identity, implemented with the best-fit language at each boundary, with Mahayana serving as the Rust implementation of Grok-equivalent coordinator/host/runtime roles where appropriate.**

## 3. Non-goals / out of scope

- Keeping the current Fabushi desktop architecture merely because it already exists.
- Requiring every Grok source file to become a unique Fabushi target file merely to match file counts.
- Creating no-op, placeholder, or empty compatibility files solely to satisfy a one-to-one source-tree mapping.
- Requiring every Grok TypeScript/JavaScript file to become Rust when TypeScript/React is objectively the better implementation boundary for Electron or browser UI.
- Requiring Grok’s `node-agent-coordinator` to remain Node. The folder/domain name is architectural provenance; Mahayana may implement that role in Rust.
- Collapsing Coordinator + Host + Runner + renderer state into one Mahayana binary simply because Rust can implement all of them.
- Keeping `desktop/src`, `desktop/electron`, `frontend/apps/web`, `third_party/mahayana`, compatibility adapters, Mini Apps, Telegram/messaging, payments, calls, or other Fabushi-only desktop subsystems after cutover unless they are mapped to a Grok counterpart or explicitly approved as a narrow Fabushi extension.
- Reinterpreting “same architecture” as only matching high-level concepts. Folder/domain ownership, module boundaries, dependency direction, process boundaries, protocols, lifecycle ownership, retry/cancellation semantics, and persistence ownership must be mirrored and verified.
- Rewriting Grok behavior into a new “cleaner” architecture when that changes responsibility or observable semantics. Improvements may be proposed only after architecture parity is proven or through an explicit spec exception.
- Treating existing Fabushi features as automatically grandfathered. If no reference counterpart or approved extension exists, the default action is removal from the desktop product/code path.
- Treating a deterministic “reply exactly X” probe as proof of ordinary user-chat correctness.
- Declaring parity from unit tests, mocks, source layout, or compile success without signed packaged-app evidence.
- Copying unlicensed upstream source text, comments, compiled code, assets, or branding. This specification requires a traceable semantic reimplementation and module mapping; direct source reuse requires independent rights review.
- Expanding this desktop rebuild into iOS/Android work.

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

### Architecture / Grok-equivalent structure with best-fit languages

- **ARCH-001 — Frozen Grok source baseline.** All structural comparisons and parity decisions use `b-nnett/grok-bot-0.18-reconstructed@a9f633e09d49a85829b8236331b9e21f7e612634`. A baseline change requires a spec update and a new complete mapping.
- **ARCH-002 — Complete module inventory.** Generate and check in a machine-readable architecture manifest covering every source-bearing Grok path. Each row records the Grok path/blob SHA, source responsibility, desktop-visible effect, platform delta, Fabushi target path(s) or explicit disposition, implementation language, owning process/crate/package, status, replacement behavior where applicable, and behavioral/test evidence. No source module may be silently skipped. Manifest completeness is an audit requirement, not a target-file-count requirement.
- **ARCH-003 — Folder/domain parity.** The canonical Fabushi source tree must preserve Grok Bot’s major relative domain hierarchy and nested feature ownership: `frontend/`, `source/electron-main/`, `source/electron-preload/`, `source/node-agent-coordinator/`, `source/host/`, `source/shared/`, and `source/packages/`. File extensions and internal file granularity may differ by implementation language when ownership and dependency direction remain equivalent.
- **ARCH-004 — Module-by-module responsibility mapping.** Each executable Grok module must map to one or more explicit Fabushi implementation modules, or to an evidenced existing equivalent, that preserve its responsibility, input/output contract, state machine, failure behavior, process ownership, dependency direction, and observable desktop effect. Many-to-one and one-to-many mappings are allowed when they do not collapse a reference architectural boundary and are justified in the manifest.
- **ARCH-005 — Best-fit language policy.** Architecture is normative; language is an implementation choice. Default targets are:
  - `frontend/**` -> React + TypeScript for DOM/UI/state projection;
  - `source/electron-main/**` -> TypeScript for Electron-native APIs, with Rust services behind typed IPC where appropriate;
  - `source/electron-preload/**` -> TypeScript, minimal and capability-scoped;
  - `source/node-agent-coordinator/**` -> Mahayana Rust preferred, preserving Grok coordinator protocols and process boundary;
  - `source/host/**` and `source/host/runner/**` -> Mahayana Rust preferred;
  - provider streaming, persistence, local execution, computer/native control -> Rust preferred;
  - OAuth/browser/Electron-specific adapters -> Rust or TypeScript according to API fit, without moving ownership out of the reference layer.
- **ARCH-006 — Mahayana Coordinator.** Create a real Mahayana Coordinator that is architecturally equivalent to Grok’s `source/node-agent-coordinator/**`: renderer-port protocol, request/reply/event multiplexing, cancellation, reconnect/resync, gateway routing, Host supervision, local-exec supervision, inference routing, MCP routing/OAuth forwarding, client-side tool relay, telemetry, and crash settlement as applicable to the frozen reference.
- **ARCH-007 — Coordinator is not Host.** Mahayana Coordinator and Mahayana Host are distinct ownership boundaries even if both are Rust crates/binaries. Host crash/restart must not require renderer ownership repair; Coordinator must be able to supervise/reconnect/resync using the Grok-equivalent contract.
- **ARCH-008 — Host/Runner parity.** Mahayana Host/Runner must map Grok’s `source/host/**` responsibilities, including send pipeline, prompt acceptance, transcript lifecycle, first-output watchdog, streaming attempt ownership, retry/backoff, checkpoint/resume, cancellation, tool/MCP execution, waiting-user states, terminal settlement, and durable persistence.
- **ARCH-009 — Renderer parity.** The renderer follows Grok’s frontend ownership and interaction model. React/TypeScript may remain and is preferred when it reduces semantic drift from the reference. Renderer code must not become the canonical owner of run truth, retries, provider lifecycle, or operation identity.
- **ARCH-010 — Electron parity.** Electron main/preload preserve Grok-equivalent boundaries and use TypeScript where Electron APIs are native to Node. Electron code may supervise/start the Mahayana Coordinator but must not absorb coordinator/host business state.
- **ARCH-011 — Dependency-direction parity.** Cross-domain imports/RPC dependencies that have no Grok counterpart require an explicit spec exception. Convenient Fabushi-only cross-coupling is forbidden.
- **ARCH-012 — State-ownership parity.** Submission, operation/run identity, transcript/checkpoint, retry/cancel, connector state, account state, process lifecycle, and renderer projection ownership must follow the corresponding Grok architecture rather than current Fabushi compatibility behavior.
- **ARCH-013 — Remove non-Grok code.** Any shipping Fabushi subsystem without a reference counterpart or approved extension must be deleted after required data migration. Git history is the archive; a parallel legacy implementation is not allowed to remain enabled or compiled into the production path.
- **ARCH-014 — No shadow architecture.** After cutover there is exactly one canonical implementation for each reference subsystem. Migration adapters must have explicit removal criteria and fail the final architecture gate if still required for normal operation.
- **ARCH-015 — Automated architecture gate.** CI compares the frozen Grok inventory against the Fabushi architecture manifest and canonical tree. Completion requires zero unclassified reference modules, zero required product responsibilities without a real production implementation/equivalent, zero unauthorized extra shipping modules, and zero unjustified boundary collapses. The gate must not require equal source/target file counts.
- **ARCH-016 — Language substitution test.** A language change is accepted only if contract tests demonstrate equivalent behavior and the change does not alter the reference process/module boundary. “Implemented in Rust” is never by itself evidence of parity.
- **ARCH-017 — Migration safety.** User data needed by the retained Grok-equivalent product must be migrated before legacy paths are deleted. Data belonging only to removed Fabushi-only features may be exported/backed up, but the feature implementation itself does not remain in the shipping architecture.

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

The shipping source tree follows Grok Bot 0.18’s major architectural domains. Implementation language may change file extensions or crate/package layout inside those domains, but the reference responsibilities remain recognizable and machine-mapped.

Representative shape:

```text
frontend/
  ... Grok-equivalent renderer/product domains in React/TypeScript ...

source/
  electron-main/
    ... Grok-equivalent Electron main modules in TypeScript,
        delegating runtime/native work through typed boundaries ...

  electron-preload/
    ... minimal Grok-equivalent preload bridge in TypeScript ...

  node-agent-coordinator/
    ... Mahayana Coordinator, preferably Rust,
        preserving Grok coordinator responsibilities/protocols ...

  host/
    ... Mahayana Host + Runner, preferably Rust,
        preserving Grok host/runner subdivision ...

  shared/
    ... shared schemas/protocols, generated or implemented in
        the language needed by both sides without changing ownership ...

  packages/
    ... Grok-equivalent package/domain segmentation ...
```

The historical folder name `node-agent-coordinator` is retained for structural provenance even when the Fabushi implementation is Rust.

Legacy Fabushi roots such as `desktop/src`, `desktop/electron`, `frontend/apps/web`, and `third_party/mahayana` are migration sources, not protected final architecture. Code may be moved/reused only when it maps cleanly into the Grok-equivalent domain structure.

### 6.2 Target runtime flow

```text
React/TypeScript Renderer
  -> Grok-equivalent submission journal / coordinator client
  -> typed preload/IPC boundary
  -> Mahayana Coordinator
       -> canonical request/reply/event protocol
       -> cancellation / reconnect / resync
       -> gateway + MCP + local-exec routing
       -> Host supervision
  -> Mahayana Host
       -> prompt acceptance / transcript owner
       -> Mahayana Runner
            -> provider attempt owner
            -> first-output watchdog
            -> bounded retry / checkpoint resume
            -> streaming
            -> tools / MCP
            -> terminal settlement
       -> durable persistence
  -> canonical event stream back through Coordinator
  -> renderer projection only
```

The renderer paints optimistic user input if desired, but assistant lifecycle state is driven by canonical Coordinator/Host events. Mahayana is therefore not merely “the Host binary”; it becomes the Rust implementation of the Grok-equivalent runtime/control-plane roles where Rust is the best fit.

### 6.3 Target product surface

The visible desktop product contains the Grok-equivalent Agent roster/conversation, create/new flows, Plugins/connectors/MCP, settings/account, transcript, computer/local execution, and corresponding overlays/dialogs proven by the reference.

Fabushi-only surfaces with no reference counterpart or explicit extension approval are removed rather than retained through compatibility menus.

### 6.4 Target power/process behavior

Process lifetime, coordinator/host startup, background services, reconnect policy, helper lifecycle, and event routing follow Grok-equivalent ownership. Rust is preferred for long-lived runtime services where it materially improves resource usage, but power acceptance is based on measured process behavior, not implementation language.

## 7. Architecture and ownership boundaries

This section is defined by **Grok reference ownership first, implementation language second**.

### `frontend/` — renderer and product presentation

Preferred implementation: React + TypeScript.

Owns:

- visual state and interaction;
- Agent roster and workspace projection;
- composer/drafts/selection;
- transcript rendering;
- create/new flows;
- Plugins/connectors UI;
- settings/account presentation;
- approved reference interaction behavior.

Must not own canonical operation identity, provider retries, Host recovery, durable transcript truth, or connector credentials.

### `source/electron-main/` — Electron desktop-main boundary

Preferred implementation: TypeScript for direct Electron APIs, with Rust services behind typed IPC when useful.

Owns Grok-equivalent:

- Electron app/window/session lifecycle;
- menu/tray/update/deep-link integration;
- secure process bootstrap;
- starting/stopping/supervising Mahayana Coordinator;
- native desktop adapters that are specifically main-process responsibilities;
- MCP/OAuth browser integration only where the reference assigns it here.

Must not become a replacement Coordinator or Host.

### `source/electron-preload/` — minimal trusted bridge

Preferred implementation: TypeScript.

Owns no business state. It exposes the smallest safe typed bridge required by Electron and forwards to the Coordinator/main boundaries.

### `source/node-agent-coordinator/` — Mahayana Coordinator

Preferred implementation: Rust.

This is a real architectural role equivalent to Grok’s Node Agent Coordinator, regardless of implementation language.

Owns:

- renderer-port protocol and lifecycle;
- request/reply/event multiplexing;
- cancellation;
- Agent/conversation routing;
- reconnect/resync and transport state;
- Gateway dispatch;
- Host supervision;
- local-exec supervision;
- inference routing where the reference coordinator owns it;
- routed MCP/tool relay;
- OAuth forwarding where the reference coordinator owns it;
- telemetry/transport-stage recording;
- process crash/protocol-breach settlement.

The Coordinator must be independently testable from the Host.

### `source/host/` — Mahayana Host and Runner

Preferred implementation: Rust.

Owns Grok-equivalent:

- prompt acceptance/send pipeline;
- canonical run/turn lifecycle;
- transcript/checkpoint persistence;
- provider selection and streaming attempts;
- first-output and overall deadlines;
- safe retry/backoff/checkpoint resume;
- tools/MCP execution;
- waiting-user/approval state;
- cancellation and terminal settlement;
- Host extensions/services.

### `source/shared/`

Implementation: shared schemas generated or authored for the participating languages.

Owns only cross-boundary contracts/protocol definitions. It must not become a dumping ground for Fabushi-only abstractions.

### `source/packages/`

Implementation: best-fit per reference package.

Preserves Grok package/domain segmentation. Language choice is recorded in the architecture manifest and may not change package responsibility.

### Native/platform capability implementations

Preferred implementation: Rust where native OS, process, sandbox, capture/input, security, local execution, or computer-control capabilities benefit from it.

These capabilities stay behind the Grok-equivalent module/process boundary rather than creating a separate Fabushi architecture.

### Language selection rule

Language is chosen per module using this order:

1. preserve Grok architectural responsibility and observable behavior;
2. preserve process/boundary semantics;
3. choose the ecosystem-native implementation when it lowers risk;
4. prefer Rust for long-lived runtime/native/concurrency/performance-sensitive services;
5. prefer TypeScript/React for Electron/DOM presentation and APIs;
6. require evidence before introducing another language.

### Removed legacy ownership

After final cutover, no legacy Fabushi tree may remain a second source of truth. Existing files may survive only if moved/mapped into the corresponding Grok-equivalent boundary or explicitly approved as a narrow extension.

## 8. Interfaces / contracts / schemas / data flow

### 8.1 Renderer -> Coordinator contract

The renderer/client contract must preserve the Grok-equivalent separation between client nonce/request identity and canonical operation identity. The wire format may be TypeScript on the renderer side and Rust types on the Coordinator side, generated from one schema where practical.

Minimum concepts:

```text
SubmitTurn {
  clientNonce,
  requestId,
  agentId,
  conversationId,
  text,
  attachments,
  references
}

TurnAccepted {
  requestId,
  clientNonce,
  operationId,
  turnId,
  runId,
  acceptedAtMs
}
```

`clientNonce` is the idempotency key. `requestId` scopes one RPC. `operationId` is created only by the canonical runtime owner.

### 8.2 Coordinator port protocol

The Mahayana Coordinator must expose Grok-equivalent lifecycle/request/reply/event/cancel semantics:

- hello/version handshake;
- strict directionality for client/server frame types;
- unique in-flight request ids;
- cancellation that aborts the underlying request;
- protocol breach settlement;
- ready/down/recovering transport state;
- pending request rejection on disconnect;
- event-family routing.

If the wire representation differs from Grok, compatibility tests must prove equivalent semantics.

### 8.3 Lifecycle envelope

Every runtime event used for a user-visible turn must carry canonical correlation fields equivalent to:

```text
TurnEnvelope {
  agentId,
  conversationId,
  operationId,
  turnId,
  runId,
  sequence,
  occurredAtMs
}
```

The renderer must reject stale/out-of-generation events deterministically and must never guess the target Agent from “only pending peer”.

### 8.4 Provider attempt contract

Mahayana Runner exposes the same responsibilities as Grok’s stream-attempt/retry layer:

- cancelable attempt context;
- first-output deadline;
- stream-output-produced state;
- resumable checkpoint;
- bounded retry policy;
- server-paced retry support;
- explicit retry/exhausted/ineligible outcomes;
- final-state persistence.

This is an architectural/behavioral mapping; the Rust API does not need to imitate TypeScript syntax.

### 8.5 Plugins/MCP contract

The Plugins/MCP subsystem exposes typed operations equivalent to the Grok reference for:

- catalog search/list;
- installed list;
- install/update/remove/enable/disable;
- auth start/callback/status/disconnect;
- accounts list/add/rename/remove/select;
- MCP server status/refresh;
- tools list/enable/disable;
- reference/tool selection and actual tool execution for a turn.

UI may be TypeScript/React while runtime/server/tool execution may be Rust.

### 8.6 Architecture-manifest contract

A machine-readable manifest is mandatory. Minimum fields:

```text
reference_path
reference_blob_sha
reference_role
desktop_effect
platform_delta
target_path
related_target_paths
target_language
target_process_or_package
status
replacement_behavior
behavioral_evidence
test_evidence
notes
```

Allowed final statuses are `implemented`, `equivalent`, `not-applicable-platform`, `not-applicable-noncode`, and `removed-extra`. `pending`, `compatibility-only`, or `unmapped` blocks completion.

`not-applicable-platform` applies only when the reference implementation mechanism genuinely has no desktop-platform counterpart for the supported Fabushi target. It may not be used to drop a still-required user-visible Grok capability. If the product effect still matters, `replacement_behavior` is mandatory.

## 9. Constraints and non-functional requirements

### Performance / latency

Performance is measured from the user gesture. A language choice is accepted only when the resulting module meets Grok-equivalent responsiveness, streaming cadence, cancellation, and failure-recovery behavior.

### Power / CPU / memory

Power gates are measured on the whole process tree. Replacing TypeScript with Rust is not by itself a power optimization. Background processes, timers, WebSockets, retries, polling, renderer work, and helper lifetime must be demand-driven and measured.

### Structural fidelity

- major source/domain hierarchy mirrors the frozen Grok tree;
- every executable reference module is mapped;
- process and ownership boundaries are preserved;
- unauthorized extra shipping modules are forbidden;
- old Fabushi trees cannot remain as parallel implementations;
- file extension/language differences are allowed only when the architecture manifest preserves the reference responsibility.

### Language fitness

Default implementation choices are:

- React/TypeScript for renderer/product UI;
- TypeScript for Electron main/preload where direct Electron APIs dominate;
- Mahayana Rust for Coordinator, Host, Runner, streaming/provider, persistence, local-exec/native/computer-control;
- Rust or TypeScript for MCP/OAuth boundary modules according to SDK/platform fit.

A deviation is allowed when documented with a concrete technical reason and contract tests. “Rust everywhere” and “TypeScript everywhere” are both explicitly rejected as architectural principles.

### Security / privacy

- preserve Electron context isolation, sandboxing, web security, and capability-scoped preload;
- OAuth/tokens remain outside renderer-visible state and logs;
- connector tool execution remains policy/approval controlled;
- Rust/native/FFI boundaries must be narrow and memory-safe;
- renderer/Main/Coordinator/Host IPC schemas must validate untrusted input.

### Compatibility and deletion

Compatibility exists only to migrate data into the new Grok-shaped architecture. It is not a reason to keep old product features or duplicate runtime ownership.

Before removing a Fabushi-only subsystem:

1. identify whether user data must be exported or migrated;
2. provide deterministic migration/backup where needed;
3. remove code, navigation, background service, storage writer, tests, and unused package dependencies;
4. verify the packaged binary contains no runtime entrypoint for it.

### Reliability

No accepted turn may be orphaned by renderer reload, Coordinator restart, Host restart, provider timeout, OAuth expiry, or transient network failure. Recovery behavior must map to the corresponding Grok architectural owner.

### Provenance / rights

The Grok reconstruction’s provenance explicitly states that no upstream source-code license is implied. The one-to-one architecture requirement is a traceable semantic reimplementation, not permission to reproduce unlicensed source text or binaries. Public redistribution of any directly reused material requires a separate rights review.

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

Generate the complete source/module inventory from `a9f633e09d49a85829b8236331b9e21f7e612634`. Establish the architecture manifest before implementation. Every source-bearing Grok module gets a row describing its responsibility, desktop effect, platform delta, and target/disposition; every current Fabushi shipping module is classified as mapped-to-Grok, approved extension, or extra-to-remove. The inventory is an exhaustive audit index, not a requirement to create one Fabushi file per Grok file.

### Phase 1 — Create the mirrored architectural tree

Create the canonical Grok-shaped domains first:

- `frontend/**`
- `source/electron-main/**`
- `source/electron-preload/**`
- `source/node-agent-coordinator/**`
- `source/host/**`
- `source/shared/**`
- `source/packages/**`

Do not invent a competing Fabushi hierarchy. Choose implementation language per module according to ARCH-005 and record it in the manifest.

### Phase 2 — Build Mahayana Coordinator

Implement the Grok `source/node-agent-coordinator/**` responsibilities as Mahayana Coordinator, preferably in Rust:

- renderer-port protocol;
- request/reply/event routing;
- cancellation;
- reconnect/resync;
- gateway routing;
- Host supervisor;
- local-exec supervisor;
- inference router;
- routed MCP/tool bridge;
- OAuth forwarding where applicable;
- telemetry/crash settlement.

Connect the existing renderer through a real Coordinator client boundary. Remove renderer-owned operation-adoption/fallback logic as canonical Coordinator correlation becomes authoritative.

### Phase 3 — Rebuild Mahayana Host / Runner to Grok responsibilities

Map Grok `source/host/**` module by module, including:

- send pipeline and prompt acceptance;
- stream attempt;
- first-token stall policy;
- transient/provider error classification;
- retry/backoff/checkpoint behavior;
- transcript persistence;
- runner lifecycle;
- provider streaming;
- tool/MCP execution;
- waiting-user/cancel/terminal state.

Existing Mahayana Rust code may be reused only when it is moved behind the correct Grok-equivalent ownership and passes parity tests.

### Phase 4 — Electron main/preload and Plugins/MCP parity

Keep Electron-native main/preload code in TypeScript where that is the lowest-risk implementation. Port the Grok-equivalent MCP/plugin/account/OAuth/catalog responsibilities into the correct layer, using Rust services where appropriate but preserving Grok ownership.

### Phase 5 — Frontend/product parity

Use React/TypeScript for the renderer unless a specific module has a stronger reason otherwise. Reproduce the recovered Grok frontend domain organization, state ownership, create/new behavior, transcript/composer behavior, Plugins UI, and approved reference captures. Do not force UI logic into Rust/WASM solely for language uniformity.

### Phase 6 — Remove everything without a Grok counterpart

After each mapped subsystem has a passing replacement, remove the old Fabushi implementation. Before final acceptance, delete unmatched shipping code/product surfaces/background services unless explicitly approved as narrow Fabushi extensions.

No compatibility adapter may remain in the final shipping path simply to preserve the previous architecture.

### Phase 7 — Structural, behavioral, performance, and power convergence

Run the architecture-manifest checker, tree/domain parity checker, process-boundary contract tests, latency suite, live chat suite, connector suite, process-tree power suite, and side-by-side UI reference capture.

### Phase 8 — GitHub Actions, packaged acceptance, merge, release

Only after the entire architecture rebuild and removal pass is complete should release validation run through GitHub Actions. Fix failures at the exact candidate HEAD. Merge only after structural/behavioral/performance/power gates pass, then independently verify the canonical-main signed release and published assets.

## 12. Verification / test strategy

### Structural / architecture-completeness gates

CI must fail if any of the following is true:

- a source-bearing Grok module has no architecture-manifest row;
- an executable Grok module is not `implemented`;
- a shipping Fabushi code module has no Grok counterpart and is not an approved extension/platform adapter;
- the canonical domain hierarchy diverges from the frozen Grok architecture without an approved spec exception;
- Coordinator/Host/Runner/reference process boundaries are collapsed without an approved exception;
- old `desktop/src`, `desktop/electron`, `frontend/apps/web`, `third_party/mahayana`, or another legacy tree still acts as a parallel runtime after final cutover;
- a temporary compatibility adapter remains required for normal production operation.

### Per-language compile / dependency gates

- Rust Coordinator/Host/Runner/native crates compile on supported desktop platforms;
- TypeScript/React renderer, Electron main, and preload typecheck/build;
- shared protocol generation/schema validation is reproducible;
- circular/cross-domain dependencies not present in the Grok architecture are rejected;
- module/process ownership is checked against the architecture manifest;
- language substitutions include contract/parity tests and do not move responsibility to a different layer.

### Coordinator contract gates

Test Mahayana Coordinator independently for:

- protocol hello/version negotiation;
- request id uniqueness;
- request/reply/event directionality;
- cancellation;
- pending-request rejection on disconnect;
- reconnect/resync;
- Host restart/generation transition;
- gateway down/up state;
- MCP/tool routing;
- protocol-breach settlement;
- crash reporting.

### Behavioral unit / contract gates

For each mapped Grok subsystem, tests cover the corresponding observable semantics. At minimum:

- submission journal duplicate/offline/reconnect/stale-generation behavior;
- operation correlation and sequence fencing;
- first-output watchdog;
- retry eligibility after zero/partial output;
- server Retry-After/backoff;
- cancellation during every phase;
- streaming parsers for supported providers;
- connector auth/account/tool state machines;
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

Assertions cover renderer-visible behavior, Coordinator state, Host/Runner state, and canonical persisted state.

### Packaged live-provider acceptance

The signed package must exercise ordinary prompts, not only marker probes:

1. simple Chinese Q&A;
2. simple English Q&A;
3. “创建一个打地鼠的小程序” or an equivalent real tool-capable task when the approved reference supports the same capability;
4. two concurrent Agent turns;
5. stop/cancel then new turn;
6. transient provider/network recovery;
7. renderer reload/reopen during a turn;
8. authenticated connector/MCP catalog -> select -> tool execution;
9. the approved reference New/+ creation journey;
10. Coordinator or Host process restart followed by deterministic recovery.

Marker prompts may remain diagnostics but cannot be the only proof.

### Removal acceptance

For every current Fabushi subsystem classified `extra-to-remove`, acceptance must prove:

- source removed or excluded from production;
- navigation/control removed;
- background process/service removed;
- package dependency removed when unused;
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
- attribute Coordinator, Host, renderer, Electron main, and helper CPU/network activity separately;
- attach CPU/memory/process/network samples and an Activity Monitor/battery-menu capture.

## 13. Acceptance criteria / Definition of Done

- **AC-01:** The exact packaged candidate answers all required ordinary live-provider prompts with canonical completed/failed terminal state; none remains indefinitely at “正在思考”.
- **AC-02:** No renderer-generated synthetic assistant “thinking” state is treated as canonical before runtime acceptance.
- **AC-03:** Concurrent Agent acceptance/stream/final events remain isolated without fallback target guessing.
- **AC-04:** A provider that emits no first output triggers the Grok-equivalent watchdog and bounded retry/terminal failure.
- **AC-05:** Supported providers stream partial output with Grok-equivalent attempt lifecycle.
- **AC-06:** Latency artifacts meet PERF-001 through PERF-005 and report p50/p95, not only eventual completion.
- **AC-07:** Plugins/connectors/MCP reproduce the mapped Grok catalog/auth/account/server/tool behavior in the packaged app.
- **AC-08:** The approved New/+ reference journey is reproduced and verified by side-by-side video/screenshots.
- **AC-09:** The complete frozen Grok source/module inventory has exactly one architecture-manifest row per reference item; there are zero silently skipped executable modules. This is an audit-completeness criterion, not a one-to-one target-file criterion.
- **AC-10:** Every product-relevant Grok responsibility has a real production-wired Fabushi desktop implementation or evidenced existing equivalent; platform/non-code exceptions are explicitly justified and cannot remove a still-required product effect. Final manifest has no `pending`, `compatibility-only`, or `unmapped` rows.
- **AC-11:** The canonical Fabushi domain/folder architecture mirrors Grok’s major boundaries and passes the automated architecture gate.
- **AC-12:** Mahayana Coordinator implements the Grok `node-agent-coordinator` architectural role with independent protocol, supervision, cancellation, reconnect/resync, routing, and crash-settlement tests.
- **AC-13:** Mahayana Coordinator and Mahayana Host/Runner remain distinct ownership boundaries; Host restart/recovery does not require renderer-side operation guessing.
- **AC-14:** Language choices follow the best-fit policy: React/TypeScript may own UI/Electron-native boundaries; Rust is preferred for Coordinator/Host/Runner/native/runtime paths; deviations are documented and contract-tested.
- **AC-15:** There are zero unauthorized extra shipping Fabushi modules/surfaces without a Grok counterpart or approved extension.
- **AC-16:** Legacy parallel runtimes/compatibility shells are removed from the production path; exactly one canonical implementation owns each reference subsystem.
- **AC-17:** With inactive background features, the packaged app satisfies POWER-008 and the macOS supplemental energy check in POWER-009.
- **AC-18:** No credentials/secrets appear in renderer state, logs, traces, or evidence bundles.
- **AC-19:** Required retained user data survives migration; data for removed Fabushi-only features has an explicit export/backup decision.
- **AC-20:** Exact-HEAD GitHub Actions tests, signed packaged acceptance, merge SHA, and canonical-main release are all separately recorded.
- **AC-21:** A final mapping report lists every Grok reference module, its responsibility/effect, Fabushi target path(s) or reviewed disposition, implementation language, owning process/package, replacement behavior where applicable, and test evidence, plus every deleted Fabushi-only source path.
- **AC-22 — Grok Bot desktop effect:** The exact packaged app demonstrates the same core Agent product effect as the approved Grok Bot 0.18 reference: a durable user turn progresses through acceptance → preparing/thinking → real tool/MCP/Runner activity when invoked → live tool state → continued inference → incremental transcript streaming → terminal completion/failure. Renderer reload, Coordinator reconnect, or temporary network loss during an active durable run must resynchronize that same run without silent loss or duplicate execution when the reference architecture would keep the run alive. Platform differences must be explicit adaptations, not unacknowledged feature reduction.

## 14. Release / migration / rollback

Implementation must be delivered from canonical main containing this specification.

Migration order:

1. freeze the Grok reference inventory and create the architecture manifest;
2. create the mirrored Grok-equivalent domain tree;
3. build Mahayana Coordinator at the `source/node-agent-coordinator` boundary;
4. rebuild Mahayana Host/Runner at the `source/host` boundary;
5. connect shared protocols and renderer Coordinator client;
6. align Electron main/preload and Plugins/MCP responsibilities;
7. align frontend/product behavior;
8. migrate/export required retained user data;
9. delete every Fabushi-only/unmapped shipping subsystem and old parallel runtime path;
10. run structural parity, Coordinator/Host fault recovery, behavioral, performance, power, and signed packaged acceptance;
11. merge;
12. verify canonical-main release and published assets.

Rollback must operate at a release boundary. Git history and tagged artifacts provide code rollback; the production binary must not ship both old and new architectures merely to simplify rollback.

No release is allowed while:

- the architecture manifest is incomplete;
- any executable Grok module remains unmapped/unimplemented;
- Mahayana Coordinator does not satisfy the Grok coordinator contract gates;
- Coordinator/Host/Runner ownership is still split ambiguously with the renderer;
- unauthorized extra Fabushi shipping modules remain;
- a legacy compatibility runtime is still required for ordinary operation;
- the architecture/tree boundary gate fails.

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
| ARCH-001..017 | blocked | Independent Mahayana Coordinator contract is exact-HEAD verified; remaining blockers are Host/Runner production parity, non-final architecture-manifest rows, product/packaged acceptance, and the final strict structural gate. |
| POWER-001..009 | blocked | Process-level measurement and demand-driven lifecycle work not yet completed. |
| OBS-001..005 | blocked | Required parity evidence bundle not yet produced. |
| AC-01..22 | blocked | This document defines the recovery gate; no implementation completion is claimed. |

Allowed statuses: `passed`, `blocked`, `not-applicable`.

Implementation must update this compliance table with exact commit/workflow/artifact evidence before any claim that Grok parity is complete.


### 2026-09-24 exact-HEAD recovery note

- PR #20 HEAD `64f8fb78c1e3309ddf18fb7839fd66ac0aa81dda` remains draft.
- Architecture manifest at this SHA: 1700 implemented, 261 planned, 41 existing-needs-parity.
- Forbidden legacy roots `desktop/src`, `desktop/electron`, `frontend/apps/web`, and `third_party/mahayana` are still present.
- Exact-HEAD workflow runs `35937158440` (Rust desktop runtime) and `35937158480` (Desktop Chat Parity CI) both ended `action_required` before any job was created, so they are not test evidence.
- This documentation-only commit exists solely to retrigger both exact-HEAD workflows under the active PR branch; it does not advance any compliance item to passed.

### 2026-09-25 shipping Agent checkpoint cutover

- Starting exact HEAD for this slice: \`2e027d98bce470266f1f7f554c0214fea7b868b1\`.
- The shipping \`runner.startRoutedProvider\` path now prepares a Runner-owned Agent-state checkpoint sink before launching the turn. A provider-success result is not allowed to settle \`completed\` until the sink succeeds.
- The checkpoint projection uses canonical frozen \`agent.v1\` wire field numbers for \`UserMessage\`, \`ConversationStep(assistant_message)\`, \`AgentConversationTurnStructure\`, \`ConversationTurnStructure\` and \`ConversationStateStructure.turns\`. Content-addressed user/step/turn blobs are written first; the prior root wire image is preserved byte-for-byte and a legal repeated field 8 is appended.
- Persistence uses the existing production transaction boundary: routed transcript mirror prepare -> \`ProductionAgentStore\` durable checkpoint/latestRootBlobId advance -> mirror commit. The \`sand_new_transcript_journal\` experiment controls journal vs legacy routing exactly at the shipping path.
- This is intentionally not full SandAgentRunner parity. Generated tool-call state, complete Agent resources/subagents/computer state, canonical generated Rust tool JSON bindings and broader settle/profile/memory semantics remain non-final and must stay represented as \`existing-needs-parity\`/planned rows.

### 2026-09-25 atomic generated-Agent confirmation watermark

- Direct user turns no longer advance Agent \`latestRootBlobId\` and the transcript recovery watermark as two independent writes.
- After content-addressed checkpoint blobs are written, the long-lived \`SandAgentDb\` owner opens one \`BEGIN IMMEDIATE\` transaction, verifies the expected prior root and addressed user entry, advances \`latestRootBlobId\`, and writes \`confirmed: true\` on that exact durable user transcript entry before commit.
- The turn-scoped durable checkpoint wrapper is used only when the shipping run carries a real \`messageId\`; background/redrive turns without an addressed transcript message retain the normal AgentStore checkpoint path.
- Contract coverage proves both success and the failure invariant: a missing addressed user entry rejects the durable checkpoint and does not advance the Agent root.



### 2026-09-25 box-exec close and Runner transient-retry recovery

- Exact implementation commits in this slice: `5d1c786bfcf5ad6232ae64c817f58e0a23af3e81`, `261c24661ddb096b0bd10f06e688154efed4b652`, and `530a0b2ff4ac883f617b7fc93c2a434d3eae5fc8`.
- The independent box-exec daemon now routes 401/405 responses through the same graceful write-half shutdown as successful Connect/HTTP responses, removing the Darwin ECONNRESET race that failed the exact-HEAD daemon contract. The disconnect/process-group contract now records the real shell PID with `$$` instead of the literal `$`, so it actually verifies process termination.
- The shipping Runner transient classifier now covers the frozen Grok transport/deadline token families used by bounded retry, with executable cargo coverage in `source/host/tests/runner_contract.rs`.
- On exact SHA `5d1c786bfcf5ad6232ae64c817f58e0a23af3e81`, the Rust workflow had already passed the independent Mahayana Coordinator contract, independent Grok box-exec daemon contract, shipping Host build, and box-exec supervisor before later commits moved HEAD. Those runs are slice evidence only, not final completion evidence.
- Final parity remains blocked: the strict architecture gate is still skipped while the architecture manifest contains non-final rows, and exact-HEAD workflows must be re-established on the latest branch HEAD after this note.

### 2026-09-25 production automation status reminder cutover

- Starting exact HEAD for this slice: `34704c8ae296ce2b62094bd749098e972a34c254`.
- The frozen Grok `source/host/automations/automation-status-reminder.ts` responsibility is now implemented in `source/host/src/automations/automation_status_reminder.rs`: it reads the immutable per-Agent automation definitions, applies the UI cap, resolves the user's current timezone, renders live run status, and excludes the currently firing routine's in-flight run from its own authoritative status snapshot.
- Shipping `runner.startRoutedProvider` now opens the production Agent automation store and injects that snapshot as system context before the Runner starts. The firing identity comes from the canonical `automationWake.id`; Renderer and Coordinator do not own or synthesize this status.
- `source/host/tests/automation_status_reminder_contract.rs` covers live-running, firing-run filtering, settled success, empty-store and explicit cleared-snapshot behavior. The architecture manifest advances this one frozen module from `planned` to `implemented` only because production wiring and executable test evidence are present.
- The previous exact-HEAD rerun already proved the independent Coordinator, box-exec daemon and Host box-exec supervisor gates after the accepted-socket fix; final architecture completion remains blocked until all other non-final manifest rows and forbidden legacy runtime roots are removed.

### 2026-09-25 automation transcript runtime audit closure

- Starting exact HEAD for this audit: `bc7a0e7e8de0e43691fd3d96e4f648d5ec0b96e0`.
- Five frozen transcript/automation rows had remained `planned` even though their Rust implementations, production AutomationRuntime wiring and executable integration contracts were already present. They were individually re-audited against the frozen Grok modules rather than bulk-promoted from filename existence.
- Finalized rows: `automation-event-fires.ts`, `automation-run-path.ts`, `automation-runtime.ts`, `automation-spend-guard-runtime.ts`, and `sand-automation-spend-guard.ts`.
- The evidence covers event debounce/coalescing and bounded drop reporting; durable run begin/finish and duplicate suppression; shipping CRUD/manual/background dispatch; unread/spend-guard nudge/pause/snooze/opt-out persistence; and run-now production routing through the Host-owned AutomationRuntime.
- This audit changes status only where implementation and tests already prove production ownership; it does not infer parity for the remaining missing Host extension targets.

### 2026-09-25 watched-directory production cutover

- Starting exact HEAD: `7e1fbb1f8a601c98354e40ae66107f742c1aacc4`.
- Added the Host-owned `source/host/src/watched_directory.rs` primitive using the crate's existing native `notify` dependency: recursive filesystem events and explicit atomic writes converge on one debounced callback generation, and stop/dispose cancels pending delivery without introducing polling.
- `FileMemoryStore` now owns this primitive, writes profile/log state through its atomic writer and exposes the same on-change boundary for external edits. This makes the frozen `source/host/watched-directory.ts` responsibility part of the production Memory path rather than a detached mirror.
- `source/host/tests/watched_directory_contract.rs` covers sorted directory discovery plus notification from both internal atomic writes and external filesystem writes. The watched-directory manifest row advances to `implemented`; the broader memory-service row remains non-final for synthesis origin/tombstone/shared-memory responsibilities.

### 2026-09-25 project-membership production ownership

- Starting exact HEAD: `b26f67753c5b2187ee0c3d8f562369b018dbeb91`.
- Ported the frozen `source/host/extensions/memory/project-membership.ts` contract to Rust: `projects.json` is path-safe filtered, sorted and atomically persisted; join/leave are idempotent; pruning removes missing projects without accepting traversal-like slugs.
- Production wiring is explicit: the Host MemoryService creates the membership owner and `ProductionSessionWorkers::compose_materialized_session` places it on every shipping `ProductionMaterializedSession`. This establishes one per-Agent ownership source for the upcoming project-memory/agent-state cutover rather than a detached compatibility object.
- `source/host/tests/project_membership_contract.rs` covers malformed input, safe-slug filtering, deterministic persistence, join/leave/prune and creation through real shipping Session materialization. The frozen project-membership row is now `implemented`.

### 2026-09-25 post-turn memory runtime cutover

- Starting exact HEAD: `d11d12b8019c5558a3a81f924b6f4e1e886039bb`.
- Added `source/host/src/runner/turn_memory.rs` with the frozen extraction and episode responsibilities: recent/archive memory context, add/remove application, durable pending episode turns in the Agent DB, configured episode interval, episode summary persistence and the dreaming/evidence branch that clears stale episode state.
- Shipping `runner.startRoutedProvider` now invokes memory maintenance only after a successful visible memorable turn and before publishing renderer-visible completion. The memory inference request has no tools and uses the already selected routed provider; failures remain opportunistic and do not overturn the user's completed turn.
- `source/host/tests/turn_memory_contract.rs` proves deduplicated extraction, six-turn episode summarization/clear and evidence-mode bypass. The manifest moves `turn-memory.ts` from `planned` to `existing-needs-parity`, not final, because the production Memory synthesis bridge and frozen auxiliary agent-message collector still need to be wired.

### 2026-09-25 post-turn memory compile blocker closure

- Exact failing HEAD `644c922c9e1e4a7eda2f22b2f9441ed3dee91268` reached the real shipping Host compile gate and exposed five local type/trait errors only: the watched-directory wrapper lacked a `Debug` projection required by `FileMemoryStore`, and the durable Agent DB represents episode timestamps as `f64` while `sand_memory::EpisodeTurn` / markdown persistence use `i64`.
- The fix is deliberately behavior-neutral: `WatchedDirectory` now has a manual debug view that does not require debug-printing callback/watcher internals, Agent DB writes preserve millisecond values as `f64`, and episode prompt/persistence projection converts finite in-range values back to `i64` with the turn timestamp as the invalid-value fallback.
- No parity status is advanced by this compile repair; exact-HEAD CI must re-prove the shipping Host and Runner contracts.

### 2026-09-25 turn-memory contract fixture closure

- Exact HEAD `bc8ca31029342fb6314754487174e4913936affa` passed the shipping Host build, independent Coordinator, independent box-exec daemon and shipping box-exec supervisor. The first Runner-suite failure was compile-only in the new `turn_memory_contract`: its persisted Agent DB `EpisodeTurn.ts` fixture used integer `1` instead of the canonical `f64` timestamp.
- The fixture now uses `1.0`; production code is unchanged. This commit advances no parity status and exists only to let the exact-HEAD Runner suite execute the intended behavior assertions.



### 2026-09-25 Runner auto-review secret-redaction deadlock closure

- Exact HEAD `43eeab1d17f320bcc0f16b990b41be487ce8e28a` passed Desktop Chat Parity and the Rust job's shipping Host build, independent Mahayana Coordinator contract, independent box-exec daemon contract, and shipping box-exec supervisor before the Runner suite stalled.
- The first real Runner blocker was `sand_auto_review_summaries_contract::automation_cloud_and_subagent_summaries_match_frozen_wording`: the Rust inline-secret redactor repeatedly rediscovered an already-redacted `token=…` assignment and replaced the ellipsis with itself forever, unlike the frozen Grok regex replacement.
- `source/host/src/runner/sand_auto_review_summaries.rs` now advances a byte-safe scan cursor after each replacement and continues past non-assignment keyword occurrences; the contract adds two same-key assignments in one prompt to prove forward progress and complete redaction.
- No architecture-manifest status is advanced by this commit. The mapped `source/host/runner/sand-auto-review-summaries.ts` entry remains `existing-needs-parity` until a fresh exact-HEAD Rust runtime run passes the Runner contract and downstream architecture gates.


### 2026-09-25 Runner Auto Review specialized parity slice

- Baseline for this slice: `e558b5a1eecfff1b14dadba2d90bfd8908b7fade`.
- Added the seven missing Grok Runner Auto Review responsibility modules under `source/host/src/runner/**`: automation writes, browser actions, cloud-agent actions/lifecycle, computer actions, shell approval binding enrichment, subagent actions, and shell/MCP escalation providers.
- These modules reuse the existing Mahayana Runner `SandAutoReviewController`, classifier decision model, fingerprinting, summaries, expiry policy, and pending approval lifecycle. They do not create a second approval runtime.
- The specialized contract `source/host/tests/sand_auto_review_specialized_contract.rs` covers off/shadow/enforce classification, approval resolution, browser/computer display binding, cloud image hashing and lifecycle fail-closed behavior, subagent risk targets, shared shell/MCP escalation, package-script definition hashing, and classifier abort propagation.
- `scripts/finalize-rust-parity.mjs` may promote only these seven previously-planned rows after the full Host cargo suite succeeds and the target/test files both exist. Existing-needs-parity core Auto Review rows remain manual-review-only.


### 2026-09-25 subagent runtime and adapter Rust ownership slice

- Starting from exact HEAD `01a3071aa960b78c1d954ba126c8018d84c4ee0b`, the frozen `source/host/runner/subagent-runtime.ts` and `agent-adapters.ts` planned rows now have concrete Rust targets instead of placeholder mappings.
- `subagent_runtime.rs` owns request-id/lineage generation, background task admission, pending-wake metadata, steer continuation, abort settlement, running/subagent projection, retained outline and computer-use usage/audit aggregation.
- `agent_adapters.rs` owns request-context projection, single-computer-use admission, resume/running fences, launch-review settlement, background dispatch metadata and the text/thinking/usage/tool forwarding contract including unresolved shell/read/await detection.
- `subagent_runtime_adapter_contract.rs` pins request identity, steer-vs-abort semantics, empty-output fallback, computer-use usage/audit projection, resume/window fencing, fail-closed request-context completeness and forwarding behavior.
- Both manifest rows move only from `planned` to `existing-needs-parity`; they are intentionally not final until generated Agent proto/session wiring and live Host callbacks replace the remaining adapter-neutral projections.


### 2026-09-25 computer-use coordination ownership slice

- Exact baseline: `32efadcd140dcdace9252bb88aee25f8967bd662`, where PR-triggered Rust desktop runtime and Desktop Chat Parity are both green and the push-only failure remains the expected final architecture gate.
- Added `source/host/src/runner/computer_use.rs` as the Runner-owned Rust port of the frozen coordination state: reentrant single desktop-window allocation, preparation lifecycle, fail-soft prewarm diagnostics, action-audit counting via the existing `sand_action_audit` mapping, turn-usage aggregation, model-id collapse and lazy navigation-probe ownership.
- `source/host/tests/computer_use_coordination_contract.rs` independently covers the single-controller invariant, preparation cleanup/failure classification, usage aggregation, mixed-model semantics, audit filtering and no-auditor navigation behavior.
- The manifest row advances only from `planned` to `existing-needs-parity`. Generated computer/shell resource accessors, real remote box prewarm and concrete navigation-probe execution remain mandatory before finalization.


### 2026-09-25 communicate/listener Runner tool foundation

- Added Rust ports for the two smallest frozen Runner tool foundations: `communicate-tool.ts` and `listener-connect-cards.ts`.
- The communicate port preserves Sand marker JSON, executing partial envelopes, success/error completion encoding, render fallback and exception conversion without inventing a second tool protocol.
- The listener-card port preserves ordered platform probing, fail-soft connectivity lookup, one card per disconnected platform, display-name rendering and the frozen resumed-automatically user guidance.
- `runner_communicate_listener_contract.rs` pins both surfaces. Both manifest rows advance only to `existing-needs-parity`: generated CommunicateUpdate/ToolCall proto wiring and live Automation listener/transcript production wiring remain required.


### 2026-09-25 subagent management tool ownership slice

- `sand-subagent-management-tools.ts` now has a Rust target that reads and mutates the existing `SubagentRuntime`; there is no parallel subagent registry.
- The port preserves frozen elapsed-time rounding, compact and detailed status text, recent activity/transcript projection, not-running guidance, steer-review denial, MessageSubagent continuation and StopSubagent confirmation.
- `sand_subagent_management_tools_contract.rs` proves the management surface against a live Rust `SubagentRuntime` fixture.
- The row advances only to `existing-needs-parity`; final requires registration through generated communicate tools and live `reviewSteer` / toolCallId / InteractionHandler production wiring.


### 2026-09-25 file-transfer Runner tool ownership slice

- `sand-file-transfer-tools.ts` now maps to a real Rust tool module that delegates byte movement to the existing `box::box_transfer::transfer_file_between_boxes`; no duplicate transport was introduced.
- The port covers connected/default computer resolution, offline/unknown diagnostics, box-preparing fencing, default `/workspace/uploads/<basename>` ingress, workspace path normalization, reverse transfer defaults and frozen binary-size labels.
- `sand_file_transfer_tools_contract.rs` uses in-memory TransferBox implementations to prove verbatim binary ingress and egress.
- The row advances only to `existing-needs-parity`; live UserComputer registry ownership plus generated communicate-tool registration through Runner prompt glue remain required.


### 2026-09-25 Runner prompt-glue ownership slice

- Added `runner_prompt_glue.rs` as the single Rust Runner join for the already-real prompt collector projection, file-transfer controller and MCP large-output spill policy.
- The glue intentionally reuses `prompt_collector_glue.rs`, `sand_file_transfer_tools.rs` and `large_output_spill.rs`; it does not copy their logic into Host/app.
- `runner_prompt_glue_contract.rs` proves durable user-message/attachment projection and enabled-vs-disabled MCP spill through the same glue owner.
- The row advances only to `existing-needs-parity`; frozen live getters for MCP/custom instructions/discovery, automation/profile/video/browser/remote-box state, shell watch and generated Agent factory construction remain required.


### 2026-09-25 subagent management contract fixture correction

- The first full-cargo failure after the subagent-management slice was test-only: the fixture expected `elapsedLabel(61_000ms)` to render `1m 1s`, but the frozen Grok implementation deliberately keeps all rounded durations below 90 seconds in seconds.
- The contract now expects `61s`. Production code and manifest status are unchanged.


### 2026-09-25 Rust 2024 box-module path correction

- Exact HEAD `a5bf42a133871b130782f35078d92273b4c00db2` reached the full cargo compile and failed before behavior tests because Rust 2024 reserves `box` as a keyword in module paths.
- The new file-transfer and prompt-glue source/tests now use the repository's required raw identifier path `r#box` (including `ports::r#box`). No runtime behavior or architecture status changed.


### 2026-09-25 file-transfer contract Debug fixture correction

- After the Rust 2024 raw-identifier fix, the next cargo failure was test-only: `unwrap_err()` requires the success value to implement `Debug`, while the in-memory `MemoryBox` fixture only derived `Default`.
- The fixture now derives `Debug`; production file-transfer/prompt-glue code and manifest status are unchanged.


### 2026-09-25 shell-terminal-watch ownership slice

- Starting from green full-cargo HEAD `d543327189f51b9145c18baabb42072f7a6e2bd9`, added a Rust Runner port of the frozen terminal-watch decision core.
- The module reuses existing `background_work` footer/time limits, `conversation_state` queued-message selection, hidden prompt markers and `system_prompt` message addressing.
- Independent contracts cover text/binary/not-found terminal reads, success/stream-error/missing/permission/300-minute timeout settlement, hidden/group watermark filtering, cache invalidation, fail-closed unreadable turns and rich-text queued-message prepend projection.
- The row advances only to `existing-needs-parity`; generated Read executor/local-tool scope, real box readiness/path, Agent proto blob decoding and the asynchronous polling owner remain required before final.


### 2026-09-25 remote-box Runner resource ownership slice

- Baseline `5d71b7e20906cc7640fa55cb2373a35714b98a7b` passed the full Rust parity cargo suite for shell-terminal-watch.
- Added `remote_box_resources.rs` as the Runner-owned decision layer for frozen remote-resource behavior: prepared/cached connection reuse, terminal-folder publication, fail-closed preparing and connection errors, retryable cache invalidation, no-monitor recovery, shell/background barrier-audit-navigation ordering, read/shell/computer execution plans and smart-mode classifier registration.
- `remote_box_resources_contract.rs` pins connection caching/retry, timeout/crash vs generic error messages, no-monitor invalidation, exact side-effect ordering and classifier gate conditions.
- The row advances only to `existing-needs-parity`; generated RegistryResourceAccessor executors, async connection-promise coalescing, real AutoReview/NavigationProbe/Host audit callbacks and executor delegation remain required.


### 2026-09-25 Computer tool and Host dependency projection slice

- Baseline `f7583daf7e833d603981b914e5767d42bc7f67a2` passed the full remote-box cargo contract job.
- Added `sand_computer_tool.rs` for Runner-owned action/schema semantics and `host_computer_tool_dependencies.rs` for the separate Host projection boundary; these responsibilities are intentionally not merged.
- Runner coverage includes drag/click/wait/follow-up validation, enforce-mode description and safe-follow-up restrictions, protocol action defaults, reported pointer position, automatic final screenshot, result rendering and screenshot persistence.
- Host coverage includes strict generated-action conversion, generated result normalization, approval→audit→shell-execute ordering and ensureReady→window lookup ordering.
- `computer_tool_projection_contract.rs` independently pins both layers. Both manifest rows advance only to `existing-needs-parity`; canonical generated proto executors, live resource accessor/AutoReview/display state and shipping first-party tool registration remain required.


### 2026-09-25 Rust Secrets owner and State Backstop service slice

- Baseline \`1093730d4fca351b3fa1103c8dd5c6939428451d\` had 1,761 implemented / 93 existing-needs-parity / 148 planned frozen modules and green exact-HEAD Desktop Chat Parity plus Rust desktop runtime; the strict final gate remained skipped because PR #20 is still draft.
- \`9b54364c769ee4fb8108d1d435a52325b6ce093b\` ports the frozen Secrets service to a Rust Host owner: box-secret validation, UTF-16 size accounting, deterministic redaction-name projection, mode-0600 atomic persistence, startup restore, generation-aware background apply, bounded retry/backoff and status projection. The shipping Host starts it against the existing ForeverBox environment-control path and stops it before ForeverBox teardown.
- The Secrets service manifest row is implemented with \`secrets_extension_contract.rs\`; the extension remains existing-needs-parity until the frozen external set/getStatus command surface is traced and wired instead of inventing a new RPC.
- The State Backstop service now has a Rust owner for \`state/store.db\` snapshot/readback, 64 MiB cap, per-agent debounce/dispose behavior and \`SAND_STATE_S3_BACKSTOP\` gate semantics. Its extension remains existing-needs-parity because the production Box Store Sync object-store provider is still a planned responsibility; no fake provider or second storage runtime is introduced.


### 2026-09-25 Client-side Tool V2 producer and Box Store hydration closure

- Re-audited PR #20 at exact HEAD `4824665f8cc8f5d2d895b2787c23497f0480100b`: 1,779 implemented / 96 existing-needs-parity / 127 planned before this slice; shipping Host, independent Coordinator and box-exec daemon gates were green while the remaining Host/Runner gates continued.
- Added a Rust `ClientSideToolV2Producer` that preserves the frozen Host ordering/lifecycle contract and emits the exact shared `protobuf-base64` envelope consumed by `source/shared/rpc/client-side-tool-v2-transport.ts`. It owns stable epoch, per-agent sequence, open-call/result fencing, unknown-result drop and reset behavior.
- The already-present Rust Box Store hydration port was not rewritten; it now has an independent contract covering normal/legacy completeness evidence, handoff manifest path matching, atomic mode-0600 marker persistence, directory sync and removal. Its manifest row is advanced only after that evidence.


### 2026-09-25 Chrome session stage evidence closure

- Audited the pre-existing Rust `chrome_session_stage.rs` against the frozen Grok module rather than reimplementing it. Added contract coverage for exact DB relative-path projection and mode preservation, retry destination pre-clean, busy/locked raw-copy fallback, non-busy skip/report behavior, raw-copy failure reporting, and staging cleanup.
- The manifest row advances from planned to implemented only after this behavioral evidence; the wider Box Store Sync extension/service remains non-final.


### 2026-09-25 Client-side Tool V2 projection inventory closure

- Ported the frozen ClientSideToolV2/Agent ToolCall inventory and explicit projection policy to Rust. Supported mappings are allowlisted; SendMessage, approval cards and Await remain ordinary-transcript-only exactly as frozen; unrecovered Agent oneofs and ClientSide variants remain fail-closed.
- Contract coverage verifies every projected source/target belongs to the frozen shipped unions and that unrecovered entries cannot silently become projected.


### 2026-09-25 Host roster, upgrade marker and cloud transcript helpers

- Ported Host roster bookkeeping to Rust with frozen active-agent fallback, live-running/busy calculation, newly-running disk-pressure enrollment, stopped-agent snapshot scheduling and source-map materialization semantics.
- Ported Host upgrade marker parsing, timing metadata and idempotent forwarding/deletion lifecycle; deferred emits do not retire markers, parse errors do, and callback failures are isolated after successful emit.
- Ported cloud-agent transcript dump path/write/format/augmentation behavior with byte-accurate size reporting and fail-soft watch augmentation.


### 2026-09-25 Host bundle source and MCP plugin skill cache

- Ported Host Upgrade bundle-source semantics to Rust: override normalization, frozen S3 paths, strict lowercase git-SHA validation, version TTL cache, fail-soft latest lookup, fail-closed tarball fetch and lazy resolved source.
- Ported MCP plugin-skill cache to Rust with frozen safe-id/absolute-path validation, legacy field defaults, positive user/team IDs, auth-block fallback, pretty JSON atomic replace and agent-readable 0755/0644 permissions.


### 2026-09-26 Action Audit / Automations evidence reconciliation

- Re-read PR #20 at exact HEAD `7c82dd0fe0e7b060fc0b8c9045424004034d796a`; exact-HEAD Desktop Chat Parity CI run `36186038197` and Rust desktop runtime run `36186038184` were both green.
- Reconciled nine stale `planned` rows whose Rust targets already exist on the implementation branch. The Action Audit backend and the Automations relay/watcher/integration/cloud-sync/cloud-trigger/fire-consumer/trigger-hub/extension owners now carry concrete behavioral/test evidence and move only to `existing-needs-parity`, not final.
- No row was marked `implemented`: generated Dashboard audit sending, authenticated automation backend clients, notify/poll scheduling, durable fire ack/completion semantics, live Host dependency composition and production transcript/session callbacks remain explicit blockers.


### 2026-09-26 channel attachment Host ownership slice

- Added `source/host/src/connectors/channel_attachment.rs` and the `connectors` Host module for frozen `source/host/connectors/channel-attachment.ts` responsibility.
- The Rust owner preserves HTTP(S) URL pass-through, local/file URL resolution, data-root reanchoring, regular-file and 50 MiB upload fences, byte reads, basename projection and image/video/generic MIME selection.
- `channel_attachment_contract.rs` covers remote URL/image detection, file URL and plain-path uploads, MIME projection, directory/empty/oversize rejection and unsupported schemes.
- The manifest row advances only to `existing-needs-parity`; final waits on canonical shared media-MIME reuse and full exact-HEAD CI evidence.


### 2026-09-26 Cloud Agent image loading slice

- Added Rust `cloud_agent_images.rs` under the existing Host cloud-agents boundary rather than merging it into the Cloud Agent tool/API owner.
- The port enforces file:// input, agent attachments/assets containment, /workspace-only box reads, image-only MIME gating, unreadable/refused distinctions and the frozen 25 MiB attachment limit.
- `cloud_agent_images_contract.rs` covers host media roots, box reads, non-file/non-image rejection, outside-root refusal, unreadable box paths and both host/box oversize rejection.
- The row advances only to `existing-needs-parity`; final waits on exact shared media MIME/error helper reuse and live CloudAgent-tool production wiring.


### 2026-09-26 MCP state executor slice

- Added Rust `ports/mcp_state_executor.rs` for the frozen MCP-state projection responsibility, reusing the shipping `RoutedToolDefinition` instead of creating a second tool inventory.
- The executor preserves first-seen provider ordering, groups tools per provider, exposes each server as `connected`, and retains description/input-schema data; provider errors remain fail-closed.
- `mcp_state_executor_contract.rs` covers multi-provider grouping/order, schema projection, empty success and provider failure propagation.
- The row advances only to `existing-needs-parity`; final waits on generated `agent.v1` result types and production executor/tool registration.


### 2026-09-26 generated-image persistence service slice

- Added Rust `extensions/attachments/generate_image_service.rs` as the Host-owned post-generation persistence boundary.
- The service decodes backend base64, persists exact bytes with MIME, returns the saved path plus original base64 and fails closed on invalid data or missing persistence.
- `generate_image_service_contract.rs` proves byte/MIME preservation and failure behavior.
- The row remains `existing-needs-parity` until authenticated Cursor image generation, model/request-id projection and production Attachments-extension wiring are connected.


### 2026-09-26 mobile push notifier slice

- Added Rust `extensions/notifications/mobile_push_notifier.rs` with the frozen notification lifecycle: baseline seeding, pre-seed buffering, focus freshness, done/needs-input transitions, message-id duplicate suppression, per-kind throttle and forget/reset behavior.
- Delivery errors are deliberately non-fatal, matching the fire-and-forget notification surface.
- `mobile_push_notifier_contract.rs` proves buffered needs-input delivery, new-message gating, focused-window suppression, stale-focus delivery and forget behavior.
- The row advances only to `existing-needs-parity`; final waits on canonical shared notification helpers plus authenticated GrokBotService/Host event wiring.


### 2026-09-26 Host crash-marker ownership slice

- Starting from exact implementation HEAD `7ea96551789c9b80bc281e70a5ec219b84069026`, restored the frozen Host crash-marker boundary in Rust rather than folding it into Coordinator or renderer telemetry.
- The module owns strict schema/error-class/signal/timestamp parsing, frozen metadata projection, file-store read/delete outcomes, compare-before-delete race fencing, forwarded-marker dedupe and deferred/pending/delivered/parse-error settlement.
- `source/host/tests/host_crash_marker_contract.rs` independently covers valid/invalid marker variants, rounded metadata, file-store lifecycle, changed-marker fencing and forwarding failure/dedupe behavior.
- The manifest row advances only to `existing-needs-parity`; final status still requires production Host startup/crash telemetry composition to invoke this owner and exact-HEAD CI evidence.


### 2026-09-26 Host event-loop telemetry decision slice

- From exact HEAD `fe4fed178bce03e81a2c078ea18fcbac6601ef15`, ported the frozen event-loop pressure/heartbeat decision and telemetry projection into the Rust Host telemetry boundary.
- The contract preserves the 50 ms p95 pressure threshold, five-window heartbeat cadence, pressure precedence, rounded p50/p95/max values and three-decimal utilization projection; a zero custom heartbeat is handled fail-safe rather than allowing a Rust modulo panic.
- `event_loop_telemetry_contract.rs` covers defaults, overrides, precedence and projection. The row advances only to `existing-needs-parity`: production still needs a Host-owned delay/utilization sampler and lifecycle wiring equivalent to Node's perf-hooks monitor.


### 2026-09-26 Turn telemetry mapper ownership slice

- From exact HEAD `0d7369cad7fdb689ebfbc7616a10fbac2fe8995f`, ported the frozen turn telemetry mapping boundary into Rust Host: interrupt, await, retry, user-message received, closing-send nudge, TTFT, turn usage and computer-use usage.
- The owner preserves event names/levels, optional-field omission, token schema v2, total-input behavior, retry error-type bounding, retry/TTFT/duration rounding and computer-use error severity.
- `turn_telemetry_mappers_contract.rs` covers all projection families. The manifest row advances only to `existing-needs-parity`; final requires shipping Runner/Transcript/Computer call sites to route through this owner rather than parallel mappings.


### 2026-09-26 Desktop health forwarder ownership slice

- From exact HEAD `1bdb19535587086c82117e3fe21cfcc7fb3c0a7e`, ported the frozen desktop-health normalization/aggregation/forwarding decision boundary into Rust Host.
- The owner validates component scopes/kinds, normalizes bounded down reasons and restart counts, preserves first-seen duplicate merge order, computes healthy/degraded/crashloop metadata, and forwards on revision change or heartbeat with absent/parse/skipped/emitted settlement.
- `desktop_health_forwarder_contract.rs` covers normalization, duplicate merge, invalid snapshots, metadata, heartbeat/revision decisions and forwarding state. The row remains `existing-needs-parity` until the production Host health-file reader and telemetry emitter are composed against it.

### 2026-09-25 automation-store final watcher cutover

- Starting exact HEAD: `0acbbe87bfe8985dd4369d97da5dfc79adf7089e`.
- `FileAutomationStore` now uses the Host-owned native `WatchedDirectory` for its root, atomic config/run writes, external filesystem edits, debounced callbacks and removal notifications instead of the prior callback-only shim.
- Authored cron schedules are normalized before durable storage, and the frozen definition-only `recordRunDefinition` / `finishRunDefinition` paths are present so cloud/backend reconciliation can mutate run state without synthesizing `nextRunAt`.
- The automation-store contract now exercises internal atomic notifications, direct external file edits, schedule normalization and definition-only next-run suppression. Together with the existing timezone/cron/trigger/run-history coverage and shipping Session factory wiring, the frozen `automation-store.ts` manifest row is final `implemented`.

### 2026-09-26 workflow library watcher/cache finalization

- Starting exact HEAD: `41e6a0c30a8ab7f7eb40bd98e705605a69fef7b3`.
- Replaced the workflow library's callback-only invalidation with the shared native `WatchedDirectory`; atomic writes, removes, direct external SKILL.md edits and helper-script directory changes now converge on the frozen 50 ms debounced boundary.
- Completed `StatKeyedParseCache` parity for missing-path eviction, stat-error parse fallback, injectable time, inode/mtime-ns/size identity, bounded FIFO eviction and the two-second racy-mtime bypass. `GlobalWorkflowLibrary::get` now consumes one module-level cache keyed by both the workflow file and containing folder, matching helper-script invalidation behavior.
- Integration coverage proves stable cache reuse, invalidation after content change, missing-file eviction, internal/external watcher delivery and helper-script refresh. The frozen `workflow-library.ts` and `stat-keyed-parse-cache.ts` rows are now final `implemented`.
- `workflow-store.ts` deliberately remains non-final: managed/plugin skill aggregation/watchers and published-plugin editing are still separate unrecovered responsibilities.

### 2026-09-26 automation-trigger final audit

- Starting exact HEAD: `c887d1116d350c37edba9a75505e3af9dfcf4cb0`.
- Re-audited all frozen `automation-trigger.ts` exports against the current Rust Host. Parse/serialize/identity, Slack matching, GitHub actor/owner/CI-branch admission, Teams/Linear/Sentry/PagerDuty filters, group matching, human event descriptions and escaped event-context projection are all present in the canonical `automation_trigger.rs` owner.
- Shipping `SandTriggerHub` and `BackendRelaySource` consume these canonical matchers; matching and trigger-foundation contracts cover direct and grouped admission plus store round-trip.
- This mapping is now final `implemented`. The neighboring `automation.ts` row remains non-final because its frozen system-prompt/wake-batch surface is still incomplete.

### 2026-09-26 canonical automation wake projection

- Starting exact HEAD: `af184f02f51f7f493b8c10a88e54b1dfa50e1462`.
- Reworked the shipping AutomationRunPath wake projection to consume the finalized canonical trigger helpers instead of serializing generic `<event_data>` blocks. Event runs now use source-specific escaped context blocks and human trigger summaries, preserve the 25-event cap, and provide the frozen group-seed projection.
- Added a timezone-aware wake-prompt variant; the production fire path resolves the Agent automation store's live user timezone before formatting fired/started timestamps.
- Contracts now cover source-specific GitHub event context, human batch summaries, untrusted XML escaping, event clamping, group seeds and timezone rendering. The broader frozen `automation.ts` row remains non-final until its routines capability system prompt is production-wired.

### 2026-09-26 routine notice ownership correction

- Restored the frozen `routineNoticeWakeLines` responsibility to `routine_notices.rs` and removed the duplicate notice strings from `AutomationRunPath`.
- The shipping fire path still computes/marks one-shot notice IDs before execution, but wake text now comes from the canonical notices owner. This repairs evidence behind the already-final routine-notices mapping rather than merely changing its status.



### 2026-09-26 automation capability prompt production cutover

- Closed the frozen `source/host/automations/automation.ts` semantic gap without copying reference source text: `automation.rs` now owns the routines capability guidance and the reference guidance/status constants, while existing Rust owners continue to own wake rendering, status reminders, timestamp rendering, persistence and trigger matching.
- The shipping routed-provider path opens the Agent-owned `FileAutomationStore`, renders the current routines capability prompt with the resolved user timezone and durable automation location, and appends it through `system_prompt_assembly` before provider execution. Renderer state is not used to reconstruct routine policy.
- Added `automation_prompt_contract.rs` covering schedule/listener/lifecycle guidance, enabled/paused current-routine projection, timezone/location context, and idempotent injection into the canonical system message. Existing wake/status/store contracts remain part of the manifest evidence.


### 2026-09-26 action-audit production cutover

- Closed the frozen action-audit backend/extension mappings in Rust. The Host now encodes the Dashboard `SandAuditEvent` oneof and `RecordSandAuditEventsRequest` wire contract, uses the existing authenticated Cursor unary transport with the live Auth token/machine id, and keeps the existing bounded durable outbox/backoff behavior.
- `ActionAuditExtension` now composes Auth + `sand_action_audit_logs` Experiments + structured Telemetry around one `SandActionAuditor` owner. Runner MCP audit records are still observable on the Host event hub, but now also flow into that owner for local JSONL and feature-gated backend delivery.
- Added a backend contract test for the Dashboard endpoint, repeated-event request envelope and all four frozen action oneof families.


### 2026-09-26 shared media inventory cutover

- Added a single Rust Host media-semantic owner mirroring frozen Grok 0.18 media extensions and attachment limits. Channel attachments and CloudAgent image loading no longer carry private approximate MIME tables.
- Corrected semantic drift: `.ico` is a standard image; HEIC/HEIF are only client-servable native image formats and are not accepted by `imageMimeFromPath`; frozen video mapping is only m4v/mov/mp4/ogv/webm.
- CloudAgent file URLs now receive POSIX-style lexical normalization before the `/workspace` boundary check, so `file:///workspace/../...` is refused instead of reaching the box reader.
- Channel attachment is now final in the manifest with shared-media and behavior contracts. CloudAgent image loading remains non-final only because the production `cloud-agent-tool` owner has not yet wired the loader to the canonical box reader.


### 2026-09-26 CloudAgent request-composition cutover

- Ported the frozen CloudAgent request-composition layer to Rust as a pure Host-owned module. It now owns requested-model/max-mode parameters, selected-image user messages, conversation actions, repository normalization/sanitization, saved-environment primary-repo policy, repo-config projection and private-worker routing labels.
- Added contract coverage for scp-style Git remotes, credential-stripping normalization, short `owner/repo` references, secondary-repo rejection, model-params-without-model rejection, image context projection, pool labels and named-machine shared-assignment policy.
- This module is final independently; network RPC, model-catalog retrieval, completion polling and Host extension lifecycle remain tracked by their own CloudAgent manifest rows.


### 2026-09-26 CloudAgent model-catalog fetch cutover

- Added a real Rust `AiService/AvailableModels` Host client on the existing authenticated Cursor unary transport. The request encodes exactly the frozen CloudAgent catalog flags: model parameters on, Markdown off, USER_AVAILABLE scope.
- Added a generated-wire decoder for the subset consumed by the frozen model catalog: model id/display name/aliases, boolean and enum parameter definitions/values, and parameter variants.
- Added byte-level request and response contract tests. Model-catalog fetch is final independently; the five-minute catalog cache and CloudAgent manager/extension lifecycle remain owned by their separate poll-loop/service/extension mappings.


### 2026-09-26 CloudAgent poll/cache cutover

- Ported the frozen CloudAgent polling/cache layer to Rust: status normalization, included-limit projection, watch result/diff formatting, saved-environment id/name resolution and environment-list hints.
- Added the five-minute model-catalog cache, five-minute fail-open team-admin policy cache with background refresh, and completion polling with 10s cadence, 30s RPC timeout, five-hour max wait, three-minute restart grace and rate-limit retry+jitter.
- Polling is exposed behind injected clock/sleep/fetch boundaries so deterministic contracts can cover restart/rate-limit/terminal behavior while the production CloudAgent manager remains responsible for running it off the request lane.


### 2026-09-26 CloudAgent manager service cutover

- Added the real Rust `SandCloudAgentManager` and `CursorCloudAgentBackend` instead of a test-only CloudAgent API. All BackgroundComposer/Dashboard calls reuse the canonical Host Cursor transport with access token, machine checksum, client metadata, Sand namespace and ghost-mode headers.
- Added a minimal generated-wire-compatible `prost` boundary pinned to the frozen Grok message field numbers for launch, follow-up, list/get, lifecycle management, artifacts, transcript payloads, PR state, optimized diffs, saved environments, teams and team-admin policy. Unknown generated fields remain safely ignored.
- The manager now owns frozen launch/reply request composition, private-worker team resolution, saved-environment routing, model-catalog cache, completion polling, team-admin policy, managed CloudAgent IDs, file-change capping, live PR state and transcript-dump retrieval.
- `cloud_agents_service_contract.rs` plus the wire/request/poll/model catalog contracts passed the complete Host Runner suite on exact HEAD `1c26d31ad8bd6f26b68fd7e34c8b9ca1481e80de`; shipping Host and independent Mahayana Coordinator also passed. The service mapping is therefore final. Host extension startup, ConversationMessage trace conversion, CloudAgent first-party Runner tool registration and raw box I/O remain separate non-final mappings.


### 2026-09-26 CloudAgent Host extension and Runner tool production cutover

- Added a single Rust `CloudAgentsExtension` owner around the production `SandCloudAgentManager`; Host startup binds real Auth/backend dependencies, prefetches team-admin policy, and threads the same manager through direct, group-member and automation routed turns.
- Added the first-party Runner `CloudAgent` tool with the frozen action surface: launch/list/models/get/dump/watch/reply/rename/cancel/archive/unarchive/delete/list_artifacts. Destructive confirmation, cancellation, model validation, saved/private-worker environments and managed-id fencing are preserved.
- Launch/reply image inputs now reach the already-audited `load_cloud_agent_images` implementation in the shipping path. `/workspace` bytes come from the existing ForeverBox Runner Read port and must be binary data; dump output uses the existing Box Write port.
- The generated CloudAgent conversation-to-trace converter remains an explicit production Host adapter, matching frozen `ProductionExtensionHostAdapters`. The current binding fails closed for dump instead of inventing a trace shape. Auto-review and reviving cloud-agent watcher ownership are also still explicit non-final dependencies, so the CloudAgent tool/extension rows remain `existing-needs-parity`; the image-loader row is now final.


### 2026-09-26 Memory agent-state ownership slice

- Starting implementation HEAD: `646e2620e010a5e2f9dc4658ad96735c97c4ac6e`.
- Added canonical user/project memory shard path helpers in `source/host/src/extensions/memory/memory_service.rs` and restored `source/host/extensions/memory/agent-state.ts` as a Rust Memory-extension owner at `source/host/src/extensions/memory/agent_state.rs` rather than extending the Runner placeholder.
- The owner now covers explicit agent/user/project memory writes/removals with project-membership fencing, routine CRUD, workflow CRUD, profile/settings mutation, channel disconnect, project create/join/leave, and avatar install/clear through the existing canonical Host stores.
- Added `source/host/tests/agent_state_contract.rs` for memory-scope routing/fencing and routine/workflow/profile/settings/channel/avatar behavior. The manifest is intentionally only `existing-needs-parity`; final status requires the shipping sand-state/update_state tool to delegate to this owner and an exact-HEAD CI pass.
- Commits in this slice: `30c5fbc4ac267165a6e122f436307888408e37dc`, `1345de2e9705583e4bdbd1a1f0d1a298e6a0fab2`, `4b0efcc836e1bd960b4bdb4ef1355159945b1e30`, `7ca2f6e126d476f1d499915e647fda3b6adfd140`, `0c27b1e42f00c3fc02c589d39a75597aa08231f1`, `8a78c3890205e6f00f148844b4c5102955d487d0`, and `ca8ed60d25256eabc33c398e639acaf5855e7ea3`.


### 2026-09-26 update_state production wiring

- Implementation baseline was `279f166fe0a66399fb1f4fe1dc699a11b8f19bde`; production-wiring exact HEAD was `ad5b0832245c1106a22a6557c562de2987ada34a`.
- Added `source/host/src/runner/tools/sand_state_tool.rs` as the Runner-owned `update_state` schema/router. It does not persist state itself: `SandStateWriter` delegates durable mutations to the Host-owned `source/host/src/extensions/memory/agent_state.rs`.
- Shipping `TurnAgentComposition` now composes the state bridge, and `source/host/app/src/main.rs` constructs one `SandAgentState` for each routed agent turn from the canonical Memory service sand root.
- Added `source/host/tests/sand_state_tool_contract.rs` covering tool advertisement/delegation, memory persistence, routine create + partial update preservation, and workflow persistence.
- Exact-head Rust runtime run `36230057493`: `rust-host` job `108371404118` succeeded, including shipping Host compile, Coordinator, box daemon, full Host Runner cargo tests, attachments, Computer takeover and ConversationActor/CapabilityBroker contracts. The workflow's renderer job remains red only at the intentionally strict final architecture gate.
- Exact-head Desktop Chat Parity run `36230057523`: Focused Electron chat E2E and Renderer typecheck/build both succeeded.
- With those gates, `source/host/extensions/memory/agent-state.ts` and both Notifications rows are now final `implemented`. `source/host/runner/tools/sand-state-tool.ts` advances from `planned` to `existing-needs-parity`: remaining work is frozen auto-review/approval semantics for routine/workflow writes, listener post-save integration, full trigger validation, communicate activity projection, and box-path avatar reads.
- Manifest after this slice: 1,815 implemented / 103 existing-needs-parity / 84 planned (187 not final).


### 2026-09-26 browser production wiring

- Browser production-wiring exact HEAD: `51496cb3fcba95b98a06a54ee32905979a62efd8`.
- The frozen Grok browser driver v2 payload remains JavaScript inside the Rust Runner owner because Playwright/CDP executes inside the box Node runtime. Rust owns upload, per-turn registration, Host resource routing, validation and result projection.
- Added a narrow Host `RunnerBoxResourcePort::browser_window_index` seam. `ForeverBoxRunnerResourcePort` resolves the real per-agent window only after `ensure_ready(agent_id)`; the default port fails closed.
- Added `ProductionBrowserToolExecutor`: it uploads the frozen driver through the Host write port, launches it through the Host shell port, redirects its marker output to a box result file, reads/parses that file through the Host read port, and retrieves screenshot bytes through the same port. Runner never owns ForeverBox lifecycle or transport credentials.
- Shipping `TurnToolset`, `TurnAgentComposition`, `ProductionRunnerCompositionInput` and `source/host/app/src/main.rs` now project this executor into real provider turns.
- Exact-head Rust runtime run `36232610771`: rust-host job `108378519115` succeeded, including shipping Host compile, full Host Runner cargo tests and subsequent Mahayana contracts. Desktop Chat Parity run `36232610733` succeeded for both Focused Electron chat E2E and Renderer typecheck/build.
- `sand-browser-driver-source.ts` is final `implemented`. `sand-browser-tools.ts` remains `existing-needs-parity` until frozen browser Auto-review preflight/capture-review-state, navigation-audit callback and screenshot persistImage callback are bound to the shipping executor.


### 2026-09-26 Coordinator OAuth retry CI stabilization

- Exact HEAD `61191947a34ec9e8a4d4bf11aa6cfba2f31ee641` kept Desktop Chat Parity green, but Rust desktop runtime run `36232906010` failed the independent shipping Coordinator production protocol at the MCP OAuth callback: the fake Host gateway intentionally returned one transient HTTP 503, while the callback surfaced HTTP 500 instead of completing on the bounded second attempt.
- The Coordinator implementation itself was unchanged from the previously independently verified Coordinator checkpoint; the failure was isolated to the production-protocol fake gateway on macOS. Its listener is intentionally non-blocking for shutdown polling, while accepted sockets could inherit non-blocking mode and race the retry request before the client finished writing it.
- Exact implementation HEAD `59e2b02c16c696bf75e22677b7430994c84153e6` normalizes each accepted fake-gateway HTTP socket back to blocking mode before applying the existing two-second read/write timeouts, matching the already-required loopback/box-exec test transport discipline.
- OAuth semantics and acceptance criteria were not weakened: the fake gateway still returns one transient 503; the shipping Coordinator must still retry exactly once, complete successfully, and preserve the assertion that `oauth_completion_attempts() == 2`.
- Exact-HEAD verification runs were triggered: Desktop Chat Parity `36233439852` and Rust desktop runtime `36233439856`. Their final results remain authoritative before advancing any additional manifest row or merge claim.


### 2026-09-26 AgentStore and Host crash-marker finalization

- AgentStore production implementation is complete. The shipping Rust `ProductionAgentStore` now owns the frozen AgentStore2 surface rather than the byte-retained TypeScript reference copy: typed metadata and subscriptions, content-addressed checkpoint/root advancement, fail-closed DB reset, reverse last-request recovery, lenient full-conversation hydration, inline/ref subagent resolution, and the Runner/transcript-mirror checkpoint adapter all share the canonical Session/AgentDb/blob owners.
- Exact implementation HEAD `45de56763f1885701e16f255bbe69f440ea4140a` passed shipping Host compile, the independent Mahayana Coordinator contract, box daemon/supervisor, full Host Runner cargo tests, prompt attachment, Computer takeover and ConversationActor/CapabilityBroker in Rust runtime run `36233783915`. Superset exact HEAD `ad55380002e3b8eb56499aa1bbe25feb85227136` passed the same rust-host chain in run `36233942302`.
- Host crash-marker production ownership is now inside the Rust Telemetry extension. It uses the canonical `.sand-host-crash.json` path, immediately attempts forwarding, retries non-terminal outcomes on the frozen five-minute cadence, persists `sand.host.crash` through the existing structured-log sink with `SAND-E0001` / `registry` / non-retryable tags, and only deletes an unchanged marker after confirmed persistence.
- `source/host/tests/host_crash_marker_contract.rs` now proves real file-store -> production telemetry JSONL -> delete behavior in addition to parser/race/dedupe/defer coverage. Exact HEAD `ad55380002e3b8eb56499aa1bbe25feb85227136` passed the complete rust-host job in run `36233942302`; Desktop Chat Parity run `36233942301` passed renderer typecheck/build and the Grok Agent shell/Mahayana chat regression.
- Accordingly `source/packages/agent-kv/agent-store.ts` and `source/host/extensions/telemetry/host-crash-marker.ts` advance to final `implemented`. No unrelated row is promoted. Manifest after this evidence update: 1818 implemented / 104 existing-needs-parity / 80 planned.
- The PR remains draft. The strict architecture gate still has non-final Host/Runner/extension rows plus the six known legacy production-root blockers; this finalization is not a merge, packaged-acceptance, or release claim.


### 2026-09-26 Desktop health telemetry production cutover

- Starting from exact HEAD `dc75982509f0f72d284f1541f1b1ac7520700fe8`, the frozen Grok desktop-health lifecycle was audited against the Rust Telemetry extension instead of promoting the existing decision helper on code presence alone.
- Production Host now owns the same source and cadence as Grok 0.18: `/tmp/sand-supervisor/desktop-health.json`, an immediate startup read, 30-second polling, revision-change forwarding, and a five-minute same-revision heartbeat. `SAND_DISABLE_TELEMETRY=1` disables the poller without moving health polling back into the renderer.
- The existing Rust parser/normalizer remains the single decision owner. The new production bridge persists its projection through `HostStructuredLogTelemetry` as `sand.box.desktop_health`, and the poller is stopped/joined with `HostTelemetryExtension` lifecycle.
- `host_telemetry_service_contract.rs` now proves file -> frozen forward decision -> durable structured-log JSONL, including degraded metadata, same-revision suppression and heartbeat re-emission. The existing `desktop_health_forwarder_contract.rs` continues to cover component/down-reason normalization, merge/clamping and forward decisions.
- Only `source/host/extensions/telemetry/desktop-health-forwarder.ts` advances to final `implemented`; no adjacent TelemetryService/event-loop/transport row is promoted. Manifest for this commit is 1819 implemented / 103 existing-needs-parity / 80 planned.
- PR #20 remains draft. The new exact HEAD must pass shipping Host/Coordinator/Runner and Desktop Chat Parity; the strict architecture gate is still expected to remain red on the other non-final rows and legacy-root blockers.


### 2026-09-26 canonical telemetry port production binding

- Starting from exact HEAD `e858424df9bd4d687b1d7b3f3eca37239624f33a`, the frozen telemetry port was audited against shipping structured-log construction. The Rust port already owned the complete frozen taxonomy, error-detail helpers, no-op surface and `SAND_BOX_*` identity normalization, but production HostTelemetryService did not consume that identity owner.
- Shipping `HostTelemetryService::open` now resolves box identity exclusively through `ports::telemetry::resolve_sand_box_identity_tags`; structured-log projections merge those identity tags before event metadata, preserving the frozen `{ ...identityTags, ...metadata }` precedence. Empty identity values are filtered and event-local keys remain authoritative.
- A deterministic `open_with_identity_tags` seam verifies the production merge without mutating process environment in parallel tests. `host_telemetry_service_contract.rs` proves identity propagation, empty-value removal and event-key override; `ports_telemetry_contract.rs` continues to pin environment trimming/taxonomies/no-op behavior.
- Only `source/host/ports/telemetry.ts` advances to final `implemented`. HostTelemetryService, structured-log transport, event-loop sampling and backend transport stay independently non-final. Manifest for this commit is 1820 implemented / 102 existing-needs-parity / 80 planned.
- PR #20 remains draft; exact-HEAD Host/Coordinator/Runner, Desktop Chat Parity and strict architecture results remain authoritative.


### 2026-09-26 production binding providers owner

- Starting from exact HEAD `3c9ef840e9043efb18ef4c580593fbc3c59e0092`, the smallest remaining planned Host construction module, `source/host/production-binding-providers.ts`, was audited against frozen Grok 0.18 before implementation.
- Added `source/host/src/production_binding_providers.rs`. Its StateBackstop runtime uses the canonical Sand agents root and the existing SQLite checkpoint owner before reading `store.db`, preserving the frozen production binding without duplicating storage logic. Secrets uses the Rust Host-native context adaptation through the same production-binding module, and shipping Host main now consumes it rather than an anonymous logger closure.
- The CloudAgent converter placeholder was also moved out of main into this canonical owner. It intentionally remains fail-closed: frozen `NO_PREAMBLE` trace conversion requires the full generated `aiserver.v1.ConversationMessage` and nested `ClientSideToolV2Result` oneof values; the current Rust tree does not yet contain those canonical generated bindings, so opaque bytes/base64 are not accepted as parity.
- `production_binding_providers_contract.rs` verifies canonical agents-root selection, real WAL-capable SQLite checkpoint/read/reopen behavior, missing-db behavior, and the fail-closed CloudAgent generated-binding fence.
- This mapping advances only from `planned` to `existing-needs-parity`, not final. Manifest is now 1820 implemented / 103 existing-needs-parity / 79 planned. Remaining final blockers are the generated CloudAgent trace adapter and live StateBackstop BoxStore/SourceMap production composition.


### 2026-09-26 production binding canonical-root contract correction

- Exact HEAD `c7e713987bc28a508972f6412bf4f06b7dfdced5` exposed a contract bug, not a shipping root bug: `production_binding_providers_contract.rs` hard-coded `~/.sand/agents`, but frozen Grok 0.18 delegates `getSandAgentsRootDir(home)` to `getSandRootDir(home)`. The frozen packaged root is `~/.grokbot/agents`; dev/lab roots follow the Sand variant, and explicit data-root/user-data overrides remain authoritative.
- The production provider already delegates to Rust `get_sand_agents_root_dir`, matching the frozen construction boundary. The contract now compares against that canonical resolver instead of inventing a second root convention.
- No shipping path, manifest status, StateBackstop semantics, or data migration behavior changed in this correction. The next exact-HEAD Rust run remains authoritative for the production binding provider row.


### 2026-09-26 Grok-shaped production Host extension owner

- Starting from exact HEAD `05ff75c470561bc428c6958f6cabb57eae84bf72`, the Rust Host's live production extension construction was still embedded in `source/host/app/src/main.rs` even though frozen Grok owns this responsibility in `source/host/host-production-extensions.ts`.
- Added `source/host/src/host_production_extensions.rs` and moved the real shipping construction for Auth, Experiments, ActionAudit, CloudAgents, NotifyBus, Memory, ManagedSetup, SourceMap, Trays, BoxLifecycle, WebAuthnProxy and BrowserUa behind that owner. `app/main.rs` now consumes the module instead of defining a parallel production graph.
- The existing 35-slot `HOST_EXTENSION_ORDER` remains the canonical frozen registry table. `host_production_extensions_contract.rs` proves the current shipping subset is duplicate-free, occupies only frozen slots, and keeps the production owner Send+Sync.
- This is intentionally not a fake 35/35 registry. `source/host/host-production-extensions.ts` advances only from `planned` to `existing-needs-parity`; the missing ContentSearch/CrossUserSharing/StateBackstop production composition/Inference/LocalExec/LocalToolPermission/MCP/BoxStoreSync/HostUpgrade/AutoReview/CodebaseTelemetry/TeachRecording declarations and production extras remain explicit blockers.
- Manifest after this cutover: 1820 implemented / 104 existing-needs-parity / 78 planned. Strict non-final row count is unchanged; this commit reduces planned architecture debt and removes a real `main.rs` ownership violation without promoting incomplete slots.


### 2026-09-26 production Host extension owner compile repair

- Exact HEAD `5e37c999688f5cdf950d3430d99c6e08cdda20db` reached the shipping Mahayana Host compile and exposed one extraction-only regression: `ActionAuditExtension` is still a live Runner dependency type in `app/main.rs`, but its import was removed when the production extension constructor moved to `host_production_extensions.rs`.
- Restored that type import and scoped `ProductionBrowserUaLog` / `ProductionHostExtensions` imports to `#[cfg(test)]`, since those names are only used by the Send+Sync boundary tests in the app target.
- No production ownership, manifest status, gateway behavior or extension lifecycle changed. The new exact HEAD must rerun the same shipping Host/Coordinator/Runner and Desktop Chat gates.


### 2026-09-26 Secrets external gateway finalization

- Implementation baseline: `4ec8c7f4e83ce45f233c5bad1d1ff1743fe2b9b2`.
- Traced the frozen Grok Host API instead of inventing a Rust RPC name: `source/host/host-gateway-api.ts` exposes exactly `setBoxSecrets` and `getBoxSecretsStatus` from the Secrets extension.
- The Rust Secrets owner now owns decoding/encoding for those two methods. `setBoxSecrets` preserves validation as a bad-request failure; `getBoxSecretsStatus` returns the frozen `keys`, `isApplied`, and `lastAppliedAtMs` shape. Unknown methods remain unclaimed for normal Host dispatch.
- Shipping `UnifiedGatewayApi` holds the same `Arc<HostSecretsExtension>` started against ForeverBox, so the externally reachable methods and startup/persisted-apply lifecycle share one production owner rather than a parallel adapter.
- `source/host/tests/secrets_extension_contract.rs` now pins the exact frozen method names, status projection, invalid-value rejection, and unknown-method fallthrough in addition to the existing validation/persistence/retry/reload contracts.
- Only `source/host/extensions/secrets/extension.ts` advances to final `implemented`. Manifest becomes 1,821 implemented / 103 existing-needs-parity / 78 planned (181 non-final). The PR remains draft; exact-HEAD Rust/Coordinator/Runner, Desktop Chat Parity, and strict architecture results remain authoritative, and the known legacy-root cutover is still required before the strict gate can pass.


### 2026-09-26 Local Exec provider transport production cutover

- Starting exact HEAD: `afb2f21862be6d5d560c07fba623373536df48f0`.
- The shipping Rust Gateway already implemented the authenticated frozen endpoints `/local-exec/requests` and `/local-exec/responses`, but production passed `local_exec: None`, so a desktop local-exec provider could never actually attach to the Rust Host.
- Added `source/host/src/extensions/local_exec/local_exec_bridge.rs` as the Host-owned provider transport: registration/welcome, hello metadata, heartbeat liveness, supervised + variant provider ranking, multiple-computer projection, request/response correlation, cancellation and approval-retirement fanout.
- Added `source/host/src/extensions/local_exec/extension.rs` with the frozen `LocalToolPermission + Telemetry` dependency declaration and a GatewayBridgeHub adapter. Shipping `source/host/app/src/main.rs` now starts this owner and supplies `Some(local_exec_extension.gateway_bridge())` to `GatewayServerDeps`.
- Added `source/host/tests/local_exec_bridge_contract.rs` covering rank/variant rules, provider registration, hello/liveness, default computer identity, request/response correlation, cancel and retire-approval transport.
- This deliberately does not claim full Local Exec parity. `gateway-local-exec-sand-box.ts` and `production.ts` remain planned, while `extension.ts` and `local-exec-bridge.ts` advance only to `existing-needs-parity`. Remaining work is generated ExecClient codecs/RemoteResourceAccessor, permission-authorized exec/upload/download, complete refusal/failure telemetry and binding the live LocalToolPermission controller.
- Manifest after this slice: **1,821 implemented / 105 existing-needs-parity / 76 planned** (181 non-final). Non-final count is unchanged because this slice converts two planned mappings into real production-backed partial mappings rather than falsely finalizing them.


### 2026-09-26 Local Exec SandBox/file-transfer owner

- Starting exact HEAD: `6f2dcc533223eacfa5c139babf511e2d4282f23a`; the preceding Local Exec slice had already passed the shipping Mahayana Host compile on its exact-head Rust run before this commit was created.
- Added `source/host/src/extensions/local_exec/gateway_local_exec_sand_box.rs`. The Rust Host now owns the reference-shaped active/selected-computer adapter, provider run state, terminals-folder fallback, 100 MiB single-file fence, permission-gate seam, approval-id propagation, base64 upload/download frames, file/file-error settlement and per-computer box resolution.
- Added `source/host/tests/gateway_local_exec_sand_box_contract.rs` with real provider channels: it proves upload/download frame shape and bytes, gate actions, approval propagation, user-computer selection, terminals-folder projection, oversized-file rejection and blocked-gate fail-closed behavior.
- This mapping advances only to `existing-needs-parity`. Generated `ExecClientMessage`/`ExecClientControlMessage` decoding, `RemoteResourceAccessor`, live LocalToolPermission UI/controller binding, and the Runner ExternalShell/Read/Copy tool cutover still block final parity.
- Manifest becomes **1,821 implemented / 106 existing-needs-parity / 75 planned** (181 non-final).


### 2026-09-26 Local Tool Permission owner and Local Exec liveness binding

- Starting exact HEAD: `abda6c7ce019d959d01bcf315b5ef1afad4b7ab6`.
- Added `source/host/src/extensions/local_tool_permission/local_tool_permission_controller.rs`: canonical SettingsService-backed Always/Ask/Never policy, scoped one-time approvals, pending request coalescing/waiting, TTL expiry, allow-once/deny/always/never resolution, settled-id idempotence, approval lifetime, begin-turn invalidation, target-size fencing and resource-path approval reuse.
- The controller implements the already-ported `LocalToolPermissionAskStore`, so the frozen stale/idempotent resolution boundary is no longer isolated from the real permission owner.
- Added `source/host/src/extensions/local_tool_permission/extension.rs` with the frozen Settings/Telemetry/Transcript dependency declaration. Shipping Host starts this owner from the same SettingsService used by Session, and binds its live-computer predicate to the production Local Exec bridge.
- Contract coverage proves standing Never/Always/Ask behavior, exact dependency identity, a real blocked waiter resolved by allow-once, approval reuse and scope retirement.
- Both mappings advance only from `planned` to `existing-needs-parity`. Final still requires the full refusal-direction/saturation memory, preparatory-action rules, transcript ask-card + boot-sweep lifecycle, stranded-retirement telemetry, approval-retired Host event propagation and shipping `resolveLocalToolPermission` Gateway command.
- Manifest becomes **1,821 implemented / 108 existing-needs-parity / 73 planned** (181 non-final).


### 2026-09-26 Local Exec production codec ownership

- Starting exact HEAD: `573cf4995af12f3b90422f45852fbcc005a6806c`, whose Rust Host, independent Coordinator, Runner contracts and Desktop Chat Parity are green; only the strict architecture gate remains red.
- Added `source/host/src/extensions/local_exec/production.rs` as the Rust owner corresponding to frozen `source/host/extensions/local-exec/production.ts`.
- The owner preserves the production boundary that the reference module is responsible for: tolerant object-shaped Exec client JSON, control projection for `throw` and `streamClose`, unknown/heartbeat non-terminal handling, and a package-owned remote-resource accessor wrapper.
- Added `source/host/tests/local_exec_production_codec_contract.rs` covering unknown-field tolerance, throw/stack projection, stream close, unknown control handling, oneof conflict rejection and accessor ownership.
- This row advances only from `planned` to `existing-needs-parity`. The repository currently ships TypeScript-generated `source/packages/proto/generated/agent/v1/exec_pb.ts` but no canonical Rust-generated `ExecClientMessage`/`ExecClientControlMessage`; therefore exact generated `fromJson(..., ignoreUnknownFields: true)` semantics and live GatewayLocalExecManager consumption remain explicit blockers.
- Manifest becomes **1,821 implemented / 109 existing-needs-parity / 72 planned** (181 non-final).


### 2026-09-26 Local Tool Permission controller policy hardening

- Starting exact HEAD: `1cc12beef74968b6f554585d0cb24c1cc576276c`.
- Ported the remaining safety-critical frozen controller policy that was still absent from the first Rust owner: refusal-direction memory, hashed refusal keys, bounded per-agent saturation, forgotten-agent fencing, standing-grant direction epochs, preparatory-action rejection, approval retirement callbacks and permission-change settlement.
- Added contract coverage proving a denied action cannot be retried in the same direction, forgotten tasks stay fenced, preparatory access cannot bypass the action the user is actually being asked to approve, and one-time approval retirement is observable when scope ends.
- The controller intentionally remains `existing-needs-parity`: AbortSignal-equivalent cancellation for individual joined waiters and the exact multi-listener subscription API are still controller-level gaps. Transcript ask cards, boot sweep and Gateway resolution remain extension-level gaps.


### 2026-09-26 Local Tool Permission controller finalization

- Starting exact HEAD: `49291e0009dc3d47bd8027601e337bd9538a5471`.
- Completed the controller-owned frozen behavior rather than conflating it with extension wiring: joined-waiter cancellation now mirrors AbortSignal semantics (one waiter can cancel without cancelling another; the last cancelled waiter retires the pending ask), created/settled events support multiple disposable subscribers, production request ids use the frozen 64-hex shape, and target-size checks use JavaScript-equivalent UTF-16 units.
- Corrected failure ordering to match the frozen controller: existing approval coverage wins before target-size fencing; ask-surface/live-computer checks precede preparatory/size failures; standing-grant direction fencing remains before ask creation.
- Added contracts for joined cancellation, subscription lifecycle, UTF-16 sizing and the prior refusal/forgotten/preparatory/retirement behavior.
- `source/host/extensions/local-tool-permission/local-tool-permission-controller.ts` advances to final `implemented`. The separate extension mapping deliberately remains non-final for transcript ask-card creation, boot sweep, stranded-retirement telemetry, approval-retired Host event fanout and Gateway `resolveLocalToolPermission` production wiring.
- Manifest becomes **1,822 implemented / 108 existing-needs-parity / 72 planned** (180 non-final).


### 2026-09-26 Transcript WidgetResponses durable LocalToolPermission slice

- Starting exact HEAD: `fa0540492f71bbca39fd43f99f7fc572675c7794`.
- Added `source/host/src/extensions/transcript/widget_responses.rs` backed by the shipping `ProductionSessionWorkers`/AgentDb owners.
- The Rust slice now performs frozen LocalToolPermission stale-card retirement, distinguishes a newly retired card from an already-settled durable card, fences by agent/entry/request ids, and performs the fail-soft boot sweep across all durable agents with `ifPendingBeforeMs`.
- Added SQLite-backed contract coverage proving durable `pending -> expired` mutation, stale-resolution idempotence and cutoff-aware multi-agent startup cleanup.
- `widget-responses.ts` advances only to `existing-needs-parity`; generic widgets, AutoReview, secret submission, reactions, spend guard and active-session/roster projection remain explicit responsibilities.
- Manifest becomes **1,822 implemented / 109 existing-needs-parity / 71 planned** (180 non-final).


### 2026-09-26 LocalToolPermission extension + HostRunnerComposition surface cutover

- Starting exact HEAD: `34632c697ab2950d0ea2202fcb34b8a567cbba8e`; Desktop Chat Parity is green and the Rust run has shipping Host, Coordinator, box-daemon and platform-worker checks green, with only the strict architecture gate expected red while the Runner suite completes.
- Added `source/host/src/host_runner_composition.rs` to restore the frozen ownership boundary used by `host-runner-composition.ts`: direct runner sessions own a LocalToolPermission controller subscription, created/settled events project into durable transcript cards, group-member turns do not expose approval surfaces, and run settlement removes the surface.
- Shipping Host now binds `bindAskSurfaces` to that composition and `bindLiveComputerCheck` to Local Exec. Agent deletion forgets permission state; approval retirement publishes `local-tool-permission.approval-retired`.
- `HostLocalToolPermissionExtension` now owns the remaining frozen extension responsibilities: durable transcript binding, background-ready stale-card sweep, `resolveAsk` through the already-final resolution module, stranded-retirement telemetry and `notePermissionChanged`.
- `resolveLocalToolPermission` is now a first-class Rust `UnifiedGatewayApi` method rather than falling through to the compatibility Host lane; `setHostSettings` still delegates its broad settings payload but now invokes the canonical permission-change settlement hook when `localToolPermission` changes.
- `source/host/extensions/local-tool-permission/extension.ts` advances to **implemented**. `source/host/host-runner-composition.ts` advances only to **existing-needs-parity** because its non-permission runner/mirror/memory responsibilities remain open.
- Manifest becomes **1,823 implemented / 109 existing-needs-parity / 70 planned** (179 non-final).


### 2026-09-27 inference extension composition slice

- Exact implementation baseline before this slice: `88526432473a0245d37577412486874dd2329299`. At that SHA Desktop Chat Parity passed; the Rust Host, independent Mahayana Coordinator, Runner, box daemon, Computer takeover and ConversationActor/CapabilityBroker jobs all passed. The Rust workflow was red only at the strict final architecture gate because non-final manifest rows remained.
- Added Rust-owned `source/host/src/extensions/inference/cursor_web_tools.rs`, `extension.rs`, and `production.rs`, exported from the inference module, plus `source/host/tests/inference_extension_composition_contract.rs`.
- The slice preserves frozen Grok request/model projection, web-fetch success/error/no-result normalization, inference readiness semantics, model-experiment listener fanout, shared auth identity, request-id forwarding and the production factory boundary without creating a second inference runtime.
- The corresponding frozen rows `cursor-web-tools.ts`, `extension.ts`, and `production.ts` advance only from `planned` to `existing-needs-parity`. Final status remains blocked on authenticated Cursor backend/generated AiService bindings, `inference-service`/`cursor-session` production wiring, and Host production-extension registry composition.
- Manifest after this slice: **1,825 implemented / 112 existing-needs-parity / 65 planned** (**177 non-final**). This is not a final parity, merge, packaged-acceptance, or release claim.


### 2026-09-27 inference session and labeling slice

- Added Rust-owned `cursor_session`, `inference_service`, and `sand_labeling` semantic cores plus `source/host/tests/inference_session_labeling_contract.rs`.
- `cursor_session` now preserves the frozen Grok model precedence (experiment/default/env/subagent/computer/browser), the exact Grok 4.5 and computer-use defaults, and strict mock-script parsing without routing Cursor through the local non-Cursor provider runtime.
- `inference_service` preserves the Cursor-vs-routed-provider ownership split and provider usage projection. `sand_labeling` preserves previous-request lineage, per-request dedupe, summarization exclusion and post-turn admission rules.
- The three mappings advance only from `planned` to `existing-needs-parity`. Remaining blockers are live authenticated Cursor prompt transport, privacy/media integration, generated inference/labeling protobuf bindings, core-message conversion, diagnostics and production Host extension-registry wiring.
- Manifest after this slice: **1,825 implemented / 115 existing-needs-parity / 62 planned** (**177 non-final**). The strict final architecture gate must remain red until those and the other non-final rows are actually closed.

### 2026-09-27 csnaps process parity closure

- Exact starting HEAD for this slice: `a5e825321736b318275f1b1d51a592ead1838056`.
- The frozen Grok `source/host/extensions/codebase-telemetry/csnaps-process.ts` responsibility is now implemented as the independent Host leaf `source/host/src/extensions/codebase_telemetry/csnaps_process.rs` rather than being folded into the protocol or service layers.
- The Rust process owner preserves the reference boundaries: environment-cleared child spawn, ping/initialize handshake, framed stdin/stdout transport, monotonic request IDs, strict response-ID correlation, per-operation deadlines, terminal failure propagation, apply/snapshot/upload/flush methods, and bounded shutdown/termination.
- `source/host/tests/csnaps_process_contract.rs` adds executable fake-process coverage for the normal operation sequence, unknown request IDs, request timeout termination, and exit-description semantics. The architecture manifest advances only this module from `planned` to `implemented`; adapter/service/extension/privacy/process integration remain independently gated.
- Local Cargo execution on the attached VPS is blocked by a full root filesystem before dependency unpacking; therefore GitHub Actions exact-HEAD Rust runtime is the authoritative compile/test gate for this slice.


### 2026-09-27 csnaps capability audit closure and process-test repair

- source/host/extensions/codebase-telemetry/csnaps-capability.ts was re-audited against the frozen Grok file at a9f633e09d49a85829b8236331b9e21f7e612634. The existing Rust leaf and executable contract already preserve the trimmed SAND_CSNAPS_BIN override, host-bundle default path, and missing / not-file / not-executable availability states, so this row advances from existing-needs-parity to implemented without inventing new behavior.
- Exact-HEAD Rust run 36298649184 proved the shipping Host compiles and the independent Coordinator/box-exec gates pass, then exposed one test-only blocker in the new csnaps-process contract: the test referenced undeclared tempfile. The test now uses the crate's existing uuid dependency plus std::env::temp_dir, avoiding a new dependency while retaining isolated executable fixtures.
- This audit/repair does not promote the remaining codebase-telemetry adapter, host, service, privacy-mode, or extension rows; those remain independently gated.


### 2026-09-27 Codebase Telemetry layered Host closure

- Exact tested HEAD c592e76af851678ca0eb24a7fe412dcb02d27b9b proves the new Codebase Telemetry layers compile inside the shipping Mahayana Host while the independent Mahayana Coordinator contract remains a separate passing gate.
- The full Host/Runner Cargo test gate passes with the new contracts for privacy-mode, codebase-telemetry-adapter, codebase-telemetry-host, codebase-telemetry-service, and the deterministic csnaps response-ID validator.
- The implementation preserves the frozen Grok separation instead of collapsing telemetry into one Rust subsystem: protocol -> process -> privacy-mode -> adapter -> host -> service. Auth/Experiments stay dependencies of the host input layer; adapter restart remains owned by the service.
- The architecture manifest advances only the four evidence-backed rows above. source/host/extensions/codebase-telemetry/extension.ts remains planned until its capability gate and production composition are wired and tested.


### 2026-09-27 Codebase Telemetry production extension closure

- Exact tested HEAD bfad04b4870967099f4c2e18e785f2ffb61e74df closes frozen source/host/extensions/codebase-telemetry/extension.ts as a production Host extension.
- Production composition now owns the same shared SandHostEventBus used by the Gateway, so transcript run-start/run-end telemetry does not depend on a parallel event source.
- The extension preserves the frozen Auth + Experiments dependency boundary, csnaps capability/no-op fallback, codebase UUID and snapshot paths, upload headers/checksum/privacy metadata, and independent privacy -> adapter -> host -> service lifecycle.
- Rust desktop runtime run 36300024477 passed shipping Host compilation, the independent Mahayana Coordinator contract, box-exec contracts, and the full Host/Runner Cargo test gate on this exact HEAD.


### 2026-09-27 Memory synthesis semantic-core slice

- Exact starting HEAD: `92513d4534727f0d90c0ec1d57b28a1864e711a4`.
- Re-read frozen `source/host/extensions/memory/memory-synthesis-service.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634` and completed the Rust service contract around the synthesis-safe FileMemoryStore introduced by the preceding commit.
- `memory_synthesis_service.rs` now owns the frozen bounded pending-agent/evidence queues, strict JSON/change validation, evidence/clock fencing, explicit-memory protection, proposal verification, bounded retry, stale requeue, temporal-review admission and target settlement.
- Local exact-tree contracts passed: `memory_synthesis_service_contract` **4/4** and `memory_synthesis_store_contract` **3/3**. The sparse working copy first lacked generated `agent/v1` protobuf sources; the missing tracked directory was added to sparse checkout before the successful compile.
- This row advances only from `planned` to `existing-needs-parity`. Final parity is still blocked on the real inference PromptExecutor synthesis/verification stages, background debounce + hourly polling, enforceable 90-second deadline/cancellation, telemetry reporting, and the shipping post-turn `recordTurn` hook.
- Manifest becomes **1,833 implemented / 116 existing-needs-parity / 53 planned** (**169 non-final**). The strict final architecture gate must remain red.


### 2026-09-27 Host bundle upgrade core slice

- Exact starting HEAD: `9afd559285466ae3b02e42e366607fadde803c71`.
- Re-read frozen `host-bundle-upgrade.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `host_bundle_upgrade.rs` with supervisor command construction, atomic bundle/command staging, update availability, sanitized swap-veto acknowledgements, bounded failed-swap restaging, crash-loop fencing, one-shot failure telemetry state, and frozen initial/next watch jitter functions.
- `host_bundle_upgrade_contract` passed **4/4** on the exact working tree.
- The manifest row advances from `planned` to `existing-needs-parity`, not final, because `host-upgrade-service` and the extension still need production peer/wake composition.
- Manifest becomes **1,833 implemented / 117 existing-needs-parity / 52 planned** (**169 non-final**).


### 2026-09-27 Host upgrade service + extension slice

- Exact starting HEAD: `f7bbb07c3a5d245bbaa67b719d5044f6f731e734`.
- Re-read frozen `host-upgrade-service.ts` and `host-upgrade/extension.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `host_upgrade_service.rs`: manual/idle update settlement, in-flight fencing, local/latest version state, Automations→CrossUserSharing→Transcript prepare ordering, interrupted-turn resume, marker forwarding, failed-swap restage, and idle-watch staged-version de-duplication.
- Added Rust `host_upgrade/extension.rs`: frozen extension identity/dependencies, auto-update env policy, watch interval/jitter parsing, idempotent marker store, and explicit service/config owner.
- Exact-tree contracts passed together: `host_bundle_upgrade_contract` **4/4**, `host_upgrade_service_contract` **4/4**, `host_upgrade_extension_contract` **3/3**.
- Both rows advance from `planned` to `existing-needs-parity`; final remains blocked on production registry peer adapters plus real marker/update polling wake composition.
- Manifest becomes **1,833 implemented / 119 existing-needs-parity / 50 planned** (**169 non-final**).


### 2026-09-27 Pressure CPU profiler state-machine slice

- Exact starting HEAD: `d7e4b3a907964bbce519f2ee5f3e2d3ce5b511ed`.
- Re-read frozen `telemetry/pressure-cpu-profiler.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `pressure_cpu_profiler.rs` with sustained-pressure admission, minimum capture interval, capture deadline, dynamic knob overrides, bounded retention, profile write/callback, and idempotent disposal behind a `CpuProfilerBackend` boundary.
- `pressure_cpu_profiler_contract` passed **3/3** on the exact working tree.
- The row advances from `planned` to `existing-needs-parity`; final remains blocked on a real production cross-platform sampling backend plus production pressure/tick lifecycle wiring.
- Manifest becomes **1,833 implemented / 120 existing-needs-parity / 49 planned** (**169 non-final**).


### 2026-09-27 Host tracing backend-boundary slice

- Exact starting HEAD: `549621b6bd26f8525ebc49ee50ef9cc4e1427786`.
- Re-read frozen `telemetry/host-tracing.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `host_tracing.rs` with token-refresh exporter replacement, previous-delegate continuity when a token temporarily disappears, fail-closed export, Grok-compatible trace URL/base headers/resource attributes, provider registration/flush/shutdown, idempotent disposal and singleton initialization.
- The OpenTelemetry SDK itself stays behind `SpanExporter` / `HostTracerProvider` boundaries instead of forcing a Node-specific SDK into Rust.
- `host_tracing_contract` passed **4/4** on the exact working tree.
- The row advances from `planned` to `existing-needs-parity`; final remains blocked on a real production OTLP exporter/provider backend and production Host signal/lifecycle wiring.
- Manifest becomes **1,833 implemented / 121 existing-needs-parity / 48 planned** (**169 non-final**).


### 2026-09-27 MCP legacy live-reference retirement slice

- Exact starting HEAD: `b19cb30cffa2be22d045180e2c4c391ab540bdd3`.
- Re-read frozen `mcp/legacy-live-references.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `legacy_live_references.rs` using the real `PluginSkillsCache` and `GlobalWorkflowLibrary`: GitHub blob URL normalization, skill-tail matching, materialized source selection and selective pointer-workflow retirement.
- `legacy_live_references_contract` passed **3/3**.
- The row advances from `planned` to `existing-needs-parity`; final remains blocked on marketplace/auth sweep invocation from production MCP/plugin-skills composition.
- Manifest becomes **1,833 implemented / 122 existing-needs-parity / 47 planned** (**169 non-final**).


### 2026-09-27 MCP plugin-skills service-core slice

- Exact starting HEAD: `0d5ad9797ec7762f2899ba45930e141c060c645d`.
- Re-read frozen `mcp/plugin-skills.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `plugin_skills.rs`: plugin/skill projection, stable duplicate-safe IDs, publisher/team facts, auth-block classification/de-duplication, listed/indexed cache pruning, prior-listed record preservation, serialized sync passes, real `PluginSkillsCache` writes and sync telemetry.
- `plugin_skills_contract` passed **4/4**.
- The row advances from `planned` to `existing-needs-parity`; final remains blocked on the authenticated Dashboard/marketplace loader plus startup/auth-change/daily production scheduling and legacy-reference sweep invocation.
- Manifest becomes **1,833 implemented / 123 existing-needs-parity / 46 planned** (**169 non-final**).


### 2026-09-27 MCP Box execution boundary slice

- Exact starting HEAD: `58e3e94b87b5dc147b8b7891ea4de8898a861376`.
- Re-read frozen `mcp/box-mcp-exec.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `box_mcp_exec.rs` over the existing MCP state-executor shape: load forwarding, filtered/all-server discovery, server/tool/clientKey projection, and tool execution errors settled as terminal error results.
- `box_mcp_exec_contract` passed **3/3**.
- The row advances from `planned` to `existing-needs-parity`; final remains blocked on the production ForeverBox/CapableBox resource accessor plus generated `McpArgs/McpResult` execution/diagnostic wiring.
- Manifest becomes **1,833 implemented / 124 existing-needs-parity / 45 planned** (**169 non-final**).


### 2026-09-27 Teach recording extension boundary slice

- Exact starting HEAD: `e0a64fbb3ca6ae685f515387f93077a584f9a58a`.
- Re-read frozen `source/host/extensions/teach-recording/extension.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634` (reference blob `37a530d2bc60a8f090e69787b475bca03f0c917b`).
- Added Rust `teach_recording/extension.rs` without collapsing it into the recording service: frozen dependency IDs, the versioned 32-byte queue-signature key contract, secure random generation, `0600` persistence on Unix, cap-delay composition, service-factory ports, best-effort pending recovery, API projection and Host-lifecycle disposal.
- Added `teach_recording_extension_contract` for key parsing/persistence/permissions, non-regeneration of valid keys, dependency/cap wiring, pending recovery and idempotent Host-owned service disposal.
- The row advances from `planned` to `existing-needs-parity`; final remains blocked on the separately-owned Rust `teach-recording-service` recording/queue state machine and production Host composition.
- After the teach-recording slice the manifest was **1,833 implemented / 125 existing-needs-parity / 44 planned** (**169 non-final**).


### 2026-09-27 Content search extension boundary slice

- Exact starting HEAD: `6e1608b72c7043226d8b176efd9f52fc6cc197f7`.
- Re-read frozen `source/host/extensions/content-search/extension.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634` (reference blob `bf4b665ead15283a1c6166151b50ce6c178546e2`).
- Added Rust `content_search/extension.rs` as the Grok-shaped composition owner only: `sand_global_search` gate application, mutation subscription attach/detach, duplicate-subscription suppression, index start, API forwarding/limits and deterministic cleanup.
- Added `content_search_extension_contract` covering disabled startup, enable/disable/re-enable, mutation forwarding, search API forwarding, constants and disposal.
- Index DB/service/worker/writer remain independent mapped modules and are not hidden behind the extension port.
- The row advances from `planned` to `existing-needs-parity`; final remains blocked on those real Rust index owners plus production Host composition.
- After the content-search slice the manifest was **1,833 implemented / 126 existing-needs-parity / 43 planned** (**169 non-final**).


### 2026-09-27 MCP extension lifecycle slice

- Rebased onto concurrent exact HEAD `d7fd26eca27e6e0e5cb4b5de9cae545283596d2e`, preserving both Teach Recording and Content Search extension slices.
- Re-read frozen `mcp/extension.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `mcp/extension.rs` with the frozen dependency identity, auth-gated startup, renewal + first-credential handling, startup/refresh polling semantics, startup-success legacy-sweep hook and idempotent shutdown ordering.
- A test-fixture self-deadlock was found during validation (listener invoked while holding the same auth-state mutex); the fixture was corrected without weakening runtime locking.
- `mcp_extension_contract` passed **3/3** after that repair.
- The row advances from `planned` to `existing-needs-parity`; final remains blocked on production Auth/Experiments/ForeverBox/Settings/Telemetry adapters plus the real `McpHostService`.
- Combined manifest after preserving all concurrent slices: **1,833 implemented / 127 existing-needs-parity / 42 planned** (**169 non-final**).

### 2026-09-27 MCP service semantic-owner slice

- Exact starting HEAD: a2862f927f7febaddc845c91a41361ad4e36386a.
- Re-read frozen source/host/extensions/mcp/mcp-service.ts at Grok baseline a9f633e09d49a85829b8236331b9e21f7e612634.
- Added Rust mcp_service.rs as the service owner above box_mcp_exec and plugin_skills: installed server/plugin projections, effective-install attribution, forced catalog refresh fallback, install/uninstall Skill + legacy-reference coordination, kick-only Box discovery and terminal status follow-up fanout, auth/server subscriptions, external-auth forwarding and idempotent disposal.
- A first implementation used a raw pointer only for unsubscribe lifetime. It was rejected before commit and replaced with Arc<Mutex<ServiceState>>; the committed implementation contains no unsafe subscription path.
- mcp_service_contract passed **4/4** after that repair.
- The row advances from planned to existing-needs-parity; final remains blocked on mcp/production.ts binding the authenticated Dashboard/plugin-manager, ForeverBox/CapableBox execution, Settings/Experiments and Telemetry adapters.
- Manifest becomes **1,833 implemented / 128 existing-needs-parity / 41 planned** (**169 non-final**).


### 2026-09-27 Agent content-search semantic leaf

- Exact starting HEAD: `ac82d4a876a88eb4aea9724d199c9a92af78bee5`.
- Re-read frozen `source/host/extensions/content-search/agent-content-search.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `content_search/agent_content_search.rs` as a pure Host semantic leaf, preserving the frozen 5-per-agent / 50-total limits, searchable entry projection, whitespace-flattened case-insensitive matching, 30/60 snippet window, hidden peer-message suppression, newest-first ordering, role defaults and timestamp defaults.
- Added `agent_content_search_contract` covering entry projection, snippet normalization/windowing, reverse ordering, hidden-peer filtering, role/timestamp projection and empty/zero-limit behavior.
- This row is promoted directly from `planned` to `implemented` because it is a side-effect-free helper with no independent lifecycle or production binding to leave unresolved; the search-index DB/service/worker/writer remain separate mapped rows.
- Manifest after this leaf: **1,834 implemented / 128 existing-needs-parity / 40 planned** (**168 non-final**).
- Exact-HEAD validation is required through `cargo test --manifest-path source/host/Cargo.toml` and the Rust desktop runtime workflow before treating the commit as CI-green.


### 2026-09-27 Content-search SQLite / FTS5 owner

- Exact starting HEAD: `b43622b397a85d3b6db2a0242b9a52a7b0c1d61a`.
- Re-read frozen `source/host/extensions/content-search/search-index-db.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `content_search/search_index_db.rs` using `rusqlite`: the frozen 5-second busy timeout, WAL/NORMAL/incremental-vacuum pragmas, STRICT message/media/meta/agent tables, external-content FTS5 tables and insert/update/delete triggers, schema version and reconcile markers.
- Ported NFKC query normalization, maximum eight quoted prefix terms, message search with per-Agent five-result cap and zero-timestamp fallback, FTS snippet whitespace normalization, recent/all media search, filtered media search, nullable metadata and unknown attachment-kind fallback.
- Added `search_index_db_contract` covering schema/pragmas/version/reconcile state, NFKC/quoting/cap behavior, message ranking/per-Agent cap/timestamp fallback, and media recent/FTS/kind semantics.
- This DB module advances directly from `planned` to `implemented`; service/worker/writer remain independently non-final and must consume this owner rather than creating a parallel index path.
- Manifest after this owner: **1,835 implemented / 128 existing-needs-parity / 39 planned** (**167 non-final**).


### 2026-09-27 Content-search writer / reconciliation owner

- Exact starting HEAD: `0b82fd0a46091d49b668243c5dbfdc3381c6cb14`.
- Re-read frozen `source/host/extensions/content-search/search-index-writer.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `search_index_writer.rs` over the single Rust SQLite owner. It ports message/notice/send-message text projection, non-Agent peer-message exclusion, 20k body cap, JS-style numeric rounding, attachment filename fallback/URL decoding, image/video/audio MIME projection, attachment classification, `store.db` fingerprint caching, transactional upsert/delete/clear, transcript `reindexAgent`, stale-agent reconciliation and incremental vacuum.
- `SearchIndexJob` is now a typed serde-tagged contract shared by the upcoming worker/service layers instead of inventing a second job protocol.
- Added `search_index_writer_contract` covering message/media projection, body/timestamp/dimension normalization, URL filename decode, MIME/kind classification, store fingerprint reindex, stale-agent reconciliation, delete/clear and job dispatch.
- This row advances from `planned` to `implemented`; worker/service remain separately mapped and must call this canonical writer.
- Manifest after this owner: **1,836 implemented / 128 existing-needs-parity / 38 planned** (**166 non-final**).


### 2026-09-27 Content-search worker fault boundary

- Exact starting HEAD: `5979c4b532dd2bbdaa6def2bc27d3e75e0484532`.
- Re-read frozen `source/host/extensions/content-search/search-index-worker.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `search_index_worker.rs` as a **separate named OS-thread fault boundary**, not a method on SearchIndexService. Startup owns a distinct DB connection + canonical Writer, ensures schema, correlates typed request IDs, returns structured success/failure, classifies SQLite corrupt/not-a-database errors, and closes Writer/DB on stop.
- Added `search_index_worker_contract` covering request correlation + persisted execution, idempotent termination/fail-closed post behavior, and corruption-code classification.
- This row advances from `planned` to `implemented`; SearchIndexService remains separately mapped and will supervise this worker rather than absorb it.
- Manifest after this boundary: **1,837 implemented / 128 existing-needs-parity / 37 planned** (**165 non-final**).


### 2026-09-27 Content-search service supervisor

- Exact starting HEAD: `f9ffe6bd62aed541e8048dd42d935296a843ce66`.
- Re-read frozen `source/host/extensions/content-search/search-index-service.ts` at Grok baseline `a9f633e09d49a85829b8236331b9e21f7e612634`.
- Added Rust `search_index_service.rs` as the supervisor **above** the independent Worker/Writer boundary. It owns reader-DB open/migration, fresh-vs-nonfresh schema stamping/recreation, readiness, mutation-to-job routing, serial dispatch, bounded corruption rebuilds, bounded worker respawns, bounded failed-job reconcile/retry, health events and the 2-second dispose drain.
- Worker writes remain behind `SearchIndexJobPort`; Service does not absorb Writer or use its reader connection for writes.
- Added `search_index_service_contract` covering fresh start/reconcile/mutation/search/dispose, worker-unavailable respawn without index rebuild, corruption-triggered index rebuild + reconcile, and nonfresh schema mismatch recreation. Frozen retry/rebuild limits are asserted.
- This row advances from `planned` to `implemented`. The content-search implementation set is now complete at module level; the separately mapped extension remains non-final until its production mutation/gate/telemetry adapters are bound to this concrete service in the shipping Host.
- Manifest after this supervisor: **1,838 implemented / 128 existing-needs-parity / 36 planned** (**164 non-final**).


### 2026-09-27 Content-search production extension composition

- Exact starting HEAD: `9d80e0e9ea60a6dc09fc1e13a40b7720c5ea68a5`.
- Shipping Host now constructs `ProductionContentSearchExtension` after the real Transcript extension. The owner uses the frozen Sand root + agents-root paths, the real `HostExperimentsExtension` gate, the global `transcript_mutation_events` bus, the canonical Rust SearchIndex Service/Worker/Writer/DB chain, and a dedicated `sand.search_index_health` structured telemetry sink.
- Gate transitions now attach/detach the actual global mutation subscription; drop removes the experiment listener, mutation listener and drains/disposes the index service.
- Added a production contract that toggles the real Experiments override, publishes a real global transcript mutation, waits for the independent index worker, queries the concrete FTS reader, then verifies the disabled gate is truly unsubscribed.
- The extension manifest row deliberately remains **existing-needs-parity**: no production caller on this branch currently publishes the five frozen mutation forms (`entries-upserted`, `entry-deleted`, `conversation-cleared`, `agent-removed`, `agent-needs-reindex`) into `transcript_mutation_events`. Marking the extension final before producer wiring would be a false green.
- Manifest totals therefore remain **1838 implemented / 128 existing-needs-parity / 36 planned** (**164 non-final**) until the producer side is wired and exact-HEAD tests prove it.


### 2026-09-27 Content-search producer audit correction and finalization

- Re-audited the frozen producer sites instead of relying on branch code-search indexing. The current Rust branch already preserves all five mutation producers at the frozen ownership points:
  - `SandAgentDb`: committed append/batch/update -> `entries-upserted`; committed delete -> `entry-deleted`; committed conversation clear -> `conversation-cleared`.
  - `SandAgentSessionStore::delete_session`: durable directory deletion -> `agent-removed`.
  - `ProductionAgentLifecycle`: clone/mint commit -> `agent-needs-reindex`.
- Therefore the previous note saying the producer side was absent was conservative but incorrect. With production Experiments, mutation subscription, Service/Worker/Writer/DB, telemetry and Host lifetime now connected, `content-search/extension.ts` is promoted from `existing-needs-parity` to **implemented**.
- Fixed the first exact-HEAD compile blocker from run `36303412610`: explicitly typed the production mutation subscription as `Option<RawTranscriptMutationSubscription>` (Rust E0282).
- Manifest after finalizing this extension: **1,839 implemented / 127 existing-needs-parity / 36 planned** (**163 non-final**).

### 2026-09-27 cross-user sharing Rust ownership slice

- Continued only on implementation PR #20 / `refactor/grok-018-architecture-rebuild`; no parallel runtime or replacement branch was introduced. Starting implementation HEAD for this slice was `4c38d91d963f79db8b84f17814223782402676d5`.
- Added Rust Host owners for the seven previously `planned` frozen Cross-user Sharing modules: extension gating, durable departure obligations, entry publication shaping, authenticated relay transport/error semantics, remote-turn admission budgeting, sharing service state, and state reconciliation. Existing durable turn-dedupe/tombstone/pending-departure/environment/wire-normalization owners remain the single persistence/normalization path.
- Added `xuser_service_contract.rs` to cover frozen user-facing relay error semantics, fail-closed remote-agent/entry shaping, room add/revoke reconciliation, dev/production environment gating, feature-gate activation, and backend-state reconciliation.
- These seven manifest rows advance only from `planned` to `existing-needs-parity`. They are intentionally **not** final: production Host composition still needs to bind live Auth, Attachments, NotifyBus and Transcript delegates and exact-HEAD CI must prove the shipping extension wiring. No status is advanced merely because a target file now exists.
- Coordinator/Host/Runner boundaries are unchanged; this work belongs only to the Mahayana Host extension layer.

### 2026-09-27 multitask owner slice

- Added the Grok `sand-multitask.ts` Rust owner at `source/host/src/sand_multitask.rs`, reusing the existing Runner `SandSubagentType` model rather than creating a second subagent type system.
- Preserved the frozen env-override truthiness, the single builtin `executor` subagent contract, todo-queue guidance and multitask prompt boundary; added `sand_multitask_contract.rs`.
- Manifest advances this row from `planned` to `existing-needs-parity` only. Final requires the production Runner composition to inject the executor config/todo description/prompt section into live turns and exact-HEAD CI evidence.

### 2026-09-27 exact-HEAD Host-suite repair after cross-user slice

- Rust runtime run `36303809965` for exact HEAD `fb1ce257ba93a82a8cfba07cc92df8129b78c977` passed shipping Host compilation, independent Coordinator, box-exec daemon, and Host box-exec supervision, then failed inside the full Host test suite on the pre-existing content-search production contract.
- Failure was `production_mutation_projection_is_typed_and_fails_closed`: frozen TS semantics allow an upserted `IndexEntry` with absent `id` to deserialize as an empty id, while the Rust struct required the field and caused projection to fail closed too early.
- Fixed narrowly by giving `IndexEntry.id` a serde default. No Cross-user Sharing behavior or Coordinator/Host/Runner boundary was changed by this repair.


### 2026-09-27 Host box-store-sync chrome-session watcher slice

- Implemented the frozen `source/host/extensions/box-store-sync/chrome-session-watcher.ts` behavior as the Rust Host owner at `source/host/src/extensions/box_store_sync/chrome_session_watcher.rs`.
- Preserved session-db prefix filtering, null-filename mtime fallback, debounce coalescing, callback-failure isolation, idempotent start/stop, and a real non-recursive filesystem watcher using the existing Host `notify` dependency.
- Added `source/host/tests/chrome_session_watcher_contract.rs` covering frozen filename semantics, mtime fallback, debounce/stop disposal, and real filesystem delivery.
- Manifest moves this row from `planned` to `existing-needs-parity`, not `implemented`: final parity still requires the planned box-store-sync production extension to compose the watcher into the live sync trigger path. This deliberately avoids claiming parity from a class/module name alone.

### 2026-09-27 MCP production composition evidence slice

- Audited frozen `source/host/extensions/mcp/production.ts` at blob `f6e50b2474adcebb33dbb52291c7be5d5256f0a9` against the existing Rust Mahayana Host owner.
- Confirmed `source/host/src/extensions/mcp/production.rs` already owns the recovered production Dashboard publish RPC paths/client, Host auth-renewal adapter, real plugin-skills polling lifecycle, and production skill-publish construction; added `source/host/tests/mcp_production_contract.rs` to pin the frozen Dashboard RPC paths plus immediate polling/stop/idempotent-dispose behavior.
- The architecture row advances only from `planned` to `existing-needs-parity`. It is deliberately not marked `implemented`: the frozen production extras still require shipping composition for the installed-plugin loader, MCP service factory, legacy auth cleanup, legacy live-reference sweep, telemetry callbacks, and Host extension-registry wiring.
- Local full Rust compilation was not used as proof because the repository policy keeps heavy builds in GitHub Actions. The attempted narrow Cargo test began compiling the full Host dependency graph and was stopped; exact-HEAD Rust desktop runtime CI remains authoritative after push.

### 2026-09-27 live Auto-review Runner binding

- Exact starting implementation HEAD: `f318d04833cf9a7250407b2cbe1824ab037aa4a7`; shipping wiring commit: `7da51e2dc1fbf38a9518652e16ad8804c59e3658`.
- Shipping `source/host/app/src/main.rs` now binds one `AutoReviewService` controller for each routed Runner turn and retires it on both normal terminal settlement and OS-thread spawn failure. Background `group-member` / `automation` turns bind non-resolvable approval surfaces rather than exposing impossible renderer approval cards.
- CloudAgent launch/reply/lifecycle actions now consume the same live controller plus the production Auth-backed `ClassifySandAutoReview` executor before tool execution; `review: None` is no longer the production path.
- Added `source/host/tests/auto_review_production_wiring_contract.rs` to pin the shipping composition and cleanup path. Exact-HEAD `cargo check` for the shipping Mahayana Host, independent Mahayana Coordinator contract, box-exec-daemon contracts, and the full `cargo test --manifest-path source/host/Cargo.toml` Host/Runner suite all passed on GitHub Actions run `36308944148` through the Host/Runner test step.
- `source/host/extensions/auto-review/sand-auto-review-awaiting.ts` is promoted to **implemented**: its frozen ordering/duplicate/fallback/clear behavior was already covered by focused contracts, and its last Host-start + live-Runner-controller blockers are now closed.
- Other Auto-review rows remain non-final on purpose. The remaining work is real: browser/computer/shell/MCP/subagent/automation preflights still need complete production binding, and backend classifier cancellation still lacks Grok-equivalent `ctx.signal` abort semantics. CloudAgent remains non-final for watcher/quiet-origin ownership and generated trace projection.


### 2026-09-27 Box Store Sync planned-owner closure

- Exact starting implementation HEAD: `4ce50719a55dd0d20f2acbc640d3727c704e38b3`.
- Added Rust Mahayana Host owners for the 12 remaining planned Box Store Sync reference modules: agent-store files, copy-in, object-store, download safety/budget, manifest/revision, pack pipeline, sync service policy, sync evaluation, transfer accounting/glob/root dedupe, multipart planning, DB bundle capture trace, and DB capture outcome classification.
- Added `source/host/tests/box_store_sync_planned_contract.rs` covering traversal/reserved-path rejection, conditional-write status, read batching/etag state, copy-in defaults/exit semantics, scoped local object-store IO, restore/symlink escape fencing, byte budget, manifest conflict/revision semantics, pack/path selection, glob/root dedupe, multipart hashing/partitioning, flush evaluation, and DB-capture failure priority.
- The 12 manifest rows move only from `planned` to `existing-needs-parity`; no row is claimed `implemented` until the shipping Box Store Sync service composes the new owners with backend/presign transport, downloader/transfer, manifest persistence, pack lifecycle and DB capture end-to-end.


### 2026-09-27 remaining planned-owner closure

- Exact starting HEAD for this slice: `1ab3fe54e3c8779958c411b465dcdd517e2e84fd`.
- Added Rust owners for the seven remaining planned mappings: Telemetry box log shipping policy, Transcript background wakes, client-side tool v2 projection boundary, roster projection/coalescing, shared-room safety helpers, Host Gateway nonce/capability policy, and SandHost readiness/health policy.
- Added `source/host/tests/remaining_planned_contract.rs` for log-shipping enable/error/source rules, wake dedupe/revival fencing, tool projection identity/phase behavior, outline coalescing, shared-room avatar/content/member gates, gateway template/purpose/nonce caps, and SandHost approval-only busy/readiness behavior.
- Architecture manifest now has zero `planned` rows. These seven mappings intentionally remain `existing-needs-parity` until production composition and the generated/external portions of their frozen responsibilities are wired and exact-HEAD CI proves them.


### 2026-09-27 shipping SandHost/Gateway policy wiring

- Slice starting HEAD: `3630fceda4a7cabf0232cc1bbdb5468cc97b55b3`; prerequisite runtime exposure commits: `c2fda99d2c4cc671f1bd2f2836f99cbaf611877b` and `fdabeba77f5bf78d3dc92afd41bc31b767a989e7`.
- Shipping `UnifiedGatewayApi` now consumes the Rust `host_gateway_api` create-Agent policy: bounded client-nonce dedupe, approved Sand purpose filtering, and template-id validation around the canonical Rust ProductionSession gateway mint.
- The production `/health` path no longer falls through to `GatewayApi::default()`. It now derives live running Agents, active Agent, durable pending wakes and Auto-review approval waits through the canonical Rust `sand_host` owner and preserves the frozen approval-only `lastBusyAtMs` behavior.
- Added `source/host/tests/sand_host_production_wiring_contract.rs` so source-level production composition and deterministic policy are checked together.
- The confirmed box-ready lifecycle is now shipping: after gateway discovery the Host reads `SAND_BOX_BOOT_ID` / `SAND_BOX_BOOT_STARTED_AT_MS`, suppresses duplicate boot ids with `/tmp/sand-box-ready-stage`, reports `ready` asynchronously, retries up to 3 times at 30 seconds, and writes the marker only after confirmed telemetry delivery. `sand-host.ts` remains non-final only because frozen background-shell / running-subagent / mid-drain revival busy sources are not all yet projected into health; no manifest status is promoted merely because an owner exists.


### 2026-09-27 live Runner capability prompt projection

- The routed-provider system prompt now consumes an immutable per-turn capability snapshot instead of hard-coding the CloudAgent-disabled variant.
- When the production `CloudAgentToolBridge` is present, the provider receives the enabled frozen Grok base prompt; when absent, the existing disabled prompt and warning remain fail-closed.
- The Host now evaluates the frozen `sand_multitask` feature gate for ordinary turns, disables multitask for group-member turns, carries that decision through `ProductionRunnerCompositionInput` / `TurnAgentComposition`, and appends `SAND_MULTITASK_PROMPT_SECTION` to the actual provider system prompt.
- `sand-multitask.ts` intentionally remains `existing-needs-parity`: the current Rust Runner still lacks the frozen per-turn executor subagent-config replacement and multitask-specific TodoWrite description injection. This slice does not mark the row final until those live tool surfaces exist.


### 2026-09-27 event-loop telemetry production-wiring checkpoint

- Continued implementation on PR #20 (`refactor/grok-018-architecture-rebuild`); PR #19 remains the merged spec-only change.
- Frozen Grok `source/host/extensions/telemetry/event-loop-telemetry.ts` was re-audited against the Rust Host owner. The prior Rust owner only preserved trigger/projection logic and therefore correctly remained non-final.
- Added a production Rust sampler in `source/host/src/extensions/telemetry/event_loop_telemetry.rs`: scheduler-overrun samples at the frozen 20ms cadence, p50/p95/max aggregation, normalized process CPU occupancy, frozen 60s window / 50ms pressure threshold / five-window heartbeat policy, dedicated low-frequency worker ownership, and deterministic disposal.
- Wired the sampler through `source/host/src/extensions/telemetry/extension.rs` so production Host telemetry owns start/stop and all reports use the existing structured-log sink. This is a platform-fit adaptation for the Rust Host rather than introducing a Node event-loop runtime.
- Extended `source/host/tests/event_loop_telemetry_contract.rs` with a live short-window sampler contract in addition to the existing frozen trigger/projection assertions.
- Implementation commits: `3e98e35b4d21ee7b1f16580f1041809ab68501c2`, `3b2830e8b39eb37e6cd66fcea7f3c4597a97a68e`, `10807c4a010a65dd43ff3348c3bdb8101ea8dd52`.
- Exact-HEAD verification was started automatically as Rust desktop runtime run `36314356518` and Desktop Chat Parity run `36314356528`. At this checkpoint the Rust Host job is still executing, so the manifest row intentionally remains `existing-needs-parity`; no incomplete CI is represented as a pass.
- Architecture inventory before this slice remains 2,002 frozen source-bearing modules with 155 non-final rows, all under the independent Mahayana Host/Runner domains. Coordinator module-level parity is already independently evidenced; global strict cutover, legacy-root removal, packaged acceptance, merge, and release remain separate blockers.

### 2026-09-27 Event-loop telemetry exact-HEAD finalization

- Exact verified implementation HEAD: `77965135a9d41a50981354f7a9a7a807ad0699ba`.
- The Rust Mahayana Host event-loop telemetry owner is production-wired through the shipping Telemetry extension with the frozen 20 ms cadence, 60 s aggregation window, 50 ms p95 pressure threshold, five-window heartbeat policy, scheduler-overrun p50/p95/max, normalized process CPU occupancy, structured telemetry emission, and deterministic disposal.
- Exact-HEAD Rust desktop runtime run `36314429799` passed shipping Host build, independent Mahayana Coordinator, box-exec, independent Mahayana Runner, prompt attachment, Computer takeover, and ConversationActor/CapabilityBroker ownership steps; its renderer job failed only at the intentionally strict final architecture gate because other manifest rows remain non-final.
- Exact-HEAD Desktop Chat Parity run `36314429842` passed both renderer build and Focused Electron chat E2E.
- Therefore `source/host/extensions/telemetry/event-loop-telemetry.ts` advances from `existing-needs-parity` to `implemented`; no unrelated row is bulk-promoted.
- Architecture inventory becomes **1,848 implemented / 154 existing-needs-parity / 0 planned** (**154 non-final**).

### 2026-09-27 fail-closed Host copy-in bootstrap and exact-HEAD audit

- Re-read merged spec PR #19 and continued implementation only on Draft PR #20 (`refactor/grok-018-architecture-rebuild`). The frozen reference remains `b-nnett/grok-bot-0.18-reconstructed@a9f633e09d49a85829b8236331b9e21f7e612634`; no language-wide Rust requirement was reintroduced.
- Audited frozen `source/host/main.ts` against the shipping Rust Host. The one missing bootstrap responsibility was Grok's early `--box-copy-in` process mode. `source/host/app/src/main.rs` now dispatches that mode before process crash guards, the single-Host lock, box-exec-daemon, Host construction or Gateway startup, then exits with the copy-in result instead of falling through into the long-lived Host.
- `source/host/src/extensions/box_store_sync/box_copy_in.rs` now owns a real local-object-store restore path: canonical manifest parsing, partially-hydrated-manifest rejection, restore-path and symlink fencing, blob restore, size/SHA-256 verification, Unix mode restoration, store.db completeness accounting and fail-closed partial/corrupt outcomes. Remote AgentStore/V2 transport remains deliberately fail-closed until its real production adapter exists; this slice does not pretend local restore is the complete frozen Box Store Sync service.
- Added contracts in `box_store_sync_planned_contract.rs` for successful local bootstrap, disabled/no-op behavior, remote-adapter fail-closed behavior and corrupt-blob rejection. Added `shipping_host_copy_in_mode_precedes_long_lived_runtime_bootstrap` to `sand_host_production_wiring_contract.rs` so a future refactor cannot move copy-in behind normal Host startup.
- The first exact-HEAD Rust compile exposed a Rust-2024 raw-identifier issue for the `box` module; fixed it narrowly (`crate::r#box`). Exact HEAD `1b066e7d34c14611dd7d7d7bea28687bdc8221c7` then passed shipping Host compilation, the independent Mahayana Coordinator contract, independent box-exec-daemon and Host supervisor contracts, the complete Host/Runner Cargo test suite, prompt attachment projection, human Computer takeover and ConversationActor/CapabilityBroker ownership in Rust desktop runtime run `36315487000`.
- Exact HEAD `20d399b05d857dcde8540eef3f6beb4237b8446a` subsequently passed Grok Rust parity finalize run `36315894265`, including the new Host bootstrap-ordering regression.
- On that evidence, only the independently-complete manifest rows were finalized: `box-store-manifest-format.ts`, `xuser-sharing-environment.ts`, and `source/host/main.ts`. Strict non-final inventory moved from 154 to 151; every remaining row is still `existing-needs-parity` and remains a real blocker.
- Legacy parallel-runtime roots required by the architecture gate are absent: `desktop/src`, `desktop/electron`, `frontend/apps/web`, and `third_party/mahayana`.
- Follow-up audit corrected several overly-broad historical blocker notes. Cross-user Sharing is not merely missing Host injection: the Rust relay client owns most backend RPCs, but `SandXuserSharingService` still lacks much of the frozen public service/event/manager delegate lifecycle before Auth/Attachments/NotifyBus/Transcript production composition can be final. MCP likewise cannot be finalized by registry wiring alone because no concrete production `PluginSkillsLoader` or `McpManagerBackend` is present yet. Inference has a production Cursor web backend but still lacks concrete `InferenceAuth` / port / backend factories and shipping extension composition. Box Store Sync still lacks the real production sync service that consumes request coalescing, workspace ignore, vacuum, upload/download and remote object-store owners. Runner's largest shared blocker remains cutover from the routed-provider facade to the generated-Agent stream/action/checkpoint/tool/resource graph while preserving the independent Runner boundary.


### 2026-09-27 Settings production closure

- Continued only on Draft PR #20 (`refactor/grok-018-architecture-rebuild`) against frozen Grok Bot 0.18 reference `a9f633e09d49a85829b8236331b9e21f7e612634`; the architectural rule remains Grok ownership/boundaries first, language chosen per module, with no return to a blanket Rust mandate.
- Frozen `source/host/extensions/settings/settings-service.ts` and `source/shared/node/settings/sand-settings-store.ts` were re-read before implementation. Rust `SettingsService` now owns the complete frozen Host-facing settings surface: canonical atomic `settings.json` persistence, timezone validation/override subscriptions, disabled-notification disk normalization, MCP legacy/server-id instructions and disabled-tool maps, account-scoped cleanup/migration, agent/computer-use models, auto-review normalization, local-tool-permission ceiling, WebAuthn, pinned/sidebar/onboarding state, inference provider/usage and feature-override listeners.
- Account scope ordering now matches Grok: an account switch is applied before MCP values from the same `setHostSettings` update, account-sensitive state is cleared only on a real account transition, onboarding ownership is preserved/cleared according to its account owner, and invalid local-tool permission input normalizes to `ask`.
- Production Host composition now starts `Settings` before `Experiments`, includes `HostExtensionId::Settings` in the shipping graph, stores the same `Arc<SettingsService>`, and retains the Settings→Experiments feature-override subscription for the full Host lifetime rather than dropping a temporary subscription.
- Added/expanded focused contracts for extension identity, production registry inclusion, MCP clamp/migration/delete and disabled tools, account-scope fencing, permission ceiling, feature-override delivery, inference usage accounting, same-update scope ordering, onboarding ownership, disabled-notification persistence and default auto-review override removal.
- Exact implementation HEAD `e05561f88d13c7bb66320aaf811fb57e3726f1cc` passed shipping Mahayana Host compilation, independent Mahayana Coordinator contract, independent box-exec-daemon/Host supervisor contracts, and the complete `source/host/Cargo.toml` test suite in Rust desktop runtime run `36317435522`. The exact-head renderer job passed production architecture boundary, Electron boundary contracts, recovered chrome contracts, Agent workspace boundary, Grok inventory, renderer typecheck/build and built-bundle boundary; it failed only at the intentionally strict final architecture gate because other rows remain non-final.
- `Grok Rust parity finalize` run `36317431399` also passed Mahayana Runner parity contracts and architecture inventory. It deliberately left Settings non-final because the curated finalizer does not bulk-promote this row.
- Manifest commit `944624bbaa588bf82b6c42f1d9f7ff360f200f6f` therefore advances only `source/host/extensions/settings/settings-service.ts` from `existing-needs-parity` to `implemented`, with production wiring and exact test evidence attached. Architecture inventory becomes **1,852 implemented / 150 existing-needs-parity / 0 planned** (**150 non-final**).
- Remaining blockers are not naming/ledger work. The shared roots are still generated-Agent Runner cutover; Transcript; real Box Store Sync production service/object-store transport; Automations; Telemetry; Cross-user Sharing; Inference; MCP production manager/skills-loader; Local Exec generated codec/Runner consumption; Memory synthesis; Auto-review preflight coverage; Host Upgrade production peers; and their dependent State Backstop/Teach Recording/Experiments/Cloud Agents rows.

### 2026-09-27 routed-provider retry telemetry production wiring

- Exact implementation parent after rebasing concurrent PR #20 work: `b465aef6c02386c0302484e038b0a6cc0eb8dcee`.
- Re-audited frozen `source/host/extensions/telemetry/turn-telemetry-mappers.ts` and `source/host/runner/stream-attempt.ts` against the shipping Rust Runner/Host path. The mapper already had focused contracts, but none of its turn-level projection functions were used by production, so the row correctly remained non-final.
- The Runner `ProviderRetryReport` now preserves bounded typed retry diagnostics at the failure-classification owner (`error_type`, stable `error_code`, and `cause`) for retried, exhausted, and gave-up-ineligible outcomes instead of forcing the Host to reconstruct classification from a display string.
- Shipping `runner.startRoutedProvider` now sends each retry report through the canonical Rust `turn_retry_telemetry` mapper and the existing production structured-log sink while retaining the separate Runner activity/observation path. This keeps retry policy/classification in Runner and telemetry transport in Host.
- `source/host/tests/production_turn_run_shell_adapter_contract.rs` now pins the typed transport classification used by the production report boundary. The architecture manifest records this production wiring but deliberately leaves `turn-telemetry-mappers.ts` as `existing-needs-parity`: interrupt, await, user-message, closing-send, TTFT, turn-usage, and computer-use production call sites still must be routed through the same owner before finalization.
- Local architecture validation passes (`node scripts/check-grok-architecture.mjs`, non-strict) and `git diff --check` passes. The connected Linux development environment does not expose Cargo/Rustfmt, so exact-HEAD GitHub Actions remains the authoritative Rust compile/test evidence after push; no CI result is pre-claimed.

### 2026-09-27 Runner await telemetry production wiring

- Continued from exact parent fa569b65651aa647abf3f66a90c9827ed3d36400 after its shipping Host compile, independent Coordinator and independent Runner gates passed.
- Frozen Grok RunnerRegistry.wireRunnerLifecycle binds setTurnAwaitHandler directly to telemetry.reportTurnAwait; the Rust Runner observation owner now exposes the same dedicated handler rather than requiring Host code to infer an await from the generic renderer observation bus.
- AwaitShell / awaitToolCall completion and unwind settlement send awaitIndex, blockUntilMs and outcome to the dedicated handler while preserving the existing runner-turn-observation event stream. Shipping Host composition binds that handler to the canonical Rust turn_await_telemetry mapper and the production structured-log sink.
- Extended turn_observation_contract.rs to prove one terminal await observation reaches the dedicated handler with the frozen fields. The telemetry mapper row remains non-final because interrupt, user-message, closing-send, TTFT, turn-usage and computer-use production call sites are still outstanding; TTFT is specifically not faked because the current Rust first-token path still lacks the frozen trace/span and cross-clock timing inputs.

### 2026-09-27 user-message and watchdog interrupt telemetry wiring

- The frozen send pipeline reports user-message receipt only for a real dispatched non-group send and captures wasInFlight before the new run begins. Shipping Rust now captures ProductionTranscriptRuntime.is_agent_running before execute_send_with_queue_observers and emits user_message_received_telemetry only from the real SendBegin::Dispatch persistence callback, so duplicate/no-op sends do not create false telemetry and group sessions remain excluded.
- The frozen RunnerRegistry watchdog interrupt records reason=watchdog, hadActiveRun and the lifecycle wasInFlight state before interruption. The shipping run-queue watchdog Trip path now reads the canonical Transcript lifecycle before cancellation and emits turn_interrupt_telemetry with the actual cancellation result after interrupt_wedged_run_for_watchdog.
- Ack-obligation semantics remain unchanged: skipAckObligation still gates only ack obligation creation, not user-message telemetry. The turn-telemetry mapper row remains non-final because superseded/agent-lifecycle interrupt sources, closing-send, TTFT, turn-usage and computer-use usage still require production wiring.

### 2026-09-27 turn-usage lifecycle telemetry wiring

- ProductionTranscriptRuntime now tracks an accepted provider operationId in both the last-request owner and the per-turn request-id set; previously the latter was never populated even though RunLifecycleState already owned the frozen request-id-count and turn-ended-seq semantics.
- ProductionTranscriptRuntime exposes the existing lifecycle settle_turn_usage owner without duplicating state. Shipping routed-provider terminal settlement emits the canonical turn_usage_telemetry projection with source, deterministic first request id, request-id count and turn-ended sequence before publishing TurnEnded.
- Provider token usage remains intentionally None because the current routed-provider streaming result contract still returns text without frozen usage metadata. The mapper row therefore remains non-final; this change closes lifecycle/request-id production wiring without fabricating input/output/cache/reasoning counts.

### 2026-09-27 TTFT first-token telemetry production wiring

- Re-audited frozen `source/host/extensions/transcript/runner-registry.ts` and `source/host/runner/sand-agent-runner.ts`: Grok binds a dedicated first-token handler at Runner lifecycle composition and forwards that observation to `telemetry.reportTtft`. The previous Rust path emitted first-token only on the generic runner observation bus, so the canonical TTFT mapper had no shipping producer.
- `TurnObservation` now owns a dedicated first-token sink alongside the existing await sink, preserves one-shot first-token semantics, and forwards the exact emitted observation to that sink without replacing the generic observation event stream.
- Shipping `runner.startRoutedProvider` binds that sink to the canonical Rust `ttft_telemetry` mapper and the existing structured-log telemetry transport. This keeps observation in Runner and telemetry projection/transport at the Host boundary, matching the frozen responsibility split rather than moving timing policy into Electron or Renderer.
- Focused contracts now prove that the dedicated first-token sink fires exactly once with chunk/model/TTFT/skew fields and that shipping Host composition registers the handler and invokes the canonical mapper. Implementation commits: `b1b00a28d6c5c5bfe2a312f56f94c0b4587733f3`, `81d4e33b54f1550bc309748f4858528cd5a3c298`, `aaa525069c37130d2fbfd965e9bb965d2ea6f2bb`, `3e69f6f045fd451bcaf4753eee2d0dee6f5272b3`; manifest evidence commit `274838ba4c76d24ab2b3b254045416079d1a7bea`.
- The telemetry mapper row deliberately remains `existing-needs-parity`. The superseded interrupt source, closing-send nudge telemetry, computer-use usage, provider token usage, and complete trace/span timing provenance remain to be wired before that row can become final. Agent-deletion interrupt wiring is completed in the following section. No token counts or tracing identifiers are fabricated.

### 2026-09-27 agent-deletion interrupt telemetry production wiring

- Re-audited frozen `source/host/extensions/transcript/agent-lifecycle.ts`: agent deletion captures the lifecycle in-flight state before interruption, interrupts the active Runner/group Runner, and reports `sand.turn.interrupt` with `reason=agent_deleted` and the actual interruption result.
- Shipping Rust `AgentDeletionRuntimeDeps.cancel_runner` now follows the same ordering. It reads `ProductionTranscriptRuntime.is_agent_running` before cancellation, derives `had_active_run` from the real `TranscriptRunnerRegistry.cancel_agent` result, and sends the event through the canonical `turn_interrupt_telemetry` mapper and production structured-log sink.
- The production wiring contract pins all three semantics: pre-cancel `was_in_flight`, actual cancellation-derived `had_active_run`, and `agent_deleted` reason. Implementation/test commits: `a75319af772f15b76a89bf7f5e535100dac744b0` and `112cf316da6b997b14f5df08d9a43b59b7dd3749`; manifest evidence commit `f00bc0b1a73dae919e7959ae3ca8ce671d857366`.
- The turn-telemetry mapper row remains non-final. The remaining interrupt gap is the frozen direct-send `superseded` source; the current Rust direct-send composition explicitly preempts group-member routed streams while 1:1 cancellation still crosses the compatibility Host lane, so reporting `superseded` before consolidating that ownership would create false telemetry. Closing-send, computer-use usage, provider token usage and complete trace/span timing provenance also remain open.

### 2026-09-27 direct-send supersede interrupt telemetry production wiring

- The direct-send ownership audit found the 1:1 supersede boundary in Mahayana Coordinator rather than the compatibility Host lane. `ActiveInferenceStreamRegistry` registers the turn before Host admission, returns `CancelNow` for an already accepted active stream, and records deferred supersede before Runner acceptance; both paths call `runner.cancelRoutedProvider` with an explicit `Superseded...` reason.
- The independent Runner registry now exposes a read-only stream-to-agent ownership projection through the Transcript registry. This does not move cancellation policy out of Coordinator and does not create a second registry; it lets the Host cancel boundary correlate the Coordinator-owned stream with the canonical Transcript lifecycle.
- Shipping Host `runner.cancelRoutedProvider` now resolves the agent before cancellation, captures `was_in_flight` from `ProductionTranscriptRuntime`, executes the real `cancel_stream`, and for explicit supersede reasons reports canonical `turn_interrupt_telemetry` with `reason=superseded` and `had_active_run` equal to the actual cancellation result. Non-supersede cancellations are not mislabeled.
- Focused Runner registry contracts pin stream ownership and retirement; the production turn-telemetry contract pins the Host supersede call site. Implementation/test commits: `d002627c7e08176cc488c3afd3c81d0c4ec2ff19`, `7b194f50f522a397ec12fdd270cda862484f9be6`, `f184d647fa233fa960897a7149d769e2e55f39a2`, `86c25858b2ca9ac97c30b5ac8b2f4cca5895c77b`, `e5af734deff4bebede734f091da4f88fd3d7f5ae`; manifest evidence commit `92856cd03d19c115a58e62c3380e70be06c76117`.
- All frozen turn-interrupt producers audited in this slice now have real production wiring: watchdog, agent deletion and direct-send supersede. The mapper row remains non-final for closing-send nudges, computer-use usage, real provider token usage, and the remaining frozen TTFT dispatch-clock/trace/span provenance.

### 2026-09-27 SQLite snapshot worker production closure

- Removed the second ad-hoc SQLite VACUUM thread from `store_db_snapshot_upload.rs`. The implemented `StoreDbSnapshotUpload` production state machine now submits real snapshot capture to the independent `BoxStoreVacuumJob` / `spawn_box_store_vacuum_job` owner with the canonical store DB busy timeout and propagates its structured terminal diagnostic.
- The behavioral snapshot-upload contract now verifies both a successful native VACUUM and propagation of a real worker failure. This complements the isolated vacuum-worker contract in `sqlite_snapshot_contract.rs` and proves the independent worker is consumed by the production snapshot/upload owner rather than existing only as a named module.
- With that real consumer in place, the manifest rows for frozen `box-store-vacuum-worker.ts` and `sqlite-snapshot.ts` move from `existing-needs-parity` to `implemented`. The Box Store Sync domain therefore drops from 17 to 15 non-final rows and the repository-wide strict remainder drops from 150 to 148. This does not imply the full Box Store Sync service is complete; backend/object-store composition, exact async service lifecycle, Chrome-session live triggering, request coalescing and workspace-ignore consumption remain independently gated.
- Implementation/test commits: `3b8ac9fca8d64d027599d00eba14831b8f525c03`, `7619d19822647bdb23fca3efd02345223c61b13c`; manifest closure commit `df8d23293a18c49130d623a9abefc2f9ae12e051`.
- The prior exact-HEAD Host compile failure in the agent-deletion telemetry composition was also a startup-scope bug rather than an architectural blocker: `dd7fb1b7fdaa16ecefb28ae52c038857d0457088` resolves Transcript and telemetry dependencies from owners already constructed at that point in Host startup, preserving the same deletion telemetry semantics without forward local-variable references.


### 2026-09-27 roster search indexed production closure

- Continued from exact parent `b917981d4308adaff87f29b40811c9cda9244d7f`, where ContentSearch itself was already final but Transcript roster search still reached only the linear durable-transcript fallback in shipping Gateway composition.
- The shipping Host now retains the real `ProductionContentSearchExtension` as an owned dependency of `UnifiedGatewayApi` instead of starting it into an underscore-only lifetime holder. Session Gateway dispatch receives that exact owner through `dispatch_production_session_gateway_call_with_content_search`; no second search runtime or compatibility index is introduced.
- `searchAgents` now uses the ready SQLite/FTS5 index fast path, filters stale/deleted agent ids, preserves newest-first result limiting, and falls back to the canonical durable transcript scan when the index is not ready. `searchMedia` is now reachable through the same shipping Gateway and preserves file name, extension, MIME, attachment kind, timestamp and dimensions while filtering stale agents.
- `source/host/tests/roster_search_contract.rs` adds a focused production-Gateway contract for both indexed message search and media search, including stale-agent filtering. This closes the frozen `source/host/extensions/transcript/roster-search.ts` mapping without changing Coordinator/Host/Runner boundaries.
- The architecture manifest moves only that row from `existing-needs-parity` to `implemented`; the strict remainder becomes 147. Other Host/Runner rows remain non-final until their own production dependencies and exact behavior are wired.


### 2026-09-27 Host Local Exec architecture-gate correction

- Exact-head Desktop Chat Parity run `36326026178` failed only in `Enforce Agent workspace architecture boundary` with `Shipping Host must own Local Exec extension composition`.
- Production Host composition already starts the consolidated Grok-shaped Host extension bundle through `start_production_host_extensions(...)`, owns `production_extensions.local_exec`, derives its live bridge, and exposes `local_exec_extension.gateway_bridge()` only through `GatewayServerDeps.local_exec`.
- The failed boundary check still required the retired literal call `start_local_exec_extension(...)`. Restoring that call would create duplicate composition and violate the single canonical Host ownership model.
- Commit `34d3ef075453ba558ffc1f2ab6e366aa538a3329` updates the architecture gate to verify the current production composition and Gateway bridge instead of the obsolete constructor spelling. No production runtime behavior was weakened or moved across Coordinator/Host/Runner boundaries.
- Focused Electron chat E2E in the same prior exact-head run passed. A new exact-head CI run for the corrected commit is still required before this gate can be marked passed.


### 2026-09-27 Cross-user entry publisher parity checkpoint

- Re-read frozen Grok Bot 0.18 `cross-user-sharing/extension.ts`, `xuser-entry-publisher.ts`, `xuser-remote-turns.ts`, `xuser-departure-obligations.ts` and `xuser-state-reconcile.ts` before changing Rust ownership.
- The previous Rust `xuser_entry_publisher.rs` only exposed pure payload helpers. It now owns the frozen per-room ordered publication chain, user-vs-agent wire projection, remote-agent echo suppression, attachment resolution with image MIME and aggregate byte caps, server timestamp restamping and failure-isolated queued publishing.
- `SandXuserSharingService` now owns an explicit bound entry-publisher slot and exposes a weak `SandXuserManagerDelegate` with `is_enabled` plus ordered `publish_room_entry`, matching the frozen Transcript delegate boundary without moving Transcript responsibilities into the sharing service.
- Focused contracts cover human-message projection, local-agent projection, remote-agent suppression, attachment filtering/base64 inlining, server timestamp restamp, ordered delegate publication and disabled-state fencing.
- No architecture-manifest row is promoted to final in this checkpoint. The strict remainder deliberately stays at **147**: production Host still lacks the live Transcript/Attachments adapter and the current Rust sharing service has not yet integrated the frozen remote-turn, departure-obligation and full room-materialization responsibilities. This checkpoint narrows that blocker without hiding it.


### 2026-09-27 Cross-user remote-turn protocol checkpoint

- Re-read frozen Grok Bot 0.18 `xuser-remote-turns.ts` at reference `a9f633e09d49a85829b8236331b9e21f7e612634` before replacing the shallow Rust helper.
- `source/host/src/extensions/cross_user_sharing/xuser_remote_turns.rs` now owns the frozen inbound admission rules, durable nonce-dedupe port, per-room/member 10-minute / 30-turn budget, shared-room guardrail prompt construction, deleted-agent cleanup, two-message result cap, outbound `sand-remote:<owner>/<agent>` routing, relay-drain wake, 600-second result deadline and 10-minute unreachable-member backoff.
- `source/host/src/extensions/cross_user_sharing/xuser_turn_dedupe_store.rs` now exposes a typed `XuserTurnDedupe` port implemented by both durable and in-memory stores, preserving the CrossUserSharing / persistence boundary.
- Added `source/host/tests/xuser_remote_turns_contract.rs` covering inbound membership + dedupe + settlement, outbound wire identity + nonce settlement, and the windowed budget owner.
- This checkpoint does **not** promote the remote-turn manifest row to final. Production Host composition still must bind the live sharing service to Transcript/Runner, Attachments, departure obligations and room materialization. The strict remainder therefore intentionally remains **147** until that production wiring exists and exact-HEAD CI is green.


### 2026-09-27 SharedRooms production owner and privacy-deadline repair checkpoint

- Re-read frozen Grok Bot 0.18 `source/host/extensions/transcript/shared-rooms.ts` and confirmed that CrossUserSharing must reach room/session materialization through Transcript rather than opening Session persistence directly or creating a parallel sharing runtime.
- `source/host/src/extensions/transcript/shared_rooms.rs` now has a production `SharedRooms` owner backed by the canonical `ProductionSessionWorkers`. It owns shared-room binding lookup, canonical room-agent selection, hosted-room materialization/update, mirror-room materialization/update, mirror revocation, shared-room activity notices, and relay timestamp restamping.
- `TranscriptManager` now constructs and owns exactly one `SharedRooms` instance. This preserves the frozen Transcript ownership boundary and gives the later CrossUser Host adapter one production port instead of direct Session database access.
- `ProductionSessionWorkers` exposes its canonical agents root read-only for Transcript-owned room binding files; no second store root or legacy runtime is introduced.
- Added `source/host/tests/shared_rooms_production_contract.rs` to pin TranscriptManager singleton ownership, hosted-room local-member filtering, remote-member persistence, activity notice persistence, timestamp restamping, mirror-room self-member filtering and revocation.
- While auditing exact-head CI, Rust desktop runtime run `36328301975` exposed an unrelated macOS scheduling race in `codebase_telemetry_privacy_mode_contract::deadline_is_retryable_and_subscribers_observe_committed_modes`: the old timeout began only after the lookup worker was spawned, allowing an already-overdue result to be accepted after the caller was descheduled. Commit `b4afd489569f4fa349b639167e9ebfbc1a323d34` changes the lookup to enforce one wall-clock deadline from before worker spawn through result commit.
- SharedRooms implementation commits in this checkpoint: `8a84c2c11d49af4d91ff9db9fe3b81fbd7258264`, `885e7e2c1b3706259deedf7ed5a3693c9756e0b8`, `a43bbc89864e77bc2e6cd1e59ba294b223a35b4c`, `ea9930b32892a4de9e0f1cf6472aee4b55a83696`.
- No CrossUserSharing or SharedRooms manifest row is promoted to final by this checkpoint. The remaining end-to-end gate is still production composition: construct the CrossUser service/relay from Auth, bind NotifyBus, bind EntryPublisher to Attachments + SharedRooms, bind RemoteTurns to SharedRooms + the real group-member Runner, consume shared-room `DeferredRemote` sends, bind departure obligations, then prove the exact path in CI. Strict architecture status remains non-final until those dependencies are live and exact-head tests pass.


### 2026-09-27 exact-HEAD compile recovery and production-composition audit

- Re-read the current PR #20 head after the SharedRooms checkpoint instead of relying on an older acceptance snapshot.
- The architecture manifest still contains 2,002 frozen source-bearing Grok modules with 1,855 `implemented`, 147 `existing-needs-parity`, and 0 `planned`. No non-final row is promoted by this checkpoint.
- Exact-head Desktop Chat Parity CI run `36329280485` failed in `Focused Electron chat E2E` while building the deterministic Mahayana Host. The Rust compiler reported `E0592` because `ProductionSessionWorkers::agents_root` had been defined twice.
- Commit `4ae698ceb926c0060188c506304ac4c2d71c208e` removes only the duplicate accessor and restores a single canonical Session owner API; it does not add a second store or relax any architecture gate.
- Exact-head verification for `4ae698ceb926c0060188c506304ac4c2d71c208e` is running as Rust desktop runtime run `36329789244` and Desktop Chat Parity CI run `36329789262`. Renderer typecheck/build has already completed successfully in the latter; the focused deterministic Host/Electron job is still running at the time of this checkpoint.
- Frozen Grok CrossUserSharing was re-read at `b-nnett/grok-bot-0.18-reconstructed@a9f633e09d49a85829b8236331b9e21f7e612634`. Its production extension explicitly depends on Attachments, Auth, Experiments, Transcript and NotifyBus; its RemoteTurns owner delegates execution back through Transcript/Runner rather than opening a parallel provider runtime.
- The current Fabushi shipping Host still has no complete CrossUserSharing production composition: the live room gateway surface, relay event dispatch/materialization path, Attachments-backed entry publisher binding, Transcript SharedRooms delegate, real Runner-backed RemoteTurns execution, departure obligations and shared-room `DeferredRemote` fanout must all be wired before any CrossUserSharing/SharedRooms row can become final.
- Coordinator acceptance is no longer a blocker at this exact HEAD. `source/node-agent-coordinator/tests/coordinator_contract.rs` independently covers protocol hello/version negotiation, request-id uniqueness/correlation, cancellation, disconnect settlement, reconnect/resync, Host restart generation settlement, Gateway down/up recovery, MCP routing, protocol-breach settlement and crash recovery. `.github/workflows/rust-desktop-runtime.yml` runs the entire independent Coordinator crate with `cargo test --manifest-path source/node-agent-coordinator/Cargo.toml`; exact-HEAD run `36329927951` passed the `Test independent Mahayana Coordinator contract` step. This evidence is distinct from Electron coordinator tests and Host coordinator-tool relay tests and therefore satisfies the dedicated Coordinator fault-boundary gate without collapsing Coordinator into Host.

### 2026-09-28 production Group Chat owner closure

- Re-audited frozen Grok Bot 0.18 `source/host/groups/group-chat.ts` blob `09c150285a8d7c23695d4a16d4a4ceb5db360d6e` against the Rust owner and the current shipping path rather than promoting the row from file existence alone.
- `source/host/src/groups/group_chat.rs` preserves the frozen caps, round ordering, nesting guard, member-set semantics, mention resolution, pass filtering, redrive note, history formatting, shared-room system prompt and per-member turn prompt contract.
- The former production blocker is now closed: `dispatch_local_group_send` constructs the single `GroupChatOrchestrator`; shipping Host `run_local_group_member_turn` sends those exact system/user prompts into the real Routed Runner; shared-room and `sand-remote:*` members use the CrossUser remote executor through the same orchestrator rather than a parallel group runtime.
- `source/host/tests/send_group_fanout_contract.rs` pins local room-history consumption, frozen SendMessage system prompt projection, durable authored messages, shared-room deferral without a remote executor, and mixed remote execution without a second group runtime. `group_chat_orchestrator_contract.rs` independently pins bounded rounds, pass filtering and epoch cancellation.
- Exact parent `7a1fe73ee5b2453e19bdf85b814ca0df86eda153` passed Desktop Chat Parity run `36333822440` and every substantive Rust Host job in run `36333822435`, including the independent Coordinator and Host Runner suites; that Rust workflow failed only at the intentional strict architecture gate on the remaining non-final rows.
- Therefore only `source/host/groups/group-chat.ts` advances from `existing-needs-parity` to `implemented`. The architecture remainder drops from 139 to 138; no other Host/Runner row is reclassified. Fresh exact-HEAD CI for this manifest/Spec closure is required before using the new commit as an acceptance baseline.

### 2026-09-28 production GroupChatOrchestrator closure

- Re-audited frozen `source/host/extensions/transcript/group-chat-orchestrator.ts` blob `af4a80b60607ee9964ab279d642d58c73b90d54b` against `source/host/src/extensions/transcript/group_chat_orchestrator.rs`.
- The Rust owner preserves the frozen three-round / ten-message bounds, round rotation, mention-based responder selection, epoch cancellation, shared-room 24-message window, group member system/turn prompt construction, pass filtering, two-message-per-member cap and optional finalization callback.
- The prior manifest blocker is no longer true: production `dispatch_local_group_send` now constructs this exact orchestrator for the shipping Host path. Local members execute through the real Routed Runner; shared/remote members use the CrossUser remote executor through the same orchestrator.
- Focused `group_chat_orchestrator_contract.rs` and `send_group_fanout_contract.rs` pin bounds, pass filtering, epoch cancellation, frozen prompt projection and shared/remote execution without a second group runtime.
- Only this orchestrator row is advanced to `implemented`. `group-chat-glue.ts` and `send-group-fanout.ts` remain non-final because they still have separately documented frozen-behavior gaps.

### 2026-09-28 Group Chat adjacent blocker correction

- Re-audited frozen `group-chat-glue.ts` and `send-group-fanout.ts` after the production CrossUser/shared-room wiring landed.
- Their old manifest notes incorrectly still listed remote/shared-room delegation itself as missing. Shipping Host now passes `cross_user.remote_executor()` into the single `dispatch_local_group_send` / `GroupChatOrchestrator` path, and shared-room relay-triggered turns reuse the same path.
- Neither row is promoted. `group-chat-glue.ts` still lacks frozen Cursor member execution, streaming preview cleanup/finalization, reactions, three-attempt DM-preemption redrive, and full trace/activity semantics. `send-group-fanout.ts` still needs the frozen mirror-room send/attachment contract, canonical user-entry publication ordering, Cursor-native execution, and the remaining glue-owned redrive/reaction/preview behavior.
- This correction keeps the strict ledger pointed at real work instead of an already-closed remote-delegate blocker.


### 2026-09-28 Inline shared-room image materialization production closure

- Re-audited frozen `source/host/extensions/transcript/inline-image-materialization.ts` against the current shipping CrossUser/SharedRooms path.
- The previous manifest blocker is closed: `TranscriptManager` owns one production `SharedRooms` service, and both `post_shared_room_guest_message` (hosted-room inbound human messages) and `append_mirror_room_entry` (mirror-room inbound human/agent messages) call the canonical Rust `materialize_inline_images` owner before durable transcript append.
- The focused `shared_rooms_production_contract.rs` now verifies both production ingress paths end-to-end: inline base64 bytes are written under the agent's `xuser-attachments` directory, projected as file URLs, preserve MIME-derived extension and alt text, and those URLs are the ones persisted into the canonical transcript entry.
- Only the frozen `inline-image-materialization.ts` mapping advances from `existing-needs-parity` to `implemented`. No adjacent CrossUser, Group Chat, or Transcript rows are promoted by this change.

### 2026-09-28 Auto-review classifier in-flight cancellation parity

- Starting exact HEAD: `c05b7d12d7af72458bfd1a1ca20ee535a2fa7d2f`.
- The Rust Cursor backend now has a cancellable unary transport for the Auto-review classifier. The live Runner cancellation token races both response-header and response-body reads so cancellation drops the in-flight HTTP future rather than waiting for the 10-second classifier timeout.
- `SandBackendSmartModeClassifierExecutor` maps transport cancellation to `AutoReviewClassifierError::Aborted`, preserving Grok's `ctx.signal` semantics instead of converting cancellation into a classifier rejection.
- Shipping CloudAgent Auto-review now supplies its real per-stream `RoutedProviderCancellation` to that executor. The focused backend contract holds the server socket open after cancellation and requires the call to abort promptly, proving the request is not merely waiting for transport failure.
- The architecture-manifest row intentionally remains `existing-needs-parity`: Browser, Computer, Shell, MCP, Subagent and Automation preflight owners still need production wiring through the same AutoReview controller/executor before this cluster can be finalized.



### 2026-09-28 Automations suspend/resume connect-watch regression correction

- Starting implementation HEAD `50a6e25bb4552151527e37fad435915eb347b8eb` exposed a real exact-HEAD failure in `automations_extension_runtime_contract`: after non-destructive `suspend_wakes()` / `resume_wakes()`, the listener reconnect callback was expected but the Rust watcher had never observed the pre-suspend disconnected state.
- Frozen Grok `ListenerConnectWatcher.watch()` immediately schedules `tick()` after registering a pending watch. The Rust deterministic seam intentionally does not yet own the production polling timer, so the focused contract now performs that eager poll before suspension and verifies the same disconnect-before-reconnect arming semantics.
- The correction landed as `06cc475b52963850a2f208e61375970ebd4d3af2` (`test(automations): mirror eager connect-watch tick before suspend`). This does not promote the Automations extension or watcher rows: the real polling-policy owner, live Transcript `resumeAfterListenerConnect` callback, authenticated listener reads, relay backend, cloud sync and fire-consumer backend protocol remain explicit blockers.
- The independent Mahayana Coordinator contract remained green before and after this correction; the failure was in Host Automations behavior, not a Coordinator architecture boundary.

### 2026-09-28 Frozen multitask prompt semantic restoration

- Re-audited frozen `source/host/sand-multitask.ts` against the shipping Rust `source/host/src/sand_multitask.rs` and found the production-injected `SAND_MULTITASK_PROMPT_SECTION` was an abbreviated paraphrase rather than the frozen Grok 0.18 contract.
- Commit `56f25afa96ad45411ae37b6818bd58f253bef41c` restores the complete frozen prompt semantics used by the real Runner system-prompt assembly, including strict short-turn delivery, Task/executor dispatch threshold, parallel stream ownership, resume-context requirements, executor SendMessage prohibition, TodoWrite reconciliation, invisible-machinery wording and group-room behavior.
- Commit `d6930484fde506a9857a9258ee789e2dfff6abba` extends `sand_multitask_contract.rs` so those clauses cannot silently regress back to an abbreviated prompt.
- The `sand-multitask.ts` manifest row intentionally remains `existing-needs-parity`: the production Runner still needs to bind `create_sand_executor_subagent_config()` into the live subagent registry and replace the live TodoWrite description with `SAND_MULTITASK_TODO_DESCRIPTION`. No status is promoted merely for restoring prompt text.
