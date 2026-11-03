# LFA/FUA integrated media admission — independent R2 structure, reuse and performance review

**Terminal verdict: APPROVED for the frozen R2 source scope. Actionable findings: 0.**

This verdict approves the reviewed structure, reuse and performance changes and the source correction of the deterministic AgentManagement batch-admission defect. It is not executable verification, native-platform acceptance, approval of a later tree, or production/shipping acceptance. The inherited workflow creation blocker and the subsequently reported adjacent-H Windows failure remain separate, explicit execution constraints below.

## Reviewer configuration and isolation

- Assigned lens/model/effort: structure, reuse and performance; `gpt-6.1-sol`, `xhigh`, explicitly supplied in this worker assignment under the repository review policy. The scope includes interacting URL normalization, native paths, real reader/materialization consumers, batch failure ordering and evidence composition. I did not change global model settings or start another reviewer/process to tune them.
- Read-only reviewed source: `/workspace/scratch/7d27ea239237/media-admission-r2` and its exact base snapshot; adjacent unchanged source was read by immutable Git object or `git show` at the stated base. No mutable foreign-checkout source was used. The other checkout was used only as a Git object database.
- No old reviewer report was opened, and no other reviewer was spawned. In-scope authored spec/evidence files contain historical decisions and report identities; these were treated as source context, without transferring any historical approval.
- No product, URL parser, build, test, linter, generator, schema, capture, acceptance or benchmark code was executed locally or on htch. Shell operations were limited to source/archive reads, text location, Git object inspection and byte/hash comparison. The sole written path is this report.

## Frozen subject and authority

| Identity | Value |
| --- | --- |
| Repository / branch | `bhrumom/fabushi-desktop` / `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Fixed upstream U / tree | `1440bb1771372334bea23d7642bb52deb7db1845` / `90c8aab39972da11cfad220361781d49e1bc6700` |
| Exact review base / tree | `0e845ce1679e0cdf95346adc54f5d58bf06464e8` / `4e69dc6d77f4ff4508e02cdd7064084530db9b6e` |
| Frozen R2 candidate tree | `a4f1c59eb577a45c1b05da1de0a8f13644dc1a5c` |
| Prior source tree, delta context only | `3129fb0c3897bdf78d50e1cd067cc3dd65310297` |
| Manifest | `/workspace/scratch/7d27ea239237/media-admission-r2-manifest.json` — complete 22-path scope |
| Complete patch SHA-256 | `4900c4d4d87ba403efa762a9bc6dc0ea9272805eb7b1c48a4b7e38f2b2988c3b` |
| Repair/incoming patch SHA-256 | `697877b87e297675e79b09d204b9210df020c4ef72a0d0bea3338fc90962c803` |

Both patch hashes and every candidate SHA-256 in the 22-path manifest were independently recomputed and matched. The manifest records the candidate remote recursive-tree readback as 10,501 entries, nontruncated, with all 22 changed blobs matching; that remote readback was supplied by the integrating runner, not repeated by this reviewer. The candidate tree was not available as a local Git tree object; the matching frozen bytes and supplied remote identity bind this review.

Governing source read: root `AGENTS.md` blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2`, OCRM `SOURCE_OF_TRUTH.md` blob `3748d734b10c9ed5b2d948467fbaa9483f1dd306` (current authority/ownership/execution sections, with historical snapshots treated as historical), main specification blob `69c9667af2328e20a1fba09ebf80c5aab4d5c336` §17, complete batch prompt blob `5ff7894d245588ed539ab075b09215a2f1a82716`, and relevant ownership-map context from blob `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3`. The base recursive Git listing has no nested AGENTS governing this scope; its sole nested AGENTS is under the unrelated `source/mahayana/codex-rs/tui/src/bottom_pane/` subtree.

Both complete LFA-v1 and FUA-v1 specifications in this snapshot were read. They retain the distinction between admission, actual read authority, persistence outcome handling and whole-migration acceptance.

## Source conclusions and causal oracles

### One existing admission owner, with caller-specific error projection

`source/host/src/extensions/attachments/media_reference.rs:20–181` owns the narrow pure path guard and the shared tri-state file-URL converter. The implementation uses the existing locked URL parser, distinguishes ordinary non-file sources from rejected file sources, preserves the remote-host-erasure violation before it can be lost, validates percent escapes and strict pathname UTF-8, rejects encoded separators/NUL, applies a Windows rooted-drive precondition before native conversion, requires an absolute result, and applies the same native path spelling guard.

