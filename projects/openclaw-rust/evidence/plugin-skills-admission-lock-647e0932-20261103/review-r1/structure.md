**REUSE / STRUCTURE / PERFORMANCE review — REQUEST CHANGES**

This is a fresh source-only review of the frozen twelve-path scope. I found four concrete issues requiring repair: a cross-process uninstall observation can disappear through an ABA transition; workflow mutation can use publisher/path facts sampled before its admission lease; authoritative reader errors do not latch the recovery fault; and publishing cleanup callbacks can reenter refresh while still holding its shared admission lease.

The more general `P → EX / backend → P` inversion discussed during investigation is **not a separate proven MUST_FIX finding** in this report. S4 below identifies a specific callback path and a direct self-deadlock, without depending on hypothetical production backend reentry.

No source was edited. No builds, tests, linters, generators, benchmarks, uploads, local validation, or `htch-runtime` validation were performed.

**Source identities and review boundary**

- Repository: `bhrumom/fabushi-desktop`.
- Actual reviewed source commit: `ac615165744da1b8462a35b07221800c06aad1fb`.
- Actual reviewed source tree: `61ef27820a6ba5dc696a276cd58b30e153a25b57`.
- Original baseline commit: `9864d1c02e7e67881d644800963abc807b3abaab`.
- Original baseline tree: `268db009331dca388e8662d95e2ce494936ee5ad`.
- Isolated text comparison carrier: `6efa4f63ec4060e11180fa32595f0d68e1ebb51b`.
- Isolated comparison tree: `d83642aca56686f5ef01cd9d0b36497565d7e1ea`.
- Cloud evidence carrier supplied for this round: `dffc80c8f72cbb1f631152f70b9d5c80491808f2`.
- Scope manifest blob: `b0b2aaa779bf8473daf48d84c467f42c2e62df4a`.
- Complete baseline diff blob: `241aa8208a796ef9f675b3647bd25912f9e6459e`.
- Scope-declared diff SHA-256: `b58d24756fd4f310b8a2cecb2be57ffc0fd2487ee164aa1929129073e8209155`.

I used the actual source subject for source reading. The isolated commit was treated solely as the complete scoped baseline comparison; it is not an execution or shipping subject. The recursive actual-source tree listing returned `truncated: false`. All twelve manifest blobs matched that tree. Counting the supplied patch additions/deletions also matched every manifest file statistic.

