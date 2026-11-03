# R6 blind structure / reuse / performance review

## Terminal verdict

**APPROVED for the bounded R5 → R6 integration source review. No blocking or non-blocking findings.**

This verdict covers the complete two-path integration delta and the affected capture functions, workflow steps, result consumers and incoming historical-harness interfaces. It is not a fresh approval of the other 21 unchanged scope paths, a complete PRID implementation review, runtime verification, status promotion or global OCRM acceptance. Exact published-H Actions evidence remains required.

Task: `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, review round 6. Lens: structure, reuse and performance. Launch-assigned model/effort: `gpt-6.1-sol` / `xhigh`, as specified by the parent launch binding and the repository's pinned Codex lens policy. The worker did not start another model/CLI or change host settings; no independent model-introspection surface was used.

## Authority and immutable identities

| Subject | Identity |
| --- | --- |
| Repository / branch | `bhrumom/fabushi-desktop` / `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Fixed OpenClaw U / tree | `1440bb1771372334bea23d7642bb52deb7db1845` / `90c8aab39972da11cfad220361781d49e1bc6700` |
| Exact integration base / tree | `370c6680b6b8c072615875d72be67ca0e45ebe45` / `b31903209a22e8fbd77e5eb4a9a865723a98f54c` |
| Frozen R5 tree | `9a9afd4841590abc646805b8308c6e716821f067` |
| Frozen R6 candidate tree | `efd035bd168369022a1a8b2131511ef20daf1646` |
| R5 → R6 patch SHA-256 | `854aa9eddb54d7832f8032b2ddbab5588a555d32463d6082f6532a8ddce0591b` |
| Complete 23-path patch SHA-256 | `a4ecb44a11428ceb73b9f351a8bc2bf34a9ca191f6d301702763f09a0a2b1047` |

The two patch digests and both focus-file digests/blobs below were independently read with hash/Git operations. The base tree was independently resolved from the immutable base object. The manifest records a complete, non-truncated candidate-tree readback of 10,522 entries, all 23 changed blobs and base blobs matching, and the remaining 21 paths identical to R5. That complete-tree API acquisition belongs to the parent's snapshot authority; this reviewer did not rerun an API or metadata audit.

| Focus path | R6 Git blob | R6 SHA-256 |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `eebbd9c203d4869e23f737b741b6e56b20cce7f2` | `ae94acf00e6d9f4548ca18cdc232fa3eb0499d9606df2a28766bddb21eaa438d` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `62bc1cd60cf33a2f5a6c4ce6f8a61d70bbf546ad` | `69ef972c95109e49937c7bab74954a6cb25f3fe3c73afdadeadbe47ca99134d1` |

The workflow's R5 blob is `580eabc6ade055fc47850b9777814f131feb0594`; its base blob is `9c3cafd372d98a9266f623234d05a0ba3b93e2ad`. The portable capture's R5 blob is `07da058409c318fcd1f19263b1a3520f0154cc7b`; its base blob is `8fccfc567a356ade337e33bed935edd4e56fee3b`.

Applicable source authority read:

- Root `AGENTS.md`, immutable base blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2`, including independent/focused review, frozen scope, pinned lens and Actions-only rules. The base tree contains no more-specific `AGENTS.md` governing either focus path.
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`, immutable base blob `3748d734b10c9ed5b2d948467fbaa9483f1dd306`: current authority/precedence, fixed-U, owner and execution contracts; historical discovery chronology was supporting context, not current authority.
- Main migration spec, immutable base blob `69c9667af2328e20a1fba09ebf80c5aab4d5c336`, complete current §17 OCRM-EXEC-02.
- Complete R6 LFA-v1 spec, blob `9249e4a93f353247e97c7ef47339382932451d72`, including LFA-03a/LFA-03b; complete R6 FUA-v1 spec, blob `5414d7193cc7cbcbcfec51dec54cda68be5130ad`.
- Complete incoming PRID-v1 spec at the immutable base, `projects/openclaw-rust/docs/specs/canonical-prior-root-identity.md`, blob `472f9242e1dd72f225bed8ebbad724df3e46ef17`.

## Review depth and integration assessment

### Portable source custody and result handling

