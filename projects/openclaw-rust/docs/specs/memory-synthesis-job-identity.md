# Background Memory synthesis snapshot identity — MSI-v1

Status: implementation contract; execution and acceptance pending.  
Task: `d56b3d1d-6125-4f6f-a8d8-e7e36728a1ba`.  
Discovery H: `859e085f490e17acb8520c3fd0f328b717c3c0e7`; implementation base:
`7a89cc44924bf900731febbb7cc0c3779d61c300`, tree
`69cfb4b90711d78d997403d837c6a4d8ce33a291`. Product owners are unchanged
between those commits. Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`,
tree `90c8aab39972da11cfad220361781d49e1bc6700`.

## Problem and decision

The shipping Memory factory invokes the independent summary API separately for
proposal and verification. Each call resolves the current provider/model and a
fresh credential context. One snapshot's history, evidence and proposed changes
can therefore be sent to different Native accounts or Codex workspaces. A final
Session admission/fingerprint refusal cannot undo that disclosure. The same
recapture is possible on the existing bounded retry loop. CWB-v1 intentionally
keeps independent summaries fresh; this is an additional consumer contract.

**MSI-01 — FIX:** give each `MemorySynthesisService::run_agent` snapshot one
Inference-owned, process-local summary job. Proposal, verification and all
attempts of that snapshot share the admitted route and existing Native/Codex
model and request bindings. No new scheduler, credentials store, Memory store,
permission truth, Agent loop or runtime is introduced.

## Entry, owners and shared contract

The existing authenticated-bootstrap experiment enables the production Memory
extension. Completed turns use its evidence bridge; temporal sweeps also operate
without new evidence. MemorySynthesisService remains the sole queue, snapshot,
deadline/retry and settlement owner. ProductionInferenceExtension owns provider
selection and summary execution. Existing provider transports load secrets;
Session's apply gate and FileMemoryStore remain the only write owners.

`SummaryInferenceJob` is opaque and allocated by
`ProductionInferenceExtension::create_summarization_job(self: &Arc<Self>,
data_dir: PathBuf)`. Allocation performs no settings/credential I/O. Its
`run_prompt(system, user, should_cancel)` method admits exactly once, inside the
existing inference worker, and delegates to the existing summary execution path.
An immutable one-time result retains either the admitted identity or the typed
admission refusal. It retains no token/auth document and cannot be serialized.
ProviderSessionError may gain Clone to replay that immutable refusal without
turning it into an untyped string.

Memory receives one pair of typed callbacks per snapshot via
`SynthesisModelJob` and `MemorySynthesisOptions::new_with_job_factory`. Its
factory is allocation-only. Existing callback-based construction remains an
adapter to the same job executor; existing verifier callers migrate to an
explicit setter/builder. Factory and executor do not create another queue.
Production callbacks preserve Authentication, Configuration and Cancelled
categories; ordinary output/transport failures retain the existing retry policy.
The public SynthesisOutcome/telemetry shape stays compatible (`Failed`).

## Required behavior

| ID | Required behavior and causal oracle |
| --- | --- |
| MSI-01 | One route is admitted per snapshot. Native/Codex models and bindings are captured before its first dispatch and retained through verification and every retry. A later settings change affects a later independent job. |
| MSI-02 | Each actual transport rereads current credentials through its existing binding. Same Native issuer/user/device/session context or same Codex absolute source/workspace may rotate tokens. A different identity refuses before another HTTP request; no fresh binding, fallback or immediate proposal retry may adopt it. |
| MSI-03 | Authentication/Configuration refusal is terminal for this snapshot. It performs no apply and does not advance temporal-review metadata. It settles the current snapshot evidence through existing finish semantics; newly queued distinct evidence remains. A later independently admitted job can succeed under the then-current identity. Diagnostics contain no tokens or memory payloads. |
| MSI-04 | Check cancellation before admission, after admission and before each phase. After a successful worker result, the consumer checks lifetime cancellation again before no-work metadata or apply. A timed-out attempt cannot apply its late result; later attempts retain the same job identity. Initialization that finishes after an attempt timeout may populate the identity, but that cancelled attempt must not dispatch. |
| MSI-05 | Stale retires the snapshot job. Existing evidence remains queued and the next run_agent resnapshots under a new admission. No old proposal/result is reused. New evidence arriving during the job is preserved by existing evidence-ID settlement. |
| MSI-06 | Temporal-only work uses the same job identity rules. Existing grounding, explicit-memory protection, tombstones, canonical Session admission, append-read integrity, Invalid/Rejected/no-op and ordinary retry rules remain. Only typed identity/configuration/cancellation refusal receives the new terminal policy. |
| MSI-07 | Generic independent summaries stay fresh, admitted turn-memory/automatic compaction keep their turn identity, summary requests expose no tools and keep usage reporting. Packaged/product mode still forces Native. Other supported routes retain their existing functionality; this batch fixes their selected route, without claiming unreviewed provider-specific credential continuity. |

OnceLock admission may serialize bounded workers waiting on the same capture;
it is not a new scheduler or a claim that synchronous filesystem I/O is
interruptible. Attempt threads remain detached on deadline under the existing
owner. A final cancellation check is not atomic with a filesystem commit; full
worker drainage and generation/commit linearization remain open requirements.

The scope is one snapshot and its retries, not one permanently bound queue item
or every original conversation. Stale and service replacement/restart are new
maintenance admissions. Original memory-to-account authorization, durable queue
recovery, complete OpenClaw dreaming behavior and all other provider identities
remain in the full migration scope. No evidence from them is manufactured here.

## Implementation and verification

Use the existing Inference and Memory production files and their adjacent
contract targets. Root serially integrates shared workflow, ledger/RTM and
source observations. Independent workers receive exact base, MSI-v1 interfaces
and disjoint paths. No dependencies, persisted schema, UI or RPC are added.

The production oracle must reach the real Memory factory, worker, production
Inference/request serialization/transport, Session gate and FileMemoryStore.
Use private synthetic files and a loopback server response barrier to mutate
settings/credentials after a real nonempty proposal dispatch. Assert the sent
model/identity, exact dispatch counts, token rotation, complete Memory metadata
bytes, one successful apply, later independent admission and secret-free errors.
Do not substitute injected proposal callbacks for the production identity proof.

Cover Native positive/changed-context refusal in product mode, Codex positive/
changed-workspace refusal, retry continuity, temporal-only refusal, cancellation
and Stale/concurrent-evidence boundaries. Existing generic service/state/store
and model/compaction/turn-memory targets provide regressions. A cfg(test)-only
worker-entry hook may enter the existing thread-affine Codex request hook from
a Send-safe fixture plan; do not move Rc across threads, change production URLs
or introduce global endpoint overrides. Preserve hook liveness checks.

The MSI-04 race oracle also uses a cfg(test), worker-local admission-result
barrier after actual capture and before OnceLock publication. It neither supplies
an identity/result nor changes production I/O: the first attempt reaches its real
deadline while capture is held, a retry enters the same job, and the fixture
changes current settings/credentials before releasing capture. Assert retained
model/context or sticky admission refusal, no dispatch by the cancelled worker,
no late apply, and subsequent independent admission. Owned fixture processes use
once-only bounded retirement and keep unknown terminal state/diagnostic roots.

The existing Host composition supplement must follow the shared job factory and
its delegation to shipping Inference. Preserve its authenticated-bootstrap,
prompt, cancellation, telemetry and Session-gate assertions. Requiring the old
independent-summary call inside Memory would contradict MSI-01; the actual HTTP
oracles remain the behavior evidence beyond this source-wiring supplement.

Freeze the whole scope, obtain independent general/security/lifetime and
structure/reuse/performance review, and run affected shipping compile and
contracts only in GitHub Actions on exact H. Source authority/coverage and
behavior evidence must belong to that H before marking a narrow unit verified.
Real accounts/services, UI/platform/package/soak and G01–G12/AC01–AC24 remain
pending or blocked with their own evidence; mocks do not establish compatibility.

## Provenance, release and rollback

The source contract records exact U blobs and honest full/partial/search-only
reading. The actual upstream dreaming path uses Gateway subagent.complete;
the separately inspected direct plugin LLM API is not substituted for it. The
review does not establish that an upstream whole dreaming sweep pins one
credential. This target contract follows the causal production gap and the
reviewed owner/admission principles. Unread downstream upstream dependencies
remain open; they are not dependencies of this existing-owner target fix.

Applicable authority: OCRM main Spec §17, source lock, current Memory/Inference
mapping, CWB-v1, model identity and memory-append-read-integrity contracts.
Source assessment: `evidence/memory-synthesis-identity-d56b3d1d-20261103/source-contract.json`.
This extends existing Memory/model responsibilities without raising global flags.
No data migration is required. Rollback retires process-local jobs using the
existing service lifetime; reverting the change restores the known between-
phase disclosure and therefore cannot claim the MSI safety contract.
