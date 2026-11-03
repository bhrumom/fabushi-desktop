# R3 focused integration review — general, security and lifetime

## Terminal verdict

**APPROVED — source-only, for the frozen R3 integration scope.** No blocking defect was found in the five-path integration delta or the affected owner/caller/build/selection composition inspected below. This verdict does not approve runtime execution, full MSU implementation acceptance, the incoming cron capability as a whole, or the complete OCRM product. It does not incorporate any prior review approval.

Execution evidence status for this review: **NOT RUN / NOT ASSESSED**. No CI result is attributed to the candidate tree or a later evidence carrier.

## Assignment, model and independence

- Repository: `bhrumom/fabushi-desktop`.
- Task: `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, source review R3.
- Lens: fresh, blind, focused general/security/lifetime integration review.
- Model: configured model retained; this session identifies itself as **GPT-6 Astra Pro**. No model override was applied.
- Applied effort: **xhigh**, inherited as specified by the launch assignment. The scope includes uncertain durable outcomes, connection retirement, cleanup authority, same-ID concurrency and shared App/CI/metadata composition; these justify xhigh.
- No prior or other review reports, root adjudications, worker report files or history report files were read. Historical evidence paths were checked only as tree entries/blob identities. The mutable `media-settlement-work` directory was not read. No subagent was started.
- Review remained bound to R3 when the parent later reported unrelated CI activity and evidence-only branch advancement. Those results and later commits were not investigated or used to decide this review.

## Exact source identity

| Authority or subject | Identity |
| --- | --- |
| Frozen original base | Commit `d70f86f33b5e3c67545954b270373e1548809f4b`; tree `4c4c9cfd3f5067fef57dca530042d149a8e2fc5c` |
| Publication parent / starting HEAD | Commit `74f11d3d31c6245ac5886dc0181460b2d4349744`; tree `aca91e1cfbedeecb039d442fb3528caac89873a3` |
| Reviewed R3 candidate | Tree `4d3b9d9faa3dfa9bbeb4083bbc7c638283b14026` |
| Previous source snapshot | R2 tree `9892b15758904c23f3888ba8af1b5e429f29b59f` |
| Fixed upstream reference | Commit `1440bb1771372334bea23d7642bb52deb7db1845`; tree `90c8aab39972da11cfad220361781d49e1bc6700` |
| Repository instruction authority | `AGENTS.md`, blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2` |
| R3 manifest | `/workspace/scratch/7d27ea239237/media-settlement-r3-manifest.json`; SHA-256 `8b122af3809facc3757a172923d597ed6d06651fa6067997e1947462f17a99d4` |
| Complete frozen 20-path patch | `/workspace/scratch/7d27ea239237/media-settlement-r3-complete.patch`; SHA-256 `1e6a5270acaa94e98d4289169d0438d00f45ff48d6a8798b4b17ed08f5aa004e` |
| Five-path integration patch | `/workspace/scratch/7d27ea239237/media-settlement-r3-integration.patch`; SHA-256 `1312852a835935efd010ef11a7355d5f4fc2bae121a1876b86f4ddf608309ceb` |
| Source snapshot | `/workspace/scratch/7d27ea239237/media-settlement-r3/` |
| Prior source snapshot | `/workspace/scratch/7d27ea239237/media-settlement-r2/` |

The authority file and both patch hashes were checked. All 20 R3 source-file SHA-256 values matched the manifest. Direct byte comparisons against R2 found exactly the five declared changed paths; the other 15 core files were byte-identical. The five-path patch was read in full, including every addition, removal and context hunk.

The candidate root tree was also read directly through the GitHub Git-tree API. It shares `.github` tree `e9ec8e45d28aed61169aec04807a3f5dba2e184d` and `source` tree `8b97e7ddd337d8477b5399d145edf8d3fd8dd218` with the exact publication-parent tree. Every other root entry except `projects` also matches. Within `projects`, only the `openclaw-rust` tree differs. Within that project, only `docs`, `parity-ledger.json` and `requirements-traceability-matrix.json` differ. The candidate docs tree was compared with the parent's exact Git tree: both contain 71 file entries, and the sole changed file is `specs/final-assistant-media-custody.md`. Evidence, reviews, inventory, schemas, scripts and all other project subtrees retain their parent identities.

