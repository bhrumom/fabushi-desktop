# Incoming 071 media integration audit

**Purpose:** narrow, read-only integration audit for the current LFA shared admission work. This is not an independent R2 approval, not a complete review of commit 071, and not execution evidence. No local/htch validation or product execution occurred. No production file, test, workflow, ledger, or review report was modified or read beyond the explicitly allowed source context. Only this audit file was written.

Observed on 2026-11-03 at approximately 03:13–03:15 UTC. The R2 authoring directory changed during this interval as the parent integrated incoming work; observations below identify their source blobs and must not be mistaken for a frozen final R2 verdict.

## Exact source authority

GitHub commit readback confirmed incoming **071aeb6af1b223cceee9d81b5f7df3bad484d5a4**, parent **b55657cf87950ca014c2bfe827f5b043adbcab7e**, tree **7e1a329345062d57bdd91091b4a45ac2a943bc34**. The recursive tree response was nontruncated. Fixed U remains **1440bb1771372334bea23d7642bb52deb7db1845**.

The two principal source blobs were read directly from the GitHub connector. The matching local incoming source snapshots were hashed as Git blobs before reading. The additional exact identities below matched entries in the same GitHub tree.

| Source | Exact incoming blob | Read depth |
| --- | --- | --- |
| source/host/src/extensions/attachments/media_reference.rs | 71b19135ed156a589767c4e7f824dc48df698863 | Full source, including real bounded reader and all private reader controls |
| source/host/src/extensions/attachments/web_media.rs | 9912720f7c6eff6b53444b1912e3b291613d33e4 | Full source, including FUA guard, reader transition, roots, HTML/image consumers and private controls |
| source/host/src/extensions/attachments/attachments_service.rs | 75a49490ee80e7416df66df2431d2f3013f1b17c | Real loadWebMedia arm and four existing bounded-reader call sites |
| source/host/tests/openclaw_media_reference_contract.rs | df52afa0b1c727b2236dc2d96328047e10daab85 | Added public bounded-reader fixture and six contracts, lines 380–497; not full old-target reading |
| source/host/tests/openclaw_web_media_contract.rs | 4c7f27cc2ac3e8533a5befd5e2a9e7f841113ac6 | FUA native/refusal/service/alias tests, related roots context, and added reader/service tests; selected ranges 80–390 and 790–1025 |
| projects/openclaw-rust/docs/specs/media-bounded-read-safety.md | f08949e5d478bbb04060f186e37b52c7fdeb3a5e | Full specification |
| projects/openclaw-rust/docs/specs/media-file-url-admission.md | af7a5ffac2435d673923c2b55010bcf84e23faf0 | Full specification including parser-normalization amendment |
| projects/openclaw-rust/scripts/capture_media_cleanup.py | 03f5385ee4687fc35a15136c98856a46248f30bf | Full source, names/input bindings/oracle and execution command selection |

Additional caller excerpts were retrieved by exact tree blobs: box_staging.rs **6339dd944730c07e880883c61b97a6778fc25ead**, generate_image_resource_accessor.rs **b7d1d911c1c269cd0e2171e80991b76ca4bdb048**, and channel_attachment.rs **356e901feb2328729f882afe487fbe0199f4a40a**. Only the bounded-reader call chain excerpts, not those entire modules, were reviewed.

Principal full-file SHA-256 identities are:

- media_reference.rs: **6cea1f5d8541e8312fc4441104ed1ba44ee97e1af6be6ef3d094634d77942041**, 30,533 bytes.
- web_media.rs: **7f71eb5280fa2f0b8160713b1575f2eb9a639f73faf008f41a22198d9ba67633**, 28,096 bytes.
- attachments_service.rs: **416e8af1d383433e37809244c4bc538c36d945f0a2d13229225b2f5c46e1a508**, 87,484 bytes.

