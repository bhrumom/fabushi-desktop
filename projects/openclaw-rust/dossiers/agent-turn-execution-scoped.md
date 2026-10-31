# OCRM Technical Dossier — Unit 9: Agent Loop, ACP & Turn Execution (`OCRM-UNIT-AGENT-TURN-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M05/M06 (Agent Harness, Turn Execution, ACP & Model Catalog Subsystems)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 core agent execution, loop detection, session compaction, model pricing, and ACP policy files (totaling 1,626 lines) were audited line-by-line and mapped into canonical shards:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0021` | `src/acp/commands.ts` | `ed35270d5976a1700d7b0adc7367b0b5b4352f44` | 45 | 1..45 (full-file) | `OCRM-RESP-AGENT-ACP-COMMAND-001` |
| `part-0021` | `src/acp/policy.ts` | `5cbf002ee4ad1c9e2529417ec4b3d054a73e69fb` | 48 | 1..48 (full-file) | `OCRM-RESP-AGENT-ACP-POLICY-001` |
| `part-0025` | `src/agents/sessions/compaction/compaction.ts` | `43601859d6ee9f015eb96a60111106d0d6026578` | 50 | 1..50 (full-file) | `OCRM-RESP-AGENT-CONTEXT-COMPACTION-001` |
| `part-0025` | `packages/agent-core/src/agent-loop.ts` | `c6d5ce6c43fe0897f47ee71f2e74ac4e93034634` | 1229 | 1..1229 (full-file) | `OCRM-RESP-AGENT-LOOP-RUNNER-001` |
| `part-0025` | `src/agents/tool-loop-detection.ts` | `3924feee594b238384b655fcb8b5db0c96c4b2b2` | 340 | 1..340 (full-file) | `OCRM-RESP-AGENT-LOOP-DETECTION-001` |
| `part-0042` | `src/model-catalog/pricing.ts` | `f69ca7e8c3395725d2db24c7f123f95b9d316cb1` | 172 | 1..172 (full-file) | `OCRM-RESP-AGENT-PRICING-CATALOG-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-AGENT-LOOP-RUNNER-001`**:
   - **Behavior**: Active agent turn run loop, `AlreadyRunning` conflict rejection, and pending turn deferred supersedes.
   - **Rust Owner**: `source/host/src/runner/mod.rs:TurnRunShell`.
   - **Oracles**: `OCRM-AGENT-O01`, `OCRM-AGENT-O02`.

2. **`OCRM-RESP-AGENT-LOOP-DETECTION-001`**:
   - **Behavior**: Tool loop detection, immediate cancellation supersedes, and monotonic generation increments in `TurnOwnerToken`.
   - **Rust Owner**: `source/host/src/runner/mod.rs:TurnOwnerToken`.
   - **Oracles**: `OCRM-AGENT-O03`, `OCRM-AGENT-O04`.

3. **`OCRM-RESP-AGENT-CONTEXT-COMPACTION-001`**:
   - **Behavior**: Context window token overflow compaction threshold triggering, history message pruning, and summary preservation.
   - **Rust Owner**: `source/host/src/runner/mod.rs:TranscriptCompactor`.
   - **Oracles**: `OCRM-AGENT-O07`, `OCRM-AGENT-O08`.

4. **`OCRM-RESP-AGENT-PRICING-CATALOG-001`**:
   - **Behavior**: Model token cost estimation, prompt and completion token multiplier accounting, and unknown model fallback rate.
   - **Rust Owner**: `source/host/src/runner/mod.rs:PricingCatalog`.
   - **Oracles**: `OCRM-AGENT-O09`, `OCRM-AGENT-O10`.

5. **`OCRM-RESP-AGENT-ACP-COMMAND-001`**:
   - **Behavior**: ACP available commands whitelist resolution, viral turn tainting on unreviewed command execution, and auto-review resolution.
   - **Rust Owner**: `source/host/src/runner/mod.rs:AvailableCommand`.
   - **Oracles**: `OCRM-AGENT-O05`, `OCRM-AGENT-O06`.

6. **`OCRM-RESP-AGENT-ACP-POLICY-001`**:
   - **Behavior**: ACP dispatch policy enforcement, runner settlement receipt emission, and zombie lease watchdog timeout.
   - **Rust Owner**: `source/host/src/runner/mod.rs:SandAutoReviewDecision`.
   - **Oracles**: `OCRM-AGENT-O11`, `OCRM-AGENT-O12`.

## 2.1 Current live-candidate drift — 2026-10-31

The historical ACP command source above is bound to the then-locked subject
`ad8d12f126d29b0db204187adf5092db80294e70` and remains historical evidence.
Read-only GitHub contents checks against the current discovery candidate
`openclaw/openclaw@b924498923dff81a02fb8a89d57828717c6e160d` found no
`src/acp/commands.ts`; the path returns 404. The command surface is now
represented by at least:

- `src/auto-reply/commands-registry-list.ts::listChatCommands`, which adds
  skill command projections and filters operator-disabled command keys;
- `src/auto-reply/commands-registry.data.ts::getChatCommands`, which caches
  the built-in registry and supplies runtime thinking-level choices; and
- `src/auto-reply/commands-registry.runtime.ts`, which exports the lazy-boundary
  runtime facade.