This independently confirms that the actual unpublished candidate delta from 74 consists of the three manifest-declared documentation/metadata files. The candidate tree was absent from the local object store, so an initial local Git diff could not resolve it; direct remote tree reads plus exact-parent blob/text comparisons supplied the source proof instead. No failed executable validation was converted into a pass.

## Honest read depth and full core disposition

“Deep” below describes the integration hunks and named affected functions, not an assertion that a large enclosing file was read in its entirety. All 20 identities remain in scope. The original full patch supplied context; this was a focused R3 integration review, not a new line-by-line review of every unchanged R2 test, unrelated workflow job or product subsystem.

| Core path | R3 blob | R3 disposition/read depth |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `673d6270d3c2a7f4b9e9f640acf88927f2f7b154` | All integration hunks; deep review of affected App inputs, receipts, build/feature observations, required selections and completion checks around lines 3944–4375; complete-patch native/portable selection context. Other jobs not reread. |
| `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md` | `e89c27654614752fd4700e6506735bee3bde25a8` | Applicable custody requirements, correction and historical/evidence limits read; direct exact-parent clarification diff reviewed. Byte-identical to R2. |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `e42ea96e12c4647c23d98154fabd4a87749ea5c7` | Full MSU-v1 contract read, including MSU-01–10 and execution/acceptance limits. Byte-identical to R2. |
| `projects/openclaw-rust/parity-ledger.json` | `99fd980c1f8bee47327efc49e59e0d5544dcf46f` | All frozen changed rows, incoming cron references and exact-parent reference correction reviewed. Unchanged ledger rows not deep-read. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `bd1a56a6990168c384bb553c269a3fc1b695a14a` | All frozen changed mappings, incoming cron references and four exact-parent reference replacements reviewed. Unchanged RTM rows not deep-read. |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | `d5376d94f569b112c5778efb529eb64d2b863766` | Changed selection/input/receipt hunks and actual 19-command composition around lines 490–560 read. Byte-identical to R2; no capture executed. |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `172485b8437151256fb5058fcec010f9a0665cc8` | Changed MSU source-oracle/tool selections, scope and count expectations read. Byte-identical to R2; unrelated capture logic not reread. |
| `source/host/Cargo.toml` | `e42cc9f56cc98337a49622ae9117ebd9823ef820` | Full manifest read; default-off feature and dependency contract reviewed. Byte-identical to R2. |
| `source/host/app/Cargo.toml` | `a2c58167a82657194b79aaf3e1e0d55577b2221c` | Full manifest read; normal dependency versus dev-only test-support dependency reviewed. Byte-identical to R2. |
| `source/host/app/src/main.rs` | `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df` | All integration hunks; deep reads of pending-media state/locking/resolve/send/abort, incoming cron helpers, Gateway dispatch, native run consumer and test registration. Principal ranges: 700–1274, 2966–3122, 4457–4474, 5417–5433, 14086–14141 and 18109–18128. Unrelated App body not reread. |
| `source/host/app/src/tests/channel_host_lane_tests.rs` | `f6139af6bde9829f7e9147e6a2028ec77680bf42` | MSU module registration, SGF name bindings and affected shared Responses fixture slices read. Byte-identical to R2; complete original SGF implementation not rereviewed. |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `57a0a3393f1bb9560eb036687688656b1f3fe6c8` | Entire R3 delta plus fixture/observation/idle-custody/reopen ownership through line 343 and all six test declarations inspected. Published focus-call correction checked against its actual owner. Remaining test bodies retained by exact identity. |
| `source/host/app/src/tests/media_settlement_unknown_tests.rs` | `f525710de5833559de608e1cb8aaa89a8d58e2b8` | Eight test declarations and selected actual refusal/lost-ACK/rollback, same-ID, Drop/reopen, busy, poison/unwind and generated-child controls read, notably 300–485, 616–724 and 960–1035. Byte-identical to R2; not all fixture/helper bodies reread. |
| `source/host/src/attachment_paths.rs` | `6e1a2b8eb5752e005bbce30ed0b6eaaacfda0cda` | Deep read of custody state transitions and Drop, lines 21–104; retained context for owned paths. Byte-identical to R2. |
| `source/host/src/extensions/session/agent_db.rs` | `6c27e9bb9f665b1160457b9acc29dd5d64fba289` | Deep media transaction/typed outcomes, exact retirement, gated fault lease and scoped authorizer reads: 878–931, 4576–4585, 4718–5104 and 6243–6341. Thirteen named fault-test declarations inspected; complete DB/test file not reread. Byte-identical to R2. |
| `source/host/src/extensions/session/production.rs` | `c5912cab4f95d3a75fc13cb264a21b1deef9a733` | Deep reads of cached/fresh owner opening, media recovery, filesystem lock boundary and nested typed result: 1655–1840, 2291–2313 and 4063–4084. Byte-identical to R2. |
| `source/host/src/extensions/transcript/production_runtime.rs` | `9f15ddbdebe0ec7fe612975e28e9a7d5d0c3beb6` | Media append and adjacent compatibility path around 1770–1837 read for pre-operation error mapping and typed propagation. Byte-identical to R2. |
| `source/host/src/runner/tools/send_message_tool.rs` | `86a9e0e394b0145173bc033ceb757b7fb10835e5` | Changed construction-error branch and adjacent interaction/send handling around 350–453 read. Byte-identical to R2. |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `ba7dde9c12eb14cb0a0615a49a013a2e126b860d` | Identity preservation, frozen change locations, named media/channel/raw-COMMIT declarations and their capture selections inspected. Source-oracle implementation was not re-proved or executed in this integration round. |
| `source/host/tests/send_message_tool_contract.rs` | `3027419ca36d652ae2ab8b2c1fff16197903fa6c` | Deep read of prepared-custody known/unknown error controls around 569–688; remaining original tests retained. Byte-identical to R2. |