The pre-integration LFA bytes observed for comparison were media_reference.rs **8d547f434ddb15e8119b724554a346e5eeb3bce0**, web_media.rs **041078abfae537ab8a9a9a0044c8fec9970d9494**, and capture_media_cleanup.py **31eeb21a7cb4a58c05adfd403bf762d4f77832bc**. The local LFA specification and App resolver excerpt were also read. Later reads already showed the parent carrying the incoming bounded reader into the mutable R2 directory; this audit neither fixes nor approves that new mutable state.

## Integration hazards that need an explicit preservation decision

### 1. A whole-file LFA replacement would remove the incoming reader repair

Both changes extend the existing media_reference owner. Preserve the incoming reader interface and its entire implementation/private-control tail while adding the pure LFA resolver and native path admission functions. In particular, keep std::io self/Read imports required by the incoming helper, rather than restoring an older import or old max_bytes-plus-one reader.

The shared LFA functions must remain pure. Do not add canonicalization, metadata, file opening or a Box operation to resolve_local_file_url/admit_local_media_path. Conversely, do not silently insert the LFA spelling policy into the generic bounded reader for all its historical callers: that changes a separate common-reader contract and may reject previously admitted paths. Place each consumer's explicit admission before its existing filesystem boundary.

Strict UTF-8/percent validation concerns the URL pathname, not the file body. The reader must continue returning arbitrary bytes, including the NUL/invalid-UTF-8 body asserted by br_public_usize_max_returns_original_bytes.

### 2. The pre-integration LFA web guard loses incoming localhost separator refusals

Incoming file_url_has_extra_leading_path_separator inspects the leading path shape after an optional authority. The earlier LFA has_excess_file_url_separators only counts separators immediately after the scheme. That earlier helper fails to preserve the incoming refusal class for:

- file://localhost//C:/safe/image.png
- mixed slash/backslash forms after LOCALHOST
- file://local<TAB>host/<CR>/<LF>C:/safe/image.png

The dependency normalizes those initial empty path segments. Shared native conversion alone does not recover the original spelling needed by this stricter existing web policy. Preserve the incoming original-input check and all matching zero-reader-entry/whole-target assertions. This is a source-confirmed integration regression risk, not a locally executed reproduction.

Keep the incoming drive-token exemption as well. On Windows, file://C://dir is a drive path with an ordinary interior repeated separator; it must not be mistaken for a remote authority followed by forbidden initial empty segments. Existing native service positives deliberately exercise ordinary interior separators, valid rooted drive forms, and query/fragment components containing encoded separators. Those components do not become pathname input.

### 3. Moving the incoming guard requires retaining its file-scheme precondition

The incoming helper begins by splitting at a colon; its original caller has already established the file scheme. The LFA resolve_web_media_file_url wrapper is invoked for every media source. Copying that helper and calling it unconditionally before the shared Result resolver could reclassify non-file input such as an HTTPS URL with extra slashes as invalid-file-url.

Preserve non-file Ok(None) behavior. Check the original separator shape only after the shared pure resolver has identified a file source, or retain a file-scheme precondition consistent with that parser's normalization. In either arrangement the check must still happen before the local reader and all filesystem operations. This retains one structural URL owner instead of introducing a competing parser.

### 4. Error projection and consumer authority must remain distinct

File URL failures stay WebMediaError::LocalAccess/invalid-file-url. Existing raw or workspace-derived network failures stay network-path-not-allowed. LFA's additional raw-path alias/NUL refusal keeps its documented path-not-allowed projection. Conversion failure must not become Ok(None), a manually reconstructed path or successful original-URL passthrough.

The final Windows workspace check must continue accepting rooted VerbatimDisk while refusing UNC/VerbatimUNC/device forms. It is followed by LFA's pure final reanchored-path guard before reader entry. Neither WebMediaOptions flags nor successful LFA parsing supply roots, a read capability, a principal or filesystem identity.

The App production resolver excerpt still uses fs::read and its existing media materialization/Box fallback. Incoming BR fixes the eight existing shared-reader call sites; it does not automatically add a cap to App reads or resolve persistence-failure/unknown-write fallback. Do not state those as delivered by this merge.