This is a source-contract drift observation, not a current-source closure or
implementation approval. The existing parity row for
`OCRM-RESP-AGENT-ACP-COMMANDS-001` must retain its historical source identity
until a fresh exact-subject Actions inventory and dependency closure accepts
the live candidate. A future rebaseline must remap the responsibility from the
old ACP-only `AvailableCommand` shape to the current chat-command registry,
including feature-flag filtering, skill command injection, aliases, argument
parsing, scope/category metadata, and the boundary between chat commands and
ACP presentation. No Rust target or completion flag is upgraded by this note.

## 2.2 ACP Rust owner/consumer audit — 2026-10-31

The current Fabushi checkout contains
`source/host/src/extensions/plugin_sdk/acp_runtime.rs`, but its
`AcpSessionManager`, `verify_session_access`, and `generate_doctor_report`
symbols have no production caller. The only references are
`source/host/tests/openclaw_plugin_sdk_contract.rs` and the module export in
`plugin_sdk/mod.rs`. The manager owns an in-memory session map and its doctor
method returns `Ready` from that local map; this is a contract slice, not the
canonical Session/Runner ACP lifecycle or a live ACP transport.

The existing Runner search likewise found no `AvailableCommand` or
`get_available_commands` implementation, and no production route exposing the
historical ACP command list. The current live OpenClaw candidate's command
registry drift described above means the old parity integration
(`runner/mod.rs` as both producer and consumer) is not evidence of real
production wiring. A valid implementation must first identify the canonical
ACP handshake/transport owner, then adapt the current Session/Runner state and
command registry without creating a second session store or treating a test
manager as a production doctor.

Disposition: keep the ACP command responsibility
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending`; retain
the historical source and implementation contract until exact-subject source
closure permits a rebaseline. The standalone `AcpSessionManager` contract
does not satisfy production consumer, security-context, or packaged ACP
acceptance evidence.

The ACP policy row requires the same separation. A repository-wide search
found no production ACP dispatch-policy producer or consumer for the historical
`src/acp/policy.ts` contract. `SandAutoReviewDecision` is a policy-shaped
Runner helper, while `source/host/src/extensions/transcript/production_runtime.rs`
owns a separate Host turn lease and watchdog lifecycle. The O11 settlement and
O12 zombie checks in `openclaw_agent_turn_contract.rs` construct local
`TurnSettlement`/`TurnWatchdogLease` fixtures and do not establish an ACP
transport or dispatch-policy call chain. Therefore the parity row
`OCRM-RESP-AGENT-ACP-POLICY-001` retains `implementation=not-started`,
`wiring=not-wired`,
`verification=not-run`, and `acceptance=pending`; the independent Transcript
runtime must not be counted as ACP compatibility.

## 2.2.1 Model-pricing and provider-fallback production reachability audit — 2026-11-01

The current checkout was re-read for `OCRM-RESP-AGENT-MODEL-PRICING-001`.
`source/host/src/runner/turn_usage.rs` owns safe token conversion and
`merge_turn_usage`; its output is carried into runner/subagent state and
telemetry, but no price lookup or monetary/token-cost calculation consumes it.

`source/host/src/extensions/inference/model_catalog_service.rs` defines
`ModelCatalogCost`, remote pricing rows, and `project_remote_model_catalog`.
That function only projects wire data into provider/pricing/recommendation
tables. A current-source search found no production consumer of
`RemoteModelCatalogProjection::pricing` or `upstream_pricing` that multiplies
turn usage by those rates.

The 429-related paths are separate transport concerns:
`source/host/src/runner/transient_stream_error.rs` classifies rate-limit
failures, `source/host/src/extensions/inference/llm_transport_rate_limit.rs`
queues/admit-waits per route, and `stream_wrappers_service.rs` provides a
service-tier cooloff projection. None selects the next configured provider or
records a provider-fallback chain for a turn. `merge_turn_usage` and transport
retry/rate-limit helpers are therefore positive partial slices only, not the
pricing-and-failover responsibility.

`OCRM-RESP-AGENT-MODEL-PRICING-001` remains
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending`. Selecting an owner requires one canonical pricing
consumer and one provider-routing/fallback admission boundary; adding a
second catalog or treating telemetry as billing evidence is out of scope.

