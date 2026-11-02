# Memory append read integrity — MWR-v1

Status: implementation contract; exact-H Actions and final acceptance pending.
Discovery source: `6034da1fcc1ecfe95db4fe5532c61cf88872fb5b`; implementation base
`bcada1ae9831aba9b021906fc878ec1a7506532b`; integrated after the parallel
media/config/search batches at `8e5545b61ad916bb9781539dfe0f28543ffc68dd`.
Fixed upstream U remains
`1440bb1771372334bea23d7642bb52deb7db1845`.

## Problem, responsibility and production path

`FileMemoryStore::add_memory` and `add_synthesized` currently turn every
`read_to_string` failure into an empty string. When an existing profile or
monthly log contains unreadable bytes, the subsequent atomic replacement can
discard those bytes and existing history while reporting a successful append.
The read-side fact parser skips that file; it does not provide a write guard.
The synthesis fingerprint also treats the same unreadable file as empty, so
unchanged corrupt input is not necessarily stale.

The existing producer is the Runner memory tool / `SandAgentState::write_memory`.
Agent-scoped writes pass through the composition-bound
`ProductionSessionWorkers::write_agent_memory`; user/project scopes retain
their existing membership and shard selection. Every append reaches the same
FileMemoryStore. Synthesis reaches it through the existing MemoryService
apply gate and Session filesystem/admission owner. Keep those owners, locks,
formats, callback/error contracts and permission decisions.

Decision **MWR-01 — FIX / IMPL_BUG**: distinguish a missing append destination
from an unreadable existing destination before replacing file contents.
For synthesis, check every declared Create/Update append destination before
the first mutation; a later corrupt destination must not be discovered only
after an earlier Remove or the removal half of Update.

## Required behavior

| ID | Contract and oracle |
| --- | --- |
| MWR-01 | A missing destination preserves existing first-write behavior. Any other destination read error, including invalid UTF-8, propagates as the original `io::Error`; do not substitute an empty file or recreate/truncate/rename over its bytes. |
| MWR-02 | Explicit append performs this read admission before its own file or synthesis-metadata writes. Empty input retains the existing no-op. Readable files retain headers/comments, normalization, deduplication, record IDs, ordering, timestamps and origin behavior. |
| MWR-03 | Synthesis checks all declared nonempty Create/Update destinations before executing any changes, including a preceding Remove and a cross-Profile/Log Update. Existing invalid-ID, explicit-origin, duplicate-change, tombstone and stale-snapshot rules remain. A known unreadable destination refuses the operation without modifying memory, origin/tombstone or temporal-review bytes. A duplicate/no-op append request may also be refused when its declared destination is unreadable: an unreadable file is not authoritative evidence that the append is unnecessary. |
| MWR-04 | Existing Session/Runner error consumers must observe failure instead of successful Remember/Committed. After fixture-owned repair, the same live owner can append successfully while preserving the repaired history. Opening the same files through another Store handle also refuses unreadable input without implicit repair; this control does not claim full process restart acceptance. |
| MWR-05 | Actual store and production consumer tests retain valid missing/readable/duplicate and synthesis create-update-remove controls, demonstrate the original corrupt bytes are present/readable as raw bytes, compare complete owned Memory trees before/after refusals, and prove a valid operation after repair. |

Use one existing-owner helper for append-path/read policy where it removes
duplicated policy. No second store, transaction journal, permission truth or
synthesis scheduler is introduced. Existing filesystem admission and deletion
fences remain in force. Readability checks are synchronous and add no accepted
background work or cancellation state.

Keep the existing fingerprint-based Stale decision before the new synthesis
preflight. With a matching fingerprint, a destination read error may precede
an independently invalid change in the same batch; preserve the read error
and refuse before mutation. This deliberately does not promise the former
Invalid-versus-I/O ordering for two simultaneous faults. Empty content retains
its original Invalid handling without requiring a destination read.

