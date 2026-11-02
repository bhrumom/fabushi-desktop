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
