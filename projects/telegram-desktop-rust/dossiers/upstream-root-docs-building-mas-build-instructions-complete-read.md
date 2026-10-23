# Upstream Mac App Store build instructions complete read — strict index 119

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `docs/building-mas.md@da7be145bb0a9cbd3c322fa7efe0c1f5729ae1bf` (921 bytes), fully read.
- Parent exact HEAD: `333a3d3cda2561ce9f78a15a89d35c0db62870ea`, tree `c12c99e9095591f553329ff8e0b223155646eaeb`.
- Parent TDRP run `39948699331`: 61/61 success; authority `12499786044` sha256:43043ad5fffae6aef3acfeea531c25e6459095ecdf659100ac7c9c1761e0c1e3; closure `12500360143` sha256:7a68345b695ebbe8d6c31746139c2f4cb9d1a1d75bd7ce32177bbaf1d047f284; reachability `12499308601` sha256:5e3b774072f76f6582e318e48011c4ab0bc463c8dbb5075aeeea3646ccb60d8c.
- Parent Rust run `39948699324`: 6/6 success; artifact `12499730947` sha256:71190f62168ffe12a74b84c35cdbe6334a5d1cfe0f849a3c5a8ecc77060ee167.
- Publication preflight: PR/ref remained `333a3d3cda2561ce9f78a15a89d35c0db62870ea` / tree `c12c99e9095591f553329ff8e0b223155646eaeb`; live main remained `7394f6dc7687f9b986ee5589d40aa680ce23b90a` / tree `fbeb0bb32d944d0c1053880c471fe5c52279f052`; live upstream dev remained `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

## Complete source responsibility

The source is a historical Mac App Store follow-on to the normal macOS developer build. It fixes `MACOSX_DEPLOYMENT_TARGET=10.12`, clones Breakpad and linux-syscall-support into the developer Libraries tree, checks out Breakpad `bc8fb886` and LSS `a91633d1`, applies `../patches/breakpad.diff`, and builds Breakpad Debug/Release plus `dump_syms` Release with Xcode.

Those dependency pins are provenance, not current dependency authority. The historical short pins resolve to Breakpad `bc8fb886486bb1c47cda103d75d86f21d6857920` and LSS `a91633d172407f6c83dd69af11510b37afebb7f9`. Current `Telegram/build/prepare/prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b` instead checks out Breakpad `dfcb7b6799b7c1e2c8d65e857d8afede185471d8` and LSS `e1e7b0ad8ee99a875b272c8e33e308472e897660` for the current stackwalk preparation path, and the current repository also owns `Telegram/build/prepare/breakpad/CMakeLists.txt@ab727514f749a53a761810a434e091c2ce850698`. The old MAS pins therefore remain historical-only provenance.

## Existing-owner-first mapping and MAS fail-closed gap

- `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84` already owns the source-neutral Electron `mas` target, App Store category, sandbox entitlement paths, forceCodeSigning and the configured `resources/mas/Fabushi_AppStore.provisionprofile` reference.
- `desktop/resources/mas/entitlements.mas.plist@b27cc6cf93c6a746b717f98bb609d201ccd134d2` owns the app sandbox, network, user-selected-file, microphone and camera entitlement boundary.
- `desktop/resources/mas/entitlements.mas.inherit.plist@96fce341a4b2fef8de4e28168d07e1423ad1e573` owns inherited child-process sandbox/JIT entitlement state.
- The referenced `desktop/resources/mas/Fabushi_AppStore.provisionprofile` does not exist at the parent exact head: the exact GitHub contents read returned 404. No repository bytes, generator or Actions injection path was found for that profile.
- All 25 exact-head workflow files were enumerated and content-scanned for MAS/App Store/provisioning/electron-builder MAS/signing markers. No workflow builds, signs or uploads a MAS target.
- `.github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643` is the only workflow matching macOS package/notary credential markers, but it runs `npx electron-builder --mac --arm64 --publish never`, then notarizes/staples a Developer ID dmg/zip candidate. It is not a Mac App Store shipping workflow.
- No Telegram/MAS runtime, second package graph, synthetic provisioning profile, fake App Store response or second release authority is introduced.

The applicable product responsibility is therefore split cleanly: existing Fabushi MAS metadata and entitlement owners are retained; provisioning, MAS signing/build/upload and App Store acceptance remain explicitly open until real exact-source shipping evidence exists.

## Traceability anchors

- Requirement: `TDRP-R9-UPSTREAM-BUILDING-MAS-BUILD-INSTRUCTIONS-PROVENANCE-001`.
- Oracles: `ORA-TDRP-BUILDING-MAS-EXACT-SOURCE-001`, `ORA-TDRP-BUILDING-MAS-HISTORICAL-PIN-DRIFT-001`, `ORA-TDRP-BUILDING-MAS-EXISTING-CONFIG-OWNER-001`, `ORA-TDRP-BUILDING-MAS-PROVISION-PROFILE-FAIL-CLOSED-001`, `ORA-TDRP-BUILDING-MAS-SHIPPING-WORKFLOW-FAIL-CLOSED-001`.
- Invariants: `INV-TDRP-BUILDING-MAS-NO-LEGACY-PIN-PROMOTION-001`, `INV-TDRP-BUILDING-MAS-NO-SECOND-BUILD-RUNTIME-001`, `INV-TDRP-BUILDING-MAS-NO-SYNTHETIC-PROVISIONING-001`, `INV-TDRP-BUILDING-MAS-NONMAS-EVIDENCE-NOT-MAS-001`, `INV-TDRP-BUILDING-MAS-EXACT-HEAD-ACTIONS-ONLY-001`.
- Contracts: `CONTRACT-TDRP-BUILDING-MAS-EXACT-SOURCE-001`, `CONTRACT-TDRP-BUILDING-MAS-CANONICAL-CONFIG-OWNER-001`, `CONTRACT-TDRP-BUILDING-MAS-PROVISIONING-BOUNDARY-001`, `CONTRACT-TDRP-BUILDING-MAS-SHIPPING-EVIDENCE-001`.
- Temporal/fault: `TEMP-TDRP-BUILDING-MAS-UPSTREAM-DRIFT-001`, `TEMP-TDRP-BUILDING-MAS-BREAKPAD-PIN-DRIFT-001`, `TEMP-TDRP-BUILDING-MAS-PACKAGE-WORKFLOW-DRIFT-001`, `FAULT-TDRP-BUILDING-MAS-BREAKPAD-ACQUISITION-001`, `FAULT-TDRP-BUILDING-MAS-MISSING-PROVISIONING-001`, `FAULT-TDRP-BUILDING-MAS-SIGNING-UPLOAD-001`.
- Security: `SEC-TDRP-BUILDING-MAS-PROVISIONING-IDENTITY-001`, `SEC-TDRP-BUILDING-MAS-SANDBOX-ENTITLEMENTS-001`, `SEC-TDRP-BUILDING-MAS-LOCAL-BUILD-NO-AUTHORITY-001`.
- Regression: `REG-TDRP-BUILDING-MAS-NO-TELEGRAM-BUILD-RUNTIME-001`, `REG-TDRP-BUILDING-MAS-NO-SECOND-MAS-OWNER-001`, `REG-TDRP-BUILDING-MAS-FAIL-CLOSED-UNTIL-SHIPPING-EVIDENCE-001`.

These anchors keep the source responsibility independently searchable without upgrading configuration into shipping acceptance.

## Fail-closed boundaries

- No local/container/Mac/Windows build or test was run; all executable evidence remains exact-head GitHub Actions only.
- Historical Breakpad/LSS pins are not current dependency authority after prepare.py drift.
- Missing `Fabushi_AppStore.provisionprofile` is blocking, never pass.
- Existing MAS package.json metadata and entitlement plists are configuration, not signed package/upload evidence.
- Existing Developer ID dmg/zip signing, notarization, ticket/codesign/Gatekeeper evidence is not MAS evidence.
- No MAS shipping workflow means no App Store package, upload, TestFlight/store acceptance or release acceptance can be claimed.
- Any source/history/pin/package/profile/workflow/PR/main/upstream drift requires fresh reconciliation.
- Independent release acceptance remains open.

Closing strict119 yields unknown=16,257, unread=16,232, omitted=0, responsibilities=137. Next strict root index 120 is `docs/building-win.md@d333539fc0709be6f98a8ef719f22fac0346b49a`. Fresh child exact-head Actions are required.
