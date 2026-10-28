# Upstream Linux build instructions complete read — strict index 117

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `docs/building-linux.md@fbdf25355734f35617849224da106f1b9fafb3c9` (2,463 bytes), fully read.
- Parent exact HEAD: `a8cc41d290561b36368e06c83d913c5ac0ada4f8`, tree `1c274243afd9866f7e5caf71b85c8a05be7eab6f`.
- Parent TDRP run `39943268249`: 61/61 success; authority `12496339838` sha256:4418187b47d33c504631439efe7bd38903734c39bb2810f37065f6ae54233bf2; closure `12497212855` sha256:931a0e500781faecb2f5e4e76320b386961d220d40256a7eff41edab5a6603ab; reachability `12496923446` sha256:6971847bd29a02c5dfc223f61dfd9c1462c3ab624dbdedb0b365ded18f550fea.
- Parent Rust run `39943268259`: 6/6 success; artifact `12496832678` sha256:acd65428eaff6fba277803a8a75a38b19b5350d91cd55bb888f39256f3ac2190.
- Publication preflight: PR/ref remained `a8cc41d290561b36368e06c83d913c5ac0ada4f8` / tree `1c274243afd9866f7e5caf71b85c8a05be7eab6f`; upstream remained `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`; live main remained `7394f6dc7687f9b986ee5589d40aa680ce23b90a` / tree `fbeb0bb32d944d0c1053880c471fe5c52279f052`.

## Complete source responsibility

The document defines Telegram's local Linux developer build: choose a BuildPath, provision Telegram api_id/api_hash, install Poetry, recursively clone source, run `Telegram/build/prepare/linux.sh`, run the generated `tdesktop:centos_env` container as the caller uid with an exact source bind mount, configure Release or `CONFIG=Debug`, and consume `out`. Its optional VS Code section reuses that build image as a Dev Container and injects the same upstream API credentials into CMake editor settings.

Same-head dependencies are pinned in the ledger: `prepare/linux.sh@e4d2f920e4becdff6292d296efe6f21455a19fcb`, `build.sh@00b5c9a2faf5e25fb981325f654d3458ce074a7d`, `configure.sh@c550d988cff5fe3047311cc330aeee943ee328e5`, Dockerfile `3fea9e6866e0ea6351f8a13ebb825ebf49cadfc2`, generator `86d27c96e1f732a1faf3e85397ea3ba38e493a0d`, pyproject `8f43692fb217f075ccc588c3b677177e42b5688f`, and lock `9d857912774d68b1f8adda1dbbccb87adefda788`.

History review preserved the behavior-changing provenance for the prepare-script handoff, quoted bind mount, caller uid mapping, Dev Containers integration and current Debug configuration.

## Existing-owner-first mapping

- `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84` already owns Linux package metadata: `syncDesktopName=true`, AppImage, deb and deb packageName `fabushi`.
- `.github/workflows/rust-desktop-runtime.yml@905fc0e17e5b4e54cb18708c55d1eb8725d18f83` already owns exact-head Electron/renderer/co-staged runtime build verification.
- strict84/strict86 already own upstream Linux build-container/CI provenance; strict111 already proves Telegram api_id/api_hash are upstream credentials only.
- No Telegram Docker/Linux runtime, second build system, second credential owner or second release authority is added.

The selected package/workflow target anchors were rechecked against the parent ledger and are unique.

## Traceability anchors

- Requirement: `TDRP-R9-UPSTREAM-BUILDING-LINUX-BUILD-INSTRUCTIONS-PROVENANCE-001`.
- Oracles: `ORA-TDRP-BUILDING-LINUX-EXACT-SOURCE-001`, `ORA-TDRP-BUILDING-LINUX-EXISTING-BUILD-PACKAGE-OWNER-001`, `ORA-TDRP-BUILDING-LINUX-NO-LOCAL-EVIDENCE-001`, `ORA-TDRP-BUILDING-LINUX-LINUX-PACKAGE-EVIDENCE-GAP-001`.
- Invariants: `INV-TDRP-BUILDING-LINUX-NO-TELEGRAM-CREDENTIAL-001`, `INV-TDRP-BUILDING-LINUX-NO-SECOND-BUILD-RUNTIME-001`, `INV-TDRP-BUILDING-LINUX-CONFIG-NOT-PACKAGED-EVIDENCE-001`, `INV-TDRP-BUILDING-LINUX-EXACT-HEAD-ACTIONS-ONLY-001`.

These anchors name the already-recorded ledger responsibility and make the source dossier independently searchable by the authority validator; they do not upgrade Linux packaged acceptance or add a new implementation owner.

## Linux packaged-evidence gap

All 25 current workflow files were enumerated and inspected for AppImage, deb and Linux electron-builder packaging. Only `release-macos-main.yml` invokes electron-builder, with `--mac --arm64`. Thus Linux AppImage/deb metadata exists and exact-head source/runtime build verification exists, but no current Actions workflow produces or installs Linux AppImage/deb artifacts. Linux packaged acceptance remains open.

## Fail-closed boundaries

- No local/container/Mac/Windows build was run.
- Telegram Docker/Poetry/CMake/editor details are provenance, not Fabushi shipping dependencies.
- Telegram API credentials are not Fabushi credentials.
- A local `out` directory is not FBCP/TDRP evidence.
- Linux package metadata is not signed/installed/released evidence.
- Any source/history/build-input/package/workflow/PR/main/upstream drift requires fresh reconciliation.
- Independent release acceptance remains open.

Closing strict117 yields unknown=16,259, unread=16,234, omitted=0, responsibilities=135. Next strict root index 118 is `docs/building-mac.md@b8c6b6bae95b87f94d090fc73acd774d6307ef12` (1,500 bytes). Fresh child exact-head Actions are required.
