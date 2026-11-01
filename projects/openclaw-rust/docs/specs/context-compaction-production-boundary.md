# Context Compaction — Production Boundary Specification

Status: active
Owner: OCRM Runner owner
Last updated: 2026-11-01
Related project: `projects/openclaw-rust`
Related responsibility: `OCRM-RESP-AGENT-CONTEXT-COMPACTION-001`

## 1. Context / problem

The fixed OpenClaw session compaction facade delegates behavior to the
agent-core compaction runtime. The behavior includes context-window threshold
decisions, retained-history cut points, summary fitting, transcript compaction
entries, split-turn handling, and no-summary recovery. Fabushi currently has a
production summarization inference binding and persisted prompt compaction
epochs, but no verified production caller that prepares and commits a compacted
conversation history.

## 2. Goal

Provide OpenClaw-equivalent context compaction through the existing Fabushi
Runner, transcript, and inference owners, with one traceable production
producer → compaction owner → durable transcript state → next-turn consumer
chain.

## 3. Non-goals / out of scope

- Do not create a second transcript/session database or parallel agent loop.
- Do not treat prompt/memory `compaction_epoch` snapshots as conversation
  history compaction.
- Do not treat `ContextWindowTracker` or contract fixtures as production
  compaction ownership without a real caller.
- Do not change provider transport behavior or add pricing/tool-schema policy
  under this responsibility.
- Do not advance wiring, verification, acceptance, or completion flags from
  static presence.

## 4. Requirements

- `CC-01`: Use the fixed source closure, including the session facade and the
  agent-core compaction implementation (`shouldCompact`, `prepareCompaction`,
  `compact`, and `compactWithoutSummary`) with exact blob identities.
- `CC-02`: Select the existing canonical transcript/session owner for the
  compaction boundary and define the durable compaction entry schema and
  replay semantics.
- `CC-03`: Connect a real production context/usage producer to the existing
  Runner turn owner; do not infer compaction from a test-only tracker.
- `CC-04`: Preserve recent entries, summary budget, split-turn latest request,
  file-operation metadata, and no-summary fallback behavior.
- `CC-05`: Make cancellation, provider failure, stale turn ownership, retry,
  and duplicate commit behavior fail closed and recoverable.
- `CC-06`: The next turn must consume the compacted state exactly once through
  the existing transcript replay path.
- `CC-07`: The replay adapter must preserve the typed semantics required by
  the fixed session projection, including compaction-summary context,
  assistant/user roles, tool-call/tool-result pairing, and attachment-bearing
  entries. It must not flatten unsupported transcript rows into plain text;
  the existing provider message contract must be extended or adapted at its
  owning boundary before production replay is enabled.

## 5. Current state

The current implementation subject for this project round is
`openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` with tree
`90c8aab39972da11cfad220361781d49e1bc6700`. The exact-subject closure
evidence is maintained in
`dossiers/context-compaction-fixed-closure-20261101.md`; it supersedes the
historical discovery-only measurements below for current-source navigation.
It records fixed-tree relative traversal, workspace/importer joins, a bounded
recursive lock snapshot graph, generated-artifact absence, and source
provenance, while leaving archive, peer-selection, license/SBOM, and Actions
authority gates open.

- Fixed `src/agents/sessions/compaction/compaction.ts` is a 50-line facade
  (blob `43601859d6ee9f015eb96a60111106d0d6026578`); the behavior-bearing
  fixed source is also
  `packages/agent-core/src/harness/compaction/compaction.ts` (blob
  `97e8a7837e695f519570d49e8607752d5fe9237e`). Its direct compaction helpers
  include `compaction-cut-point.ts`
  (`a5995579ac494b118d8dddc8147d1c8dd6ee39c7`), `compaction-details.ts`
  (`26f973ce17756ebe0ae6754798b31eabd2773ba8`),
  `summarization-completion.ts`
  (`1abb82ee55b8501b53ad257863257abdf6899ad5`),
  `summary-checkpoint-prompt.ts`
  (`e04fa9be9e91ae99d40390cc122690905577732e`), and `utils.ts`
  (`eabba4f7b0d8e1d6ad6b4756e60c4d32c1004487`).
