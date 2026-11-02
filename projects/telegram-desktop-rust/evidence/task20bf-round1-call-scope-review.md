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

Reviewer `/root/current_delivery_read` independently reread exact H `8bcfd464691534acaa04b13852cd523ab49a3dcc`, complete reconciliation logs, run/job/artifact metadata, spec and executed fixture. **Direct positive evidence supports bounded AC-CALL-SCOPE-01 and AC-CALL-SCOPE-02; no discrepancy found.** Run41521134666/job125537147556 passed5 new +3 Native +1 exact local call cases, plusHuman54/upload3/sharebox1/journal12/Agent4/private3/HTTP1. Artifact13177343396 digest `sha256:3b65ad1fde0b105aee7120483dd5ea7fc7f252d31ae2df8c90bc40d5d34a9001` agrees between upload log/server metadata; ZIP was not independently rehashed. Source run41521134644 passed157 source/37 schema and consistent unaccepted185-row candidate. Limits: controlled Ubuntu24.04 loopback (runner attribution corrected from the full job log below); logical SQLite preservation/in-process reopen; cached-owner malformed admission; no deployed-service/media/device, process-kill or complete product proof. Shippingjob125537147935 queued. No full batch/product ACCEPT, merge or release granted.

## Evidence count correction

Root reread the complete exact8bcfd UI job125537148138 log and source blobs. Actual results are Transcript20, Human-cursor17, settlement5 and Chromium forward-dialog16, all zero failure/skips. The earlier review/rebinding prose mentioning Chromium22 is **not evidence for this H**. Root decision: **FIX the evidence description**, retain the actual16-case gate/test content, and do not import an older contract or fabricate additional execution. At8bcfd, coordinator-client blob `b35d5899c8de48bbeff8c5ba55c017371c905308` has the two-argument call surface; dialog test blob `da87497e6fcb12e3c306e137467f837eff63c4ba` has16 cases. Any later UI lifetime work must first reconcile the latest shared-line dispatch contract. This correction does not alter the direct5+3+1 call proof.

## Exact8bcfd runner correction

Root reread job125537147556's complete log: Operating System Ubuntu24.04.5, runner image ubuntu-24.04, checkout8bcfd464691534acaa04b13852cd523ab49a3dcc. The earlier macOS loopback description was incorrect. **FIX evidence attribution only**: Native call contracts are Ubuntu evidence; macOS remains a separate queued shipping job125537147935. No test or platform gate is removed.

## CALL-CRED-01–06 frozen implementation and independent source review

Spec committed at1b8a448055b22dfe326f20969d81160949bbab5d before this implementation. Worker owns only the Native call family and new credential tests; root owns eight fixed-Human production bindings. Frozen8bcfd-base blobs: Native20a9d5a1cbe2ac4a8f18671beb9b2fc99adb569d, Production9ca5209c8eed190a718b39a4fafb07c2d92a789f, test9ef2ce91a0ad29dd6515e3cb986a7748c4415da0. Distinct reviewer `/root/current_gap_read/call_credential_frozen_review` independently read all three, spec and pinned gateway/store APIs. **No concrete implementation or fixture defect found; no fix proposed.** Root retains the frozen changes. Six cases cover five-family wrong-account zero HTTP, same-snapshot signal sender, all six held-GET credential fields, stale ICE/discovery, and already-admitted create/event recovery. This is source review; no local or remote test execution is inferred.

The final-body-byte fixture uses bounded waits, blocking accepted streams, release-before-join cleanup and full SQLite call-table snapshots. The held GET starts with signal/cursor1 and unapplied media2, then restored replay/ring reaches cursor3 while signalSeq stays1. Admitted create/event recovery explicitly preserves the possible remote effect and avoids a second POST. Snapshot comparison does not implement epoch/ABA history or atomic revocation.

## Reconcile concurrent migration input without replacing owners

