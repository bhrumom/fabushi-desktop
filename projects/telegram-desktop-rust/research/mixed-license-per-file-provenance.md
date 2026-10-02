# Mixed-license per-file provenance

Frozen Telegram root: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`  
Status: P0 research evidence; first exact per-file implementation mapping completed for `fcitx/fcitx5-qt`, broader mixed-license/resource review remains open  
Captured: 2026-10-02

This file is provenance evidence, not a legal compatibility conclusion and not permission to copy upstream code or assets. Telegram dependencies remain research inputs unless the Fabushi production/build graph actually adopts them.

## Method

For a mixed-license gitlink, inspect the repository at the exact commit pinned by `upstream.lock.json`, enumerate SPDX-bearing implementation files, separately classify generated files and legacy full-text notices, and retain unclassified build/resource metadata as an explicit open set instead of assigning a repository-wide license by inference.

## `fcitx/fcitx5-qt@0285a5d18367d8f3af80dab7e4819d5555982340`

The exact pinned commit was fetched on htch-runtime and verified with `git rev-parse HEAD`. The tree contains 215 tracked files. An exact `git grep` found 88 tracked files with explicit `SPDX-License-Identifier` lines. The implementation-file identifiers observed at this commit are:

| SPDX identifier | Explicitly tagged files | Relevant examples |
| --- | ---: | --- |
| `BSD-3-Clause` | 46 | `common/fcitxflags.h`; Qt4/Qt5/Qt6 DBus type/watch/input-context implementation files; Qt5 platform input-context/theme/key files |
| `LGPL-2.1-or-later` | 40 | Qt5/Qt6 GUI wrapper, immodule probing, widgets addons, Qt6 quickphrase editor, `test/testkeytrans.cpp` |
| `GPL-2.0-or-later` | 2 | `qt5/platforminputcontext/font.cpp`, `qt5/platforminputcontext/font.h` |

This disproves any repository-wide single-license shortcut for the pinned dependency: at least BSD-3-Clause, LGPL-2.1-or-later and GPL-2.0-or-later occur on implementation files in the exact frozen tree.

### Generated DBus proxy files

Twenty tracked `.cpp/.h` files contain the exact `qdbusxml2cpp` generated-file notice but no SPDX identifier. They are kept separate from hand-authored SPDX mapping:

- Qt4 DBus addons: `fcitxqtinputcontextproxyimpl.{cpp,h}`, `fcitxqtinputmethodproxy.{cpp,h}`.
- Qt5 DBus addons: `fcitxqtcontrollerproxy.{cpp,h}`, `fcitxqtinputcontextproxyimpl.{cpp,h}`, `fcitxqtinputmethodproxy.{cpp,h}`.
- Qt5 platform input context: `fcitx4inputcontextproxyimpl.{cpp,h}`, `fcitx4inputmethodproxy.{cpp,h}`.
- Qt6 DBus addons: `fcitxqtcontrollerproxy.{cpp,h}`, `fcitxqtinputcontextproxyimpl.{cpp,h}`, `fcitxqtinputmethodproxy.{cpp,h}`.

Their headers identify the generator and command line; they are not silently assigned the license of neighboring files. If Fabushi ever copies or adapts one of these generated outputs, the generator/input/output provenance and applicable Qt/generator terms require dedicated review.

### Legacy full-text LGPL notices without SPDX identifiers

Six implementation files under Qt5/Qt6 widgets addons have a full GNU Lesser General Public License notice and upstream/fork copyright text but no `SPDX-License-Identifier` line:

- `qt5/widgetsaddons/fcitxqtkeysequencewidget.cpp`
- `qt5/widgetsaddons/fcitxqtkeysequencewidget.h`
- `qt5/widgetsaddons/fcitxqtkeysequencewidget_p.h`
- `qt6/widgetsaddons/fcitxqtkeysequencewidget.cpp`
- `qt6/widgetsaddons/fcitxqtkeysequencewidget.h`
- `qt6/widgetsaddons/fcitxqtkeysequencewidget_p.h`

These remain recorded by their literal notice rather than normalized to an SPDX conclusion in this engineering ledger.

### Explicitly unresolved metadata/input files

The exact tree also contains code-adjacent files with neither an SPDX identifier, a qdbus generated marker, nor the above LGPL full-text notice. They include `CMakeLists.txt` files, package config templates, DBus XML interface definitions, desktop templates, JSON templates and quickphrase metadata. They are not materialized as a guessed license assignment. Their provenance remains open if one of those files materially informs or is copied/adapted by a Fabushi production change.

## `hime-ime/hime@9b3e6f9ab59d1fe4d9de73d3bf0fed7789f921c5`

The exact pinned commit was fetched and verified on htch-runtime. Its README states `LGPLv2.1 (Qt immodules are GPLv2)`. The two Qt5 input-module public headers inspected at this exact commit, `src/qt5-im/hime-imcontext-qt.h` and `src/qt5-im/hime-qt.h`, carry an explicit GNU GPL version 2 notice, matching that repository-level exception instead of the LGPL default.

The icon trees are separately licensed assets rather than being inferred from source-code licensing. `icons/30x30/COPYING`, `icons/black/COPYING`, `icons/blue/COPYING`, `icons/dark/COPYING`, `icons/gray/COPYING`, and `icons/pink/COPYING` have identical SHA-256 `acdb77f0d233c377321768b02732a200427387fb59b98876e3244a6c27633298` and state GNU LGPL v2.1 or later for the icon work. `distro/dev-tools/icons/COPYING` carries the same license statement with SHA-256 `75bbac0d99d7d4fd82ecc3b8bb0079970646ce52aa90c315f6759823dc4d8150`.

This closes the previously coarse hime source-vs-Qt-immodule-vs-icon license boundary at the frozen commit. It does not imply that Fabushi ships these icons or Qt modules; current PR #26 adoption evidence still says no Telegram resource tree is distributed.


## `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`

The exact pinned commit was fetched and verified on htch-runtime; it contains 854 tracked files. Because this tree does not use SPDX identifiers, the exact per-file map is recorded in `research/license-maps/hunspell.tsv` rather than inferring one repository-wide license. The scanner records every tracked path and classifies explicit file-header evidence separately from files with no detected notice.

The core Hunspell/parser/tool implementation is predominantly covered by the literal `MPL 1.1/GPL 2.0/LGPL 2.1` tri-license header: 59 tracked files match that exact header at the frozen commit. `README.md` independently describes Hunspell as licensed under an LGPL/GPL/MPL tri-license. The repository also carries distinct top-level `COPYING`, `COPYING.LESSER`, `COPYING.MPL`, `license.hunspell`, and `license.myspell` texts. Files without an explicit detected file-local notice remain `UNRESOLVED-NO-FILE-NOTICE` in the map; that is an evidence state, not an inferred license.

## `KDE/kcoreaddons@fd84da51b554eac25e35b1e3f373edaab3029b15`

The exact pinned commit was fetched and verified on htch-runtime; it contains 377 tracked files. `research/license-maps/kcoreaddons.tsv` records every tracked path. Exactly 200 files carry explicit SPDX identifiers at this commit. Observed identifiers include `LGPL-2.0-or-later` (104), `LGPL-2.0-only` (45), `LGPL-2.1-only OR LGPL-3.0-only OR LicenseRef-KDE-Accepted-LGPL` (23), `GPL-2.0-only OR GPL-3.0-only OR LicenseRef-KDE-Accepted-GPL` (12), `LGPL-2.1-only WITH Qt-LGPL-exception-1.1 OR LicenseRef-Qt-Commercial` (5), `LGPL-2.1-only` (4), `CC0-1.0` (3), plus one each of `MPL-1.1 OR GPL-2.0-or-later OR LGPL-2.1-or-later`, `LGPL-2.0-only OR LGPL-3.0-only OR LicenseRef-KDE-Accepted-LGPL`, `BSD-3-Clause`, and `BSD-2-Clause`. Untagged paths remain explicitly unresolved rather than inheriting a neighboring file's license.

## `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec`

The exact pinned commit was fetched and verified on htch-runtime; it contains 375 tracked files. `research/license-maps/kimageformats.tsv` records every tracked path. Exactly 52 files carry explicit SPDX identifiers: 34 `LGPL-2.0-or-later`, 8 `LGPL-2.1-only OR LGPL-3.0-only OR LicenseRef-KDE-Accepted-LGPL`, 4 `BSD-2-Clause`, 3 `LGPL-2.1-or-later`, 2 `BSD-3-Clause`, and 1 `CC0-1.0`. Product-relevant format implementations demonstrate the mixed boundary directly: `avif.cpp/.h` and `jxl.cpp/.h` are BSD-2-Clause, while `heif.cpp/.h` and most legacy format handlers are LGPL-2.0-or-later. Untagged metadata/build paths remain explicit unresolved rows.

## Current closure boundary

For the frozen fcitx5-qt commit, implementation-source licensing is now mapped by explicit file evidence rather than only by the two files present in `LICENSES/`. The remaining P0 mixed-license work is narrower but not closed globally: any later source-derived mixed-license dependency still requires the same exact-file treatment where behavior research depends on it, and Telegram resources/fonts/icons/themes/sounds still require their own source/license provenance.
