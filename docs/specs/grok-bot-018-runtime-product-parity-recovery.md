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
| ARCH-001..017 | blocked | Current Fabushi tree/process model is not yet Grok-equivalent; Mahayana Coordinator, Host/Runner boundary parity, architecture manifest, language-fit decisions, removal, and structural gates remain incomplete. |
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

