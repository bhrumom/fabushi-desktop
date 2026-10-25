# desktop-app/MicroTeX res/cyrillic/LICENSE complete read — strict221

## Authority

- PR #154 lease / pre-receipt HEAD: 285962256732264555f6fdb8ce17573470e47d68
- Live Fabushi main: 16aae12ea258f6a872a46aa47824766f64e1bdca
- Accepted/live tdesktop dev: a7f453612f4896ac04d0007a36ab0d377e5bcbcf
- Pinned/live MicroTeX master: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f; recursive tree truncated=false, 315 entries.
- Target: res/cyrillic/LICENSE@f288702d2fa16d3cdf0035b15a9fcbc552cd88e7, 35,149 UTF-8 bytes / 35,149 characters / 675 line slots.
- Pre-write independent responsibility row count for TDRP-R9-MICROTEX-RES-CYRILLIC-LICENSE-PROVENANCE-001: 0.

## Complete target read and license truth

The target was read completely. It is the full GNU General Public License, version 3, dated 29 June 2007. The blob contains the GPLv3 title/preamble, TERMS AND CONDITIONS, numbered sections 0 through 17, END OF TERMS AND CONDITIONS, and the complete "How to Apply These Terms to Your New Programs" appendix through the final LGPL guidance.

MicroTeX README.md@824af04fd8607532efed3f73752b81e5093577ef explicitly excludes fonts and XML resources under res from the project MIT blanket and directs readers to res/fonts/license, res/greek and res/cyrillic for distinct licenses. The exact creation commit message says "add license for cyrillic fonts and xml resources". This audit records that upstream provenance boundary and does not infer broader GPL scope.

## Exact-path history and creation boundary

Exact-path history contains one commit, 59f45372bed05c743f9a37a67a18be71e5521d87; page 2 is empty. The commit changes exactly one path: res/cyrillic/LICENSE. Parent 4658e41a97ba1b499c4085f4611304c8f1e857ca has a recursive tree with truncated=false and 275 entries, and the target is absent.

## Resource manifest and standalone install boundary

res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee lists exactly 20 Cyrillic runtime entries: 8 TTF and 12 XML. LICENSE is adjacent provenance for the subtree and is not one of those runtime manifest entries.

meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 installs res/cyrillic as a whole directory with install_subdir('res/cyrillic', install_dir: get_option('datadir')/'clatexmath'). Therefore standalone MicroTeX install includes LICENSE with the Cyrillic subtree.

## Accepted bundled.qrc and tdesktop shipping exclusion boundary

res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa has zero Cyrillic <file> entries and explicitly states that Cyrillic and Greek alphabet fonts are NOT loaded by the bundled resource. desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 builds external_microtex_bundled from bundled.qrc and target_prepare_qrc.

Accepted tdesktop iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2 registers Q_INIT_RESOURCE(bundled) off macOS or external_microtex_bundled.rcc on macOS and then calls tex::LaTeX::initBundled(). Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 copies external_microtex_bundled.rcc into app Resources on macOS. Accepted production shipping therefore consumes bundled.qrc/.rcc and excludes the Cyrillic subtree/license from this runtime bundle; standalone Meson installation is a separate boundary.

## Existing-owner-first audit

Current Fabushi exact-head owners were re-read: frontend/src/production/runtime-assets.ts@470eb11f3d7f22b959008755e639a96a7f71aba0, source/electron-main/attachments/attachment-manager.ts@0d055b7d65c49911deb18ff08af2caa1639d93f7, desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84, and .github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643. Selected owner is the existing TDRP recursive source/resource/license/install/packaging/provenance governance. No second Resource owner, license store, MicroTeX runtime, Cyrillic font runtime, install root or packaging root is introduced.

## Negative oracles

- Do not flatten the project MIT blanket over res/cyrillic.
- Do not reduce the exact GPLv3 provenance to a filename or partial summary.
- Do not represent standalone Meson install_subdir(res/cyrillic) as accepted tdesktop runtime shipping.
- Do not conflate RES_README's 20 runtime entries with the adjacent LICENSE file.
- Do not claim accepted bundled.qrc/tdesktop shipping contains Cyrillic resources; it has zero Cyrillic file entries and explicitly excludes them.
- This row does not close the 20 individual Cyrillic runtime resource entries.
- No Fabushi source-specific MicroTeX/Cyrillic runtime or parallel license/package owner is justified.

## Coverage

Before strict221: total 16,379; unknown 16,156; unread 16,131; omitted 0; responsibilities 238.
After strict221: total 16,379; unknown 16,155; unread 16,130; omitted 0; responsibilities 239.
baseline_identity_accepted=false, source_closure_ready=false, independent release acceptance remains not accepted.

Next deterministic non-directory entry: res/cyrillic/cyrillic.map.xml@5431dc86108e722f7899fbf3db4db7af67badcc4.

## Parent exact-head evidence

Lease 285962256732264555f6fdb8ce17573470e47d68 is terminal-success: TDRP Revision 9 run 40170692344 is 61/61 success; authority 12590094543 sha256:224c30c95b4006ca4052f10ac9ac6902a90c24cbf5ce64894003d6ca84e0aff1; closure 12590515563 sha256:f218287f8474e4a9b1b24cc8a132221e42e93c61c0065de3ac1401b05550c115; reachability 12590705304 sha256:aa9d8e0884e89b5a4376e937b0ff348939fcfbd26bfc748c37e3e3659f23ccdd; Boost 12590495458 sha256:6227f02a7ca734aa088763a5f1bded86741c513c78c787254bac1786485e266c. Rust desktop runtime run 40170692312 is 6/6 success, artifact 12590370442 sha256:39e6f66038def5f566f838358f8e8c692eb6f9293e12c034d1972330a7399caf. Official MCP marketplace run 40170692405 is 1/1 success. These become historical after the receipt commit; fresh receipt-head Actions are required.

## Traceability

Requirement: TDRP-R9-MICROTEX-RES-CYRILLIC-LICENSE-PROVENANCE-001. Target symbol: TDRP_MICROTEX_RES_CYRILLIC_LICENSE_PROVENANCE_001. Gates: G-FILE, G-TRACEABILITY, G-EVIDENCE.
