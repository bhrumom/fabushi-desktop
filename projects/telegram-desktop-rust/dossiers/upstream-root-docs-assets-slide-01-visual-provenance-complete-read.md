# Upstream AppStream slide 01 visual-resource provenance — strict index 113

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source entry: `docs/assets/slide.01.jpg@07ed727d48e286f9a04d933ac7203bdb7205ff4a` (477,277 bytes; JPEG 2880×1800), exact immutable binary identity revalidated from the accepted upstream tree.
- Parent exact HEAD: `bf535567d7d75d53bac960951ab82f9c70601b96`, tree `f180263f70c08e9d2ce69950ec83c5a0ab5be711`.
- Parent TDRP run `39935933661`: 61/61 success; authority `12493058592` sha256:730bfb80f5de9a3a02c78d910c9c0eefd1d3bdb515522094a279ac31ce2fe09a; closure `12493732242` sha256:512a3ffc7c886d25694bfd8ef1574c7726d2653ec5e9c44cc81d59ab3de379d9; reachability `12494157226` sha256:81cd0402bd7874272496dc76867db6aa51078a466d37b2566a30d6e29347993e.
- Parent Rust desktop runtime run `39935933679`: 6/6 success; artifact `12494226066` sha256:b0725680eacce09ea32e150ccd23d2d704f9c660ee99517d27820916e0e7d0b0.
- Publication preflight re-read PR/ref/tree, upstream and live Fabushi main: PR/ref remained `bf535567d7d75d53bac960951ab82f9c70601b96` / tree `f180263f70c08e9d2ce69950ec83c5a0ab5be711`; upstream `dev` remained `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`; live Fabushi main was `7394f6dc7687f9b986ee5589d40aa680ce23b90a` / tree `fbeb0bb32d944d0c1053880c471fe5c52279f052`.

## Binary asset role and provenance

The binary itself is not executable behavior. The exact JPEG was visually inspected: it is a blue-framed Telegram Desktop marketing slide showing a group conversation, conversation list, search, transcript, file attachment, sticker/media and composer, with the copy “A lightning-fast native app that seamlessly syncs with your other devices”.

- `lib/xdg/org.telegram.desktop.metainfo.xml@c8b26f262bfdfa93f97b22f9aba4131073504223` references this file as a supplemental AppStream screenshot and repeats the same sync caption.
- Exact-path history on accepted `dev` has one commit: `3661442acde378b36d9c5f8029d98e3334a9a657` (“Add more screenshots to metainfo”, 2024-12-04). That commit added `slide.01.jpg` plus `slide.02.jpg`–`slide.04.jpg` and added all four metainfo screenshot/caption entries.
- The accepted `docs/assets` directory contains preview.png and four slide JPEGs, but no asset-local LICENSE/NOTICE sidecar. Repository `LICENSE@d70f4d6a87eece19c4ec184de36a8aba72b4a639` is recorded. The metainfo declares `metadata_license=CC0-1.0` and `project_license=GPL-3.0`; neither field is promoted here into an inferred standalone license for the screenshot bytes.
- The embedded sync sentence is a product/distribution presentation claim, not proof that Fabushi currently implements or has accepted multi-device sync. Runtime sync/messaging responsibility remains with canonical executable owners and exact-head tests.

The applicable source-neutral responsibilities are: retain exact visual-resource provenance; preserve the screenshot/caption relationship; distinguish rendered marketing presentation from executable product truth; keep current Fabushi visual baselines bound to exact Fabushi evidence rather than imported screenshots; prevent source branding/assets from becoming Fabushi brand authority; and keep any future package/store screenshots in the existing visual/release evidence lifecycle.

## Existing-owner / validator uniqueness

The exact parent branch was re-read at the selected owners:

- `projects/fabushi-communication-platform/visual-acceptance.md@16855454ff7897f3aa205bc33f1672077e32955d` owns approved screenshot baselines, external-brand scanning, the fixed screenshot matrix and current-head visual evidence.
- `projects/fabushi-communication-platform/design-system.md@bab234e73227686ba978f6d0b332e2fc896488fa` owns Fabushi semantic visual language and excludes external product brand icons from common UI assets.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` owns source/baseline staleness and requires visual artifacts to carry baseline/actual/diff evidence and verdict context.
- `projects/fabushi-communication-platform/quality/release-entry-exit-criteria.md@c5eb9d0897234bb2fdad4f505549b2350c44aa72` remains the packaged UI/visual/a11y release gate; an upstream AppStream slide cannot create a second store/packaging or acceptance owner.

The validator enforces unique `target_path_set#symbol` ownership. Strict113 therefore binds to additional real exact-head symbols not consumed by strict112: “扫描用户可见 Telegram/Grok/Gok/其他来源品牌文案与 asset；合法 provenance 不属于产品 UI。”, “Approved baseline 必须来自已验收 design contract，不能把失败状态重新录成 baseline。”, “外部产品品牌图标不得成为 Fabushi 通用 UI asset。”, “相关 source/spec/oracle/baseline 改变后旧证据降 historical。” and “截图/视频存在但未关联 case/verdict 不算 pass。”. No parallel visual, brand, sync, store or Telegram runtime owner is introduced.

## Fail-closed boundaries

- The upstream slide is not a Fabushi visual baseline, design token source, brand asset or UI parity proof.
- The sync caption is not runtime/E2E evidence and cannot satisfy multi-device messaging acceptance.
- The AppStream screenshot reference does not prove Fabushi Linux package metadata, store publication, signing, installation or release acceptance.
- The asset is not copied into Fabushi; immutable provenance is retained instead.
- Neither metainfo licensing fields nor repository placement are treated as asset-local authorization that was not observed.
- Any source/ref/tree, metainfo reference/caption, visual owner, brand policy or release-evidence drift requires fresh reconciliation.

## Traceability

- Requirement: `TDRP-R9-UPSTREAM-SLIDE01-VISUAL-ASSET-PROVENANCE-001`.
- Oracles: `ORA-TDRP-SLIDE01-EXACT-BLOB-001`, `ORA-TDRP-SLIDE01-REFERENCE-CAPTION-ROLE-001`, `ORA-TDRP-SLIDE01-EXISTING-VISUAL-OWNER-001`.
- Invariants: `INV-TDRP-SLIDE01-NO-TELEGRAM-BRAND-ASSET-001`, `INV-TDRP-SLIDE01-NO-IMPORTED-BASELINE-001`, `INV-TDRP-SLIDE01-NO-MARKETING-AS-RUNTIME-PROOF-001`, `INV-TDRP-SLIDE01-NO-RELEASE-ACCEPT-001`.

Closing strict113 yields unknown=16,263, unread=16,238, omitted=0, responsibilities=131. Next strict root index 114 is `docs/assets/slide.02.jpg@2a19cfdeb4a831d8bbe553dede2b1cdae7aec40f` (430,861-byte JPEG, 2880×1800). Fresh child exact-head TDRP/Rust evidence is required.
