# Upstream macOS build instructions complete read — strict index 118

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `docs/building-mac.md@b8c6b6bae95b87f94d090fc73acd774d6307ef12` (1,500 bytes), fully read.
- Parent exact HEAD: `9c00291985cb3d1216fc26984e769c7dc0dd3b73`, tree `b24b3dfbd1a1d64af847bd3a002034123c560319`.
- Parent TDRP run `39945908779`: 61/61 success; authority `12497274631` sha256:68fed234157ce70ae51c0b7feca763a7b06c4d55d134052882ac13eccc5e8536; closure `12498596674` sha256:8f85766c29986e73d1fe86a807a0f9abd723716858285e3524c2f1ebbece7d79; reachability `12498507220` sha256:25229e1e889b03c2f38c3a87aabe1c5f92f2bf459268f1cd383b9911363c6ad7.
- Parent Rust run `39945908761`: 6/6 success; artifact `12498097968` sha256:84c9ee43d0af2bbcdcdc8e762bb36461bd489ac8c32440c82c78fd363ca38ec2.
- Publication preflight: PR/ref remained `9c00291985cb3d1216fc26984e769c7dc0dd3b73` / tree `b24b3dfbd1a1d64af847bd3a002034123c560319`; live main remained `7394f6dc7687f9b986ee5589d40aa680ce23b90a` / tree `fbeb0bb32d944d0c1053880c471fe5c52279f052`; live upstream dev remained `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

## Complete source responsibility

The document defines Telegram's local macOS developer build: reserve about 55 GB (about 35 GB libraries for x64+arm64 plus about 20 GB compiled output), provision Telegram `api_id`/`api_hash`, install Homebrew and the declared build tools, select full Xcode with `xcode-select`, recursively clone tdesktop, run `Telegram/build/prepare/mac.sh`, configure with the upstream API credentials, then open `out/Telegram.xcodeproj` and build Debug or Release.

At the accepted source, `prepare/mac.sh@82810b6cd5a13c78233232bee38602dcf1707de4` delegates to `prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b`. The Python owner selects/fences a macOS toolchain, SDK and deployment target, persists `Libraries/macos_toolchain.json`, and prepares universal x86_64/arm64 dependencies. `configure.sh@c550d988cff5fe3047311cc330aeee943ee328e5` delegates into the configured-build path. History review retained the dependency-script migration, Apple Silicon dependency repair, Homebrew-link/storage updates and the current minizip prepare.py move as provenance.

## Existing-owner-first mapping and signed/notarized validator uniqueness

- `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84` already owns the Fabushi macOS dmg/zip targets, hardened runtime, entitlements, forceCodeSigning and `afterSign`.
- `.github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643` already owns exact-source Developer ID certificate import, arm64 electron-builder packaging, Apple notarization, DMG ticket staple/validation, codesign/Gatekeeper checks, byte-exact installed-candidate binding and real signed packaged acceptance.
- `desktop/scripts/notarize-after-sign.cjs@5f140c6ff8587b4969e1611ec9030a8c5dc5a6f1` restores Fabushi-owned nested code identifiers, reseals the app, submits exact app bytes to `notarytool`, requires `Accepted`, staples/validates the ticket and re-verifies code signing.
- `.github/scripts/validate-apple-ticket.mjs@8e0279b3cf9e6447f54d2db59d51f7c538f2f593` is the bounded fail-closed ticket helper.
- All 25 exact-head workflow files were enumerated and scanned for certificate/electron-builder/notary/ticket/codesign/Gatekeeper ownership. `release-macos-main.yml` is the sole workflow that imports the signing certificate, builds the macOS package and submits notarization. `apple-ticket-validation-ci.yml@87f428cce0785d69ed64bc60dae170e02ebe84fc` only tests the ticket helper; `miniapp-packaged-iteration.yml@1cfcf7aa9250c4f2b3b9405c6cc42ff4d23b8b37` only validates/iterates an already signed candidate.
- No Telegram macOS runtime, second build graph, second signer/notary, second package owner, second credential owner or second release authority is added.

The selected package/sign/notary target anchors are unique at the parent exact head. Existing upstream strict88/strict89 macOS CI dossiers remain provenance; their unsigned/signature-stripped upstream artifacts do not become Fabushi release evidence.

## Traceability anchors

- Requirement: `TDRP-R9-UPSTREAM-BUILDING-MAC-BUILD-INSTRUCTIONS-PROVENANCE-001`.
- Oracles: `ORA-TDRP-BUILDING-MAC-EXACT-SOURCE-001`, `ORA-TDRP-BUILDING-MAC-EXISTING-PACKAGE-OWNER-001`, `ORA-TDRP-BUILDING-MAC-SIGNED-NOTARIZED-ACTIONS-001`, `ORA-TDRP-BUILDING-MAC-NO-LOCAL-EVIDENCE-001`.
- Invariants: `INV-TDRP-BUILDING-MAC-NO-TELEGRAM-CREDENTIAL-001`, `INV-TDRP-BUILDING-MAC-NO-SECOND-BUILD-RUNTIME-001`, `INV-TDRP-BUILDING-MAC-SIGNED-NOTARIZED-PACKAGE-OWNER-001`, `INV-TDRP-BUILDING-MAC-EXACT-HEAD-ACTIONS-ONLY-001`.
- Contracts: `CONTRACT-TDRP-BUILDING-MAC-EXACT-SOURCE-001`, `CONTRACT-TDRP-BUILDING-MAC-CANONICAL-PACKAGE-OWNER-001`, `CONTRACT-TDRP-BUILDING-MAC-SIGNED-NOTARIZED-ACTIONS-001`.
- Temporal/fault: `TEMP-TDRP-BUILDING-MAC-UPSTREAM-DRIFT-001`, `TEMP-TDRP-BUILDING-MAC-TOOLCHAIN-DRIFT-001`, `TEMP-TDRP-BUILDING-MAC-PACKAGE-WORKFLOW-DRIFT-001`, `FAULT-TDRP-BUILDING-MAC-RECURSIVE-CHECKOUT-001`, `FAULT-TDRP-BUILDING-MAC-PREPARE-TOOLCHAIN-001`, `FAULT-TDRP-BUILDING-MAC-SIGNING-NOTARY-001`.
- Security: `SEC-TDRP-BUILDING-MAC-CREDENTIAL-SEPARATION-001`, `SEC-TDRP-BUILDING-MAC-SIGNING-SECRETS-001`, `SEC-TDRP-BUILDING-MAC-LOCAL-BUILD-NO-AUTHORITY-001`.
- Regression: `REG-TDRP-BUILDING-MAC-NO-TELEGRAM-BUILD-RUNTIME-001`, `REG-TDRP-BUILDING-MAC-NO-SECOND-MACOS-OWNER-001`, `REG-TDRP-BUILDING-MAC-PACKAGE-GATE-STAYS-FAIL-CLOSED-001`.

These anchors name the ledger responsibility and keep the source dossier independently searchable by the authority validator. They do not upgrade source closure, package acceptance or release acceptance.

## Fail-closed boundaries

- No local/container/Mac/Windows build or test was run; all FBCP/TDRP executable evidence remains exact-head GitHub Actions only.
- Telegram Homebrew/Xcode/SDK/local disk/path instructions are developer-build provenance, not Fabushi shipping dependencies.
- Telegram `api_id`/`api_hash` are not Fabushi identity, account, signing or notarization credentials.
- Missing Developer ID or Apple notarization credentials is blocked, never pass.
- A local `out/Telegram.xcodeproj` Debug/Release build is not Fabushi package, installed acceptance or release evidence.
- Existing package/sign/notary source ownership does not itself prove a current release; package/notary/ticket/codesign/Gatekeeper failures remain blocking.
- Any source/history/build-input/package-workflow/PR/main/upstream drift requires fresh reconciliation.
- Independent release acceptance remains open.

Closing strict118 yields unknown=16,258, unread=16,233, omitted=0, responsibilities=136. Next strict root index 119 is `docs/building-mas.md@da7be145bb0a9cbd3c322fa7efe0c1f5729ae1bf` (921 bytes). Fresh child exact-head Actions are required.
