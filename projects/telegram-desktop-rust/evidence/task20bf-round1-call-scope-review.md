# CALL-SCOPE-001 frozen review and root decisions

Task `20bf9475-a885-4314-b509-bea53ef8849f`, round 1. The review is source-only and does not establish executable passage or product ACCEPT. No local validation is permitted.

## Frozen source reviewed

- Base mirror: `e2d64c2c53d9a91f3188b99203bce65e5d68f894`; only the `create_call_session` delta may be integrated into the latest live file.
- Production mirror Git blob: `66866934ddf95a1d3952be6ef004b251a20a4c8d`; SHA-256 `6cec1a6cb70087753c61370af8edef20afa8760161fab851b06fbc2ce8cfdaf1`.
- New test Git blob: `1314758393def1d1e2327dad39412e4636c6b3e7`; SHA-256 `d0c733600f9fc8be477181a98ba1eabf4904b9b4cfca566167b94d0a85961e48`.
- Task spec SHA-256: `de80e743a90488468254e18d1fb93c8a149cb11b45a95c8fb4b558d15e39850a`.
- Independent reviewer: `/root/current_gap_read/call_scope_review`, inherited model and xhigh effort; implementation and test files were held read-only after freeze.

## R1 — TEST_FLAW / P2: accepted fixture socket mode

The listener uses nonblocking accept, while the request reader assumes blocking reads with a finite timeout and treats `WouldBlock` as failure. Accepted-socket mode inheritance is platform-dependent. This can make a valid Native request fail according to TCP arrival timing on the macOS lane. Existing bounded Human/forward fixtures explicitly restore blocking mode.

**Root decision: FIX.** Before any review-driven edit, authorize exactly `stream.set_nonblocking(false)` on the accepted stream before request parsing, with an error mapped into the fixture result. Retain the accept deadline, per-request read deadline, frame limits, write timeout and bounded join. This repairs the oracle environment; it does not weaken assertions or alter production behavior. Re-freeze and independently re-review this correction before publication.

No other material compile, admission, persistence, oracle or lifetime issue was found within the reviewed scope by source inspection. The production guard compares a valid canonical stored pair with the normalized requested pair before UUID, HTTP or persistence; the non-Native early return and credential/remote response checks remain. The five authored tests use actual gateway, Native client and SQLite paths. Compilation and execution remain pending exact-head GitHub Actions.

## Publication and evidence limits

## R2/R3 — ledger source review and root decisions

Independent source/ledger reviewer `/root/call_source_scope` read the original draft SHA-256 `ca04aed9faf0f7f4d846771911ce4c56bf3ce655ae3a0f8bb704628cc96bf481`, the scoped source dossier/spec and canonical schema blob `e52b2ba92c8cf966bb506cc8b9f81320462474ae` at `1a2b864aee290db21548262c983033d184cab356`. Source-only; no schema execution.

- R2, P2, evidence semantics: generic “rejection has no remote request” incorrectly includes backend refusal after POST. **Root decision: FIX** the row to distinguish admission rejection (zero HTTP, zero insertion) from backend refusal (POST occurred, zero local insertion). No production behavior or oracle is weakened.
- R3, P2, traceability: file-only case IDs do not identify the five authored cases. **Root decision: FIX** by using exact file-plus-symbol identities in applicable case categories. Root had already prepared this narrow case mapping before receiving the finding; publish the mapped row and obtain a focused reread. Do not claim those cases executed.

The reviewer found no other material source identity, owner, schema-shape or acceptance-boundary discrepancy in the declared row. Preserve mapped/pending status, null execution fields, scoped source limits and all open product gates.

R2/R3 focused independent re-review: **SOURCE-ONLY APPROVED / FIXES VERIFIED** on corrected draft SHA-256 `12edc701223dc42e0dcf092fe39d170c9d414c6a4ad65c07c014270b597b8a14`. The reviewer checked the changed semantics, exact five test symbols and corresponding bodies; null execution fields and all broader acceptance limits remain.

R1 focused independent re-review: **SOURCE-ONLY APPROVED / FIX VERIFIED**. Test blob `ac113b2f6338efcf14767f192a735b900ec3d7ed`, SHA-256 `cb1db6130b0cc79619683aae88e7157bc7aaf9149c0cc6e3d58f40ac2c51d453`. The accepted stream is set blocking before parsing; deadlines, frame bounds, joins, five cases and all assertions are retained. Production guard identity is unchanged. The reviewer read the corrected source; freeze hashes were supplied by the implementation owner. No executable passage is inferred.

Reconcile the function delta against a freshly fetched live branch; preserve concurrent NMR49/50/51, CLI/package and runtime-source work. Root owns shared workflow, canonical ledger, RTM and source identity publication. The fixed upstream remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. This review grants no full-file read credit, whole-call/media acceptance, independent product ACCEPT or release approval.

## Integration workflow review