The context-compaction mapping was also rechecked. A non-test search found no
production caller for `ContextWindowTracker::check_compaction`; production
prompt code currently reads a persisted compaction epoch through
`HostRunnerComposition`. The O09 contract calls the tracker directly, while
O10 defines a local `validate_tool_schema` fixture inside the test file rather
than a production tool-admission owner. The epoch projection and those tests
therefore do not prove the mapped threshold-compaction plus tool-schema
responsibility is implemented or wired. The parity row
`OCRM-RESP-AGENT-CONTEXT-COMPACTION-001` is corrected to
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending` until the real Runner producer/consumer path exists.

## 2.2.2 Context-compaction source-closure and production-boundary recheck — 2026-11-01

The fixed `src/agents/sessions/compaction/compaction.ts` file is a session SDK
facade: its `generateSummary` delegates to the agent-core runtime. The
behavior-bearing fixed-source closure therefore also includes
`packages/agent-core/src/harness/compaction/compaction.ts`, whose
`shouldCompact`, `prepareCompaction`, `compact`, and `compactWithoutSummary`
functions own the cut-point, retained-history, summary-budget, and fallback
semantics. The current responsibility mapping records only the facade path and
does not by itself prove the transitive agent-core source closure.

The current Fabushi tree does have a real summarization transport producer:
`source/host/app/src/main.rs` binds `InferenceService::run_summarization_prompt`
to `ProductionTurnAgentBuildBindings`, and
`ProductionTurnAgentOwner::run_summarization_prompt` can forward to that
binding. A repository-wide non-test search found no caller that performs
cut-point preparation, commits a compacted transcript entry, or replaces the
live turn history through that binding. The existing `compaction_epoch`
projection is used by prompt/memory snapshot rendering and is not evidence of
OpenClaw conversation-history compaction.

Disposition remains fail-closed: the summarization producer is a reusable
dependency, but no compaction producer→owner→transcript state→turn consumer
chain is present. Do not wire `ContextWindowTracker` or a new parallel
compactor into the turn path until the transitive fixed-source contract,
canonical transcript commit owner, and recovery/abort behavior are specified.
`OCRM-RESP-AGENT-CONTEXT-COMPACTION-001` therefore remains
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending`.

### 2.2.3 Canonical transcript owner contract audit — 2026-11-01

The current source is sufficient to identify the canonical state owner even
though it is not sufficient to implement compaction. `ProductionSessionWorkers`
opens the per-agent `SandAgentDb` and exposes transcript read/page/window/tail,
append, update, single-entry delete, and conversation-clear operations through
`source/host/src/extensions/session/production.rs:1152-1189` and
`:1711-1748`. The underlying owner implements those operations in
`source/host/src/extensions/session/agent_db.rs:1266-1401`.

The owner-local contract is now implemented by
`SandAgentDb::commit_compaction` and
`SandAgentDb::get_compacted_transcript_entries`, exposed through
`ProductionSessionWorkers::commit_agent_compaction` and
`read_agent_compacted_transcript_entries`. The commit uses one immediate
transaction, validates the expected lifecycle revision and cut point, advances
the revision, rejects conflicting duplicate identities, and leaves prior rows
unchanged on failure. The focused owner contract test is
`source/host/tests/agent_db_owner_contract.rs::owner_commits_guarded_compaction_and_replays_from_cut_point`.

The existing `compaction_epoch` field remains a prompt/profile snapshot
projection and is not used for conversation-history compaction. The remaining
blocker is now the real Runner producer and next-turn consumer: no production
caller prepares a cut point, invokes summarization, commits this owner API, or
replays its projection. The full responsibility therefore remains
`not-started/not-wired/not-run/pending` until that chain exists.

### 2.2.3a Fixed-source transitive closure correction — 2026-11-01

The fixed-source compaction closure is wider than the seven helper files
previously recorded in this dossier. The agent-core implementation directly
imports the pinned blobs for `runtime-deps.ts`
(`0092aa108cb8810564f437f3f83d93e6b9ec153a`), `types.ts`
(`503be247d7e28cc0cc3e89a29fcf202fffa91872`), harness `types.ts`
(`10241e64990e8e0eeee928788c32381f8f68bca5`), harness `messages.ts`
(`6a26db0428ea6e4338fe998261f1cbfc2158b1ba`), session `transcript-model.ts`
(`1dff7bbc2604ffa5f964cd358c3819ca1cbe659b`), `operator-messages.ts`
(`747aad850dae0a37355ed2e730eb344ba8afbf2d`), and `reasoning.ts`
(`cc6dedb7ef6728653e88fc3f669a7f2f01048d3c`). The summarization path also
crosses package-owned AI transport/runtime and normalization modules. Those
package blobs and their recursive imports are not yet recorded in the
compaction source ledger, so `AC-CC-01` remains pending and no Rust equivalence
claim is promoted from this correction.

The earlier fixed-tree closure measurement from the seven compaction
entry/helper roots reached 11,895 exact-tree files, with four unresolved
relative edges and 261 lexical external-module candidates; its recorded digest
was `c10d9f6fd8dcf4b978309026fd622252884b6ed43dc136eb17292b8a24f3ed07`.
Because that earlier parser contract was not retained, it is historical and is
not silently treated as equivalent to the corrected instrument.

The corrected read-only instrument is
`projects/openclaw-rust/scripts/trace_fixed_relative_imports.py`. It reads only
the pinned Git tree, masks comments, rejects matches beginning inside quoted
literals, follows relative TypeScript imports including emitted `.js`
specifiers, and never writes the source checkout or ledger. At the same
commit it found 11,902 exact-tree files, zero missing roots, two unresolved
relative edges, and 243 unique lexical external-module candidates. The sorted
`(path, blob)` digest is
`d803c15d6a52b5b9a64d1941f18c3e66c17526434ea9a6ce6ea903070131fa1a` for
`ad8d12f126d29b0db204187adf5092db80294e70`. The two unresolved edges are the
generated Kysely imports from `src/state/openclaw-agent-db.generated.ts` and
`src/state/openclaw-state-db.generated.ts` into `.artifacts/kysely`. This is
still a measurement frontier, not accepted source/package closure: parser
limitations, alias and dynamic-import completeness, generated/native inputs,
package manifests, licenses, and external archive identities remain open.

