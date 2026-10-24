# Upstream AppStream slide 04 visual-resource and unlimited-media claim provenance — strict index 116

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source entry: `docs/assets/slide.04.jpg@d0598198b67a87638865f775f69a34fda792d493` (602,137 bytes; JPEG 2880×1800), accepted under `docs/assets@25c03ca28afb00315d8f85f908e1266abf9ef443`.
- Parent exact HEAD: `83b1bbd97a28b92db31e7e42ec0f766bf772d3ca`, tree `97e76b3567717c7b020cc701ea6dec9fe9bfa70c`.
- Parent TDRP run `39941133413`: 61/61 success; authority `12496043098` sha256:ba7f0cab8215381c0e73759c70062fcf70932d83c3e21b75158ad2a4352002fc; closure `12496417489` sha256:7b8e9b89dc2e2ca82226b2bffde6f3296e69a67ed1361b2a5bb1e727ed37365e; reachability `12496222074` sha256:02195b9661c55025bbd927a43a65e1f024e06507ca0d27cfde2d3375bf1df536.
- Parent Rust desktop runtime run `39941133415`: 6/6 success; artifact `12495064306` sha256:b6a62ce097692efdf26e5a080a98e9492687a051ccde0881a095f265bf769425.
- Publication preflight: PR/ref remained `83b1bbd97a28b92db31e7e42ec0f766bf772d3ca` / tree `97e76b3567717c7b020cc701ea6dec9fe9bfa70c`; upstream `dev` remained `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`; live Fabushi main remained `7394f6dc7687f9b986ee5589d40aa680ce23b90a` / tree `fbeb0bb32d944d0c1053880c471fe5c52279f052`.

## Binary asset role and provenance

The exact JPEG was read through the GitHub connector and visually inspected. It is a blue Telegram marketing slide with a conversation list on the left, a photo-grid/media collection on the right, and large embedded copy: “Send and store unlimited media and files of any type”.

- `lib/xdg/org.telegram.desktop.metainfo.xml@c8b26f262bfdfa93f97b22f9aba4131073504223` lines 54–55 reference `slide.04.jpg` as a supplemental AppStream screenshot and repeat that exact caption.
- Exact-path history is the single introduction commit `3661442acde378b36d9c5f8029d98e3334a9a657` (“Add more screenshots to metainfo”, 2024-12-04), shared with slide.01..03.
- `docs/assets` contains no asset-local LICENSE/NOTICE sidecar. Repository `LICENSE@d70f4d6a87eece19c4ec184de36a8aba72b4a639` and metainfo metadata/project-license fields are recorded without inventing a standalone screenshot reuse right.
- The source asset is presentation provenance. Its word “unlimited” is not executable storage/quota proof.

## Existing-owner-first executable truth

Current Fabushi exact-head behavior is bounded and has explicit owners:

- `source/shared/media/attachment-limits.ts@b4e6c2036503804d6d8605e203ff95deb898f347` defines `ATTACHMENT_BYTE_LIMIT = 25 MiB` and `VIDEO_BYTE_LIMIT = 200 MiB`.
- `frontend/src/recovered/features/conversation/workspace/desktop.ts@489b066d8cb3727dec1539c63409ef33aa4f90b8` owns `stageComposerFiles` / `commitComposerAttachments`; staged failures expose the 25 MB / 200 MB contract instead of claiming unbounded capacity.
- `source/electron-main/attachments/attachments.ts@764654b8f97b94215a015168c6ba82ae5e80aa45` owns `createAttachmentEdgePort`; it rejects staged bytes above the canonical per-file limit, commits through the existing upload leg, bounds preview reads and cleans partial failed downloads.
- `source/host/src/media_mime.rs@20f47798376971d53ecab97f888669f46f1b87df` reproduces the 25 MiB / 200 MiB server-side contract. `source/host/src/extensions/attachments/attachments_service.rs@783a78e21ad4c87db1930cadf3ba46dd6bbb806e` enforces the limit before ingest and owns the single agent attachment-store/read-chunk boundary.
- `native/mahayana-messaging/src/media.rs@b0d184c369ad55c1742a44026da00cdae379a8e7` owns transfer offset/verification/pause/fail/cancel state. `media_cache.rs@e578cb7dd5e9b5c7105f68e741f4cb1a71cd0ce2` requires a configured nonzero byte budget, evicts under budget and rejects oversized cache entries.

The target anchors `ATTACHMENT_BYTE_LIMIT`, `VIDEO_BYTE_LIMIT`, `stageComposerFiles`, `createAttachmentEdgePort`, `ingest_attachment` and `MediaCache` are unused by earlier ledger target-symbol rows at the parent.

## Test and fail-closed boundary

- `source/host/tests/media_mime_contract.rs@8f2ec26b01b38bc2f16ef563a8b6c9da39855e16` directly asserts the frozen 200 MB video / 25 MB ordinary-file contract.
- `source/host/tests/attachments_service_contract.rs@1cfcf453ef562cf13f05efca66b59d3ded981a51` verifies upload/read/chunk behavior through one agent media store.
- `native/mahayana-messaging/tests/m4_media_contract.rs@37811c22b7a1a318f216167f57711a3f6f8e57b4` verifies bounded-LRU cache and transfer/blob verification.
- Parent exact-head Rust run `39941133415` is 6/6 success and includes both native messaging and Host suites.

The upstream claim is stronger than current Fabushi behavior. Current limits stay authoritative until an approved canonical change proves otherwise. Bounded client cache is not durable cloud storage. Any equivalent no-user-visible aggregate quota, materially higher/unbounded service capacity or retention promise requires an explicit Fabushi server/storage contract covering authorization, abuse controls, retention, economics, persistence, deployment and real E2E evidence. Removing current guards merely to mirror the caption would be a regression.

No Telegram photo-grid UI, branded asset, second attachment/media store, second cache/transfer owner or second quota owner is introduced. The AppStream screenshot does not prove Fabushi package/store publication or release acceptance, and the JPEG is not copied.

## Traceability

- Requirement: `TDRP-R9-UPSTREAM-SLIDE04-VISUAL-ASSET-PROVENANCE-001`.
- Oracles: `ORA-TDRP-SLIDE04-EXACT-BLOB-001`, `ORA-TDRP-SLIDE04-REFERENCE-CAPTION-ROLE-001`, `ORA-TDRP-SLIDE04-EXISTING-ATTACHMENT-LIMIT-OWNER-001`, `ORA-TDRP-SLIDE04-UNLIMITED-CLAIM-GAP-001`.
- Invariants: `INV-TDRP-SLIDE04-NO-TELEGRAM-BRAND-ASSET-001`, `INV-TDRP-SLIDE04-NO-UNLIMITED-CLAIM-AS-CONTRACT-001`, `INV-TDRP-SLIDE04-NO-SECOND-MEDIA-STORE-001`, `INV-TDRP-SLIDE04-NO-RELEASE-ACCEPT-001`.

Strict116 source read/decomposition yields unknown=16,260, unread=16,235, omitted=0, responsibilities=134; the unlimited-storage/service equivalence remains explicitly open. Next strict root index 117 is `docs/building-linux.md@fbdf25355734f35617849224da106f1b9fafb3c9` (2,463 bytes). Fresh child exact-head TDRP/Rust evidence is required.