Publication reconciliation: live ledger advanced to `a73e18e1d1e44dfa28cb28755ce13812be9534cc` at `06decb74cede0830b49170cde98c2d92590d4293` while review was running. Root reread the complete remote blob; existing NMR/forward responsibility updates are retained by binding the preparation to this new input. Row count remains 184, schema identity and source reviews remain unchanged. Only the exact expected-input identity changes in the reviewed preparation algorithm; no earlier prepared full ledger is promoted.

Independent reviewer `/root/current_delivery_read` source-reviewed the temporary ledger-object workflow and the native-service workflow delta. **SOURCE-ONLY APPROVED; no blocking finding.** The preparation job binds checkout/input ledger `59d18f5de12fe6c4fe72bb110996c0cb0e07e9f4`/schema `e52b2ba92c8cf966bb506cc8b9f81320462474ae`, validates before/after, restores every prior value to prove the narrow mutation, and writes only an unattached Git blob plus receipt. It cannot move a ref. The product workflow delta adds one push path and three locked selectors (5 new + 3 existing Native + 1 exact local lifecycle); existing gates and always-upload evidence are preserved. Root must reread input identities at publication and re-review any semantic reconciliation.

## Final narrow source rebind

The independent integration reviewer approved the exact call guard and three-step workflow delta against parent `29c4f4eec8b5928f674d20107e041eee7c5a081b`; current canonical-owner/local-peer/credential/store helpers remain consistent. Its nonblocking observation of one extra trailing newline concerns scratch copies created for review. **Root decision: REJECT as a publication diff**: Git payloads are built directly from exact API-returned source plus the approved replacements, and do not include that scratch-only newline. No behavior fix is involved.

The live branch then advanced to `2d03120fe107b94f389409ac4f6890947940af4e` with the independent forward-ownership recovery batch. Root reread every overlapping file and reapplied the same guard, path and steps, preserving Human53, Chromium22 and the forward recovery changes. Ledger input remains `a73e18e1d1e44dfa28cb28755ce13812be9534cc`; prepared object `2ce3f0f2d9bf2a4439ba1443666e036eb9f5abf3` was read back in full and preserves all prior 184 rows and source reviews. Exact containing-H execution remains required.

Publication input changed through independent evidence updates. Preparation rerun `41520347187` at `19f631c5d74f342109859f8f1d509173b86305cc` successfully validated input `82cf93f16e7b4c5f7bc8482822977c92ad6ee18a` and produced `a0b3d7f0c1c043f5197375c25d47214841d7dee4`; the full remote object was reread with all prior rows/source reviews preserved. This is justified new-input validation, not retrying a failed oracle. Root refetched all overlapping publication paths at `63c59566aae7fa9b424a364df1429176c792d249`; approved call function/step deltas are unchanged, all concurrent content is retained, and the temporary workflow is removed by this integration.

## Publication isolation decision

Three expected-head attempts did not land on the shared checkpoint; immediate remote reads showed target advancement each time. **Root decision: use an isolated child branch on the same migration lineage**, `checkpoint/tdrp-native-call-scope-20bf9475-r1`, created at reviewed production commit `249675ae30502e72e495a4c6aee53e8214f82bd1` (parent810246). This preserves the shared branch and provides a stable exact-H verification subject. The spec now requires a PR back to the original checkpoint and fresh overlap/ledger reconciliation before integration. Native/source workflow selectors add only this exact branch, including the existing fixed-source-candidate job; all original branches, jobs and assertions remain. This is batch execution isolation, not source rebaseline, gate removal or release approval.

Independent source review `/root/current_delivery_read`: **APPROVED** the exact three selector lines against published249675. Both existing workflows retain their branch/path filters and exact github.sha checkout; the source candidate retains `needs: contract`. No wildcard, new token scope, assertion removal or acceptance promotion. Execution remains pending on the selector commit.

## Independent exact-H evidence verdict

Reviewer `/root/current_delivery_read` independently reread exact H `8bcfd464691534acaa04b13852cd523ab49a3dcc`, complete reconciliation logs, run/job/artifact metadata, spec and executed fixture. **Direct positive evidence supports bounded AC-CALL-SCOPE-01 and AC-CALL-SCOPE-02; no discrepancy found.** Run41521134666/job125537147556 passed5 new +3 Native +1 exact local call cases, plusHuman54/upload3/sharebox1/journal12/Agent4/private3/HTTP1. Artifact13177343396 digest `sha256:3b65ad1fde0b105aee7120483dd5ea7fc7f252d31ae2df8c90bc40d5d34a9001` agrees between upload log/server metadata; ZIP was not independently rehashed. Source run41521134644 passed157 source/37 schema and consistent unaccepted185-row candidate. Limits: controlled macOS loopback; logical SQLite preservation/in-process reopen; cached-owner malformed admission; no deployed-service/media/device, process-kill or complete product proof. Shippingjob125537147935 queued. No full batch/product ACCEPT, merge or release granted.
