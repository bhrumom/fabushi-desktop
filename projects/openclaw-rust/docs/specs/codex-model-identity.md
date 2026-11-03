# Persisted Codex model identity in the existing routed Runner

Contract: `OCRM-CODEX-MODEL-IDENTITY-v1` (`CMI-v1`)  
Authoring base: `961d3ca65fc72cacd38eb0e81da768a44d6a1bc2`  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`  
State: specified before implementation; exact-subject executable evidence pending.

## User behavior and current gap

When existing route policy and authorization permit Codex, a saved
`agentDefaultModel` bound to `provider=codex` must reach the actual Codex Responses
request. One admitted turn retains that model through its tool steps, automatic
attempts, same-worker nudges and turn-memory calls. A later admission observes a
later acknowledged selection. Selecting a model never authorizes a provider.

At the authoring base the existing Settings writer, checked reader and desktop
bridge already preserve provider-bound defaults. The complete Fabushi model,
credential-binding and accepted-checkpoint chain also exists. Codex admission
does not consume that default: each actual Codex adapter invocation separately
calls `configured_codex_model()`. Thus the saved selection can be ignored, and
an effect-free retry without a checkpoint can choose a changed config model.
An accepted known-model checkpoint already refuses mismatch; this is not a claim
that it currently replays accepted tools across models.

Decision `CMI-01 — FIX / MISSING_TARGET_CONSUMER`: extend the existing identity
path for the already-authorized Codex route. Do not duplicate earlier Fabushi,
desktop bridge, Settings transactions, checkpoint persistence or retry work.

## Existing owners and public seam

Existing default-model RPC/Host Settings writer -> `SettingsService` checked
read -> `ProductionAgentInferenceOwner` -> shipping routed-turn admission ->
`ProductionRunnerCompositionInput` -> `TurnAgentComposition` ->
`RoutedProviderRun` / actual production attempt executor -> existing Codex adapter
-> `CodexDirectOptions` / Responses serializer -> existing checkpoint and effect
custody. Session remains the sole owner of its existing provider/model pair.

Add `codex_model: Option<String>` beside the existing Fabushi field in owned
composition/run values, and `Option<&str>` in `RoutedProviderOptions`. Add
`with_codex_model` to the existing composition builder. Existing constructors
default to None. Keep `fabushi_model`, its meaning, Native request binding and
all current provider interfaces intact. Root integration owns shared struct
shapes, constructor updates, Host composition and workflow; independent work
must not edit those same paths concurrently.

No new owner, runtime, loop, catalog, permission truth, persistence format,
dependency, session-model mutation API or credential-bearing endpoint is added.
The packaged/ProductMode forced-Fabushi policy is unchanged. Native original-turn
recovery proof remains Fabushi-only; a Codex model must never manufacture it.

## Required behavior

- **CMI-01**: Add `ProductionAgentInferenceOwner::resolve_codex_turn_model` with
  the same three raw inputs as the existing Fabushi resolver. Precedence is a
  complete matching persisted Codex session pair, then explicit turn model,
  checked provider-bound Codex default, then existing Codex environment/config/
  builtin precedence. Validate a present explicit value before precedence so
  a valid session pair cannot conceal malformed input. Missing/null explicit
  input means absent; non-string, blank or control-containing selection fails.
  Preserve case, paths and other ordinary identity characters after trimming.
- **CMI-02**: A partial, blank or different-provider session pair is a refusal.
  An unrelated valid bound default or legacy unbound Cursor default does not
  select the Codex model. Consult the existing checked Settings reader only
  when the default is needed; malformed/unreadable source is not absence.
  Keep route authorization independent and before model admission.
- **CMI-03**: Preserve fallback order `SAND_CODEX_MODEL`, existing `CODEX_HOME`
  `config.toml` string reader, then the current builtin `gpt-5.4`. Do not rewrite
  that file or infer aliases, catalog availability, provider access or fallback
  ladders. Missing/blank optional fallback inputs retain their current behavior.
  A resolved nonempty identity with control characters is an explicit
  Configuration refusal rather than a silent alternative model.
- **CMI-04**: Shipping admission captures Some(model) for Codex once, within the
  existing lease/cancellation boundary and before permission direction,
  attachment staging, tool/registry admission or worker launch. Refusal releases
  the exact accepted lease and does not advance permission direction. Preserve
  concurrent cancellation/replacement controls and all incoming preentry work.
- **CMI-05**: Carry the owned value unchanged through actual composition, every
  attempt and same-worker nudge. Runtime model metadata uses the admitted Codex
  value. For a direct public `run_routed_provider_in_runner` invocation without
  a supplied Codex model, resolve the existing compatibility fallback once
  before tool listing and executor construction; all its attempts retain it.
  Other providers ignore this field and retain their own selection paths.
- **CMI-06**: The actual Codex adapter consumes the borrowed value. Add
  `resolve_codex_request_model(requested, resume)` in the current provider owner:
  validate the chosen identity and refuse a known checkpoint-model mismatch
  before auth/transport/tool activity. Generic Codex legacy checkpoints whose
  model is absent remain supported; the stricter Fabushi legacy refusal remains
  unchanged. Standalone Codex calls with no admitted value use their current
  compatibility fallback for that independent call. Provider-tag, cancellation,
  tool-effect/unknown-outcome and checkpoint persistence fences remain intact.
- **CMI-07**: The existing turn-memory method accepts and forwards the admitted
  Codex value alongside its existing Fabushi values; it must not consult a later
  default. A Codex turn-memory call without that admitted value refuses as
  Configuration before transport, after the existing cancellation check. A
  standalone summarization operation resolves the current authorized
  Codex model through the same Inference owner once. Cursor summarization models
  cannot replace a bound Codex selection. Neither operation grants new access.
- **CMI-08**: Configuration and protocol failures remain typed and nonretryable;
  cancellation retains precedence. Model identity is already represented in the
  existing checkpoint schema. Retain accepted tool/results and count effects;
  model forwarding cannot reset custody, make an unknown effect safe or imply
  interrupted process recovery. Existing checkpoint storage is not a new disk
  load/claim/resume ingress.

## Causal verification and review

All executable checks run only in GitHub Actions on the resulting frozen H.
Retain shipping Host compilation and the current Settings, Inference, actual
Runner/composition, Fabushi model/credential/custody and route controls.

**CMI-O1** uses real persisted/reopened Settings and the actual Inference owner
for precedence, provider isolation, malformed source/raw/pair refusal, preserved
fallback order and later valid admission. App tests exercise the real accepted
lease, permission direction, cancellation and explicit Fabushi-only recovery
proof. No new session mutation authority is fabricated for a fixture.

**CMI-O2** must drive actual production composition/Runner/attempt execution and
the actual Codex request serializer. A private `#[cfg(test)]` thread-local
factory may replace only the final `CodexDirectTransport` boundary. It cannot
replace the resolver, run loop, attempt executor, checkpoint owner or request
assembler. Production retains the existing fixed Codex URL and authentication.
The test seam has no public feature, endpoint override or runtime configuration.
Tests prove that attempts execute on the installed thread, scope the factory
with RAII, release its borrow before callbacks, and restore prior state. If the
real executor moves threads, the seam must refuse/fail rather than claim a pass.
Use owned exact-test child environments for env/config controls; no process-wide
environment mutation in the shared test process. Keep child lifetime bounded,
retain failure output safely and verify actual terminal exit before root cleanup.