The applicable policy was root `AGENTS.md`, blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2`; the exact tree contained no nested `AGENTS.md` governing these source/project paths. The review contract was the canonical Plugin Skills specification, including AR-01–09 and AR-LOCK-R1, and the current `source-contract.json`.

The repository lens policy requests `gpt-6.1-sol` at `xhigh` for this lens. As supplied by root, the actual root spawn tool invocation explicitly set `fork_turns=none`, `model=gpt-6.1-sol`, and `reasoning_effort=xhigh`, and returned this worker's canonical task name. These are the requested/applied tool settings reported by root; I do not independently attest to hidden runtime model identity.

No prior review-report file, `initial-decisions.json`, author handoff file, failure assessment, or another reviewer’s results was opened. The required canonical specification itself contains historical rationale; that text was treated as contract material rather than independent validation or arbitration evidence.

**S1 — MUST_FIX / P1: Cross-process ABA can release a newer confirmed uninstall using an older materialization**

Affected source:

- `source/host/src/extensions/mcp/plugin_skills.rs:483–486`, `PluginSkillsAdmissionState::observe_durable`.
- `source/host/src/extensions/mcp/plugin_skills.rs:1208–1215`, `import_live_refusals_if_active`.
- `source/host/src/extensions/mcp/plugin_skills.rs:1275–1283`, `publish_snapshot_if_active`.
- The unsafe release occurs at `source/host/src/extensions/mcp/plugin_skills.rs:1293–1298`.

Cross-process fencing compares the current `confirmed` and `pending` sets against the pass’s original sets, plus the refresh operation ID. Those final values cannot prove that no uninstall transition happened between observations. The opaque in-process uninstall observation is advanced only when this owner observes a different set, so it cannot recover a transient change that another process completed before this owner next reads.

A concrete failure schedule is:

1. Plugin `9` is already confirmed uninstalled; the image has `confirmed={9}`, `pending={}`.
2. Owner A starts an actual refresh, reserves refresh operation `R`, and captures those sets. Its loader materializes a legitimate fresh `9` result, then is held before returning that result to A.
3. Owner B, in a separate process, calls `begin_uninstall("9")`, performs the existing backend uninstall, and settles `Removed`. A new pending entry exists during B’s operation. Once B settles, the final sets are again `confirmed={9}`, `pending={}`, and B has preserved `R`.
4. A has not read the intermediate pending image. Its final comparisons all succeed: the original and current sets match, and the operation remains `R`.
5. A publishes its pre-second-uninstall materialization and removes confirmed `9` at lines 1293–1298. This reopens `9` after the newer confirmed uninstall.

The protocol explicitly permits a new pending operation to coexist with an older confirmed fact, so step 3 is supported behavior. A simpler reserve/NotRemoved ABA also defeats the promised observation fence, but the repeated-confirmation case demonstrates a positive-release error with a newer actual confirmed removal.

This violates AR-LOCK-R1’s cross-process stale-publication fence, PR-01’s observation advancement for every phase transition, and the requirement that fresh materialization begin after the relevant terminal uninstall observation before releasing that fact.

**Specific fix:** persist a causal uninstall observation identity that changes on every real uninstall transition, or use an equivalent protocol that preserves those intervening transitions. A final-state equality check is insufficient. Bind the refresh pass to that identity at reservation and verify it before negative import, publication, positive release, and cleanup as applicable. This should remain part of the existing durable image/owner, with any necessary private-format contract change made explicitly.

**Required causal control:** use the existing owned-process harness to hold A’s actual loader while B performs the complete confirmed→pending→confirmed cycle. Do not let A read the intermediate pending image. A’s stale result must fail without cache replacement, refusal release, pruning, or success reporting. A subsequent explicit post-observation fresh materialization should still restore the exact ID normally.

This conclusion is derived from source and the stated schedule; it was not executed.

**S2 — MUST_FIX / P1: Workflow writes do not revalidate the committed publisher and record after acquiring their mutation lease**

Affected source:

- `source/host/src/workflows/workflow_store.rs:582–604`, `FileWorkflowStore::update_plugin_skill`.
- Relevant precedent: `source/host/src/extensions/mcp/skill_publish.rs:216–235`, `require_published_plugin_skill`.

`update_plugin_skill` reads the plugin record and cache index, checks publisher/current-user equality, reads the selected file, and encodes the replacement **before** calling `reader.acquire`. That acquisition can wait behind a publisher’s exclusive admission lock. After it succeeds, the function only knows that the earlier plugin ID is not negatively denied; it does not know that the earlier cache record, publisher identity, current user, or file path is still current.

A concrete schedule is:

1. The committed index has publisher/current user `A` for plugin `9`. A workflow update reads that record, passes the publisher check, and prepares bytes for the recorded path.
2. Before the update obtains SH, a real refresh publishes a new committed index under EX. It changes the current user to `B`, or replaces the canonical record/path. Plugin `9` remains installed and has no negative admission fact.
3. The update acquires SH after that publication. `acquire("9")` returns a lease because it checks negative admission, not publisher permission or record currentness.
4. The update writes the earlier path using the earlier publisher authorization.

This is a committed-snapshot race. It does not require a claim about full account lifecycle or backend incarnation after a response. The new mutation lease is the serialization boundary for the write, and the permission/record facts must be checked within that boundary.

The changed publishing consumer already implements the appropriate local precedent: it locates an initial ID, obtains the lease, rereads the index, and checks the canonical plugin plus publisher facts while the lease remains held.

**Specific fix:** after acquiring SH, reread the authoritative committed record and its current publisher/current-user facts, verify that it is still the intended canonical plugin, and select/read/serialize the file to be replaced from that current record. Refuse rather than modifying an earlier path if the record or edit grant changed. Avoid introducing a second permission policy in `FileWorkflowStore`.

**Required causal control:** gate the workflow update immediately before actual lease acquisition, publish a changed identity/path through the real Plugin Skills service, then resume the update. Assert refusal and unchanged bytes at both the earlier and current skill files. Retain an ordinary current-publisher positive control.

This conclusion is derived from source; it was not executed.

**S3 — MUST_FIX / P2: Authoritative admission reads return refusal on image errors without latching the recovery fault**

Affected source:

- `source/host/src/extensions/mcp/plugin_skills.rs:516–519`, `PluginSkillsLiveAdmission::is_denied`.
- `source/host/src/extensions/mcp/plugin_skills.rs:544–548`, `PluginSkillsLiveAdmission::acquire`.
- Contrast with the existing owner’s fault-latching read at `source/host/src/extensions/mcp/plugin_skills.rs:760–767`.

When an authoritative read obtains SH but `load_uninstall_image` fails, `is_denied` returns `true` and `acquire` returns an error. Neither records `recovery_fault` nor advances the uninstall observation. An ordinary read can therefore discover an invalid initialized image without putting the retained owner into the recovery-fault state required by AR-02/PR-03.

A direct reproducer is:

1. Seed a healthy initialized service and retain its reader.
2. With no writer active, remove only `uninstall-state.json`, leaving its initialization marker, or replace the image with malformed bytes.
3. Call the retained reader. It refuses this call, but status has no latched integrity fault.
4. Restore the exact previously accepted bytes.
5. Call the same retained reader again. It admits unrelated allowed plugins and can allow later refresh/mutation, because no fault was retained.

The new total-custody-loss control exercises a different branch: both files absent decode as the default image, so `observe_durable` notices the disappearance and latches a fault. It does not exercise the image-loader `Err` exits above. Existing recovery semantics require an unsafe initialized image to produce a recovery fault that a retained owner does not silently heal after repair.

**Specific fix:** under an authoritative SH observation, route invalid/unreadable/nonregular/missing-initialized image results through the same owner-local fault-latching mechanism used by writer reads, preserving known facts and fencing old passes. Preserve the distinction from a read performed without SH while a writer is active; an unlocked intermediate observation must not be promoted to authoritative positive state.

**Required causal control:** extend the retained-owner first-observer matrix to malformed image, invalid marker, missing initialized image, and nonregular/unreadable image. Test both `is_denied` and lease acquisition as the first observer, then restore exact bytes and require the same owner to remain faulted. A genuinely fresh owner may inspect the repaired image.

This conclusion is derived from source; it was not executed.

**S4 — MUST_FIX / P2: Publishing cleanup callbacks run under SH and can deadlock on ordinary refresh reentry**

Affected source:

- `source/host/src/extensions/mcp/skill_publish.rs:129–140`, `resync`.
- `source/host/src/extensions/mcp/skill_publish.rs:167–194`, `upload`, particularly lines 190–192.
- Reentrant refresh waits at `source/host/src/extensions/mcp/plugin_skills.rs:877–885`.

`resync` now holds its admission lease throughout `upload`, releasing it only after `upload` returns. On staging-directory cleanup failure, `upload` invokes the public `with_log` callback and `with_edge_reporter` callback before returning. Those callbacks therefore run while the shared admission file lock is held.

A specific failure path requires no concurrent backend cycle:

1. Invoke `resync` through the real service and force the real upload cleanup operation to fail.
2. Supply a supported log or edge-report callback that calls the real Plugin Skills owner’s `sync`.
3. The reentrant `sync` reserves its worker and reaches `begin_refresh`, which obtains P and then blocks acquiring admission EX.
4. EX cannot be granted until `resync` drops SH. `resync` cannot drop SH until the callback and `upload` return.

The call self-deadlocks. This conflicts with AR-LOCK-R1’s callback/reporting boundary. It is distinct from holding the lease through the required backend mutation: the cleanup notification happens after the backend call and need not execute under the lease.

**Specific fix:** defer cleanup log/edge notifications until after the publishing operation releases its admission lease, preserving the original operation result and cleanup diagnostics. Keep the required lease coverage through the actual protected mutation.

**Required causal control:** a narrow real-cleanup I/O failure control whose reporter reenters the owner. Require completed return and verify the lease is no longer held when the reporter runs. Preserve the new backend-entry lease probes and post-operation refresh assertions.

The broader P→EX/backend→P concern is supporting lock-order analysis only. I do not report it as an additional proven defect.

**NON_BLOCKING — repeated full-image I/O on the workflow/prompt read path**

`is_denied` now opens/checks the lock file and loads/strictly decodes both custody files for every query. `filter_live_records` invokes it once per skill, and `FileWorkflowStore::plugin_skill_to_workflow` invokes it twice per projected record. Even a warm workflow parse-cache hit therefore performs repeated full admission-image reads and decoding.

For `S` skill rows and admission-image size `J`, this adds approximately `O(S × J)` image processing plus filesystem operations to list/get/prompt projection. Multiple skills from the same plugin repeat the same work. The existing bounded `StatKeyedParseCache` prevents unrelated workflow parse retention from growing without bound, but it does not reduce this new admission cost.

I did not measure runtime impact and therefore do not promote this to a performance blocker. A follow-up should consider an owner-controlled observation/batch query that obtains one fresh authoritative image per projection while retaining the current live refusal, exact-byte, writer-contention, and currentness rules. A naive metadata cache must not weaken AR-06 or the newly added loss detection.

I found no unbounded strong retention of process leases in the changed design. `LiveRefreshCustody` holds a `Weak`, while actual passes hold the strong process lease; the new terminal-worker and successor tests target that distinction appropriately.

**Scope coverage and read depth**

Every changed hunk in all twelve paths was read. The following are the exact actual-source blobs:

| Declared path | Actual blob | Review depth |
|---|---|---|
| `projects/openclaw-rust/docs/specs/plugin-skills-snapshot-admission.md` | `72d89cb1bff022ca28ad16985450bf21cb58a82f` | Normative behavior, interfaces, lifecycle, verification, AR-01–09, AR-CI and AR-LOCK-R1 |
| `projects/openclaw-rust/evidence/plugin-skills-admission-lock-647e0932-20261103/source-contract.json` | `19ea28aefe8f64b62a433e06e30c65a80917c3b0` | Complete contract |
| `projects/openclaw-rust/parity-ledger.json` | `7f64df2733279150ee9285d2237898033c35329f` | Every changed hunk, affected Skills responsibility row and completion flags; not a full ledger semantic review |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `5ddcb6dadee256edfd26678a7708455383465965` | Every changed hunk, affected row/status structure and completion flags |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `395231acc9ef1a41e716cc7ff11589dea70c0d6e` | Every changed hunk, case selection, source binding, command/capture exits, named positives, receipt aggregation |
| `source/host/src/extensions/mcp/plugin_skills.rs` | `8f382cda84834415f2fd02c3bc6dcffb9bc37417` | Complete current production owner and state machine |
| `source/host/src/extensions/mcp/plugin_skills_auth_recovery_tests.rs` | `88043d9a9d02064eec052a306e65b8e42ff400d4` | Every changed hunk, new tests, shared helpers and adjacent reservation/negative/release/successor controls; not every retained test line |
| `source/host/src/extensions/mcp/plugin_skills_uninstall_state.rs` | `0cbada753423020706cd8c9660be2b227be867e1` | Complete production image/parser/publication/lock implementation and failure instrumentation; retained tests reviewed selectively |
| `source/host/src/extensions/mcp/skill_publish.rs` | `ae2d1e8dc6a7ed3bf132f19d4347a3d214ac41ce` | Complete file, protected consumers, upload/restore/callback and artifact helpers |
| `source/host/src/extensions/session/production.rs` | `4c88b00b2dfe563ba957bd0669f177a422a330a7` | All changed hunks, forwarding owner, binding/open-store consumers and adjacent tests; not the unrelated large Session implementation |
| `source/host/src/workflows/workflow_store.rs` | `a7ada5f4c2b704d7599671694700e1f352ad03e1` | Changed function plus actual plugin list/get/update projections, caches and callers |
| `source/host/tests/mcp_service_contract.rs` | `e70044f6f765ba18a7a6cfa5588065ce53a26f76` | Every changed hunk, new process/publishing tests, actual consumer assertions and owned child request/receipt/lifecycle harness; not all unrelated MCP tests |

Adjacent current-source reading included:

- `plugin_skills_cache.rs`, blob `f395533056fd0203a68a51c51a412ff835925001`: eligibility, strict snapshot parsing and stage/publish/readback contract.
- `plugin_skills_sync_tests.rs`, blob `cb8abb868bef2469bded2dda87e0680e5c54b329`: relevant lifecycle, terminal settlement, integrity repair, callback and observation precedents.
- `host_production_extensions.rs`, blob `36b5dbac27d346f883150df179372034e0bea223`: actual binding, preparation and one-shot successor chain.
- `mcp/production.rs`, blob `fb672e8c3b9473607438a0db4fc733e205d5c59b`: concrete publishing client, Skills adapter, status and polling lifecycle.
- `mcp_service.rs`, blob `e903112e400783e70e9735b41745223a0f29f06c`: actual uninstall reservation, backend dispatch, settlement and refresh consumer.
- `watched_directory.rs`, blob `eb7bb0106966e9a8b5adb7205667ff81678dce80`: actual atomic replacement and asynchronous notification behavior.
- `stat_keyed_parse_cache.rs`: bounded cache/currentness precedent.
- `skill_publish_contract.rs`, blob `c6ea55bf02ad9eafaece2a547eec3c9d52487fde`: retained publishing port and artifact/control design.
- `source/host/Cargo.toml`, blob `e42cc9f56cc98337a49622ae9117ebd9823ef820`, and `Cargo.lock`, blob `9895ce7752768b1c8d3fe107ef95e513457ca518`: actual fs2 dependency/version binding.
- `.github/workflows/rust-desktop-runtime.yml`, blob `cfde0aa75688099b9ae9ec25efe528b71a82c406`: actual invocation of the existing portable capture owner; not a full workflow review.
- Project source-of-truth/module-map context and the current main migration specification §17.

For fs2, I inspected actual-use API and platform lock contracts from `danburkert/fs2-rs` using the supplied exact blobs: portable API `51b22db6acd0e964adf224e6a155b45660f7abf8`, Unix `91867f7e3f3b91c4edd24b3353464158cdc1fd8a`, and Windows `0e37c66579c0ca7eb4c8138f65d309285e438b4a`. The code uses the platform contention sentinel correctly, and shared `Arc` custody does not reacquire or explicitly unlock the same `File`. This was API/platform-contract reading, not full dependency or legal-closure acceptance.

**Reuse, capture and evidence assessment**

The implementation generally stays with the intended owners: the existing Plugin Skills live/durable owner supplies the lease, Session forwards it after releasing its slot, workflow replacement and publishing consume it, and the existing capture/harness mechanisms carry verification. I found no second marketplace, permission database, scheduler, or reconciliation loop introduced by this scope.

The new process control checks a completed foreign-owner refusal with zero loader calls; it does not treat timeout or an empty channel as exclusion. The cleanup unit uses actual OS lock probes around the real service hook, and the contract accurately distinguishes that same-process cleanup-window control from the separate foreign-process held-loader control. The publishing test probes EX before acquiring any probe-owned SH and uses the concrete Skills owner, so its lease contention is not explained by the probe itself.

The capture changes bind their explicit input sets before and after execution, retain command and capture exits, and require the new named positives. The preserved Box and LocalExec increments remain present in capture/ledger/RTM. I reviewed those shared-file changes as retained integration metadata and capture behavior; I do not infer acceptance of their unrelated production implementations. The inspected global completion flags remain false, and the Skills responsibility remains pending / not-run.

Exact-candidate GitHub Actions execution is still pending. Source review does not establish compiler success, test success, supported-platform lock behavior, registry-archive equivalence, recursive source/dependency closure, physical crash/power-loss behavior, real external accounts/ecosystems, packaging, or release acceptance.
