# Media admission R5 — blind structure, reuse and performance focused re-review

**Terminal verdict: APPROVED for the declared R5 focused source review.** No concrete blocking or non-blocking defect was found in the eight-path integration delta or the edited functions and adjacent consumers inspected below. This verdict is source convergence for that focus. It is not a new complete deep review of all 21 content-reviewable paths, runtime verification, bounded product acceptance, or full migration acceptance. The runner must retain the unchanged-source verdicts separately; I neither inspected nor inferred those reports.

Task: `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, review round R5. Lens assignment: structure/reuse/performance, with shared security-admission, concurrent composition and capture integration in scope. The parent launch assignment applies `gpt-6.1-sol` at `xhigh`; this review used that assigned session and started no replacement worker or CLI. The effort is justified by the shared URL admission boundary, Windows dependency semantics, production sink error/custody ownership and concurrent Device/capture preservation.

## Immutable subject and blind boundary

- Repository: `bhrumom/fabushi-desktop`; branch: `codex/openclaw-rust-fusion-chat-20261101-writer`.
- Base and starting HEAD: `78aa967cc9400bb5c207441b2a78dbf7021d596c`; base tree: `f62fc666ade7ad6f8dbf40e889e5ec100a74e974`.
- R5 candidate tree: `9a9afd4841590abc646805b8308c6e716821f067`.
- Prior R4 tree: `76e6c26d754179ee5af447ecec689a2de844b476`.
- Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`; tree: `90c8aab39972da11cfad220361781d49e1bc6700`.
- Frozen input: `/workspace/scratch/7d27ea239237/media-admission-r5`, its supplied manifest, R4 snapshot and R5 base snapshot.
- Complete 23-path patch SHA-256: `16458c52ca55332fbba2b33fb2ada8a946ff7cbd065e0fa6fcee1e7b9cff771f`.
- Eight-path R4→R5 integration patch SHA-256: `a7306e8809b3c079636f2167a313b94820e2b61e92ce7fed46bfcc41bd459989`.

I independently recomputed SHA-256 and Git blob identities for every one of the 23 frozen files and both patch hashes. All matched the manifest. Exactly the eight focus paths differ from R4; all other 15 are byte-identical. The supplied complete-tree API claim remains the runner's provenance; my independent check covers the frozen file bytes and identities, not a separate acquisition of the complete 10,516-entry remote tree.

The two scoped historical evidence JSON files were processed solely as opaque hash inputs. Their content and their complete-patch hunks were not displayed, decoded or assessed. I read no other review, root decision/arbitration/checkpoint, publication metadata or incoming evidence decision file. No mutable foreign working tree was used as source authority: adjacent repository policy and related unchanged facts were read through `git show` at the exact base. The later R6 subject was not inspected.

Only source/text, Git, archive-member and hash reads were performed. No product, URL parser, dependency code, test, build, lint, schema, generator, capture, benchmark or acceptance execution occurred locally or on htch-runtime.

## Findings and reasoning

**Findings: none.** There is therefore no severity/trigger/path/symbol/fix item to carry into a repair round. The following are the source facts that support the verdict, rather than runtime result claims.

**Concurrent Device work is preserved at the incoming base.** The R4→R5 patch restores the `device.token.rotate` App command arm, `UnifiedGatewayApi::rotate_device_token`, the existing Device test module, Device source bindings, owner/HTTP/retained-stream selections, three actual App names and Device ledger/RTM references. Comparison against the incoming base shows that these Device surfaces have no residual change in the R5 complete patch. The hook still delegates optional sealed authorization and the singular scope ceiling to the existing `DevicePairingOwner::rotate_token_with_authority`; R5 does not add a second Device owner. Workflow and portable-capture Device test additions relative to R4 likewise restore the incoming base rather than displacing its selection set. This is preservation evidence, not a fresh deep review or acceptance of the entire unchanged Device implementation.

**The Windows dependency oracle and the current product oracle are now distinct and compatible.** In `source/host/tests/openclaw_web_media_contract.rs::test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references` (approximately lines 270–286), the direct `url::Url` assertion preserves the locked converter's refusal of `file:///C|/`. The product loop separately requires all three incoming colon/encoded-colon positives, the incoming `file:/C|/` positive and the LFA triple-slash pipe positive. Every value is labelled in failure diagnostics. This adds no selected test name and reduces no capture count. The one older product negative is deliberately replaced by the LFA-03a positive under the explicit LFA-03b/FUA transition; the underlying dependency-negative claim remains asserted.

