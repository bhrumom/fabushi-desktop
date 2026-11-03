# R5 structure, reuse, performance and transaction-oracle integration review

Terminal verdict: **SOURCE-ONLY APPROVAL — zero findings** in this review lens. No blocking or non-blocking source defect was established. This is not an execution, source-authority gate, runtime, packaged-product or migration acceptance result. All execution-dependent actions below remain pending.

Task: `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, independent review R5. Repository: `bhrumom/fabushi-desktop`; branch: `codex/openclaw-rust-fusion-chat-20261101-writer`.

The accepted host launch configuration supplied for this worker was `model=gpt-6.1-sol`, `reasoning_effort=xhigh`, `fork_turns=none`. The applicable repository policy pins the structure/reuse/performance lens to that model. Xhigh is appropriate because this assignment judges a shared transaction oracle, unknown-outcome custody, integration with existing consumers and the separation of accounting from evidence. This records the accepted launch setting; it does not claim independent backend/model introspection. No additional agents or executable validation workers were started.

## Immutable binding and method

The reviewed source is the immutable snapshot `/workspace/scratch/7d27ea239237/media-settlement-r5/`, bound to candidate tree **`fc752f889ce0d402f6f5af9911d9777c7cab7999`**. No later live head was substituted.

| Authority | Exact identity |
| --- | --- |
| Original complete-scope base | commit `d70f86f33b5e3c67545954b270373e1548809f4b`, tree `4c4c9cfd3f5067fef57dca530042d149a8e2fc5c` |
| Starting publication parent | commit `14faff5d9605c99df1494303f1aa0ce31bcfd420`, tree `518e019acc61861af909fcfd48a5c103a7ac7fe3` |
| Prior round context | tree `46797d50cf4bd6841e78c6f4e1ba740fdff26102` |
| Fixed OpenClaw U | commit `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700` |
| Scope manifest SHA-256 | `1a33405b8920f79a0f2967e0caa9aa00cfd8dcb0c0f25cb52cdd31a4bbe272f6` |
| Complete 21-path patch SHA-256 | `4f8dd714c38ee73d7e720f360a6d4490f01cc09cb0e494b0ec1cb2a71b9972b3` |
| Focused prior-round delta SHA-256 | `c8834ebfa123a5adf3d6d9535e271cc7907dea3a95b4776fa6069a0226a1bad0` |
| Three-path publication delta SHA-256 | `34d5daa754d98cd20ad36b2dab321757d89c8b92980bfe5041ed443e946576a0` |

The four SHA-256 values were read back with `sha256sum`. All 21 candidate snapshot files were independently hashed with both SHA-256 and Git `hash-object`; every scoped value matched the manifest. A read-only directory diff against `media-settlement-r5-parent/` confirmed exactly the declared publication paths: the two media specifications and `source/host/tests/openclaw_sessions_and_config_contract.rs` (136 insertions, 81 deletions). The adopted DB, ledger and index are unchanged relative to that publication parent. These are scoped file/hash checks; I did not independently reconstruct an entire repository tree from the partial filesystem snapshot.

I read the applicable `media-settlement-authority/AGENTS.md` first, the complete MSU-v1 contract and the complete MAC-v1 specification including its current correction. I read all **5,148 lines of the complete declared diff**, all 281 lines of the focused delta, and the complete publication delta. Review remained blind to prior/other reviewer reports, diagnoses, collector summaries, runner adjudication and mutable work. Directory names observed during initial source discovery were not treated as review evidence, and none of the forbidden report contents was opened.

Only source/data reading, searches, diffs, Git/hash inspection and exact-ref GitHub source retrieval were performed. No parser, test, test listing, schema check, validator, generator, build, lint, capture, benchmark, local/htch validation or Actions dispatch was executed. The only write is this report.

## Exact scoped blobs and actual reading depth

“Complete diff” below means every changed hunk and its supplied context was read; it does not mean every byte of a large unchanged file was read. The large workflow, App main, AgentDb, Session, RTM and ledger are not claimed as full-file reads.

| Candidate path | Git blob | Reading depth |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `673d6270d3c2a7f4b9e9f640acf88927f2f7b154` | Complete diff; adjacent actual-App capture/feature artifact selection and required-test execution sections around 3920–4015, 4110–4240 and 4280–4415. |
| `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md` | `e89c27654614752fd4700e6506735bee3bde25a8` | Complete file, with MSU correction distinguished from historical source-wiring records. |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `4caeb8ca80252351acc8b59b2528d1c75b9e8d44` | Complete file, all MSU-01–10 requirements and verification/exit text. |
| `projects/openclaw-rust/inventory/index.json` | `aa5ef87202903d9ba344a4e1e7fbe6ffedb04772` | Complete file and both deltas. |
| `projects/openclaw-rust/parity-ledger.json` | `673c8defe212bc07d0bc031d9a10126c2df7ef24` | Complete diff; global completion/accounting fields, changed read-scope risk and complete final-custody responsibility. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `bd1a56a6990168c384bb553c269a3fc1b695a14a` | Complete diff of all changed evidence, blocker, requirement and responsibility links. |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | `d5376d94f569b112c5778efb529eb64d2b863766` | Complete file: selected names, inputs, result-oracle qualification, invocation/receipt and acceptance logic. |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `172485b8437151256fb5058fcec010f9a0665cc8` | Complete diff; adjacent required claim integration, generated SendMessage command/count and result acceptance logic (300–390, relevant command declarations, 1065–1115). |
| `source/host/Cargo.toml` | `e42cc9f56cc98337a49622ae9117ebd9823ef820` | Complete file. |
| `source/host/app/Cargo.toml` | `a2c58167a82657194b79aaf3e1e0d55577b2221c` | Complete file. |
| `source/host/app/src/main.rs` | `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df` | Complete diff; entire changed pending-media state/helper and SendMessage owner flow (719–1285), adjacent success publication/ACK, and production per-turn sink composition (11695–11775). All incoming cron helper/wrapper hunks read as bounded context. |
| `source/host/app/src/tests/channel_host_lane_tests.rs` | `f6139af6bde9829f7e9147e6a2028ec77680bf42` | Complete diff, module inclusion and reused Responses/process/wire fixture interfaces. |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `57a0a3393f1bb9560eb036687688656b1f3fe6c8` | Complete diff, retained durable idle assertions and terminal-entry replacement assertion. |
| `source/host/app/src/tests/media_settlement_unknown_tests.rs` | `f525710de5833559de608e1cb8aaa89a8d58e2b8` | Complete 1,035-line new test module. |
| `source/host/src/attachment_paths.rs` | `6e1a2b8eb5752e005bbce30ed0b6eaaacfda0cda` | Complete diff; complete changed custody methods and Drop, adjacent owner/root interface. |
| `source/host/src/extensions/session/agent_db.rs` | `87186bc4b246d88a5ce51ebda0623b76f4425d3d` | Complete diff; entire real media transaction including unchanged receipt/replay/write/CAS body (4799–5105), fault/command/FFI owners, exact shared retirement helper (4576 onward), DB getter and row reader (5843 onward; 7777 onward), all added private media tests, and original channel command/failure/transaction functions (6338–6650). |
| `source/host/src/extensions/session/production.rs` | `c5912cab4f95d3a75fc13cb264a21b1deef9a733` | Complete diff; transcript projection (41 onward; 1290–1320), cached/first-open DB/recovery owner (1640–1850), filesystem nested-result boundary (2285–2322), changed media facade and adjacent append owners (3990–4110). |
| `source/host/src/extensions/transcript/production_runtime.rs` | `9f15ddbdebe0ec7fe612975e28e9a7d5d0c3beb6` | Complete diff; adjacent generated send variants and complete typed media Runtime facade (1750–1850). |
| `source/host/src/runner/tools/send_message_tool.rs` | `86a9e0e394b0145173bc033ceb757b7fb10835e5` | Complete diff; counting/delegation (110–180), message build/materialization (247–306), actual Tool build-error/encode/selection/send boundary (360–458). |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `c0162f99b896c00d0cd100ba59ad700eb01ad802` | Complete diff; complete media proof and every positive/negative control; complete original channel proof, global residual assertion and its controls (1569–2820); adjacent owner-local admission recognizer (665–980) and file imports. |
| `source/host/tests/send_message_tool_contract.rs` | `3027419ca36d652ae2ab8b2c1fff16197903fa6c` | Complete diff and complete prepared-custody classification fixture/cases. |

Additional exact-parent source was retrieved through GitHub at commit `14faff5d9605c99df1494303f1aa0ce31bcfd420`, without a mutable branch lookup:

- `source/host/src/runner/tools/mcp_meta_tools.rs`, blob `923288198b898083cbfe5da9a1f76e9f309ee516`: the guard/tracker path around 160–240. It marks `DownstreamOutcomeUnknown` before receipt completion and result delivery.
- `source/host/src/runner/stream_attempt.rs`, blob `6eb8a3eae5556eb6d6a6c03d00059d7c1180190f`: custody/report/tracker definitions and retention/retry/checkpoint admission around 1–213 and 411–520. Unknown is sticky and blocks later effect admission/checkpoint acceptance.
- `source/host/tests/openclaw_chat_cron_node_contract.rs`, blob `2bde1d3f9ab87f53e00079140a414ad492ac2ab6`: the incoming cron projection/helper/wrapper assertions near 635–685 and relevant boundary references. Its exact blob matches the declared unchanged incoming component, and its helper/wrapper source assertions match the incoming App layout.

Fixed-U source directly read in full: `agent-run-final-media.ts` (blob `f4bf5b4308cb2a7160ad1db0142160d14fb71551`) and `agent-run-media.ts` (blob `27a3a65974fad078872b257ee28cafccdad4691f`). Their finalizer retains on canonical unknown and still requires broader rewrite, placement and source authority. Two more fixed-U file identities were hash checked only: `sqlite-worker-error.ts` (`b13f8354e7244d630ee274b3b0debf9a3f5bb8c6`) and `managed-image-attachments.custody.ts` (`1975104ea6cdcfea25d3bc4362edbaea5e673adf`). No full-reading or review completion is claimed for these hash-only files, `store.read-scope.ts`, their dependency closure or the complete upstream media responsibility.

## Integration assessment

### Media exception remains separately bounded

At contract lines 1693–1888, `media_commit_inherent_impl` excludes trait implementations from inherent-owner candidacy, but does not delete or exempt their source. It requires exactly one top-level, exact unqualified inherent implementation with no conditional/unsafe/generic owner shape. The method checker parses before and after removing the one exact literal method, then proves the actual named method vanished from the actual helper and that only one helper item was removed. A comment, string, nested function or foreign owner cannot earn that removal merely by containing matching text.

`media_commit_owner_is_handled` separately checks the exact public typed signature, complete entry/BEGIN/retirement prefix, directly invoked synchronous write closure and its three admission guard calls, and the complete returned settlement tail. The tail preserves attempted-COMMIT uncertainty, rollback/autocommit classification, exact connection retirement and success-only publication. The helper’s rollback is proven but remains in the residual source. Only the media commit method and the already-proven single `commands.commit(db)` call are replaced in the oracle-local image.

The composition order is media proof followed by the original channel proof. The original channel `Some(1)` argument-taking caller requirement is retained, as are its complete method/tail, malformed/ambiguous owner checks, alias/associated-reference checks, opaque macro inputs and raw-identifier normalization. The final global raw-COMMIT prohibition still inspects the complete residual source. Neither a whole impl exemption nor a raised global count was introduced.

The R5 actual-source positive explicitly requires the real `impl Drop for SandAgentDb`, preserves that entire impl in both the media-only and final residual images, and accepts harmless independent trait implementations without exempting them. LF and physical CRLF positives require equal normalized residuals and both run the existing owner-local admission checker. The normalization is limited to this syntax image’s physical CRLF/LF distinction.

### Mutation controls cover the actual integration boundary

Every textual media replacement now requires exactly one intended source site and a changed replacement; every negative fixture starts from a proven actual-source baseline. Both line endings are exercised for missing/foreign/trait/duplicate/conditional helpers and owners, malformed boundaries, changed COMMIT/ROLLBACK acknowledgments and cleanup, direct/aliased/raw/associated/macro callers, discarded or changed settlement handling, changed poisoned/prior/BEGIN handling and deferred closure entry.

The separate raw-COMMIT controls preserve the unchanged bounded adapter proof, then require the forbidden statement to remain visible before/after the actual source, in a helper sibling, inside the media write closure, and in trait implementations for both owner types. Both the initially declared hidden-trait controls and the current receiver-method trait controls remain present. This is the right residual assertion for the unchanged global prohibition, rather than incorrectly expecting the bounded adapter proof itself to reject every raw SQL statement.

The original channel controls still operate on the complete actual source after only the independently checked media exception is applied. Existing zero-argument RAII positives, unary-not positives, raw identifiers, associated references, nested/forwarded macro negatives and hidden raw owners remain source-visible. These are source-design observations; none of their assertions was executed here.

### Existing owners and consumers stay aligned

The runtime correction remains inside existing owners: Attachment paths has only subordinate deletion disposition; AgentDb owns canonical send/receipt/fence truth; Session preserves the inner typed result through its existing `Result<T, String>` filesystem boundary; Runtime forwards that typed media error; App alone maps transaction unknown to `ToolOutcomeUnknown`. Other String APIs remain in place. Existing Runner guard/custody source consumes that typed unknown before ordinary tool-error serialization can discard its meaning.

The App’s map lock is released before entry `try_lock`, Session/DB work and external callbacks. The per-entry reservation covers materialization/capture/cache changes and owner settlement. Busy or poisoned entries refuse conservatively; normal abort cannot make pending/unknown eligible for deletion. Successful settlement clears path/cache ownership into a retained terminal entry before App publication, ACK and counting. The actual per-turn production sink is constructed with its own map at App line 11733, so these tombstones belong to that existing sink lifetime. Their retention is an explicit MSU-04 requirement. Distinct source collection and exact-source cache reuse remain available before sealing.

The MAC clarification correctly distinguishes DB’s existing post-COMMIT listeners from later App-local publication. AgentDb releases its connection guard before these listeners; App’s held entry makes same-ID listener reentry refuse. The correction does not invent an assertion that DB listeners run after the App terminal transition.

The adopted DB fixture now uses an explicit expected canonical sequence: send first, hidden settlement receipt second, including exact payloads. Independent SQL reads and reopened `SandAgentDb::get_transcript_entries` agree on that sequence. Session’s separate `hide_session_work_receipts` remains the presentation boundary. The fixture does not impose presentation filtering on the canonical getter.

### Verification ownership is preserved

The empty default-off Host feature and App dev dependency introduce no new runtime dependency package or second factory. Fault state, leases and authorizer support are gated by `cfg(any(test, feature = "test-support"))`. The real COMMIT executes before acknowledgement loss; the real ROLLBACK authorizer is scoped to its owning borrowed connection and removed before retirement. Outcome classification remains in the existing transaction owner and is not driven by observation flags.

The native media capture retains the existing contracts and adds 13 DB fault/owner cases, five custody cases and 27 transaction-commit selections. Those 27 include all eight media oracle tests, the global assertion, original channel controls and adjacent transaction owner regressions. The result assessor requires exact named positives, zero failed/ignored results, one expected summary and successful command/capture exits. Portable capture adds the media positives to the existing complete session contract target and retains the original channel tests in that target. Its SendMessage complete-target expectation changes from 17 to 19 alongside the two new Tool controls, preserving the other two target counts.

The existing Linux actual-App step retains SGF and LFA cases and selects all eight new App cases by exact test names. It retains Cargo JSON and binds one ordinary Host library artifact by manifest, source and target kind for the shipping feature-off check and another feature-on App test build; neither observation can be satisfied by a cfg(test) Host harness. Source-only review cannot establish that either artifact or any selected case has actually completed.

### Bounded accounting adoption and performance

The current shard manifest is `/workspace/scratch/7d27ea239237/media-settlement-r5-context/inventory-shard-manifest.json`, Git blob `911019b1c4c3b629a4425610bf941caf88a97e18`, independently hash checked. It names canonical inventory digest `d0049087bdbcc8eecdc8d0a69bd48bfc7d70688f8917122198a6e15c80863d8a` and the same fixed U. The manifest explicitly describes source accounting rather than accepted review/implementation/dependency closure. The R4-context manifest/selected rows were not used as current post-materialization row authority.

The unchanged gate source was read at the owning boundaries: `source_review.py` pending-addition declaration and exact missing-link validation (1–110), `expand_inventory.py` conservative row preservation and mapping union (110–165), `shard_inventory.py` materialization/counter ownership (1–175), and `verify_audit_integrity.py` exact links/counters/global completion checks (85–124). The removal of `pending_mapping_additions` and the four newly classified paths are consistent with that existing Actions materialization interface. The incoming index keeps unread at 62067, all decomposition/read/global closure booleans false, and the new responsibility at partial/not-run/pending. The separate read-scope responsibility remains explicitly incomplete. I did not re-expand or validate the shards locally and do not treat the mapping count, supplied run/artifact identity, or this gate-source reading as semantic source review.

No new production scheduler, registry, durable store or broad owner was introduced by this slice. Short index access plus per-entry immediate lock refusal avoids adding a map-wide I/O serialization point; cached repeated sources avoid another materialization/capture. The test oracle deliberately reparses a large source image for many counterfactuals, so its elapsed cost must be observed in Actions. I found no demonstrated production performance defect or unneeded dependency growth in the changed source. No performance number or benchmark result is claimed.

## Findings

`findings: []`

There is no source fix requested by this lens. The approval covers only the exact bound source scope, not an inferred future merge, later head or execution result.

## pendingActions

1. Bind any publication to the exact three-path candidate and stated parent. If concurrent integration changes this candidate or an actual adjacent consumer, freeze that new tree and review the changed boundary before carrying this verdict forward.
2. Execute the existing native media capture and portable contracts only in GitHub Actions on the newly frozen source. Retain named positives, complete logs, command/capture exits, subject tree/commit and input hashes. The two line-ending positive paths and every mutation rejection still require execution; source inspection is not a passing parser/test result.
3. Execute the existing ordinary shipping App check and App test build, retaining the two manifest/source-bound Host compiler-artifact feature observations. Execute all eight exact App cases, including real applied lost-COMMIT ACK and rollback-denial faults, same-ID/Drop/fresh-owner observations and the actual generated Runner case. Preserve the existing SGF/LFA counts and oracles.
4. Assess current exact-revision source-authority checks against the current materialized manifest/shards. Keep historical reading/review facts and all global completion flags unchanged unless their independent evidence authorizes a transition. This report independently checked the current manifest and unchanged gate source, not all compressed canonical row contents or an Actions gate result.
5. Keep MSU verification and acceptance pending until current-source execution evidence is adjudicated. Broader assistant-content rewrite, delegated read-scope/physical identity, detached-result settlement, no-fence durability, arbitrary crash points, real provider/channel/device compatibility, platform packages and final migration gates remain independent obligations.

Terminal state: **review complete; zero source findings; executable verification and acceptance pending.**
