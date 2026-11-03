# Independent R2 GENERAL / security / lifetime / concurrency / oracle review

## Decision

**APPROVE for the declared source-review scope. No MUST_FIX findings.** This is a source review of the frozen actual candidate, not a build, test, runtime, platform, ecosystem, or shipping acceptance result.

One previously documented cost remains **NON_BLOCKING**, recorded here as **PSLOCK-R2-G-N1 (P3)**: repeated durable admission-image reads and parsing on per-record query paths. Its presence is established in source; its performance impact has not been measured. It does not by itself justify another fix round and does not waive eventual performance/soak acceptance.

The review was fresh and blind to prior reviewer reports, runner decisions, and author handoff/outcome reports. Canonical Spec/history and the actual source were used as authority. No R1 review report or decisions file was opened.

## Immutable identity and scope

- Repository: `bhrumom/fabushi-desktop`, repository ID `1375312893`; owner organization `bhrumom`, ID `281146136`.
- Actual source subject: `ca32f83f13a8aed439e9e3341d175405b1d1a655`.
- Actual source tree: `f609dcdc22418f313177c3d86e01103a226799de`.
- Immutable original scope base: `9864d1c02e7e67881d644800963abc807b3abaab`.
- Previous source candidate for the proper R2 fix delta: `ac615165744da1b8462a35b07221800c06aad1fb`.
- Contract: **PS-LOCK-RECONCILE-v2**; canonical Spec blob `fce914e9e59a275b3ddd9da5266eee384b30c27e`.
- Scope blob: `ab57f8983acc660f7eb4f8ca80778d2e6418ab7b`, containing **all 14 paths** below.
- Complete original-scope diff: blob `1a60df0cd1d7cd1a19b982f89ddf6adf0c1d3319`, declared UTF-8 bytes **253512**, SHA-256 `bf710c776576cb761dcc509a653a4bd1c048088342c4fb25ba6193d634824093`.
- Complete proper fix delta: blob `ad0276d15ab16cd6d109240c3942e54491700102`, declared UTF-8 bytes **146456**, SHA-256 `2e1e989586f24805145499c05c82516a1bbf1f91c776037b6bed7583a28c0c33`.
- Full-diff carrier `7f79c0361c055e06574beced8594d74477eff640` and delta carrier `eedf98fbe9cbd903ab39586609dc185b5a63c1b7` are **diff-only artifacts**, not execution or shipping subjects.
- The parent reported the final evidence-only artifact carrier as `8fb24ba966e4048569b6a768b6cdad56d409ef43`, tree `84f7b4e6e0445abde2d149edc865d14938077980`, with source blobs unchanged. This review fetched the supplied immutable scope/diff/source blobs; it did not treat that evidence carrier as the source subject.
- The byte counts and SHA-256 values above are the scope artifact's declared values, not locally executed checksum attestations. The parent reported independently matched API addition/deletion statistics and source readbacks; this reviewer does not relabel those parent checks as its own executed validation.

| Declared path | Frozen source blob |
| --- | --- |
| [projects/openclaw-rust/docs/specs/plugin-skills-snapshot-admission.md](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/projects/openclaw-rust/docs/specs/plugin-skills-snapshot-admission.md) | `fce914e9e59a275b3ddd9da5266eee384b30c27e` |
| [projects/openclaw-rust/evidence/plugin-skills-admission-lock-647e0932-20261103/source-contract.json](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/projects/openclaw-rust/evidence/plugin-skills-admission-lock-647e0932-20261103/source-contract.json) | `155f4dff0f6fe7c7f9f1f3210296beac7bf81a6a` |
| [projects/openclaw-rust/parity-ledger.json](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/projects/openclaw-rust/parity-ledger.json) | `3a966124085449eaa6684e76f9a4cc597e6b0b94` |
| [projects/openclaw-rust/requirements-traceability-matrix.json](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/projects/openclaw-rust/requirements-traceability-matrix.json) | `0c95f4c725bc8dcd8a2c4e668e6b8824e2c79ab6` |
| [projects/openclaw-rust/scripts/capture_portable_contracts.py](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/projects/openclaw-rust/scripts/capture_portable_contracts.py) | `4d36a626d71b3794b2b64ef8a899aaa739ee65fa` |
| [source/host/src/extensions/mcp/plugin_skills.rs](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills.rs) | `56c8c4fc5596e63121ec09624bc8eaff693f9595` |
| [source/host/src/extensions/mcp/plugin_skills_uninstall_state.rs](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills_uninstall_state.rs) | `8b8a03ccde91dfbf5a20ca141d84a073ab9c19b5` |
| [source/host/src/extensions/mcp/plugin_skills_auth_recovery_tests.rs](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills_auth_recovery_tests.rs) | `93ed0cccd79bcb4c8a7a68ed537017ca6fcc1c8f` |
| [source/host/src/extensions/mcp/skill_publish.rs](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/skill_publish.rs) | `d93b54fb169ae09dd6db809dd125f665be5f3545` |
| [source/host/src/extensions/session/production.rs](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/session/production.rs) | `4c88b00b2dfe563ba957bd0669f177a422a330a7` |
| [source/host/src/workflows/workflow_store.rs](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/workflows/workflow_store.rs) | `dd3945c0b454eea446397e033632fac93e60a17d` |
| [source/host/tests/mcp_service_contract.rs](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/tests/mcp_service_contract.rs) | `2fa0aba1e0fa7ac3c3e67c8b392fe43197f0a14a` |
| [source/host/tests/workflow_store_contract.rs](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/tests/workflow_store_contract.rs) | `7d927e0dc69d9183d84dcdec7a31469ae062f74e` |
| [source/host/src/extensions/mcp/plugin_skills_sync_tests.rs](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills_sync_tests.rs) | `80807849a210630af4617ac5db6ab41b0b134230` |

