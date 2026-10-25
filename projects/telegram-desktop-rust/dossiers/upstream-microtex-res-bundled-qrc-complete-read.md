# desktop-app/MicroTeX res/bundled.qrc complete read — strict220

## Authority

- PR #154 lease / pre-receipt HEAD: 106299cf2e385174f4483c1e7cfae4219deee818
- Live Fabushi main: 16aae12ea258f6a872a46aa47824766f64e1bdca
- Accepted/live tdesktop dev: a7f453612f4896ac04d0007a36ab0d377e5bcbcf
- Pinned/live MicroTeX master: 61aaa7cc354de91d5898ffb0b2a6c62628d9a76f; recursive tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2, truncated=false, 315 entries.
- Target: res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa, 2,329 UTF-8 characters / 55 line slots.
- Pre-write independent responsibility row count for TDRP-R9-MICROTEX-RES-BUNDLED-QRC-PROVENANCE-001: 0.

## Complete target read and manifest truth

bundled.qrc has prefix /res and exactly 30 file entries. All 30 are unique and all 30 resolve to blobs in the pinned tree. The file comment says 28 unique TTFs, but the executable manifest proves 30; 30 is authoritative and 28 is retained only as drift/provenance evidence.

- res/fonts/base/cmex10.ttf@4df1d6c83b039817d1d97d0ee794e5c0fb0124f2
- res/fonts/base/cmmi10.ttf@c12851691ef7a5424f83752e7d39737af9860c79
- res/fonts/base/cmmib10.ttf@6772eafd5b8c4d0d0a85b12d9e92c4a05d2caf64
- res/fonts/euler/eufb10.ttf@935f84bd27c682b1ed2ed6e79c89c139d55943b9
- res/fonts/euler/eufm10.ttf@93c3452962c71ac7364c6184a58b2ef06b337c79
- res/fonts/latin/bi10.ttf@c303ae8672c9a0ad6b1e51f8a92c6593f477a28c
- res/fonts/latin/bx10.ttf@7d0059279d4592b6f625be45a6148125bf6ab57e
- res/fonts/latin/cmr10.ttf@b9badf318845a706a03689de0d9165130f4232d6
- res/fonts/latin/i10.ttf@c270fddf36ee8f49adcf86c5dc53903b9d94ef6f
- res/fonts/latin/r10.ttf@adec5895ebc935037d85e57476c739b62cad234b
- res/fonts/latin/sb10.ttf@0097770d94221459400e827de7f13de5847155af
- res/fonts/latin/sbi10.ttf@47a52a2cb30ea06a23c130564c8da255da9cb59c
- res/fonts/latin/si10.ttf@857d81e7b996dbca80afb8617a1e189d41af8ce4
- res/fonts/latin/ss10.ttf@dd2a57ab01bcd441ef35e178caa707ad7e98f8e8
- res/fonts/latin/tt10.ttf@35afb15d29281019281b3b8c91b3ae19b86b9006
- res/fonts/latin/optional/cmbx10.ttf@82264d64c9a88279ff08e8fd9f89d6fc35fff36b
- res/fonts/latin/optional/cmbxti10.ttf@6a5ae5ab1f90e39bcc13c5b0fc480ce9c7e0f260
- res/fonts/latin/optional/cmss10.ttf@e6c46a32b5dd3ad5f50266763955adb2287df28f
- res/fonts/latin/optional/cmssbx10.ttf@41d9e881a02024f4932bca6eb855b6a11ef87e70
- res/fonts/latin/optional/cmssi10.ttf@893d35b0b2e8a8dd775e45ba3f552d9f4077f704
- res/fonts/latin/optional/cmti10.ttf@f6453986ccd27da7fbb3b40ec682e1efb5795f36
- res/fonts/latin/optional/cmtt10.ttf@677ba6286a094018973856b0efc1fb7a0255c4cd
- res/fonts/maths/cmbsy10.ttf@ee93249b7a03c079a18a0a943370410eca6cbb64
- res/fonts/maths/cmsy10.ttf@ee9d12b353430fe2fef1d44456021b2a4df76100
- res/fonts/maths/msam10.ttf@9f5d1330c381ea86ff73ab3da8d2ec120ae403d6
- res/fonts/maths/msbm10.ttf@b8d417a45959a44e144852639578952f013c9f7f
- res/fonts/maths/rsfs10.ttf@e2e91898ebebe16ced4dd9338bb441a8526aa75a
- res/fonts/maths/special.ttf@6c859a1ee00901e48d6f97691d76e2b4391e3259
- res/fonts/maths/stmary10.ttf@0c4244c96380457f8f5251be9c158ff22b3614e5
- res/fonts/maths/optional/dsrom10.ttf@b49e53c947088865378604a7ba3413ca61de550a

The same 30 paths occur in res/RES_README. The qrc contains zero Greek and zero Cyrillic paths. RES_README separately lists 20 Greek and 20 Cyrillic entries, including 8 TTFs in each alphabet subtree.

