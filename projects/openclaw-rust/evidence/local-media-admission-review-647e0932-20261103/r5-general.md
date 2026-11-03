# R5 blind general / security / correctness re-review

## Terminal verdict

**APPROVED for the focused R4 → R5 source integration delta. No blocking findings.**

This is a terminal source-review verdict for the exact frozen candidate below. It is not an execution result, a new full review of all 21 non-evidence paths, an approval of the historical evidence JSON contents, or full migration acceptance. The parent retains any earlier verdicts for unchanged source separately; none of those reports or decisions was read or used here.

- Task: `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, review round R5.
- Repository / branch: `bhrumom/fabushi-desktop`, `codex/openclaw-rust-fusion-chat-20261101-writer`.
- Exact base / starting H: `78aa967cc9400bb5c207441b2a78dbf7021d596c`; base tree `f62fc666ade7ad6f8dbf40e889e5ec100a74e974`.
- Exact R5 candidate tree: `9a9afd4841590abc646805b8308c6e716821f067`.
- Prior R4 tree: `76e6c26d754179ee5af447ecec689a2de844b476`.
- Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`; tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Frozen source: `/workspace/scratch/7d27ea239237/media-admission-r5`; prior and base snapshots are the manifest-designated `media-admission-r4` and `media-admission-r5-base`.
- Model / effort: configured model retained; inherited xhigh according to the parent launch record, appropriate for shared security admission and concurrent composition/capture integration. No worker-side model change, additional worker, or external AI process was used.

Later branch movement does not alter this subject or verdict.

## Findings

No falsifiable blocking correctness, security, assertion-preservation, or integration finding was identified in the eight-path delta and the adjacent source read below. There is therefore no severity / trigger / defective function / corrective patch entry to report. The outstanding runtime evidence is an acceptance boundary, not a source defect or a passing result.

## Review conclusions and supporting source

### 1. The old native converter fact and current product oracle are distinct

`source/host/tests/openclaw_web_media_contract.rs::test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references` now directly characterizes `Url::parse("file:///C|/").to_file_path().is_err()` under `cfg(windows)`, while the product table requires exact `C:\` output for all five spellings: the three original colon / encoded-colon forms, incoming `file:/C|/`, and LFA's `file:///C|/`. Value-labelled diagnostics remain. This is a deliberate product compatibility transition, not contradictory accept/reject expectations on one function.

The complete LFA and FUA specifications make that transition explicit as LFA-03a / LFA-03b, preserving the historical FUA-WIN-ORACLE-01 fact and the distinction between an unlabeled runtime failure and source-derived diagnosis. The exact pinned `url` parser and Windows converter support the dependency characterization: empty/localhost authority handling can leave a leading pipe drive beyond first-segment normalization; the Windows converter accepts colon / encoded-colon in that drive position. The shared owner changes only a parsed leading `/<ASCII letter>|/` before native conversion, after authority/host-erasure refusal. Its URL setter preserves encoded pathname and query/fragment boundaries.

The fixed-U Windows file URL test was read in full. It covers uppercase single-slash native paths, spaces/Unicode and unsafe URL refusals, and contains no pipe-specific oracle. This review therefore does not infer complete Node/Rust pipe parity from that test.

### 2. No admission or reader-boundary weakening accompanies the test transition

The shared converter and web adapter are byte-identical to R4. Context reading of `media_reference.rs::resolve_local_file_url`, `admit_local_media_path`, and `web_media.rs::resolve_web_media_file_url` / `load_web_media_with_local_reader` confirmed the relevant order: structural parsing; original-host-erasure and authority refusal; the narrow Windows pipe normalization; escape, encoded-separator, UTF-8 and NUL checks; rooted-drive precondition; native conversion; absolute-path / alias admission; then the web adapter's retained original-spelling and resolved-network checks before its private reader.

The web prefix rule continues to preserve four-or-more / localhost repeated leading-separator refusals, including mixed separators and ignored tab/CR/LF characters, while leaving the drive-token interior-separator positive available. The private reader counter controls remain unchanged, including default/rooted/sandbox variants, zero-entry malformed and pipe-alias negatives, the local positive, no-retry error propagation, and native verbatim-drive workspace control. No new public reader callback or `loadWebMedia` Gateway registration is introduced.