Additional exact-parent sources were read where they decide integration:

- Cron spec `988f36d855e3efffc5b5563006dddf926393fcbe` and cron tests `646d24ab3d27dede7bfcb4f5fe421585694c390b`: full requirement/test-boundary review, including all five incoming tests and real owner fixture lifetime. These were read by exact blob and from the matching frozen context files.
- `extensions/transcript/session_runtime.rs`, `bf7e3533d2691aaf204888fcdb64e1f548716c10`: `set_window_focused` at lines 315–344, verifying the published LFA caller's receiver, sessions argument and timestamp contract.
- `automations/automation_id.rs`, `7565275096e3351ffd18a6e55b7591e89292c534`: full stable mapper; existing Agent/local-ID hashing and version-5 UUID-shaped namespace.
- `extensions/session/agent_session.rs`, `0a54da8349bc3edc9ef21c5f1c1862e13a9e3039`: actual Agent/automation enumeration facades.
- `extensions/transcript/automation_runtime.rs`, `044f53c8eb94924168cc7dd953f3732cd70fa668`: actual get/run-now owner boundaries; `automations/automation_store.rs`, `57f39e545e525a36de3c7e9830970623862ee1fd`: current run ordering/read contract.
- `host_gateway_api.rs`, `127b340f120f931af778783aabcf338869fe0bc8`, and `gateway_server.rs`, `2cc83cf387ae37c3b1c28663f72d9d0492e84ccd`: registered cron ownership and relevant authentication/operator/write-scope admission slices.
- `runner/tools/mcp_meta_tools.rs`, `923288198b898083cbfe5da9a1f76e9f309ee516`: unknown result is recorded as `DownstreamOutcomeUnknown` before the worker receipt completes; `runner/stream_attempt.rs`, `6eb8a3eae5556eb6d6a6c03d00059d7c1180190f`: typed unknown custody declaration.
- Applicable authority context: root AGENTS fully read; current-entry portions of `SOURCE_OF_TRUTH.md` (`3748d734b10c9ed5b2d948467fbaa9483f1dd306`), relevant module-map owner rows (`15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3`), and OCRM-EXEC-02 §17 (`69c9667af2328e20a1fba09ebf80c5aab4d5c336`). Large historical/supporting sections were not used as current acceptance evidence.

The unchanged adjacent owners' blobs were matched against the exact parent tree before immutable object reads. No new fixed-U semantic-completeness claim is made by this focused target-integration review.

## Findings and source-only approval rationale

**Blocking findings: none. Non-blocking defect findings: none.** No speculative cleanup or unrelated product acceptance concern is promoted into an R3 finding.

### 1. Shared App composition preserves the media owner chain

The App integration hunks add/extract the cron list/trigger functions and add their test module. They do not modify `ProductionSendMessageSink` or the retained Session/Runtime/DB/custody implementations. Direct R2/R3 comparisons independently preserve every subordinate media owner and test/capture file outside the declared five paths.

The inspected media chain still has the required boundaries:

- The one map stores `Arc<Mutex<PendingOutgoingMedia>>`; its lock is released before entry `try_lock`. Busy/poisoned entries refuse through typed unknown. Collecting resolution caches exact sources, terminal entries remain indexed, and abort does not remove or recreate an identity (`main.rs:795–895,1004–1092`).
- Cleanup authority is sealed before the possibly committing append. `TranscriptCommitUnknown` is matched as a typed variant; it returns `ToolOutcomeUnknown` while retaining the entry. Known refusal alone authorizes the pending-to-cleanup transition. An invalid successful result shape remains unknown (`main.rs:1115–1224`; `attachment_paths.rs:56–104`).
- Session preserves the inner `AgentDbProjectionError` through its existing outer filesystem-result boundary. Runtime maps only its pre-operation errors and propagates the media append error (`production.rs:4063–4084`; `production_runtime.rs:1795–1837`).
- Any attempted COMMIT error retires the exact connection and returns unknown. Failed rollback or remaining non-autocommit also retires it. Success notifications are reached only after confirmed COMMIT/autocommit (`agent_db.rs:4806–4850,5064–5104`).
- Fresh-owner recovery remains separate from cached reads and local abort/Drop (`production.rs:1666–1720,1752–1821`). The one-shot fault lease and scoped rollback authorizer retain their owner/token/lifetime boundaries; the authorizer guard is removed before connection retirement (`agent_db.rs:878–931,4718–4794,6243–6341`).
- The Tool construction-error branch avoids aborting unknown custody, and the actual guard records downstream uncertainty before receipt completion (`send_message_tool.rs:405–413`; exact-parent `mcp_meta_tools.rs:187–209`).

These are source observations, not a claim that fault injection, concurrency or recovery passed execution on R3.

### 2. MAC clarification matches the actual publication order

The exact-parent MAC change narrows “publication” to **App publication** and records the existing DB-listener ordering. This is accurate: the DB connection guard is dropped and DB listeners run after COMMIT at `agent_db.rs:5082–5091`, before the App receives the result and commits local custody at `main.rs:1219–1224`. The entry guard is still held during that DB callback phase, so same-ID reentry refuses promptly through `try_pending_media`; it cannot materialize, append again or acquire cleanup authority. App delivery/event/ACK consumers follow the local commit and entry-guard release (`main.rs:1225–1273`).

The clarification changes no executable behavior and does not claim DB notifications were moved. The corrected spec is byte-identical to R2, so the incoming cron composition has not erased it.

### 3. Incoming cron identity changes retain the existing native owner and authority

The new private helpers are reached by the existing Gateway dispatch and thin methods (`main.rs:4467–4468,5417–5433`). Both list and trigger use the same existing stable mapper for the exact Agent/local-routine pair. Trigger resolves all matches before the single `FnOnce` native run port, refuses multiple canonical/legacy matches, and prevents a stale version-5 canonical ID from falling back to another routine's local alias (`main.rs:3037–3090`). User-supplied `agentId` does not override the resolved pair.

The run port continues to call `runAgentAutomationNow` with the resolved **local** routine ID. That existing command consumes `agentId` and `automationId`, selects the configured Host/Runner lane, and enters the existing AutomationRuntime; the helper adds no scheduler, persisted alias map, retry loop, permission owner or media settlement path (`main.rs:14086–14141`). Dispatcher errors propagate before a success response. Observation uses the same resolved pair and preserves the existing before/after-history semantics.

The actual Gateway's authentication, operator restriction and mutating-command scope checks are unchanged and remain ahead of these registered methods. The helpers do not introduce another route that bypasses them. The five incoming tests use real file-backed Session/AutomationRuntime/FileStore owners, a controlled execution result, and cleanup of exclusively created fixture roots after owner shutdown. Their source-wiring assertions are explicitly supplementary. Full HTTP/App construction, transactional lookup-to-execution identity, concurrent run-result correlation and full upstream cron grants/wait behavior remain outside this narrow repair, as its spec and receipt state.

### 4. Published LFA correction is preserved verbatim

The candidate LFA test file's computed blob is `57a0a3393f1bb9560eb036687688656b1f3fe6c8`, the exact publication-parent blob. R2-to-R3 changes only the `serde_json::{json, Value}` import ordering. It does not rewrite the already-published focus-call correction: the fixture continues to call `session_runtime().set_window_focused(&deps.session_workers, true, 1_000.0)` (`local_media_file_url_tests.rs:199–202`), matching the actual method contract.