## Exact-path history and creation boundary

Exact-path history contains one commit e9564812854b6cc3f4386f78864f927a53b7b208; page 2 is empty. Parent fe129888b2107b7863c8ec7f1379b226b28b37ef has recursive tree 7ee53c87eb7c9601e6d6b0883da75c34a96e149f, truncated=false, 314 entries, and no target. Creation changes exactly res/bundled.qrc, src/latex.cpp, and src/latex.h. The creation message also says 28 fonts; the committed 30-entry qrc overrides stale prose.

## Runtime resource contract

src/platform/qt/graphic_qt.cpp@016b07c72ad92fab12af5f2daae435976fc82de5 checks QFile::exists and prepends :/ on a disk miss, yielding :/res/fonts/... from default RES_BASE=res. src/latex.cpp@9ed2d29525b057943da537e8d2dd9e3afedc45b2 and src/latex.h@0608e92d3b4821fb8cf5c402014f31c1692f014f add LaTeX::initBundled, bypassing disk resource-root discovery.

## Accepted build and production shipping

desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095 external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 compiles external_microtex_bundled, adds bundled.qrc to target_sources, calls target_prepare_qrc, and requires resource registration plus initBundled. target_prepare_qrc.cmake@68c3f44c7b350fade14d6492f5ea93bbcc58d0d6 uses AUTORCC off Apple and Qt::rcc --binary on Apple.

Accepted tdesktop iv_markdown_microtex.cpp@5c854b265e6dc67c1c500a72e04fa58b7fc6c3a2 calls Q_INIT_RESOURCE(bundled) off macOS; on macOS it registers external_microtex_bundled.rcc; both then call initBundled. Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 copies the .rcc into app Resources on macOS. Standalone MicroTeX Meson install is broader and distinct.

## License boundary

MicroTeX README explicitly excludes fonts/XML under res from the project MIT blanket. Pinned license texts include OFL.txt@6836a0be000498319aebcc92a172159b6f80bb52, License_for_dsrom.txt@35e3ff2160fe16cb26f7830f936557ebc44c3e1c, and Knuth_License.txt@5b03bf8a533e66c79d113b82ba95586f8c7af3b8. This records a boundary only; it does not infer unsupported per-font attribution.

## Existing-owner-first audit

Live authority and current owners were re-read, including frontend/src/production/runtime-assets.ts@470eb11f3d7f22b959008755e639a96a7f71aba0, source/electron-main/attachments/attachment-manager.ts@0d055b7d65c49911deb18ff08af2caa1639d93f7, desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84, and release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643. Selected owner is existing TDRP recursive source/runtime-resource/font/license/build/packaging/provenance governance. No second Resource/font/license store, MicroTeX runtime, or packaging root is introduced.

## Negative oracles

- Stale 28 prose never overrides the 30-entry manifest.
- Greek/Cyrillic resources are outside this qrc.
- Standalone Meson install is not accepted tdesktop shipping composition.
- Resource-specific licenses are not flattened into project MIT.
- Upstream qrc shipping does not itself require a parallel Fabushi MicroTeX/font runtime.
- This row does not close the 30 individual font entries or neighboring Greek/Cyrillic entries.

## Coverage

Before strict220: total 16,379; unknown 16,157; unread 16,132; omitted 0; responsibilities 237.
After strict220: total 16,379; unknown 16,156; unread 16,131; omitted 0; responsibilities 238.
baseline_identity_accepted=false, source_closure_ready=false, independent release acceptance remains not accepted.

## Parent exact-head evidence

Lease 106299cf2e385174f4483c1e7cfae4219deee818 is terminal-success: TDRP Revision 9 run 40168277972 is 61/61 success; authority 12589885820 sha256:5240a8cf31ab6ec5157d7185b04ffaea41fecac63aa602dc0383db4c8d863db1; closure 12589846114 sha256:b72cc6b1ebd5c5e6997a2b7c6f36c453c94cd4269e9436c871c3deab11647c23; reachability 12589911143 sha256:e9c67b9619675d1c680b28c4b26bbb20757c11f6ac6245d28d46f9e18ef5f2d9; Boost 12590070570 sha256:083e320d34838a84600cd7f19064c914dd4eb9f084e08d9de74d11bdd2e68cab. Rust run 40168277964 is 6/6 success, artifact 12589095911 sha256:ea50f045c1a9cb0fd29563f26390487e38c4b162ab7db5d98b39be2f5b6465b0. Official MCP run 40168277936 is 1/1 success. These become historical after the receipt commit; fresh receipt-head Actions are required.

## Traceability

Requirement: TDRP-R9-MICROTEX-RES-BUNDLED-QRC-PROVENANCE-001. Target symbol: TDRP_MICROTEX_RES_BUNDLED_QRC_PROVENANCE_001. Gates: G-FILE, G-TRACEABILITY, G-EVIDENCE.