Observe model A in final Responses JSON after a real Settings save/reopen. At a
controlled effect-free transient failure commit B (or change fallback config),
then require A on the same turn's retry and B on a new admission. After one
counted guarded tool and durably accepted checkpoint A, require continuation A,
preserved exact history/checkpoint bytes and no repeated effect. Include
known-model mismatch, cancelled dispatch and malformed identity negatives with
zero transport/tool/output/checkpoint effects, plus a real generic legacy
checkpoint positive. Exercise actual turn-memory and summarization consumers.
Preserve the existing Fabushi A/retry/B, Native binding, route and custody cases.

Receipts require actual selected command and capture outcomes, exact positive
names, complete failure diagnostics and subject/input/log identities. A mock
transport proves this bounded request/state contract, never real Codex account,
service, UI or package compatibility. Freeze the complete diff and independently
review all changed paths and necessary consumers before publication; findings
are adjudicated here before a fix and receive a fresh focused review.

## Provenance, open obligations and rollback

`CMI-ORACLE-01 — FIX / TEST_FLAW`: source review of the new shipping admission
fixture found that its two bound default writes omitted the existing required
boolean `maxMode`. Both would be rejected by the unchanged Settings validator
before reaching admission. Add `maxMode: true` to those two fixture values;
retain the production validator and every behavioral assertion.