## Configuration and read-only boundary

The requested general-review model/effort was **gpt-6-astra / xhigh**. The parent supplied the exposed launch inputs exactly as `task_name=plugin_admission_r2_general, fork_turns=none, model=gpt-6-astra, reasoning_effort=xhigh`, and the returned task identity `/root/plugin_admission_r2_general`. These are requested settings and exposed tool inputs/results only; no hidden runtime model/effort attestation is claimed.

Root `AGENTS.md` at the actual subject was read, including the complete-scope, immutable-round, blind review, falsifiable finding, source-only evidence, and configured-model rules. The canonical Spec was read in full. Source-of-truth material was read selectively for applicable authority and execution constraints, not represented as a complete history audit.

**No tests, builds, scripts, shell commands, local/htch validation, or other executable validation were run. No GitHub Actions job was triggered.** Tools were used for read-only GitHub source acquisition and textual review, plus the separately authorized report-blob action. No source, tree, commit, ref, or PR was changed by this reviewer.

## Read-depth evidence

The review covered the complete declared original-scope diff and the complete proper R2 delta, not just a proposed fix or one suspected bug. The proper delta was read with duplicate text already present in the fully read original-scope diff reused, and the delta-specific context/removal lines read separately. All 14 scoped source blobs were fetched. Fetching a file is not equated with reading its entire contents.

| Area | Actual reading depth |
| --- | --- |
| Canonical Spec | Full 1356-line Spec, including current contract and applicable historical constraints; no prior review reports or author handoffs. |
| `source-contract.json` | Declared source bindings, case requirements, dependency identity and source-only acceptance limits, including the full changed content. |
| `plugin_skills.rs` | Full product file and state machine: construction, admission observation/acquisition, refresh reservation/custody/disposition, uninstall transition, successor/dispose, coalescing, import, publication, pruning and reporter boundaries. |
| `plugin_skills_uninstall_state.rs` | Complete production portion through lock/persistence/failure-injection helpers, plus full changed tests and adjacent strict-image/migration/owned-staging tests. This is not a claim to have read every unchanged test in the 1543-line file. |
| `plugin_skills_auth_recovery_tests.rs` | Full module: previous controls and new R2 cases, including exact bytes/errors, known-write outcomes, pending custody, unknown recovery, successor and process-lease lifetime. |
| `skill_publish.rs` | Full production implementation and new cleanup-reentry test module; actual trait-to-owner lease adapter and resync/unpublish/upload call chains. |
| `workflow_store.rs` | Affected whole functions and adjacent cache/index/read/update call chains, including authorization and record revalidation under the lease. Not a full-file or whole-workflow-subsystem claim. |
| Session `production.rs` | Complete admission forwarder functions and adjacent binding/test context; not the entire large Session file. |
| `mcp_service_contract.rs` | Complete changed tests plus real Coordinator/relay fixture, actual Session/workflow/prompt consumers, owned subprocess request/receipt/deadline/child lifecycle, and relevant retained adjacent cases. Not the entire integration test file. |
| `workflow_store_contract.rs` | Complete changed real-store mutation cases and relevant fixture/lease-gate context. |
| `plugin_skills_sync_tests.rs` | Complete changed hunks plus relevant existing coalesced result, successor, poisoning, dispose, pruning, uninstall and retained-reader controls. Not all 3122 lines. |
| Capture script | Complete original-scope and proper-delta hunks plus command selection, named positive-test requirements, source bindings before/after execution, compile/capture handling and final acceptance aggregation. Not a claim to have audited every unrelated legacy capture case. |
| Ledger and RTM | Every scoped changed hunk, including preserved incoming changes and evidence/status boundaries; not the entirety of the large unchanged ledgers. |
| Adjacent mechanisms | Plugin cache reader/publication, stat-keyed parse cache and watcher write scheduling; Cargo dependency declaration; the pinned fs2 public lock API and Unix/Windows implementations. No whole-repository, whole-dependency-closure or upstream-product equivalence claim. |

