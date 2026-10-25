# desktop-app/MicroTeX res/RES_README complete read — strict218

## Authority

- PR #154 lease / pre-receipt HEAD: `413d2b1b38b8c676bc331a03839ee4f150429124`.
- Live Fabushi main: `16aae12ea258f6a872a46aa47824766f64e1bdca`.
- Accepted/live tdesktop dev: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`.
- Pinned/live MicroTeX master: `61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`; recursive tree `462d2794c73bb87f57d9426621bc6ce0f57c5fc2`, `truncated=false`, 315 entries.
- Target: `res/RES_README@386c018fee9c3fd41cd42860bd0c1c33ef61a9ee`, mode `100644`, 1,661 bytes, 73 lines.
- Pre-write independent responsibility row count for `TDRP-R9-MICROTEX-RES-README-PROVENANCE-001`: **0**.

## Complete target read

The file is a newline-delimited manifest of exactly 73 unique relative resource paths. Every entry resolves to a blob in the pinned MicroTeX tree; there are no missing or duplicate entries. Classification is exact: 3 license text files, 46 TTF fonts, and 24 XML resource/mapping files.

## Exact-path history and creation boundary

The exact-path history contains exactly one commit: `e45a487bc3c3b8f67dacd4c1373dcd17b30c0876` (`move builtin resource into static code`); page 2 is empty. Its parent is `bab5cffc87e848a991165de8f4fb5c595c714e3a`. The parent recursive tree is `truncated=false` with 217 entries and lacks `res/RES_README`. Creation changes exactly 90 paths, covering the manifest, XML/font registration data and generated static registration code. This bounds the file to the resource-registration migration, not an unrelated product runtime.

## Bundled-resource boundary

Pinned `res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa` contains exactly 30 `<file>` entries. All 30 occur in `RES_README`. The remaining 43 manifest entries are exactly:

- 3 font-license texts: `OFL.txt`, `License_for_dsrom.txt`, `Knuth_License.txt`;
- 20 `greek/*` entries;
- 20 `cyrillic/*` entries.

The qrc comment explicitly says Greek and Cyrillic alphabet resources are **not bundled** and fall back when the stripped XML parser path is unavailable. Therefore `RES_README` is broader standalone resource/install inventory; it is not equivalent to accepted tdesktop bundled package membership.

## Meson install and license boundary

Pinned `meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5` installs all three resource subtrees (`res/fonts`, `res/greek`, `res/cyrillic`) and separately installs `RES_README`, `SAMPLES.tex` and `.clatexmath-res_root`.

Pinned `README.md@824af04fd8607532efed3f73752b81e5093577ef` states the project is MIT **excluding** fonts and XML under `res`, which carry different licenses and must be checked under the font-license/Greek/Cyrillic resource areas. `RES_README` therefore has a legal/provenance responsibility: it enumerates resource payloads across mixed license domains and must not flatten them into project MIT.

## Existing-owner-first audit

Live Fabushi authority was re-read. `current-owner-inventory.md` keeps attachment/resource ingress under the existing Composer + Electron attachment manager + Host attachment extension, and artifacts under existing transcript/tool/artifact surfaces. `architecture-map.md` maps files/media to attachments/artifacts/resource lifecycle. `owner-resolution-matrix.md` keeps desktop/platform lifecycle and packaging in existing Electron/desktop owners. `desktop/package.json` remains the existing shipping package composition.

Selected owner: **existing TDRP recursive source/build/resource/license/packaging provenance governance**. Product Resource/attachment/artifact and desktop packaging owners remain canonical runtime context only. No `MicroTeXResourceManifest`, second font registry, second license store, second packaging root, Telegram runtime, or source-specific Resource owner is introduced.

## Negative oracles

- `RES_README` inventory is not accepted bundled shipping membership.
- The 43 non-qrc entries must not be silently treated as packaged runtime resources.
- Project MIT must not be applied to the mixed-license `res` payload.
- A resource manifest does not prove font rendering, XML parsing, Greek/Cyrillic behavior, or package inclusion.
- Neighboring resource/font/license files remain independently auditable source entries.

## Coverage

Before strict218: total 16,379; unknown 16,159; unread 16,134; omitted 0; responsibilities 235.

After strict218: total 16,379; unknown **16,158**; unread **16,133**; omitted **0**; responsibilities **236**.

`baseline_identity_accepted=false`, `source_closure_ready=false`, and independent release acceptance remains not accepted.

## Parent exact-head evidence

Lease `413d2b1b38b8c676bc331a03839ee4f150429124` was already terminal-success at exact head:

- TDRP Revision 9 run `40165548344`: **61/61 success**.
  - authority `12588556259` — `sha256:4e7956b90dca5ec46b375aad90356a5d9971e2993cf378ec1b462bb0b0c3cc6d`
  - closure `12588546650` — `sha256:0861ce729a4d76bed7e1b4d47d36528de72e269c08e1be1ab501077b71f1f4e1`
  - reachability `12588297013` — `sha256:f7c0813c174702148f51fbf994c6ec5079f81859aef616fa54d42e6036e35760`
  - Boost `12588646290` — `sha256:124f6ec56afab8927d0590b752aacc723ccd44c6469c39aba3bef0bbb0a4b5b1`
- Rust desktop runtime run `40165548329`: **6/6 success**; artifact `12587847541` — `sha256:f20f12249379f91e5c0b2ccadd37f08d3bd619139bba0dfd1bce065c872f6206`.
- Official MCP marketplace run `40165548321`: **1/1 success**.

These become historical parent evidence after the receipt commit; fresh receipt-head Actions remain required.

## Traceability

- Requirement: `TDRP-R9-MICROTEX-RES-README-PROVENANCE-001`
- Target symbol: `TDRP_MICROTEX_RES_README_PROVENANCE_001`
- Gates: `G-FILE`, `G-TRACEABILITY`, `G-EVIDENCE`
