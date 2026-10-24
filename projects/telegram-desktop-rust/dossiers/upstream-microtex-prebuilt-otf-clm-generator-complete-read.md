# desktop-app/MicroTeX prebuilt/otf_clm.py complete read — strict192

## Authority

- Fabushi PR: bhrumom/fabushi-desktop#154
- Branch: tdrp-r9-entry31-includes-r27-20261020
- Pre-receipt live HEAD: 01e5ed8bd2a489a36e254e1b676f7ac702e2be54
- Pre-receipt tree: ed7a5c61ed54f775b0491d67280c10e2d1e8ce6f
- Live main observed before write: 16aae12ea258f6a872a46aa47824766f64e1bdca.
- Accepted tdesktop pin remains telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf (tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5).
- Direct child pin remains desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f (tree 462d2794c73bb87f57d9426621bc6ce0f57c5fc2).
- CMake helper pin remains desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095 (tree f5c386cb34208309aae521187611fac4a2045577).
- Target: prebuilt/otf_clm.py@aba6e00be5221f7ee18d4ae71f8acbf8c92cf17b, mode 100755, 14,267 bytes, 507 physical lines.
- Pinned MicroTeX recursive tree remains truncated=false with 315 entries / 279 blobs.
- prebuilt contains the tracked sibling .gitignore and this generator; the fixed tree contains zero tracked .clm blobs.
- Pre-write independent target responsibility registration count: 0. The only prior target mention is strict191 sibling .gitignore context.

This dossier closes only the generator/source-format/generated-input provenance responsibility. It does not claim a shipping CLM runtime, formula renderer, resource owner, package consumer, or release acceptance.

## Exact command and source lifecycle

The file is a Python 2 script with FontForge and struct dependencies. The only executable entrypoint is direct positional indexing:

1. sys.argv[1] — input OpenType font path.
2. sys.argv[2] == 'true' — exact lowercase true enables MATH serialization; every other supplied value is false.
3. sys.argv[3] — caller-selected output path.

There is no argv-count validation, usage/help path, argparse layer, extension check or output-directory enforcement.

parse_otf() performs source collection before output mutation:

1. fontforge.open(input).
2. optional font.math constant collection.
3. discover pair-kerning subtables, ligature subtables and kerning-class subtables.
4. iterate FontForge glyph names, print originalgid/name to stdout, collect Unicode mapping, glyph-name→originalgid map, glyph metrics, pair kerning, ligatures and optional glyph MATH data.
5. capture font.em and font.xHeight.
6. font.close().
7. only then open(output, 'wb') and serialize.

Therefore an import/argv/FontForge/open/read-stage failure occurs before the output file is opened by this script. Once open(output, 'wb') succeeds, an existing output is truncated and subsequent failures can leave a partial binary.

## Glyph and lookup extraction

### Metrics and Unicode

For each glyph:

- width = glyph.width.
- height = boundingBox()[2].
- depth = -boundingBox()[1].
- if glyph.unicode != -1, append (unicode codepoint, originalgid).
- glyph_name_id_map records glyph name -> originalgid.

Unicode records are sorted by codepoint before serialization.

### Pair kerning

read_kern_subtables selects GPOS lookups whose lookup type is gpos_pair and excludes FontForge kerning-class subtables. read_kern() flattens glyph.getPosSub(subtable), keeps records with horizontal kern_info[5] != 0, and records (other glyph name, horizontal kerning value).

During serialization pair kerns are converted to glyph IDs, sorted by glyph ID and emitted as glyph/value pairs.

### Kerning classes

read_kerning_class selects gpos_pair subtables where font.isKerningClass is true and captures FontForge left classes, right classes and the flattened value table.

Serialization deliberately excludes class-zero membership lists and skips the first matrix row and first matrix column. For each side it writes the number of nonzero classes, then sorted (glyph id, class index) membership records. Remaining class-to-class values are signed 16-bit entries.

### Ligatures

read_ligature_subtables selects GSUB lookups of type gsub_ligature. read_ligatures collects component glyph-name sequences and the resulting glyph.originalgid.

write_ligas maps components to glyph IDs and builds a trie/forest. Children are recursively sorted by glyph ID. Each terminal stores the resulting ligature glyph ID. Serialization starts with a synthetic root node glyph=0/liga=-1 and recursively writes every node.

### Optional OpenType MATH

When math mode is enabled, the generator reads the font-wide MATH constants from font.math and per-glyph:

- italicCorrection and topaccent;
- horizontal and vertical variants;
- horizontal and vertical component assemblies;
- four directional math-kern collections: top-left, top-right, bottom-left, bottom-right.

No MATH payload is written for non-math mode.

## Exact CLM serialization contract

The stream has no magic, version or checksum prefix. Order is:

1. is_math_font boolean.
2. em.
3. xHeight.
4. Unicode→glyph map.
5. kerning classes.
6. ligature trie.
7. optional font-wide MATH constants.
8. glyph records.

### Byte order and field widths

