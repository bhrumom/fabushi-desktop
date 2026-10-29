# OCRM Technical Dossier — Unit 20: QR Code Generation, Terminal Half-Block Formatting, Raw PNG Encoding & Shared Media Store Specs (`OCRM-UNIT-MEDIA-007`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Media QR Code Generation, Terminal Half-Block Formatting, Raw PNG Encoding, Fallback BMP Transcoding & Shared Media Storage Invariants)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 core media QR code, terminal display, raw PNG encoding, fallback raster transcoding, and shared storage specification files (totaling 269 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/qr-runtime.ts` | `59a90da9e549b15fc7d0a0626174fff0475454df` | 7 | 1..7 (full-file) | `OCRM-RESP-MEDIA-QR-TERMINAL-001` |
| `part-0042` | `src/media/qr-terminal.ts` | `5612d19c427114583ab7e25f3ea2c410eade8aef` | 44 | 1..44 (full-file) | `OCRM-RESP-MEDIA-QR-TERMINAL-001` |
| `part-0042` | `src/media/qr-image.ts` | `8bf0b227d7946b5ff80f12396863daee6cdb64c8` | 92 | 1..92 (full-file) | `OCRM-RESP-MEDIA-QR-IMAGE-001` |
| `part-0042` | `src/media/png-encode.ts` | `4b552143b36d863446898a1690b22f689675681f` | 74 | 1..74 (full-file) | `OCRM-RESP-MEDIA-PNG-ENCODE-001` |
| `part-0042` | `src/media/photon.runtime.ts` | `1089b4ede000a03e5746f23cc449df7025f02735` | 14 | 1..14 (full-file) | `OCRM-RESP-MEDIA-PHOTON-RUNTIME-001` |
| `part-0042` | `src/media/store.shared.ts` | `3e06d13268b774a149cdd657983f82e74ee51772` | 38 | 1..38 (full-file) | `OCRM-RESP-MEDIA-STORE-SHARED-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 5 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-MEDIA-QR-TERMINAL-001`**:
   - **Behavior**: Terminal ANSI QR text renderer supporting compact half-block characters (`█`, `▀`, `▄`, space), 1-module margin, quiet zone black-on-white formatting (`\x1b[47m\x1b[30m`), ANSI reset placement (`\x1b[0m`), and standard full-block terminal fallback.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_qr_png.rs::render_qr_terminal`.
   - **Production Call Chain**: Producer `AttachmentsService::handle_openclaw_media_gateway_call("renderQrTerminal")` → Owner `render_qr_terminal` → Consumer `DevicePairingOwner` / Gateway (pending CLI and gateway chat-assistant terminal display pipeline).
   - **Oracles**: `OCRM-QR-PNG-O01`, `OCRM-QR-PNG-O02`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-QR-TERMINAL-CALLER-001"]`: pairing CLI commands and Gateway chat-assistant terminal display pipelines do not yet route setup code payloads through `render_qr_terminal`).

2. **`OCRM-RESP-MEDIA-QR-IMAGE-001`**:
   - **Behavior**: Bounded QR code PNG rendering (scale 1..12, margin 0..16), Data URL generation (`data:image/png;base64,`), and scoped temporary directory workspace file generation with path segment sanitization.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_qr_png.rs::render_qr_png_data_url, write_qr_png_temp_file`.
   - **Production Call Chain**: Producer `AttachmentsService::handle_openclaw_media_gateway_call("renderQrPngDataUrl")` → Owner `render_qr_png_data_url` → Consumer (pending Gateway device pairing RPC and desktop pairing UI data URL presentation).
   - **Oracles**: `OCRM-QR-PNG-O03`, `OCRM-QR-PNG-O04`, `OCRM-QR-PNG-O05`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-QR-IMAGE-CALLER-001"]`: device pairing gateway RPC and UI pairing presentations do not yet consume `render_qr_png_data_url`).