Relative to incoming base 78, the web-test removal is precisely the obsolete product-level triple-slash-pipe negative; it is replaced by the explicit dependency negative and specified product positive. Every other old refusal and each incoming rooted-drive positive remains. The real native file/service byte, filename, cap, retry and original-source assertions remain, with the R4 pipe variants preserved.

### 3. Incoming Device App and capture bindings are preserved

The R5-to-base diffs establish that the Device restoration is exact incoming source preservation. `main.rs` differs from base 78 only in the LFA Result-based resolver, its two sink propagation sites, and LFA test-module registration. The restored `device.token.rotate` native dispatch arm, `GatewayApi::rotate_device_token` implementation, and `device_token_rotation_wire_tests` module match the incoming source.

The adjacent immutable `gateway_server.rs` routing was inspected. HTTP `device.token.rotate` goes through the bound hook with the admitted optional authorization and singular `x-sand-scope`; the App hook forwards those arguments to the existing `DevicePairingOwner::rotate_token_with_authority`. R5 adds no parallel permission owner or fallback route. This is an integration-preservation conclusion, not a fresh full functional review of Device rotation internals.

Workflow and portable-capture diffs against base 78 likewise retain incoming Device inputs, all selected names, the complete wire-rotation owner/HTTP targets, retained-stream selections, three App helper cases, and strict whole-target result checks. Both recorders require exact required-positive occurrence and zero failures/ignored counts; the Device complete-summary gates also retain zero measured/filtered results.

### 4. Shared capture strictness and LFA consumer wiring remain intact

The portable shared SendMessage group adds the two named LFA facade cases to the incoming 15-test target, requiring 17 alongside the unchanged 13- and 14-test summaries. Required-name uniqueness, total observed positives, command exit and capture exit conditions remain. The unrelated canonical-result oracle qualification and existing other group gates are unchanged.

The Linux App capture retains exact test listing, exact execution, one-passed / zero-failed / zero-ignored summaries, command failure propagation, all-case completion, and its existing startup controls. The six LFA cases are added to that established mechanism with source-input bindings; Device App selections remain present. The native media capture is byte-identical to R4 and still binds before/after inputs, exact H/platform, required names, duplicate/unexpected-result rejection, synthetic oracle qualification, and command/capture outcomes.

Adjacent App / Runner reading confirmed that invalid local file URL results propagate from `resolve_production_attachment_source` before native read, Box download or materialization. `ProductionSendMessageSink` retains prior Session work admission and custody ownership; `SendMessageToolBridge` invokes the existing abort path on later resolution failure, and the App abort path rolls back custody and releases its Session admission. AgentManagement retains complete pure value preflight before any per-image resolver call. These reads provide integration context; the unchanged source is not represented as newly reviewed in its entirety.

### 5. Ledger and RTM remain declarations, without status promotion

The complete eight-path delta was reviewed, including Device ledger additions and all RTM additions. Comparing the candidate ledger and RTM against incoming base 78 leaves only LFA declaration/evidence/test mappings and related risk / rollback descriptions. Existing Device entries remain. No status, verification, acceptance or completion-flag field changes occur in either comparison. Historical receipts remain references, not current-H results. This review did not open their contents.

## Identity verification

All 23 frozen file byte lengths, SHA-256 values and Git blob identities were independently recomputed and matched the manifest. Every available base-file snapshot matched its declared base blob. Exactly the eight declared focus paths differ from R4; all other 15 files are byte-identical. The candidate full-tree API identity/count is the supplied manifest record, not a new independent remote-tree read by this worker.

Both patch files were hash-checked:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `media-admission-r5-complete.patch` | 194229 | `16458c52ca55332fbba2b33fb2ada8a946ff7cbd065e0fa6fcee1e7b9cff771f` |
| `media-admission-r5-integration.patch` | 81687 | `a7306e8809b3c079636f2167a313b94820e2b61e92ce7fed46bfcc41bd459989` |

### Eight focused paths

