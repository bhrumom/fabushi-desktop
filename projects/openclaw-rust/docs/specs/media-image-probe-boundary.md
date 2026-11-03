# PNG header probe bounds — IPB-v1

Status: implementation contract; independent review and exact-H Actions pending.
Integration base H `8cf6143501b8c69b051d98df05f3ed523b364268`, fixed U
`1440bb1771372334bea23d7642bb52deb7db1845`. The image-owner and existing
image contract-test files are unchanged from exact target `5593e6d96e849ce5bf93de299aedeffca70b9568`.

## Observed failure and decision

The existing Rust `image_ops::read_image_probe_from_header` admits a PNG
buffer at length 24, then indexes byte 25 for color type. A 24- or 25-byte
buffer with PNG signature and IHDR tag reaches an out-of-bounds panic.
`read_image_metadata_from_header`, `prepare_attachment`, existing local
`web_media::load_web_media`, and the Attachments service consume this same
probe. This is a concrete existing production admission defect exposed by
source review of the adjacent HTML loader, not a new image codec requirement.

Root decision **IPB-F01 / IMPL_BUG / FIX**: require the bytes actually read
before recognizing PNG metadata. Return `None` for a candidate missing the
color-type byte. Keep the current header-only metadata semantics for buffers
that contain it; do not invent a full PNG decode/CRC admission gate in this
bounded repair. The existing real consumers must survive the short prefixes
and preserve their ordinary no-metadata behavior.

## Source and ownership boundary

Fixed-U `src/media/image-ops.ts`, blob
`8ee68607c533dac5134996a21919710de046d3e7`, imports/reexports the header probe
from Rastermill (1–17) and uses it for optional display dimensions (52–66).
The fixed root package declares `rastermill@0.4.0` at package.json:2458, blob
`6989c0618742d3c2b93cc336cbe1d9c3c4f0a657`; pnpm-lock.yaml blob
`83b4975eb96942ec007a2eb9847ccfe98d380bda` binds its resolution at 9903–9905
to `sha512-5WrZ6tuK54oVk45w/riFm0/VwTSieJ45c/zAGIaNtTKOSYL2zwx1r3VdBB2pKwlOO/tG+A9lmvN+3S6hwU6znA==`
and dependency snapshot at 18096–18098. These source ranges establish the
consumer/optional-metadata contract and exact dependency identity; they are
not a claim to have reviewed every Rastermill codec or dependency binary.
The actual npm distribution `rastermill-0.4.0.tgz` was acquired without
installation or execution: 41,458 bytes, SHA-512
`e56ad9eadb8ae78a15938e70feb8859b4fd5c134a2789e3973fcc018868db5328e4982f6cf0c75af755d041da92b094e3bfb46f80f659af37edd2ea1c14eb39c`,
matching that fixed resolution. Its exported `dist/index.js` header probes
and PNG helpers were read at 161–200 and 637–699, with buffer normalization
and dimension admission at 82–87 and 155–160; declaration and package/license
contracts were also inspected. Full source/member identities and actual
reading depths are retained as batch evidence, not a second OCRM inventory.

The distributed PNG probe returns dimensions at 24/25 bytes with nullable
unknown alpha. Rust's existing result instead requires boolean alpha and
already reads byte 25. Root retains IPB-F01's 26-byte totality correction:
do not introduce a shared result-schema change in this repair or claim
all-prefix Rastermill equivalence. Optional alpha, `tRNS`, other formats,
Photon 0.3.4 internals and full image compatibility remain applicable open
work. This repair's direct oracle is the existing target function's admitted
byte indices and real downstream consumers.

The existing `image_ops.rs` owns probing and remains the sole implementation.
No new module, codec, dependency, worker, store or permission truth is needed.
The operation consumes borrowed bytes synchronously and has no durable state,
side effect, pending job, cancellation or restart protocol. The actual file
loader retains its existing bounded read and path checks.

## Requirements and oracle

| ID | Required behavior |
| --- | --- |
| IPB-01 | Every prefix of a representative PNG header through the missing color byte returns no metadata without panic, including the exact former 24/25-byte failures. A direct normal call is the oracle; do not catch and reinterpret a panic as success. |
| IPB-02 | Once required probe bytes are present, existing width, height, format, alpha and byte-count behavior is preserved. Check alpha/non-alpha and zero-dimension refusals; retain existing complete PNG/BMP/JPEG/WebP controls. Do not call a header-only fixture a decodable or CRC-valid PNG. |
| IPB-03 | Existing `prepare_attachment`/metadata facade and actual file-backed loader/Attachments service handle the short headers without crashing and retain bytes/no-metadata behavior. Repeat after replacing the same file with a sufficient header and prove fresh metadata is observed. |
| IPB-04 | Keep the same byte cap, root, ordinary file and MIME behavior; no substitute decoder, data reader or generated-source trust. All existing image-ops assertions remain. |

Only the existing image owner and `openclaw_image_ops_contract` target need
production/test edits. Root integrates the spec, narrow ledger/RTM evidence
and adds the complete image contract target to the existing strict media
Actions capture, sharing its native Linux/macOS/Windows jobs and cache.
Preserve exact named positives, commands/exits, raw logs, input hashes and
H/tree/U binding. No local or htch build/test/validator is permitted.

Freeze the complete changes and obtain fresh independent general/security
and pinned structure/reuse/performance review before publication. No test
has been run at contract creation. Current full image policy/decoder/codec,
real platform/package/UI and G01–G12/AC01–AC24 acceptance remain open; all
existing OCRM source history and completion flags are preserved.