The retained idle oracle still demands no durable work ID or media paths, permits only Committed/Aborted entries, rejects busy or poisoned entries, and requires no active pending custody/admission/cache (`local_media_file_url_tests.rs:280–298`). It has not been weakened to treat unknown entries as idle. Original six case declarations remain selected.

### 5. Test selection and build contracts compose additively

The integration adds the cron input hashes, scope, five required exact tests and module registration. It does not remove any existing MSU/LFA/SGF selection or replace the media captures.

| Retained or incoming selection | Source-observed contract |
| --- | --- |
| Native media capture | 19 command groups remain in `capture_media_cleanup.py:518–538`. |
| Real DB media faults | 13 required named tests remain in `MEDIA_SETTLEMENT_NAMES`; corresponding DB declarations are present. |
| Local custody | 5 required named controls remain selected. |
| Transaction/source owner regressions | 27 retained required names remain under the `commit` selection, including media and channel oracle negatives. |
| SendMessage contract target | Whole target retained with 19 required positives; portable expected count remains 19 for that target. |
| Linux App MSU | 8 exact names remain at workflow lines 4324–4335. |
| Linux App LFA | 6 exact names remain at workflow lines 4305–4313. |
| Linux App SGF | 3 exact names remain at workflow lines 4300–4304. |
| Incoming cron | 5 exact names added at workflow lines 4314–4322; each is declared in the exact incoming cron test blob and the module is registered once. |

The required-name loop still checks exact listing identity, successful process exit and one-passed/zero-failed/zero-ignored completion before recording a case (`rust-desktop-runtime.yml:4338–4349`). These numbers describe source selection, not observed execution counts.

The normal App dependency remains feature-off and its dev-dependency enables Host `test-support`. The workflow still observes the actual Host ordinary-library compiler artifact separately for the shipping check and test build, binds the manifest/source identity, and rejects the wrong feature or cfg(test) profile (`Cargo.toml` files; workflow lines 4137–4170). The actual App test executable remains bound to its source and owned target directory. R3 has introduced no additional fault surface into the shipping feature contract.

### 6. Six reference fixes and incoming canonical metadata are retained without promotion

Direct comparisons with the exact parent blobs show two ledger replacements and four RTM replacements, with no other unpublished metadata edits:

- Ledger line 36791 now references `fixed-u-contract.md`; line 36792 references `current-owner-audit.md`.
- RTM lines 1399, 1556, 2922 and 3102 now reference `fixed-u-contract.md`, preserving the R23/R24/AC23/AC24 mappings.

The corrected target files exist in the unchanged evidence subtree: `fixed-u-contract.md` is blob `b143ea8d36f84c02793263bbadf39a2e138619bf`; `current-owner-audit.md` is blob `01b724729a97f33ad1ef63b937360022ee5e48b9`. Their contents were deliberately not read. Searches found the six corrected references and no residual old aliases in the two changed metadata files.

The R3-versus-R2 ledger/RTM integration hunks add the incoming cron references only. Its referenced spec/test identities are present; `source-contract.json` is the existing exact-parent blob `79b752dbc6aab3d4e2fd9612aa5aae89f5cbd24f`, checked for existence only. The media responsibility remains partial/implementing/not-run/pending, the separate read-scope responsibility remains incomplete, and no unrelated source-review or global completion field is changed by these direct deltas. The root-tree comparison independently preserves the entire evidence and inventory subtrees.

## Execution gaps and practical limits

No build, test, executable test listing, source-oracle parser, schema check, generator, lint, capture, htch validation or CI dispatch was performed. Inspection used source text, read-only Git/API metadata, byte comparison and hashing only. No production file was edited; no commit, push, dispatch or message to external recipients was made.

This review supplies no new compilation, feature-mode, SQL fault, FFI, race, restart, provider/channel, platform-package or end-to-end evidence. In particular, the source-observed positive names and fault-applied assertions do not prove those checks ran. No H74 execution result, unrelated approval or later evidence-only commit is relabeled as an R3 result. Exact-subject Actions evidence and the governing acceptance process remain separate requirements.

The declared open boundaries remain open: legacy no-fence durable settlement, arbitrary crash points, broader read-scope/physical identity, assistant rewrite/detached-result semantics, complete cron protocol behavior, real external services and final OCRM gates. They are not new R3 integration findings and have not been treated as accepted.

**Final terminal decision: APPROVED for the frozen R3 source integration only; execution and broader product acceptance are not adjudicated by this report.**