`CMI-ORACLE-02 — FIX / TEST_FLAW`: the App fixture's default accepted-send
context cannot qualify for Native original-turn binding. Use the actual
persisted-send callback and prove an independent eligible lease can bind;
otherwise `original_turn=None` is a vacuous isolation assertion. Keep the Codex
refusal and absence assertions, and do not change production recovery policy.

`CMI-ORACLE-03 — FIX / TEST_FLAW`: the private test child's post-kill `wait()`
was unbounded. Bound actual retirement observation on failure and Drop; retain
the root when termination cannot be observed. A kill request or exhausted
observation budget is not child completion and cannot permit cleanup or pass.

`CMI-ORACLE-04 — FIX / TEST_FLAW`: the drafted macOS App receipt assumed direct
binary invocation, while the retained producer uses locked Cargo with an exact
test filter. Match that actual command vector, unique command/capture outcomes,
one named pass and 1/0/0; keep the Linux direct-binary receipt separate.

Integration decision `CMI-I01 — FIX / SHARED_CONTRACT_ADAPTATION`: incoming
`6273f9252f308355526c68e833da8dea194bb6cf` publishes PRC-v1 in the existing
Transcript lease and actual shipping admission. Retain its opaque captured
control, one preparation admission, cancellation token, exact retirement and
typed `RoutedTurnAdmissionError`; unwrap only `Refused` in existing model-refusal
assertions. Add a fourth exact App control for Codex: cancellation before a
deliberately invalid persisted model read must return Cancelled, and cancellation
at actual ModelCaptured must refuse permission admission while preserving the
accepted transcript and source bytes. Observe both production boundary points;
a later valid turn must capture the new default and advance permission once.
Do not copy the PRC owner or infer provider/tool execution from this helper test.
Keep all four incoming PRC App controls and the existing registered-worker,
registry, group and Coordinator oracles in both Actions selections.

The canonical unit/responsibility remain `OCRM-UNIT-MODEL-IDENTITY-001` /
`OCRM-RESP-MODEL-IDENTITY-001`; do not create a second inventory or ledger.
The earlier `provider-bound-model-identity.md`, current task and model dossier
retain their historical facts. This extension does not rewrite earlier reads.

Necessary fixed-U observations were directly reread at their exact blobs:
`src/agents/model-selection-persisted.ts`
`4cd75b75b5013ec450fda79be2ca454cdbd60d13` (full 62 lines),
`src/agents/model-selection-config.ts`
`045ebdf5c49479871758fdfd4d15b6f95a5c38d5` (full 148 lines), and
`src/agents/embedded-agent-runner/model-resolution.ts`
`e2b4cc262a782e9c968d87d2b0efe38772db1a5a` (full 79 lines).
They retain resolved provider/model identity and run-scoped selection.
`packages/ai/src/transports/openai-responses-params-internal.ts`
`84cdbbbf4f3c915df2ffe2dfcd218bedcb70c280`, lines 95–172 only, serializes
`model.id`; this is not a full transport/replay/auth review. Their broader
normalizer/catalog/plugin/fallback/auth dependencies remain outside this narrow
target-consumer claim and remain required for full equivalence.

Rollback reverts the additive ephemeral fields and all their callers together;
keep persisted provider/model selections and checkpoints intact. Do not silently
downgrade a bound default into legacy Cursor input or replay a known checkpoint
under another model. Codex account pinning across attempts, real account/service
acceptance, interrupted-turn disk recovery, catalog/aliases/fallback ladders,
full source/legal/dependency closure, UI/platform/upgrade/soak and G01–G12 /
AC01–AC24 remain open. No scoped verified/accepted state or global flag is
promoted by this contract, source review, authored tests or publication alone.
