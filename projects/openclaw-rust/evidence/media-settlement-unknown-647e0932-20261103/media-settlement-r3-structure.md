# R3 focused integration review — structure, reuse and performance

**Terminal verdict: APPROVED_SOURCE_ONLY.** No blocking structure, reuse or performance finding was established in the frozen five-path integration delta. This verdict is limited to source integration. It is not a build, test, schema, runtime, full MSU acceptance or final migration verdict.

## Reviewer and frozen authority

- Reviewer configuration: **gpt-6.1-sol / xhigh**, as applied by the delegated review host. No separate CLI worker, model-setting mutation or further delegation was used. The selected effort is appropriate because the shared App/CI/canonical paths compose concurrent changes around typed persistence uncertainty, cleanup authority and reentrant consumers.
- Task: `647e0932-bea4-4113-9a61-65ae3db58955`, implementation round 1, review round 3, repository `bhrumom/fabushi-desktop`.
- Original frozen base: `d70f86f33b5e3c67545954b270373e1548809f4b`, tree `4c4c9cfd3f5067fef57dca530042d149a8e2fc5c`.
- Publication parent: `74f11d3d31c6245ac5886dc0181460b2d4349744`, tree `aca91e1cfbedeecb039d442fb3528caac89873a3`.
- Reviewed candidate tree: `4d3b9d9faa3dfa9bbeb4083bbc7c638283b14026`.
- R2 comparison tree: `9892b15758904c23f3888ba8af1b5e429f29b59f`.
- Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Complete 20-path patch SHA-256 independently read back: `1e6a5270acaa94e98d4289169d0438d00f45ff48d6a8798b4b17ed08f5aa004e`.
- Complete five-path R2-to-R3 integration patch SHA-256 independently read back: `1312852a835935efd010ef11a7355d5f4fc2bae121a1876b86f4ddf608309ceb`.

I reviewed the immutable `media-settlement-r3/`, the immutable R2 comparison and the supplied R3 context. Additional adjacent source was read through explicit `git show 74f11d3…:<path>` from the available object database. I did not read mutable `media-settlement-work`, another review, an adjudication, worker reports or historical report bodies. Historical evidence references were inspected only as names/blob identities, not as semantic evidence.

The root AGENTS blob read for policy is `e4682748087759b2d4bb2b3a47916b6148e8a6f2`. Exact74 tree enumeration exposed no applicable nested AGENTS under the reviewed paths. The unrelated nested TUI instructions do not govern this scope.

## Identity and integration checks

I computed Git blob identities for every one of the 20 files in the R3 snapshot; all 20 match their manifest entries. A recursive byte comparison of R2 and R3 identified exactly the declared five changed files. No file was added or removed from that 20-path snapshot.

I also compared all 20 snapshot blob identities with the exact74 Git objects. Only these three differ from74:

| Path | Exact74 blob | R3 blob |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md` | `bab905e889a472027a906b551a874081131cc85e` | `e89c27654614752fd4700e6506735bee3bde25a8` |
| `projects/openclaw-rust/parity-ledger.json` | `2f0d7d43f54c9abedebbe25f40f21acd20a19c30` | `99fd980c1f8bee47327efc49e59e0d5544dcf46f` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `d62c6816905709d9dfed939a3581633488d9921c` | `bd1a56a6990168c384bb553c269a3fc1b695a14a` |

Direct text comparison against those exact74 objects confirms that the unpublished delta is the MAC clarification about DB-listener timing plus two ledger and four RTM evidence-reference replacements. App, workflow, LFA test and all other core source blobs already belong to74. The incoming cron implementation was retained; no second version was manufactured.

The candidate tree object itself is not available in the local Git object database. Consequently, the whole-tree parentage and preservation of paths outside the snapshot are taken from the manifest's explicit non-truncated remote tree readback. The 20 snapshot identities and their relationship to exact74 were independently checked by the reviewer. This distinction does not turn a local snapshot comparison into independent reconstruction of the entire candidate tree.

## Read depth and disposition

The complete 418-line five-path integration patch was read. Every changed hunk was judged with its required adjacent owner or consumer. The complete 5,001-line original-base patch was available and indexed for retained scope; I did not claim a fresh line-by-line deep read of all original MSU changes or of the entire large App/workflow/JSON files.

| Surface | Actual reading and disposition |
| --- | --- |
| Root policy and applicable specs | Root AGENTS read; MSU-v1 and MAC-v1 specs read fully; exact74 cron-identity spec read fully. Current OCRM source-of-truth/placement guidance, execution entry and §17 risk/ownership/execution provisions were consulted selectively. Historical project chronology was not treated as current authority. |
| Five integration files | Full delta; App list/trigger functions and production wrappers, registration and native dispatch references; workflow input capture, feature observations and strict App selection loop; affected ledger/RTM references and status/ownership rows; full LFA idle helper and import change. Large unchanged file bodies were not globally deep-read. |
| Adjacent cron test and owner | Exact74 cron test file read fully, including fixture lifecycle, native creation, real Runtime dispatch, ambiguity/no-effects controls, reopen/delete and retained outcomes. Existing stable-ID function read fully. AutomationRuntime/RunPath/FileAutomationStore interfaces and relevant list/history/native dispatch boundaries were inspected selectively through exact74 objects. |
| Retained MSU runtime | App pending-media state and lock helpers, attachment resolution, abort, send settlement and publication path read; attachment custody transitions/Drop read; Session typed append and first-open/cached-owner recovery boundary, Runtime media append, Tool construction-error classification, DB media entry/command/completion/fault boundaries read selectively. Central unchanged DB admission/receipt guards and all regression test bodies were not freshly deep-read. |
| Retained build and evidence selection | Both Cargo manifests read; native capture required-name sets, 19 command groups, exact result assessment and input binding read; portable capture MSU support references inspected. Relevant App test names/fault observations and test registration were inspected. The two large source-contract test files and complete retained App test bodies were identity-dispositioned, not represented as fully reread. |
| Remaining original scope | All 20 identities checked; the 15 files outside the integration delta are byte-identical to R2. All 17 files outside the actual three-file publication delta are byte-identical to74. This is preservation evidence, not fresh acceptance of every original requirement. |

Exact adjacent context identities were checked against the supplied context identity record and exact74 tree: cron spec `988f36d855e3efffc5b5563006dddf926393fcbe`, cron tests `646d24ab3d27dede7bfcb4f5fe421585694c390b`, and transcript `session_runtime.rs` `bf7e3533d2691aaf204888fcdb64e1f548716c10`. The latter was consulted as adjacent runtime context, not counted as a changed file.

## Source conclusions

### Existing ownership remains singular

`main.rs:2969–3128` holds one list projection and one trigger adapter. The handlers at `main.rs:5417–5433` supply their existing `SandAgentSessionStore` and `self.call("runAgentAutomationNow", …)` port. The registered public dispatch at `main.rs:4467–4468` continues through those handlers. The prior in-method implementations are removed by the integration delta; the new production seams therefore replace their bodies rather than duplicating them.

Canonical job IDs reuse `automations::automation_id::stable_automation_id`, whose exact74 blob is `7565275096e3351ffd18a6e55b7591e89292c534`. The existing import and App UUID dependency already exist. The resolver keeps only call-local selected identities and the previous run ID. It introduces no durable alias registry, scheduler, permission owner, history schema, database or retry owner. The resolved Agent/local pair is passed to the existing native execution command and used again for history observation; the canonical ID is a protocol projection.

The full scan before dispatch is necessary to decide ambiguity across both canonical and legacy namespaces. The code counts each actual record once, refuses a second matching pair, propagates enumeration errors, and reserves version-5 UUID input from accidental legacy fallback. Existing Runtime/RunPath ownership of completion, interruption and retained uncertainty is preserved. The test port calls the actual Runtime with the pair produced by the real adapter; it does not substitute an independently expected pair as execution input.

### MSU custody and lock invariants are retained

The integration patch does not modify any MSU-owned function. The retained `main.rs:722–880` uses the existing `HashMap<String, Arc<Mutex<PendingOutgoingMedia>>>` as the sole index. The map guard is released when `pending_media_entry` returns its Arc, before the entry `try_lock` or Session/DB work. Busy entries return typed unknown; poisoned collection disarms cleanup. Entry identity remains retained through Committed/Aborted transitions, preventing an old Arc from coexisting with a replacement under the same ID.

`resolve_attachment_source`, `abort_attachment_custody` and `send_message` retain the same entry-state checks and generation identity. Distinct images can collect before seal; exact repeated sources use the cache. The entry is sealed before a possibly committing owner call. Only a confirmed ordinary refusal can restore rollback eligibility. Typed unknown, busy access, sealed unwind and abort/Drop do not gain deletion authority.

The retained typed chain is DB `AgentDbProjectionError` → Session nested result → Runtime typed result → App `ToolOutcomeUnknown`. The Tool build-error branch still excludes typed unknown from destructive abort. The DB media completion tail still retires the exact owner on uncertain COMMIT/rollback and publishes no successful result for that path. The R3 integration introduces no second settlement command adapter or raw-COMMIT owner.

The MAC publication clarification matches the inspected implementation: DB notifications occur after confirmed COMMIT while the App entry is still held; same-ID reentry fails at `try_lock`. On the successful return, local custody reaches Committed and the entry guard is dropped before channel dispatch, App transcript publication, ACK fulfillment and success activity. The amended spec no longer overstates the earlier ordering as applying to every DB notification.

### Incoming edits and strict selections survive composition

The workflow integration adds cron spec/test/ID-owner input hashes and five exact cron cases to the existing receipt. It appends these to `required`; the retained MSU8, LFA6 and SGF3 groups remain selected. The common loop requires an exact unique test listing and a one-passed/zero-failed/zero-ignored successful summary for each selected App case. Existing shipping compilation and the test build retain separate Cargo JSON observations requiring ordinary Host library artifacts with `test-support` absent and present respectively.

The retained native capture still selects 19 command groups, including the 13 real DB fault cases, five local custody controls, 27 transaction-source/commit controls and the complete SendMessage19 group. Its assessment rejects missing, duplicate, extra, failed, ignored, zero-test or incomplete positives. The incoming cron additions do not alter this capture or its source-contract oracle. These are source properties of the selection; this review did not execute them.

The LFA integration delta changes only `serde_json` import ordering. Its retained idle helper continues to assert empty durable work identity and paths, locks each retained entry with `try_lock`, accepts only Committed/Aborted, and requires no pending cleanup authority, owned media, work ID or source cache. The integration preserves the existing terminal-entry repair and its durable/file assertions.

The ledger/RTM retain the incoming cron references. The final-custody responsibility occurs once as a ledger row and remains attached to the Session/AgentDb truth owner. The separate media read-scope responsibility remains `implementation=not-started`, `wiring=not-wired`, `verification=not-run`, `acceptance=pending`; final custody remains partial/implementing/not-run/pending. All eight top-level completion flags in both canonical files remain false. The actual publication delta repairs evidence paths without promoting source wiring or historical reference existence into execution acceptance.

### Proportional structure and performance assessment

The cron extraction adds private same-file test seams at the existing App adapter boundary. It removes the old handler bodies and creates no wider build dependency or abstraction hierarchy. List retains its existing whole-owner enumeration and output allocation behavior. Trigger now performs a complete ambiguity scan instead of first-match dispatch; it retains one target tuple rather than materializing a second global registry. Recomputing the existing hash per enumerated record is bounded by that necessary scan. No polling, replay loop, background task or additional lifetime owner is introduced.

MSU per-entry serialization, compact terminal identity retention and source-cache clearing are unchanged by this delta. Long-lived tombstone capacity and complete runtime performance remain broader measurement questions; the inspected integration provides no concrete new regression scenario that would justify a blocker or speculative cleanup request.

## Findings and execution boundary

**Blocking findings: none. Non-blocking change requests: none.** The evidence supports approval of this frozen integration source, including the retained MSU contracts, incoming cron composition, canonical metadata and LFA repair.

No build, test, test listing, parser, schema check, generator, lint, benchmark or capture was run locally or on htch-runtime. Only source/Git/text/byte comparisons and hashing were performed. The candidate's current-H Actions results, named positive outcomes, fault application, feature-mode artifacts and complete logs were not established by this review. An existing74 run retains74 as its subject; this report does not assign it to tree `4d3b9d9…`, a later evidence carrier or a moving branch head. Any failures or fixes outside the frozen five-path delta require their own causal adjudication and corrective scope.

The remaining execution gates include the selected native groups, actual App MSU8/LFA6/SGF3 and cron5, ordinary feature-off shipping check, exact compiled feature observations and applicable source-authority/schema gates on their proper immutable subject. Live provider/channel/backend, arbitrary crash points, full read-scope equivalence, packaged platforms and final G01–G12/AC01–AC24 acceptance remain outside this source-only approval.

**Final terminal status: APPROVED_SOURCE_ONLY — frozen R3 structure/reuse/performance integration review complete.**