Pinned shared input00036927d789913f08062cd303475fc5c55d57f2 follows inspected6b8361efbecf6706d5d175d31ea8c5206a4a9ba9 with preparation-only changes. Shared Native2c86ddf9c9432334e862a062d72a2de8903368dc, Production7b2fd1bd349d6e810fe4af76875af49982d007ef, workflowe60ec6e6f2d5c9e489e8200fa1bda10b3a4f4c1d. Root reapplies only the reviewed call regions and declared workflow gates. Independent `/root/call_credential_source_review` approved the frozen merged Nativee654932a7cb572b5f66a7842888049f2bb3c4b36, Productionb2daaab48b10f11580534b11b5b423e0a5016f4d, workflow3bf0dd3b35b8672095595cc76a2b78c646b7bd87 and unchanged test9ef2ce91a0ad29dd6515e3cb986a7748c4415da0: **no concrete merge defect**. All shared message-ID/history/reaction/forward changes and direct-FFI15+ignored1, Human62, Transcript29, package-scope10 and promotion7 gate sources remain. New credential6 and retained call5+3+1 are selected. These source assertions are not execution results at the new containing H.

Canonical ledger preparation reads shared260c5dac9312c1cb4d456a2643c1f7313124adef against unchanged schemae52b2ba92c8cf966bb506cc8b9f81320462474ae, preserves all184 shared rows and six source_reviews, and adds the existing task's CALL-SCOPE responsibility with explicit prior8bcfd scope evidence versus new authored credential cases. No global unknown/unread or acceptance promotion; existing owner/RTM appendices are retained. Independent source/owner review of exact-U account/session guards approves this extension without a new owner/ADR. Full platform, service/media, temporal, security, performance, soak, upgrade/Bot and independent migration ACCEPT remain open.

## Source workflow integration finding CRED-MERGE-01

Independent reviewer found the isolated branch missing from the source workflow push selector although the candidate-job condition was expanded. Root decision before repair: **FIX** this integration defect by adding only `checkpoint/tdrp-native-call-scope-20bf9475-r1` to the existing push branch list. Preserve all existing branches, paths, exact-H checkout, contract dependency and assertions. Re-review both selectors before publication; no workflow gate is removed. The unreferenced tree7f06c297577e5bd9f37039e6c8436bac0db583d3 is superseded, not delivered.

Focused independent re-review verified CRED-MERGE-01: source workflow blob3f6adb6a405dd545944f0ac7d3e9584e3d93ad8b changes only the two authorized branch selectors; other paths, exact-H checkout, dependency and assertions remain unchanged. Ledger, owner/RTM/STATUS and receipt separation are source-only approved. No remaining concrete integration defect; containing-H execution remains required.

## Current receipt reconciliation CRED-EVIDENCE-02

Independent reviewer found stale top-level blocker/reconciliation fields saying the credential batch was still being authored or lacked execution proof, contradicting the new exact523 completed result. Root decision before correction: **FIX current-status wording**, retain older execution subjects as explicitly historical, and leave actual job/artifact identities, acceptance limits and queued shipping gates unchanged. No new execution or acceptance is inferred by this correction.

The same evidence correction also updates the current production chain from the compatibility create_human_call method name to the shipping create_human_call_for_user consumer. Root disposition: **FIX** the stale symbol description; implementation and wire contracts are unchanged.

Focused independent re-review confirmed CRED-EVIDENCE-02 and the current production symbol correction: only those descriptions changed; exact523 evidence and acceptance flags remain. The twelve-field qualified-ledger helper is source-approved with exact input/schema, original-document reconstruction and POST-only unattached blob behavior.

## Exact523 credential execution and bounded independent assessment

Run41525143801/job125550827148 completed success on Ubuntu24.04.5 at523c8ca4883e6eaa10d1d19a0e237f9d3964b3c6: actual credential6, scope5, shipping3, local1, Human62, upload3, sharebox1, journal12, Agent4, private settlement3 and HTTP1 passed with zero failures/ignored. Artifact13179882022 (59,527 bytes) upload/server digest agrees: sha256:8defdbefda95277677c14067bc3c7f0e946439d07edb91634fbe02eb23b5d77d. Independent `/root/call_credential_source_review` reread code/spec/oracles, complete logs and artifact metadata: **AC-CALL-CRED-01–03 supported in the bounded loopback scope**, no concrete defect. Logical SQLite/reopen is not process-kill/power-loss proof; snapshots are not atomic revocation/ABA; admitted remote effects are recovered rather than rolled back; ZIP not independently rehashed.