The helper has no filesystem, Box, Session, network, persistence or outward publication operation. Its `Cell` holds only synchronous per-call parser observation. No module, crate, dependency, registry, persistent store, permission capability or parallel runtime is added. The small malformed-scheme classifier only classifies parser failures; it never converts a pathname. The web helper only retains raw prefix evidence that the URL dependency otherwise erases. Neither helper is a second general URL parser.

The existing metadata-only `send_message_tool::file_path_from_file_url` delegates to this owner and preserves its Option facade. The real App resolver consumes the Result directly (`main.rs:639–714`), before `fs::read`, Box fallback and managed-file materialization. Both real sink adapters propagate that Result (`main.rs:899–906`, `1180–1187`). The ordinary non-file passthrough remains before any local materialization. File-context and Channel consumers use the same owner; Channel additionally guards the final reanchored path before metadata/read (`channel_attachment.rs:57–63`). Web-media preserves its typed `invalid-file-url` projection and existing stricter prefix/network policy rather than transferring read/root authority to the converter (`web_media.rs:294–386`).

Causal controls in source include native exact-path and byte positives, malformed/remote/encoded-path negatives, non-file HTTPS passthrough, and the private web-reader zero-entry counter. The web counter proves boundary entry, not arbitrary OS syscall absence. Source reading confirms the actual filesystem operations occur after that boundary. These controls were inspected, not executed.

### R2 AgentManagement repair closes the stated deterministic batch defect

`sand_agent_management_tools.rs:61–65` evaluates `parse_images(...)` completely before starting `.into_iter().map(resolve_image_source)`. The R2 addition at `:99–104` delegates each file URL to the same pure shared owner while constructing that vector. A malformed later URL therefore returns before the first image resolver can materialize a file. The actual production sink retains its own direct-call Result boundary. This fixes the accepted valid-first/invalid-later URL scenario without inventing another rollback/custody owner or changing the sink trait.

The bridge contract `lfa_agent_management_batch_admits_all_urls_before_resolving` supplies a valid native first URL and several invalid later URLs, then checks zero resolver entries, zero sends and no image result. Its native/HTTPS positive uses the same bridge after refusals. The actual App case `local_media_file_url_app_agent_management_refuses_image_before_outward_send` adds the causal product oracle: actual source/target Session owners, a real source PNG, single-image direct-sink refusal, repeated valid-first/later-invalid calls, unchanged source/target transcripts and admission records, unchanged managed-file set, no wake and no outgoing tracking, followed by real owner reopen and the same checks. Its positive then proves exactly one materialized managed file with exact bytes, an outgoing transcript image URL, one wake carrying exact PNG bytes, and persisted managed bytes/transcript after another real reopen. The wake port observes the real messaging owner; it does not execute the recipient Agent or prove external delivery.

The preflight covers deterministic URL refusal. Existing later read/persistence/delivery failure cleanup and unknown write outcomes remain explicitly outside this repair and are not reclassified as solved.

### Existing SendMessage custody remains the only rollback mechanism

The real SendMessage bridge already calls `abort_attachment_custody` on a failed message build/encoding. The changed App source now makes invalid file URLs reach that existing error path. The production abort (`main.rs:969–996`) removes only the call's pending custody and releases the captured Session media admission. The new observer delegates the real sink rather than supplying a replacement resolver. In `local_media_file_url_app_rolls_back_earlier_image_on_later_refusal`, it records the first actual managed file, exact bytes, pending identity and durable Session fence before the later refusal, then verifies actual rollback and release. This avoids a vacuous test that passes because the first source never materialized.

The complete six-case App module also preserves native/Unicode file materialization, original bytes, normal transcript fields/dimensions, successful repair after repeated refusal, durable reopen behavior, and HTTPS without local custody. The fixture reuses the existing `idle_runner_owners`; it does not establish a second production composition.

### Incoming BR and FUA semantics are retained

The complete base-to-R2 diff leaves the incoming BR bounded-reader implementation and its entire private test tail unchanged. Independently hashing the extracted section from `pub fn read_outbound_media_file_bounded` through EOF produced `b67cfc8cb493d016b6c5382741531c55c159eb74aeb606044ce2d328f6603334` for both base and candidate. The body retains same-handle file-kind/size admission, Unix `O_NONBLOCK`, the 64 KiB scratch read, one-byte EOF/overflow probing, fallible bounded geometric growth and original errors. The unchanged direct service cap conversion was read at base `attachments_service.rs` blob `75a49490ee80e7416df66df2431d2f3013f1b17c`; no service source is modified by R2.

