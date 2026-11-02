# OCRM Dossier: Model Picker Capabilities, Placement Fences & Session Selection

- **Unit ID**: `OCRM-UNIT-040`
- **Module ID**: `OCRM-M06` (`model-picker`)
- **Responsibility IDs**:
  - `OCRM-RESP-MODEL-PICKER-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `acceptance=pending`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 6 upstream files in `src/model-picker/` (1,328 lines total, 100% of the directory) are audited, mapped, and natively implemented:

1. `src/model-picker/capabilities.ts` (14 lines)
2. `src/model-picker/apply-session-model-selection.ts` (466 lines)
3. `src/model-picker/apply-session-model-selection.test-support.ts` (152 lines)
4. `src/model-picker/apply-session-model-selection.test.ts` (565 lines)
5. `src/model-picker/apply-session-model-selection.operator-context.test.ts` (191 lines)
6. `src/model-picker/apply-session-model-selection-placement.test.ts` (152 lines)

## 2. Invariants & Implementation Details

- **Model Picker Capability Profiles**:
  - `ModelPickerCapabilityProfile` encapsulates presentation constraints (markdown support, max text length 4096), callback limits (1024 units, UTF-8 bytes / UTF-16 units), and response interaction capabilities (`supportsEphemeral`, `supportsEdit`, `supportsReplace`).
- **Session Model Selection & Invariants**:
  - `SessionModelSelectionRequest` handles provider, model, explicit `resetToDefault` intent, `alias`, `profileOverride`, and runtime directives (`unchanged`, `clear`, `set`).
  - **Lock Enforcement**: If a session entry is marked `is_locked`, any selection request is immediately rejected with reason `locked` and message `"Model selection is locked for this session."`.
  - **Concurrency & Generation Fence**: Mismatched initial and current session generations trigger a `conflict` rejection with `"Model change was not applied because the session changed. Retry."`.
  - **Operator Authority**: Disallowed selections for operator role are rejected with reason `not-allowed` and `"Model <provider>/<model> is not available for this operator role and agent."`.
  - **Model Visibility Policy**: Disallowed models outside the agent policy allowlist are rejected with `"Model <provider>/<model> is not available for this agent."`.
  - **Active Cloud Worker Placement Fence**: When cloud-worker placement is active (non-local), selecting a runtime without execution mode compatibility is rejected with reason `invalid-runtime`:
    - If execution mode changes: `"Session cannot change cloud placement execution mode while placement is <state>."`.
    - If execution mode is missing: `"Session cannot select a runtime without cloud placement support while cloud worker placement is <state>."`.
  - **Authoritative State Transition**:
    - Default/reset clears `provider_override` and `model_override`.
    - Explicit override updates `provider_override` and `model_override`.
    - Profile overrides set `auth_profile_override`.
    - Any model or runtime change flags `live_model_switch_pending = true` and updates `updated_at`.
  - **Event Formatting**: Switch events formatted as `"Model switched to <alias> (<provider>/<model>)."` or `"Model switched to <provider>/<model>."`.

## 3. Rust Native Owner & Verification Evidence

- **Rust Implementation**:
  - `source/host/src/extensions/inference/model_picker_service.rs`
  - Registered in `source/host/src/extensions/inference/mod.rs`
- **Contract Tests**:
  - `source/host/tests/openclaw_model_picker_contract.rs`
    - `test_ocrm_model_picker_o01_capability_profile_and_event_formatting`
    - `test_ocrm_model_picker_o02_locked_and_conflict_rejections`
    - `test_ocrm_model_picker_o03_operator_and_visibility_policy`
    - `test_ocrm_model_picker_o04_placement_fences_and_successful_mutation`
- **Historical contract record**: the dossier records four contract tests as passed in an earlier run. That record is not current exact-head Actions evidence and does not establish production wiring or acceptance during the active development freeze.

## 4. Current production reachability audit (2026-10-31)

The current Host source exposes `SandAgentSessionStore::apply_session_model_selection`
and persists an applied result in the agent-scoped SQLite `sessionModelSelection`
key. A non-test caller search found no production session mutation route invoking
that method. Separately, `runner.startRoutedProvider` reads the persisted
selection and projects it into `SandSessionOptions`, so the read-side Runner /
Inference consumer edge is present. The missing authenticated mutation producer
keeps `BL-MODEL-PICKER-EXEC-WIRING-001` at `wiring=not-wired`,
`verification=not-run`, and `acceptance=pending`.

## 2026-11-01 current-writer exact-head refresh

The preceding source identity is superseded for this writer by commit
`295947e2342847c51c7a1848f4a80c138765fe52`, tree
`62a7ad5c0eb58e3310c356d5270b3b0cab5c4117`. A fresh repository search on that
exact subject finds the persistence facade at
`source/host/src/extensions/session/agent_session.rs:310-329`, the policy
owner at `source/host/src/extensions/inference/model_picker_service.rs:288`,
and contract-only calls in `source/host/tests/openclaw_model_picker_contract.rs`
and `source/host/tests/agent_session_contract.rs`; it finds no production
caller in `source/host/app` or `source/host/src`.

The consumer remains directly evidenced by
`source/host/app/src/main.rs:7675-7742`, which reads the persisted selection,
and `:8637-8653`, which projects its model override and thinking level into
the routed provider session options. The authenticated `sessions.patch`
surface still handles communication-policy and archive fields only
(`source/host/app/src/main.rs:4225-4365`), while the model-picker presentation
validator remains validation/fallback rendering without an action executor
(`source/host/src/extensions/interactive/payload_service.rs:271-277`).

This refresh changes provenance only. It does not create a producer, accept
catalog or operator policy from request JSON, add a thin model-selection route,
or promote status: `BL-MODEL-PICKER-EXEC-WIRING-001` and
`BL-SESSION-MODEL-MUTATION-WIRING-001` remain
`wiring=not-wired`, `verification=not-run`, and `acceptance=pending`.

## 2026-11-01 current-branch evidence refresh

The preceding exact-head audit was superseded by the current chat branch at
commit `e5dc54cbfdf3fe969c40ea20489fa299e2a88563`, tree
`4ca749e6f6bd075fcf98a28d13ce796b88e61f03`. Rechecking that exact subject
finds the same boundary after the session admission and profile/memory
recovery commits.

The non-test call-site search still finds only the owner method at
`source/host/src/extensions/session/agent_session.rs:309-329`; all other
matches are the inference owner or contract tests. The authenticated Gateway
dispatch at `source/host/app/src/main.rs:3654-3682` still exposes session
listing/creation/deletion, chat send/abort, and an authenticated-context-only
`sessions.patch`; its implementation at `:4171-4282` handles communication
policy and archive lifecycle fields, not model selection. The model-picker
payload validator at `source/host/src/extensions/interactive/payload_service.rs:271-277`
still validates presentation data only and has no action executor or
authenticated session context.

The persisted consumer remains present: `start_routed_provider_task` reads the
session model selection around `source/host/app/src/main.rs:8580-8596` and
projects the override into the provider options. This refresh keeps
`BL-MODEL-PICKER-EXEC-WIRING-001` and
`BL-SESSION-MODEL-MUTATION-WIRING-001` at
`wiring=not-wired`, `verification=not-run`, and `acceptance=pending`. No
runtime change is justified until an existing authenticated producer can
derive the canonical catalog, operator authority, placement and generation
context.

## 5. Fixed-source producer comparison and safe-wiring boundary

The pinned OpenClaw subject has two concrete production mutation paths for
this responsibility. The Telegram callback router resolves a callback against
the live provider/model catalog and invokes `applySessionModelSelection` at
`extensions/telegram/src/bot-handlers.callback-router.ts:546`; the general
auto-reply directive path invokes the same service at
`src/auto-reply/reply/get-reply-directives-apply.ts:355`. Both paths construct
their request from resolved catalog, default-model, session-entry and operator
context rather than accepting visibility or allowed-model policy from an
untrusted request body.

The Fabushi target currently has only the portable
`MessagePresentationAction::ModelPicker` schema/fallback renderer and the
Rust persistence/evaluation owners. The current gateway surface exposes
`sessions.*` and `chat.*`, but no model-selection mutation method; a non-test
search also found no caller for `SandAgentSessionStore::apply_session_model_selection`.
Therefore the model-picker envelope is not evidence of a usable UI or
production mutation path. A route that accepts `visibilityAllow` or
`operatorAllowedModels` from request parameters would violate the existing
policy owner and is explicitly out of scope. Wiring remains pending until a
canonical model catalog/operator-authority producer and an authenticated
session-action caller are available in existing owners.

The policy owner additionally requires an exact canonical-catalog match for
every non-default selection. A request that passes visibility and operator
allowlists but names a `(provider, model)` absent from the catalog must return
the existing `UnknownProvider` rejection and leave the session entry
unchanged. This is an owner-local safety invariant, not production ingress or
wiring evidence.

## 2026-10-31 current-source correction

The consumer portion of the preceding audit is superseded by the current
checkout. `start_routed_provider_task` reads the persisted entry at
`source/host/app/src/main.rs:7528-7530`, and the worker projects its
`model_override` and `thinking_level` into `SandSessionOptions` at
`source/host/app/src/main.rs:8424-8440`. The existing Runner/Inference
consumer edge is therefore confirmed.

The missing edge is the producer: no non-test caller of
`SandAgentSessionStore::apply_session_model_selection` was found. The model
picker and SQLite owners remain valid state/mutation owners, but no
authenticated session-action or gateway/UI mutation ingress reaches them.
`OCRM-RESP-MODEL-PICKER-001` and
`OCRM-RESP-SESSION-MODEL-OVERRIDES-001` remain `not-wired`; this correction
narrows the gap and does not promote verification or acceptance.

## 2026-11-01 exact ingress audit

The negative producer result was rechecked against the current checkout with
the following source boundaries:

- `source/host/app/src/main.rs::call_openclaw_gateway_command` dispatches
  `sessions.list/get/create/delete` and `chat.send/abort`, but no model-picker
  or session-model mutation method.
- `source/host/src/extensions/session/gateway.rs` owns session/profile/transcript
  gateway operations; its exposed mutation cases do not call
  `apply_session_model_selection`.
- `source/host/src/extensions/interactive/payload_service.rs` validates and
  fallback-renders `MessagePresentationAction::ModelPicker`, but has no action
  executor or authenticated session context.
- A repository search for
  `SandAgentSessionStore::apply_session_model_selection` and
  `apply_session_model_selection(` found only the owner definition and the
  inference implementation; contract tests are not production ingress.

This keeps `BL-MODEL-PICKER-EXEC-WIRING-001` and
`BL-SESSION-MODEL-MUTATION-WIRING-001` open. The safe next ingress must be an
existing authenticated session-action producer that derives catalog,
visibility, operator authority, placement, and generation context from their
canonical owners. Accepting those policy inputs from request JSON, or adding a
thin model-selection gateway route solely to reach the existing method, remains
out of scope under the ADR.

## 2026-11-01 exact-head evidence refresh

The preceding negative producer audit and catalog-owner guard were revalidated
against commit `17f175f41`, tree
`dc60c6eaa0b6c648af65adb00412cfb9365ceb5b`. The gateway dispatch at
`source/host/app/src/main.rs:3647-3658` exposes session listing, creation,
deletion, chat send/abort, and an authenticated-context-only `sessions.patch`,
but no model-selection mutation. The `sessions.patch` implementation at
`:4144-4255` accepts only archive lifecycle fields and calls the session
archive owner.

The sole non-test owner entry point remains
`source/host/src/extensions/session/agent_session.rs:266-287`; an exact
repository search for `apply_session_model_selection(` finds that definition,
the inference owner, and contract tests only. The model-picker presentation
validator at `source/host/src/extensions/interactive/payload_service.rs:271-277`
still has no action executor or authenticated session context. Therefore the
consumer edge is wired, while `BL-MODEL-PICKER-EXEC-WIRING-001` and
`BL-SESSION-MODEL-MUTATION-WIRING-001` remain `wiring=not-wired`,
`verification=not-run`, and `acceptance=pending`.

## 2026-11-02 fixed-U model selection to Fabushi request investigation

This is a source-reading addendum, not implementation, executable verification,
source-closure approval, or an acceptance/status promotion. The investigation
continued with the inherited `xhigh` effort. No local test, build, formatter,
generator, or validator was run. Earlier source anchors and production line
numbers above remain historical; they are not evidence of complete reading of
the following locked source.

- Locked upstream `U`: `1440bb1771372334bea23d7642bb52deb7db1845`, tree
  `90c8aab39972da11cfad220361781d49e1bc6700`.
- Target examined: `H=2e1bae4bf081de857eea3f79297c271c6c11b3f9`, tree
  `ca90c3402b62c5360246a31149beda578417e640`. The production blobs below are
  unchanged from `38cbc11c9fa522e69ff02627b8d0de714b2615ee`; the intervening
  changes examined affect management records and source inventory.
- Source was read through immutable Git objects in the fixed-source repository;
  no moving upstream branch was followed.

### Read extent and source dependencies at U

The ranges below record actual source reading, not search-result counts or a
claim that imported dependencies were also read. For partial files, only the
listed ranges are credited by this addendum.

| U path | Exact blob | Actual read extent | Relevant evidence |
| --- | --- | --- | --- |
| `src/agents/model-selection.ts` | `ff1d47f130b0695a5efec953100ccee5d70a70b5` | Full, lines 1-257 | Selection facade keeps persisted selection, runtime model information, and configured/child selection responsibilities distinct. |
| `src/agents/model-selection-config.ts` | `045ebdf5c49479871758fdfd4d15b6f95a5c38d5` | Full, lines 1-148 | `resolveDefaultModelForAgent`; `resolveModelFallbackAvailability` distinguishes a different user-selected provider/model from configured primary and inherited fallback policy. |
| `src/agents/model-selection-persisted.ts` | `4cd75b75b5013ec450fda79be2ca454cdbd60d13` | Full, lines 1-62 | `resolvePersistedOverrideModelRef` preserves an already-resolved provider/model pair instead of reinterpreting aliases; `normalizeStoredOverrideModel` preserves the pair. |
| `src/agents/model-selection-resolve.ts` | `1c46418286644f4febcfd0deb94d7b81ceb01c02` | Full, lines 1-76 | Alias resolution starts with a provider/model pair; allowed-model resolution delegates to the existing alias and status owners. |
| `src/agents/model-selection-shared.ts` | `a98c02f4678c68c469147d079b7693d646c99f4b` | Partial, lines 560-799 | `resolveConfiguredRouteModelLabel`, `resolveConfiguredPrimaryProviderFallback`, and `resolveConfiguredModelRef` preserve resolved identity, process authored primary/aliases, and only then choose configured/builtin fallback. Earlier truncated output is not credited as a full-file read. |
| `src/agents/configured-provider-fallback.ts` | `750fe40902176d40cb8afed3a5ef026683d5f405` | Full, lines 1-54 | Missing-default fallback returns a provider/model pair and excludes a utility-only row from primary intent. |
| `src/agents/embedded-agent-runner/model-resolution.ts` | `e2b4cc262a782e9c968d87d2b0efe38772db1a5a` | Full, lines 1-79 | `resolveTieredModel` carries provider/model and selected-vs-input route provenance into the existing resolver, then checks cancellation/current generation. |
| `src/agents/embedded-agent-runner/run-entry-model-selection.ts` | `e9d494da1192bcd08fe872bde1e179083c6e3cea` | Full, lines 1-68 | Run-entry model hooks use the retained plugin generation and the selected provider/model; they do not establish a new model owner. |
| `packages/ai/src/transports/openai-responses-params-internal.ts` | `84cdbbbf4f3c915df2ffe2dfcd218bedcb70c280` | Partial, lines 95-172 | `buildOpenAIResponsesParams` serializes the resolved `model.id` at line 152. |
| `packages/ai/src/transports/openai-responses-continuation.ts` | `7afa5d6664ab9151d4f9ae996dd3f0ebddc5d74a` | Partial, lines 1-36, 129-160, 445-493, 640-682 | Ordinary automatic continuation compares the request excluding input/instructions/tools and selected turn metadata; `model` remains in that comparison. The explicit-previous-response-id and required-input branches are distinct and are not generalized away. |
| `packages/ai/src/transports/openai-responses-reasoning-update.ts` | `81844147afc9110811ed54838ef0078bf1594ab5` | Partial, lines 22-49 | The imported reasoning-history compatibility guard handles retained configuration-update compatibility; it is not itself a general model-equality check. |

The read boundary does not close the imports behind model normalization,
agent-scope/configured fallbacks, canonical catalog preparation, provider
manifests, runtime hooks, `embedded-agent-runner/model.ts`, or the complete
Responses continuation/replay machinery. Corresponding fixed-U model-selection,
model-resolution-consistency, and continuation tests were located but not read
to completion or executed in this investigation. No full M06/source-equivalence
claim follows from this table. Fabushi-specific environment precedence and the
legacy settings migration policy are target integration contracts, not namesake
upstream contracts.

### Exact target owners and production call chain at H

| H path | Exact blob | Read production boundary |
| --- | --- | --- |
| `source/host/src/extensions/settings/settings_service.rs` | `6c6f94b0098d76b13f8f60d378c9c9ce0c7cbda8` | `get_agent_default_model` at line 778; `agent_default_model_from_settings` at line 1622. |
| `source/host/src/extensions/inference/production.rs` | `0c233457f4cf4410fbd5d482404ae3c7733a5f7a` | Provider routing at line 131; model parsing at line 173; `ProductionAgentInferenceOwner::resolve_requested_model` at line 217; auth projection at line 425; summarization producer at line 447. |
| `source/host/src/extensions/inference/cursor_session.rs` | `eab16367e3a8f3b51fa92e993ebd9b4140d5275a` | Builtin Cursor model at line 18; `RequestedModel` at line 141; `resolve_sand_requested_model` at line 206. |
| `source/host/src/extensions/inference/model_picker_service.rs` | `626c257e65ea3cc82129adda6dce4822ad0092f2` | `SessionModelEntryState` at line 176; existing policy transition at line 288; non-default provider/model pair assignment at line 398. |
| `source/host/src/extensions/session/agent_session.rs` | `57f41acb4c54527309841bd748b685b3786c78fb` | Existing SQLite reader at line 51 and `SandAgentSessionStore::read_session_model_entry` at line 324. |
| `source/host/app/src/main.rs` | `9a6b18d680c92f9a6d6c6a98db1338d8b176ea5a` | Session entry read at line 7825; override projection at line 8721; `turn_model_id` at line 10771; Runner input at line 10847; direct turn-memory options at line 11560. |
| `source/host/src/runner_production_bridge.rs` | `56b1af467b70d7f2ac8f1f156e1e497441928801` | `ProductionRunnerCompositionInput` at line 79 and existing composition factory. |
| `source/host/src/runner/turn_agent_composition.rs` | `69a3e149a4b16d118eb2912c67035d9bf05db76c` | `RoutedProviderRun` construction at line 1066. |
| `source/host/src/runner/routed_provider_runtime.rs` | `62e0e2e3e31bb0e98ce2db0273cd6c5dbff842eb` | Durable checkpoint store at line 253; attempt options at line 345; attempt executor construction at line 488. |
| `source/host/src/extensions/inference/provider_session.rs` | `bd13533ca2b39e6f702de50cd1a08e0810a6f81e` | Checkpoint provider tag at line 210; Fabushi configured fallback at line 387; Fabushi request helper at line 1387; options at line 1455; routed Fabushi arm at line 1572. |
| `source/host/src/extensions/inference/codex_direct_responses.rs` | `14f84d1cdfb93af4d6bb198cbca4f85e59ed6bf8` | Checkpoint at line 69; resume input at line 466; actual JSON model at line 497; accepted tool-boundary checkpoint creation at line 662. |
| `source/host/src/runner/production_turn_run_shell_adapter.rs` | `05ab3efff44de9820d1a9fead3f1bae16322cfb2` | Persist-only checkpoint trait at line 138; per-run `resume_from = None` at line 272; persist-before-resume acceptance at line 318. |

`SettingsService` is already consumed by `ProductionAgentInferenceOwner`.
`main` obtains its Cursor-auth projection, reads `requested_model().model_id`,
and uses that value as `turn_model_id`. Existing Runner composition passes the
Cursor auth through to `RoutedProviderOptions`; it does not pass a separately
resolved Fabushi model. The Fabushi arm ignores that requested model and calls
`configured_fabushi_model()` again. The resulting value enters
`CodexDirectOptions::new` and the actual JSON serializer. This is a missing
production data edge, not a missing settings getter or a reason to add another
runtime owner.

### Three concrete gaps, without status promotion

1. **Provider binding is absent or discarded.** `agentDefaultModel` has the
   existing modelId/maxMode/parameters shape, with no canonical provider
   provenance. Packaged/product mode forces Fabushi even when the stored
   `inferenceProvider` remains Cursor. Current route state cannot establish the
   provenance of an older model selection. Separately, session state already
   has `provider_override` plus `model_override`, and the policy owner writes
   both; `main` currently consumes only the model and thinking level. Its
   existing admission order is persisted session override before raw
   `args.modelId`. The Cursor resolver also folds explicit input, environment,
   experiments and builtin role defaults into a `RequestedModel` with no origin
   marker. Forwarding it wholesale would send implicit grok or the internally
   generated summarization gemini to Fabushi. Model-name blacklists cannot
   distinguish an explicit selection of those same names.
2. **The selected model is not the Fabushi JSON model.** The production owner
   reads persisted defaults, but Fabushi helper line 1407 selects
   `FABUSHI_RESPONSES_MODEL` or `deepseek-chat` independently. Its runtime prompt
   uses that second value, and the actual request at
   `codex_direct_responses.rs:497` uses it as well. `main` turn metadata can
   therefore report a different model. The same main value also feeds the
   existing Cursor web executor at line 10773; correcting Fabushi turn identity
   must not silently change that separate capability's established input.
3. **Continuation has no model identity, and there is no production disk-load
   ingress.** `RoutedProviderCheckpoint` records provider via its enum tag, but
   Fabushi/Codex share a checkpoint containing only input, text, response_id,
   usage and step/tool counters. Resumption replays accepted input under current
   `options.model`. The current serializer sends `store:false` and does not
   send `previous_response_id`; the concrete risk is switching model while
   replaying accepted tool/reasoning context. The existing store persists to
   disk before the adapter accepts an in-memory resume checkpoint, but exposes
   no load method. The adapter initializes its resume state to `None` on each
   invocation, and no production `RoutedProviderCheckpoint` disk-deserialization
   consumer was found. Existing retry continuity and durable serialization are
   real; production cross-process resume is not established by them.

### Pending integration direction, not implemented evidence

The integration owner has selected the following direction for the next spec
and public-interface freeze: an optional canonical provider binding on the
existing `agentDefaultModel`; legacy unbound values retain their prior Cursor
semantics and are not automatically enabled for Fabushi. A Fabushi resolver in
the existing inference owner will consume the raw persisted session
provider/model pair and raw per-turn `modelId`, preserving the existing session
override priority. The internal summarization producer will explicitly provide
no user model instead of forwarding its generated Cursor model. A model is to
be frozen for the existing Runner's requests/retries. The existing checkpoint
will receive an additive `Option<String>` model identity; new Fabushi resume
must reject missing identity or a conflicting model before dispatch.

Those are constraints for subsequent spec/implementation, not behaviors present
at H. No new model mutation route, catalog authority, session store, Agent loop,
scheduler, or permission owner is justified. The eventual direct oracle must
observe the real settings/inference owner, shipping Fabushi request builder,
actual JSON serializer and accepted-checkpoint retry. Configuration reload and
checkpoint serde round trips must be labeled separately from the still-missing
production process-restart resume ingress. This addendum changes none of the
source/ledger, wiring, verification, or acceptance states above.

### 2026-11-02 necessary dependency follow-up at H4

This supplements the preceding read history without replacing its full/partial
labels. U remains `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`. Target H4 is
`ad60d4edb86e385243a67cb12e39187a073c7c01`, tree
`0ba7a184ddf4d5ad26bdf664e6d4f1ac8e98fbc0`. The scoped target blobs cited below
are unchanged from the preceding investigation. The integration owner's current
`../docs/specs/provider-bound-model-identity.md` (`OCRM-MODEL-IDENTITY-v1`) was
read in full; its frozen requirements are not implementation evidence at H4.

| Additional U source | Exact blob | Actual additional read extent |
| --- | --- | --- |
| `src/agents/model-selection-normalize.ts` | `f05e55fdb871be46b4ae2b13ded4e38c93c54bba` | Full, 1-46 |
| `src/agents/model-ref-shared.ts` | `6419f7728cf41a76c62364efb15337b823fb9d14` | Full, 1-180 |
| `src/config/model-input.ts` | `ac8f21493649864348ab503fa7643b4600c5d821` | Full, 1-160 |
| `packages/model-catalog-core/src/provider-id.ts` | `07278f5052d2be88c2244bd25b6bd265e6fb4cfd` | Full, 1-37 |
| `packages/model-catalog-core/src/model-catalog-refs.ts` | `48b2d09a598f59d0e7c1fb2354af03c88f33f3bb` | Full, 1-78 |
| `packages/model-catalog-core/src/model-catalog-refs.test.ts` | `e8b02394338eb4a6cd6d3543dc006ded65fe9ada` | Full source read, 1-38; not executed |
| `packages/normalization-core/src/string-coerce.ts` | `445a9cc4f8daff07f033c3958baf2b798e240e9d` | Partial, 1-18, 60-68, 104-113 |
| `packages/model-catalog-core/src/provider-model-id-normalization.ts` | `95c7a1b613233a925eacb60d06c68d2804ddcb0e` | Partial, 46-63, 106-178, 180-221 |
| `src/agents/model-selection-shared.ts` | `a98c02f4678c68c469147d079b7693d646c99f4b` | Additional partial, 1-72, 396-435 |
| `packages/ai/src/transports/openai-responses-continuation.ts` | `7afa5d6664ab9151d4f9ae996dd3f0ebddc5d74a` | Additional partial, 703-777; reread 126-147, 447-480 |
| `packages/ai/src/transports/openai-responses-client.ts` | `0ea1b3c6063ea1a2a4dc1a9fa0ce988684811282` | Partial, 278-423, 504-662 |
| `packages/ai/src/transports/openai-responses-continuation.test.ts` | `a45413846d46aa94e3b8e370cee9b689c98857db` | Partial source read, 215-253, 361-408; not executed |

The normalization dependency supports separating provider normalization from
selected model identity: U trims/lowercases provider IDs, while model ref parsing
preserves nested model paths and model case. The catalog-ref tests cover those
properties and incomplete refs. `model-input.ts` distinguishes absent primary,
explicit primary and an explicitly empty fallback list. General model-ref
normalization may invoke built-in aliases, manifest rules and runtime hooks;
it is not a harmless cleanup operation for an already-resolved pair. This
supports the preceding persisted-selection rule against reinterpreting selected
identity. The target spec's canonical-only provider input and model syntax
checks remain target contracts, not claims to implement every upstream alias
or plugin normalization rule.

The additional continuation consumer read narrows the upstream claim precisely.
For ordinary automatic continuation, a changed model remains in the request
comparison and produces `request_changed`; the resolver returns the full
request rather than adding the old `previous_response_id`.
`claimOpenAIResponsesHttpContinuation` consumes that result at lines 723-731;
`store:false` also selects a full request. The client passes the claim's request
to stream creation at lines 371-379, commits a new baseline only after a terminal
result at lines 592-594, and releases the claim in `finally` at line 642. This
does not reject every upstream model change. Explicit previous-response-id and
required-input steering retain their separate branches. The stricter Fabushi
accepted-checkpoint missing/conflicting-model refusal in the target spec is a
recovery constraint for its existing tool/reasoning replay, not an assertion of
complete continuation equivalence. The read test slices cover model inclusion
in the request fingerprint and changed-envelope/explicit-continuation behavior;
they do not replace the target checkpoint conflict oracle.

| H4 checked dependency | Exact blob | Actual follow-up read extent |
| --- | --- | --- |
| `source/host/src/extensions/settings/settings_service.rs` | `6c6f94b0098d76b13f8f60d378c9c9ce0c7cbda8` | Partial, 418-488, 1235-1293, 1913-1950 |
| `source/host/src/runner/production_turn_run_shell_adapter.rs` | `05ab3efff44de9820d1a9fead3f1bae16322cfb2` | Additional partial, 349-419, 617-665; earlier persist/acceptance read remains recorded above |
| `source/host/src/runner/transient_stream_error.rs` | `21397c0e1d36abb95e0a0ad070432fea5d407ff7` | Full, 1-75 |
| `source/host/src/extensions/inference/provider_session.rs` | `bd13533ca2b39e6f702de50cd1a08e0810a6f81e` | Additional partial, 34-97, 246-295 |

At H4 the legacy Settings model getter still calls `load`, which projects
`load_checked` failures as empty settings. The snapshot getter likewise preserves
its legacy default projection with invalid source metadata; it is not a checked
`Result` model-reader API. Consequently a new resolver using only that getter
would confuse invalid/unreadable configuration with absent default. The spec's
new checked owner view is required work, not already-present strict behavior.
The existing complete-document and specialized writer compatibility contracts
are not changed by this observation.

The adapter still accepts a resume checkpoint only after durable persist, and
passes the accepted object to the next attempt. Normal protocol/configuration
failures are nonretryable in the existing classification; watchdog handling
remains its existing separate rule. H4's unchanged production checkpoint store
(`routed_provider_runtime.rs`, blob
`62e0e2e3e31bb0e98ce2db0273cd6c5dbff842eb`) still has no disk-load/resume ingress;
the adapter still begins each invocation with no resume checkpoint. Settings
reopening, durable checkpoint serde and in-memory retry must remain separate
claims from production process-restart execution. Catalog/manifest/hook closure,
service model availability and full continuation/restart responsibilities remain
open. This follow-up used source reading only, changed no source/ledger status,
and ran no executable verification.

### 2026-11-02 recovered literal source facts and current bridge boundary

The interrupted batch's prior literal reading observations are preserved in
`projects/openclaw-rust/inventory/read-observations.json`: 34 exact fixed-U
path/blob records, 31 full and three partial, with their original ranges and
attribution. At integration base `0533ca72870a2794c2db32959b9d01427abf325a`,
all 34 identities still match the committed shards. This is identity/history
reconciliation, not a claim that the sources were reread in full during recovery.
The input feeds the existing inventory; it grants no review, verification,
acceptance or global completion state. Existing historical source versions and
all prior reading facts above remain intact.

The full additional reads cover persisted selection, normalization helpers,
selection/config fallback boundaries, Responses parameter/payload policy,
continuation/reasoning update and the complete transport client. Package
manifests establish exports and declared dependencies only. Source tests are
reading evidence, not executed upstream oracles. Earlier dated reads above
remain historical facts, including those not repeated by this reader.

The resolved branch in `model-selection-persisted.ts:18-27` returns trimmed
provider/model identity before the raw parser. The string helper does not
lowercase model IDs. Raw catalog/alias/manifest/plugin normalization therefore
does not execute in that selected branch. The still-open broader frontier
includes `src/plugins/manifest-model-id-normalization.ts`
(`97b0115228cc1deb5a3354e2c0b32f365132e626`),
`src/agents/provider-model-normalization.runtime.ts`
(`109b9e92763d3d4ec819a632c25584a44fc3f2a3`) and `src/shared/model-key.ts`
(`9d5926a9ddbb4f130b08ce360424e336a53c913b`); acquisition of these identities
is not a full-reading claim or a scope exemption.

`buildOpenAIResponsesParams` initializes `model: model.id`. The complete
consumer read establishes two exceptions to any universal wire-identity
claim: `openai-responses-client.ts:232` permits `onPayload` to mutate or
replace the payload, and its Azure path at 666-680 assigns `deploymentName`
to `params.model`. Ordinary continuation compares the prepared request;
`requestWithoutInput` retains model and the normal changed-envelope branch
uses a full request. Explicit previous-response ID and required-input paths
remain separate. The new target Fabushi checkpoint conflict fence is an
explicit target recovery invariant, not a claim of universal upstream refusal.

Necessary source review found no unresolved behavior-changing dependency for
the bounded canonical-provider/literal-model path under the existing authorized
Fabushi route. This is not full transport/catalog/hook closure or a scoped
G01/G02 acceptance verdict. The model responsibility remains mapped/not-run/
pending until exact-H authority, coverage, behavior and independent evidence
are available. Global flags stay false.

The target Rust model path at this integration base already includes later
credential binding and tool-effect custody. Its real current integration oracle
is `fabushi_model_identity_production_contract::persisted_fabushi_model_is_frozen_across_tools_retry_and_summarization`;
the unpublished old four-request test and 17-file tree are not current evidence.
The actual remaining desktop RPC and mirror lose `provider` while reconstructing
the model object. `OCRM-MODEL-BRIDGE-v1` preserves that field through the existing
transport/mirror; Rust Settings remains the only model validation and default
authority. This bridge repair does not supply the trusted catalog/policy inputs
needed for the separate session mutation/picker ingress. Completed-turn marker
reconciliation now exists; interrupted provider continuation and real-service
acceptance remain distinct open responsibilities.