### 2.2.3b Generated Kysely artifact frontier — 2026-11-01

The two unresolved relative edges are deliberate generated-artifact imports,
not evidence that the fixed Git tree contains a second source spelling. At the
fixed commit, both `src/state/openclaw-agent-db.generated.ts` and
`src/state/openclaw-state-db.generated.ts` contain only type re-exports into
the ignored `.artifacts/kysely/` directory, and no `.artifacts/kysely` blob is
present in the Git tree. The fixed source identities relevant to a future
reconstruction are:

| input | blob | role |
|---|---|---|
| `scripts/generate-kysely-types.mts` | `08060fa786ea57601647279c8302cba8b78be544` | generator and output/fingerprint contract |
| `scripts/lib/direct-run.mjs` | `d8ae6de3c3111423342c0a479ecfb858a947b292` | generator direct-run gate |
| `src/state/openclaw-agent-schema.sql` | `2cd8721c3db180afd9ff935f5128c659d52bd38d` | 882-line agent schema input |
| `src/state/openclaw-state-schema.sql` | `140a0771581c95bd193c402bd02b99224730cf6b` | 2,759-line state schema input |
| `package.json` | `af543c4c033b5c5e0c466061ada470feeda777f3` | `db:kysely:gen` / `db:kysely:check` command contract and `kysely` version declaration |
| `pnpm-lock.yaml` | `80fb4d9fa985297a24ac88931ef534e85a40ba47` | exact dependency-lock input |

This establishes a bounded generated-input frontier, not generated-output
identity or package closure: no generator execution was performed under the
development freeze, the ignored output bytes are not available in the fixed
Git tree, and the Kysely/runtime package archive, licenses, native/runtime
inputs, and generated-output comparison remain unverified. The two unresolved
edges therefore remain open for source-closure purposes.

### 2.2.3c External-module frontier reconciliation — 2026-11-01

The corrected census reports 243 unique external specifiers. A read-only
package-root normalization against the same fixed tree reduces these to 107
roots: 40 Node built-in specifiers, 18 `@openclaw/*` or root `openclaw`
workspace/root-package entries, and 57 third-party or otherwise external
roots. The fixed tree contains manifests for the relevant workspace packages
such as `@openclaw/agent-core`, `@openclaw/ai`, `@openclaw/llm-core`,
`@openclaw/normalization-core`, `@openclaw/acp-core`, and the root
`openclaw` package, but it does not contain the published package archives for
the external roots. This is a package-name frontier only: subpath exports,
lockfile resolutions, package tarball bytes, native/optional platform inputs,
license/notice payloads, and provenance remain to be independently qualified.
The same instrument's optional `--include-external-sources` mode preserves the
fixed-tree source paths for all 243 specifiers (12,392 source/specifier
bindings in this run), so later responsibility review can trace a package
frontier back to concrete producers rather than relying on package names alone.
It therefore does not promote `source_closure_ready` or
`dependency_closure_enumerated`.

The closure instrument's optional
`--include-workspace-package-frontier` mode further reconciles each non-Node
specifier against fixed-tree `package.json` exports. The same frozen run
reported 203 non-Node package specifiers: 102 had a fixed-tree package
manifest, while 101 had no fixed-tree package manifest. For manifest-backed
entries it recorded the manifest blob, export key, declared `dist` targets,
whether those targets exist in the fixed tree, and conservative `src/*.ts`
candidate paths. Of 198 declared targets, 4 were present in the fixed tree
(`packages/session-url-contract/src/*` and
`packages/workboard-contract/src/index.ts`); the remaining targets were
absent. Some source candidates (for example
`packages/normalization-core/src/agent-id.ts`) were present. This is useful
source-to-build provenance, but source candidates are not generated-output
equivalence and absent package manifests remain an external archive frontier.
The closure digest remains
`d803c15d6a52b5b9a64d1941f18c3e66c17526434ea9a6ce6ea903070131fa1a`.

As a separately labelled inferred measurement, recursively following the 91
present `src/*.ts` candidates with the same lexical relative-import resolver
reached 545 candidate files and 2,577 relative edges, with zero unresolved
relative edges and 68 unique external specifiers. This candidate closure is
not merged into the 11,902-file imported closure or its digest: the `dist` to
`src` mapping is a manifest-based source hypothesis, not proof of the runtime
build graph, generated-output identity, or package equivalence. It is retained
only to guide the next exact package/source qualification step.

The same fixed-tree `pnpm-lock.yaml` was then read without installing or
resolving dependencies. Among the 75 non-Node package roots in the imported
frontier, 60 had one or more exact package keys in the lockfile's `packages`
section, while 15 were workspace/root packages with no external lock package
key (`@openclaw/acp-core`, `@openclaw/agent-core`, `@openclaw/ai`, the other
fixed workspace packages, and `openclaw`). This is lock declaration evidence
only: it does not prove importer selection, peer/optional platform closure,
downloaded archive bytes, registry integrity, source provenance, or
license/notice completeness. The lockfile remains bound to blob
`80fb4d9fa985297a24ac88931ef534e85a40ba47` at the fixed upstream commit.

### 2.2.3.a Fixed lockfile structural limitation — 2026-11-01