Decision **MWR-02 — FIX / IMPL_BUG**: the real temporal synthesis consumer must
also preserve that refusal. Keep Store I/O failure as the internal
`SynthesisApplyResult::Failed` through the existing Session gate and target,
and report `SynthesisOutcome::Failed`. The service must not mark temporal
review complete after a failed apply. Preserve the existing Invalid, Stale,
Committed and canonical/admission refusal decisions. The current attempt still
finishes through the existing queue owner; it does not automatically replay a
possibly partially applied write. New evidence or the normal due poll obtains
a fresh snapshot. This changes no wire schema, persisted format or scheduler.

## Verification and integration

Decision **MEMORY-CANONICAL-ORACLE-01 — FIX / TEST_ORACLE_BUG**: the newly
selected shared `bound_synthesis_gate_rejects_a_noncanonical_store_path` control
failed on exact H `32a9a391199b03d8b8df76380701c14d56f84834` and
`c6be0cbc7bd47ac3392d02bec5b3ee8aa75a61bb`: its supposed healthy agent had never
been created. The existing Session admission gate correctly returned Invalid
before Store apply. Materialize the positive Session through its actual owner,
use its issued ID, and preserve both original Committed and noncanonical Invalid
assertions. Keep missing-agent refusal as an explicit negative; use a nonempty
proposal to show foreign bytes remain unchanged and a valid canonical write
still succeeds after refusal. No production admission rule changes. These
failed subjects remain failed; only the repaired exact-H Actions can establish
the corrected control. The downstream preview MCP receipt absence caused by
the skipped Automation step is not an independent preview implementation bug.

Production changes use `source/host/src/extensions/memory/memory_service.rs`,
the existing Session synthesis gate, and MemorySynthesisService's apply-result
consumer. Independent review identified the temporal marker side effect in
MWR-02; no replacement owner is needed. Store oracles belong in the existing
`memory_service_contract.rs` and `memory_synthesis_store_contract.rs`;
production consumer controls use existing Session/Runner composition,
`SandAgentState`, and the actual production synthesis target plus Session gate
and service. A controlled proposal exercises the real consumer without claiming
provider compatibility. Root integrates workflow
and RTM changes serially. Preserve MGS-v1 selector controls and every inherited
shared contract.

Decision **MWR-FIXTURE-01 — FIX / TEST_FLAW**: Runtime `41537633075`, Linux
job `125593431475`, at `c6be0cbc7bd47ac3392d02bec5b3ee8aa75a61bb` reached
`bound_synthesis_gate_rejects_a_noncanonical_store_path` but its canonical
positive used an ID without materializing a Session DB. The production gate
correctly returns Invalid when Session work admission fails. Create a real
Session through the existing owner for this positive and its foreign-path
negative; retain Committed for the canonical admitted store and Invalid for
the foreign store. Also retain an explicit canonical-but-missing-Session
Invalid control. Do not relax production admission or replace the positive
with a refusal expectation. Reuse the already-selected complete contract
target; downstream missing receipts from this failure are not compilation
or compatibility evidence.

Freeze the complete implementation/tests/CI candidate and obtain independent
source review. Extend the existing Linux portable Host command and macOS
recovery command with the affected targets and named positives, bind receipts
to actual H/tree/U and command/log identities, and read their results from
GitHub Actions. All executable validation remains in Actions; authored tests
and source review are not runtime evidence. This batch preserves existing
Fabushi behavior under OCRM-R13/R24 and AC13/AC17/AC18/AC24; it does not grant
new upstream reading, equivalence or global completion credit.

## Failure, migration, rollback and limits

### Completed-turn episode append custody — MEA-v1

Discovery H: `79920666e475920109690520cd913df40b28c0f7`; U is unchanged.
The shipping Host completed-turn path constructs `ProductionEpisodeProgress`
and calls `runner::turn_memory::run_turn_memory_with`. After a nonempty episode
summary, that consumer currently discards `FileMemoryStore::add_memory` errors
and unconditionally clears the Session-owned pending episode turns. The MWR
store refusal therefore protects the file but can still lose the input needed
to create the episode after repair.

