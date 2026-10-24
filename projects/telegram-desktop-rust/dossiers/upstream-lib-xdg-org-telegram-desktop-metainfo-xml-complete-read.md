# Strict124 complete read: lib/xdg/org.telegram.desktop.metainfo.xml

Source: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`  
Path/blob: `lib/xdg/org.telegram.desktop.metainfo.xml@c8b26f262bfdfa93f97b22f9aba4131073504223` (8,227 bytes)  
Read status: complete  
Parent Fabushi head/tree: `3d136f7703caaa6c9943c292767262f7a90a386d` / `5eef1e22e01fb4bbae4951081de759aa469aea84`

## Responsibility decomposition

The upstream AppStream document carries several distinct responsibilities. They are not imported as one Telegram-branded metadata file:

- application catalog identity: component id, name, summary, categories, developer identity and launchable id;
- product description / marketing claims: feature, reliability, privacy, security, scale and licensing claims;
- project/support URLs: homepage, FAQ, bug tracker, translation, VCS and contribution links;
- branding metadata: light/dark primary colors;
- storefront screenshots and captions;
- discovery keywords;
- declared environment requirements/capabilities: minimum display length, always-on internet, keyboard, pointing device and touch;
- OARS content rating assertions;
- platform provides: executable, D-Bus service and URI/media handlers;
- metadata/project license declarations.

## Existing-owner audit

- `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84`: canonical Fabushi package identity (`com.ombhrum.fabushi`, `Fabushi`), description/homepage, Linux AppImage/deb targets, Linux category/maintainer, package icon and `fabushi` protocol declaration.
- `.github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643`: exact-source signed/notarized package creation and immutable-artifact / packaged-acceptance / factory-promotion release authority. It is macOS-only and does not prove Linux AppStream installation.
- `desktop/e2e/openbot-packaged-acceptance.spec.ts@4075f76ec7e4b74e2e219cb3f49e984bd96132a6`: exact candidate source binding plus signed installed production renderer acceptance, screenshots/video/diagnostics.
- `desktop/e2e/miniapp-packaged-acceptance.spec.ts@2663a61e3c1f5284d2a30403b6c633b3df6dc807`: exact signed app/source identity and installed/offline/account-isolation evidence.
- `projects/fabushi-communication-platform/design-system.md@bab234e73227686ba978f6d0b332e2fc896488fa`: Fabushi semantic visual authority; brand logo/illustration colors are separately owned and cannot be inferred from legacy/external colors.
- `projects/fabushi-communication-platform/visual-acceptance.md@16855454ff7897f3aa205bc33f1672077e32955d`: user-visible external brand/copy/asset scan and current-head visual evidence contract.
- `projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md@283858bb70bab14e240da781a20e749ed1f1359e` and `projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md@c181c90ac52043975ef508042f4c20181211d9f5`: fail-closed shipping/release acceptance authority.

A complete recursive read of the current Fabushi exact tree found no shipping AppStream/metainfo/appdata file, no generated `.desktop` file tracked in source, and no Linux package/release acceptance workflow. Therefore there is no evidence for a second AppStream owner and no evidence that Linux catalog metadata is implemented.

## Source-specific statements that are not Fabushi authority

The following remain provenance only and MUST NOT be copied as Fabushi claims:

- `org.telegram.desktop`, Telegram/FZ-LLC identity, Telegram URLs and Telegram keywords;
- Telegram screenshots, captions and upstream asset URLs;
- Telegram primary colors;
- Telegram scale/performance/reliability/privacy/security/crypto/free/open/unlimited-size claims;
- Telegram OARS ratings;
- Telegram binary, D-Bus name and `tg` / `tonsite` / `ton` handlers;
- Telegram GPL-3.0 project-license statement as a substitute for Fabushi package/repository licensing authority.

## Current Fabushi disposition

Applicable source-neutral responsibilities map to existing owners only:

1. package/product identity -> `desktop/package.json`;
2. release truth / exact binary provenance -> existing exact-source release and packaged acceptance workflows;
3. user-visible brand/style/copy/screenshots -> Fabushi design-system and visual-acceptance authority;
4. protocol/platform capability claims -> existing package/runtime owners, but only when direct packaged/installed evidence exists;
5. system requirements/content-rating/storefront declarations -> release metadata responsibility remains open because current exact-head has no AppStream shipping artifact or Linux installed acceptance.

No new Telegram/AppStream runtime, package root, brand root, content-rating root, release root or URI/DBus owner is justified.

## Evidence boundary and blockers

- `desktop/package.json` declaring Linux AppImage/deb targets and a `fabushi` protocol does not prove an installed AppStream record.
- Current exact-head package acceptance is macOS/signed-candidate focused; it does not validate generated Linux metainfo, desktop entry, D-Bus activation, URI registration or store ingestion.
- There is no current shipping AppStream/OARS artifact to validate.
- No direct current-head evidence establishes Fabushi storefront marketing/security/privacy claims equivalent to Telegram's prose.
- Source closure and independent release acceptance remain open.

## Traceability

Requirement: `TDRP-R9-UPSTREAM-XDG-APPSTREAM-METAINFO-PROVENANCE-001`

Oracles: `ORA-TDRP-XDG-APPSTREAM-EXACT-SOURCE-001`, `ORA-TDRP-XDG-APPSTREAM-PACKAGE-OWNER-001`, `ORA-TDRP-XDG-APPSTREAM-BRAND-AUTHORITY-001`, `ORA-TDRP-XDG-APPSTREAM-RELEASE-EVIDENCE-001`, `ORA-TDRP-XDG-APPSTREAM-CLAIM-BOUNDARY-001`

Invariants: `INV-TDRP-XDG-APPSTREAM-NO-TELEGRAM-BRAND-001`, `INV-TDRP-XDG-APPSTREAM-NO-UNPROVEN-MARKETING-CLAIMS-001`, `INV-TDRP-XDG-APPSTREAM-NO-UNPROVEN-OARS-001`, `INV-TDRP-XDG-APPSTREAM-NO-TELEGRAM-DBUS-OR-SCHEMES-001`, `INV-TDRP-XDG-APPSTREAM-NO-MANIFEST-AS-LINUX-E2E-001`, `INV-TDRP-XDG-APPSTREAM-NO-SECOND-PACKAGING-OWNER-001`

Contracts: `CONTRACT-TDRP-XDG-APPSTREAM-EXACT-SOURCE-001`, `CONTRACT-TDRP-XDG-APPSTREAM-CANONICAL-OWNERS-001`

Security/regression: `SEC-TDRP-XDG-APPSTREAM-NO-FALSE-SECURITY-CLAIM-001`, `REG-TDRP-XDG-APPSTREAM-NO-EXTERNAL-BRAND-001`, `REG-TDRP-XDG-APPSTREAM-NO-DECLARATION-AS-E2E-001`

## Parent exact-head evidence

TDRP run `39961896876`: 61/61 success; authority `12504409984` / `sha256:f6d3682a6922b59c8277b76a9605532ec3984900aca7306ac37d81a78efd4368`; closure `12504769659` / `sha256:2f3f4af69d1a20f8ae3a6f5a1a27ac97b05444b5dc6430fd0febb4baac12d30f`; reachability `12505835838` / `sha256:f5dd5c8959cc2a042a53a8c94430f2167033664166c7b1dbd08dca0a41f5a42e`.

Rust run `39961896845`: 6/6 success; artifact `12505247386` / `sha256:8deadb2c32d85289e8b2c0f0ee49bfc9c05cb6b8388810c4b3633f9ccffe5489`.

Live main before publication: `7394f6dc7687f9b986ee5589d40aa680ce23b90a` / `fbeb0bb32d944d0c1053880c471fe5c52279f052`. Live upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

## Disposition

Strict124 is mapped-open, not accepted. It closes semantic read/responsibility decomposition for this one source entry only.

Counters become unknown=16,252, unread=16,227, omitted=0, responsibilities=142.

Next deterministic root entry: `lib/xdg/org.telegram.desktop.service@8935a4fa834df4e12cbf4f5f466758a14d2c2357` (84 bytes).