## Required I/O and state order

The merged web path must preserve this sequence:

1. Existing reference cleanup and raw network classification.
2. Shared file URL classification/conversion and native spelling admission; retain the original-input web separator policy. Non-file sources retain existing workspace/native-path resolution.
3. Existing resolved Windows namespace/network classification plus LFA final-path admission.
4. Enter the private local reader exactly once for admitted input, with the same options. Every URL refusal remains at zero entries.
5. If local_roots is set and sandbox_validated is false, canonicalize the target and allowed roots and reject an outside target before the bounded reader. Do not move this into the generic reader or remove it because LFA succeeded.
6. Compute the existing per-call/default byte cap and enter the existing common reader.
7. On a successful complete byte read, perform filename extraction, explicit HTML actual-byte policy or existing image handling. A failed read never reaches successful format/publication output.

Inside read_outbound_media_file_bounded retain all of the following:

| Stage | Required operation and observable behavior |
| --- | --- |
| Preflight | Path metadata; require an ordinary file and admitted length. Nonregular gives NotFound, oversize gives InvalidData. |
| Open | A fresh read-only OpenOptions handle; Unix O_NONBLOCK is set before the open. Ordinary symlinks and hardlinks remain supported. No O_NOFOLLOW or blanket Windows reparse prohibition is added. |
| Actual-handle admission | Recheck ordinary-file kind and length with metadata on the opened handle, before buffering. |
| Read | Continue through that same handle. A second path-based open/read would discard the actual-handle check and nonblocking guarantee. |
| Loop | Read bounded chunks from a 64 KiB scratch buffer; retry Interrupted; a short read is not EOF. |
| Cap boundary | When accumulated bytes equal max_bytes, consume at most one sentinel byte without computing max_bytes + 1 and without appending the sentinel. Only EOF succeeds; an extra byte is InvalidData. |
| Allocation | Start from an empty Vec; reserve fallibly based on actual input, first request at most 64 KiB, then bounded amortized growth. Do not reserve the entire metadata length or repeatedly copy the whole accumulated prefix. |
| Failure | Preserve the real I/O error and discard partial returned bytes. Allocation refusal is OutOfMemory, not TooLarge or empty success. |
| Release/retry | Release only the attempt's owned handle on all returns. A later call opens fresh data; no cache, staging file, marker, background worker or replay state is introduced. |

Actual-handle admission is not original-path identity confinement. A later in-cap file-content change is not an immutable snapshot. O_NONBLOCK addresses the owned Unix FIFO replacement scenario; it is not a guarantee of general synchronous-read cancellation or hostile Windows-device deadlines.

## Actual service cap conversion and callers to preserve

The real AttachmentsService loadWebMedia arm must keep:

Value::as_u64 → optional usize::try_from → transpose → explicit conversion error.

An unrepresentable cap returns exactly **loadWebMedia.maxBytes exceeds the supported byte-count range**. Do not restore an as-usize cast, saturate, substitute a default, or impose a new lower universal maximum. Existing omitted/non-u64 JSON handling remains default behavior; the tests retain null, negative, string, fractional and boolean controls. The conversion precedes load_web_media, and htmlPublication/optimizeImages forwarding is unchanged.

Preserve the eight current common-reader call sites and their existing result projection:

| Caller | Existing limit and projection |
| --- | --- |
| Attachment path ingestion | attachment_byte_limit_for_name; InvalidData → TooLarge, other errors → existing I/O error; write only after successful read |
| Rooted attachment image read | Existing path containment/MIME/limit; reader error remains None |
| Rooted attachment video read | Existing containment/MIME/VIDEO_BYTE_LIMIT; reader error remains None |
| Rooted image dimensions | Existing containment/MIME/limit; reader error remains None |
| Box staging | 50 MiB limit, eligibility and metadata checks; read failure skips before uploads/staged entries |
| Generated image resource read | Existing agent media-store containment and per-name cap; error becomes the existing Error result |
| Channel attachment preparation | Existing nonempty-file/channel cap plus metadata and MIME rules; failure remains None; add LFA final admission before its metadata |
| Local web reader | Existing explicit/default cap and prior root gate; InvalidData → TooLarge, other read errors → existing not-found local access projection |