Numeric fields using struct format prefix ! are network byte order (big-endian):

- !H — unsigned 16-bit counts, glyph IDs, em/xHeight, variant IDs, assembly fields.
- !I — unsigned 32-bit Unicode codepoint.
- !h — signed 16-bit metrics, kern values, MATH values/corrections.
- !i — signed 32-bit ligature terminal glyph ID, including -1 for non-terminal/root.
- !IH — Unicode codepoint + glyph ID.
- !HH — glyph ID + kerning-class index.
- !Hh — glyph ID + pair-kern value.
- !hh — MATH correction-height + kern value.

Boolean fields are written with struct.pack('?') rather than !?. They are single boolean fields for the top-level math flag and glyph-assembly presence.

### Unicode map

- !H record count.
- sorted records: !IH codepoint, glyph ID.

### Kerning class map

- !H number of kerning-class tables.
- for each table and each left/right side:
  - !H nonzero-class count.
  - !H flattened membership count.
  - repeated !HH glyph ID, class index.
- signed !h values for matrix cells excluding class-zero row/column.

### Ligature trie

Each node:

- !H glyph ID.
- !i ligature result glyph ID or -1.
- !H child count.
- recursively encoded children in glyph-ID order.

### Glyph records

- !H glyph count.
- each glyph metrics: width, height, depth as three !h values.
- pair-kern list: !H count then repeated !Hh glyph ID/value.
- if math mode:
  - !h italic correction.
  - !h top-accent attachment.
  - each variants list: !H count then repeated !H glyph IDs.
  - each assembly: '?' presence; if present, !H part count, !h italic correction, then each part glyph ID !H plus the remaining component values as !H.
  - four math-kern directions: each !H count followed by repeated !hh correction-height/value.

Font-wide MATH constants are emitted as signed !h values before glyph records.

## Failure and atomicity oracles

- Missing sys.argv[1..3] can raise IndexError before parse_otf and before output mutation.
- Missing Python2/FontForge or import failure occurs before output mutation.
- fontforge.open and read-stage failures occur before output open because font.close() precedes open(output,'wb').
- Output-open failure is uncaught.
- If output open succeeds, wb truncates an existing path before the first serialized byte.
- Missing glyph-name mappings referenced by kerns/variants/assemblies/ligatures can raise KeyError during serialization and leave partial output.
- Values outside the signed/unsigned struct ranges can raise struct.error and leave partial output.
- Write/storage failures are uncaught and can leave partial output.
- There is no temp file + rename, checksum, magic/version envelope, explicit fsync, file lock, generation fence, cancellation or recovery journal.
- Concurrent writers to the same caller-selected path are not serialized by this script.
- The output suffix is not enforced; .clm is a convention, not a parser assertion.

A partial or successfully generated CLM is not source or shipping authority merely because this tool produced it.

## Build/package consumer audit

Exact pinned candidates were re-read for direct otf_clm.py, .clm and prebuilt references:

- MicroTeX CMakeLists.txt@75059dec500cff8114728331eb5cb317abf6daf9 — zero.
- MicroTeX meson.build@d9968045f0162b8f1f8f518f554fd71199f067d5 — zero.
- MicroTeX src/meson.build@84ce4f05fceb8419a326bf6a059d2583c793fc9a — zero.
- MicroTeX src/QtLatex.pri@e5f4efcd78ec9d1e808127e414ce5689b44363b7 — zero.
- ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764 — zero.
- windows-gdi.yml@4adace4d194d60ffc027a1b5d99fa33d200b07aa — zero.
- windows-qt.yml@d42c1b75d40fd2b3ce8574cfd90cfc85721ff838 — zero.

README.md@824af04fd8607532efed3f73752b81e5093577ef uses “Prebuilt packages” for external openSUSE cLaTeXMath/cairo packages, not as a consumer of this directory or generated CLM.

No checked fixed build/package/CI path invokes this generator.

## Accepted tdesktop shipping boundary

The accepted tdesktop chain was re-read at exact pins:

- desktop-app/cmake_helpers external/microtex/CMakeLists.txt@a1f479d2833bbb0cf0e007ee8fd33a40c10d9334 builds the pinned MicroTeX C++ sources, defines BUILD_QT, links Qt and attaches res/bundled.qrc via target_prepare_qrc.
- external/microtex has zero direct otf_clm.py/.clm/prebuilt references.
- res/bundled.qrc@303fb6bdf780d66a6e8960a095d17be466b909fa has zero direct references.
- Telegram/cmake/td_iv.cmake@ec39405fc3c7bec58f8acb42aa49ed69c60d1b30 links desktop-app::external_microtex and has zero direct references.
- Telegram/CMakeLists.txt@2ea9badfc5750b9b4eab111052faafe946fea277 packages external_microtex_bundled RCC on macOS and has zero direct references.

Thus this generator is not a checked accepted-shipping producer or runtime consumer. external_microtex/resource/formula responsibilities remain separately open.

## License and provenance boundary