Decision **MEA-01 — FIX / IMPL_BUG**: an episode append I/O error must return
without clearing pending episode turns. Keep the existing Runner, FileMemoryStore,
Session DB and production progress adapter; add no retry queue, journal or timer.

| ID | Required behavior and oracle |
| --- | --- |
| MEA-01 | If a nonempty summary reaches the actual Store and append returns Err, do not call pending-turn clear and do not report an added episode. Retain every current pending turn, including a turn appended through the Session owner while the model callback was running. |
| MEA-02 | Refusing an unreadable destination preserves its raw bytes under MWR. A real write-stage failure on a readable destination also retains pending input and must not remove or repair the fixture-owned obstacle. Reopen the actual Session storage and observe the same pending turns. |
| MEA-03 | After an explicit fixture-owned repair, a subsequent ordinary completed turn uses the retained input, persists the summary through the existing Store and then clears pending input. No automatic replay or provider call is scheduled by the failure branch. Acknowledged duplicate/no-op append keeps the existing successful-settlement behavior without adding a second fact. |
| MEA-04 | Preserve existing extraction, model refusal/empty/NONE handling, evidence mode, provider/account binding, normal episode limits and successful result flags. This narrowly changes Store append failure settlement; it does not claim a new account authority, exactly-once cross-store transaction, or concurrent-success settlement fence. Those broader Memory requirements remain open. |

Use the existing `turn_memory_contract.rs` target with the real
`ProductionEpisodeProgress`, `ProductionSessionWorkers` and file store. Pin the
episode interval in a fixture-owned child process so parallel tests cannot mutate
process-global environment. The write failure is a directory occupying the exact
owned atomic-write temporary path, after a readable destination is established;
it must exercise Store write failure rather than a stubbed error. Inspect complete
pending-turn values and raw memory bytes before/after refusal and after reopen;
retain normal, duplicate and repair positives. Add named controls to the existing
Linux and macOS recovery receipts and preserve all earlier controls. Freeze the
complete scope for independent source review, then verify only its exact H in
GitHub Actions. Source review or authored tests do not constitute execution proof.

No persisted format, public RPC or new UI state is added. Restart uses existing
Session pending-turn storage. This is an existing Fabushi consumer preservation
under OCRM-R13/R24 and AC13/AC17/AC18/AC24, not new upstream reading or full Memory
equivalence credit. Real provider, concurrent-success, cancellation/account-policy,
device/platform/package and final acceptance requirements remain independently open.

No persisted format or migration changes. The existing `StateWriteResult`
failure / tool error and synthesis apply failure carry refusal; the user must
repair or restore the existing file before retrying. Do not log memory content
or silently perform that repair. A source rollback needs no data conversion.

This contract prevents replacement caused by a known append-destination read
failure. It does not establish a multi-file transaction for arbitrary later
write/disk failures, protection against external writers changing files after
the read, cross-process locking, symlink confinement, strict read-side listing
errors, clear/delete recovery for unrelated corrupt files, or complete Memory
equivalence. Those remain required work under the full migration. Real
providers, UI, devices/platform packages, source/dependency closure and final
G01-G12/AC01-AC24 acceptance remain open; no inventory or global flag is raised.

## Source anchors

Discovery source: `memory_service.rs::{add_memory, add_synthesized,
apply_synthesis, read_synthesis_state}`; `watched_directory.rs` atomic
file replacement; `host_runner_composition.rs` memory-writer binding;
`session/production.rs::{write_agent_memory, synthesis apply gate}`;
`memory/agent_state.rs::write_memory`; `runner/sand_memory.rs` error consumer;
`memory/production.rs::ProductionSynthesisTarget` and
`memory_synthesis_service.rs::run_agent` temporal settlement.
Authority: OCRM §10, §12 and §17; existing Session work admission and Memory
synthesis contracts. Historical Memory dossiers retain their original source
identities and review depth.