Source run41525143819 passed157 source/37 schema,0 errors and consistent185-row candidate; all acceptance flagsfalse. Current UI29/17/5/16, Electron, Windows publisher and retained FFI15+ignored1 gates passed. Separate macOS shipping125550827119 remains queued, so the overall Native run is not fully passed. Real service/platform/media/device/UI-a11y/performance/soak/upgrade/Bot/global ACCEPT remain open. Media-capture work is separately active under spec1094, not qualified by this run.

## Media source integration decision CAPTURE-SOURCE-01

Independent source/owner review approved the bounded capture responsibility and exact seven source tuples, with two wording corrections. Root decision before edits: **FIX** the dossier path prefix so Telegram/SourceFiles applies only to the five tdesktop paths, while tgcalls/lib_crl paths stay relative to their own pinned repositories. **FIX** any description of monotonic target checks to state a required adaptation until frozen code review and execution support implementation. Preserve all upstream callback/camera-restoration limits; no new owner, review completion or acceptance credit.

The new source tuples retain case-sensitive repository spellings from fixed-U .gitmodules and the pinned canonical inventory producer. Actual candidate membership must still be validated in Actions; no source gate is bypassed.

## Frozen media review CAPTURE-REVIEW-01

Classification: **IMPL_BUG (P2)**. Independent frozen review confirms Stop sharing releases display ownership before awaiting old-sender restoration/false metadata. Reconnect or same-call generation reconciliation can invalidate that old peer/generation, while the next configurePeer samples no display and skips screenShare=false. Existing CallSessionStore transition preserves media_capabilities, so remote/durable screenShare=true can remain despite physical release.

Root decision before edits: **FIX within the existing component**. Retain an ephemeral local stop obligation bound to the originating component/client/conversation lifetime and call ID; carry it across same-call peer/generation replacement, and reconcile false through the current call/generation only. Clear only the same obligation on successful current acknowledgment, terminal/scope disposal, or a newly adopted display. Do not infer local sharing ownership from remote/global mediaCapabilities and do not introduce a durable store, public schema, or new owner.

Add a decisive actual-component case: first establish acknowledged local sharing=true, hold Stop's camera-restoration replacement, trigger local reconnect/current-peer replacement, require stopped display plus current-generation screenShare=false, then settle the old operation without extra stale mutation. Cover same-call remote generation refresh as a second interleaving, preserve a newer call/share, and retain all original twelve cases. Source/test changes require focused independent re-review; execute only at the later containing H in Actions. An already dispatched remote command is not claimed rolled back.

## Frozen media review CAPTURE-REVIEW-02

Classification: **IMPL_BUG (P2)**. Independent reviewer traced an old Accept ICE rejection after same-call remote generation refresh. The refresh can close the prior peer and build a current-generation peer while keeping the component lifetime epoch. Rejection bypasses configurePeer's post-await success guard; the outer catch admits failure using only isCurrentLifetime and failCall can tear down the new peer/local tracks and fail callRef's newer generation. Existing End/unmount negatives change epoch and do not cover this supersession.

Root decision before edits: **FIX error admission at the existing operation boundaries**. Start, Accept and reconnect failures must be admitted only by the same still-current operation/call/generation/peer attempt that initiated the rejected asynchronous work, not a reusable component-lifetime predicate alone. Stale rejection must preserve newer media/call state and emit no fail or follow-on signaling/media command. Retain real current failure cleanup and error presentation. No new public API or owner.

Add a decisive actual-component regression retaining previous cases: hold old ICE during Accept, reconcile a newer generation and current peer with retained local media, reject the old ICE, and prove the new peer/tracks/call survive with zero stale fail mutation. Cover analogous obsolete operation rejection boundaries as needed; request focused independent re-review and run only in containing-H Actions. Do not weaken the successful late-result gates or hide a current operation's real failure.