The fs2 inspection was of the declared primary source at commit `e1d4843b7c19e3ce1ecbae92255223de31b36d3b`, including API blob `51b22db6acd0e964adf224e6a155b45660f7abf8`, Unix blob `91867f7e3f3b91c4edd24b3353464158cdc1fd8a` and Windows blob `0e37c66579c0ca7eb4c8138f65d309285e438b4a`. This establishes the source semantics inspected; it does not attest a fetched crate archive, resolved dependency closure, actual target build, or operating-system behavior.

## Findings

### MUST_FIX

**None identified.** No concrete correctness, security, lifetime, concurrency or oracle failure was found in the reviewed source against the selected contract. This is a bounded review conclusion, not proof that no defect exists.

### PSLOCK-R2-G-N1 — P3 / NON_BLOCKING — durable admission-image work repeats per record

**Locations:** [`plugin_skills.rs:519-527`](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills.rs#L519-L527), [`plugin_skills.rs:1184-1185`](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills.rs#L1184-L1185), and [`workflow_store.rs:339-352`](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/workflows/workflow_store.rs#L339-L352).

**Cause and scenario:** Each real `is_denied` call attempts the shared file lock and reloads/parses the durable admission image. Filtering a list calls it per skill record; converting each plugin record to a workflow calls it before and after reading facts, including warm parse-cache hits. Multiple skills belonging to one plugin therefore repeat lock/open/image work, and the total work grows with both record count and admission-image size.

**Impact and qualification:** The repeated work is directly visible in source. No benchmark, latency regression, starvation, or practical user-visible slowdown was measured or inferred as established fact. The canonical Spec already records this image-I/O fanout as NON_BLOCKING at lines 1350-1351. This is a retained observation, not a new failure scenario reversing that decision.

**Disposition:** Retain for measurement in the proper performance/soak evidence stage. If measurements show material cost, use a narrowly scoped immutable observation under the appropriate shared lease for each logical read/batch while preserving live rechecks at externally meaningful mutation/publication boundaries. Do not silently cache positive admission beyond its safe lifetime. No source edit or new fix round is requested by this note alone.

## Correctness, security and concurrency assessment

### 1. Durable format and failure classification retain adverse facts

The production image reader and writer distinguish legacy, initialized, malformed, missing and interrupted states instead of treating every failure as an empty image. Required nullable fields are actually required. V3 uninstall observation tokens are canonical UUIDs or explicit null, and a known non-null observation cannot disappear through an ordinary transition.

The persistence path compares the expected durable image before publication, uses uniquely owned staging files, synchronizes the staged file and parent directory, and preserves an image already exposed when a later marker step fails. A failed write is called known only when the actual bytes match the exact expected image or exact candidate image (including the permitted initialization intermediate). A foreign semantically similar image is not accepted as proof of this writer's outcome. This is material to retaining refusal facts after partial writes.

Relevant anchors: [strict loading/publication and failed-write inspection](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills_uninstall_state.rs#L410-L514) and [regular-file/identity checks](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills_uninstall_state.rs#L523-L568).

The checked paths reject nonregular files and symlink/reparse substitutions at the inspected boundaries. The source review does not extend this to an adversarial process with unrestricted ability to replace arbitrary ancestor directories, delete lock files, or directly rewrite all same-user storage; the implemented contract is the cooperating owner/process serialization and durable admission protocol.

### 2. Observation and mutation admission use different wait and lifetime rules coherently

Read-only `is_denied` uses a nonblocking shared-lock attempt. If a writer holds exclusive custody, an unlocked observation may deny the present query but cannot install a new accepted image or positively release a previously denied identity. A malformed observation under contention returns denied without permanently latching a fault from a transient image; a malformed image under acquired shared custody latches the recovery fault.

Mutation `acquire` waits for shared custody, reads and reconciles under that custody, and returns the actual file lock as the lease. The shared lock remains alive through the caller's protected work. Image I/O precedes the admission mutex, avoiding a state mutex held over filesystem wait/read.

Relevant anchors: [admission observation and lease acquisition](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills.rs#L518-L578).

The lock wrapper uses the inspected fs2 error semantics, a single held File, and RAII close; the refresh pass shares the owning lock through Arc instead of performing independent explicit unlocks on cloned holders. That supports the intended last-owner lifetime in source.

### 3. Refresh ownership persists through real work and is released before callbacks

The refresh pass obtains the separate process refresh lease before reserving the durable refresh intent. A different process cannot enter an overlapping loader while that lease is held. Same-root successor reuse requires the already-fenced predecessor's live owner/token relationship and upgrades its weak lease reference; it does not create independent conflicting process custody.

The actual pass retains strong custody through the loader, imported negative facts, staging/publication, failure cleanup and post-commit pruning. Owner/token-aware guard disposal cannot clear a successor's distinct state. Worker terminal cleanup drops the guard/pass before public reporter work. Same-root handoff preserves the existing admission reader, fences the departed owner, and leaves the successor to establish its own valid token.

Relevant anchors: [refresh reservation/custody](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills.rs#L897-L1051), [successor and dispose](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills.rs#L1103-L1181), and [actual sync/pass execution](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills.rs#L1416-L1691).

Retained source tests cover a completed coalesced result being re-filtered after service-state release, successor claim before old cleanup, the converse entered-prune ordering, and ordinary dispose after a valid commit still permitting valid cleanup/reporting. These are distinct lifetime cases and were not collapsed into one successor test.

### 4. Cross-process completed uninstall cycles cannot evade stale-pass fencing by restoring equal sets

The pass captures the durable uninstall observation before loading. Real uninstall reservations and settlements rotate that observation even when the final confirmed/pending sets equal the earlier sets. Import and publication re-read the durable state under exclusive custody and compare the observed token, sets and exact refresh identity. This catches a complete foreign cycle that the in-process admission reader never observed.

Positive release after successful publication also captures the resulting post-transition observation for cleanup. Pruning requires both the live observation and the current durable observation to match; it refuses to prune while pending uninstall custody exists. The code therefore distinguishes a valid current pass's own rotation from a later foreign change.

Relevant anchors: [uninstall reservation/settlement](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills.rs#L706-L760), [transition persistence](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills.rs#L817-L853), [import and publication](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills.rs#L1219-L1369), and [guarded pruning](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills.rs#L1371-L1414).

### 5. Recovery does not use stale cached authorization as a positive grant

Unknown recovery continues to deny the relevant plugin view and suppresses old-record fallback. Current refusal facts win over apparent success in the same load. Successful identity facts are used exactly: zero-skill success can release the exact ID, while aliases such as `09` and `9` remain distinct. Pending uninstall custody survives fresh loading and cannot be removed by a normal auth success.

The state machine preserves locally imported refusals after a failed negative write, including later ordinary reads, partial refreshes, and durable reconciliation. It does not infer a completed release from an arbitrary image mismatch. Positive release follows the actual cache publication and its durable/readback boundary.

The tests retain exact error and byte-preservation checks rather than merely requiring an empty list or a nonzero error count. Relevant complete test module: [auth recovery tests](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills_auth_recovery_tests.rs).

### 6. Actual mutation consumers revalidate and hold the correct lease

The concrete publisher adapter obtains the real owner's admission lease. Resync acquires the lease, re-reads the current committed index, checks the same plugin identity and current publisher/team authority, and holds the lease through source reads, staging, backend work and cleanup. Unpublish holds it through restoration and the actual backend mutation. Follow-up sync is after release.

A cleanup error is captured first; the lease is then dropped before public logging or edge reporting. This matters because those callbacks may synchronously reenter the skills owner. Both backend success and backend error remain the original result after cleanup diagnostics.

Relevant anchors: [publisher operations and cleanup boundary](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/skill_publish.rs#L135-L207) and [current publisher/team check](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/skill_publish.rs#L233-L256).

Workflow update re-reads the committed record and publisher facts after obtaining the actual shared lease, comparing the selected record rather than proceeding with the pre-lease record. A removed/remapped record or changed/unknown publisher cannot authorize a write using an earlier grant. The relevant stat cache uses file identity and metadata, and the read of the index occurs after custody acquisition; watcher scheduling does not synchronously call arbitrary change handlers inside the protected write.

The Session forwarder clones the bound delegate and releases its forwarding mutex before calling `is_denied` or `acquire`. The unbound construction behavior is retained explicitly; the reviewed production binding uses the live owner.

## Test and capture oracle assessment

This section assesses test source and causal design, **not test execution**.

- **Full foreign uninstall cycle:** `cross_process_uninstall_cycles_fence_unobserving_refresh_before_publication` at [MCP contract lines 2188 onward](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/tests/mcp_service_contract.rs#L2188) holds the real Coordinator loader while an owned child performs real Removed and NotRemoved cycles. The parent deliberately avoids admission/current/status reads until the child completes. The final refusal/pending sets return to their prior values while the durable UUID differs. The held old pass must return the specific stale-observation error, preserve cache/files/sentinel, and emit no success report. A later valid fresh pass must recover, prune and report. This directly tests the lasting observation token rather than accidentally relying on an intermediate local read.

- **Refresh-process lifetime:** [the process-lease integration case](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/tests/mcp_service_contract.rs#L1929) checks actual competing child completion and explicit exclusion before loader entry, then a later process is admitted after the failed owner terminates. The unit cleanup-lifetime case at [auth recovery line 1004](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/plugin_skills_auth_recovery_tests.rs#L1004) holds the relevant terminal boundary. These inspect actual lock ownership rather than treating a timed-out wait as proof of exclusion.

- **Publisher lifetime and callback reentry:** [the real-owner integration case](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/tests/mcp_service_contract.rs#L2009) invokes concrete resync/unpublish, checks exclusive contention before acquiring a compatible shared probe, and retains denied/wrong-publisher/missing-team controls. [The cleanup test at line 504](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/src/extensions/mcp/skill_publish.rs#L504) causes an actual filesystem cleanup failure, proves exclusive admission is available before the callback's real reentrant sync, and preserves the backend success/error result. Ordering the exclusive probe first avoids the test manufacturing its own exclusion evidence with an extra shared lock.

- **Workflow stale authorization:** [the mutation contract cases](https://github.com/bhrumom/fabushi-desktop/blob/ca32f83f13a8aed439e9e3341d175405b1d1a655/source/host/tests/workflow_store_contract.rs#L815) pause before the real lease, publish changed committed facts with the actual service, and then resume. Different publisher, unknown publisher, remapped record and removed record reject without modifying either old or new file bytes. The unchanged-publication case remains a positive control.

- **Real consumers:** The MCP fixture creates real ProductionSessionWorkers, binds the actual admission reader, and exercises retained and reopened workflow stores, command dispatch, run-now/reference expansion and prompt assembly, with user/managed/allowed-plugin controls and exact preserved bytes. The controlled component is remote I/O at the actual relay boundary, not the admission decision itself.

- **Owned subprocess discipline:** The fixture binds requests and receipts to a nonce, expected PID/phase and exact ignored child entry; it retains the actual Child handle and limits cleanup to that owned child. Explicit deadlines detect failure to progress. File-backed logs avoid pipe-drain dependence. This does not claim arbitrary descendant-process cleanup.

- **Capture selection and result accounting:** The changed capture selection includes the broad plugin owner tests and the specific new publisher, workflow and cross-process cases. Acceptance requires successful command/capture exits, exact positive test-name occurrence for required cases, successful nonempty result summaries, and before/after source bindings matching HEAD bytes. Final receipt acceptance includes the plugin source-unchanged condition together with the other preserved cases. The changed script does not upgrade source inspection into a claimed execution result.

## Remaining evidence boundaries

No exact-subject compile/test result is established by this review. The capture script and tests still need the authorized GitHub Actions execution on the proper actual source subject, with receipts tied to its exact source inputs. Target-specific lock and filesystem behavior, resolved dependency closure, crash/durability behavior on actual platforms, real desktop/ecosystem integration, performance/soak, and overall migration/shipping acceptance remain outside this source-only approval.

Existing status/custody assertions were not weakened as a review recommendation. Shared ledger/RTM/capture changes were included in the declared diff review; unrelated preserved source/evidence claims were not upgraded.

## Report-save authority

Before the separately authorized one-report-blob save, fresh GitHub API checks returned:

- expected repository ID `1375312893` and owner ID `281146136`;
- public, unarchived repository;
- current repository `admin` and `push` permissions;
- authenticated profile nickname/login `bhrum`, ID `222737880`;
- explicit collaborator permission `admin` for authenticated login `bhrum`.

An initial collaborator lookup used the organization name `bhrumom` and returned `none`; that is not the authenticated actor. The subsequent lookup used the authenticated login and established current admin permission. No sensitive profile fields are included here.

Only this new review-report blob is authorized for this reviewer. Its creation/readback outcome is recorded in the final delivery message. No commit, tree, ref or source publication is performed by this report save.
