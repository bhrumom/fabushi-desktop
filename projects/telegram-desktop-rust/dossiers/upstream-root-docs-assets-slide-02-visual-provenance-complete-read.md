# Upstream AppStream slide 02 visual-resource provenance — strict index 114

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source entry: `docs/assets/slide.02.jpg@2a19cfdeb4a831d8bbe553dede2b1cdae7aec40f` (430,861 bytes; JPEG 2880×1800), exact immutable binary identity revalidated from the accepted upstream tree.
- Parent exact HEAD: `8ab1deff05209e37ca1a5bfb77a512b71a856b82`, tree `eff1e410c41fea750988c17b0012a8e7d9423b2c`.
- Parent TDRP run `39938129941`: 61/61 success; authority `12494439036` sha256:32b3f414ed7008271a55d9307f5670febe7d7bd5a61b4f88630ec10e6e490375; closure `12495276289` sha256:61a215f5f463d14b291eb8e9894edb64ff6b3a77b680608e7400c625fb93a209; reachability `12493859452` sha256:b48c8c23438b775e8fb3e7f6e6b13762e4a044d8aa7b8f8a6427c3293caca321.
- Parent Rust desktop runtime run `39938130025`: 6/6 success; artifact `12494801788` sha256:8c35c1bea24dd08ef9046fcf50d2f8d5ac1c9069b54619f254fd49d53834ac88.
- Publication preflight re-read PR/ref/tree, upstream and live Fabushi main: PR/ref remained `8ab1deff05209e37ca1a5bfb77a512b71a856b82` / tree `eff1e410c41fea750988c17b0012a8e7d9423b2c`; upstream `dev` remained `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`; live Fabushi main remained `7394f6dc7687f9b986ee5589d40aa680ce23b90a` / tree `fbeb0bb32d944d0c1053880c471fe5c52279f052`.

## Binary asset role and provenance

The binary itself is not executable behavior. The exact JPEG was visually inspected: it is a blue-framed Telegram Desktop marketing slide showing a direct conversation with conversation list/search, an image, voice/audio message, document attachment, reply/text content, delivery/read marks and composer, with the copy “A fully autonomous Telegram app, no phone connection required”.

- `lib/xdg/org.telegram.desktop.metainfo.xml@c8b26f262bfdfa93f97b22f9aba4131073504223` references this file as a supplemental AppStream screenshot and repeats the same autonomy/no-phone caption.
- Exact-path history on accepted `dev` has one commit: `3661442acde378b36d9c5f8029d98e3334a9a657` (“Add more screenshots to metainfo”, 2024-12-04). That commit added `slide.02.jpg` together with `slide.01.jpg`, `slide.03.jpg` and `slide.04.jpg`, and added all four metainfo screenshot/caption entries.
- The accepted `docs/assets` directory contains preview.png and four slide JPEGs, but no asset-local LICENSE/NOTICE sidecar. Repository `LICENSE@d70f4d6a87eece19c4ec184de36a8aba72b4a639` is recorded. The metainfo declares `metadata_license=CC0-1.0` and `project_license=GPL-3.0`; neither field is promoted here into an inferred standalone license for the screenshot bytes.
- The embedded autonomy/no-phone sentence is a product/distribution presentation claim, not proof that Fabushi currently satisfies independent account/session/sync semantics. Those responsibilities remain with canonical executable owners and exact-head tests.

The applicable source-neutral responsibilities are: retain exact visual-resource provenance; preserve the screenshot/caption relationship; distinguish rendered marketing presentation from executable product truth; keep current Fabushi visual baselines bound to exact Fabushi evidence rather than imported screenshots; prevent source branding/assets from becoming Fabushi brand authority; and keep any future package/store screenshots in the existing visual/release evidence lifecycle.

## Existing-owner / validator uniqueness

The exact parent branch was re-read at the selected owners:

- `projects/fabushi-communication-platform/visual-acceptance.md@16855454ff7897f3aa205bc33f1672077e32955d` owns approved screenshot baselines, external-brand scanning, the fixed screenshot matrix and current-head visual evidence.
- `projects/fabushi-communication-platform/design-system.md@bab234e73227686ba978f6d0b332e2fc896488fa` owns Fabushi semantic visual language and excludes external product branding from common UI assets.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` owns exact-head evidence identity, source/baseline staleness and independent acceptance evidence.
- `projects/fabushi-communication-platform/quality/release-entry-exit-criteria.md@c5eb9d0897234bb2fdad4f505549b2350c44aa72` remains the packaged UI/visual/a11y release gate; an upstream AppStream slide cannot create a second store/packaging or acceptance owner.

The validator enforces unique `target_path_set#symbol` ownership. Strict114 therefore binds to additional real exact-head symbols not consumed by strict112/113: “用户可见外部品牌 asset 无非法残留。”, “本文件定义 gate，不代表仓库已经实现 screenshot harness、token lint 或 visual baseline。后续实现只能在 GitHub Actions 取证。”, “这些是 migration input，不是来源品牌永久 API。必须在不破坏现有行为的前提下建立 Fabushi-owned semantic layer。”, “通用 UI 只使用一个 canonical icon registry/family。” and “独立验收记录 reviewer、reviewed evidence IDs、异常与 verdict。”. No parallel visual, brand, account/session/sync, store or Telegram runtime owner is introduced.

## Fail-closed boundaries

- The upstream slide is not a Fabushi visual baseline, design token source, brand asset or UI parity proof.
- The autonomy/no-phone caption is not runtime/E2E evidence and cannot satisfy account/session/sync independence acceptance.
- The AppStream screenshot reference does not prove Fabushi Linux package metadata, store publication, signing, installation or release acceptance.
- The asset is not copied into Fabushi; immutable provenance is retained instead.
- Neither metainfo licensing fields nor repository placement are treated as asset-local authorization that was not observed.
- Any source/ref/tree, metainfo reference/caption, visual owner, brand policy or release-evidence drift requires fresh reconciliation.

## Traceability

- Requirement: `TDRP-R9-UPSTREAM-SLIDE02-VISUAL-ASSET-PROVENANCE-001`.
- Oracles: `ORA-TDRP-SLIDE02-EXACT-BLOB-001`, `ORA-TDRP-SLIDE02-REFERENCE-CAPTION-ROLE-001`, `ORA-TDRP-SLIDE02-EXISTING-VISUAL-OWNER-001`.
- Invariants: `INV-TDRP-SLIDE02-NO-TELEGRAM-BRAND-ASSET-001`, `INV-TDRP-SLIDE02-NO-IMPORTED-BASELINE-001`, `INV-TDRP-SLIDE02-NO-MARKETING-AS-RUNTIME-PROOF-001`, `INV-TDRP-SLIDE02-NO-RELEASE-ACCEPT-001`.

Closing strict114 yields unknown=16,262, unread=16,237, omitted=0, responsibilities=132. Next strict root index 115 is `docs/assets/slide.03.jpg@588f2950d5f6fdd2625f2bd250db82761ce0e6f1` (466,747-byte JPEG, 2880×1800). Fresh child exact-head TDRP/Rust evidence is required.
