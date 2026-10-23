# Upstream AppStream slide 03 visual-resource provenance — strict index 115

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source entry: `docs/assets/slide.03.jpg@588f2950d5f6fdd2625f2bd250db82761ce0e6f1` (466,747 bytes; JPEG 2880×1800), exact immutable binary identity revalidated from the accepted upstream tree.
- Parent exact HEAD: `347848c0207487c6acca95fab69bb0b685fed1de`, tree `97b13bbb65a2b6ebce6e9ae6e47891f2f150a6c8`.
- Parent TDRP run `39939734488`: 61/61 success; authority `12495123323` sha256:8959056af87ae58357a98d828d9015962a5494e47e19ec792aa012616c79986f; closure `12496121227` sha256:fdd7cc82665d634c1a1de5b06dea2c17fb2cb5f950afd0345c70c5517becd28b; reachability `12494844973` sha256:7ca50f261834398b0f9465ff554597b2099b58afca91c4aebcdcf7b2b37835d0.
- Parent Rust desktop runtime run `39939734546`: 6/6 success; artifact `12495646699` sha256:a70928b858522fe3ee9a3d4ca81c8551723076599ed5d7ac5a074cdcba320afe.
- Publication preflight re-read PR/ref/tree, upstream and live Fabushi main: PR/ref remained `347848c0207487c6acca95fab69bb0b685fed1de` / tree `97b13bbb65a2b6ebce6e9ae6e47891f2f150a6c8`; upstream `dev` remained `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`; live Fabushi main remained `7394f6dc7687f9b986ee5589d40aa680ce23b90a`.

## Binary asset role and provenance

The exact JPEG was visually inspected. It shows a Telegram Desktop full-screen photo viewer over a dimmed conversation with previous/next navigation, close, download/menu controls and photo metadata, plus the copy “A quick way to view photos, videos and files”.

- `lib/xdg/org.telegram.desktop.metainfo.xml@c8b26f262bfdfa93f97b22f9aba4131073504223` references the file as a supplemental AppStream screenshot and repeats the same caption.
- Exact-path history has one commit, `3661442acde378b36d9c5f8029d98e3334a9a657` (“Add more screenshots to metainfo”, 2024-12-04), shared with the four-slide screenshot set.
- The accepted `docs/assets` directory has no asset-local LICENSE/NOTICE sidecar. Root `LICENSE@d70f4d6a87eece19c4ec184de36a8aba72b4a639` and metainfo licensing fields are recorded without inferring an asset-local standalone reuse right.
- The media-viewer image/caption is presentation evidence only. It does not prove Fabushi Resource/Media viewer behavior, permissions, navigation, metadata, persistence, cancellation, storage or service semantics.

## Existing-owner / validator uniqueness

- `visual-acceptance.md@16855454ff7897f3aa205bc33f1672077e32955d` owns current-head visual baseline/evidence and structural acceptance.
- `design-system.md@bab234e73227686ba978f6d0b332e2fc896488fa` owns the canonical Fabushi visual language and accepted semantic layer.
- `quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` owns exact-head evidence identity and independent acceptance.
- `release-entry-exit-criteria.md@c5eb9d0897234bb2fdad4f505549b2350c44aa72` remains the packaged UI/visual/a11y release gate.
- Actual media/resource behavior remains with canonical Fabushi Resource/Media owners; this source binary does not establish a second runtime owner.

The selected exact-head target symbols are unique to this responsibility and are not consumed by earlier visual-asset rows. No Telegram media-viewer/store root, second visual baseline, second brand authority or second Resource/Media runtime is introduced.

## Fail-closed boundaries

- The slide is not a Fabushi visual baseline or Resource/Media implementation proof.
- The viewing caption is not functional/E2E evidence.
- AppStream presentation does not prove Fabushi package/store publication, signing, installation or release acceptance.
- The asset is not copied into Fabushi.
- Licensing is recorded without inventing asset-local authorization.
- Any source/ref/tree, metainfo reference/caption, visual/resource owner or release-evidence drift requires fresh reconciliation.

## Traceability

- Requirement: `TDRP-R9-UPSTREAM-SLIDE03-VISUAL-ASSET-PROVENANCE-001`.
- Oracles: `ORA-TDRP-SLIDE03-EXACT-BLOB-001`, `ORA-TDRP-SLIDE03-REFERENCE-CAPTION-ROLE-001`, `ORA-TDRP-SLIDE03-EXISTING-VISUAL-OWNER-001`.
- Invariants: `INV-TDRP-SLIDE03-NO-TELEGRAM-BRAND-ASSET-001`, `INV-TDRP-SLIDE03-NO-IMPORTED-BASELINE-001`, `INV-TDRP-SLIDE03-NO-MARKETING-AS-RUNTIME-PROOF-001`, `INV-TDRP-SLIDE03-NO-RELEASE-ACCEPT-001`.

Closing strict115 yields unknown=16,261, unread=16,236, omitted=0, responsibilities=133. Next strict root index 116 is `docs/assets/slide.04.jpg@d0598198b67a87638865f775f69a34fda792d493` (602,137-byte JPEG, 2880×1800). Fresh child exact-head TDRP/Rust evidence is required.