The existing `file_url_has_extra_leading_path_separator` helper is also byte-identical between base and R2: extracted-function SHA-256 `8ea516c75e070dd9d27f1f4aea64b915c76a4ac3979fc8cd4ad35168a85704ed`. R2 invokes it only after the shared owner identifies an admitted file source. This preserves the incoming localhost/doubled-separator refusal, mixed separator and ignored ASCII tab/CR/LF handling, and avoids applying this file-prefix rule to a non-file source. Its drive-token exception remains for ordinary interior separators; query/fragment bytes stop prefix inspection. Native service/loader tests retain interior-separator and query/fragment positives alongside the original refusals. No reader policy is broadened and no original assertion is removed to secure this source verdict.

### Added cost is bounded by supplied input size

The new converter does a constant number of linear passes over one URL/path and linear-sized temporary decoding/native-path allocations. Windows colon/NUL checks and the raw-prefix inspection are linear. The complete-image preflight adds one shared conversion per image before the existing resolver conversion; this intentional repeat protects both batch and direct sink boundaries. It remains O(total supplied URL bytes) and does not add recursive traversal, all-pairs image work, cross-call state, background work, repeated reads, retained caches or whole-file allocation to admission.

The BR loop's bounded/amortized storage behavior is preserved byte-for-byte. R2 does not change the App resolver's existing unbounded `fs::read` or its persistence fallback; those are existing responsibilities, not a cost improvement claimed by this patch. No performance measurement or executable result is claimed.

## Capture and integration disposition

The native capture reuses the existing script and target selection. Its complete selected source names include all inherited FUA/BR names and all 13 oracle controls. `assess` still requires successful command/capture exits, exact unique required positive names, no extras, and one exact zero-failure/zero-ignored summary. Input capture still checks committed and normalized worktree blobs before/after, exact H/dispatch identity and native platform identity. No oracle was executed in this review.

The common Channel contract is selected once, with the union of BR/LFA assertions. Manual source accounting gives the following expected selections; these are **not observed passes**:

| Existing/new group | Linux | macOS | Windows |
| --- | ---: | ---: | ---: |
| Cleanup owner | 7 | 7 | 8 |
| Cleanup consumers | 17 | 17 | 16 |
| HTML policy | 12 | 12 | 9 |
| Host read format | 1 | 1 | 1 |
| FUA private admission | 5 | 5 | 6 |
| Web media consumers | 28 | 28 | 26 |
| Image probe consumers | 16 | 16 | 16 |
| BR private owner | 11 | 11 | 8 |
| BR public contracts | 17 | 17 | 17 |
| BR Box consumer | 1 | 1 | 1 |
| BR image consumer | 1 | 1 | 1 |
| Combined Channel target | 7 | 7 | 8 |
| LFA shared owner | 10 | 10 | 11 |
| File context consumers | 10 | 10 | 10 |
| SendMessage consumers | 17 | 17 | 17 |
| AgentManagement consumers | 3 | 3 | 3 |
| **16 groups total** | **163** | **163** | **158** |

The declared input list contains 46 source inputs. The native receipt explicitly separates its shared/native consumer scope from the six actual Linux App cases. The workflow adds those six exact App names to the existing actual-App harness with exact listing and one-pass/zero-fail/zero-ignored result requirements. Both existing SGF required-name lists include the two new LFA metadata-facade tests; the complete-target summary expectation changes only from 15 to 17 while retaining the 14/13 summaries, cardinality and failed/ignored assertions.

The complete ledger/RTM delta adds narrow owners, dependencies, contracts, risk statements, test/evidence references and failure-preserving rollback language. It changes no responsibility status block, completion flag, literal historical read fact, source authority or acceptance state. Evidence files retain original failures and distinguish source expectations from execution. This is reuse of the existing ledger, not a second completion authority.

## Complete scope reading dispositions and identities

Every listed candidate SHA-256 was independently compared with the manifest. Git blob identities below are the frozen manifest identities. “Deep” denotes the changed behavior plus relevant caller/consumer/fixture context, not a claim that all unrelated lines of a large inherited file were newly read. Large unchanged workflow/ledger/RTM bodies were disposed by complete diff plus the affected adjacent contracts.