- `InferenceService::run_summarization_prompt` is production reachable and is
  bound by `main.rs` to `ProductionTurnAgentBuildBindings`.
- The previously recorded compaction helper list is not the complete fixed
  source closure. The agent-core implementation directly imports the exact
  blobs `packages/agent-core/src/runtime-deps.ts`
  (`0092aa108cb8810564f437f3f83d93e6b9ec153a`),
  `packages/agent-core/src/types.ts`
  (`503be247d7e28cc0cc3e89a29fcf202fffa91872`),
  `packages/agent-core/src/harness/types.ts`
  (`10241e64990e8e0eeee928788c32381f8f68bca5`),
  `packages/agent-core/src/harness/messages.ts`
  (`6a26db0428ea6e4338fe998261f1cbfc2158b1ba`),
  `packages/agent-core/src/harness/session/transcript-model.ts`
  (`1dff7bbc2604ffa5f964cd358c3819ca1cbe659b`),
  `packages/agent-core/src/operator-messages.ts`
  (`747aad850dae0a37355ed2e730eb344ba8afbf2d`), and
  `packages/agent-core/src/reasoning.ts`
  (`cc6dedb7ef6728653e88fc3f669a7f2f01048d3c`). The summarization path also
  imports package-owned AI transport/runtime and normalization modules whose
  exact blobs are not yet recorded here. CC-01 therefore remains pending until
  the complete recursive source/package closure is enumerated and reconciled
  with the fixed tree.
- The canonical Fabushi transcript/state owner is now identified as
  `ProductionSessionWorkers` delegating to the per-agent `SandAgentDb` owner.
  It now also exposes an owner-local guarded compaction commit and replay
  projection. The Runner still has no production caller for those APIs.
- An earlier fixed-commit relative-import census recorded 11,895 exact-tree
  files, four unresolved relative edges, and 261 lexical external-module
  candidates with digest
  `c10d9f6fd8dcf4b978309026fd622252884b6ed43dc136eb17292b8a24f3ed07`.
  That result is retained as historical evidence because its parser contract
  was not preserved well enough to reconcile it with the current instrument.
  The corrected read-only instrument is
  `projects/openclaw-rust/scripts/trace_fixed_relative_imports.py`; at the same
  commit it masks comments, rejects matches starting inside quoted literals,
  follows relative TypeScript imports including emitted `.js` specifiers, and
  reports 11,902 exact-tree files, zero missing roots, two unresolved generated
  artifact edges, and 243 unique lexical external-module candidates. Its
  sorted `(path, blob)` digest is
  `d803c15d6a52b5b9a64d1941f18c3e66c17526434ea9a6ce6ea903070131fa1a` for
  historical commit `ad8d12f126d29b0db204187adf5092db80294e70`. Both this
  historical measurement and the current dossier's bounded measurements are
  closure evidence only, not package closure approval: alias/dynamic import
  completeness, generated bytes, package manifests, licenses, and external
  archive identities still require separate qualification.
- No non-test caller currently performs compaction cut-point preparation,
  compaction-entry commit, or live-history replacement through that binding.
- `runner.startRoutedProvider` currently decodes request `messages` at
  `source/host/app/src/main.rs:6535-6540` and constructs provider messages from
  the request projection at `:6771-6772`. The summarization closure is bound at
  `:9476-9504`, but no caller at that boundary reads the canonical compaction
  replay projection or commits a compaction result.
- Prompt/memory code reads persisted compaction epochs for snapshot rendering;
  this is a separate responsibility.
- Status remains `implementation=not-started`, `wiring=not-wired`,
  `verification=not-run`, `acceptance=pending`.

