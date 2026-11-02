# OCRM-P0-CONFIG-001 — Configuration, Schema Normalization & Single-Writer Settings Authority

Status: **Historical scoped mapping retained (6 responsibilities, 8 source reads at the historical source); current fixed-U authority, exact-head verification and acceptance pending**
Unit: `OCRM-UNIT-CONFIG-001`  
Candidate requirements: `OCRM-R04`, `OCRM-R09`, `OCRM-AC05`, `OCRM-AC17`  
Dossier: `projects/openclaw-rust/dossiers/config-settings-scoped.md`  
Historical scoped source oracle (not current authority): `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`
Fixed implementation source: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` / tree `90c8aab39972da11cfad220361781d49e1bc6700`; accepted identity and scoped evidence remain pending. Later discovery candidates do not reopen this fixed subject.
Investigated target: `bhrumom/fabushi-desktop` HEAD

### Active shared-gate recovery — CANCEL-SOURCE-01 (2026-11-02)

Exact b3 Runtime `41411715447` / job `125166533584` is failed: the real held
connector cancellation test did not find its preemption wake marker. Independent
log and producer→classifier→scheduler→cancellation source tracing classified
this as **FIX / IMPL_BUG** before edits; see `CANCEL-06` in
`docs/specs/cron-session-reaper.md`. The production classifier loses the exact
`connector` source while retaining User lane. Repair that projection, retain
the valid assertion, and add real-admission and shipping cancellation negatives.
The worker's secondary disconnected release channel does not establish an
environment fault. Preserve b3's passed build/configuration/app compilation and
other eight shipping cases plus fifteen renderer tests as b3-only evidence;
the shared integration step did not execute.

Concurrent `4fe16f4689e23da2f027d09b2e8d39f56755c6ec` supplied the equivalent
classifier repair and real-admission oracle on ec1 before this control batch
published. Adopt those exact changes and add the shipping producer/precondition
and negative cancellation controls without duplicating the implementation or
admission test. Preserve Host-lock, canonical checkpoint and compaction changes.
C3 H
`d047bfa4d9705f54a5e069e17c53fcc001a475d0` is remotely read back; source run
`41412511336` succeeded, while runtime `41412511366` was cancelled with no jobs
after a concurrent push. Those are not C3 runtime approvals. Freeze the repair
for independent source review and exact-H Actions before advancing dependent
Plugin batches. Fixed U, all false global flags and final acceptance are intact.

Review disposition `CS-ORACLE-QUEUE-01` — **DEFER / TEST_FLAW**: independent
review of the earlier candidate found an inherited 30 ms sleep before asserting
that a second thread enqueued in `routed_send_acceptance_contract`. This batch
adopts 4fe's already-selected `turn_runtime_recovery_contract` source oracle,
so it does not add that unrelated timing-dependent target. Retain its existing
assertions; replace the sleep with causal synchronization before relying on it
for full acceptance. This is an open oracle repair, not a production exemption.

Queue oracle recovery `CS-ORACLE-QUEUE-02` — **FIX / TEST_FLAW**, on base
`c7fc0decf452f7c5a34580479bc05e2fc9d070d1`: replace that scheduling assumption
with the existing `accept_routed_send_with_queue_observers` admission event.
Wait for the actual second queue admission, retain all lease/count/terminal
assertions, and assert its Agent, User lane and active predecessor. No production
API or scheduling policy changes. Select this contract in the existing scoped
Actions and receipt; independent review and exact-H execution remain required.

## Atomic closed behavior

Configuration reads and mutations enter through canonical Fabushi settings API → check immutability / deployment mode guards → acquire serial writer mutex → traverse or update properties along safe dot notation paths with prototype pollution defense → persist atomically via temporary staging files and sync → notify subscribers of effective field changes.

**Historical readiness:** Unit recorded in `parity-ledger.json` (`OCRM-RESP-CONFIG-PATH-MUTATION-001` .. `OCRM-RESP-CONFIG-AGENT-WORKSPACE-DIRS-001`) with the earlier eight source reads. That historical claim does not describe the current fixed-U shards or confer current authority. `source/host/tests/settings_service_contract.rs` contains source-level contract assertions; this is not a current exact-head Actions receipt and does not by itself prove production wiring. Full project closure and release acceptance remain false until every path is dispositioned and every applicable product responsibility is reviewed, implemented, wired and accepted.

## Current production batch — OCRM-CONFIG-CAS-v1 (2026-11-02)

Implement [the atomic Settings contract](../../docs/specs/config-settings-atomic-commit.md)
through the existing SettingsService and authenticated Gateway. The frozen
parent is `3b797be6b65095fd16b0a978e5fcd19a8ab504b3`, with main
`5528062177e3865eef60a073a066cca144f9e3e8` integrated. The current dossier records
new exact-U reads separately from historical counts above. Gateway hash CAS,
commit receipt and notification normalization share the writer transaction;
strict mutation reads must preserve invalid/unreadable prior bytes.

Subject `38cbc11c9fa522e69ff02627b8d0de714b2615ee`, tree
`8618ec4e2bed66385be7f825ab7356c3d09f39cf`, is pushed and read back through
GitHub. Independent source review found no new blocking defect after fixing
notification reentrancy ordering and stale roster receipt application. This
review covered actual Experiments, Wallpaper, diagnostics and roster consumers;
it did not execute tests or establish scoped verification. Actions run
`41381141155` passed 27 review and 10 materialization tests, then rejected an
existing malformed ledger reference before runtime dispatch; the recovery is
recorded in the P0 source task. Runtime verification remains not-run.

Use the existing `ocrm-p0-authority.yml` for the shared
risk contract, conservative materialization and exact-U structural gates; use
`rust-desktop-runtime.yml` with `validation_scope=ocrm-config` for the shipping
Host check and affected Settings/Gateway/inference contracts after freezing H.
An Actions-generated inventory commit creates a new H and requires its own
checks. No local or htch-runtime validation is authorized.

Remaining config equivalence includes cross-process/source-path authority,
HMAC/redaction and key lifecycle, JSON5/replacement/includes, complete model and
plugin validation, and post-publication recovery. Full source closure and all
other product/platform responsibilities remain in scope. GitHub delivery and
exact-H results are required before any verified or accepted status.

Current blockers are `OCRM-CONFIG-CAS-EXACT-H-EVIDENCE` (clear only with the
frozen subject's affected build/contracts and reviewed consumer wiring) and
`OCRM-CONFIG-FULL-SOURCE-AND-PROTOCOL-ACCEPTANCE` (clear only with scoped source
closure/risk review and the remaining configuration responsibilities above).
The first may close independently; it cannot close the second or project flags.

## Exact-H integration recovery — 2026-11-02

Actions materialized conservative review fields at
`2e1bae4bf081de857eea3f79297c271c6c11b3f9` / tree
`ca90c3402b62c5360246a31149beda578417e640`. Its source run `41381873559`
passed tracked-source/structural gates, 27 review tests, 10 materialization
tests and four negative probes; artifact `13113716764` has digest
`sha256:cd74bc1f9b122910703598157e35e7ec7e755397567cf5b6adcfa74f9a012981`.
The artifact's internal receipt binds this H and fixed U; dependency/LFS closure,
final review and independent acceptance remain open. Literal inventory history
is still 62,130 unread/partial rows, with 815 mapped rows; these are accounting
facts, not product completion.

The same H's runtime run `41381875376`, rust-host job `125074236805`, failed
`cargo check --profile ci --bin mahayana-app-host --manifest-path
source/host/app/Cargo.toml` with exit 101 and 14 compiler errors. No Settings
contract target ran. Classification: implementation integration defects in
existing Session/Transcript/Runner code, not a runner/environment failure.

Recovery contract `OCRM-CONFIG-INTEGRATION-RECOVERY-v1`:

- Preserve Session admission, canonical replay, automation settlement and
  pending-wake ownership. Fix duplicate imports, owned/borrowed identity use,
  exact collection/sum types and existing error-type adaptation at their
  current boundary; errors remain errors and admission runs before mutations.
- A raw turn request starts with no trusted typed replay. Only the existing
  Session owner read in `start_routed_provider_task` supplies that projection;
  caller JSON must not manufacture it. Retain CC-07 typed attachment/tool
  semantics and the source/consumer checks in the compaction spec.
- Pending-wake payloads contain an existing `f64` timestamp. Their wrappers
  may retain `PartialEq` but cannot promise Rust `Eq`; do not change persisted
  timestamp representation, drop markers or fake equality to satisfy a derive.
- Keep all existing production paths and assertions. Recheck shipping Host,
  the seven Settings batch targets, and the affected Session, automation,
  shared-room, pending-wake and turn-projection contracts in Actions. Retain
  the source run above as its own subject evidence; new code requires a new H.

The model-selection batch remains preparatory until this shared compile
boundary is repaired. No verified/accepted state is raised by this recovery.

Static review also found one `TEST_FLAW`: the typed-replay content-fallback
fixture supplied a lone tool result despite CC-07's required call/result
pairing. Add its matching assistant call and keep both original content/result
assertions and the production pairing guard. Independent review found no
existing direct unpaired-result rejection test, so add that negative oracle
in the same Session test module. This is a fixture correction identified
from source, not a claim that
the skipped test ran. The affected lib filters name existing test modules;
their actual nonzero test counts must be checked in the next Actions log.

### H4 shipping app recovery and independent exhaustion assessment

H4 `ad60d4edb86e385243a67cb12e39187a073c7c01`, tree
`0ba7a184ddf4d5ad26bdf664e6d4f1ac8e98fbc0`, is pushed and GitHub commit/ref,
production file and workflow contents have been read back. Source run
`41383067764` passed; artifact `13114211097`, digest
`sha256:b0c569d4f92be692cc13550eecaf4a3a008aa42c0b2f3307ba82ff862e20dbdf`,
contains the exact H4/U structural receipt. Its dependency/LFS closure and
independent acceptance remain open.

Runtime run `41383249411`, job `125078414436`, reached the shipping
`mahayana-app-host-runtime` after the previous 14 library errors were removed.
It failed with seven app compiler errors (exit 101): two malformed dispatch
error expressions, a u64/f64 wake timestamp boundary, two missing facade
summary calls, a Result return with an extra `?`, and an immutable JSON entry
that needs its admitted session key. Settings and recovery tests were skipped.

The normal plus focused campaign cap triggered an independent directness /
exhaustion assessment before further dispatch. Classification: recoverable
`IMPL_BUG`, not environment or test flaw. The failed assumption was that the
library repair also established the previously unreachable app boundary.
Retain H4's source proof and library compile progress, but neither proves
shipping compilation or behavior. Do not rerun H4, lower the target to a lib,
remove assertions, reset the budget merely by changing SHA, or claim a pass.
The causal next instrument is the same shipping-bin check on repaired source,
followed by the affected checks that have never run. Independent review also
identified an adjacent partial-move risk in the automation dispatcher; clone
its dependency bundle for dispatch so settlement retains the same Arc owners.

Recovery preserves each error as an error: upgrade resume returns Result so
its caller clears a marker only on success; automation dispatch failure does
not enter its terminal wait loop; doctor serialization returns its mapped
Internal error. Convert the existing millisecond timestamp at the existing
f64 persistence boundary without changing its format. Add only a thin
`SandAgentRunner::run_summarization_prompt` forwarder to its existing
`ProductionTurnAgentOwner`; retain unbound Configuration, cancellation/error
identity and no extra turn settlement. The existing owner contract gains a
facade oracle, and Actions also selects existing upgrade, automation wiring
and cloud-worker contracts. No production model-identity implementation is
included until this shared boundary is repaired.

H5 `5dae743427127f3b154f2b86bfd89b2850b1fa4c` / tree
`3b54e5d8e3fb7ecddbc140f4862d11cce14006d7` is also pushed/read back. Source
run `41383826415` passed with exact-subject artifact `13114153766`, digest
`sha256:bfc11b6103dc7bf2bcf54d92564d5a5cdecb32851ab6a93cb8b1c77451b9b175`;
its structural-only scope and open flags are unchanged. Runtime `41384019149`,
job `125080763324`, passed the prior seven app error sites but failed with one
newly exposed E0382: `ProductionSendMessageSink` consumed `worker_sessions`
before the generated-image persistence consumer cloned it. Exit 101; all
behavior targets still skipped. Keep the same Session Arc for both consumers
by cloning the handle at the sink boundary. This is a concrete ownership
repair, not a retry of H5 or a budget reset; independent focused assessment
checks both consumers before another exact-H shipping check. No gate or
assertion is removed, and no earlier runtime result is relabeled as passing.

### H6 contract-oracle recovery (task d56b3d1d, round 1)

Recovery starts from the existing branch at
`c43b7b2cb1a9cf1b3cb5b511974b3f0bccdadcd9`, tree
`9a3e875539a0f0550c3f14e82d7d0cf91a369646`; live main remains
`5528062177e3865eef60a073a066cca144f9e3e8` and fixed U is unchanged.
Runtime run `41384534729`, job `125082349656`, passed the shipping Host
check. The first contract command compiled successfully: Gateway 2/2,
Inference composition 6/6 and roster 10/10 passed; sessions/config passed
36/38 and exited 101. The remaining selected targets did not run.

Independent read-only directness/exhaustion assessment classifies both failures
as `TEST_FLAW`, not implementation or environment failures. Decisions before
repair: `FIX` the obsolete global guard count (eight versus ten actual guards
in seven existing transaction methods), retain the exact count and add named
method coverage so a duplicate guard cannot hide a missing edge. `FIX` the
first correlation fixture: it registered a nested thread but supplied no parent,
contrary to SCP-09. Use a root-scoped first case and preserve readiness,
claim/release/retry/completion assertions; add explicit missing/wrong-parent
negatives to the separate nested-thread case. Do not relax production
correlation or identity checks, change timeouts, or delete valid assertions.

Selected effort is xhigh for source diagnosis and independent assessment;
the native worker applied that effort and retained the configured model.
The causal next instrument is the same existing Actions scope on repaired H,
with `--no-fail-fast` for each contract target group to expose all selected
binary results in that group. Earlier positives remain evidence for H6 only.
This repair does not add production model behavior or confer scoped source
authority, verified/accepted state, or any project completion flag.

The same independent assessment inspected the selected but unexecuted
`agent_db_owner_contract` target. `FIX` its positive fixtures to bind
`session_id` to the existing metadata `agentId`, keep synthetic media entries'
session provenance, and supply the current lifecycle when testing the pending
media fence. Preserve the separate rebound/stale/deletion/content-conflict
negative controls and assert the typed admission failure and unchanged state.
`FIX` the test-only moved `CompactionCommit` by cloning for the missing-cut
case, retaining the subsequent empty/oversized-summary negatives. These are
source-identified fixture defects, not additional observed CI failures.

Independent review of the frozen recovery diff found one further fixture
precondition: the missing-cut compaction case inherited the pre-compaction
tail, so the existing stale-tail fence would run before the missing-cut check.
`FIX` its expected tail to the preceding successful `compaction-1` entry while
retaining the typed `CompactionInvalid` assertion. No production fence changes.

### H7 recovery settlement (task d56b3d1d, round 2)

The exact starting H is `5f0d548c325998ce4dcbedd1f1f04f91489df323`;
live main remains `5528062177e3865eef60a073a066cca144f9e3e8`. Fixed U is
`1440bb1771372334bea23d7642bb52deb7db1845`, with no source-lock change.
Actions source run `41397585766` passed its 27 source-review checks and
62,130 tracked rows. Artifact `13120510178` has digest
`sha256:130062d75840a6033ad070f2551c5a634900a367fb14bf84a3cb641e9d6d36b4`.
This is structural source evidence, not recursive dependency, semantic,
compatibility or final acceptance closure. No inventory delta was produced.

Runtime `41397585774`, job `125122153232`, passed the shipping Host compile
and the nine configuration/inference targets (95 tests). The recovery target
group failed 13 tests across seven binaries; later library and integrity
checks were skipped. Logs identify implementation and oracle defects, with
no evidence of an environment failure. Previous positives remain bound to
that H and do not certify this repair.

Failure dispositions and retained behavior:

- `IMPL_BUG`: owner reads discarded durable background-result/media receipts
  and compaction records; decode the three explicit control kinds for their
  existing owners, retain the presentation parser, and reject malformed or
  rebound receipt identities. Reopen/replay, malformed payload rollback and
  compaction-cut negatives accompany the correction. Null compaction cuts
  retain messages appended after the summary.
- `IMPL_BUG`: an admitted no-op transcript deletion reported owner refusal;
  acknowledge the committed operation while preserving busy/closed/fenced
  non-success, no extra write generation and no duplicate mutation event.
- `IMPL_BUG`: Human handoff checked the source Agent key in the destination
  Human DB. Hold the existing source lifecycle boundary through destination
  append/enrichment and validate exact source admission. Preserve the Human
  owner, no Agent admission in its DB, stream/content idempotency and legacy
  unbound calls. Test mismatched, absent, archived and both fast/durable
  deletion fences with unchanged destination rows and metadata.
- `IMPL_BUG`: repeated cloud-watch rearm cleared its sole durable marker.
  Keep custody until matching completion; rearm must not resurrect a marker
  removed concurrently by settlement. Test repeated rearm, reopen, failed
  arming, same-key replacement and completion during registration.
- `TEST_FLAW`: use the production send-message envelope in media fixtures;
  require lifecycle revisions in generated-subagent positive fixtures;
  expect the established early Session facade refusal and prove no write;
  inspect the admitted multitask facade rather than a prohibited raw DB
  handle; update the cloud wiring oracle to the current typed-turn API.
  The fake cloud watch now models its actual pending-marker producer.
  Valid atomicity, replay, isolation and rejection assertions remain.

Diagnosis and the three disjoint implementation workers ran at actual
`xhigh` effort with the configured model. The next causal instrument is the
existing exact-H shipping check and the complete affected contract group,
including DB serde/pages/recovery and cloud-wake production wiring. No local
build, test, formatter, generator, schema check or runtime was executed.
Source persistence dispatches another runtime run only when it actually
materializes a new commit; the ordinary push already checks an unchanged H.

Open production gaps remain explicit: compaction currently captures the
Session admission lifecycle revision but compares/advances metadata revision;
these can diverge and require the existing owner's reconciliation. The
watch port can also report failed arming when a watch completed before its
armed-state recheck, although settled custody must stay absent. This batch
does not establish real provider/channel/plugin/node/device compatibility,
cross-process Runner recovery, scoped semantic authority or any global
completion flag. Provider-bound model identity remains the next production
batch after shared contracts pass. Independent frozen-scope assessment and
new exact-H Actions evidence are still required before this batch is called
verified.

Independent read-only assessment of frozen recovery tree
`d117615b7f9fb8a84a6dae68f09d125c26c64cce` (base H above; 17 changed paths)
identified two concrete integration defects. Decisions before editing:

- `FIX` R1: a cloud completion after the durable snapshot but before the first
  armed check still reached the fresh-watch unconditional pending producer.
  Admit a recovery watch by atomically replacing only its exact observed
  marker through the existing pending store; absent/replaced markers must not
  arm a watch, publish pending state, or start another completion worker.
  Retain normal new-work admission and add the missing deterministic race.
- `FIX` R2: exposing internal control rows to full owner reads changed the
  length/offset assumptions in `session_maintenance::backfill_transcript`.
  Compare only the existing presentation projection when filling a missing
  checkpoint tail, preserving all durable receipts and compaction records.
  Add a real-owner recovery case containing an interleaved control row.
- `FIX` R3: making compaction records readable activates the strict typed
  consumer, which did not represent canonical outgoing text SendMessage rows.
  Preserve those messages as assistant replay, and validate the retained
  projection before committing a new boundary. Keep unpaired/unsupported
  rows as explicit failures rather than dropping or stringifying them; an
  invalid candidate must leave prior transcript and metadata unchanged.

Read-only preparation also found two pre-existing `CodexDirectCheckpoint`
literals in cfg(test) library code without the already-added `model` field
(`production_turn_run_shell_adapter`, `turn_shape`). `FIX` as fixture schema
drift before the previously skipped library compilation; preserve their
watchdog/tool-boundary assertions and provide explicit model identity. This
is source evidence, not an observed compiler result.
The newly selected maintenance target also contained obsolete calls without
its required `SandAgentDb` owner; update those fixtures to use the same real
owner while retaining their recovery, prefix, cleanup and idempotency oracles.

The independent assessment completed the full frozen scope and adjacent
callers at actual xhigh effort. It confirms the environment reached the
shipping and contract subjects; this is not an environment blocker or a
campaign-budget reset. The follow-up is justified by the concrete changed
invariants above, not by a new SHA alone. Do not repeat unchanged failed
fixtures, restore receipt-dropping, weaken typed replay or remove durable
custody to obtain green. The existing integrated target command remains the
practical shared-owner instrument; it now also selects watch-owner, pending
store, manager, maintenance and accepted-checkpoint/turn-shape consumers whose
contracts were changed or previously skipped. Source authority and full
compatibility/final acceptance remain separate unmet claims.

Focused immutable review of tree `22546cb46ece9cf9ec6409502cb2addc822aefff`
found R4, a test fixture requesting `Deserialize` for the intentionally
Serialize-only quiet-origin type. `FIX` before compiling: use the existing
`coerce_quiet_origin` at the fixture's Value boundary, matching production;
do not widen the durable/public schema or alter the concurrency assertions.
That reviewer completed all 29 frozen paths and the 21-path fix delta, with
no additional blocking findings. The R4 correction requires a fresh focused
snapshot review; no executable result is inferred from this source verdict.

### H7 exact-commit result and newly reached planning recovery

Commit `49200bf2577c9d83aee1f02eef2c65150ca2c2d8`, tree
`97d939de1c931623cac5703b0f727bde311c18b9`, was pushed and read back through
the GitHub API. Independent xhigh review converged on that tree; the resumed
focused review confirmed R4 and the immutable v3 diff without another full
review. Source run `41400613798` passed; artifact `13120274361` has digest
`sha256:3e774f6257fe48431df5e35f4b5485d6fd607d053565cb9dfa2927513798c65d`.
Runtime `41400613869`, job `125131426581`, passed shipping Host compilation,
95 configuration/inference tests, all 22 integration targets (191 tests),
and both typed replay/serialization library groups (8 tests each). The prior
13 failures are resolved at this H. The newly reached planning library group
passed 11/14; later library groups and runtime integrity were not executed.

Decisions before the focused repair, based on the actual logs and exact-U
source: `FIX/IMPL_BUG` latest-user selection scans only the discarded prefix;
retain the latest-user assertion and include the retained tail. `FIX/TEST_FLAW`
the split-turn fixture ends at a user turn start; supply an assistant suffix
and retain its prefix/history assertions plus a non-split control.
`FIX/TEST_FLAW` a 10-token positive fitting fixture cannot contain its required
details; retain it as a refusal negative and use a sufficient bounded budget
for the truncation positive. `FIX/IMPL_BUG` the reverse walk checks its token
threshold before recording the crossing row's cut point, unlike fixed-U
`compaction-cut-point.ts`; add a direct threshold/pairing counterexample.
There is no environment-failure evidence. Run all selected library groups even
when one fails, preserving a nonzero aggregate exit; no gate or assertion is
removed. The shared gate must pass before dependent model integration. Wider
source/compatibility/restart/final acceptance and all global flags remain open.

Frozen planner review of tree `19f75ef0fd62f177c123010317e165a4e41cf680`
against H7 returned one P2 finding. Decision `FIX/IMPL_BUG`: the newly selected
threshold row can move the cut into an earlier overlapping tool span because
the resolver compares against its original index. Resolve the connected spans
transitively and retain both the single-span oracle and the new A/B overlap
counterexample, including the real typed replay/serializer consumer.
Decision `DEFER` for the separate, pre-existing `main.rs` fallback fitting
bypass: it is outside this four-path planner recovery and will be repaired in
the next compaction production-boundary batch. The helper refusal test does
not establish end-to-end refusal. The independent review found no other
blocking issue in this scope; the fix requires a fresh focused review.

### H8 shared gate and provider-bound model implementation

Before publishing the local planner tree `44a346f6bc51e8603ef1148d9f0476d571abbe4e`,
the branch lease check found concurrent successor
`126dea3a7f077677e86f2e0fbfe73b660e11bd60` / tree
`17568a0110ea7201c8fb202e093ce7ba73769dd3`. A fresh independent xhigh
reconciliation inspected all four changed paths and adjacent consumers, found
the production corrections equivalent and the strict replay/serializer oracles
intact, and approved adoption. The unpublished snapshot was retained; no
remote change was overwritten and no duplicate Actions run was started.

At H8, runtime `41402597254`, job `125137462760`, completed successfully:
shipping Host compilation; 95 configuration/inference tests; 191 recovery
integration tests; and 50 tests across the seven selected library filters,
including all 19 compaction planner tests. Audit integrity passed. Runtime
artifact `13122263293` has digest
`sha256:1b80c1f4da117088dbfd1a3bef70103595550e8995cfb543f76055107c6e84b1`.
Source run `41402597240` passed, with artifact `13122386669`, digest
`sha256:9b4700ab99622d4f4d5f5cda2375170c55309f12afd7a0f199a2b1a35ad7f2b7`.
These are H8 results, not evidence for the following working delta.

After those shared checks passed, implementation began against H8 and
`OCRM-MODEL-IDENTITY-v1`. Settings owns checked default reads and atomic
provider-bound writes; Inference resolves identity; the existing Host/Runner
composition carries the frozen value; the Fabushi adapter validates accepted
checkpoint identity before transport creation. Main admission now guards the
accepted lease before session/model reading and permission direction changes.
Its configuration/session refusals and later valid admission are exercised
through the actual shipping helper. Cursor web inputs remain separate, and
summarization resolves the Fabushi default through the same inference owner.

Disjoint Settings, provider and integrated-contract workers used actual xhigh
effort; the integrator alone changed shared composition, the entrypoint,
workflow and this task. The new loopback contract observes five real Runner
requests `[A,A,A,A,B]`, then a separate summarization request using B, plus
accepted checkpoint JSON and two native message deliveries. This is authored
coverage pending frozen-H Actions, not an executed result or real-service claim.
The selected workflow also compiles and runs the shipping model-admission test;
four missing imports in its pre-existing cfg(test) module were repaired from
source inspection without dropping their assertions. No local executable
validation was run. U, all source/coverage acceptance states and the eight
global completion flags remain unchanged. Production disk-load/resume ingress,
real ecosystem compatibility and full G01–G12 / AC01–AC24 remain open.

Frozen model review tree `0612b281dcce3f3e724312f01a2c64cf315be6f5`
received independent general, recovery/security and oracle inspection at xhigh.
Decision `M1 FIX / TEST_FLAW`: the new inference `save_default` and shipping
admission fixture omit the existing required `parameters` array. Bound writes
therefore refuse before the intended oracle; the legacy write returns
`Ok(false)`. Add `parameters: []` to those valid fixtures and assert the setup
write changed the document. Keep the production validator and every behavior
assertion. This is a source-proven fixture defect, not an executed CI result.
The complete frozen scope stays 17 paths; the fix requires a fresh immutable
focused review before publication and exact-H Actions.

`M2 NON_BLOCKING / DEFER`: the new account-clearing test's `allow` permission
spelling already normalizes to `ask`, so that assertion does not independently
prove permission clearing. Existing coverage uses `SandLocalToolPermission::Always`
and checks clearing; this batch's model clearing/replacement assertions remain
discriminating. No standalone fix round or permission-behavior claim follows.

### H9 model evidence and app-test recovery

H9 `76095c27539c2be57e3c1ba0e6b33b8158d92766`, tree
`a5945fc67abac4c1a6da411503c7e45411a88371`, was pushed and all 17 changed
files were read back by full content and blob identity through GitHub. Fresh
focused xhigh review converged on the fixture fix and complete frozen scope.
Source run `41404775846` passed (artifact `13123176112`, digest
`sha256:91a5edde3ab2235c95fff4754f93f379306bf3d4d6bf50d525fb358e5bef70b8`).
Runtime `41404775725`, job `125144212143`, passed shipping compilation and
114 tests across the 11 Settings/inference/Runner integration targets. The
loopback child reports the same test a second time; it is not counted twice.
The app test build failed with E0432 at the obsolete
`super::listener_connect_resume_args` import. The model admission test and
later recovery/library/audit steps therefore did not execute on H9.

Decision before recovery: `M3 FIX / TEST_FLAW`. The import and its test predate
H9, but the newly selected app test exposed them. Current listener resume
calls `listener_connect_resume_prompt` → `resume_with_hidden_handoff` →
`run_local_background_revival_turn_with_context`; its admission JSON is inline
in the last function. Extract that exact value producer without changing its
fields or nonce generation, and bind the old test to it. Retain agent/nonce,
hidden/no-user-append, request-source and Slack invitation assertions; use the
current invitation wording. Execute both the model admission and repaired
listener contract in Actions. This is a test/production-seam correction, not
an environment fault or evidence to reintroduce a retired listener helper.
Preserve H9 positives as H9 evidence; the repair needs fresh immutable focused
review and its own exact-H checks. No migration flags or acceptance advance.

### Compaction continuation and concurrent model reconciliation

The unpublished combined model/compaction candidate was frozen against H8 as
tree `0e45f315d20d747f46b92d3653dca94f10ad9015`, full diff SHA-256
`94e0bd2c325998b72652636c40da4087f70978d853cc5b1a1c25cfa8ac3412a0`.
The publication lease found the concurrent model successor H9 and its app-test
repair `81d48636b06d20f8bec551cdc334fe40f31e614b`, tree
`13b8852dbbf52c4e26af1fc7ab73898eb8c45b13`. Both exact commit objects and all
17 changed file contents/blob identities were read back and imported without
overwriting the branch. The unpublished candidate remains preserved; the
continuation adopts the remote model implementation and its tests. Independent
xhigh source reconciliation checks the model invariants and retained controls.

This delta implements `CC-05` in the existing Session owner: admission revision
is the canonical generation when present; busy work prevents rotation; entry,
metadata and admission CAS commit together; stale/idempotent paths do not
publish. Production facade tests cover repeat compaction, reopened replay,
settleable cron custody, raw CAS loss and archive/restore fencing. The shipping
summary/fallback must fit before commit. Media/tool replay, full compaction
semantics and actual provider acceptance remain open.

Pre-freeze decision `C1 FIX / IMPL_BUG`: real cron custody exposed missing SQL
token separators in the existing begin, finish, finish-with-fences, publish and
list statements. Repair whitespace only, preserving all predicates; exercise
actual begin/finish/list/publication and stale/terminal negatives.

Review decision `C2 FIX / IMPL_BUG`: synchronous compaction admission listeners
could read the same owner while its connection mutex remained held after
COMMIT. Release that guard before all publication. The bounded regression
requires both callback and commit to return, checks the observed committed
generation, and retains raw rollback and no-publication controls. Fresh xhigh
focused review approved the three-path fix delta
`71497baa3cf87b3f9bc4c998a730c57475d28ebf8e0c28237db0863eb655b471`;
that source approval is not execution evidence for the reconciled successor.

Decision `C3 DEFER / NON_BLOCKING`: the unchanged split-prefix cancellation
fallback remains a Runner recovery follow-up; it must recheck cancellation at
commit and prove no durable mutation. Fitter refusal alone does not prove it.

2026-11-02 follow-up selection: C3 is now the distinct production contract
`OCRM-COMPACTION-CANCEL-v1`, after preserving the earlier fit/custody batch.
The prior review's deferred scope decision is retained, not rewritten as a
failed old check. **FIX / IMPL_BUG** for this new batch: source tracing proves
a cancelled prefix error can select a fitting fallback, and pre-lock checks
cannot cover cancellation while waiting for canonical Session/DB ownership.
The integrator will add guarded compatibility delegates and a distinct
cancelled outcome before wiring the real worker token; exact raw rollback and
actual shipping prefix controls are required. Base is H740, fixed U unchanged.
The Session/DB shared API, locks and main composition remain integrator-owned;
an independent xhigh oracle worker may edit only the two existing DB/Session
contract test files. No local executable validation, global promotion or
process-restart recovery claim is authorized by this source finding.

The authored C3 implementation now passes the actual worker token through the
existing Session/DB owners and checks it after each canonical mutex and before
COMMIT. Split-prefix Ok and Err are both refused after cancellation; ordinary
prefix failure still selects a fitting fallback. The two DB cases compare raw
rows/all KV, admission, generation and three publication channels, then prove
the identical candidate commits and retries idempotently. A Session case
launches the public call while holding the real filesystem mutex and cancels
before release; its handshake does not claim kernel waiter introspection.
Two shipping app cases use the real planning, prefix-selection, fitter and
Session commit boundaries, including a fitting fallback and positive commit.
These five new tests are authored, not executed. Incoming H881's checked-root
owner changes were preserved during serial reconciliation; C3 remains a
separate candidate from the Plugin snapshot batch.

C3 freezes on parent `41e845a3f246a1420796bbac0a323d335f6c7f36`, whose complete
22-file Plugin candidate and commit/tree/parent identities were read back
through GitHub. H881 Runtime `41409355083` and source `41409355085` passed;
its log directly includes the two shipping channel handoff controls and the
checked checkpoint-root recovery cases. Its runtime artifact `13124992828`
has digest `sha256:74ecd5c5b8aea7c0801588d1c159105c17d78183465f7a88b672b1450e6d9492`.
That parent evidence is not relabeled as C3 or Plugin execution. C3's frozen
eight-path scope requires independent general and cancellation/oracle reviews
and its own exact-H Actions. No local executable checks ran.

The source Actions for Plugin H41 (`41411027949`) passed and generated
`58f1fb359c71e203cb82a6fe43cb52336f4542e5`, tree
`c6c28cb07c107dd340de117c6d0724e67156063e`. Its only two changed files are the
part-0049 gzip and its compressed manifest digest: decompressed bytes and the
canonical inventory hash are identical; the gzip OS header changes from 03 to
ff under the Actions generator. Preserve that generated successor as C3's
integration parent. H41 Runtime `41411028013` was cancelled while queued and
is not positive evidence; the generated H58 requires its own exact-H checks.
The frozen C3 production/test/workflow blobs are unchanged by this rebase.

Reconciliation decision `C4 PRESERVE`: keep the earlier public Inference/Settings
consumer controls, post-call cancellation priority, pre-503 checkpoint durability,
checkpoint bytes unchanged after a later model-B turn, and absence of inherited
tool results or a new tool checkpoint on that turn. Extend the remote fixtures
instead of adding a second loopback harness. No production model implementation
is replaced. Preserve generic Codex request-model/legacy-serialization/reasoning
controls and the unrelated Settings writer's refusal to silently repair a bad
model binding. Independent reconciliation approved adoption of the remote
production implementation, with these complementary controls retained. The
reconciled snapshot requires fresh source review and exact-H Actions; fixed U
and all global completion flags remain unchanged.

C3 integration additionally preserves concurrent
`b3a3fd398d3057b264ac673fae1b352aaecf1c92`: cancellation requests remain
acknowledgments, and only the Runner finalizer publishes its frozen terminal.
The combined workflow retains every terminal/compaction app selector and the
three additional terminal/telemetry integration targets. Both changes still
require the integrated candidate’s own Actions evidence.

The final C3 parent also preserves normalized checkpoint-root CAS and the two
separate storage/reopen contract targets from
`3ce1cd31898d090f38ba3a8c5b895da5b2de2892`; no original-turn provider replay
or accepted restart result is inferred from that incoming implementation.

### Reconciled compaction publication and canonical-root prerequisite

H `3aa679586d76e446f76a1c94700ff7abab623d42`, tree
`96e5cab0322a8ef746ef0da87c1353344dcaf1eb`, was pushed on the existing branch.
The exact commit/ref and all fourteen file contents/blob identities were read
back through GitHub; the large ledger used the Git blob endpoint because the
Contents endpoint returned an empty payload. Fresh xhigh complete-scope
review approved the frozen diff
`505afd7e393a23f54c40ae07eca17cace49c1348d15a8364cfa10f4ef9459d7a`.
Source `41407135213` and runtime `41407135148` are the new-H runs; their terminal
evidence remains pending at this checkpoint.

Parent `81d48636b06d20f8bec551cdc334fe40f31e614b` passed runtime `41405911556`,
job `125147756907`: shipping compilation, 114 configuration/inference/Runner
tests, two shipping app tests, 191 recovery integration tests, 51 library tests
and audit integrity. The 358 unique Rust tests do not double-count the loopback
child. Artifact `13124026042` digest is
`sha256:01d9ef2b9c03b02e6d02d3ac82dd561385001c1d2d08f494c040b29128743237`.
Source `41405911644` passed; artifact `13123217838` digest is
`sha256:d47e122c8c9390df2c268b3745c9d535900e6e0a5d6a43bc7786689513c9058c`.
These remain parent-H evidence, not approval of the next changes.

Next admitted contract is `session-work-admission.md` CKR-v1: strict canonical
checkpoint readiness through the existing Session/AgentStore/Runner owners.
Read-only investigation at H81 found ignored reset failure, five canonical
empty fallbacks, metadata seeding/coercion, unsafe non-ASCII hex slicing and
candidate repair without content-hash validation. The same production paths
are unchanged at this base. Required APIs and cache/reentrancy semantics are
frozen in CKR-01..07 before implementation. Store/maintenance implementation
and independent actual-owner tests use disjoint paths; root owns DB contract,
Session/composition, workflow and ledger integration. All run at inherited
xhigh due to persistence, reentrancy and failure ownership. Generic lenient
hydration, subagent fallback and direct DB short-root fixtures remain valid;
only fabricated positive canonical-read fixtures change to real hashes. This
is a recovery prerequisite, not provider checkpoint loading or full restart.

CKR integration decision `T1 FIX / TEST_FLAW`: the shipping wiring contract
asserts the spelling `latest_checkpoint_bytes()` for watermark preparation.
That assertion enforces the retired optional API rather than the retained
owner/watermark behavior. Replace only that API assertion with the checked
snapshot call, preserving its permission, blob decoding, confirmed-user and
session-admitted write assertions. Actual corruption/refusal contracts cover
the new behavior. Select AgentStore, checkpoint sink, Session preparation,
conversation state, canonical recovery and lenient transcript-provider targets
in the existing scoped Actions group alongside its current controls.

The canonical snapshot validates captured root bytes and current direct
references; checked reset/reopen reloads the root blob. Discovery of later
out-of-band removal of an already cached root blob remains a reset/reopen
boundary. `materialize_new_session` only opens the DB owner, so the existing
conversation-state fixture's direct root CAS does not leave a cached empty
AgentStore; its actual hashed-child fixture needs no oracle change.

Before CKR freeze, the remote branch advanced from `3aa679586d76e446f76a1c94700ff7abab623d42`
to `d94aeb372c2fe4483bcf60ebbc5aaae4e6db52ef`, tree
`1a0359454e5a82c945732ce51a3eb6da3584e339`. Its twelve Automation-history and
Spend Guard paths were fetched completely and checked against their Git blob
identities. Preserve that exact parent. The Session callback addition merges
without overlapping CKR functions; workflow reconciliation retains both
owners' target lists and evidence fields. A separate xhigh read-only reviewer
examines the complete incoming delta. CKR still requires an independent frozen
scope review and its successor exact-H Actions before verification. The OCRM
ledger/RTM receives only contract/test anchors, with every prior read fact and
completion state retained.

Incoming integration decisions, before fixes: `A1 FIX / IMPL_BUG` accepts the
independent review's bound Spend Guard collision finding. The new Runtime
refuses a zero-row card append, but the DB helper still commits `nudgedAt`,
card IDs and unread activity; the next evaluation consumes a card that was
never issued. Preserve the collision negative and extend its unchanged-state
assertions to bound sessions. Roll back the existing DB transaction on zero
insertion and publish nothing. `A2 FIX / IMPL_BUG` addresses the same helper's
COMMIT-to-notification boundary: it retains the connection mutex while invoking
synchronous metadata listeners, so a listener reading that owner blocks the
real Runtime. Release the committed DB guard before publication and add a
bounded actual-Runtime callback-read control. These are ACK-06 prerequisites,
not a new automation store or a cross-store settlement guarantee. The focused
follow-up must include the final CKR scope and the added ACK contract/test paths.

H3aa terminal readback now passes both attempt-1 workflows. Runtime
`41407135148` / job `125151615284` passed shipping compilation, 119 config/
inference tests, two app tests, 224 integration tests and 51 library tests
(396 unique Rust tests, excluding the repeated loopback child result), plus
audit integrity. Artifact `13124143339` digest:
`sha256:b85a276b8294768e33f4023fd03886604c268ad89f2ade3282d44eea1e734135`.
Source `41407135213` passed 27 risk and 10 inventory tests; artifact
`13123908094` digest:
`sha256:615627895bbd2cbbcb8d65ff57fc075961173f2b532a9b5ce479ee69c1ae7dd7`.
The 62,130 source paths, 815 mapped paths and 62,130 literal unread/partial
count remain honest and do not close recursive dependency/LFS/semantic gates.
Neither result approves d94 or CKR. A2 proves the public same-DB-owner callback
contract; no default shipping callback registration is claimed to deadlock.

The independent complete-scope CKR v1 review approved tree
`b260748ffdee9167390244db37449b06eedc3e5f` (16 paths; patch SHA-256
`a8a194350f092e878b3eecbd80dfa01f86f094aa32d9425f4a9a671de6f2b7d1`).
A1/A2 extend that scope with the ACK contract and existing spend-guard target;
their changed transaction/callback paths require fresh focused review before
publication. None of these source approvals is an execution or acceptance flag.

The fresh focused A1/A2 review approved full tree
`a5d30669dd27f7d586263ad1bfb28a79b7aa2355` (18 paths; fix delta SHA-256
`9b92a0773e46982563e2b2bba0079d2b21b1b163f9973494f9a79a41839862d9`).
The remote then advanced to `74073f19037a6eaec5f34331b27dfe9fd8c0b334`, tree
`156ae25b8c79229a83971b4a81598a026e8ad3d8`, adding the existing inbound
queue owner's atomic idle handoff. Its seven full files/blob identities were
read back and preserved. The reconciled workflow retains its two new exact app
controls and queue target alongside all CKR, Automation and prior gates.
Only this incoming delta and reconciliation invalidate the earlier source
review scope; unchanged CKR/A1/A2 source approvals carry forward. Exact-H
execution remains required for the combined successor.

### CKR / SWA-ROOT reconciliation at 88143ef — review decisions

The first publication lease advanced from `74073f19037a6eaec5f34331b27dfe9fd8c0b334`
to `88143ef99a21e87d2da0a31cc509aff3e5176552`; unpublished commit object
`1a162b9658448dbd71b7ddbd7b63a8ac9134ce0f` was not branch delivery. Preserve
the incoming implementation, tests and evidence targets; integrate on that head.

- R-LEGACY — `FIX` / `TEST_FLAW`: the draft CKR length/hash negatives and its
  two positive-fixture substitutions assumed all retained IDs were SHA-256.
  `agent-store.ts::fixedRootBlobId`, `reference.ts::LazyReference`, the actual
  generated stream's direct-reference validation and the existing maintenance
  positive fixture prove arbitrary nonempty fixed IDs remain legal, including
  32-byte IDs. Preserve those positives; replace only invalid new ID/hash
  assumptions with malformed/empty/missing/read-error or same-ID drift controls.
  New writers still prove their SHA-256 outputs. No data is rekeyed.
- R-REENTRY — `FIX` / `IMPL_BUG`: carry incoming root-byte revalidation into
  CKR's authority/revision state machine, before pending recovery and sink
  side effects. A matching metadata ID alone cannot prove durable bytes remain.
- R-API — `FIX` / integration: preserve incoming boolean checked-reset and
  poison-aware cache projection APIs. Production uses the unified fallible
  root/bytes snapshot; known pre-write refusal remains retryable after durable
  repair, while an uncertain commit or failed reset requires checked reload.

These decisions precede code changes. Prior full CKR and ACK reviews remain
context; the reconciled delta requires fresh independent review and exact-H
Actions. Fixed U, historical source facts and global acceptance flags remain.

- R-HEX — `FIX` / `TEST_FLAW`: incoming's newly added whitespace-only-root
  rejection contradicts both existing Rust `decode_hex` and retained
  `serde.ts::fromHex` normalization. Keep normalized-empty reads and add a
  positive read plus raw-string-CAS nonwrite control; all other malformed
  metadata negatives remain. This changes no CAS or persisted data format.

### Frozen d58730d review decisions

- CKR-F1 — `FIX` / `IMPL_BUG` (P2): a cached fixed root can drift to other
  valid bytes while a sealed profile journal remains. Preparation currently
  repairs that journal/files before its later cache comparison refuses. Move
  the cached snapshot guard to the existing DB-open recovery boundary, before
  any pending recovery; clone outside the cache-map lock. Retain empty-root
  maintenance's explicit refresh. Add actual cached-drift/journal/file controls.
- CKR-F2 — `FIX` / `IMPL_BUG` (P2): the final root check is still before the
  first blob write but shares the uncertain-write error branch. A binding
  change after preflight incorrectly marks authority Failed. Move every known
  prewrite check into Preserve-on-refusal and inject that exact boundary in a
  private production-used helper test; explicit durable repair must permit
  retry without checked reset. Failures after attempted persistence stay Failed.

These findings were independently demonstrated on immutable tree
`d58730d7355bbe48232a83086dd0d2bd0dc4963c`; decisions precede fixes.

- CKR-F3 — `FIX` / `TEST_FLAW` (P2 integration, d8b10b3 focused review):
  the existing media restart fixture closes only its DB while retaining a
  cached AgentStore that owns that closed DB. The new strict guard correctly
  refuses it. Source-callsite review finds shipping teardown uses the existing
  paired `retire_agent_owners` operation; the public DB-only close has no
  shipping reopen caller. Use paired retirement in this restart fixture,
  assert both owner caches retire, and retain every media-file rollback and
  cleared-admission assertion. Do not weaken the guard or treat closed data
  as a valid empty checkpoint.

### Normalized-root continuation reconciliation

Integration decision `R-NORMALIZED PRESERVE`: incoming
`3ce1cd31898d090f38ba3a8c5b895da5b2de2892` compares decoded nonempty root
identity at both DB CAS edges while retaining their raw metadata race fence.
Preserve this continuation compatibility, its stricter hexadecimal guard and
its real separate-process storage tests. CKR still rejects whitespace-only
empty-base writes; clarify that narrow exception rather than preserving a
blanket raw-string-identity claim. Keep all cached-root/refusal/cancellation
and existing writer controls, then independently review the integrated delta.
Neither the incoming tests nor this reconciliation establishes provider
load/claim/settle recovery or changes U/global acceptance flags.

### Compaction cancellation-owner reconciliation

Decision `C3-INTEGRATION FIX / IMPL_BUG`: preserve incoming
`d047bfa4d9705f54a5e069e17c53fcc001a475d0`'s Session/DB cancellation guards
and all rollback, lock-wait and positive retry assertions. Route the complete
shipping compaction producer through that guarded entry. The incoming prefix
selector discards typed errors with `.ok()`; retain typed `Cancelled` refusal
independently of the shared token and the existing before-prefix guard. Use
that same selector in the full shipping producer, retaining both sets of app
oracles rather than leaving a helper tested without its shipping caller.
The earlier unpublished summary-only `CC-CANCEL-01..05` identifiers become
`CC-SUMMARY-CANCEL-01..05` under the same compaction specification, distinct
from incoming canonical-commit `CC-CANCEL-01..03`. Every assertion and behavior
remains required. A single receipt lists both scopes without duplicate keys.
Cancellation after the DB's final token read does not undo its commit; no
unknown-effect or original-turn resume acceptance is inferred. Fresh focused
integration review and the final H's Actions union remain mandatory.