| Path | R5 Git blob | R5 SHA-256 |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `580eabc6ade055fc47850b9777814f131feb0594` | `af9cad942a4b5f9a91b4e5d3d1ad728b4982679426e9e8acfb526c1629eb3299` |
| `projects/openclaw-rust/docs/specs/local-media-file-url-admission.md` | `9249e4a93f353247e97c7ef47339382932451d72` | `146dadba7911290f7f78a4b6e436e02b1b0332754e7b219b62aba39690f65526` |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `5414d7193cc7cbcbcfec51dec54cda68be5130ad` | `637616e95d63ca7e57b75a9267dc2a7b98f0a71c7941f88e48d5547d33ebbdc6` |
| `projects/openclaw-rust/parity-ledger.json` | `3fb5f2b4b5cade7d15d3cbedacc01c8a5dcd1cf0` | `7aadc1a486ed5626d271434ca50790134d72f72c416cb20fb82c41c714c563c3` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `dddcfc37286add1e5c3be86bed9eb346057b0fbb` | `99777c80b3a643ce4cf67b5499a8ce43b82dedfc5209ce74e7b1fd59fe44e700` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `07da058409c318fcd1f19263b1a3520f0154cc7b` | `7fd42a01d20869accf7f9d1c8e110e8eca6a995a6b9907b77912c364b1d3648c` |
| `source/host/app/src/main.rs` | `4831ae8cf002ce64e6409f2d695b48aaa1914c45` | `36605022e09e91269ee17770a48fa9897e5ba4adcf5bbc7b21b6f382e3cdd4fa` |
| `source/host/tests/openclaw_web_media_contract.rs` | `4d7829bd42995f9503d6c9fee390986e8c1c724e` | `059c4c533b289f557b6ac4cef92428e0713835d98495f11c14aee0e99e359a91` |

### Fifteen unchanged identities

Every identity in this table was byte-compared to R4. This is identity preservation, not a claim of full fresh substantive review.

| Path | R4 = R5 Git blob |
| --- | --- |
| `projects/openclaw-rust/evidence/local-media-file-url-647e0932-20261103.json` | `0722207d574b638c1e9984737f85cb05fa3aa3b7` |
| `projects/openclaw-rust/evidence/media-file-url-normalization-recovery-647e0932-20261103.json` | `721190ec1e165a8fbdf9e5764d779796000e71e9` |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | `dfc83dd491046de7708643ce48b0fbebd74a19bb` |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `5682e951f1eac8f107efd3a3948b0748870912bd` |
| `source/host/src/connectors/channel_attachment.rs` | `723006c2a5b054fc9ddc47f58fdb4ebd8639351e` |
| `source/host/src/extensions/attachments/media_file_context.rs` | `0777b2fff28f09e5a6f276ed71c6ec776d6601e6` |
| `source/host/src/extensions/attachments/media_reference.rs` | `0956a0db7df81390002fde917ebc3fe782e75296` |
| `source/host/src/extensions/attachments/web_media.rs` | `6447eff570c31939d3165ac72271cc1914d84055` |
| `source/host/src/runner/tools/sand_agent_management_tools.rs` | `22e7f260ca0d23250802af3d862cf836b21d675b` |
| `source/host/src/runner/tools/send_message_tool.rs` | `9e12d7d2b457715f0fac168e5e4f262e6ef15985` |
| `source/host/tests/channel_attachment_contract.rs` | `887f2cb3ea78060be617ad1c039f5b28306c6ac0` |
| `source/host/tests/local_media_file_url_contract.rs` | `ff2d348d55572394f547deb15c2e36b2347ea43f` |
| `source/host/tests/openclaw_media_file_context_contract.rs` | `2c2365d9f2ab380bd2fc4374dba4096a1911fd0f` |
| `source/host/tests/sand_agent_management_tools_contract.rs` | `28372cd0046620bc88e9d84ca016692e0c89fa24` |
| `source/host/tests/send_message_tool_contract.rs` | `8c334d36e4b507842a38509b945956fe5085beb7` |

The two evidence JSON files were **hash-only**, with SHA-256 respectively `bd7bbcdd2fd6b536599b636c0904bdab129668fd0965753fac83f95d5b966a40` and `68187df3c539f99703f0e7b06460b33884ee13dc1d8ddc499693832dff2d23fe`. Their JSON contents and complete-patch hunks were not opened, printed, parsed, or adjudicated. The complete patch was used only for identity hashing; the eight-path integration patch and permitted individual source diffs supplied review content.