| Path | Frozen Git blob | Reading disposition |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `1e9529f7036b0c8559417d5f3d3ae2c0fe1cb0b5` | All six changed hunks and adjacent SGF/App selection, identity and result gates read; inherited workflow size blocker separate. |
| `projects/openclaw-rust/docs/specs/local-media-file-url-admission.md` | `8a8a7a77a641297cb0e8f46e7235fad63991991f` | Complete spec, R2 batch repair/incoming preservation and limits read. |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `5204354c54a4a600f30eed21c3ee3ed7c121ed6f` | Complete FUA spec and locked-parser correction read. |
| `projects/openclaw-rust/evidence/local-media-file-url-647e0932-20261103.json` | `0722207d574b638c1e9984737f85cb05fa3aa3b7` | Complete evidence declaration read; source assertions checked against actual owners and captures, execution declarations not promoted. |
| `projects/openclaw-rust/evidence/media-file-url-normalization-recovery-647e0932-20261103.json` | `721190ec1e165a8fbdf9e5764d779796000e71e9` | Complete recovery declaration read; historical failures/identities and pending exact-H state retained. |
| `projects/openclaw-rust/parity-ledger.json` | `62b97f81d3d49a57a0af6a010783628887001f0a` | Complete changed hunks and affected responsibility context read; no status/flag promotion. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `5053c28408e67cdd8235d01e53343f396922df55` | Complete changed hunks read; existing R15/R23/AC15/AC23 and shared references extended without state changes. |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | `dfc83dd491046de7708643ce48b0fbebd74a19bb` | Complete source read: inputs, all selected groups, 13 oracle controls, invocation/result/identity capture. |
| `source/host/app/src/main.rs` | `bf65f7ee95cfe1dd76568383eb7f81fe13f06b08` | Deep resolver, both sink adapters, existing custody abort/admission and test-module integration; all change hunks read. |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `5682e951f1eac8f107efd3a3948b0748870912bd` | Complete 769-line actual-owner fixture/observer and six cases read, including full strengthened AgentManagement oracle. |
| `source/host/src/connectors/channel_attachment.rs` | `723006c2a5b054fc9ddc47f58fdb4ebd8639351e` | Complete reader/classification source read; final reanchored guard precedes metadata and bounded read. |
| `source/host/src/extensions/attachments/media_file_context.rs` | `0777b2fff28f09e5a6f276ed71c6ec776d6601e6` | Deep shared-admission delegation/local-path/home expansion and changed scope; adjacent service projection read. |
| `source/host/src/extensions/attachments/media_reference.rs` | `860278b4d775e11fbdfb1f5b8124b1b40d8bb0a0` | Deep complete new converter/path guard; existing classification/consumer context and complete BR reader/test tail read, tail hash preserved. |
| `source/host/src/extensions/attachments/web_media.rs` | `98bf139069ce0e4bcb13b1bffbf960feeb311840` | Deep adapter, original-prefix helper, full admission/read body and private boundary controls; prefix hash preserved. |
| `source/host/src/runner/tools/sand_agent_management_tools.rs` | `22e7f260ca0d23250802af3d862cf836b21d675b` | Complete bridge/source read, with deep parse-all-before-resolve ordering. |
| `source/host/src/runner/tools/send_message_tool.rs` | `9e12d7d2b457715f0fac168e5e4f262e6ef15985` | Deep Option facade, actual image loop, build/error/abort call paths and observation/delegate context; all change hunks read. |
| `source/host/tests/channel_attachment_contract.rs` | `887f2cb3ea78060be617ad1c039f5b28306c6ac0` | Complete source read; old reader assertions plus added native/invalid/retry/Windows contracts retained. |
| `source/host/tests/local_media_file_url_contract.rs` | `5170951a59fb94d8c1a3673586690a9dc8315a88` | Complete shared/native platform contracts read. |
| `source/host/tests/openclaw_media_file_context_contract.rs` | `2c2365d9f2ab380bd2fc4374dba4096a1911fd0f` | All changed native fixture/dispatch tests and new owner/Windows/Unix controls plus adjacent existing assertions read. |
| `source/host/tests/openclaw_web_media_contract.rs` | `7e3b39ff61e0776748b0416f517fd45d0107ed6e` | All changed hunks plus complete FUA native/converter tables, interior/query positives, incoming BR service/root/retry controls and new LFA consumer cases read. |
| `source/host/tests/sand_agent_management_tools_contract.rs` | `28372cd0046620bc88e9d84ca016692e0c89fa24` | Complete bridge contract source read, including resolver-entry negative and fresh positive. |
| `source/host/tests/send_message_tool_contract.rs` | `8c334d36e4b507842a38509b945956fe5085beb7` | Complete added facade contracts and affected existing target/producer/observation context read; original assertions retained by full diff. |