## 6. Target state

The existing Runner owns the per-turn compaction decision and delegates summary
generation to the existing inference owner. The existing transcript/session
owner atomically persists a guarded compaction entry. Replay reconstructs the
same compacted context for subsequent turns, and cancellation/failure leaves a
valid prior transcript state or an explicit no-summary fallback.

## 7. Architecture and ownership boundaries

- Runner owns threshold decision, preparation, cancellation, and turn identity.
- Inference owns summarization transport and token usage reporting.
- `ProductionSessionWorkers`/`SandAgentDb` remains the canonical owner for
  compaction-entry persistence, replay, and lifecycle/CAS guards. Its
  owner-local guarded compaction contract is implemented; the Runner boundary
  still must supply the real context/usage producer and next-turn consumer
  before this responsibility can be promoted.
- Gateway/Host supplies request context but does not own compaction state.
- Memory prompt snapshots remain owned by the memory/prompt owners.

## 8. Interfaces / data flow

Required flow:

`provider usage/context estimate → Runner compaction decision → fixed-source
cut-point preparation → existing inference summarizer → guarded transcript
compaction entry → existing replay/context builder → next turn`.

The durable entry must identify the first retained entry, summary, token count,
details/file operations, source turn, and lifecycle revision sufficiently to
reject stale or duplicate commits.

The replay/context-builder boundary must expose a typed representation rich
enough for the fixed OpenClaw projection. `ProviderMessage { role, content }`
alone is insufficient for transcript rows that carry tool calls, tool
results, or attachments. Unsupported or malformed rows are an explicit
fail-closed outcome, never an implicit text conversion.

Provider serialization is an inference-owner concern and is provider-specific:
Cursor has an internal typed tool checkpoint but no attachment protobuf;
Codex/Fabushi use Responses input; OpenRouter uses Chat Completions; and
Claude Code currently receives one CLI prompt string. A production replay
adapter must select a serializer for the selected provider and obtain a
provider-supported attachment representation from the existing attachment
owner, or reject the turn without changing durable transcript state.

## 9. Constraints and non-functional requirements

- Preserve user data and transcript ordering on every failure path.
- Bound summary size and retained-context work.
- Keep compaction cancellation-safe and idempotent.
- Avoid leaking sensitive transcript content into logs or diagnostics.
- Retain exact source revision and run identity for verification evidence.

## 10. Failure modes and edge cases

Handle disabled compaction, invalid context-window metadata, zero/negative
reserve values, oversized latest user requests, split turns, empty history,
summary-provider failure, cancellation, stale lifecycle revision, duplicate
commit, concurrent turn admission, malformed prior compaction entries, and
replay after worker recreation. Unknown transcript ownership is a blocker, not
permission to add a new store.

## 11. Implementation strategy

1. Complete the fixed-source transitive closure and map exact source blobs.
2. Identify and document the canonical transcript compaction-entry owner and
   its existing replay contract.
3. Add or repair the production Runner boundary only after steps 1–2 are
   approved in this spec/decision record.
4. Implement the smallest owner-local adapter and guarded persistence path.
5. Add provider-specific typed replay serializers at the inference owner and
   connect only after attachment resolution and unsupported-provider failure
   paths are explicit.
6. Add mutation-sensitive contract tests, then run the required frozen exact
   head verification lane when the development freeze is lifted.

## 12. Verification / test strategy

- Static source review proves `CC-01` and the complete producer/consumer chain.
- Unit or contract oracles cover threshold/cut-point, summary fitting,
  split-turn, fallback, cancellation, stale-owner, and duplicate-commit cases.
- Integration evidence proves replay in a newly constructed worker/turn.
- Exact-head Actions evidence is required for verification and acceptance;
  local OCRM build/test/CI remain intentionally deferred during the freeze.

## 13. Acceptance criteria / Definition of Done