The checksum-bound `url` source supports that distinction: empty/localhost file-host handling can put the drive token beyond the first normalization condition, whereas the Windows converter accepts a colon or encoded colon in the first drive segment, not a pipe. The unchanged shared production owner performs the narrow Windows leading `/<ASCII letter>|/` path rewrite before native conversion, after original-host checks. Its URL setter preserves existing escapes and restores query/fragment offsets. I inspected that existing owner and these dependency APIs as adjacency; I did not execute them. The fixed-U Windows test file contains native uppercase/single-slash and refusal controls, with no pipe-specific oracle. The new product contract is not presented as an observed fixed-U pipe result or as full Node/Rust equivalence.

**The full edited web-test functions retain causal coverage.** Native temp-file bytes, exact native path and filename, Unicode/literal-percent preservation, actual direct service calls, active root checks, one-byte-over-cap refusal and successful subsequent calls remain in the native byte/service case. Added pipe spellings run through that same case, including query/fragment variants. Other malformed-authority, host-erasure, separator, bare-drive and bad-encoding refusals are retained. No production admission routine is replaced by a test-only converter to obtain the new positive.

**Composition keeps the existing admission and custody owners.** In App `resolve_production_attachment_source`, `Err` from the shared `resolve_local_file_url` propagates before `fs::read`, ForeverBox status/download, materialization or original-URL fallback. Both actual sinks consume that Result. SendMessage's established Session work-admission and pending-media custody/release path remains in place. The adjacent bridge still invokes `abort_attachment_custody` on a later image-resolution failure. AgentManagement's adjacent pure whole-image preflight completes before any per-image resolver call, which preserves its separate no-rollback constraint. R5 Device restoration touches separate command/hook/test-module locations and does not alter these media functions.

The shared owner continues to own URL parsing, authority refusal, strict pathname decoding, native absolute-path conversion and Windows spelling admission. Channel's final reanchored path guard remains before metadata/read. The web adapter retains its stricter original separator guard and typed errors before its existing private reader. No permission, durable-state, scheduler, registry or media-reading owner is introduced. Existing read-capability, TOCTOU, persistence-outcome and live-service obligations stay explicitly open.

**Capture gates and existing selections are preserved.** The complete portable-capture diff against base contains only the two LFA facade names and the corresponding SendMessage target expectation change from 15 to 17; the other target summaries stay 13 and 14. Required-positive uniqueness, command/capture exits, summary checks and total-name checks remain. The Device owner44/HTTP13/retained27 complete-target checks and all prior names are restored unchanged from base. In the workflow, the existing macOS recovery receipt adds the two facade positives and keeps command-evidence checks; Linux App adds six exact named LFA cases while retaining Device and other App selections. Each App case is still listed with `--exact`, run from the linked actual App test executable and required to complete with one pass, zero failures and zero ignored tests.

The unchanged native media capture binds the shared production/test/spec inputs, platform identity and fixed U, checks before/after input identity, qualifies its own oracle and demands the exact positive set with no unexpected/duplicate names and zero failures/ignored tests. The R5 test-table reconciliation does not alter those selections. Native Windows and Linux App execution remain required; these source checks do not substitute for them.

**No status or cost regression is introduced by this integration.** The ledger/RTM deltas add or restore responsibility descriptions, dependencies, invariants and references; they contain no status/global-flag promotion or gate deletion. Historical literal reading and verification fields are not rewritten as LFA success. R5 changes no shared-parser runtime implementation relative to R4 and restores the Device production hook already present in base. Added admission-related computation stays with the existing synchronous shared owner; no cache, background worker, new dependency or repeated filesystem operation is added. The existing AgentManagement batch preflight intentionally reuses the same pure converter before the direct sink's defensive check. The bounded extra URL work is linear in the input values and removes no ownership boundary. No benchmark or quantified runtime performance claim is made.

## Actual read scope

Before judging source, I read the exact-base repository AGENTS policy, the SOT authority/current-entry and applicable state-separation material, the complete main specification's effective §17, and both complete frozen LFA/FUA specs. Base policy blobs are AGENTS `e4682748087759b2d4bb2b3a47916b6148e8a6f2`, SOT `3748d734b10c9ed5b2d948467fbaa9483f1dd306`, and main specification `69c9667af2328e20a1fba09ebf80c5aab4d5c336`. The module map's relevant media/current-owner entries were also read.

All eight integration-delta files/hunks were inspected. For the large files this means the complete delta plus relevant edited functions and adjacent consumers, not every unrelated line of the 440 KB workflow, 991 KB App file or 1.45 MB ledger. Both specs were read in full. I also compared all eight focus paths against the incoming base to distinguish restored Device content from this slice's changes.

Adjacency content reads were limited to the media-reference converter/path guard; web URL admission/reader ordering; Channel resolver and file-context local resolver; AgentManagement batch preflight and sink dispatch; SendMessage build/abort paths; native media-capture inputs/selections/oracles; workflow triggers/exact-H/platform setup, affected recovery receipt and Linux App executable/selection checks; and the relevant dependency/fixed-U contracts. App LFA tests and the other unchanged contracts were identity-checked, with capture selection examined, but were not freshly deep-reviewed in full. This report does not claim a full current 21-path content review.