A raw fixed-blob structure check found that `pnpm-lock.yaml` at the same H is
not one ordinary YAML document: it contains a first
`lockfileVersion`/`importers`/`packages`/`snapshots` fragment at lines 2–158,
then another `lockfileVersion` and lockfile metadata at line 159, a second
`importers` and `packages` section at line 3193, and a second top-level
`snapshots` section at line 11165. The blob is 19,772 lines long. A generic
YAML loader therefore silently applies last-key-wins behavior to duplicate
top-level keys and cannot serve as semantic lockfile authority.

The earlier 75-root/60-lock-key reconciliation is consequently retained only
as bounded raw-text declaration evidence from this fixed blob. It must not be
read as a valid parsed importer/package graph. Exact package archive,
integrity, peer/optional-platform, provenance, and license/notice closure
remain unqualified until a source-approved instrument can distinguish the
lockfile fragments and prove which fragment is authoritative. This limitation
does not alter the imported-closure digest or any completion flag.

For bounded diagnostics only, the new raw-blob instrument
`scripts/qualify_fixed_lockfile.py` split on the repeated
`lockfileVersion` header and counted only exactly-two-space section children.
Fragment 2 is 157 lines with one importer, 15 package keys, and 15 snapshots;
fragment 3 is 19,614 lines with 194 importers, 1,844 package keys, and 1,180
snapshots. Against fragment 3 alone, the 75 non-Node roots reproduce 60
matched roots and 15 workspace/root exceptions. The matched roots expose 82
distinct declared resolution-integrity values in the raw fragment; several
roots have multiple versions (for example `zod` has four and `commander` has
three). This narrows the diagnostic frontier but does not select fragment 3 as
authoritative, prove importer selection or peer/optional closure, or establish
archive bytes and license/notice provenance.

The reproducible command was:
`python3 projects/openclaw-rust/scripts/qualify_fixed_lockfile.py --repo .
--commit ad8d12f126d29b0db204187adf5092db80294e70 --frontier-json
/tmp/fabushi_fixed_closure_20261101.json`. It reported raw lockfile SHA-256
`8ee0f9e7c0bcdecec3d791dff04f2df1f5c608c60e57fbafaf241c765eebd8d7`; the
SHA-256 is an observation of the fixed blob content, not a package archive or
dependency-closure receipt.

The closure instrument was re-run read-only on 2026-11-01 with
`--include-workspace-package-frontier` against the same commit. It reproduced
`file_count=11902`, `external_module_candidates=243`, and
`path_blob_sha256=d803c15d6a52b5b9a64d1941f18c3e66c17526434ea9a6ce6ea903070131fa1a`;
the frontier remained 203 entries, with 15 fixed-tree manifests, 101
no-manifest entries, four present declared targets, and 91 present source
candidates. The two generated Kysely edges remained unresolved. This is
reproducibility evidence for the instrument and fixed-tree observations only;
it does not promote source, dependency, build, license, or production
acceptance flags.

### 2.2.4 Recovery/checkpoint owner boundary audit — 2026-11-01

The adjacent production state owners were read as possible hidden compaction
consumers. `ProductionAgentStore::get_full_conversation` only hydrates the
latest AgentStore checkpoint through the blob resolver, while
`SessionConversationState::resolve_conversation_state_bytes` only resolves the
checkpoint's existing blob references into turns, todos, and an optional
summary. Neither path selects a cut point or reads the owner-local transcript
replay projection.

`ProductionAgentStateCheckpointSink` likewise remains a turn-settlement owner:
it captures `prior_state_bytes`, validates that the live checkpoint root has
not changed, appends the generated turn, and persists the resulting checkpoint
and transcript mirror. Its summary-archive handling only suppresses stale token
details and does not invoke summarization or create a compaction entry.
The generated turn stream validates existing canonical blob references before
provider execution, then stages and persists the ordinary generated turn; it
contains no pre-run compaction decision or post-summary replay substitution.

This closes the likely alternate-owner audit without changing disposition:
the Runner still lacks the producer that estimates canonical context, prepares
the fixed-source cut point, calls the bound summarizer, commits
`commit_agent_compaction`, and supplies `read_agent_compacted_transcript_entries`
to the next turn. The responsibility remains
`not-started/not-wired/not-run/pending`.

### 2.2.5 Runner request-projection boundary — 2026-11-01

The current shipping route does not provide a safe compaction producer. In
`source/host/app/src/main.rs:6535-6540`, `start_routed_provider_task` decodes
`messages` from the routed request; at `:6771-6772` it builds provider messages
from that request through `RunnerPromptGlueOwner`. The summarization binding is
constructed later at `:9476-9504` and passed into
`ProductionTurnAgentBuildBindings`, but the turn path contains no call that
estimates canonical transcript context, prepares a cut point, calls
`run_summarization_prompt`, or commits/replays the owner-local compaction API.

Replacing request messages with the replay projection at this point would be
an unreviewed change to the existing prompt/input owner and would leave system
prompt, attachment, tool-result, split-turn, and cancellation semantics
undefined. The next implementation seam is therefore a Runner-owned pre-run
context coordinator that consumes canonical session state and compaction
settings before prompt projection; the request body remains an input hint, not
durable compaction state.