Independent refinement: the supersession can occur in the **same generation** when polling starts a second configurePeer while Accept's ICE is held. A call-ID/generation check alone therefore does not close CAPTURE-REVIEW-02. The FIX must admit or cancel rejected asynchronous work using the exact peer/operation attempt as well as the current call/lifetime, and the new regression must include this same-generation replacement before rejecting the old ICE. Preserve the broader old-generation variant and current-failure positive.

## Media fixed-freeze independent source receipt

Distinct reviewer `/root/current_gap_read/call_media_frozen_review` reread durable decisions, changed functions/adjacent callers, both appended regressions and fixture support. Verdict: CAPTURE-REVIEW-01 and CAPTURE-REVIEW-02 resolved by source inspection; no additional concrete finding. No local executable validation or remote mutation occurred. The first663 lines of the final test file hash to original b349dab4d0aafd72be8bca9195a499bf6b7a29b9, retaining all12 original tests; two decisive top-level regressions bring the authored total to14. The fixture reconnect increments generation/resets signal sequence and resume preserves generation, matching pinned Host rules.

| Frozen subject | Git blob | SHA256 |
| --- | --- | --- |
| frontend/src/production/human-call-media.tsx | 45641d86e7c2c5adcd0fa7b70609f3a0e28bff78 | 1cfdaf59595a18ae0484c61347eb8138a79c5473571d4e7a2de0f0236fe9b768 |
| frontend/src/production/human-call-media.fixture.tsx | ad9c9f52fb467f8849a23f528f8857cb5fc1b1d7 | 875b43c8a94767968fa3738efa94475c42d9686f829b34975295cddf3554b98a |
| frontend/src/production/human-call-media.behavior.test.mjs | 31f7ad37d08623dc55534d1f99def9dc129a8623 | 47aa1c585f180236486b987de8fe4bad6664c4ae535f27fdcbb66d4adb773564 |
| desktop/human-call-media-test.html | d8aedc6cf7af015802b77d2445cb3b2a4ab96cec | 71009f5a479c4aacd689a9be9487580ae1c259896e25adc4c90a1e13d6c8f403 |
| desktop/human-call-media-test.ts | 41d68e59999432b7f31e046b4ea65d34b0a8dcb3 | 3dfccc5fa118dca0018c09214bc536e35aeaebf0e62ee6c9a014ceafd957f9a7 |

Code base523c8ca4883e6eaa10d1d19a0e237f9d3964b3c6; integration parenta7e03787dcfe7310c2e28bedcfa226c212eafe0b. The fixed spec paragraph narrows provenance under SOURCE01; CAPTURE requirements/ACs are unchanged, and its status now distinguishes reviewed implementation from pending execution. Final publication binds the actual blob. Code uses explicit stale-media stop logic, including media-producing promises, while awaitCurrentMedia admits ordinary success and rejection using the exact operation guard. Genuine current errors retain cleanup.

`/root/call_media_integration_review` separately read current AGENTS/FMEP/schema/validator and approved exact FMEP01dc632975eca100c9bdc80746efdc2b0b044a57 plus source-test3ef07bee2d0eb93126c26317d9007c2f4e6a4008 import from pinned sharedf3ee5d8874ab74b07c60c0751035843b24319deb. Only two imports and one actual authority-entrypoint negative/positive-control test are added; no original test/gate is weakened. The schemae52b2ba92c8cf966bb506cc8b9f81320462474ae and validatorad5d07125850d2050020f66f0e515e889ad8b8c7 are unchanged. The dedicated workflow builds the actual renderer/fixture with locked dependencies, runs actual Chromium, and requires14 pass with zero failure/cancellation/skip/todo; exact checkout and always-upload evidence retained. This is review approval, not execution. Existing Native workflow/input subjects remain unchanged by the UI batch.

Root will append the distinct capture responsibility into the existing canonical ledger with the actual containing execution subject after the media run, preserving all185 existing rows, six source reviews/counters and false acceptance. The containing-ledger candidate must validate all seven exact source tuples before batch qualification. Active-call rejoin after process restart, hardware/OS/WebRTC/TURN, supported packages and all final product gates remain open.
