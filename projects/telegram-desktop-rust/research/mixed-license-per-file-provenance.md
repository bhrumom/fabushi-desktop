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

## Current closure boundary

For the frozen fcitx5-qt commit, implementation-source licensing is now mapped by explicit file evidence rather than only by the two files present in `LICENSES/`. The remaining P0 mixed-license work is narrower but not closed globally: other mixed-license gitlinks (`hime`, `hunspell`, `kcoreaddons`, `kimageformats`, and any later source-derived dependency) still require exact-file mapping where behavior research depends on them, and Telegram resources/fonts/icons/themes/sounds still require their own source/license provenance.