Deep read of the complete integration patch and affected `capture_portable_contracts.py` sections: input enumeration and deduplication (lines 1–227), the preserved SendMessage group (616 onward), incoming PRID group definitions (943–953), result parsers and canonical oracle qualification (969–1010), receipt/source identity construction and the complete execution/aggregation loop (1012–1087). Adjacent case definitions and the incoming base versions were read to establish that the restored PRID definitions and strict branch preserve the existing contract.

The new historical support-module and PRID-spec input entries participate in the existing sorted/deduplicated `inputs` set and the receipt's `inputs_sha256`. The checkpoint owner, process parent, Cargo inputs and generated protocol inputs were already in that source-binding set. This composes with the established receipt mechanism without a second source store or new evidence owner.

The four private writer-image controls and three current-writer/process parents exactly match the incoming base selections. Both groups use the existing execution/log/capture machinery. Their exact-count branch requires one successful summary with respectively `(4, 0, 0)` and `(3, 0, 0)`, in addition to zero command/capture exits and every required name occurring once. Their individual acceptance is included in the same final `all(case["oracle_accepted"])` aggregation used by all existing cases.

The process filter explicitly skips the two ignored export/import utilities. Source adjacency confirms that those two entries are ignored utilities requiring their bound absolute directories/provenance, while the three selected ordinary tests are the two current-writer positives and the diagnostic-specific rehashed semantic refusal. Utility successes therefore cannot inflate the ordinary three-positive gate.

The incoming PRID contract identifier is appended to the existing receipt's `additional_contracts`; its addition does not replace or remove an earlier contract. Source hashing and old/current process evidence remain distinct from product acceptance.

### Both LFA gates remain strict

The two facade cases `lfa_send_message_file_url_facade_preserves_admitted_metadata` and `lfa_send_message_file_url_facade_refuses_unsafe_local_sources` remain required in `generated-send-message-contracts` (lines 632–633). The strict acceptance branch remains `sorted(counts) == [(13, 0, 0), (14, 0, 0), (17, 0, 0)]` with `len(passed) == len(required)` (1061–1063). R6 adds PRID strict labels to another branch; it does not relax this LFA/SGF gate.

The existing App gate retains all six `tests::local_media_file_url_tests::...` selections, their LFA spec/product/test inputs, exact test listing, successful command requirement, one-pass/zero-failed/zero-ignored summaries and completed-case aggregation (workflow 3940–4305, especially 3965–3971 and 4249–4272). The historical step consumes the App gate only after it succeeds. There is no alternative path converting a failed LFA App result into a successful historical prerequisite.

The unchanged native `capture_media_cleanup.py` was read at its platform selection and aggregation boundary (395–477), and its relevant source input/list declarations were inspected as adjacent context. It retains the 16 existing native groups, platform-specific LFA cases, the common channel target once, and before/after-input equality plus all-group acceptance. R6 does not alter this owner or its tests.

### Historical step, retained binaries and downstream consumers

Deep read of workflow job setup, exact-HEAD/environment binding, shared-cache ownership, portable-result palette consumer, App executable/result production, inserted historical step and subsequent link-preview/result/artifact consumers. The new step (4306–4314) restores the incoming base's declared `capture_prior_root_contracts.py` execution after the actual App success, on the existing writer-push / `ocrm-config` dispatch scope. It uses the same owned native fixture root, existing Cargo target and serial build setting. It adds no product dependency or production owner.

The complete incoming historical script was read directly with immutable `git show`, not from the foreign working tree: blob `225cb2d0e1afa09f6ac24164dcea0fb28dedd0c2`, SHA-256 `ea7e9bd95264133c18f9a5daa6508a782a91dd28d60f23cf26f63cf2402462b1`. Interface/consumer adjacency was inspected in incoming support module blob `f6bfd9b91c1d1e0428e6827a186b3edd20a5eb06`, process parent blob `15a0fa812fa6f893a26a9f31cee41a3b284c832c`, and private-test name declarations in checkpoint-owner blob `6443bd7b8636babe665237ee1e6153652df3b924`.

That existing helper checks the current App receipt's successful exact commit/tree and executable digest, retains the App and H Host binaries before rebuilding exact historical BASE `e19a0a265e400a4c9b1d96520c84fb612a3843e8`, and resolves the Host artifact from a unique Cargo compiler-artifact record. It permits only the identified historical support/declaration overlay and compares all tracked source images before and after. The source checkout remains H while historical production builds in an owned detached BASE checkout, so downstream current-source commands do not silently adopt BASE source. The current App recovery utility uses the retained, digest-bound App executable. The subsequent link-preview capture continues to bind current HEAD and reuse the earlier same-H composition receipt rather than treating historical build outputs as its evidence.