The tool-loop mapping was rechecked after the detector implementation landed.
`TurnRunShell::ToolLoopGate` now canonicalizes tool arguments, bounds per-turn
history, classifies the third consecutive identical call, and requests
cancellation through the existing `RoutedProviderCancellation`. The live
chain is `ProductionTurnAgentOwner` -> `TurnAgentComposition` ->
`CheckpointRecordingRoutedToolBridge::call_tool` -> the existing routed
provider executor. The parity row
`OCRM-RESP-AGENT-LOOP-DETECTION-001` is therefore
`implementation=implemented`, `wiring=wired`, `verification=not-run`, and
`acceptance=pending`; O03/O04 remain lifecycle oracles and do not replace the
dedicated frozen tool-loop verification.

The model-pricing mapping was rechecked as well. `TurnUsage` is a real
production-carried usage shape, but the current non-test source has no
`PricingCatalog`/model-cost calculation caller. The remote catalog projection
and `RateLimitAdmissionController` are defined under the Inference extension
without a production consumer in this path; the Runner adapter's HTTP status
handling provides retry/timeout classification, not the mapped provider
fallback chain. The O07/O08 tests therefore do not establish the combined
pricing and fallback responsibility. The parity row
`OCRM-RESP-AGENT-MODEL-PRICING-001` is corrected to
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending` until model pricing and provider fallback have a real
producer/consumer path.

The loop-runner mapping has a narrower partial positive. The production
`ProductionTurnRunShellAdapter` calls `TurnRunShell::begin_run`, marks the
owner dispatched, and settles the active run, so the active-run lifecycle is
reachable. However, the deferred supersede behavior in O02 is implemented by
the local `DeferredTurnQueue` declared inside
`openclaw_agent_turn_contract.rs`; no production queue producer/consumer was
found. Because the mapped responsibility requires both `AlreadyRunning`
handling and deferred supersede, the parity row
`OCRM-RESP-AGENT-LOOP-RUNNER-001` is corrected to
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending`. The active-run adapter remains recorded as partial
evidence, not full responsibility acceptance.

### 2.3 Exact-head correction: production transcript queue (2026-11-01)

The preceding paragraph is stale against the current checkout and is corrected
here. `ProductionTurnDispatch` is a real owner in
`source/host/src/extensions/transcript/send_turn_dispatch.rs`: it owns a
`RunScheduler`, enqueues accepted turns, starts the first turn, waits for an
active generation, and settles the completed generation before starting the
next queued turn. `ProductionTranscriptRuntime::accept_routed_send` and
`execute_send_with_acceptance_observer` are production callers; they also run
the watchdog path and advance the queue after watchdog escape or terminal
settlement (`source/host/src/extensions/transcript/production_runtime.rs`).

This proves a production active-run admission/serialization and deferred
turn-queue path for O01/O02. It does not prove the exact historical
`TurnRunShell`/steering implementation, nor packaged/runtime acceptance. The
responsibility should therefore be calibrated to
`implementation=implemented`, `wiring=wired`, `verification=not-run`, and
`acceptance=pending`, with the existing production queue as owner and the
historical contract test retained only as oracle evidence.

The parity row's blocker attribution was rechecked at the same time. The prior
`BL-MEDIA-DOC-TRUNCATION-UPSTREAM-CALLER-001` reference did not describe this
ACP transport/consumer gap; the row now points to the existing
`BL-PLUGIN-SDK-ACP-WIRING-001` blocker. This is a blocker-classification
correction only and does not promote implementation, wiring, verification, or
acceptance.

## 2.3 Fixed-subject loop-runner source and owner recheck — 2026-10-31

The fixed implementation subject was re-read at the exact OpenClaw commit
`ad8d12f126d29b0db204187adf5092db80294e70`, rather than using the moving
default branch. The relevant production loop sources are:

| Source | Blob | Verified responsibility |
| --- | --- | --- |
| `packages/agent-core/src/agent-loop.ts` | `c6d5ce6c43fe0897f47ee71f2e74ac4e93034634` | turn loop, tool batches, steering checkpoints, final follow-up drain |
| `packages/agent-core/src/agent-loop-steering.ts` | `8c6f1bade7526eb67e6e7813c34542cf0b454ba9` | bounded/drained steering access at loop checkpoints |
| `src/agents/tool-loop-admission.ts` | `bf5d08bb075195a0488c0e0747141f2d61f16605` | tool-loop admission and batch reservation, not turn queue ownership |

This recheck corrects the earlier source association: the deferred behavior
does not come from `tool-loop-admission.ts` alone. It is an agent-core
checkpoint/steering contract. In Fabushi, `TurnRunShell` remains the sole
active-run owner and `ProductionTurnRunShellAdapter` is a real active
lifecycle consumer, but `DeferredTurnQueue` is still declared only in
`source/host/tests/openclaw_agent_turn_contract.rs`.