## Actual read scope and blind boundary

Before source judgment, the review read the root `AGENTS.md`, project `SOURCE_OF_TRUTH.md`, main migration specification §17, and both complete current LFA/FUA specifications. Applicable policy identities at base 78 were `AGENTS.md` blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2`, SOT blob `3748d734b10c9ed5b2d948467fbaa9483f1dd306`, and main specification blob `69c9667af2328e20a1fba09ebf80c5aab4d5c336`. Relevant module-map excerpts were context only.

The substantive review covered the entire eight-path integration delta, current edited functions and surrounding consumers, and individual candidate-to-base source diffs needed to check preservation. It did not read the roughly 1 MB App main, complete shared workflow, complete portable capture, or entire ledger line by line. Unchanged-context reads were bounded to:

- Shared converter and local-path guard in `media_reference.rs`.
- Web adapter, private admission/reader transition, and its file URL boundary tests in `web_media.rs`.
- SendMessage facade/resolution/abort call chain and AgentManagement complete-image preflight.
- Native media capture input/selection/assessment/qualification/receipt code.
- Immutable base `gateway_server.rs` bound rotation hook, adjacent authority guards and actual dispatch; blob `2cc83cf387ae37c3b1c28663f72d9d0492e84ccd`.
- The locked URL dependency's file parser, host parser, drive normalization, `set_path`, `to_file_path`, and Windows conversion functions; not the complete dependency package.
- The fixed-U Windows file URL test in full; not all fixed-U media sources.

The acquired `url-2.5.8.crate` SHA-256 is `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`. Its archive `src/parser.rs` and `src/lib.rs` bytes were independently compared to the selected files and match. Their Git blobs are `dbdf9b906238c20ef0896d114d5cb8ad69197e34` and `f1558682b6287d0cba1cd93d84cee6cf6cbeaf44`; SHA-256 values are `2dd4210e1773fc5d3fdc15ce506ecff30dbf8eb358a79bbbc2835cc30d513c3a` and `51f47a93654528f3b8cb92c94fc26ad25775860d84197fff0c3124a687f8c5e3`. The fixed-U Windows test matches blob `bdaedcde111ea2880b96d9afa547a9c1d2a36b42`, SHA-256 `96fa78a117e2da707540777269c8fcf9395cd76ac845c1c4503f63fe9a25491a`. The fs-safe archive was hash-checked only, SHA-256 `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`; no complete fs-safe package review is claimed.

No other review report, prior runner decision, arbitration/checkpoint, publication metadata, or incoming evidence decision file was read. The foreign checkout supplied only explicit immutable Git-object reads; its mutable working tree was not authority. No product, parser, test, build, lint, schema, capture, generator, or acceptance code was executed locally or on htch. Operations were source/text/Git/archive/hash reads plus this report write. Source declarations that refer to tests are not test results.

## Outstanding exact-H evidence and acceptance

The source verdict permits the parent to combine this focused conclusion with its separately retained unchanged-source review and serial provenance audit. Runtime acceptance still requires the eventual exact published H, matching input identities, immutable raw logs and exits, positive-name uniqueness and all selected assertions.

In particular:

1. **Native Windows** must execute the direct dependency characterization, all five product rooted-drive positives, real native path/byte/service/cap/retry variants, and zero-reader-entry refusals. Linux results cannot decide Windows parsing/conversion behavior.
2. **Linux App** must compile the actual non-test App and linked harness, then execute all six LFA production sink / Session custody cases alongside preserved incoming Device/App selections. Helper or native-host tests cannot substitute for those App cases.
3. The existing native media and shared portable capture requirements remain. This review supplies no current-H result for Linux, Windows or macOS, no reclassification of older failed receipts, and no acceptance of real provider/channel/device or package behavior.

G01–G12, AC01–AC24, independent ACCEPT, complete dependency/source closure, public Gateway/read-authority work, identity/race confinement, persistence unknown-outcome behavior, UI/security/performance/soak and supported package/install/upgrade/rollback obligations remain outside this bounded source verdict and unaccepted by it.