No path in the declared 22-path scope is left unreviewed or excluded by extension/directory. No full-source, dependency-closure or whole-unit review completion is inferred from this bounded review.

## Exact adjacent dependency and consumer reading

Both published dependency archive SHA-256 values were independently verified:

- `url-2.5.8.crate`: `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`. Read actual `parser.rs` file-host/drive normalization, ignored-control behavior and path normalization (`510–570`, `1057–1111`, `1188–1385`, `1774–1813`), and `lib.rs` public/native file conversion (`2726–2755`, `2990–3202`). This directly confirms host-erasure signaling, lost initial empty segments and the need to reject bare Windows drive conversion before its assertion. It is a bounded dependency/API review, not a complete crate read.
- `fs-safe-0.27.0.tgz`: `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`. Read packaged `dist/local-file-access.js`, `windows-path-alias.js` and relevant `windows-path-syntax.js` definitions. The Rust path guard's authority/colon/network distinction reuses that actual policy shape; no Node service/runtime is introduced.

Fixed-U source files were read from the supplied exact-source cache after independently obtaining their Git blob hashes: `src/media/local-media-path.ts` = `f105ca514b5754849284c429b032af30d1487f1a` (complete consumer); `src/media/web-media.ts` = `f48bca504f6fb77c3d094864eb656cb2b1bf25e2` (actual URL admission, options and local reader boundaries); `src/media/web-media.file-url.windows.test.ts` = `bdaedcde111ea2880b96d9afa547a9c1d2a36b42` (complete native Windows oracle). These identities match the governing scope/provenance; no cache file was executed.

Additional immutable target context included `send_message_schema.rs` blob `0e11c27ace3ae51888fdef4089cd11b09271ea7d` (existing scheme acceptance), `attachments_service.rs` blob `75a49490ee80e7416df66df2431d2f3013f1b17c` (direct service cap conversion, loader and local-path dispatch), and `agent_to_agent_messaging.rs` blob `e1b3769993c5086f3fbce654d431a42b9bfa1783` (actual outbound transcript, target admission and selected-image wake bytes). Existing `channel_host_lane_tests.rs` fixture identity is `ff4ebba71eae35f4be98a695654a0d33b84f1818`; the new fixture's reuse was traced through its imported helper and real owner fields, without claiming a fresh deep read of unrelated inherited App scenarios.

## Separate execution constraints and remaining acceptance

The inherited base workflow is 530,211 bytes; this frozen candidate is 532,528 bytes. The supplied run history records `41569288081` as a zero-job creation failure for the earlier `497c2c7...` subject. This is not a source defect newly assigned to this media patch. An existing separate WFX owner has the extraction recovery; this review neither duplicates that work nor approves a later WFX tree. Final execution requires a workflow that GitHub accepts and evidence bound to the exact published subject.

During this review the integrating runner reported adjacent subject `e19a0a265e400a4c9b1d96520c84fb612a3843e8`, runtime `41571103928`, Windows job `125703932910`, failing an incoming rooted-drive positive in `test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references`. The supplied raw panic is `rooted drive remains valid: LocalAccess { code: "invalid-file-url", message: "File URL cannot be converted to an absolute local path" }`; no specific table variant was labeled. The runner confirmed that subject used old web-converter blob `9912720f7c6eff6b53444b1912e3b291613d33e4`, including its `path_segments().nth(1)` precondition, rather than R2 shared owner `860278b4...` / web adapter `98bf1390...`. That adjacent-H failure is preserved as a known unresolved native gate and is not relabeled as a frozen-R2 run, a passing result, or evidence of a source defect in a different blob. All four incoming root assertions remain present in R2 at `openclaw_web_media_contract.rs:270–271`; none is weakened here. Exact-R2 native execution must decide them, including encoded-drive behavior, before bounded acceptance.

No R2 executable validation was performed or inferred. Native Linux/macOS/Windows receipts, actual Linux App results, source/identity gates, and any invalidated dependent evidence remain required at the final repaired exact H. Generated HTML origin, authentic read capability/root confinement, filesystem identity/races, existing media persistence unknown outcomes, live services/providers/channels/devices, packaging and full G01–G12/AC01–AC24 acceptance remain open as stated by the specs. No OCRM status or global flag is promoted by this report.

**Final source adjudication: APPROVED; 0 actionable structure/reuse/performance findings; no source fix requested. Execution and shipping acceptance remain pending their exact-subject gates.**