- `AC-CC-01`: Fixed transitive source closure and exact blob identities are
  recorded.
- `AC-CC-02`: One production call chain reaches the canonical transcript owner.
- `AC-CC-03`: All required failure and recovery oracles pass against the same
  source revision.
- `AC-CC-04`: No fixture, helper, prompt epoch, or static symbol is counted as
  production acceptance.
- `AC-CC-05`: A newly constructed production turn consumes a typed compacted
  projection without losing summary, role, tool-pair, or attachment semantics;
  any unsupported row is rejected with the prior transcript state preserved.

## 14. Release / migration / rollback

No release or migration is authorized until the canonical transcript owner and
durable entry compatibility are selected. Rollback must preserve existing
transcripts and disable only new compaction attempts.

## 15. Observability / evidence

Retain source/blob references, exact producer/consumer paths, compaction
decision outcomes, persistence/CAS outcomes, cancellation/failure logs with
redacted content, test run identities, and exact-head Actions artifacts.

## 16. References / provenance

- `AGENTS.md`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/parity-ledger.json`
- `projects/openclaw-rust/dossiers/agent-turn-execution-scoped.md`
- `projects/openclaw-rust/dossiers/not-started-responsibility-frontier-20261031.md`
- `projects/openclaw-rust/dossiers/context-compaction-fixed-closure-20261101.md`
- `projects/openclaw-rust/decisions/2026-11-01-context-compaction-owner-contract.md`
- Historical fixed upstream commit `ad8d12f126d29b0db204187adf5092db80294e70`
- Current implementation subject `1440bb1771372334bea23d7642bb52deb7db1845`
- `src/agents/sessions/compaction/compaction.ts`
- `packages/agent-core/src/harness/compaction/compaction.ts`
- `source/host/src/runner/production_turn_agent_owner.rs`
- `source/host/app/src/main.rs`
- Fixed-source direct dependency audit performed against the pinned upstream
  commit `ad8d12f126d29b0db204187adf5092db80294e70`; unresolved package-owned
  transport/normalization dependencies remain explicitly open above.

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| CC-01 | pending | Current dossier closes only bounded relative/source and selected lock graph observations; recursive source/package/archive/license closure and Actions authority remain open. |
| CC-02 | passed (owner contract) | `ProductionSessionWorkers`/`SandAgentDb` is the canonical owner and implements the transactional compaction/replay contract defined in `decisions/2026-11-01-context-compaction-owner-contract.md`; Runner integration remains pending. |
| CC-03 | blocked | The production route still trusts its request message projection; no pre-run canonical context producer estimates usage, prepares a cut point, and invokes the existing summarizer/owner contract. |
| CC-04 | pending | Existing summarization binding is insufficient to prove compaction behavior. |
| CC-05 | pending | Requires production compaction state and cancellation contract. |
| CC-06 | blocked | No compaction entry is committed or replayed by a production caller. |
| CC-07 | blocked | A strict owner-local typed replay projection now preserves summary, roles, tool-call/result identity, and attachments, and rejects unsupported rows. The owning inference/provider contract still has no verified serialization path; `ProviderMessage` remains role/text-only and unsupported rows remain fail-closed. |
| AC-CC-01 | pending | Exact current-subject dossier exists, but complete transitive source/package/archive/license closure and Actions receipt are not yet present. |
| AC-CC-02 | blocked | Owner commit/replay exists, but the production producer→owner→next-turn consumer chain is still missing; request `messages` cannot serve as that proof. |
| AC-CC-03 | pending | Frozen verification has not run. |
| AC-CC-04 | passed | Current audit explicitly excludes fixtures, helpers, and prompt epochs as acceptance evidence. |
| AC-CC-05 | blocked | No newly constructed production turn consumes a typed compacted projection, and the current inference boundary cannot preserve summary, tool-pair, or attachment semantics without an owner-approved typed provider contract. |