The reviewed dependency archive identities are `url-2.5.8.crate` SHA-256 `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed` and `fs-safe-0.27.0.tgz` SHA-256 `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`. Selected Rust members: `src/parser.rs` SHA-256 `2dd4210e1773fc5d3fdc15ce506ecff30dbf8eb358a79bbbc2835cc30d513c3a`; `src/lib.rs` SHA-256 `51f47a93654528f3b8cb92c94fc26ad25775860d84197fff0c3124a687f8c5e3`. Packaged `dist/local-file-access.js` SHA-256 is `7530a957ca9d56f56deba8da1dd06be03c894c6f6472bc8571080937b57bfda8`. Fixed-U Windows-test cache SHA-256 is `96fa78a117e2da707540777269c8fcf9395cd76ac845c1c4503f63fe9a25491a`. These are contract/actual-use reads, not complete dependency reading or recursive dependency closure.

## Exact focused identities

All values below were independently recomputed from frozen R5 bytes.

| Focus path | Git blob | SHA-256 |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `580eabc6ade055fc47850b9777814f131feb0594` | `af9cad942a4b5f9a91b4e5d3d1ad728b4982679426e9e8acfb526c1629eb3299` |
| `projects/openclaw-rust/docs/specs/local-media-file-url-admission.md` | `9249e4a93f353247e97c7ef47339382932451d72` | `146dadba7911290f7f78a4b6e436e02b1b0332754e7b219b62aba39690f65526` |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `5414d7193cc7cbcbcfec51dec54cda68be5130ad` | `637616e95d63ca7e57b75a9267dc2a7b98f0a71c7941f88e48d5547d33ebbdc6` |
| `projects/openclaw-rust/parity-ledger.json` | `3fb5f2b4b5cade7d15d3cbedacc01c8a5dcd1cf0` | `7aadc1a486ed5626d271434ca50790134d72f72c416cb20fb82c41c714c563c3` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `dddcfc37286add1e5c3be86bed9eb346057b0fbb` | `99777c80b3a643ce4cf67b5499a8ce43b82dedfc5209ce74e7b1fd59fe44e700` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `07da058409c318fcd1f19263b1a3520f0154cc7b` | `7fd42a01d20869accf7f9d1c8e110e8eca6a995a6b9907b77912c364b1d3648c` |
| `source/host/app/src/main.rs` | `4831ae8cf002ce64e6409f2d695b48aaa1914c45` | `36605022e09e91269ee17770a48fa9897e5ba4adcf5bbc7b21b6f382e3cdd4fa` |
| `source/host/tests/openclaw_web_media_contract.rs` | `4d7829bd42995f9503d6c9fee390986e8c1c724e` | `059c4c533b289f557b6ac4cef92428e0713835d98495f11c14aee0e99e359a91` |

## Remaining 15 identities

Each is byte-identical to R4 and matches its manifest Git blob and SHA-256. Identity continuity does not itself renew a content-review verdict.

| Unchanged path | Git blob |
| --- | --- |
| `projects/openclaw-rust/evidence/local-media-file-url-647e0932-20261103.json` (hash only) | `0722207d574b638c1e9984737f85cb05fa3aa3b7` |
| `projects/openclaw-rust/evidence/media-file-url-normalization-recovery-647e0932-20261103.json` (hash only) | `721190ec1e165a8fbdf9e5764d779796000e71e9` |
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

The two opaque evidence SHA-256 identities are respectively `bd7bbcdd2fd6b536599b636c0904bdab129668fd0965753fac83f95d5b966a40` and `68187df3c539f99703f0e7b06460b33884ee13dc1d8ddc499693832dff2d23fe`; their content provenance remains the root's serial audit responsibility.

## Pending evidence and acceptance boundary

Fresh exact published-H Actions must still supply native Windows execution of the dependency characterization, current LFA rooted-drive positives, malformed/alias zero-entry controls and actual native byte/service/cap/retry behavior. Linux must supply the actual App shipping check, linked test harness and all six selected LFA production-sink/custody/reopen cases with the retained Device and existing App controls. Linux/macOS native media coverage and every other selected required group remain required where applicable. Receipts must bind H/tree/U, the exact inputs, raw logs/hashes, command and capture exits, unique positive names and zero failure/ignored outcomes.

No earlier failed or passing receipt was reclassified here. Source inference about an unlabeled historical Windows iteration is not runtime observation. Missing, queued, refused or unexecuted jobs remain unverified. Full read authority, persistence unknown outcomes, original-turn/crash timing, live provider/channel/plugin/node/device, UI/security/performance/soak, supported package/install/upgrade/rollback, G01–G12, AC01–AC24, independent ACCEPT and shipping-SHA acceptance remain open under their original contracts.