- prebuilt/otf_clm.py is outside res and falls under MicroTeX root MIT LICENSE@78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2.
- README explicitly excludes fonts and XML resources under res from the blanket project MIT statement.
- Pinned font-specific notices remain under res/fonts/licences with separate Greek/Cyrillic license texts.
- A generated CLM derives glyph/metrics/kerning/ligature/MATH information from the caller-selected input font. Generator-code MIT does not convert the font-derived output into unconditionally MIT material.
- The fixed tree has zero tracked .clm blobs, so no generated binary redistribution conclusion is made here.

This is provenance, not legal advice.

## Negative oracles

- The generator is part of accepted tdesktop shipping. False for the checked exact composition.
- Any generated CLM is accepted source. False; the pinned tree has zero tracked CLM.
- All fields are network-order. False; numeric fields with ! are network-order, while bool uses '?'.
- Non-math mode is selected only by the literal string false. False; every supplied value other than exact lowercase true yields false.
- Input-read failure can truncate the output. False for failures before output open; source collection and font.close precede open(output,'wb').
- Serialization failure is atomic. False; post-open failures can leave truncated/partial output.
- Output must end in .clm or live under prebuilt. False; output path is caller-selected and unvalidated.
- Root MIT automatically licenses generated font data. False; input font/resource provenance remains relevant.
- This row closes formula rendering, resources, packaging or release acceptance. False; those remain independently open.

## Coverage accounting

Before strict192:
- recursive source entries: 16,379
- unknown: 16,185
- unread: 16,160
- omitted: 0
- responsibilities: 209

After independently registering this target only:
- recursive source entries: 16,379
- unknown: 16,184
- unread: 16,159
- omitted: 0
- responsibilities: 210

baseline_identity_accepted=false and source_closure_ready=false remain fail-closed.

## Parent exact-head evidence

The pre-receipt parent 01e5ed8bd2a489a36e254e1b676f7ac702e2be54 was revalidated before this receipt:

- TDRP Revision 9 source authority run 40119481933: completed/success; 61/61 jobs success.
  - authority 12570970145 — sha256:11c3bce8b1d837dcac3f6b6f4d4f026e0a26f25a9cd6f9c4e64d020a77ab7785
  - closure 12571330390 — sha256:772deee1d234bd74b9034b206acb1ac27bb07b56e3f202e968f43609899ab6c4
  - reachability 12571325355 — sha256:cd51321a414ce0c6048780e8559934ada8ad3e2992877e74fbeaf29f88a2eb7e
  - Boost 12570865418 — sha256:aca48fe13b975dcdcc230e18c40ed9e0ddeeefa7b3c71812cbc2026e2e0f3784
- Rust desktop runtime run 40119481942: completed/success; 6/6 jobs success.
  - artifact 12569984795 — sha256:bc0d45354e05bdce74b14dce2720ea9b1b3d13ba803e2616b989f4882beacfb9
- Official MCP marketplace run 40119481920: completed/success; 1/1 job success.

These become historical parent evidence after this receipt commit. The new receipt exact-head must be independently revalidated before current-head success is reported.

## Traceability

- Requirement: TDRP-R9-MICROTEX-PREBUILT-OTF-CLM-GENERATOR-PROVENANCE-001
- Target symbol: TDRP_MICROTEX_PREBUILT_OTF_CLM_GENERATOR_PROVENANCE_001
- Exact-source oracle: ORA-TDRP-MICROTEX-OTF-CLM-EXACT-SOURCE-001
- Serialization oracle: ORA-TDRP-MICROTEX-OTF-CLM-SERIALIZATION-SCHEMA-001
- Byte-order oracle: ORA-TDRP-MICROTEX-OTF-CLM-NETWORK-BYTE-ORDER-001
- Kern/ligature/MATH oracle: ORA-TDRP-MICROTEX-OTF-CLM-KERN-LIGA-MATH-001
- Failure/atomicity oracle: ORA-TDRP-MICROTEX-OTF-CLM-ARGV-FAILURE-ATOMICITY-001
- Build/package consumer oracle: ORA-TDRP-MICROTEX-OTF-CLM-BUILD-PACKAGE-CONSUMER-001
- Shipping bypass oracle: ORA-TDRP-MICROTEX-OTF-CLM-TDESKTOP-SHIPPING-BYPASS-001
- License boundary oracle: ORA-TDRP-MICROTEX-OTF-CLM-LICENSE-BOUNDARY-001
- Invariant: INV-TDRP-MICROTEX-OTF-CLM-GENERATED-OUTPUT-NOT-AUTHORITY-001
- Invariant: INV-TDRP-MICROTEX-OTF-CLM-PARTIAL-OUTPUT-NOT-AUTHORITY-001
- Invariant: INV-TDRP-MICROTEX-PRODUCTION-RESPONSIBILITIES-NOT-CLOSED-001

Next pinned recursive-tree non-directory entry: readme/example_bw_false.svg@9e812c823d3c3be06ab43ee77f10b6122bd638dd.
