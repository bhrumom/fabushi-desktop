# desktop-app/MicroTeX latex.qrc complete read — strict188

## Authority

- Fabushi PR: `bhrumom/fabushi-desktop#154`
- Branch: `tdrp-r9-entry31-includes-r27-20261020`
- Pre-receipt live HEAD: `909cf2db0ee137f4067389e784912a6d8c81fe23`
- Pre-receipt tree: `31a83b86df8044a57ba2eb3765d4547a4a99dc7b`
- Live main observed before write: `16aae12ea258f6a872a46aa47824766f64e1bdca` (tree `82126ec4a712fca352a49d6a3caae70c7d21ca8c`)
- Accepted source pin remains `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf` (tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`).
- Direct child pin remains `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`.
- CMake helper pin remains `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095`.
- Target: `latex.qrc@cf58a5aea4e53ed0e76ce3de2bbd9ab821af3270`, mode `100644`, fully read at 75 line slots / 3375 characters.
- Pre-write independent ledger registration count for this target/blob: **0**.
- Pinned MicroTeX recursive tree: `truncated=false`, 315 entries (279 blobs in the established inventory).

This dossier is source/resource provenance. It does **not** close the separately owned production formula renderer, resource packager, or release acceptance.

## Exact Qt resource manifest

`latex.qrc` declares one `<qresource prefix="/">` containing exactly **70 files**:

- **46 TTF files**
- **24 XML files**
- **20 Cyrillic entries** under `res/cyrillic/`
- **20 Greek entries** under `res/greek/`
- **30 common/core font entries** under `res/fonts/`

The Cyrillic and Greek groups combine font binaries with language/font/symbol/mapping XML. The common group is TTF-only and covers base, Euler, Latin, optional Latin, maths, optional maths and special fonts.

The manifest is declarative: it names resources and their Qt prefix. It does not itself initialize MicroTeX, load a font, parse XML, own application state, or define a second product runtime.

## Direct build/Qt consumers

### qmake

`src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7` is the direct consumer:

`RESOURCES += latex.qrc`

That is the exact pinned qmake/Qt resource-compiler path for the broad manifest.

### Root CMake

`CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9` does **not** attach `latex.qrc` to the `LaTeX` target. It copies `res` into the build tree for filesystem lookup and, for `QT`, compiles the Qt graphics backend. It also documents that tinyxml2 and the XML loaders were removed; non-Latin alphabet XML loading therefore no longer forms part of the current compiled parser path.

### Meson

`meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5` installs `res/fonts`, `res/greek`, `res/cyrillic` and resource marker/sample data as filesystem resources. The pinned Meson graph does not compile `latex.qrc`.

Therefore a reference to `latex.qrc` must retain its build-system context: it is a broad qmake resource bundle, not a universal statement about every current build.

## Current tdesktop shipping boundary: bundled.qrc, not latex.qrc

`res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` is a separate resource manifest designed for the accepted tdesktop path.

Its exact XML contains **30 TTF <file> entries** under prefix `/res`. The comment in that file says “28 unique TTFs”; the executable manifest itself contains 30 entries, so the prose count is stale and must not override exact-source accounting.

Comparing both manifests:

- all 30 `res/fonts/...` entries from `latex.qrc` are in `bundled.qrc`;
- the exact **40 entries present only in `latex.qrc`** are the 20 Cyrillic and 20 Greek resources;
- `bundled.qrc` explicitly documents that Cyrillic and Greek resources are **not bundled** because their XML parser path was stripped.

The shipping producer is `desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334`:

- builds `external_microtex_bundled` from the pinned MicroTeX fork;
- sets `BUILD_QT`;
- adds the pinned MicroTeX `res/bundled.qrc`;
- runs `target_prepare_qrc(external_microtex_bundled)`;
- exports the result through `desktop-app::external_microtex`.

No line in this shipping CMake owner consumes `latex.qrc`.

## Runtime resource resolution

The current runtime chain is also separate from the broad manifest:

1. `td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30` links `td_iv` privately to `desktop-app::external_microtex`.
2. `iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2` initializes once:
   - non-macOS: `Q_INIT_RESOURCE(bundled)`;
   - macOS: `base::RegisterBundledResources("external_microtex_bundled.rcc")`;
   - then `tex::LaTeX::initBundled()`.
3. `src/latex.cpp@9ed2d29525b057943da537e8d2dd9e3afedc45b2` keeps `RES_BASE = "res"` and `initBundled()` bypasses filesystem resource discovery.
4. `src/platform/qt/graphic_qt.cpp@016b07c72ad92fab12af5f2daae435976fc82de5` prepends `:/` when a font path does not exist on disk, mapping compiled `res/fonts/...` references to `:/res/fonts/...`.
5. `Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277` copies `external_microtex_bundled.rcc` into the macOS app Resources path.

`iv_markdown_math_renderer.cpp@46864b7ce87207c6e57b6bcc7610337120580bea` remains the formula measure/render/cache production owner. Those production responsibilities are not closed by this resource-manifest read.

## Resource-license boundary

The pinned root `LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2` is MIT for MicroTeX code. `README.md@824af04fd8607532efed3f73752b81e5093577ef` explicitly excludes fonts and XML under `res` from that blanket license and points to resource-specific licenses.

Exact pinned resource-license documents include:

- `res/fonts/licences/Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8`
- `res/fonts/licences/License_for_dsrom.txt@35e3ff2160fe16cb26f7830f936557ebc44c3e1c`
- `res/fonts/licences/OFL.txt@6836a0be000498319aebcc92a172159b6f80bb52` (SIL OFL 1.1 notice; named AMS reserved font names)
- `res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` (GPLv3 license text)
- `res/greek/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7` (same GPLv3 license text)

This dossier records provenance, not legal advice. The engineering invariant is exact package membership plus preservation of the applicable notices. In particular, current `bundled.qrc` excludes the Greek/Cyrillic groups rather than silently inheriting all 70 resources from `latex.qrc`.

## Negative oracles

The following conclusions are forbidden by the exact source:

- “`latex.qrc` is the accepted tdesktop shipping bundle.” False: shipping CMake consumes `res/bundled.qrc`.
- “The shipping bundle contains all 70 broad qmake resources.” False: it contains 30 TTF entries and excludes 40 Greek/Cyrillic entries.
- “MicroTeX MIT covers every font/XML resource.” False: README explicitly separates resource licenses.
- “The bundled manifest has 28 entries.” False as exact manifest accounting; the comment is stale and there are 30 `<file>` elements.
- “Resource compilation alone proves runtime availability.” False: static-library retention requires explicit resource initialization / registered RCC, and font loading can still fail.
- “This source read closes formula rendering or package acceptance.” False: external_microtex, td_iv, renderer, packaging and release evidence remain separate owners/gates.

## Coverage accounting

Before strict188:

- recursive source entries: 16,379
- unknown: 16,189
- unread: 16,164
- omitted: 0
- responsibilities: 205

After registering this one previously independent-unregistered source:

- recursive source entries: 16,379
- unknown: **16,188**
- unread: **16,163**
- omitted: **0**
- responsibilities: **206**

`baseline_identity_accepted=false` and `source_closure_ready=false` remain fail-closed.

## Parent exact-head evidence

The pre-receipt parent `909cf2db0ee137f4067389e784912a6d8c81fe23` is terminal-success:

- TDRP Revision 9 source authority: run `40112384872`, **61/61 jobs success**
  - authority `12567730104` — `sha256:78f21152671ebf2e41f81e2a754f1c0d64a33fb361ded7f0f8cc76e591dcebff`
  - closure `12566788318` — `sha256:c7ac0bb9858287f71426876acc1b9b196c88aa0629f44e7551ade950064f6c6a`
  - reachability `12567725535` — `sha256:914c62d81b1611257217112f2ffc56d9c02309c1d4c1b15fb84886e570f004ac`
  - Boost `12566997376` — `sha256:f5f4a129fdffbaae5981939656a5d94c4317a0a93c9e96d0af6d2e4c5d035842`
- Rust desktop runtime: run `40112384870`, **6/6 jobs success**
  - artifact `12566588503` — `sha256:2aa0bd744659b2525a8277d14353a355311acc288c7bc467d48d08f37bbe6890`
- Official MCP marketplace: run `40112384901`, **1/1 job success**

These are parent evidence only after this receipt commit is created. The receipt exact-head must be revalidated before this round can report its Actions terminal state.

## Traceability

- Requirement: `TDRP-R9-MICROTEX-LATEX-QRC-PROVENANCE-001`
- Target symbol: `TDRP_MICROTEX_LATEX_QRC_PROVENANCE_001`
- Exact-source oracle: `ORA-TDRP-MICROTEX-LATEX-QRC-EXACT-SOURCE-001`
- Manifest oracle: `ORA-TDRP-MICROTEX-LATEX-QRC-MANIFEST-001`
- Consumer oracle: `ORA-TDRP-MICROTEX-LATEX-QRC-QMAKE-CONSUMER-001`
- Shipping-separation oracle: `ORA-TDRP-MICROTEX-LATEX-QRC-SHIPPING-SEPARATION-001`
- License-boundary oracle: `ORA-TDRP-MICROTEX-LATEX-QRC-LICENSE-BOUNDARY-001`
- Invariant: `INV-TDRP-MICROTEX-LATEX-QRC-NOT-SHIPPING-BUNDLE-001`
- Invariant: `INV-TDRP-MICROTEX-LATEX-QRC-NO-PARALLEL-RESOURCE-RUNTIME-001`
- Invariant: `INV-TDRP-MICROTEX-LATEX-QRC-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001`