The existing helper keeps separate old-producer, current-consumer, binary, fixture inventory and external archive digest identities. The inspected utility entry points preserve actual old-reader/current-reader/App-ack/App-no-op contracts, complete closed-state copies, unavailable recovery credentials and zero additional effect assertions. This is sufficient interface review for the step restoration; the incoming PRID business implementation is outside this review's approval scope.

### Failure artifacts and cost

The failure-fixture predicate now includes `steps.prior_root_identity.outcome == 'failure'` (4463), and the archive command restores the incoming `--exclude='original-turn-credential.json'` (4468). The parent harness creates that exact credential basename under owned fixtures. The historical support path removes it before recovery/export and checks exported state for credential values. The restored exclusion also protects failure paths before removal. The existing always-upload step still includes the historical `.evidence/sqlite-native-path/prior-root-identity` receipts, logs, overlay and archive.

The integration reuses the existing portable execution loop, process target, App build, cache and historical harness. It adds the contract-required four/three ordinary controls and historical evidence operation rather than duplicating a runner or ownership subsystem. Exact historical rebuilding and exhaustive tracked-source hashing have a real Actions cost, but they are explicitly required by incoming PRID and were already established in the base. No concrete new avoidable cost or demonstrated timeout regression was established by source review. The existing job/step budgets require observation at exact H; this review does not predict an Actions duration.

## Findings

None. No falsifiable severity/path/symbol/trigger/fix finding was established in the changed integration. Preference-only cleanup or speculative timing concerns were not promoted into findings.

## Blindness and execution limits

This was a fresh blind review. No prior/other review report, runner decision, arbitration/checkpoint, publication metadata or incoming review-decision evidence was read. The complete patch was hashed but its historical evidence JSON hunks were not read. The two historical evidence JSON contents were not opened; only manifest-supplied identities were observed:

- `projects/openclaw-rust/evidence/local-media-file-url-647e0932-20261103.json`: SHA-256 `bd7bbcdd2fd6b536599b636c0904bdab129668fd0965753fac83f95d5b966a40`.
- `projects/openclaw-rust/evidence/media-file-url-normalization-recovery-647e0932-20261103.json`: SHA-256 `68187df3c539f99703f0e7b06460b33884ee13dc1d8ddc499693832dff2d23fe`.

Only frozen snapshot source, explicit manifest/hash data and immutable Git objects were used as authority. The other repository's working tree was not used as source authority. No product code, metadata, status or artifact other than this report was written. No local or htch build, lint, test, schema check, capture, parser, generator or acceptance command was run.

## Exact published-H Actions still required

1. Publish/read back the frozen integrated H with the same candidate tree and focus-file identities, preserving concurrent incoming work and fixed U. Obtain the applicable exact-H scoped source/identity gates; this report does not supply G01/G02 runtime evidence.
2. Execute the retained native media capture on Linux, macOS and actual Windows. Require each unchanged native oracle, complete planned groups, unique required-positive names, zero failures/ignored results, command/capture exits, source inputs before/after and raw-log digests. Windows LFA-03a/LFA-03b drive cases require actual Windows execution; older subjects and source inference cannot substitute.
3. On Linux, execute the shared portable capture with the restored PRID image/process groups and the unchanged 17/14/13 SendMessage summary gate. Preserve its actual exact-H receipt and logs, including four/three PRID positives and exclusion of the ignored utilities from ordinary counts.
4. Execute the existing actual App compilation/harness gate and all six LFA App cases at H. Retain the exact App compiler-artifact identity and per-case results before the historical step consumes them.
5. Execute `prior_root_identity` at that H after App success. Require exact historical BASE source plus only its declared overlay, real two-case historical production and two old-reader checks, fresh H readers, App ack/no-op with unavailable credentials and no replay, both exact single-success utilities, source/binary/log/inventory/archive hashes and unchanged original historical payload. Utility completions are evidence operations, not extra product-test positives.
6. Retain downstream same-H result consumers and failure artifacts, including the credential-file exclusion; record actual job/step outcomes and any timeout/storage/environment failure without converting them into a pass. No exact-H result was inspected or claimed in this review.

Real provider/channel/device compatibility, full product/platform/package acceptance and global completion flags remain outside this bounded approval and retain their original gates.
