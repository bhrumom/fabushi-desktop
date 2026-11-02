# OCRM-P0-CONFIG-001 — Configuration, Schema Normalization & Single-Writer Settings Authority

Status: **Historical scoped mapping retained (6 responsibilities, 8 source reads at the historical source); current fixed-U authority, exact-head verification and acceptance pending**
Unit: `OCRM-UNIT-CONFIG-001`  
Candidate requirements: `OCRM-R04`, `OCRM-R09`, `OCRM-AC05`, `OCRM-AC17`  
Dossier: `projects/openclaw-rust/dossiers/config-settings-scoped.md`  
Historical scoped source oracle (not current authority): `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`
Fixed implementation source: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` / tree `90c8aab39972da11cfad220361781d49e1bc6700`; accepted identity and scoped evidence remain pending. Later discovery candidates do not reopen this fixed subject.
Investigated target: `bhrumom/fabushi-desktop` HEAD

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