This is a caller-expression census for the inspected tree, not proof of external channel/Box/provider acceptance.

## Assertions and capture obligations that cannot be lost

Keep the exact incoming private reader controls:

- br_loop_short_reads_and_interrupted_preserve_bytes
- br_loop_zero_and_one_over_probe_without_allocating_extra_bytes
- br_loop_late_read_error_discards_prefix
- br_loop_allocation_failure_discards_prefix
- br_loop_growth_is_bounded_and_amortized
- br_initial_allocation_ignores_large_metadata_hint
- br_opened_admission_rechecks_size_after_preflight
- br_opened_file_growth_is_not_successful_prefix
- Unix: br_unix_opened_admission_rejects_directory_replacement
- Unix: br_unix_open_owns_nonblocking_descriptor
- Unix: br_unix_fifo_replacement_after_preflight_does_not_block

The FIFO test first proves ordinary-file preflight, then substitutes an owned FIFO and invokes the real opener in the same exact selected test in an owned child. Keep its deadline, owned release/kill/reap fallback, exclusive acknowledgment and success/blocked assertions. The child stdout/stderr must remain in BR_FIFO_CHILD_OUTPUT_BASE64 and the parent group must retain --show-output. A zero-test child success or nested printed positive cannot satisfy the parent oracle.

Keep all six added public bounded-reader tests for zero/exact/over, usize::MAX, missing/directory, fresh replacement, hardlinks and native symlinks. Native Windows symlink creation is a required actual fixture; privilege failure is not a synthetic pass.

Keep the added actual-service tests for u64::MAX, zero/exact/default caps, failed-read/fresh retry, and Unix ordinary-alias/root refusal. A 64-bit success does not validate the unrepresentable-cap branch on native 32-bit.

Retain incoming FUA zero-entry cases under all root/sandbox combinations and the real native-byte/service/cap-recovery controls, including the localhost-double-separator negatives, valid root/drive cases, literal percent and Unicode filenames, query/fragment positives, exact original options, reader-error single invocation, and rooted VerbatimDisk positive.

Merge the capture by existing groups and owners:

- Keep bounded-reader-owner, bounded-reader-public, bounded-reader-box-consumer and bounded-reader-image-consumer selections and all incoming input identities.
- The incoming bounded-reader-channel-consumers and LFA channel-attachment-consumers target the same whole channel_attachment_contract. Preserve every old and new case in one authoritative full-target selection instead of accidentally restoring an old four-name list or needlessly running two identical whole-target groups.
- Preserve the expanded complete web-media target name union, LFA shared/adapter/AgentManagement groups and separate real App receipt. The native media receipt does not prove App execution.
- Keep strict per-group positive-name uniqueness, missing/extra rejection, one complete summary, zero failed/ignored counts, command and capture exit checks, matching exact-H before/after input identities, and all thirteen synthetic qualification controls.
- Recalculate the final source-selected group/name expectations only after freezing the merged candidate. Neither the old BR 117/110 expectations nor the earlier LFA 129/128 expectations are the merged result, and no source-selected count is executed evidence.

## Boundaries and next verification

No runtime run or artifact for 071 was fetched by this narrow audit. Nothing here converts the earlier H497 zero-job failure, a source review, or a mutable R2 snapshot into an exact-H native pass.

The parent must finish the serial source/test/capture/spec integration, freeze its full tree/diff and send that candidate through the required fresh review and GitHub Actions gates. Preserve all received 071 production changes and historical failures. The final subject still needs native platform receipts, real App consumer checks and the existing authority/coverage conditions before any scoped verified claim. Full source/dependency/legal/permissions/recovery/platform/ecosystem and global G01–G12/AC01–AC24 acceptance remain independent.