The shipping host path creates one `ProductionTurnAgentOwner` for a worker
request before calling `run_routed_provider_with_projected_turn_input`; no
production ingress currently submits a deferred item to that owner, and no
production checkpoint consumer drains such a queue. Adding a queue copied
from the fixture would therefore create an unproven second contract and could
leak state across worker recreation. The task-specific specification
`projects/openclaw-rust/docs/specs/agent-loop-runner.md` records the required
producer/consumer boundary and keeps this responsibility at
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`,
`acceptance=pending` until that boundary is found or deliberately designed
under the canonical owner.

## Steering-path disambiguation — 2026-10-31

A fresh non-test source review found two real steering-shaped paths, but neither
is the missing main-Agent-loop deferred-input contract:

- `source/host/src/runner/subagent_runtime.rs:187-197,303-365` owns
  `SubagentRuntime::pending_steers`. Its producer is `steer_subagent`; settlement
  converts the pending value into a `SteerContinuation` for the same subagent.
  This is a subagent-specific runtime owner with its own session/running sets,
  not `TurnRunShell` state and not the main agent-core checkpoint queue.
- `source/host/src/extensions/plugins/hook_runner.rs:235-310` contains
  `ActiveRunSteeringManager`, which validates prompt-injection policy, streaming
  state and idempotency. It returns an admission-shaped outcome but does not
  enqueue, checkpoint-consume, or settle a Runner turn.

The actual production main-turn path remains
`source/host/app/src/main.rs:9651-9657` ->
`ProductionTurnAgentOwner::run_routed_provider_with_projected_turn_input`
(`source/host/src/runner/production_turn_agent_owner.rs:473-505`) ->
`run_production_turn_shell_lifecycle`, with no call from either steering path
into `TurnRunShell` deferred state. Treating either path as RLR-02/RLR-04 would
cross owner boundaries and falsely promote a second queue. The parity row
therefore remains `implementation=not-started`, `wiring=not-wired`,
`verification=not-run`, `acceptance=pending`; this is stronger negative
evidence for the existing architecture blocker, not a new implementation.

## Fixed-subject ACP source and transport boundary recheck — 2026-11-01

The fixed upstream subject was checked directly at
`ad8d12f126d29b0db204187adf5092db80294e70`; all four ACP roots used in this
boundary audit are present and their identities are:

| Source | Blob | Direct contract observed |
| --- | --- | --- |
| `src/acp/commands.ts` | `ed35270d5976a1700d7b0adc7367b0b5b4352f44` | Returns a copied static `AvailableCommand[]` containing built-in command names, descriptions, and input hints. |
| `src/acp/policy.ts` | `5cbf002ee4ad1c9e2529417ec4b3d054a73e69fb` | Separately gates ACP enablement, dispatch enablement, explicit turns, and normalized allowed-agent identities. |
| `src/acp/server.ts` | `4d63e0c08b9a18eac1825b3ead16cc669e84c8ff` | Owns the NDJSON ACP process boundary, Gateway bootstrap/readiness, session store, ordered transport, and shutdown/database cleanup. |
| `packages/acp-core/src/index.ts` | `91e9a6b6c010ded1356e28e9740a0b876705ea25` | Re-exports ACP session, identity, runtime-error, metadata, and type contracts; it is a package surface, not a transport owner. |

This corrects only the fixed-source existence question. It does not close the
transitive ACP dependency/package closure or establish Fabushi equivalence. In
particular, the fixed `server.ts` makes the missing ownership boundary
explicit: a valid Rust implementation needs a canonical packaged ACP entrypoint
that performs Gateway/session admission, command projection, policy rejection,
ordered request/response transport, and shutdown custody in one existing
production lifecycle. The current `AcpSessionManager` remains an in-memory
plugin-SDK helper consumed by `openclaw_plugin_sdk_contract.rs`; it is not that
entrypoint. `SandAutoReviewDecision`, `TurnSettlement`, and the transcript
watchdog are independently real Host mechanisms, but no production ACP caller
connects them to the fixed policy contract.

Disposition is therefore unchanged and deliberately split from source
presence: `OCRM-RESP-AGENT-ACP-COMMANDS-001` and
`OCRM-RESP-AGENT-ACP-POLICY-001` remain
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`,
`acceptance=pending`. The next direct instrument is an exact ACP dependency
closure plus a producer -> canonical ACP transport/owner -> session/security
state -> client/turn consumer call graph. No ledger flag is promoted by this
recheck.

The fixed-root import census also identifies the immediate ACP closure
frontier without treating package names as source closure:

- `commands.ts` imports the ACP SDK type surface and
  `src/auto-reply/thinking.shared.ts`.
- `policy.ts` imports the OpenClaw config type, session-key normalization, and
  `src/acp/runtime/errors.ts`.
- `server.ts` imports the ACP SDK stream/connection surface, the
  `packages/acp-core/src/{session,types}.ts` runtime contracts, normalization
  helpers, Gateway readiness/client/bootstrap, state-database shutdown, event
  ledger, secret-file, session ordering, translator, and provenance types.
- `packages/acp-core/src/index.ts` is an export barrel whose twelve local
  modules must be recursively qualified; its package manifest and the root
  `pnpm-lock.yaml` are exact inputs but do not by themselves prove published
  package/archive or native/runtime closure.

This is a direct-import frontier record, not an accepted dependency closure.
The four unresolved relative edges and the external SDK/normalization/runtime
packages remain open until a closure instrument records every resolved path,
blob, package identity, generated/native input, and license/notice fact against
the same fixed tree.

## ACP transport-adjacent owner disambiguation — 2026-11-01

The current checkout was also checked for a reusable transport that could be
mistaken for the fixed ACP server. Two protocol surfaces are adjacent but are
not ACP production ownership:

