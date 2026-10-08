# TDRP Revision 9 complete read: application identity art 225-244

Authority: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

All twenty blobs were read in full and matched to the live deterministic sequence immediately following order 224. Exact byte metadata and blob SHAs are recorded in `inventory/source-dispositions.json`.

## Consumer and responsibility findings

- **240 icon_green.png / 242 iconbig_green.png** — exact current-tree code/QRC/build searches found no consumer. They remain legacy/orphan candidates with consumer and provenance review open; they are not marked omitted or not-applicable merely because their names look legacy.
- **241 icon_round512@2x.png** — `Telegram/Resources/qrc/telegram/mac_icons.qrc` embeds it and `settings/sections/settings_advanced.cpp@e06c1cc267c9b6ad63475bf6cf3caf75f4179246` uses it for the non-Mac-App-Store Advanced setting `advanced/round_icon`. The state machine compares the current custom-icon digest, calls `SetCustomAppIcon` or `ClearCustomAppIcon`, overrides the application icon, refreshes it, stores the digest, and saves settings.
- **225-228, 230-239** — `Telegram/CMakeLists.txt@7ffda8f085b8917889f5c1b06edbef160d975c5b` installs the size/scale matrix into Linux hicolor application icon locations. `Telegram/Telegram/Images.xcassets/Icon.appiconset/Contents.json@441d0c24ccd704976723496622b4657a10120f1c` binds the macOS 16/32/128/256/512 1x/2x inputs. `.github/scripts/generate_changelog.py@6cdd12652d31b2a47ed9a8ef906308af59cca3d0` additionally copies icon16/icon32 as generated changelog favicons.
- **229 icon256.ico** — `Telegram/Resources/winrc/Telegram.rc@28abde8cdc8d99796e7020f6991b4699b0c9b868` makes it the Windows application resource icon and `Telegram/build/setup.iss@b71266e876c7947ef1ab758d17faf229c64c508d` uses the same file as the installer setup icon.
- **243 logo_256.png / 244 logo_256_no_margin.png** — QRC runtime art. `Telegram/SourceFiles/window/main_window.cpp@ae2b4949734e6c8826310c96b65f51b630ab600d` exposes them via `Logo()` and `LogoNoMargin()`.

## Platform and ownership implications

The applicable responsibility is native application identity and shell integration, not Telegram artwork. Windows, Linux and macOS have distinct installer/resource, hicolor, asset-catalog and custom-Dock-icon contracts. Fabushi must keep one canonical product identity/packaging owner and use its own authorized artwork while preserving install, upgrade, shell-cache, runtime icon and preference behavior.

## Rights/provenance

The upstream repository provides GPLv3 `LICENSE@d70f4d6a87eece19c4ec184de36a8aba72b4a639` and `LEGAL@09c9e383db122fe0694e73c48d6224676afe60f6`, but the current tree does not provide a per-file artwork/trademark derivation grant for these assets. The migration records behavior and packaging responsibilities without treating Telegram-branded pixels as reusable Fabushi production assets.

## Accounting

Orders 225-244 are read-complete and responsibility-decomposed. Read prefix advances to 244 and unread becomes 15,876. No Fabushi exact-head production evidence was closed by reading these source assets, so unknown remains 16,033 and omitted remains 0.