3. **`OCRM-RESP-MEDIA-PNG-ENCODE-001`**:
   - **Behavior**: Pure Rust PNG chunk encoder (signature, IHDR, IDAT with zlib deflate compression, IEND, CRC32 checksum) supporting tightly packed RGB (3 channels) and RGBA (4 channels) pixel buffers and bounds-checked `fill_pixel` buffer manipulation.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_qr_png.rs::encode_png_rgb, encode_png_rgba, fill_pixel`.
   - **Production Call Chain**: Producer `render_qr_png_buffer` → Owner `encode_png_rgb` → Consumer (pending upstream image processor and canvas renderer consumption).
   - **Oracles**: `OCRM-QR-PNG-O06`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-PNG-ENCODE-CALLER-001"]`: native image processor and canvas export pipelines do not yet route raw pixel rasters through `encode_png_rgb` / `encode_png_rgba`).

4. **`OCRM-RESP-MEDIA-PHOTON-RUNTIME-001`**:
   - **Behavior**: Fallback BMP-to-PNG transcoder specification and runtime safety isolation with zero-leak buffer lifecycle management for unsupported raster image formats.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_qr_png.rs::convert_bmp_to_png`.
   - **Production Call Chain**: Producer `AttachmentsService::handle_openclaw_media_gateway_call("convertBmpToPng")` → Owner `convert_bmp_to_png` → Consumer (pending Rastermill fallback and media understanding transcode consumption).
   - **Oracles**: `OCRM-QR-PNG-O07`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-PHOTON-BMP-PIPELINE-001"]`: image understanding and format transcoding pipelines do not yet invoke `convert_bmp_to_png` on unsupported BMP inputs).

5. **`OCRM-RESP-MEDIA-STORE-SHARED-001`**:
   - **Behavior**: Shared media storage invariants (`MEDIA_FILE_MODE = 0o644`), legacy-binary byte size formatting with unit capping at megabytes, and typed `SaveMediaSourceError` error category dispatch.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_store_shared.rs::MEDIA_FILE_MODE, format_media_size, SaveMediaSourceError`.
   - **Production Call Chain**: Producer `AttachmentsService::handle_openclaw_media_gateway_call("formatMediaSize")` → Owner `format_media_size` → Consumer (pending `src/media/store.ts` file persistence integration).
   - **Oracles**: `OCRM-QR-PNG-O08`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-STORE-SHARED-PIPELINE-001"]`: media persistence pipeline `src/media/store.ts` has not yet reached implementation stage to enforce `MEDIA_FILE_MODE` and `SaveMediaSourceError`).

---

## 3. Strict Conservation Invariant Verification

- Previous mapped paths: **176**
- Newly mapped in Unit 20: **6**
- Updated mapped paths: **182**
- Updated unclassified paths: **58,568**
- Total tracked paths: **182 + 58,568 = 58,750** (strictly conserved, zero omitted)
- Global completion flags: All 8 flags remain `false` pending audited release.

---

## 4. Verification & Contract Test Coverage

All contracts verified in `source/host/tests/openclaw_qr_png_contract.rs`:
- `test_ocrm_qr_terminal_compact`: Half-block terminal characters, ANSI escapes, quiet-zone bounds.
- `test_ocrm_qr_terminal_standard`: Standard full-block terminal fallback rendering.
- `test_ocrm_qr_image_options_validation`: Scale bounds 1..12, margin bounds 0..16, finite float flooring, path segment traversal rejection.
- `test_ocrm_qr_png_data_url_and_buffer`: PNG magic byte validation, Data URL prefix, base64 roundtrip fidelity.
- `test_ocrm_qr_png_temp_file_lifecycle`: Scoped temp workspace directory creation, file write, root exposure, cleanup.
- `test_ocrm_png_encode_raw_rgb_and_rgba`: 24-bit RGB and 32-bit RGBA scanlines, None filter (0), zlib deflate compression, chunk CRC32, bounds-checked `fill_pixel`.
- `test_ocrm_bmp_to_png_conversion`: 24-bit uncompressed BMP parsing, BGR-to-RGB conversion, 4-byte row alignment, negative height support, truncated header and invalid compression rejection.
- `test_ocrm_media_store_shared_invariants`: `MEDIA_FILE_MODE = 0o644`, legacy-binary `format_media_size` with MB cap, `SaveMediaSourceError` typed codes.