- `source/host/app/src/main.rs` routes ordinary stdin lines through
  `UnifiedAppHost::dispatch_json`; `platform.request` is a desktop platform
  bridge, not an ACP initialize/session/turn protocol.
- `source/host/src/runner/routed_provider_runtime.rs` serves bounded HTTP JSON-RPC
  MCP `initialize`, `tools/list`, and `tools/call` requests for routed plugin
  tools. It has no ACP session creation, available-command projection, ACP
  policy, or Agent Client Protocol response/event contract.

The fixed `src/acp/server.ts` therefore has no existing Rust transport to
reuse by simple symbol matching. This narrows the owner decision to an
explicit adapter at the existing Host/Gateway lifecycle boundary; it does not
authorize copying either platform or MCP transport into a second ACP runtime.

## Compaction replay consumer boundary recheck — 2026-11-01

The exact current Rust production route was re-read after the fixed-source
session projection inspection. `start_routed_provider_task` decodes the
request body's `messages` into `ProviderMessage` values through
`decode_provider_messages` (`source/host/app/src/main.rs:6964-6988`), then
creates the provider prompt from those values at
`source/host/app/src/main.rs:7541-7542`; no non-test caller reads
`ProductionSessionWorkers::read_agent_compacted_transcript_entries` before
the provider turn. This is direct producer/consumer evidence, not an absence
claim based only on a symbol search.

The existing owner replay result cannot be substituted into that route by a
type cast. `SandAgentDb::get_compacted_transcript_entries`
(`source/host/src/extensions/session/agent_db.rs:2077-2117`) returns the
latest `kind: "compaction"` entry followed by rows from `firstKeptEntryId`,
while
the fixed OpenClaw projection turns a compaction row into an
`AgentMessage` with role `compactionSummary`, and `convertToLlm` renders it
as a user-context envelope before sending provider messages. Fabushi's
transcript also contains `send-message`, tool, attachment, and ordinary
`message` rows with distinct ownership and visibility rules. A consumer that
only maps `text`/`content` would therefore lose the fixed summary envelope,
role semantics, or tool-pair ordering.

The next implementation boundary is consequently a typed adapter owned by
the existing Runner: read the canonical transcript under the admitted turn,
decode supported rows into the fixed session-entry model, apply the fixed
cut-point/replay projection, preserve the current request and provider
system/prompt projection, and fail closed on unsupported or malformed rows.
Only that adapter may call the existing compaction owner replay API. Until a
real producer commits such an entry and a newly constructed turn consumes the
adapter output, the compaction responsibility remains
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`,
`acceptance=pending`; no completion flag is promoted.

### Provider contract limitation

The current Rust `ProviderMessage` contract is only `{ role: String,
content: String }` (`source/host/src/extensions/inference/provider_session.rs:89-92`).
The transcript owner and recovery code nevertheless recognize distinct
`message`, `send-message`, and `tool-call` rows, and the fixed OpenClaw
projection preserves tool-result/tool-pair semantics rather than flattening
them to ordinary text. A summary-only adapter could represent the fixed
`compactionSummary` envelope as a user message, but it cannot faithfully
reconstruct a compacted history containing tool calls, tool results, or
attachment-bearing rows through the current provider contract. Extending or
adapting that contract is therefore a prerequisite for a real next-turn
consumer; silently flattening those rows is explicitly non-equivalent.

---

## 3. Falsifiable Test Oracles (`source/host/tests/openclaw_agent_turn_contract.rs`)

1. **`OCRM-AGENT-O01`**: `test_ocrm_agent_o01_active_stream_conflict_rejection`
2. **`OCRM-AGENT-O02`**: `test_ocrm_agent_o02_deferred_supersedes_queue`
3. **`OCRM-AGENT-O03`**: `test_ocrm_agent_o03_immediate_cancellation`
4. **`OCRM-AGENT-O04`**: `test_ocrm_agent_o04_owner_token_monotonic_generation`
5. **`OCRM-AGENT-O05`**: `test_ocrm_agent_o05_viral_turn_tainting`
6. **`OCRM-AGENT-O06`**: `test_ocrm_agent_o06_auto_review_approval_gate`
7. **`OCRM-AGENT-O07`**: `test_ocrm_agent_o07_model_catalog_pricing_calculation`
8. **`OCRM-AGENT-O08`**: `test_ocrm_agent_o08_provider_fallback_chain`
9. **`OCRM-AGENT-O09`**: `test_ocrm_agent_o09_context_window_compaction_trigger`
10. **`OCRM-AGENT-O10`**: `test_ocrm_agent_o10_tool_argument_validation`
11. **`OCRM-AGENT-O11`**: `test_ocrm_agent_o11_runner_settlement_receipt`
12. **`OCRM-AGENT-O12`**: `test_ocrm_agent_o12_zombie_turn_lease_expiration`

---

## 4. Invariant Verification & Global Conservation

- **Exact Mapped Paths**: 112 (cumulative across Units 1–9)
- **Omitted Known**: 0
- **Total Tracked Paths**: 58,750
- **Unread / Unclassified Paths**: 58,638
- **Strict Conservation**: $112 + 58638 = 58750$.
- **All Global Completion Flags**: `false` strictly preserved.
