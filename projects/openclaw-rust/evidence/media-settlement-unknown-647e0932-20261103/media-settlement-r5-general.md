# R5 general, security, lifetime and oracle integration review

## Terminal verdict

**NO FINDINGS — source-only review completed.**

I identified no concrete, falsifiable defect in the frozen R5 integration or its affected consumers. This verdict does not establish compilation, parser/test execution, real fault application, runtime verification, source-accounting gate execution, or product acceptance. No R5 runtime result was supplied or produced by this review.

Task `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, review round 5. Lens: general/security/lifetime/oracle integration. Fresh blind review; no prior or other reviewer report, worker diagnosis, root adjudication/decision record, collector summary, or mutable `media-settlement-work` was read.

Model provenance: configured and inherited **GPT-6 Astra Pro**, with no model override. The assigned scope selected **xhigh** because it combines transaction-oracle composition, uncertain-outcome custody, test fault lifetimes and evidence ownership. The worker inherited its session setting; the host exposes no independent effective-effort introspection to this worker. No separate effort override or CLI/model replacement is claimed.

## Frozen identities and verified boundaries

Repository: `bhrumom/fabushi-desktop`.

Branch: `codex/openclaw-rust-fusion-chat-20261101-writer`.

| Subject | Exact identity |
| --- | --- |
| Original base commit | `d70f86f33b5e3c67545954b270373e1548809f4b` |
| Original base tree | `4c4c9cfd3f5067fef57dca530042d149a8e2fc5c` |
| Starting publication parent | `14faff5d9605c99df1494303f1aa0ce31bcfd420` |
| Starting publication parent tree | `518e019acc61861af909fcfd48a5c103a7ac7fe3` |
| Prior review tree | `46797d50cf4bd6841e78c6f4e1ba740fdff26102` |
| Frozen R5 candidate tree | `fc752f889ce0d402f6f5af9911d9777c7cab7999` |
| Fixed OpenClaw U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed OpenClaw U tree | `90c8aab39972da11cfad220361781d49e1bc6700` |

The review used `/workspace/scratch/7d27ea239237/media-settlement-r5/`, the supplied original-base, prior-round and exact-parent snapshots, and the current R5 inventory manifest. No later live head was substituted.

The following SHA-256 values were independently read from the supplied immutable artifacts and match the assignment:

| Artifact | SHA-256 |
| --- | --- |
| `media-settlement-r5-manifest.json` | `1a33405b8920f79a0f2967e0caa9aa00cfd8dcb0c0f25cb52cdd31a4bbe272f6` |
| `media-settlement-r5-full.patch` | `4f8dd714c38ee73d7e720f360a6d4490f01cc09cb0e494b0ec1cb2a71b9972b3` |
| `media-settlement-r5-fix.patch` | `c8834ebfa123a5adf3d6d9535e271cc7907dea3a95b4776fa6069a0226a1bad0` |
| `media-settlement-r5-publication.patch` | `34d5daa754d98cd20ad36b2dab321757d89c8b92980bfe5041ed443e946576a0` |
| R5 `inventory-shard-manifest.json` | `9b4dc21b09d80a94259d0688b552c920f41fb93363589235ba4d14a55470b6d3` |

All 21 candidate file SHA-256 identities match the frozen manifest. Non-writing `git hash-object` reads also match all 21 declared candidate Git blob identities listed below. A byte comparison of the supplied parent and candidate snapshots reports exactly the three declared publication changes: the MAC specification, the MSU specification, and `openclaw_sessions_and_config_contract.rs`. In particular, DB, inventory index and parity ledger bytes are unchanged from the exact publication parent.

The complete original delta and focused prior-round delta were read. Candidate tree membership and preservation of all other repository leaves are bound by the supplied manifest's exact remote-tree readback; this worker did not reconstruct the full repository tree locally. An exact-parent GitHub source read independently returned `source/host/tests/openclaw_chat_cron_node_contract.rs` blob `2bde1d3f9ab87f53e00079140a414ad492ac2ab6`, matching the stated incoming boundary. Its preservation in the candidate is also part of that manifest readback.

## Findings

None. No rejected or speculative concern is promoted into a finding, and no implementation or specification edit is requested by this review.

## Source conclusions

### 1. The media exception remains bounded and composes with the channel proof

Deeply read `source/host/tests/openclaw_sessions_and_config_contract.rs` around lines 1539–2804, including the complete media implementation and the original channel recognizer and negative controls. Also read the admission recognizer and its eight-owner table around lines 723–975.

`media_commit_inherent_impl` excludes trait implementations only when selecting an inherent owner. The complete source still flows to the global command checks. The actual `Drop for SandAgentDb` therefore cannot become a second inherent owner, and its body is not exempted. Exact method removal is checked by reparsing both images and requiring the one actual method to disappear from the one actual helper implementation.

`media_commit_owner_is_handled` pins the public typed signature, initial unknown/retirement handling, directly invoked synchronous write closure, three media admission checks, and the returned settlement tail. Its tail requires attempted-COMMIT errors and unconfirmed rollback to remain unknown and retire the connection. It also preserves release of the connection guard before successful DB notifications. The media-only source transformation removes only the proven COMMIT method and replaces the one already-proven media call in an oracle-local image.

The original channel method and returned tail are still pinned independently. The channel global argument-taking COMMIT caller rule remains `Some(1)`; it was not raised to accommodate another owner. Its macro, alias, method-reference, raw-identifier, hidden-owner and discarded-tail controls remain present. The raw `db.execute_batch("COMMIT")?;` prohibition is applied after both bounded exceptions, and the new controls require extra raw owners to survive in the residual source.

### 2. The revised controls are bound to actual source sites and preserve line-ending semantics

`media_commit_oracle_fixture` first requires the actual source baseline to be accepted. `replace_media_commit_site` requires exactly one matching source site and a real replacement. The owner duplication fixture separately checks its unique signature and adjacent complete-method boundary. Each relevant textual mutation is therefore tied to an intended production site rather than silently becoming an unchanged negative fixture.

The actual-source positive requires the full existing Drop implementation. LF and CRLF images are compared for complete residual-source equality after normalization, both for the media-only proof and the composed proof. Harmless trait implementations remain in the residual source. Trait-only helper/owner replacements, extra ordinary and raw-identified aliases in trait bodies, macro cases, changed command handling, malformed/conditional owners and extra raw-COMMIT owners remain exercised by both-line-ending controls. Normal zero-argument RAII commit expressions retain their positive controls.

These are source-level judgments about the authored assertions. They are not claims that `syn` parsed the frozen candidate or that any assertion executed successfully in this review.

### 3. The oracle matches the actual DB owner and its consumers

Read the actual media owner in `agent_db.rs` around lines 4799–5104, the command adapter/authorizer around lines 6237–6341, the exact retirement helper at 4576, the getter at 5843 and the owner-row reader at 7777. The DB getter uses the owner parser and retains hidden settlement rows. The adopted `expected_committed_entries` helper and both fresh-owner assertions require the exact send plus receipt payload/order; independent SQL-backed assertions also check storage and the released fence. No presentation filtering was imposed on the canonical getter.

The production owner preserves the existing admission, receipt replay, transcript validation and fence-CAS logic in the same transaction. Successful COMMIT followed by lost acknowledgement and unsuccessful COMMIT are both classified as unknown by the owner; normal precommit errors require successful rollback and autocommit before returning an ordinary refusal. Unknown paths retire the exact connection and do not issue successful transcript/admission notifications.

The fault port remains confined to `cfg(any(test, feature = "test-support"))`. Its lease borrows the owner, uses allocation identity for token ownership, follows connection-before-slot locking, and releases the slot before SQL. The rollback authorizer guard holds its state allocation and connection borrow until callback removal. The real command adapters own the COMMIT and ROLLBACK calls; fault observations do not select the production outcome classification.

Read the affected Session append and existing filesystem/open/recovery boundaries, the Runtime media append and adjacent compatibility methods, the App pending-entry/custody and publication path, and the Tool build-error/result branches. Session's nested result preserves the inner typed DB error. The App seals cleanup before history allocation and the potentially committing call, retains unknown media, rejects busy/poisoned or terminal same-ID reuse, and releases the entry guard before its delivery/event/ACK consumers. The MAC clarification accurately distinguishes prior DB notifications from later App publication. The ordinary collection/abort and first-open recovery paths retain their own cleanup authority.

### 4. Capture and failure evidence remain explicit

Read the full new App and DB media test modules, the changed Tool controls, the affected LFA/SGF fixtures, and the command/result paths in both capture scripts and the workflow.

The native media capture retains **all 27 transaction test selections**: 19 existing transaction/channel/raw-COMMIT controls plus the eight media-oracle tests. It also declares the 13 DB media cases and five custody cases. The result assessor requires unique named positives, no unexpected positives, one matching all-passed/zero-failed/zero-ignored summary, and successful command and capture exits. Logs, command identities and failure states are retained in the receipt; failed cases cannot become success because subsequent cases run.

The App workflow still selects the eight real App/Session/Runner cases, separately observes the feature-off shipping Host library and feature-on ordinary Host dependency library for the App test build, retains compiler JSON and command exits, and saves a failed/incomplete receipt on an exception. Existing failed native/App fixture preservation and `always()` artifact uploads remain present. The generated Runner case checks the real claimed/applied fault, zero sends/retries/success hooks, retained ACK/media, canonical-history preservation and two fresh owners, while keeping controlled-provider limitations explicit. These authored controls remain unexecuted for R5.

### 5. Incoming materialization is preserved without promotion

The current R5 inventory manifest identifies canonical inventory SHA-256 `d0049087bdbcc8eecdc8d0a69bd48bfc7d70688f8917122198a6e15c80863d8a`. The review manifest binds incoming subject `fc187ecba404e2fa95d65871301aa8d83c7f0314`, carrier `14faff5d9605c99df1494303f1aa0ce31bcfd420`, Actions run `41582614099`, attempt 1 and artifact `13206770678`.

The R5 index and ledger are byte-identical to the supplied carrier snapshot. The prior-round delta shows only the materialized mapping-count change and removal of the consumed declaration. The index retains unread `62067`, false source/semantic closure, and fixed U; ledger/RTM global completion flags remain false and the media responsibility remains partial/implementing/not-run/pending. The pending-addition clarification matches the unchanged source gate's explicit digest-bound declaration, exact missing-link check and materializer consumption. Historical R4 selected rows and manifest were not used as current materialization authority. No generated shard was locally rewritten or validated, and no new source-reading, gate or runtime claim was inferred.

## Complete 21-path identity and read-depth disposition

Every row below received identity comparison and complete declared-diff inspection. “Deep affected scope” means the changed logic and relevant whole functions/adjacent consumers were read; it does not mean unrelated giant files were reread end-to-end.

| Path | Candidate Git blob | Read-depth disposition |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `673d6270d3c2a7f4b9e9f640acf88927f2f7b154` | Deep affected capture, feature observation, selection, failure and upload paths; incoming cron hunks contextual. |
| `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md` | `e89c27654614752fd4700e6506735bee3bde25a8` | Full document, including full MSU correction and historical-status distinction. |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `4caeb8ca80252351acc8b59b2528d1c75b9e8d44` | Full authoritative task specification, MSU-01 through MSU-10 and exit contract. |
| `projects/openclaw-rust/inventory/index.json` | `aa5ef87202903d9ba344a4e1e7fbe6ffedb04772` | Full index, exact-parent preservation and current manifest identity. |
| `projects/openclaw-rust/parity-ledger.json` | `673c8defe212bc07d0bc031d9a10126c2df7ef24` | All changed hunks, complete added media responsibility and top-level flags/counts; unrelated ledger rows not deeply reread. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `bd1a56a6990168c384bb553c269a3fc1b695a14a` | All changed requirement/acceptance links and top-level flags; unrelated rows contextual. |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | `d5376d94f569b112c5778efb529eb64d2b863766` | Deep changed selectors/inputs plus result qualification, command, receipt and failure consumers. |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `172485b8437151256fb5058fcec010f9a0665cc8` | Deep affected inputs/selections, scope statement and result/receipt consumers. |
| `source/host/Cargo.toml` | `e42cc9f56cc98337a49622ae9117ebd9823ef820` | Full manifest and test parser dependency contract. |
| `source/host/app/Cargo.toml` | `a2c58167a82657194b79aaf3e1e0d55577b2221c` | Full manifest and dev-only feature wiring. |
| `source/host/app/src/main.rs` | `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df` | Deep media state/locking/cleanup/append/publication functions; complete incoming cron diff contextual. |
| `source/host/app/src/tests/channel_host_lane_tests.rs` | `f6139af6bde9829f7e9147e6a2028ec77680bf42` | Deep changed shared fixture, Responses and owned-child lifecycle; remaining test bodies contextual. |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `57a0a3393f1bb9560eb036687688656b1f3fe6c8` | Changed observation and idle assertions, preserving durable identity/path checks. |
| `source/host/app/src/tests/media_settlement_unknown_tests.rs` | `f525710de5833559de608e1cb8aaa89a8d58e2b8` | Full new module, including all eight App controls and actual generated Runner child. |
| `source/host/src/attachment_paths.rs` | `6e1a2b8eb5752e005bbce30ed0b6eaaacfda0cda` | Deep complete custody state machine, Drop and changed tests; callers traced. |
| `source/host/src/extensions/session/agent_db.rs` | `87186bc4b246d88a5ce51ebda0623b76f4425d3d` | Deep media owner, fault/FFI lifetime, retirement, getter, publication and full new DB test module. |
| `source/host/src/extensions/session/production.rs` | `c5912cab4f95d3a75fc13cb264a21b1deef9a733` | Deep changed typed append and recovery use, whole relevant open/recovery/filesystem boundaries. |
| `source/host/src/extensions/transcript/production_runtime.rs` | `9f15ddbdebe0ec7fe612975e28e9a7d5d0c3beb6` | Deep whole media append and adjacent admitted/compatibility append methods. |
| `source/host/src/runner/tools/send_message_tool.rs` | `86a9e0e394b0145173bc033ceb757b7fb10835e5` | Deep changed construction-error and adjacent success/error delivery branches. |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `c0162f99b896c00d0cd100ba59ad700eb01ad802` | Deep full focused media proof/tests, original channel proof/tests and admission recognizer; complete diff read. |
| `source/host/tests/send_message_tool_contract.rs` | `3027419ca36d652ae2ab8b2c1fff16197903fa6c` | Full added prepared-custody controls and surrounding declared diff. |

The supplied repository `AGENTS.md` was read. Additional source-only context included the unchanged inventory gate helpers needed to judge the new spec clarification, exact-parent project orientation material, and the exact-parent incoming cron boundary. No historical report or generated-row interpretation was substituted for current source.

## Execution and promotion limits

Only source, Git/API, diff, hash and data reads were used. No local or htch parser, test/list, build, lint, generator, schema/validator, capture or dispatch ran. No production, spec, ledger, index or snapshot file was modified. This report is the sole written artifact.

A later exact-candidate Actions execution remains necessary for parser/compile behavior, real fault application and all required positive results. Full migration acceptance, broader read-scope identity, live provider/channel/device and packaged-platform claims remain outside this source verdict and retain their existing pending status.
