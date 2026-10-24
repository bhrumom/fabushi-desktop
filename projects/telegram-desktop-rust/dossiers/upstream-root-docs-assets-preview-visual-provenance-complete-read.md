# Upstream preview visual-resource provenance — strict index 112

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source entry: `docs/assets/preview.png@808d016f34fe2d4156a2bd26e8cdbe384f888b65` (1,241,708 bytes), exact immutable binary identity revalidated from the accepted upstream tree before publication.
- Parent exact HEAD: `27214da001dcb49abd0eb9e377254743439cd189`, tree `86e76f8979ecbc2d6b0c248d31dc80952e9bcf81`.
- Parent TDRP run `39933340394`: 61/61 success; authority `12492058206` sha256:b40f0abde900b2d4f8fc3ef062c51bdb96ac3a0e54dea04f412d897be12ec2fd; closure `12492773034` sha256:a21239415b85f6171ccc2c7f3180187d2e19bd6d978ffb6e2b47e202e6d99ff0; reachability `12493500318` sha256:defb53ae934190a68c8b2891647eebfb9a8d8425ddfd1fd55aecacdcf8a09f45.
- Parent Rust desktop runtime run `39933340523`: 6/6 success; artifact `12492398271` sha256:1ec1e02ba54dca9f3cae67cc8b8355aaa3a40fb6fb44be2cf7dbb874a423f0ce.
- The preceding strict111 child run exposed one accounting-only defect: its `fabushi_target_symbols` used prose/composite labels that the exact-source validator correctly rejected. Commit `27214da001dcb49abd0eb9e377254743439cd189` replaced them with real source symbols only; the fresh exact-head TDRP 61/61 and Rust 6/6 evidence above confirms that repair without production-code changes.
- Live upstream `dev`, the PR/ref lease and live Fabushi `main` were re-read immediately before publication. Upstream remained `a7f453612f4896ac04d0007a36ab0d377e5bcbcf/c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`; live Fabushi main remained `7394f6dc7687f9b986ee5589d40aa680ce23b90a/fbeb0bb32d944d0c1053880c471fe5c52279f052`.

## Binary asset role and provenance

The binary itself is not a runtime state machine. Its product responsibility comes from where the accepted upstream publishes it:

- `README.md@15a6194f73ae6084a2ab25d14005dc10262cdfb3` embeds the file as the repository's “Preview of Telegram Desktop” and links the preview to the raw image.
- `lib/xdg/org.telegram.desktop.metainfo.xml@c8b26f262bfdfa93f97b22f9aba4131073504223` names this same file as the default AppStream screenshot, making it part of upstream product/distribution presentation metadata as well as repository documentation.
- The accepted `docs/assets` tree binds this path to blob `808d016f34fe2d4156a2bd26e8cdbe384f888b65` and size 1,241,708 bytes.
- Path history is explicit: `7e680624a0f223370f93c72dcd92e78a2d8406d5` added the preview, `0f7567970cbf8c9e2d872365b49df6aee3c2caee` updated it, `b922839490700ad6e973389c742f2ada13ea8e8c` recompressed it while stating no visual-result change, and `f232d329c58bda86dcfef30ff205467120f1ec5f` added transparency.
- The repository root license blob is `LICENSE@d70f4d6a87eece19c4ec184de36a8aba72b4a639` (GPLv3 text), while the accepted `docs/assets` tree contains no asset-local LICENSE/NOTICE sidecar. This dossier therefore does not infer a broader standalone reuse right from repository placement.

The applicable source-neutral responsibilities are: retain exact visual-resource provenance; distinguish a repository/distribution preview from shipping UI truth; keep visual baselines bound to exact Fabushi evidence rather than imported screenshots; prevent external-source branding/assets from becoming Fabushi brand authority; and keep any future package-store screenshot metadata inside the existing release/visual evidence lifecycle.

## Existing-owner / validator uniqueness

The exact parent branch was re-read at the selected owners:

- `projects/fabushi-communication-platform/visual-acceptance.md@16855454ff7897f3aa205bc33f1672077e32955d` owns the fixed screenshot matrix, current-head visual evidence identity, content/brand acceptance, artifact bundle requirements and fail-closed visual acceptance.
- `projects/fabushi-communication-platform/design-system.md@bab234e73227686ba978f6d0b332e2fc896488fa` explicitly keeps logo/illustration color under Fabushi brand-asset authority and forbids external product brand icons from becoming Fabushi common UI assets.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` owns exact-head visual evidence identity and requires baseline/actual/diff provenance.
- `projects/fabushi-communication-platform/quality/release-entry-exit-criteria.md@c5eb9d0897234bb2fdad4f505549b2350c44aa72` remains the release gate for packaged functional/UI/visual/a11y evidence; an upstream AppStream screenshot reference cannot create a second packaging or acceptance owner.

No existing owner requires the Telegram image bytes. The correct migration is provenance-only: Fabushi must produce its own current-head screenshots from its own canonical product/design system when visual or store metadata is implemented. No Telegram preview asset, second visual-baseline store, second brand authority, second packaging metadata owner or Telegram-branded UI root is introduced.

## Fail-closed boundaries

- The upstream preview is not a Fabushi visual baseline, design token source, brand asset or proof of UI parity.
- The upstream AppStream reference does not prove Fabushi Linux package metadata, store publication, signing, installation or release acceptance.
- The asset is not copied into Fabushi; repository provenance and exact identity are retained instead.
- Root repository licensing is recorded without inventing asset-local licensing that was not observed.
- Visual acceptance remains open until current Fabushi screenshots satisfy the fixed matrix and exact-head evidence contract.
- Any source/ref/tree, README/metainfo reference, visual owner, brand-asset policy or release-evidence drift requires fresh reconciliation.

## Traceability

- Requirement: `TDRP-R9-UPSTREAM-PREVIEW-VISUAL-ASSET-PROVENANCE-001`.
- Oracles: `ORA-TDRP-PREVIEW-EXACT-BLOB-001`, `ORA-TDRP-PREVIEW-REFERENCE-ROLE-001`, `ORA-TDRP-PREVIEW-EXISTING-VISUAL-OWNER-001`.
- Invariants: `INV-TDRP-PREVIEW-NO-TELEGRAM-BRAND-ASSET-001`, `INV-TDRP-PREVIEW-NO-IMPORTED-BASELINE-001`, `INV-TDRP-PREVIEW-NO-RELEASE-ACCEPT-001`.

Closing strict112 yields unknown=16,264, unread=16,239, omitted=0, responsibilities=130. Next strict root index 113 is `docs/assets/slide.01.jpg@07ed727d48e286f9a04d933ac7203bdb7205ff4a` (477,277-byte binary visual asset). Fresh child exact-head TDRP/Rust evidence is required.
