Live Revision 9 authority (2026-10-11): telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 (root tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6). Root non-directory=6,653; recursive non-directory=16,125; deterministic read-through=10,540; first unread=10,541 Telegram/ThirdParty/nimf::docs/html/executables.html@c627d62e1c8e6bf766845a9e7b5116330cb04d12; unread=5,585; unknown=15,845; omitted=0. Orders 10,477–10,540 are exact-read from exact-head source-authority run 38128873892 / job 114438921053 / artifact 11698811298 (digest sha256:278638d19fd8a347cf20681953a9589ec06413033ff72e7cf9e3b4c6b5e1c209). Nimf exposes build/package/settings, daemon/IPC activation and input-context focus/preedit/candidate/surrounding-text state-machine semantics. Exact-head Fabushi audit found these applicable text-input responsibilities already owned by canonical PromptRichTextEditor/platform-input: compositionstart/update/end, event.isComposing plus internal composition fencing, scope-settlement fencing, preedit/focus lifecycle and fail-unknown native input-source probing. No Nimf daemon/runtime or second IME owner is added.
<!-- TDRP_CURRENT_SUMMARY read-through=10540 unread=5585 unknown=15845 omitted=0 first-unread=10541 path=Telegram/ThirdParty/nimf::docs/html/executables.html -->

### Revision 9 deterministic read-through — Nimf build/settings/input-context API 10,477–10,540

Exact identities/bytes: `84250c295a108a9b5f108b6c474cbeff71dc1097`, run `38128873892`, source-authority job `114438921053`, artifact `11698811298`, digest `sha256:278638d19fd8a347cf20681953a9589ec06413033ff72e7cf9e3b4c6b5e1c209`. Strict Revision 9 validation, review-window regression, exact materialization and accepted-mode fail-closed checks all passed.

This range covers Nimf build/install/package/license metadata, settings persistence/control, daemon lifecycle, per-user Unix-socket AppArmor boundary, system IM activation variables, visual resources and the generated public candidate/preedit/engine/server/service/input-context APIs. The applicable state machine includes focus in/out, event consumption, preedit start/update/end, commit, surrounding-text retrieval/deletion, cursor location, reset, engine/method changes, candidate paging/selection and service active lifecycle.

Existing-owner-first audit found canonical Fabushi already implements the critical shipping composition boundary: `PromptRichTextEditor` listens to `compositionstart/update/end`, rejects submit while either browser `event.isComposing` or internal composition state is active, fences scope changes through the old composition's final input, reconciles after composition end, and reports preedit/focus/lifecycle state. `source/electron-main/platform-input.ts` probes only installed native input providers with bounded values/timeouts and returns unknown rather than guessing unsupported method/language/candidate state. Therefore no Nimf daemon, system IM override or second input runtime is justified by this tranche.

All 64 rows remain `mapped-open` / `unknown_closed=false`. Accounting: **10,540/16,125 read; 5,585 unread; 15,845 unknown; 0 omitted**. Next: 10,541 `Telegram/ThirdParty/nimf::docs/html/executables.html@c627d62e1c8e6bf766845a9e7b5116330cb04d12`.


### Revision 9 deterministic read-through — LZ4 tail + Nimf entry 10,413–10,476

Exact bytes come from exact-head `362099f6a830c1de3a493d4e718f1fe850bec4a1` run `38128655830`, source-authority job `114438220656`, artifact `11698361416` (digest `sha256:bac2f13bc1492b139d1670a2c106151be410618636049e852999cb7fb4f248ee`). Revision 9 validation, review-window regression, exact materialization and accepted-mode fail-closed checks succeeded on that head.

Orders 10,413–10,470 finish pinned LZ4. The tail adds adversarial insufficient-buffer compression/decompression, full/partial/dictionary/stream round trips, deterministic fuzz parameter generation, frame concatenation/skippable-frame behavior, CLI file/stdin/stdout/overwrite/remove/sparse semantics, ABI/tag/link-version checks, cross-version interoperability, install/uninstall, custom block-size boundaries, freestanding/custom allocator coverage and performance-regression monitoring. GPL programs/tests are treated as behavioral/quality authority only; their implementation is not copied.

The current Fabushi pack owner already uses bounded gzip + tar/resource paths: compressed and decompressed member caps, checked offsets/sums, path containment, disk/resource budgets, exact size and SHA-256 verification, and deterministic cleanup. Thus no LZ4-specific shipping gap or second compression/archive owner is proven. Orders 10,471–10,476 begin Nimf with issue templates, Debian release workflow, ignore rules and authorship provenance; metadata alone does not justify a Nimf/IME runtime.

All 64 rows remain `mapped-open` / `unknown_closed=false`. Accounting is **10,476/16,125 read; 5,649 unread; 15,845 unknown; 0 omitted**. Next is 10,477 `Telegram/ThirdParty/nimf::BUILD.md@fbea94fb3836ea4d04faf23734f741891731393b`.


### Revision 9 deterministic read-through — LZ4 manuals / examples / core / first OSS-Fuzz 10,349–10,412

Exact identities and bytes come from exact-head `99e027714e9ee707e544073c6e9240fa0c77c289` run `38127755807`, source-authority job `114435611847`, artifact `11697459714` (digest `sha256:c53bc4284dd3e358aa891df11d89083625fa56c2092d4f4b8ab1a509513a4953`). The range covers generated API-manual tooling, Meson and Snap packaging, block/frame format contracts, reference streaming/dictionary/file examples, library build/DLL/pkg-config/license inputs, the block/HC/frame/file/xxHash implementation surfaces, and the first OSS-Fuzz frame target.

Applicable source-neutral behavior is explicit: decoders require exact compressed/output bounds and reject malformed input; frame state carries version/block mode/content size/dictionary/checksum semantics; streaming callers own retained history and context cleanup; allocation, overflow and block-size limits are fail-closed; file wrappers have open/read-write/close error lifecycles; xxHash is corruption detection rather than cryptographic authentication; fuzz coverage deliberately exercises undersized output buffers and exact round trips. Existing-owner-first maps these to canonical Resource/Storage, restore/archive, Build/Quality, Packaging/Updater and License owners.

Current shipping Host already declares `flate2` + `tar`, and current Box/resource paths enforce path containment, byte/hash/size checks, disk-space/resource limits and deterministic cleanup. No current LZ4/Zstd dependency or LZ4-specific shipping consumer was proven by this range. Therefore this tranche adds no compression runtime or second archive owner. All 64 rows remain `mapped-open` / `unknown_closed=false` until later LZ4 consumer reachability and product/release evidence determine applicability.

Accounting is **10,412/16,125 read; 5,713 unread; 15,845 unknown; 0 omitted**. Next is 10,413 `Telegram/ThirdParty/lz4::ossfuzz/compress_fuzzer.c@edc8aad9a929dc7d384de191a812e83b632f22d8`.


### Revision 9 deterministic read-through — LZ4 build / quality / packaging 10,285–10,348

Exact bytes come from `c95e04afee5e8fa712c5329f6331c9d945668345` run `38127286071`, source-authority job `114434237346`, artifact `11697319116` (digest `sha256:45e8495d4f0f015a97191be79bc98247423d00797a42aa6b05a532b7cc0c5314`). The range covers LZ4 CI/toolchain matrices, Windows VS2010/2017/2022 build/test/package definitions, CMake export/install contracts, Debian packaging and legacy DJGPP portability/license inputs.

Existing-owner-first keeps Fabushi's current Build/Quality/Platform/Packaging/Updater/License owners. Current Fabushi has no LZ4/Zstd production dependency; accepted tdesktop reachability seen so far is the direct gitlink plus Snap `liblz4-dev` build package, with no direct Telegram source/CMake consumer found in this tranche. Therefore no LZ4/Telegram runtime or second compression owner is introduced. Historical upstream allow-failure cases are retained only as risk signals and do not weaken Fabushi gates. All 64 rows stay `mapped-open` / `unknown_closed=false` until later LZ4 core source and consumer reachability establish applicability.

Accounting is **10,348/16,125 read; 5,777 unread; 15,845 unknown; 0 omitted**. Next is 10,349 `Telegram/ThirdParty/lz4::contrib/gen_manual/.gitignore@6ea967f7c148b0a5e27b39ee35afaa509c588b33`.


### Revision 9 deterministic read-through — libfido2 tail / libprisma / LZ4 entry 10,221–10,284

Exact bytes come from `f810ac1eb2edba61afd6d2f89141d439b3f6e407` run `38124618782`, source-authority job `114426354062`, artifact `11696915688` (digest `sha256:306a7639eb06e2851ca640b6d6bc48ef5d552d7f194541c1263a42c72e041b9e`). The range finishes libfido2's reset/U2F/TPM/WinHello/security-key-management/platform packaging responsibilities, reads all of libprisma's Prism-derived grammar/tokenization implementation and generated grammar database, and begins LZ4's cross-platform quality matrix.

Existing-owner-first found one concrete desktop product gap: Rust/TUI already has bounded `syntect/two_face`, but the shipping React Conversation Transcript rendered fenced code as raw text. The descendant keeps Transcript as the only desktop owner and uses pinned PrismJS 1.30.0 (MIT, zero dependencies) as an offline tokenizer dependency. Token output is rendered as React text/span nodes rather than HTML, streaming content and unknown/over-budget inputs fall back to plain text, and the existing 512 KiB/10,000-line bound is preserved. Existing Focused Electron acceptance is extended to prove TypeScript keyword/number tokens on the shipping assistant transcript. The libprisma rows nevertheless remain `mapped-open`: its upstream language catalogue exceeds the initial common grammar set and exact-head descendant evidence is still required.

Accounting is **10,284/16,125 read; 5,841 unread; 15,845 unknown; 0 omitted**. Next is 10,285 `Telegram/ThirdParty/lz4::.circleci/images/primary/Dockerfile@77670148ccfef16d6490070911b62e2a8ff4ae8a`.

### Revision 9 deterministic read-through — libfido2 core/transports 10,157–10,220

Exact identities and materialized blobs come from exact-head `8cd23e6ec07d69b186be674947ac45ce38f3453a` Rust runtime run `38123961987`, source-authority job `114424363470`, artifact `11695949458` (digest `sha256:fa16db976f759301292bf23dd454bed9f6e70ad386c772333bfbb93c0b329291`). Orders 10,157–10,220 cover ES384/RS256 regressions, mock authenticator transport, build/export composition, assertion and credential ceremonies, PIN/UV and device timeout/cancel state, CBOR, biometric/configuration/credential management, HID across supported desktop platforms, getInfo capability discovery, ISO7816/NFC/PCSC, largeBlob, Linux device discovery, diagnostics and OS CSPRNG behavior.

Existing-owner-first keeps the current Coordinator WebAuthn/security runtime plus Host bridge, canonical Settings/platform-device, Secrets/Security/Crypto, Platform/Resource, Telemetry/Diagnostics and Build/Packaging/Release owners. No Telegram/libfido2 runtime, second credential store, second Settings owner or custom cryptography is introduced. The source review exposed one concrete lifecycle gap in the existing spawned signer: explicit cancellation was supported, but a helper could remain alive indefinitely when no remote cancel arrived. This descendant derives the RP timeout hint from the frozen ceremony options, clamps it to the WebAuthn client 300,000–600,000 ms range with a 300,000 ms default, and kills/reaps a stuck helper at the deadline. Focused Rust tests cover timeout extraction/clamping and a synthetic stuck Unix child.

All 64 source rows remain `mapped-open` / `unknown_closed=false`: source reading and one production hardening do not close packaged helper provenance, real HID/NFC/PCSC hardware, resident-credential/biometric/largeBlob product policy or release evidence. Accounting is **10,220/16,125 read; 5,905 unread; 15,845 unknown; 0 omitted**. Next is 10,221 `Telegram/ThirdParty/libfido2::src/reset.c@4e09dbbca5cf968ddb9d728dcc13da88cdfd9264`.

### Revision 9 deterministic read-through — libfido2 API/compat/regression 10,093–10,156

Exact identities and materialized blobs come from predecessor exact-head `b8a52c954b9ea63e9f64b31d68fc537e71e1f2bf` Rust runtime run `38120045447`, source-authority job `114412753321`, artifact `11695420570` (digest `sha256:f08d296699a1041550c1fd1421f78b47c04f53f5063eab901369157f37275ab5`). Orders 10,093–10,156 cover libfido2 assertion/credential/key APIs, RP/client-data/authdata/signature verification, UP/UV/resident credential/HMAC/largeBlob behavior, PUAT/device timeout/cancel/touch/PIN state, biometric/credential management, OpenBSD portability/security primitives and assertion/credential/device/EdDSA/ES256 regression suites.

Existing-owner-first keeps the current Coordinator WebAuthn/security runtime plus Host bridge, canonical Settings/platform-device/secure-storage/Secrets and Build/Quality/Release. No Telegram/libfido2 side runtime, second identity store or custom cryptography is introduced. The source review exposed an applicable secret-lifetime gap in the existing helper path: PIN strings and their serialized helper request bytes were ordinary buffers. This descendant moves the PIN out of the control reply and uses pinned `zeroize 1.9.0` with `Zeroizing<String>` and `Zeroizing<Vec<u8>>` until the helper write completes. Packaged `sand-webauthn-signer` producer/co-staging/provenance and wider hardware-management evidence remain open; all rows in this tranche stay `mapped-open` / `unknown_closed=false` pending full production/release closure.

Accounting is **10,156/16,125 read; 5,969 unread; 15,845 unknown; 0 omitted**. Next is 10,157 `Telegram/ThirdParty/libfido2::regress/es384.c@b55ce015e17a2d58728fef19dd945017f05359ab`.


### Revision 9 deterministic read-through — libfido2 ceremony/device/fuzz/man 10,029–10,092

Exact identities and exact blob materialization come from `53166126816c059a99298b28d4b3d8c2ca204910` Rust runtime run `38119139308`, source-authority job `114410128913`, artifact `11693708963` (digest `sha256:29a6feb40f3cbaa28f932d822bab4242821378bba701bdfbba1fb18f26a35542`). Orders 10,029–10,092 cover macOS/OpenSSL3/Windows/ShellCheck build matrices; libfido2 feature/dependency hardening; assertion/credential verification; PIN/UV, resident credentials, credProtect, enterprise attestation, largeBlob, biometric enrollment, device enumeration/touch selection/reset; secure serialization; fuzz/sanitizer/fault-injection and FIDO2/U2F corpora; coverage evidence; and API/man/key-lifetime conformance.

Existing-owner-first keeps the current Coordinator WebAuthn/security owner plus canonical Settings/platform-device/secure-storage and Build/Quality/Release owners. No Telegram/libfido2 runtime, second Identity/Credential store, or custom cryptography is introduced. Comparing the upstream fail-closed semantics exposed a production defect in the existing spawned helper path: a non-zero helper exit could still be accepted when stdout said `ok:true`, and a success result could omit a usable credential object. Staging commit `6495e0ad637075c499d49ffd8d05c18f59987195` now rejects those cases and adds focused unit tests. The broader device-management, helper packaging/co-staging, adversarial security and release responsibilities remain `mapped-open` until exact-head production evidence exists.

Accounting is **10,092/16,125 read; 6,033 unread; 15,845 unknown; 0 omitted**. Next is 10,093 `Telegram/ThirdParty/libfido2::man/es384_pk_new.3@e865913b780700f95fe8f16c406e5b83065cbdd1`.


### Revision 9 deterministic read-through — libcbor tail + libfido2 start 9,965–10,028

Exact-head `f936a5e…` source range resolution and attestation succeeded in run `38118212864` / job `114407396444`, artifact `11693652953` (digest `sha256:e9307971683ac2f1920de0aed9d9cd9f9be0a4dbfd0f2d31910459cd80878970`). Revision 9 validation then failed solely because `inventory/index.json.note` did not use the strict validator-required `unread_minimum is … / unknown remains … / omitted remains …` narrative. This descendant restores that contract; the validator is not weakened.

Orders 9,965–10,005 close the libcbor component with adversarial tests for malformed/incomplete/memory/syntax separation, exact streaming required bytes, SIZE_MAX arithmetic/reallocation overflow, nesting limits, UTF-8 rejection, allocation-failure cleanup and reference-count stability. Orders 10,006–10,028 begin `Yubico/libfido2@b974e7c…` and establish cross-platform Debug/Release regression, scan-build, ShellCheck, CodeQL, ASan/MSan/OSS-Fuzz/CIFuzz and pinned build-dependency provenance. All map existing-owner-first to canonical WebAuthn/Account Auth security and Build/Quality/Release; no second FIDO/CBOR runtime is introduced.

At `f936a5e…`, Focused Electron chat E2E, renderer/platform-worker, all three updater jobs and all three pressure-profiler jobs succeeded; the independent Coordinator contract also passed with bounded WebAuthn helper-output hardening present. The same-head source-authority job is not final-green because of the narrative contract failure, so fresh descendant exact-head evidence remains required.

Accounting is **10,028/16,125 read; 6,097 unread; 15,845 unknown; 0 omitted**. Next is 10,029 `Telegram/ThirdParty/libfido2::.github/workflows/macos_builds.yml@5d4bc4def3bbea615f089b3a153d117563e65be4`.

### Revision 9 deterministic read-through — libcbor core 9,901–9,964

Exact-head source-authority run `38117917553`, job `114406468676`, at `2bc832466e640f7a8bb11c569450bd1d5cb89045` succeeded through validation, source-range attestation, review-window regression, exact materialization and accepted-mode fail-closed checks. Artifact `11694536942` (digest `sha256:e1d914906cc18e4a1debb277afba53553e9e3f6c755f69fdef7b8988bd586187`) materializes orders 9,901–9,964.

The range includes CBOR conformance fixtures, streaming/load examples, OSS-Fuzz with an explicit allocator budget, build/release tooling, and the core codec through `src/cbor/tags.c`. Applicable behavior includes malformed/incomplete/memory/syntax failure separation, size_t range checks, overflow-safe collection growth and serialized-size accounting, bounded parser nesting, strict UTF-8 validation, deterministic reference cleanup, and exact streaming progress/required-byte semantics.

Existing-owner-first audit maps these behaviors into the current Coordinator WebAuthn provider/signing lifecycle plus canonical Account/Auth security and Build/Quality/Release. The staged production hardening caps external signer stdout at 1 MiB and signer event output at 256 KiB, fails closed on oversize/IO failure, and makes non-interactive signing reuse the same bounded path; focused Rust unit tests cover exact-boundary, over-boundary and IO-failure reads. This implementation is not verified until fresh descendant exact-head GitHub Actions pass.

The external `sand-webauthn-signer` packaged-helper provenance/staging gap remains open. Accounting is **9,964/16,125 read; 6,161 unread; 15,845 unknown; 0 omitted**. Next is 9,965 `Telegram/ThirdParty/libcbor::src/cbor/tags.h@a7365df10208c949b303c7fb8afc9710a7ade78f`.

### Revision 9 deterministic read-through — libcbor build/docs/examples 9,837–9,900

Exact-head source-authority run `38116111367`, job `114401252125`, at `9e9d83a7944a02d269fbb0820cbc2e67a465bedc` succeeded and artifact `11693503533` (digest `sha256:30e45c473073318b0f159d391d6b27f5a5bb206a32e5fe9b481e4cab377a7e7c`) materialized exact identities for orders 9,837–9,900. The six Windows CMake screenshots were visually inspected as build documentation only.

The range establishes RFC 8949 parsing/serialization, reference ownership, streaming/incremental decode, bounded parser-stack and allocation/overflow/fuzz hardening responsibilities. Current exact-head audit corrects the earlier owner-absence note: Fabushi already ships a source-neutral Coordinator WebAuthn provider/signing lifecycle; Account/Auth passkeys remain a distinct canonical product lifecycle. These rows therefore stay mapped to existing authentication/security plus Build/Quality/Release owners. No libcbor, FIDO or Telegram runtime is introduced.

A shipping gap remains open: the Coordinator resolves an external `sand-webauthn-signer`, while the inspected desktop build/staging path does not yet prove that helper is produced and co-staged at the packaged lookup path. No row is marked verified from source reading alone.

Accounting is **9,900/16,125 read; 6,225 unread; 15,845 unknown; 0 omitted**. Next is 9,901 `Telegram/ThirdParty/libcbor::examples/create_items.c@a6e67a9870ef366248263bffff467a002e38bfa7`.

### Revision 9 deterministic read-through — KImageFormats tail and libcbor/WebAuthn entry 9,773–9,836

Exact-head source-authority run `38115822404`, job `114400440327`, at `17af29979829565d5d7cc0d7ec81b6db78d03b82` completed Revision 9 validation, review-window regression, materialization and expected fail-closed accepted-mode checks. Artifact `11692899297` (digest `sha256:95b4ca47e543b78d48f45a7bd3298655e187b8f9227e890e3265fdd3794abb9f`) re-attested exact Git blob identities for orders 9,773–9,836.

Orders 9,773–9,828 close the remaining KDE KImageFormats source range: JPEG XL, KRA/ORA package projection, PCX/PIC/PSD/QOI/RAS/RAW/RGB/TGA/XCF, shared RLE/scanline/allocation helpers and codec test utilities. The component is therefore fully read, but its responsibility rows remain `mapped-open` because direct KDE KImageFormats product composition is absent from pinned tdesktop CMake and applicability must be closed responsibility-by-responsibility rather than inferred from recursive checkout.

Order 9,829 begins `PJK/libcbor@170bee2b82cdb7b2ed25af301f62cb6efdd40ec1`. Unlike KImageFormats, libcbor has confirmed shipping reachability: pinned `Telegram/cmake/lib_fido2.cmake` directly compiles libcbor C sources into tdesktop's USB-HID CTAP2/WebAuthn security-key backend and pins libcbor 0.11.0 with `CBOR_BUFFER_GROWTH=2` and `CBOR_MAX_STACK_SIZE=2048`. Current Fabushi search found no dedicated WebAuthn/passkey/FIDO owner, so this responsibility cannot be declared non-applicable merely because there is no Telegram network dependency. Existing Fabushi Identity/account/security owners must be audited first; only if owner absence is proven may a minimal source-neutral WebAuthn capability owner be introduced.

The first libcbor rows cover cross-platform CI, sanitizer/Valgrind testing, fuzz workflows and repository quality contracts. Ongoing source review has already identified protocol-neutral requirements for overflow-proof allocation, parser stack limits, allocator-failure cleanup, malformed-input fail-closed behavior, incremental decoder required-byte semantics, UTF-8/codepoint handling and fuzzing of network-like CBOR. These remain mapped to the future canonical authentication/protocol/resource boundary pending full lib_fido2/WebAuthn context.

Persisted accounting is **9,836/16,125 read; 6,289 unread; 15,845 unknown; 0 omitted**. Next is 9,837 `Telegram/ThirdParty/libcbor::.gitmodules@b1f6f9c259a83355bd23576a06702fd1d8541d44`. Source reading is not product verification; exact-head CI and responsibility-level closure remain mandatory.

### Revision 9 deterministic read-through — XCF/write harness and image-plugin implementations 9,709–9,772

GitHub Actions source-authority run `38114691755` at exact head `9ee0141a340df211836d139db8d7fe0a5e9ac657` materialized orders 9,709–9,772 in artifact `11693336517` (digest `sha256:372545e780f881f81a5043b5e5aa47b62ed40ab7c67e920633884cc8f72b75aa`) after strict source-range attestation. The exact window closes the remaining XCF ICC/reference fixtures, shared random/sequential read and write/readback harnesses, LibRaw/build/plugin metadata, and the first ANI/AVIF/EPS/EXR/HDR/HEIF implementation sources.

Pinned tdesktop reachability was checked separately. The direct KImageFormats gitlink is recursively checked out, but the exact root/Telegram product CMake does not add or include KDE KImageFormats; accepted preparation and packaged dependency paths select Qt `qtimageformats`. Recursive checkout therefore proves source availability, not direct KDE codec shipping composition. No KDE/Telegram codec runtime or second media registry is introduced.

Existing-owner-first exposed one applicable production delta in a capability Fabushi already ships: AVIF/HEIF attachment preview dimensions. The canonical Host ISO-BMFF parser already resolves the primary item `ispe` and `irot`; it now also applies a valid shrinking AVIF `clap` clean aperture before rotation. Malformed or expanding `clap` data is ignored rather than expanding dimensions or failing preview ingestion. Commit `e5f8efc750d2c00db98760311da78f5a5ef7a2b0` implements this in `link_preview_image_bounds.rs` with a focused synthetic primary-item `ispe + clap + irot` contract test. Descendant exact-head GitHub Actions remain required before marking the production delta verified.

All 64 source rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`: exact reading plus one implemented sub-responsibility does not close the full codec/product responsibility set. Accounting is **9,772/16,125 read; 6,353 unread; 15,845 unknown; 0 omitted**. Next is 9,773 `Telegram/ThirdParty/kimageformats::src/imageformats/jxl.cpp@bb3f09bf96a794c455b1f002309e623eed88d1db`.

### Revision 9 deterministic read-through — SGI RGB / TGA / XCF window 9,645–9,708

`cbf48857db75d2c5557385723ae449acaa6de54d` exact-head Rust runtime run `38114391045` passed source-range attestation, strict Revision 9 validation and the review-window helper before materializing orders 9,645–9,708. Artifact `11692612830` (`sha256:c6c1d7244ee4495e3671e4003189e526e7c83ede7b914871038cb5fc05d5d4ab`) contains all 64 exact blobs and a window manifest with recomputed Git blob SHA for every entry.

SGI fixtures constrain 32×32 three-channel one-byte/channel decode across aggressive RLE, ordinary RLE and verbatim storage. TGA fixtures constrain 24/32-bit RGB/RGBA, RLE/raw storage and upper/lower-left origin handling. XCF fixtures constrain v0/v11/v12, RGB/grayscale/alpha output, precision codes for U8/U16/U32 integer plus half/float samples, a historical bug case and ICC-profile variants. Exact headers/reference dimensions were inspected from the artifact, not inferred from filenames.

Existing-owner audit finds one canonical Fabushi Media/Resource path: `source/host/src/media_mime.rs` plus Electron attachment/media owners. That shipping path currently declares AVIF/BMP/GIF/ICO/JPEG/PNG/SVG/WebP and servable HEIC/HEIF, not SGI RGB/TGA/XCF. Since pinned tdesktop build/shipping reachability for kimageformats remains unproven, all 64 rows stay `mapped-open`, `unknown_closed=false`, `omitted=false`; no KDE/Telegram codec runtime or second media registry is added. Accounting is **9,708/16,125 read; 6,417 unread; 15,845 unknown; 0 omitted**. Next is 9,709 `Telegram/ThirdParty/kimageformats::autotests/read/xcf/fruktpilot_icc.xcf@c57c6548d7ddb8a3b0e9556a2df8323cde65be06`. Fresh descendant exact-head Actions remain mandatory.

### Revision 9 deterministic read-through — PSD/QOI/RAS/RAW/SGI window 9,581–9,644

GitHub Actions run `38113926190` at exact head `05ac0fb3c69f4f5b8c5d2266ec9eefaec1f5feb6` materialized the persisted first-unread window into artifact `11692219924` (`sha256:f4bc0854618bc82f39638d59d728538f3e5be77488ccc71c9d779a24bf1942b4`) and recomputed every Git blob SHA before upload. The prior >1 MiB connector gap is therefore closed by exact Actions evidence rather than inference.

Orders 9,581–9,625 cover PSD/PSB/PDD/TIFF acceptance inputs and references: PSD version 1 / PSB version 2, 1/8/16/32-bit depths, 1–57 channels, Bitmap/Grayscale/Indexed/RGB/CMYK/Multichannel/Duotone/Lab modes, Photoshop/Affinity/GIMP compatibility fixtures and paired output dimensions. Orders 9,626–9,637 cover QOI RGB/RGBA from 1×1 through 256×256; 9,638–9,639 cover 24-bit Sun Raster; 9,640–9,641 cover Kodak RAW plus its 1216×912 reference; 9,642–9,644 begin SGI RGB coverage. Exact binary headers/dimensions/depth/channel fields were inspected from the Actions artifact.

All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`: source reading establishes codec/resource acceptance responsibilities but does not prove pinned tdesktop shipping reachability or Fabushi codec parity. Existing-owner-first keeps the source-neutral canonical Media/Resource owner; no KDE/Telegram media runtime or second registry is introduced. Accounting is **9,644/16,125 read; 6,481 unread; 15,845 unknown; 0 omitted**. Next is 9,645 `Telegram/ThirdParty/kimageformats::autotests/read/rgb/rgb-gimp-2.8.10-aggressive-rle.rgb@c5f7f03f816516189825acad40721c83558d8ea9`. Fresh descendant exact-head Actions remain mandatory.

### Revision 9 deterministic read-through — kimageformats decode corpus 9,518–9,580

Orders 9,518–9,580 exact-read the next pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec` acceptance corpus: EPS, OpenEXR, Radiance HDR, HEIF/HEJ2, JPEG XL, KRA/ORA, PCX and the opening 16-bit PSD cases. Binary content was read directly and inspected by magic/header/reference dimensions rather than inferred from file names. JXL orientation 1–8 converges through the shared `readtest.cpp` auto-transform contract; KRA/ORA flatten to one 200×200 reference; PCX spans mono, indexed and RGB planes; PSD begins 16-bit RGB/RLE and grayscale depth coverage.

Pinned tdesktop reachability was also audited. Repository search at `telegramdesktop/tdesktop@6fed91ff…` currently finds `kimageformats` only in `.gitmodules`, with no direct `kimg_*`, `QImageIOPlugin` or `ThirdParty/kimageformats` shipping-build reference. Therefore codec fixtures remain source authority but their product applicability cannot be upgraded merely because the submodule contains a decoder. Fabushi's existing Media/Resource owner remains canonical.

All 63 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **9,580/16,125 read; 6,545 unread; 15,845 unknown; 0 omitted**. The next exact source is 9,581 `Telegram/ThirdParty/kimageformats::autotests/read/psd/16bit_photoshop.psb@c09d6bb5bc492441d4684531aaa1a6fbe67b4c60`. The GitHub Contents connector currently does not return that >1 MiB binary payload, so it remains unread rather than inferred; work continues on other non-blocked responsibilities. Fresh descendant exact-head Actions remain mandatory.

### Revision 9 deterministic read-through — PIC/AVIF media acceptance 9,474–9,517

Orders 9,474–9,517 exact-read the next pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec` range. PIC fixtures and `pictest.cpp` establish raw/RLE compression, alpha/non-alpha output, deterministic encoded bytes, short/full-length Description metadata, long runs, input-format conversion, and pre-read text/size/image-format semantics. AVIF fixtures cover grayscale, grayscale-alpha, RGB/RGBA and the full auto-transform orientation matrix: unrotated/90/180/270 combined with horizontal or vertical mirroring all converge to the same canonical 480×160 reference orientation.

Binary fixtures were read by exact base64/raw bytes; PNG dimensions and selected reference visuals were inspected. Existing-owner-first keeps the canonical attachment/media, Resource and Media/Quality owners. The current production branch also bounds the existing local-image preview fast path before decode using `ATTACHMENT_PREVIEW_BYTE_CAP`; the strict Grok semantic-adaptation evidence lock is rebound to those intentional target blobs rather than weakened.

All 44 source rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; the bounded preview fix is narrower than full codec parity. Accounting is **9,517/16,125 read; 6,608 unread; 15,845 unknown; 0 omitted**. Next is 9,518 `Telegram/ThirdParty/kimageformats::autotests/read/eps/bw.eps@3af28d5d28351a749cbcaf0e30e3652a159d830a`. Fresh descendant exact-head GitHub Actions remain mandatory.

### Revision 9 deterministic read-through — kimageformats opening range 9,454–9,473

Orders 9,454–9,473 exact-read the opening range of pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec`: repository/build policy, optional codec dependency gates, exact license corpus, declared read-only/read-write image capabilities, format acceptance topology, the ANI binary fixture plus all three frame PNGs, its decoder contract, and the bounded fuzzy image comparator. Binary fixtures were read by exact base64/raw bytes and the PNG frames were visually inspected rather than inferred from filenames.

The ANI contract preserves 32×32 dimensions, Title/Author metadata, four-frame custom sequencing, 166/333 ms delays, end-of-stream behavior and jump/reset semantics. Existing-owner-first keeps attachment/media, Resource, Build/Packaging and Release/NOTICE owners canonical. The current branch separately fixes the shipping local-image fast path so the canonical preview-byte cap is enforced before file decode; that narrow production fix does not mark this broader codec range verified.

All 20 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **9,473/16,125 read; 6,652 unread; 15,845 unknown; 0 omitted**. Next is 9,474 `Telegram/ThirdParty/kimageformats::autotests/long-runs.pic@620a3bb8eb322d06e0468eb641c56bd72b6c6545`. Fresh descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — kcoreaddons test/demo tail 9,445–9,453

Orders 9,445–9,453 exact-read the nine final non-directory blobs of pinned `KDE/kcoreaddons@fd84da51b554eac25e35b1e3f373edaab3029b15`. They cover the optional Qt Widgets manual-test build, system-user face-icon discovery with missing-icon fallback and bounded scaling, directory-watch dirty/created/deleted events with explicit scan start/stop/restart plus multi-watcher file/subdirectory observation, deterministic seeded random-sequence/shuffle behavior, and a debounced plaintext-to-HTML demo surface.

Existing-owner-first keeps Build/Quality, platform user/profile Resource, Resource/filesystem-watch, security randomness/test utility and Transcript safe-rendering owners canonical. The Qt demo UI is replacement-only; system-user data is not Fabushi account identity; deterministic `KRandomSequence` behavior is never security authority for tokens/nonces/identity. This range corrects the exact-head CI defect that proved `src/mimetypes/kde5.xml` was not the component tail. The validator remains strict; no source entry is skipped.

All nine rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **9,453/16,125 read; 6,672 unread; 15,845 unknown; 0 omitted**. Kcoreaddons is now exact-read through all 377 non-directory entries. Next is 9,454 `Telegram/ThirdParty/kimageformats::.git-blame-ignore-revs@b34bba3cd8fb4c6785690422f5c46762d60bb7dc` in pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec`. Fresh descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — kcoreaddons component tail 9,401–9,444

Orders 9,401–9,444 exact-read the remainder of pinned `KDE/kcoreaddons`: safe text/emoticon interface, legacy configuration migration, locale-aware byte/duration/date formatting, executable/open-file discovery, memory telemetry, `os-release`, cross-platform process enumeration, runtime-platform hints, sandbox host bridging, Unix/Windows shell parsing and quoting, signal integration, Unix/Windows user/group/SID semantics, and the final MIME corpus (57 types, 73 globs, 26 subclass relations, 9 magic rules, 3 aliases).

Existing-owner-first maps these into canonical Localization, Settings/Persistence, Resource/platform, Host/Runner/Computer, Permissions, Transcript and attachment/MIME owners. Critical boundaries stay explicit: migrations never overwrite existing target state; open-files reports typed unsupported rather than fake success; memory distinguishes free from available; OS user/group data is platform permission context, not Fabushi account identity; sandbox/shell helpers cannot bypass approval; MIME and rendered-link inputs stay validation-bound. No KDE runtime or second platform/settings/identity truth is introduced.

This also fixes an authority-accounting defect exposed by exact-head CI: `upstream.lock.source_disposition_evidence` had remained at 9,201 while master/index/ledger had advanced. The lock summary is now advanced from the same source prefix instead of weakening the validator.

Accounting: **9,444/16,125 read; 6,681 unread; 15,845 unknown; 0 omitted**. Kcoreaddons source reading is complete; next 9,445 is `Telegram/ThirdParty/kimageformats::.git-blame-ignore-revs@b34bba3cd8fb4c6785690422f5c46762d60bb7dc` in pinned `KDE/kimageformats@df82311a1081e576c4ac020204578bb8a81b21ec`.


### Revision 9 deterministic read-through — plugin/text/security utility source 9,381–9,400

Orders 9,381–9,400 exact-read static-plugin helpers, random/shuffle APIs, fuzzy matching/ranking, localized JSON lookup, macro expansion and platform shell quoting, string utilities, and bounded plain-text → HTML/linkification. The security disposition is explicit: upstream KRandom itself says it is not suitable for serious randomness, so it is never authority for Fabushi tokens/nonces/identity; macro expansion remains behind canonical Host/Runner argument, quoting, approval and sandbox boundaries; text rendering must validate/bound links and escape entities under Transcript/Resource safe-link policy.

Existing-owner-first maps fuzzy scoring to canonical Search, localized JSON to Localization, shell-aware expansion to Host/Runner/Computer, and linkification/text projection to Transcript/Resource. Static Qt plugin registration remains non-shipping implementation detail. Rows remain mapped-open / unknown_closed=false until production and same-head evidence close them.

Accounting: **9,400/16,125 read; 6,725 unread; 15,845 unknown; 0 omitted**. Next 9,401 `Telegram/ThirdParty/kcoreaddons::src/lib/text/ktexttohtml_p.h@93bbb72f1eeb0adc8c2ec57ebc6ab5c3309d197d`.


### Revision 9 deterministic read-through — licenses and plugin lifecycle 9,361–9,380

Orders 9,361–9,380 exact-read the KCoreAddons runtime-version contract, installed license corpus, legacy desktop metadata parser, plugin factory/loader and typed plugin metadata. Applicable semantics include runtime-vs-build provenance, exact third-party license availability, strict typed metadata conversion, service-type/path resolution with malformed-input failure, concurrency-safe parser caching, stable plugin/source identity, load/error/lifecycle separation, MIME inheritance, default-enable state, localized metadata fallback and author/license/version provenance.

Existing-owner-first audit found stronger shipping equivalents in `source/packages/cursor-plugins/manifest-parser.ts` and Mahayana `core-plugins`: manifest size is bounded, plugin/schema/path/id data is typed, invalid/duplicate entries are fenced, variables are recursively validated, and loader/store/cache/version lifecycle stays under the canonical Marketplace owner. Therefore no KDE/Qt plugin runtime or second schema truth is introduced. These rows remain `mapped-open` / `unknown_closed=false` until full applicability and same-head production evidence close them.

Accounting: **9,380/16,125 read; 6,745 unread; 15,845 unknown; 0 omitted**. Next 9,381 `Telegram/ThirdParty/kcoreaddons::src/lib/plugin/kstaticpluginhelpers.cpp@1bf196d1b85d1969f0fd70a10e5023b4a4a5522c`.


### Revision 9 deterministic read-through — portal/job/provenance 9,345–9,360

Orders 9,345–9,360 exact-read the XDG file-transfer/KIO-fuse interfaces, KJob/KCompositeJob/tracker/UI-delegate lifecycle, and KAboutData/core version source. Applicable product semantics include explicit portal Start/Add/Retrieve/Stop/closed lifecycle; bounded resource ownership; exactly-once task terminal signals; kill/suspend/resume capabilities; first-error propagation; explicit subtask ownership; progress/speed observability; and typed SPDX/component provenance distinct from Fabushi product identity.

Existing-owner-first maps these to canonical Resource/attachment + platform portal, Task/Coordinator + Task UI, and product identity/NOTICE owners. No KDE Job framework, KDE UI delegate, KDE product identity or DBus runtime is introduced as a second owner. Rows remain mapped-open until implementation/evidence obligations close.

Accounting: **9,360/16,125 read; 6,765 unread; 15,845 unknown; 0 omitted**. Next 9,361 `Telegram/ThirdParty/kcoreaddons::src/lib/kcoreaddons.h@77b482d1b75330c1e38908a7d9c974fced309dd7`.


### Revision 9 deterministic read-through — kcoreaddons production source 9,309–9,344

Orders 9,309–9,344 exact-read the first 36 blobs of pinned `KDE/kcoreaddons/src`. The window covers build and localization extraction, strict desktop-metadata → typed JSON conversion, shared derived-cache versioning/eviction/corruption recovery and process-safe locking, stale autosave/backup ownership, create/delete/recreate/move-aware filesystem watching, filesystem/network-mount classification, collision-safe file naming, typed diagnostics, child-process argv/shell/environment/timeout semantics, and URL/attachment transfer through local/portal boundaries.

Existing-owner-first mapping keeps Fabushi build/packaging, Plugins/MCP/Marketplace, Persistence/Recovery, Resource/platform, Settings, Host/Runner/Computer and attachment owners canonical. The recently added Marketplace cache-hit recovery closes only a narrow corrupted-derived-cache behavior and is not used to overclaim the broader shared-cache rows. No KDE/Qt runtime, second cache truth, second process owner, second Settings owner or second attachment owner is introduced. All rows remain `mapped-open` / `unknown_closed=false` until their production/evidence obligations close.

Accounting is **9,344/16,125 read; 6,781 unread; 15,845 unknown; 0 omitted**. Next is 9,345 `Telegram/ThirdParty/kcoreaddons::src/lib/io/kurlmimedata.h@15bd05e4bc36d6c5b377029c695bb54b876f2332`. Fresh descendant exact-head GitHub Actions are mandatory.


### Revision 9 deterministic read-through — kcoreaddons localization resources 9,202–9,308

All 107 `poqm/*/kcoreaddons5_qt.po` blobs from pinned `KDE/kcoreaddons@fd84da51b554eac25e35b1e3f373edaab3029b15` were read by exact blob identity. The common live catalog shape is 133 msgids / 132 contexts / 6 plural entries, but English is intentionally a six-entry plural shim and the four Serbian variants are sparse 117-msgid catalogs. Canonical localization therefore must preserve locale/context/plural/UTF-8/placeholder semantics while supporting missing-category and missing-entry fallback; fuzzy/obsolete translation history is evidence only and must not become shipping copy.

Existing-owner audit finds the shipping owner in `frontend/src/recovered/features/settings/overlay/localization.ts`, consumed by the real Settings panels. This descendant keeps that single owner and strengthens the live accessibility-count projection with locale-safe `Intl.PluralRules` category selection, category-to-`other` fallback and invalid-locale/non-finite-count fail-safe behavior. No KDE/gettext runtime, second localization store or KDE product wording is introduced. The broader rows remain `mapped-open` / `unknown_closed=false` pending complete production evidence.

Accounting is **9,308/16,125 read; 6,817 unread; 15,845 unknown; 0 omitted**. Next is 9,309 `Telegram/ThirdParty/kcoreaddons::src/CMakeLists.txt@1c15425347a9ffc4e12bbd63797616337c78fe33`. Fresh descendant exact-head GitHub Actions are mandatory.


### Revision 9 deterministic read-through — kcoreaddons build/plugin/recovery window 9,077–9,140

The first 64 entries of pinned `KDE/kcoreaddons@fd84da51b554eac25e35b1e3f373edaab3029b15` were exact-read. Applicable responsibilities include reproducible cross-platform dependency/build feature detection; typed legacy desktop-entry → JSON plugin metadata conversion with malformed-schema rejection; stable plugin factory metadata and unload lifecycle; OS release parsing; application/about/SPDX metadata; crash/stale-file/locking autosave recovery; composite asynchronous job parenting/terminality; legacy settings/data migration; and directory watcher create/modify/delete/recreate/move/hardlink/subdir/stop-restart/refcount/thread behavior and performance. License files are retained as legal/provenance inputs under the release-compliance owner, not as runtime product logic.

Existing-owner-first mapping is Plugins/MCP/Marketplace + Host, build/packaging/Actions, Settings/About/Product Identity, Persistence/Storage/Recovery, Task/Coordinator/Host, Settings migration and platform Resource/filesystem-watch owners. Qt/KDE implementation is research/dependency source authority only; no KDE runtime or parallel product truth is introduced. Every row remains `mapped-open`, `unknown_closed=false`, `omitted=false` pending shipping evidence.

Accounting is **9,140/16,125 read; 6,985 unread; 15,845 unknown; 0 omitted**. First unread is order 9,141 `Telegram/ThirdParty/kcoreaddons::autotests/kfileutilstest.cpp@0f65e9d86e4d71e50249b38d5c17546cd94fa2b0`. Fresh descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — Hunspell source tail 9,050–9,076

Orders 9,050–9,076 complete the pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b` recursive source range. Exact reads preserve BOM stripping at affix/dictionary boundaries, supplementary-plane Unicode dictionary/suggestion behavior, Unicode-aware compound minimums, WARN/FORBIDWARN policy semantics, explicit dictionary word-pair disambiguation, and zero-length affix derivation with stable analyze/stem metadata.

This is a **source-reading milestone only**. Every applicable row remains `mapped-open` / `unknown_closed=false` / `omitted=false` until the canonical Fabushi text-input/platform-spellcheck/Resource/security owners have equivalent shipping behavior and same-head evidence. No Hunspell engine or source-specific parallel owner was introduced.

Accounting is **9,076/16,125 read; 7,049 unread; 15,845 unknown; 0 omitted**. The next source is order 9,077 `Telegram/ThirdParty/kcoreaddons::.git-blame-ignore-revs@4fa1a941b387399913902b44873d41a58c87661e` in pinned `KDE/kcoreaddons@fd84da51b554eac25e35b1e3f373edaab3029b15`. Fresh descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — Hunspell suggestion/Unicode/bounded-work fixtures 8,986–9,049

Orders 8,986–9,049 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b` using recursive identities materialized by GitHub Actions run `38107313762`, job `114375483242`, artifact `11689378924` and direct exact GitHub blob/file reads. The range preserves exact affix wildcard conditions; dictionary `ph:` and PHONE phonetic ranking including compound/affix interaction; boundary/space/apostrophe and UTF-8 REP replacement rules; Arabic diacritic plus U+200F normalization; simplified triple-letter compound acceptance; slash/colon tokenization; capitalization, transposition, keyboard-neighbor, multi-word/dash and forbidden-word suggestion ranking; a 4,024-line attributed misspelling corpus with measurable recall/rank harness; fail-closed good/wrong/analyze/stem/suggestion and memory-error execution; adversarial compound time bounds; and UTF-8 affix conditions.

Existing-owner-first mapping is the canonical Composer/text-input/platform spellcheck, Unicode/tokenization, Resource/security, performance and GitHub Actions quality boundary. Upstream fixtures and harnesses are requirements/evidence sources, not a production Hunspell engine. Every row remains `mapped-open`, `unknown_closed=false`, `omitted=false`; no Telegram/Hunspell runtime, second editor, second spell settings owner or second dictionary store was introduced.

Accounting is **9,049/16,125 read; 7,076 unread; 15,845 unknown; 0 omitted**. First unread is order 9,050 `Telegram/ThirdParty/hunspell::tests/utf8_bom.aff@f56998b9f3465bd83bee84a07a7503042f58ae1c`. Fresh descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — Hunspell compound/forbidden/KEEPCASE fixtures 8,953–8,985

Orders 8,953–8,985 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. The range preserves malformed-input no-crash behavior, ONLYINCOMPOUND eligibility, forbidden-word rejection without suggestion leakage, and KEEPCASE casing constraints inside compounds. Existing-owner-first mapping remains the canonical platform spellcheck/text-input/Resource/security boundary; no Hunspell runtime, Telegram provider, second editor, second dictionary truth or source-specific UI is introduced.

Exact-head GitHub Actions run `38107313762`, job `114375483242`, artifact `11689378924` (digest `sha256:34579bcea7de0f6412a68ffb2927b40f5dfec7c3354570f3c01264e44ed5e583`) materialized the accepted recursive inventory at `65338263987558dad9948090fe77318a8964ba46` and fixes the next boundary as order 8,986 `Telegram/ThirdParty/hunspell::tests/pfxdot.aff@e1cdc6186d1785b1572d350c32fee2b0ad6a4187`. The source-authority job failed at the authority validator because lock/index unread counts lagged the newly registered shard; this descendant repairs that accounting without weakening the gate. Fresh descendant exact-head Actions are mandatory.

Accounting is **8,985/16,125 read; 7,140 unread; 15,845 unknown; 0 omitted**. First unread is order 8,986 `Telegram/ThirdParty/hunspell::tests/pfxdot.aff@e1cdc6186d1785b1572d350c32fee2b0ad6a4187`.

### Revision 9 deterministic read-through — Hunspell robustness/morphology regression window 8,749–8,824

Orders 8,749–8,812 were exact raw-byte materialized by GitHub Actions run `38105710749`, job `114370739825`, artifact `11689521875` at `9f6ea014ffa7a33bc21f547a6ad79a4c802c83bd` (artifact digest `sha256:70664b45fadb9af72b43dc45c02602161ce27ef49d4a7032fddc7731ce6ab22b`). The previously unread binary/non-UTF entries were independently checked: order 8,749 `gh-hunzip-overflow.aff.hz` matches Git blob `aa6cb05899ed69bb7b295d915669f79f14df4700` and SHA-256 `6369b66257fe542ae733984937110c8b9f1f6752876cd67ead2c20b7bb8d1640`; order 8,766 `gh1044.aff` matches Git blob `e49cfdd84a9c566501770f1c816a6d330c3bd407` and SHA-256 `d77119833b6d1d9a7b03a1469ccd0aa6dafe02fa44a22a2d93991b95980b5ea2`. Orders 8,813–8,824 were exact-read directly from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`.

The responsibility groups cover fail-closed compressed/affix parsing, ispell command-vs-text framing, OCONV/suggestion correctness, morphology alias robustness, dynamic dictionary add/remove ownership, UTF-8 case derivation, no-self-suggestion, prefix/circumfix/FULLSTRIP/NEEDAFFIX semantics, bounded REP/regex/table handling, slash-as-WORDCHARS parsing, AF/AM alias round-trip, homonym-aware morphology rewriting, Persian/Arabic digit tokenization, derivational stem non-duplication, bounded n-gram work, UTF-8 affix conditions, and explicit personal-dictionary path independence from HOME/USERPROFILE.

Existing-owner-first remains mandatory: parser/UAF/overflow/fuzz responsibilities map to the platform spellcheck dependency + security/Resource boundary; text behavior maps to the canonical text-input/spellcheck Unicode/morphology/suggestion owner; explicit dictionary path semantics map to canonical Resource/settings. No Hunspell engine, Telegram provider, sidecar, second editor, second spell settings owner, or second dictionary store is introduced. The `gh1058` responsibility is implemented on the current branch by filtering a native suggestion equal to the normalized misspelled word before canonical context-menu projection; the rest remain mapped-open pending source-neutral production/replacement evidence.

Accounting is **8,824/16,125 read; 7,301 unread; 15,845 unknown; 0 omitted**. First unread is order 8,825 `Telegram/ThirdParty/hunspell::tests/hu.aff@8345e113130a8aa47dc8fa8e759443eb9f06bdf4`.


### Revision 9 deterministic read-through — Hunspell forbidden/case/full-strip/German compounding 8,730–8,748

Orders 8,730–8,748 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`, with repository/path/blob identities cross-checked against the source-authority 8,730–8,793 manifest from GitHub Actions run `38104688586`, job `114367624265`. The range completes the FORBIDDENWORD dictionary and good/wrong oracle: forbidden roots propagate to suffixed/compound forms while permitted homonyms remain valid. It also fixes FORCEUCASE behavior for compound members and suggestions, FULLSTRIP whole-root affix removal, and current/legacy German compounding requirements covering compound begin/middle/end roles, Fuge elements, ONLYINCOMPOUND/COMPOUNDPERMITFLAG, case-boundary rejection, dash composition, forbidden exceptions, and legacy circumfix positioning.

These are source responsibilities, not evidence that Fabushi now embeds Hunspell. They map existing-owner-first to the canonical platform-backed text-input/spellcheck, morphology and dictionary boundary and remain `mapped-open`, `unknown_closed=false`, `omitted=false`. The native Electron spelling-suggestion projection implemented on the current PR is a narrower product interaction and does not close these morphology/compound semantics.

Accounting is **8,748/16,125 read; 7,377 unread; 15,845 unknown; 0 omitted**. Order 8,749 `Telegram/ThirdParty/hunspell::tests/gh-hunzip-overflow.aff.hz@aa6cb05899ed69bb7b295d915669f79f14df4700` is intentionally still first-unread: it is a non-UTF-8 compressed regression blob, and the connector's UTF-8 file reader rejected it. The adjacent harness proves the responsibility is malformed compressed-dictionary input must not crash, but that does not justify falsely marking the binary blob itself read; raw-byte evidence is still required.


### Revision 9 deterministic read-through — Hunspell compound flags, conditions, casing and continuation grammar 8,666–8,729

Orders 8,666–8,729 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. The range completes the next COMPOUNDRULE sequence/repetition/optionality corpus; proves equivalent ordinal grammar under base, long and numeric flag encodings; covers digit, decimal, percent and per-mille composition with stable analyze/stem metadata; exercises legacy ISO-8859-2 and UTF-8 affix-condition character classes and negation; and preserves continuation-class gating for derived prefixes plus homonym morphology.

The range also fixes product requirements for digit-and-hyphen word compounds, Turkish dotted/dotless-I casing, ISO-8859-15 oe-ligature case conversion, semantic parity across single/long/numeric/UTF-8 flag formats, and Swedish linking-morpheme composition through COMPOUNDBEGIN/ONLYINCOMPOUND/COMPOUNDPERMITFLAG. Order 8,729 begins FORBIDDENWORD configuration semantics; its dictionary and good/wrong oracles remain the next unread files rather than being guessed across the deterministic boundary. No Hunspell/Telegram side runtime, second input truth or second dictionary truth was introduced.

All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; source reading therefore reduces only unread. Accounting is **8,729/16,125 read; 7,396 unread; 15,845 unknown; 0 omitted**. Next is 8,730 `Telegram/ThirdParty/hunspell::tests/forbiddenword.dic@a6b69bea0b713dfc5ff5e7d7e6d2d33ef8363ebb`. The exact-head `b1ab6c8e…` workflows were fully green and bound the source window, but become predecessor evidence after this descendant commit; fresh descendant exact-head GitHub Actions are mandatory before acceptance.

### Revision 9 deterministic read-through — Hunspell compound/casing/morphology regressions 8,602–8,665

Orders 8,602–8,665 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. They close the remaining CHECKCOMPOUNDREP acceptance oracle, then preserve CHECKCOMPOUNDTRIPLE boundary rejection; German `ß`/SS special capitalization and KEEPCASE behavior in legacy/UTF paths; CIRCUMFIX pairing with comparative/superlative analyze/stem metadata; punctuation WORDCHARS tokenization; two-level COMPLEXPREFIXES including Coptic Unicode; a concrete compound recursion `wnum` overflow/no-crash regression; compound-affix base, permit and forbid rules; COMPOUNDMIN/COMPOUNDFLAG eligibility; COMPOUNDFORBIDFLAG precedence; and the beginning of COMPOUNDRULE exact sequence, repetition and optionality grammar.

The predecessor 8,494–8,601 shard had valid range/accounting content but serialized the row collection under `entries`; the Revision 9 authority validator requires `rows`. This descendant fixes that authority serialization contract rather than weakening the validator. All 64 new rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`.

Accounting is **8,665/16,125 read; 7,460 unread; 15,845 unknown; 0 omitted**. Next is 8,666 `Telegram/ThirdParty/hunspell::tests/compoundrule3.dic@7d07bbc89a80ce1992b566104fa91dad014b54a8`. Fresh descendant exact-head GitHub Actions are mandatory; upstream regression fixtures do not by themselves prove Fabushi production closure.

### Revision 9 deterministic read-through — Hunspell IJ, ABI, aliases, casing, breaks and compound regressions 8,494–8,601

Orders 8,494–8,601 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. The 8,494–8,501 tranche was deliberately re-read rather than inheriting the prior unaccounted observation: every repository/path/blob identity matched, including Dutch IJ case correction, the Automake regression graph, the public C/C++ ABI baseline, and the fail-closed ABI checker whose baseline may be refreshed only after a deliberate SONAME break and which explicitly handles LP64/ILP32 and non-libstdc++ differences.

The remaining rows establish prefix/suffix cross-product behavior; AF/AM morphology aliases and complex prefixes; all-caps, mixed-case, forbidden-case and homonym behavior; UTF-8 and Arabic combining/IGNORE handling; legacy/UTF replacement suggestions including Turkish dotted-I; recursive/default/disabled word-break semantics; compound-case and duplicate-component constraints; CHECKCOMPOUNDPATTERN boundary transformations including language-specific sandhi; and CHECKCOMPOUNDREP rejection, including the regression where a replacement applied at the wrong byte position. These map to existing canonical text-input/spellcheck/tokenization and Build/Release owners; no Hunspell/Telegram parallel runtime or second input/dictionary truth is introduced.

All 108 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **8,601/16,125 read; 7,524 unread; 15,845 unknown; 0 omitted**. Next is 8,602 `Telegram/ThirdParty/hunspell::tests/checkcompoundrep2.good@737578025367c6a45d4ab904aa1c9f18d8265fe3`. Fresh descendant exact-head GitHub Actions are mandatory; upstream fixtures and ABI scripts do not by themselves prove Fabushi production closure.

### Revision 9 deterministic read-through — Hunspell tools, fuzzers and regression fixtures 8,430–8,493

Orders 8,430–8,493 were exact-read from `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. The tools tail adds explicit bounded compressed-dictionary, morphology, parser and personal-dictionary fuzz contracts; a targeted >65,535-byte `add_with_affix` UAF regression; safe temporary-file and hzip framing/error behavior; affix/morph alias conversion; munch/unmunch FULLSTRIP/cross-product/capacity semantics; SET-aware Unicode word-form transforms; and deterministic wordlist-to-dictionary generation. These map to existing canonical spellcheck/Resource/dictionary/Unicode/security/Build owners; Hunspell CLI/script identity is replacement-only.

The regression fixtures are product requirements, not ignorable test data. They lock capitalized German n-gram suggestion behavior in legacy and UTF-8 forms; German homonym, NEEDAFFIX and COMPOUNDRULE semantics; large AF/AM alias-count crash handling including malformed directives; Arabic UTF-8 IGNORE combining-mark/tatweel normalization with affix conditions; three-part compound pattern ordering; flag-qualified compound patterns; and interaction between COMPOUNDRULE and COMPOUNDBEGIN/COMPOUNDEND. Empty `.good` files in the alias-crash cases represent no-crash expectations rather than absence of responsibility.

All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **8,493/16,125 read; 7,632 unread; 15,845 unknown; 0 omitted**. Next is 8,494 `Telegram/ThirdParty/hunspell::tests/IJ.aff@c817c2e9132f389983b698b6f13555669ca0f317`. Fresh descendant exact-head GitHub Actions remain mandatory; upstream tests/fuzzers do not by themselves prove Fabushi production closure.

### Revision 9 deterministic read-through — Hunspell core, parsers and tooling 8,366–8,429

Orders 8,366–8,429 were exact-read from pinned `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. Predecessor source-authority artifact 11686195087 (sha256:5d777a472ca1c07d3ea5b0f31250925802bb1fc3ec508518e22d7c6dc1120aaa) materialized all 64 accepted blobs and fixed path/mode/size/object identity; after this descendant it remains source-identity evidence only, not descendant acceptance evidence.

The core range carries production responsibilities for affix/prefix/suffix and compound state machines, hard suggestion/time bounds, Unicode and legacy-encoding conversion, language-sensitive casing, dictionary/custom-word mutation and persistence, compressed-dictionary failure handling, phonetic/replacement/suggestion ranking, public API lifecycle and ABI/version/visibility contracts. Parser sources add text/HTML/XML/ODF/LaTeX/man/first-field segmentation with bounded state, URL/entity/markup shielding and escaping-safe replacement. Tooling adds deterministic Unicode/resource generators, dictionary discovery and personal-dictionary persistence, secure filename/temp behavior, parser/encoding selection, and fuzz coverage for aff/dic, public spell APIs and compressed dictionaries. Standalone Hunspell CLI/C++ identity remains replacement-only; no second input, dictionary or messaging runtime is introduced.

All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **8,429/16,125 read; 7,696 unread; 15,845 unknown; 0 omitted**. The next source is 8,430 `Telegram/ThirdParty/hunspell::src/tools/hzfuzzer.options@e39560b87eed0e2afe3ed217893d757ec8efb942`. Fresh descendant exact-head GitHub Actions remain mandatory; source reading does not grant production or release credit.

### Revision 9 deterministic read-through — remaining Hunspell localization 8,335–8,365

Orders 8,335–8,365 were exact-read from `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. The remaining locale catalogs preserve the same 65-message spellcheck error/help surface and 12 c-format entries with materially different translation completeness, so they map to canonical localization/Resource fallback, Unicode, formatting-safety and text-input/spellcheck owners rather than locale-specific runtimes. `quot.sed` preserves deterministic Unicode quotation substitution, `remove-potcdate.sin` removes only the first non-semantic POT creation timestamp for reproducible catalogs, and `stamp-po` is a generator freshness sentinel rather than runtime state.

All 31 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **8,365/16,125 read; 7,760 unread; 15,845 unknown; 0 omitted**. The next source is 8,366 `Telegram/ThirdParty/hunspell::src/Makefile.am@45b7703d662c0a1465ebaa17d6f3741972f6184a`, which begins the actual Hunspell production source ownership range. Descendant exact-head GitHub Actions remain required.

### Revision 9 deterministic read-through — Hunspell docs, Windows, fuzzing, gettext 8,258–8,334

Orders 8,258–8,334 were exact-read from `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b` and decomposed existing-owner-first. The manpage range contributes mature spellcheck responsibilities rather than presentation-only documentation: dictionary discovery, personal dictionary persistence, interactive correction and pipe semantics, suggestion/morphology/compound behavior, Unicode and encoding rules, API allocation/free lifecycle, locale behavior, parser modes, and compressed dictionary portability. Source-specific Hunspell CLI/C++ APIs and hzip formats are not retained as product owners.

The MSVC range records Windows Win32/x64/ARM64, Debug/Release, static/DLL runtime, warnings/linking/security settings and binary version metadata; these map to canonical Windows Build/packaging/provenance owners. `oss-fuzz-build.sh` maps core, aff/dic, personal-dictionary, parser, morphology and compressed-dictionary fuzz coverage plus representative corpora into canonical security/fuzz responsibilities.

The gettext range records explicit supported locales, POT/PO/GMO generation and deterministic update/install lifecycle, UTF-8 extraction policy, localized diagnostic/help messages with c-format placeholders, quote/Unicode terminal transforms, partial/fuzzy catalog coverage and fallback expectations. These map to canonical localization/Resource + text-input/spellcheck owners; no source-specific gettext runtime is introduced.

All 77 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Accounting is **8,334/16,125 read; 7,791 unread; 15,845 unknown; 0 omitted**. Next source is 8,335 `Telegram/ThirdParty/hunspell::po/nb_NO.po@8bb372e8c9f2728a81365d13bdfbbf3b892c07e0`. Descendant exact-head GitHub Actions remain required.

### Revision 9 deterministic read-through — Hunspell licenses and portable build macros 8,243–8,257

Current-head authority artifact 11685032724 fixed immutable recursive identity for this window and exposed one predecessor ledger drift: order 8,232 `ChangeLog.O` is blob `a2c712d73bc4f5d8cbfbe5eea5f0696ef9d921dd` with Git-tree size 20,359 bytes, not 21,588. The correction changes no source object, responsibility, disposition or gate; it restores exact tree identity so the existing validator can continue to fail closed.

Orders 8,243–8,257 were then exact-read from `hunspell/hunspell@a698d8b53cab3507d1d619d5fa08a88777abe50b`. The two license files preserve Hunspell tri-license and MySpell redistribution/attribution provenance. The m4 range carries mature source-neutral build responsibilities: C++ standard capability detection; gettext/NLS and PO catalog generation; iconv/Unicode portability; linker and transitive-library/rpath/multilib discovery; deterministic executable lookup; and compiler symbol-visibility capability detection. These map existing-owner-first into canonical NOTICE/provenance, text-input/spellcheck, Unicode/Resource/localization, Build/toolchain/linker/packaging/security owners. No Hunspell or Autoconf product runtime is introduced. Every row remains `mapped-open`, `unknown_closed=false`, `omitted=false`.

The same descendant adds the minimal platform-input bridge demanded by the earlier Hime/fcitx source analysis. macOS reads the OS-owned current input-source identifier; Linux probes installed Fcitx5/IBus providers; Windows and unavailable fields fail closed to typed unknown. Chromium composition events remain the authoritative preedit/focus/lifecycle owner, and native async snapshots are accepted only when scope, composition generation and latest-request fencing still match. No browser locale inference and no second input runtime are introduced.

Accounting is **8,257/16,125 read; 7,868 unread; 15,845 unknown; 0 omitted**. Next source is 8,258 `Telegram/ThirdParty/hunspell::man/Makefile.am@ee37fb3aec4520edab3b32be072d8f9d22f02c64`. Descendant exact-head GitHub Actions remain required.


### Revision 9 deterministic correction — Hime GTAB / GTK native input / runtime 8,096–8,162

Fresh exact-head validation on `77c411df86d5f4314be10e09a5a67478fdbf4262` failed closed at order 8,101: the accepted recursive path was `src/gtk-im/Makefile`, while the recorded row had drifted to `src/gtk_bug_fix.c`. The repair does not relax `tdrp-recursive-inventory-contract.mjs` or `validate-tdrp-revision9-authority.mjs`. Instead it reconstructs the range from the pinned Hime Git tree.

Orders 8,101–8,105 were newly read as the GTK2 input-context branch. Its applicable product semantics are client-window/input-context lifecycle, key press/release routing with printable fallback, preedit start/change/end and typed attributes/cursor, commit, focus in/out, reset, cursor geometry, teardown, exactly-one provider registration and locale-as-default-hint-only. Orders 8,106–8,110 are GTK3 build composition plus four exact mode-`120000` symlinks whose literal targets point back to the GTK2 source; those aliases do not duplicate product responsibility. Orders 8,111–8,162 are re-bound by exact source path/blob identity to their true recursive positions, preserving the already-read responsibility decompositions for resource/config/security/module/phrase/IPC/locale/Anthy behavior.

Hime/GTK/XIM runtime identity remains replacement-only. The canonical `PromptRichTextEditor` DOM composition path truthfully reports focus/composition/preedit lifecycle, but current OS input method, language, width, script variant and candidate state still require a source-neutral native platform adapter and cannot be inferred from locale or browser state.

All rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Correct accounting is **8,162/16,125 read; 7,963 unread; 15,845 unknown; 0 omitted**. The true next source is 8,163 `Telegram/ThirdParty/hime::src/modules/chewing-conf.c@93cc1fd31b57e7b45c1fe272ebd945ed7f6632c5`.

### Revision 9 deterministic read-through — Hime IMdkit/runtime/input catalog 8,077–8,095

Orders 8,077–8,095 were read directly from the pinned `hime-ime/hime@9b3e6f9ab59d1fe4d9de73d3bf0fed7789f921c5` source; each GitHub-returned blob SHA matched accepted recursive inventory. The range closes source reading for typed/nested input-context mutation, input-server capabilities, per-client protocol dispatch/sync/pending queues, client identity/message framing, bounded transport and deterministic disconnect cleanup; then maps Hime's central focus/composition/output/candidate state machine, parse cache, full-width mapping, rendering compatibility, candidate buffer editing/learning, table loading/validation and method-catalog/default-selection semantics into existing source-neutral Fabushi owners.

No XIM, Hime or GTK product runtime is introduced. Existing `PromptRichTextEditor` remains the canonical DOM composition/scope owner; native platform-input method/language/width/script/candidate signals remain open and must be truthful rather than inferred. All rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; source reading alone closes no unknown. Accounting is **8,095/16,125 read; 8,030 unread; 15,845 unknown; 0 omitted**. Next source is 8,096 `Telegram/ThirdParty/hime::src/gtab-tsin-fname.c@59ed9cbeb62f6a0accb99dd2ec097b56a10576d4`. Fresh exact-head GitHub Actions are still required.

### Revision 9 deterministic read-through — Hime pink tail, settings/data lifecycle and IMdkit 8,013–8,076

Exact-head source-authority artifact 11681780325 (sha256:92d496e33e3f5fdd523a0033ce41bcac995dd829643116fda6581650fb38ced8) from `46dfaf8102187cc46824362b143e9ac4f3d4b4b9` materialized the complete 64-entry review window with accepted Git identity, mode, byte length and SHA-256. Orders 8,013–8,023 are the remaining pink method/language/symbol status glyphs and were visually inspected; their artwork/palette is replacement-only. 8,024 is LGPL v2.1 provenance. 8,025–8,047 expose manpage responsibilities for deterministic table transforms, platform diagnostics/lifecycle, script/full-width/input settings, virtual keyboard, notifications, phrase learning/edit/import/export and the input-service contract. Symlink manpages preserve exact alias provenance only.

Orders 8,048–8,060 cover symmetric Settings launcher packaging, localization catalog lifecycle and localized input-setting semantics, a fail-fast static-analysis gate, helper packaging, first-run non-clobbering data initialization, diagnostics, deterministic phrase-data merge and external-editor policy. Fabushi maps these into its single canonical Settings/Resource/platform-input/localization/Build owners rather than copying Hime surfaces.

Orders 8,061–8,076 begin the IMdkit implementation: per-editable input-context identity/focus/preedit/status lifecycle, typed capability/attribute negotiation, connection/auth/open/close and focus/event/sync/commit/reset/preedit/status state machines, endian-aware length/count/padding framing, exactly-one transport selection with fail-closed teardown, and async/sync callback settlement. Hime/XIM/X11 protocol is not retained; applicable mature semantics map to the existing source-neutral platform-input, `PromptRichTextEditor`, networking/serialization, Settings and recovery owners. Every row remains `mapped-open`, `unknown_closed=false`, `omitted=false`.

The predecessor `46dfaf8102187cc46824362b143e9ac4f3d4b4b9` required workflows are fully green (Rust desktop runtime 38084031087; Desktop Chat Parity CI 38084031095), with source authority artifact 11681780325, rust messaging artifact 11681353908 and Playwright artifact 11682075159. After main reconciliation and this accounting commit those are predecessor evidence only; fresh descendant exact-head evidence is mandatory. Accounting is **8,076/16,125 read; 8,049 unread; 15,845 unknown; 0 omitted**. Next source is 8,077 `Telegram/ThirdParty/hime::src/IMdkit/lib/i18nIc.c@61b576f996efb6d4964a776e5a6ea5ca78074ddb`.

### Revision 9 deterministic read-through — Hime gray tail, brand art and pink theme 7,949–8,012

Exact-head source-authority artifact 11681552029 (sha256:83a3108678dafcdda74afec1bcffffb9606f48315623fe24d4b5bf9a1e5d8fd4) from `7d155a0461bfe318da513ada5db7dfb65d71b71f` materialized the full 64-entry review window. All 63 PNGs were visually inspected and order 7,975 `icons/pink/COPYING` was text-read. Orders 7,949–7,972 finish the gray input-method/status set; 7,973 `icons/hime.png` is 64×64 Hime mascot/brand artwork and is replacement-only; 7,974–8,012 begin the pink theme copy of the same method/language/width/script semantics. Gray/pink palette choice is presentation, not a second input state owner. The pink full/half × simplified/traditional resources repeat the orthogonal state axes already proven from pinned Hime production source. The exact LGPL text remains provenance-only.

Existing-owner audit still finds the canonical composition lifecycle in `PromptRichTextEditor`. This descendant adds a source-neutral typed input-status snapshot and an accessible Composer status projection driven only by actual DOM composition lifecycle. It deliberately leaves method, language, width and script variant as `unknown` unless a future native platform adapter supplies truthful evidence; browser locale and keyboard guesses are prohibited. This is production progress, not full closure: native OS input-source reporting remains mapped-open and requires focused descendant exact-head evidence.

Accounting is **8,012/16,125 read; 8,113 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 8,013 `Telegram/ThirdParty/hime::icons/pink/scj.png@0d50a486136715f720a4bbc813098b2cce9d83ca`.

### Revision 9 deterministic read-through — Hime gray language/input-mode axes 7,933–7,948

Exact-head source-authority artifact 11680793020 (sha256:b767c2137f6f7c3e0d451197583a45868c7da1c2c33f7eb7e917950ec8ff22ee) from `2b3711e8f0f35d664af5dafe567e2089201f869d` materialized all 16 accepted PNG blobs with exact Git identity, mode, byte length and SHA-256; every PNG was visually inspected. The window spans en-kana-nippon, en-tsin, erbi, Esperanto, ez, fcitx-qxm, fcitx-wanfeng, gdayi3, Greek, Hakka and Hangul identities plus `full-simp`, `full-trad`, `half-simp`, `half-trad` and `halfwidth-kana`. The paired full/half × simplified/traditional resources demonstrate that width mode and script variant are independent typed axes rather than one opaque IME state, while the language/method glyphs demonstrate a separate input-method/language identity axis. Browser locale or a generic composition boolean is therefore not accepted as a production substitute. Hime artwork, palette, labels and branding remain replacement-only; full shipping closure requires a real platform-input signal feeding canonical Resource/design-system/Composer projection with accessible text, without a second input runtime. All rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; source reading closes no unknown. Accounting is **7,948/16,125 read; 8,177 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,949 `Telegram/ThirdParty/hime::icons/gray/hime-tray.png@040374994c12a7f448018f3dba80ee768d2351e7`. Descendant exact-head GitHub Actions remain required after the accounting commit.

### Revision 9 deterministic read-through — Hime dark-tail and gray input-status icons 7,917–7,932

Exact-head source-authority artifact 11680279359 (sha256:8a11aad8a061722e2f1a7024cb5fd3bdbfb3bfdaec01e2de86c9ab04f1d6ca82) from `11caed45b8b0a4e8215a6d93cc814b9f048c95a0` materialized all 16 accepted entries with a window manifest carrying Git blob identity, mode, byte length and SHA-256. The 15 PNGs were visually inspected: orders 7,917–7,922 finish dark Telecode/Tsin/Tsou/Vims/wm2/Wubi, and 7,923 plus 7,925–7,932 begin the gray 4corner/NewCJ3/Amis/array/Chewing/Cangjie/Dayi status set. Order 7,924 `icons/gray/COPYING` is the exact 151-byte text, Copyright (C) 2011 Lu, Chao-Ming (Tetralet), GNU LGPL v2.1 or later. These rows preserve typed source-neutral input-method/language/mode identity, contrast/status meaning and deterministic resource provenance; Hime artwork, palette, names and branding remain replacement-only, while the license row maps only to canonical NOTICE/license accounting. All rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`; reading establishes no shipping verification and closes no unknown. Accounting is **7,932/16,125 read; 8,193 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,933 `Telegram/ThirdParty/hime::icons/gray/en-kana-nippon.png@915bba6d50df2c8bbd2fb1cbf17124220cc043b1`. The predecessor `11caed45b8b0a4e8215a6d93cc814b9f048c95a0` required workflows are completed/success; this descendant accounting commit requires fresh exact-head GitHub Actions.

### Revision 9 deterministic read-through — Hime blue-tail and dark input-status icons 7,869–7,916

The accepted `hime-ime/hime@9b3e6f9ab59d1fe4d9de73d3bf0fed7789f921c5` blobs for 7,869–7,916 were fetched directly and matched against the pinned Git-tree identity/mode/size. PNGs were visually inspected; 7,874 `icons/dark/COPYING` was read as text and preserves Copyright (C) 2011 Lu, Chao-Ming (Tetralet), GNU LGPL v2.1 or later. The window finishes blue Tsou/Vims/wm2/Wubi and continues the dark input-method/language/status set through Taiwan. Repeated full/half × simplified/traditional variants confirm independent width-mode and script-variant axes across themes. Theme artwork, palette, branding and filenames remain replacement-only; typed state stays with canonical Resource/design-system/platform-input owners with accessible non-glyph alternatives, while COPYING maps to canonical NOTICE/license accounting. Reading closes no unknown. Accounting is **7,916/16,125 read; 8,209 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,917 `Telegram/ThirdParty/hime::icons/dark/telecode.png`. Descendant exact-head GitHub Actions remain required.

### Revision 9 deterministic read-through — Hime black language/method/symbol icons 7,805–7,820

Current-head artifact 11680830198 (sha256:e05eecc728d2b69fdbf944ef0d600354c698197730832fb9ef7120629f7d1b2a) materialized the whole window; archive digest, Git blob SHA and byte lengths matched accepted Hime authority. Visual review confirmed Latin letters, NewCJ, noseeing, Paiwan, Pinyin, Russian, Sakura, SCJ, Shuangpin, Simplex, Symbols, Taiwan, Telecode, Tsin, Tsou and Vims glyphs. These encode typed language/input-method/symbol-mode identity and high-contrast status projection; Hime artwork/names remain replacement-only. Applicable semantics stay with canonical Resource/design-system/platform-input owners, with accessible alternatives rather than glyph-only meaning. Reading closes no unknown. Accounting is **7,820/16,125 read; 8,305 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,821 `Telegram/ThirdParty/hime::icons/black/wm2.png`.

### Revision 9 deterministic read-through — Hime black input-mode/status icons 7,789–7,804

Current-head artifact 11680630074 (sha256:472fbfbab38fa8b5dbadf9a3aafa2778b24fa0ac0ce8bbfb11bc04398e383581) materialized the whole window; archive digest, Git blob SHA and byte lengths matched accepted Hime authority. Visual inspection confirmed fcitx Wanfeng, full/half simplified/traditional, gDayi, Greek, Hakka, half-width Kana, Hangul, tray/service presence, internal-code, JTCJ GB, Zhuyin, Jyutping and Kana Nippon glyphs. The full/half × simplified/traditional quartet is an explicit state-model signal: width and script variant are independent typed axes, not one opaque icon enum. Hime artwork/names remain replacement-only; applicable semantics belong to canonical Resource/design-system/platform-input owners with accessible non-glyph-only projection. Reading closes no unknown. Accounting is **7,804/16,125 read; 8,321 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,805 `Telegram/ThirdParty/hime::icons/black/latin-letters.png`.

### Revision 9 deterministic read-through — Hime black input-mode icons 7,773–7,788

Exact-head artifact 11680335933 (sha256:e425c4b675c000fe5effe8170f9b294272be66606acab7c380b7d7ca7eed17bf) materialized the whole window; all Git blob SHA/size values were recomputed and matched. Visual inspection confirmed 30×30 high-contrast black glyphs for distinct input methods/modes (four-corner, NewCJ3, Amis, array/row-column, Chewing, Cangjie variants, Dayi, Kana/Tsin, Erbi, Esperanto, EZ and fcitx-qxm). Those glyphs carry typed mode/status identity and contrast semantics but Hime artwork/names are replacement-only. Order 7,774 establishes copyright Lu, Chao-Ming (Tetralet) and LGPL v2.1-or-later provenance for the black icon set. Reading closes no unknown. Accounting is **7,788/16,125 read; 8,337 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,789 `Telegram/ThirdParty/hime::icons/black/fcitx-wanfeng.png`.

### Revision 9 deterministic read-through — Hime quality gate, license, icon alias and packaging 7,769–7,772

Order 7,769 is a strict fail-fast pre-commit formatting gate. 7,770 is GPLv2 provenance. 7,771 is explicitly a Git symlink blob, mode `120000`, exact content `blue`; it records alias/build-composition intent only and does not inherit target image semantics before those assets are read. 7,772 installs default plus black/dark/gray/pink icon sets and uninstalls the same resources/directories symmetrically. Hime artwork/branding is replacement-only. Reading closes no unknown. Accounting is **7,772/16,125 read; 8,353 unread; 15,845 unknown; 0 omitted**. Next source 7,773 `Telegram/ThirdParty/hime::icons/black/4corner.png` requires exact binary plus visual review.

### Revision 9 deterministic read-through — Hime Qt input test, distro lifecycle, and output transforms 7,743–7,768

Orders 7,743–7,747 are a minimal Qt5 line-edit compatibility harness; Qt/Hime UI is replacement-only, while standard editable-field platform-input compatibility belongs to canonical focused tests. Order 7,748 validates release version text, refuses conflicting archives, strips VCS metadata, normalizes archive ownership and creates the license-sensitive DFSG variant. Order 7,749 isolates locale/module/table paths and XIM identity, regenerates module registries, rejects a non-executable filter and cleans generated registries/socket/temp state. Orders 7,750–7,757 preserve distribution dependency, install/upgrade/uninstall registration/cache lifecycle and idempotent status-surface configuration without retaining distro/Hime mechanics. 7,758 is FDL provenance; 7,759 installs/uninstalls the declared filter set symmetrically. Orders 7,760–7,768 are deterministic single-input/single-output text transforms: unmapped text passes through, mappings/encoding representability are explicit, and Hime's random branded `nobopomofo` image side effect is replacement-only. Reading closes no unknown. Accounting is **7,768/16,125 read; 8,357 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,769 `Telegram/ThirdParty/hime::git-hooks/pre-commit`.

### Revision 9 deterministic read-through — Hime icon-state source and rxvt input patch 7,727–7,742

Exact-head source-authority artifact 11679359489 (sha256:674f0e23e3393b806ef183ff38aa937b16faeca9261c9c2a7611c2052893284f) materialized 7,727–7,742; all 16 entries matched accepted Hime blob identity, mode/type and byte length. Order 7,727 is a 30×30 layered XCF whose inspected layers encode base/full/half variants across pink/blue/gray/dark treatments plus grayscale/effect layers; 7,728–7,741 are the corresponding 30×30 PNG exports. Hime artwork, palette and branding are replacement-only; the applicable responsibility is typed input-status meaning plus deterministic asset provenance under canonical Resource/design-system/platform-input owners. Order 7,742 is the rxvt-unicode integration patch: key press/release forwarding with fallback, focus lifecycle, input-context lifecycle/cleanup and preedit spot/cursor positioning map to the existing canonical platform-input + Composer/RichText owner, where composition/key/scope fencing already exists. Reading closes no unknown. Accounting is **7,742/16,125 read; 8,383 unread; 15,845 unknown; 0 omitted**. Next deterministic source is 7,743 `Telegram/ThirdParty/hime::distro/dev-tools/qt5-test/Makefile`.

### Revision 9 deterministic read-through — Hime phrase data and distro lifecycle 7,692–7,726

Current-head source-authority review evidence materialized and hash-verified the two >1 MB phrase corpora that ordinary GitHub Contents represented without inline text: 7,692 `s-tsin.src` is 1,422,385 bytes / 42,245 lines / blob `88212bec...`, and 7,699 `tsin.src` is 1,423,794 bytes / 42,297 lines / blob `2848fc07...`. Through 7,701 the remaining Simplex/symbol/Vietnamese/phonetic/T↔S inputs are read; upstream explicitly warns that naïve inverse Simplified→Traditional mapping is not authoritative. Orders 7,702–7,724 cover distro build/package/install/registration lifecycle, optional adapter reconciliation and symmetric alternatives cleanup. Orders 7,725–7,726 establish icon-license provenance and the development icon generator; Hime branding/art is replacement-only. At that historical checkpoint the next source was the binary XCF; it is now closed by the 7,727–7,742 section above. The checkpoint accounting was **7,726/16,125 read; 8,399 unread; 15,845 unknown; 0 omitted**.

### Revision 9 deterministic read-through — Hime license and functional input data 7,660–7,691

Order 7,660 is the exact Array Input Method Public License, materialized by source-authority artifact 11679557131 and verified as the 70,021-byte Git blob `df8a370e9ab8900e8050129628b0f25844c94f71` before two-page render inspection. Orders 7,661–7,691 cover functional Cangjie/multilingual input maps, phonetic keyboard layouts and source tables, phrase/symbol tables, Dayi's distribution-gated data, Hime method-catalog metadata, and the `extr1.c` generated-input filter. They are not presentation-only. No Hime runtime or second input truth is introduced; applicable semantics map to existing platform input, Resource, TextField/Composer, Build/Release, settings diagnostics and License/NOTICE owners. The next deterministic source is >1 MB `data/s-tsin.src`, which requires the exact raw review-window artifact rather than treating an empty GitHub Contents response as source content. Accounting is **7,691/16,125 read; 8,434 unread; 15,845 unknown; 0 omitted**. Reading closes no unknown.

### Revision 9 deterministic read-through — Hime build / generated input data 7,643–7,659

Orders 7,643–7,650 exact-read Hime source-format policy, distro Docker-action entrypoints, multi-distro build workflow, CodeQL/package-build security evidence and generated-artifact hygiene. Orders 7,651–7,656 preserve contributor/license provenance, historical behavior discovery (without treating changelog prose as production proof), fail-fast build/install composition, documented input-method shortcuts, deterministic dependency/platform probing including the Dayi license gate, and source-to-derived input table/keymap provenance. Orders 7,657–7,659 are functional Array30/Array40 mapping resources and are therefore treated as runtime data inputs, not documentation. Their explicit license reference is the next deterministic entry, 7,660 `data/array_license.pdf`; no secondary-source license text is substituted for that unread blob. No Hime/GTK/Qt/XIM runtime or parallel input owner is introduced. Accounting is **7,659/16,125 read; 8,466 unread; 15,845 unknown; 0 omitted**. Reading closes no unknown.

### Revision 9 deterministic read-through — complete fcitx5-qt component 7,428–7,642

The pinned `fcitx/fcitx5-qt@0285a5d18367d8f3af80dab7e4819d5555982340` component has **215 non-directory entries**, all now exact-read and responsibility-decomposed. The final tranche 7,566–7,642 preserves Qt6 service/portal owner fencing, exactly-one input-context creation and pending cleanup, generated DBus provenance, requested-vs-active input diagnostics, quick-phrase UTF-8/line parsing and safe persistence, dirty/save/file-switch/destructive-operation settlement, Qt6 Settings/plugin/shortcut semantics, and focused key-normalization behavior. Qt6→Qt5 symlinks are recorded only as compatibility/build composition and do not duplicate target behavior. Accounting is **7,642/16,125 read; 8,483 unread; 15,845 unknown; 0 omitted**. First unread is **7,643 `Telegram/ThirdParty/hime::.clang-format@8dfce611a060025a35ab44967831b2a8984d5d69`**.


### Revision 9 deterministic read-through — fcitx5-qt widgets + Qt6 input contracts 7,541–7,565

Orders 7,541–7,565 exact-read Qt5 widgets-addons build/configuration factory, Settings save lifecycle, localization helper, shortcut capture/key translation and the first Qt6 DBus/controller/input contracts. Applicable semantics preserve normalized config-plugin keys, exact IID/metadata validation, deterministic single-factory ownership, load/reset/save and async `saveFinished` settlement, shortcut policy/rollback/clear with modifier semantics and 600 ms modifierless settlement, deterministic Unicode/keypad/media key translation, and the same canonical Settings/input lifecycle across Qt5 and Qt6. Toolkit-specific widget/plugin/DBus mechanics do not create a second Settings/Input runtime. Accounting is **7,565/16,125 read; 8,560 unread; 15,845 unknown; 0 omitted**. First unread is **7,566 `Telegram/ThirdParty/fcitx5-qt::qt6/dbusaddons/fcitxqtinputcontextproxy_p.h@226d1917e943bc3801a53b4bb11b5cfecdb39c3d`**.


### Revision 9 deterministic read-through — fcitx5-qt Qt5 canonical platform input 7,526–7,540

Orders 7,526–7,540 exact-read plugin metadata, candidate window, theme/font configuration, hybrid input selection, plugin factory, canonical Qt5 input-context implementation and the real qtkey targets. Applicable semantics preserve non-focus-stealing candidate/preedit interaction, selection/paging, validated theme paths with fallback/hot reload, single active input owner with failover fencing, focus/preedit/reset/surrounding/cursor/virtual-keyboard lifecycle, async key-event window/object liveness fencing, compose fallback and deterministic keypad/navigation/modifier/function/international/media/text mapping. Orders 7,539/7,540 are the formally read targets for Qt4 aliases 7,483/7,484. Fcitx/Qt/QPA/DBus/XKB mechanisms and Fcitx visual identity are not retained. Accounting is **7,540/16,125 read; 8,585 unread; 15,845 unknown; 0 omitted**. First unread is **7,541 `Telegram/ThirdParty/fcitx5-qt::qt5/widgetsaddons/CMakeLists.txt@939af6dc8dc1f698aa2ff0867c5ce2143c280699`**.


### Revision 9 deterministic read-through — fcitx5-qt Qt5 input recovery 7,516–7,525

The Fcitx4 compatibility transport itself is not retained. Applicable recovery semantics are exact service-owner fencing, a second registration check before binding, exactly-one async input-context creation, pending-call cleanup and Destroy-on-teardown; fallback endpoint discovery validates payload shape and daemon/process liveness; private connections have unique identity; owner/socket changes and disconnects clean stale state and immediately rediscover/reconnect to avoid races. These map to existing Fabushi platform input/network-recovery owners. Accounting is **7,525/16,125 read; 8,600 unread; 15,845 unknown; 0 omitted**. First unread is **7,526 `Telegram/ThirdParty/fcitx5-qt::qt5/platforminputcontext/fcitx5.json.in@48a0da5591026516fe6d1e9b2669f628d312624c`**.

### Revision 9 deterministic read-through — fcitx5-qt Qt5 helper 7,506–7,515

The Qt5 GUI-wrapper/helper executable is not a Fabushi product surface. Its retained responsibilities are config path normalization/capability probing, dirty/apply/reset state, asynchronous save fencing, authoritative change notification, visible failure settlement, close-after-success only, and parent/modal continuity under canonical Settings/ProductShell owners. IM-module probing preserves platform/requested/active input implementation observability through existing platform diagnostics. No Fcitx runtime, helper App, second Settings root or second input owner is introduced. Accounting is **7,515/16,125 read; 8,610 unread; 15,845 unknown; 0 omitted**. First unread is **7,516 `Telegram/ThirdParty/fcitx5-qt::qt5/platforminputcontext/CMakeLists.txt@c461f4e4e528fc7c55378d3e40aadec841f0232e`**.

### Revision 9 deterministic read-through — fcitx5-qt Qt5 DBus/control 7,485–7,505

Qt5 build/export rows remain Build/Release/platform evidence. The Qt5 session transport extends the Qt4 lifecycle with explicit input-context-creation failure, virtual-keyboard visibility query/event reconciliation and the same service-owner race fencing. Controller1 adds current/available input methods, layouts/groups, typed config, refresh/reload and capability-gated restart/update operations. Those are platform input Settings responsibilities; Fcitx addon records are not Fabushi Plugin/Marketplace ownership. Mutations require exact-state settlement and stale-request fencing in the existing Settings/platform owner. Accounting is **7,505/16,125 read; 8,620 unread; 15,845 unknown; 0 omitted**. First unread is **7,506 `Telegram/ThirdParty/fcitx5-qt::qt5/guiwrapper/CMakeLists.txt@a459b1b1a362d8d485b7a794f27d07ead7a3bf27`**.

### Revision 9 deterministic read-through — fcitx5-qt Qt4 7,458–7,484

The complete Qt4 subtree is exact-blob read. Qt4 build/export/plugin/probing mechanics map to existing Fabushi Build/Release and thin platform-adapter owners rather than a retained Qt/Fcitx runtime. DBus watcher/proxy/interface rows preserve service-owner recheck, availability/reconnect, exactly-one input-context creation, pending-call cleanup, feature introspection, focus/reset/key/preedit/surrounding/candidate/virtual-keyboard event contracts. The inputcontext rows preserve password/sensitive surrounding-text denial, 4,096-character bound, Unicode code-point cursor/anchor conversion and deletion validation, focus/window liveness, formatted preedit and DontCommit settlement, async key-event fencing, XKB compose/international/media key mapping and teardown behavior. No source-shaped provider or second Composer/TextField owner is introduced. Accounting is **7,484/16,125 read; 8,641 unread; 15,845 unknown; 0 omitted**. First unread is **7,485 `Telegram/ThirdParty/fcitx5-qt::qt5/CMakeLists.txt@a802a509a22e1ca2cf43f42f0c7171cce91ffb7b`**.

### Revision 9 deterministic read-through — fcitx5-qt 7,428–7,457

Orders **7,428–7,438** cover pinned fcitx5-qt formatting, CI, top-level build/dependency discovery, compiler/link visibility/RPATH rules, license/provenance and translation extraction inputs. Order **7,439** is product-applicable input-method behavior: preedit/formatted preedit, password/sensitive and surrounding-text policy, content-type hints, unfocus commit, key order/repeat, text formatting, relative cursor rectangle, candidate layout and client-side input-panel capability. Orders **7,440–7,457** cover translation installation, locale inventory and catalogs for applicable input/settings/quick-phrase/key-sequence actions, destructive confirmations and error/warning feedback. These map to existing source-neutral Build/Release, localization, canonical TextField/Composer, keyboard/input, accessibility/security and platform owners; no Fcitx runtime/provider or second input/UI root is introduced. Reading closes no unknown. Accounting is **7,457/16,125 read; 8,668 unread; 15,845 unknown; 0 omitted**. First unread is **7,458 `Telegram/ThirdParty/fcitx5-qt::qt4/CMakeLists.txt@39a412ba4a1610f47eb1c339686c9d46709ab74c`**.

### Revision 9 deterministic read-through — expected 7,404–7,427

Orders **7,404–7,412** cover pinned component build/CI/package/license/provenance inputs. Order **7,413** is the header-only typed result/error contract: value/error discrimination, construction/assignment, move/copy/triviality, observers, emplacement/swap, void specialization, constexpr/noexcept and monadic map/map_error/and_then/or_else behavior. Orders **7,414–7,427** are focused regression oracles for assertions, assignment/conversion, move-only ownership, constructors, emplace, extensions/issues, observers, relational operations, swap and noexcept/constexpr behavior. Applicable semantics map into existing Rust `Result`/`Option` and TypeScript discriminated-result contracts owned by canonical Coordinator/Host/Runner/domain owners; build/license provenance stays with Build/Release. No source-shaped Expected runtime, Telegram provider, or second Identity/Conversation/Message/Search/Settings owner is introduced.

The predecessor `de430cbdff6fd55b7a527de7c5f7cdca93951a64` correctly failed the Revision 9 validator because order 7,288 `cmark-gfm::bench/stats.py` had a malformed 38-character recorded blob identity. The accepted object is `c244b419f66624bd2d2d2d3cb028d1f65edabc22`; this descendant repairs all three durable identity fields. Accounting is **7,427/16,125 read; 8,698 unread; 15,845 unknown; 0 omitted**. First unread is **7,428 `Telegram/ThirdParty/fcitx5-qt::.clang-format@04b3745d495e42f358142112a044b42c7600794b`**.

# Telegram 源码 → Fabushi 全量等价重写 — Source of Truth

Status: active  
Project ID: TDRP-001  
Revision: 9  
Date: 2026-10-07  
Parent: FBCP-001 Revision 7

## Role

本项目负责完整 `telegramdesktop/tdesktop` 源码及其递归依赖、资源、工具链的逐文件、逐模块理解、责任对照、等价重写、生产接入和 exact-head 取证。它不是 capability research 子项目，也不能以 survey、ledger 填写、局部 vertical slice 或 UI Demo 作为最终完成。

上游是完整产品行为、状态机、边界条件与工程责任的 source authority，不是最终 runtime/network/product。全部 applicable 产品职责必须吸收入 current canonical Fabushi；C++ 非 UI 产品逻辑默认 Rust，UI 使用 Fabushi React/TypeScript 表现层，平台职责采用最薄 adapter。UI 文件中的业务、权限、生命周期、错误处理、键盘与可访问性行为仍然必须迁移。

## Must read

- root `AGENTS.md`
- `projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md` Revision 7
- `docs/specs/fabushi-bot-communication-platform.md` Revision 7
- `docs/specs/telegram-desktop-rust-equivalence-migration.md` Revision 9
- `module-map.md`
- current upstream lock / inventory / ledger / dossiers / validators / quality plans
- current canonical Fabushi shipping owners and GitHub Actions release evidence

较早 Revision 2/4/5 中 research-only、选择性吸收、where accepted、blocked 可算最终验收、htch-runtime 可执行本项目验证等口径全部废止。历史记录只能保留 provenance，不能继承为 implemented/verified。

## Baseline

Historical predecessor authority: 2026-10-09 live rebaseline 后 tdesktop accepted discovery HEAD 为 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e（tree 5030985204963cbbd362ced7412d231b04ebd0cc）；42f8a36d43b8c805bc821905bea4cfeb3af1d41d 以及更早 baseline 仅作历史证据。本次三提交 delta 修改 15 个 root paths（12 modified / 3 added），root non-directory 6,651，recursive denominator 16,123。当前 source closure 仍 open，baseline_ready/acceptance.accepted 仍必须为 false。

accepted baseline 必须在 GitHub Actions 中重新确认，并递归闭合：
- root tracked files / gitlinks；
- nested submodules；
- build-time downloads / patches；
- LFS / external objects（如有）；
- generated-input contracts；
- functional resources / locales / assets；
- build / packaging / updater / tests / tooling / docs / licenses。

只有当 current lock、inventory、ledger、dossiers 对同一 exact upstream commit 一致，且 unknown=0、unread=0、omitted=0，才能把 `baseline_ready` 设为 true。旧 baseline 的 verified 状态不得自动继承。

## Mandatory chain

`source file -> source symbol -> responsibility -> capability -> existing Fabushi owner -> target path/symbol/language -> production entrypoint -> tests -> current-head evidence -> verified`

允许多源文件汇聚到一个高内聚 owner，也允许一个 mixed 文件拆到多个 owner；禁止机械一文件对一文件复制。任何 responsibility 无去向都不能完成。

## Existing-owner-first

每项 responsibility 都必须先审计 current exact-head Fabushi：
- ProductShell / navigation / ConversationList / ConversationWorkspace / Transcript / Composer；
- Participant/Profile / Shared Room/member / Permissions / Identity/account / Contacts / Search；
- Resource / attachments / artifacts / storage / recovery / notifications / settings；
- Coordinator / Host / Runner / Agent；
- Plugins / MCP / Marketplace / Computer / Automations / Tasks；
- networking / platform / packaging / updater。

有合适 existing owner 时必须扩展它。只有 exact-head audit 证明没有 owner，且记录 owner-absence evidence、rejected candidates、最小 state/lifecycle responsibility、commands/events、persistence/concurrency/security/service dependency 并经 ADR 批准后，才能新增 source-neutral 最小 owner。

禁止新增 TelegramCore、TelegramRuntime、TelegramProvider、TelegramMessaging、CommunicationCore、MessengerRuntime 或第二套 Identity / Conversation / Message / Search / Settings / Marketplace。

## Product invariants

- 最终只有一个 Fabushi Desktop 产品。
- Human / Agent / Group / Channel / Hybrid / Topic 共用 canonical Conversation 架构、ConversationWorkspace、Transcript、Composer、Draft、routing 与 resource lifecycle。
- Telegram 的 reply/quote/forward/edit/delete/reaction/poll/scheduled/silent/read/pin/expiry/album 等能力优先升级 canonical Message/Transcript model，不建立来源型 message store。
- Participant / Contacts / Profile 使用统一 framework；联系人发消息进入 canonical Direct Conversation。
- Conversation creation 只有一个 typed `ConversationCreationSurface`。
- Search 只有一个 canonical owner；collection/object/universal search 与 participant/member/admin pickers 复用同一 provider/permission contract。
- 新迁移 UI 只消费 Fabushi semantic tokens 与 canonical components；legacy `sand-*` / `cursor-*` 只能作为兼容实现细节。
- Telegram/MTProto 不进入 shipping runtime；但 ordering/replay/gap/dedupe/idempotency/durability/reconnect/retry/backoff/resume/session/rate-limit/cancellation/security 等成熟责任必须在 Fabushi 自有协议与服务中等价实现。
- 远端 service 缺失只能标 blocked；mock、fixture、UI-only 或 static response 不能算完成。
- 现有 Agent / Host / Coordinator / Runner / Plugins / MCP / Marketplace / Computer / Automations / Task / attachments / approvals / recovery / account / packaging/update 不得回退。

## Verification

所有 FBCP/TDRP 可执行 build/test/lint/generator/schema/benchmark/fuzz/E2E/visual/a11y/performance/soak/fault/package/acceptance 只允许 GitHub Actions。

每个 responsibility 的 Definition of Done 同时要求：
- production implementation 已进入唯一 shipping composition；
- requirement/oracle/invariant/test 可追溯；
- exact-head GitHub Actions evidence；
- 无 flaky/skipped/blocked/not-run 冒充 pass；
- packaged temporal acceptance 验证 terminal result 在 quiet window、切换、reconnect、reload、restart 后不消失、不回滚、不被 intermediate 替代。

release candidate 只有在 independent acceptance 为 ACCEPT、0 open P0/P1/blocker、0 omitted/unmapped、0 stub/fake fallback 后才可发布。packaged acceptance 必须 fail closed；不得用 `continue-on-error`、跳过步骤或软化 assertion 绕过。

## Current status

迁移尚未完成。当前 durable specs 已升级到 FBCP Revision 7 / TDRP Revision 9，但历史 lock / inventory / ledger / dossiers / STATUS / validators 中仍可能包含旧 Revision 4/5 语义，必须逐项 rebaseline/reconcile，不能据此宣称 source completeness 或 production completeness。

每个模块继续执行 `unreviewed -> understood -> mapped -> implemented -> verified` 的真实证据路径。遇到 service/account/signing blocker，记录解除条件并继续推进所有不依赖该 blocker 的 responsibility。

Historical predecessor authority: Current live authority (2026-10-09): `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` (root tree `5030985204963cbbd362ced7412d231b04ebd0cc`), three commits ahead of historical `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`; 15 root paths changed (12 modified, 3 added), recursive denominator is 16,123, read-through is 5,773, unread is 10,350, unknown is 15,844, omitted is 0, and source closure remains open.

Current source accounting: deterministic read-through `5599/16123`; unread `10524`; unknown `15844`; unknown-closed `279`; omitted `0`. Orders 5001-5599 are exact-blob read-complete. Media-view/menu responsibilities and MTProto-derived transport/session/auth/config/security/proxy/error/schema/reconnect/bootstrap responsibilities remain mapped-open except for explicitly cited existing partial Fabushi slices; MTProto wire/socket/DC mechanics are source-neutral platform/protocol replacements, not a second runtime and not omitted. Unknown stays unchanged until complete responsibility and exact-head verification gates close; no baseline-ready or release credit is granted.


### Read-through 5291-5300

Orders 5291-5300 are exact-blob read-complete at upstream `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`. WebProxy transport/WebView invariants are mapped-open to existing Coordinator/Host/backend/desktop-bridge security, lifecycle, bounded-backpressure and recovery owners; no MTProto/WebProxy runtime is introduced. Overview selection/media layout responsibilities are mapped-open to canonical Message/SearchIndex/MediaCache, Conversation/shared-media collection, MediaViewer/Resource and design-system Checkbox/ResultRow owners; no Telegram/Overview UI root is introduced. Unknown remains 15,841 and omitted remains 0 until production + exact-head evidence closes each responsibility.


### Read-through 5301-5310

Orders 5301-5310 are exact-blob read-complete at upstream `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`. Passport-derived secure authorization responsibilities are mapped-open to current source-neutral account/identity, provider authorization/OAuth, credential/security, recovery, resource upload and canonical form owners. Telegram Passport protocol/crypto/UI is not introduced as a second runtime. Unknown remains 15,841 and omitted remains 0.


### Read-through 5311-5320

Orders 5311-5320 are exact-blob read-complete. Secure form/panel/scan responsibilities remain mapped-open to canonical account/identity, provider authorization, attachment/resource, recovery, form-validation and design-system owners. No Telegram Passport UI/runtime is introduced. Unknown remains 15,841; omitted remains 0.


### Read-through 5321-5330

Orders 5321-5330 are exact-blob read-complete. Passport detail/form/password responsibilities plus checkout lifecycle are mapped-open to canonical account/identity/provider authorization, attachment/resource, form validation/design-system and existing payment/payment-provider owners. No Passport or Telegram payment runtime is introduced. Unknown remains 15,841; omitted remains 0.

### Read-through 5331-5340

Orders 5331-5340 are exact-blob read-complete. Payment orchestration, balance/reaction fencing and provider tokenization responsibilities are mapped-open to native Mahayana payment/payment_provider/wallet/service, canonical auth/external-URL/security, and production reaction owners. Telegram Stars/MTProto/SmartGlocal runtime is not introduced. Unknown remains 15,841; omitted remains 0.

### Read-through 5341-5350

Orders 5341-5350 are exact-blob read-complete. Provider card/token/error decoding and Stripe request lifecycle responsibilities are mapped-open to canonical payment-provider/security/error owners; provider wire implementations are source-specific and are not copied. Unknown remains 15,841; omitted remains 0.


### Read-through 5351-5360

Orders 5351-5360 are exact-blob read-complete. Stripe card metadata/input validation/decode/error responsibilities are mapped-open to canonical PaymentProvider/payment, form-validation, security and error-projection owners. Stripe SDK/wire behavior is not copied as a second runtime. Unknown remains 15,841; omitted remains 0.


### Read-through 5361-5370

Orders 5361-5370 are exact-blob read-complete. Provider form encoding/configuration/token decode and checkout card-editor behavior map to canonical PaymentProvider/payment, security, form-validation and canonical payment UI owners. No Stripe runtime or Stripe-specific UI root is introduced. Unknown remains 15,841; omitted remains 0.

### Read-through 5371-5380

Orders 5371-5380 are exact-blob read-complete. Requested customer information, payment-field normalization, checkout summary and panel orchestration map to canonical payment, form, webview/security and command owners. Commit `39da06252e33ef7f8d53061514f8234108ee609e` adds fail-closed source-neutral invoice-requested customer validation before charging plus a focused Rust contract; exact-head Actions and remaining UI/provider responsibilities stay open. Unknown remains 15,841; omitted remains 0.

### Read-through 5381-5390

Orders 5381-5390 are exact-blob read-complete. Paid-reaction amount/identity/anonymity/balance maps to canonical Reaction/Wallet/payment/Dialog; Linux location, XDG Open-With, activation/sleep/lock/theme and protected update/relaunch map to existing platform and update/packaging owners. Unknown remains 15,841; omitted remains 0.

### Read-through 5391-5400

Orders 5391-5400 are exact-blob read-complete. Global menu/unread badge/focus state, native notification capability negotiation/actions/inline reply/activation-token/exact-scope cleanup, DBus/Flatpak schemas, autostart/single-instance/scheme launch and platform capability deltas map to existing application-menu, notification, deep-link, lifecycle, settings and packaging owners. Native Linux services remain thin adapters. Unknown remains 15,841; omitted remains 0.


### Read-through 5401-5410

Orders 5401-5410 are exact-blob read-complete. Linux text-recognition availability, external translation-provider process lifecycle, tray icon/menu/cache behavior, protected updater security, WebAuthn passkey routing and macOS authorization-aware location/reverse-geocode behavior map to existing source-neutral capability/platform owners. The protected updater remains high-risk mapped-open until trusted-path/package/staging/privilege/post-install verification obtains exact-head evidence. Unknown remains 15,841; omitted remains 0.

### Read-through 5411-5475

Orders 5411-5475 are exact-blob read-complete at accepted upstream `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` after correcting deterministic order 5464-5475 against the live root tree. The range contains macOS platform responsibilities through order 5463, followed by shared `platform_*` contracts for current location/reverse geocode, file bookmarks/utilities/dialogs/external launch, Integration composition, launcher/MainWindow dispatch, notification policy, MediaViewer overlay/window controls and platform lifecycle/permission/security settings. Windows-specific files begin at order 5481 and are not credited by this range. All responsibilities remain mapped-open to existing source-neutral canonical owners until production implementation and same-head evidence close them; reading alone does not reduce unknown. No Telegram-derived parallel runtime or platform-named product UI was introduced. Unknown remains 15,841; omitted remains 0.


### Read-through 5476-5500

Orders 5476-5500 are exact-blob read-complete at accepted upstream `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`. Shared platform text-recognition/translation/tray/WebAuthn/window-title contracts and Windows exact location, file/Open-With/dialog security, lifecycle/power fencing, updater/relaunch parent identity, locked-window privacy, exact-scope native notification actions, screenshot/settings/autostart integration, native OCR/translation capability absence, tray/taskbar behavior and Windows Hello wallet protection have been decomposed and mapped to existing source-neutral canonical owners. Windows native OCR/translation absence is recorded as a capability delta rather than an excuse to invent parallel providers. Existing Fabushi updater and scoped-notification slices are credited only as partial production evidence; inline notification reply/mark-read, locked-content/capture privacy, Windows Hello wallet equivalence and other mapped responsibilities remain open. Unknown remains 15,841; omitted remains 0; reading alone grants no production/release credit.


### Read-through 5501-5529

Orders 5501-5529 are exact-blob read-complete at accepted upstream `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`. Windows Hello registration timing, native WebAuthn/fallback routing, AppUserModelID/shortcut/toast identity, autostart policy, safe dynamic symbols, Quiet Hours, taskbar media controls and toast activation map to canonical security/packaging/settings/media/notification owners. Poll option-link and token-fenced media upload responsibilities expose a current production gap: no canonical poll/survey owner was found, so these rows remain mapped-open rather than being credited to generic Composer attachments. Profile header/block/drop-area presentation maps to canonical ProfileSection/design-system; settings globals map to canonical settings/persistence/security/lifecycle, including passcode retry backoff and working-directory permission semantics. Unknown remains 15,841; omitted remains 0.


### Read-through 5530-5540

Orders 5530-5540 are exact-blob read-complete. Away Message, Chat Intro, Business Chat Links, delegated Chatbot permissions, Greeting and Business Location are applicable product responsibilities rather than Telegram-only UI mechanics. Current Fabushi repository search found no equivalent production owners, so these rows are explicit mapped-open product gaps under source-neutral business automation, conversation onboarding/link/deep-link, Agent/Bot CapabilityBroker and profile/location owners. Elevated delegated-bot permissions require explicit warnings; scheduling/recipient/shortcut limits and map capability/fallback semantics remain fail-closed. Unknown stays 15,841; omitted stays 0.


### Read-through 5541-5571

Orders 5541-5549 are exact-read business quick-reply, recipient-scope, shortcut-message and working-hours responsibilities. They enforce premium/count/name/message limits, include/exclude invariants, per-day interval normalization including next-day ranges, and timezone fallback/loading; no equivalent complete canonical owner has yet been proven. Orders 5550-5571 are the cloud-password/account-security flow: transient StepData, password create/check/change, recovery email, email/login-email codes, password hint constraints, reset/pending-reset/cancel-reset, manage/disable, 10-minute idle expiry, and cross-device password-change invalidation. Repository search did not prove an equivalent complete 2SV/recovery state machine. These remain critical mapped-open security responsibilities; unknown stays 15,841 and omitted stays 0.


### Read-through 5572-5580

Orders 5572-5580 are exact-blob read-complete. Detailed settings rows explicitly carry Switch/checkable accessibility semantics. Active Sessions manages current/incomplete/other authorizations, 60-second refresh, device naming, terminate-one/all and auto-terminate TTL. Advanced settings span source-neutral network/proxy, download/storage, auto-download, title/frame, tray/taskbar/close behavior, autostart, updater, power/archive and platform capability surfaces. Blocked Peers is a canonical privacy list with reactive server-backed state. These rows remain mapped-open until same-head product/security/a11y evidence proves equivalence; unknown remains 15,841.

### Rebaseline 42f8a36d -> 863cf10d

Live root tree `5030985204963cbbd362ced7412d231b04ebd0cc` has 6,651 non-directory entries; recursive total is 16,123. Deterministic read-through is 5,599. First unread is order 5,600 `Telegram/SourceFiles/settings/sections/settings_local_storage.cpp` blob `b5371a40e0dce0d21583ca6a026bcd7df43ab15f`. Unknown is 15,844 and remains fail-closed.

Changed prior-read blobs were re-read at their 863cf10d exact blob identities and re-decomposed. New `bot_menu.tgs` and `history_view_bot_menu_button.cpp/.h` are read-complete but mapped-open to canonical Composer + Button/IconButton + animation/resource owners. No source-named BotMenuButton product component is allowed. `chat.style` (order 5,941) and `wallet_content.cpp` (order 6,314) remain outside deterministic read-through; delta semantics are mapped-open without granting prefix credit.

### Read-through 5590-5593

Orders 5590-5591 are exact `settings_credits.cpp/.h` blobs `dbe0b4cf48e52c30e2066c6ad4181893ed604646` / `56c88e68b16b4c2c49bcbcc846b9a5969dfb4867`. They expose account-scoped Stars/currency balance projection, top-up option loading, subscriptions and transaction history, receipt routing, statistics/gift/affiliate actions, TON/USD presentation and withdrawal availability/action. These remain mapped-open to canonical Wallet/Payments/Credits + billing/history/earnings owners; no Telegram Stars runtime is introduced.

Orders 5592-5593 are exact `settings_folders.cpp/.h` blobs `8c5b9b4d6b7a5b3166c12ac8e20c50775a84e9dd` / `8a2984febbf9753dd4aeb698eb7924d4960ca19a`. They expose canonical conversation-filter create/edit/remove/restore/reorder behavior, shared-chat-list leave suggestions and peer removal, recommended filters and limits, premium-gated color tags with 500 ms debounce plus teardown flush, and vertical/horizontal/tab display preferences. Server mutations are intentionally ordered and locally reconciled; these remain mapped-open to the existing Conversation collection/filter + Settings + account messaging-service owners.

Unknown stays 15,844 and omitted stays 0 because source reading alone does not prove production closure. Deterministic read-through is 5,593/16,123; first unread is order 5,594 `Telegram/SourceFiles/settings/sections/settings_global_ttl.cpp` (`3a22266764ef7fe1496a9fd89d5853afe93bd34a`).

### Read-through 5594-5595

Orders 5594-5595 are exact `settings_global_ttl.cpp/.h` blobs `3a22266764ef7fe1496a9fd89d5853afe93bd34a` / `6484c81b829bb59278f5480034c93d7e4796863c`. This is the global conversation-retention/auto-delete settings contract: server-authoritative default history TTL, first-enable confirmation, predefined/custom periods, eligibility-filtered existing-conversation selection, and per-peer application through independent history-TTL mutations. Self/replies/verify-code/ineligible peers are excluded and current per-peer TTL is projected in the picker.

The responsibility remains mapped-open to canonical Privacy/Retention + Conversation policy + account messaging owners. Production evidence must cover account switching, stale peer eligibility, permission changes, server rejection/retry, bulk partial success/failure, duplicate application, default-vs-per-conversation reconciliation, teardown/cancellation, and keyboard/focus/a11y/light-dark/responsive behavior. Unknown stays 15,844; omitted stays 0. Read-through is 5,595/16,123; first unread is 5,596 `settings_information.cpp`.

### Read-through 5596-5597

Orders 5596-5597 are exact `settings_information.cpp/.h` blobs `8982b515b5a30600f5565c3b7e81dc5218c96950` / `54435faa2f1a8d156d5da3501a9e9b0c13b96bae`. Responsibilities are canonical Identity/Profile and Account projections: profile photo optimistic local preview followed by upload, name/username/phone and copy/edit routing, birthday plus privacy state, personal channel/color, Chat Automation, Bio editing with premium/default limits and one-second debounce plus teardown flush, multi-account add/switch/new-window/logout/reorder/limit handling, and account/premium/unread badge projections.

These remain mapped-open to existing source-neutral Identity/Profile/Account/Business/Privacy owners. Exact-head production evidence must cover failed/stale avatar upload rollback, bio save races and remote replacement, account switch guard/new-window continuity, reorder persistence, locked-account limits, logout fencing, birthday privacy, chatbot/personal-channel account scope, unread/badge state, and keyboard/focus/a11y/light-dark/responsive behavior. Unknown remains 15,844; omitted remains 0. Read-through is 5,597/16,123; first unread is 5,598 `settings_local_passcode.cpp`.

### Read-through 5598-5599

Orders 5598-5599 are exact `settings_local_passcode.cpp/.h` blobs `0f1b6ace690851441a588074cbff816fb3e5a8a1` / `b0292c43e503bc8b7ad2fa66f9681be59c1c272c`. This is a local-device security domain, distinct from Cloud Password/2SV. It retains verified passcode bytes only inside the passcode settings area with a 60-second interaction-reset inactivity expiry, invalidates retained bytes on external passcode changes, performs derivation/verification on worker paths, tracks wrong attempts, and cleanses transient UTF-8 buffers. Create/check/change/manage navigation hands secure bytes only across the immediate settings hop and removes stale passcode sections from navigation history.

Manage semantics include wallet key-protection revalidation/migration before passcode removal, fresh verification for writes that weaken app lock, refusal/stale recovery back to Check, app-lock launch protection, auto-lock settings, and platform system-unlock availability/toggles including Windows Hello, Touch ID, Apple Watch, or system password. These remain mapped-open to canonical Local Security/App Lock + Wallet Key Protection + platform-unlock owners; they do not close Cloud Password/2SV. Unknown remains 15,844; omitted remains 0. Read-through is 5,599/16,123; first unread is 5,600 `settings_local_storage.cpp`.


### Read-through 5600-5604

Orders 5600-5604 are exact `settings_local_storage.cpp/.h/.style` and `settings_main.cpp/.h` blobs. Local Storage owns account-local dual-cache statistics, category/all clearing, device-capacity projection, coupled total/media cache quotas and retention policy; clearing is duplicate-fenced and settles only after both databases plus the minimum presentation interval. Settings Main composes active-account profile plus canonical Notifications/Privacy/Chat/Folders/Advanced/Devices/Power/Language/Wallet/Business/Help routes, add-account/logout, scale confirm/restart, validation/support actions and account-scoped refreshes.

All responsibilities remain mapped-open to source-neutral owners. Production closure must cover failure/cancel/retry/idempotency/account-switch/stale-result/teardown/reload/restart/recovery plus keyboard/focus/a11y/light-dark/responsive/reduced-motion evidence. Unknown remains 15,844; omitted remains 0. Read-through is 5,604/16,123; first unread is 5,605 `settings_notifications.cpp` (`a16c3ee9690de934d795a2803ada0e82c3f9db94`).


### Read-through 5605-5613

Revision 9 validation correctly rejected the transient branch mapping that placed Passkeys at order 5608. Exact recursive order is now restored: 5605–5607 Notifications, 5608–5609 Reactions, 5610–5611 Notification Type, 5612–5613 Passkeys.

Notifications covers global/app policy projection, multi-account cleanup, privacy preview, badge/event/call settings and native/custom platform-manager configuration. Reactions adds server-backed None/Contacts/All scopes for message reactions and poll votes plus preview privacy. Notification Type owns private/group/broadcast defaults for mute, sound/tone/volume and exact peer exceptions. Native notification reply/mark-read/open still requires canonical exact account/conversation/topic-or-sublist/message authorization and exact cleanup; existing scoped click-only OS manager is partial, not closure.

Passkeys is an Account Authentication product flow: server registration challenge → bounded platform authenticator/WebAuthn → server finalize, with unsupported/unsigned/cancel failure closed and exact server credential-id deletion. Fabushi already has remote Agent WebAuthn proxy/signer infrastructure, but that is not evidence of an Account Passkeys UI/list/create/delete owner, so this remains mapped-open.

Accounting: read-through 5,613/16,123; unread 10,510; unknown 15,844; omitted 0. First unread is 5,614 `settings_premium.cpp` (`60ac3cbbfcd4b74eda0d3cbca46c969a22878b94`).

### Current live source accounting after 5614–5616

Accepted upstream remains `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` (tree `5030985204963cbbd362ced7412d231b04ebd0cc`). Deterministic read-through is **5,616 / 16,123**; unread **10,507**; unknown **15,844**; omitted **0**. Orders 5614–5616 remain mapped-open to canonical Entitlement/Subscription, Wallet/Payments/Commerce, Settings and design-system owners. First unread is 5617 `settings_privacy_security.cpp`.

### Read-through 5614–5616

Premium entitlement/subscription/commerce and presentation are exact-blob read-complete and mapped-open; no production closure is credited.

### Read-through 5617–5622

Privacy/Security composition, editable keyboard shortcuts, and server-backed Website/Bot authorization sessions are exact-blob read-complete and mapped-open to existing canonical Account Security/Privacy, Authorization, Command Registry, Bot/Blocked-Peer, Payments/Retention and Settings owners. Exact authorization hash, account/session fencing, destructive confirmation/retry/idempotency and keyboard/a11y behavior remain production gates. Accounting: read-through 5,622; unread 10,501; unknown 15,844; omitted 0. First unread: 5623 `settings/settings_builder.cpp`.

### Read-through 5623–5632

Settings Builder/Common/Search/section-factory infrastructure, hidden diagnostic codes, and Credits/Gift commerce graphics are exact-blob read-complete. UI infrastructure maps to canonical Settings/UniversalSearch/design-system owners; diagnostic codes require controlled development/support policy rather than product copying; Credits/Gift actions remain mapped-open under canonical Wallet/Payments/Credits + Gift/Commerce with exact-account/id, cancel/retry/idempotency/rollback/settlement/restart gates. Accounting: read-through 5,632; unread 10,491; unknown 15,844; omitted 0. First unread 5633 `settings_experimental.cpp`.

### Read-through 5633–5642

Experimental flags/support, FAQ suggestion loading, Settings layer/focus/keyboard navigation, notification-common presentation and Power Saving are exact-blob read-complete. They map to canonical feature-flag policy, Help/Support, Settings/design-system navigation, Notification Settings and Performance/Power owners. No source-named product UI or parallel state owner is introduced. Accounting: read-through 5,642; unread 10,481; unknown 15,844; omitted 0. First unread 5643 `settings_power_saving.h`.

## Deterministic source-order correction at 5,623 and live read-through 5,653

Accepted recursive tree `5030985204963cbbd362ced7412d231b04ebd0cc` proves `Telegram/SourceFiles/settings/settings.style@8b9e56dd…` is blob order **5,623**, between `sections/settings_websites.h` (5,622) and `settings_builder.cpp` (5,624). The earlier 5,623–5,642 labels were therefore off by one and are superseded. Their shard/attestation artifacts are replaced rather than treated as parallel authority.

The corrected contiguous batches are 5,623–5,633 (settings.style through Credits/Gift graphics), 5,634–5,643 (Experimental/FAQ/Settings shell/navigation/Power Saving implementation), and 5,644–5,653 (Power Saving interface, privacy controllers, recent Settings search, scale preview, Settings search/type). Current accounting is **5,653 / 16,123 read**, **10,470 unread**, **15,844 unknown**, **0 omitted**. Reading alone closes no unknown. First unread is 5,654 `Telegram/SourceFiles/statistics/chart_lines_filter_controller.cpp@40c3b612…`.

## Deterministic statistics batch 5,654–5,665

Exact blobs for line-filter animation, ruler generation, interactive chart widget, range min/max segment tree, statistics style/common types and chart JSON deserialization are read/decomposed. They map to canonical Analytics/Insights data projection + reusable Chart/design-system owners; business/account truth must not live in the chart. JSON parse errors, empty columns and column-length mismatches fail closed. Hover/zoom/filter/footer state is derived and lifetime-bounded. Accounting: **5,665/16,123 read; 10,458 unread; 15,844 unknown; 0 omitted**. First unread: 5,666 `statistics_format_values.cpp`.


## Deterministic statistics formatting/export batch 5,666–5,674

Exact blobs for locale/timezone analytics labels, TON/Credits chart graphics, analytics-to-sheet projection, 64-bit chart values, and XLSX/OpenXML serialization are read/decomposed. They map to canonical Analytics/localization, Wallet/Credits visual projection and Export owners. Export must preserve inline-string formula-injection safety, XML/control escaping, legal unique sheet names, empty/short-series behavior, numeric/date boundaries, ZIP failure, cancellation/account fencing and cross-platform save behavior. No Telegram-derived statistics/export runtime or UI is introduced. Accounting: **5,674/16,123 read; 10,449 unread; 15,844 unknown; 0 omitted**. First unread: 5,675 `statistics/view/abstract_chart_view.cpp@4b5f2929…`. Reading alone closes no unknown.


## Deterministic statistics chart-view batch 5,675–5,684

Exact blobs for chart-view abstraction, double-line ratios, bar/stack bars, animated dual-axis/currency rulers, closed chart-type dispatch and linear/DPR cache + pointer hit-testing are read/decomposed. They remain mapped-open to one canonical Analytics/Chart design-system owner. The Fabushi port must strengthen empty/one-point, zero-range/max, all-disabled, mismatched-series, invalid-type and pointer-endpoint cases fail closed; rendering state owns no analytics truth. Accounting: **5,684/16,123 read; 10,439 unread; 15,844 unknown; 0 omitted**. First unread: 5,685 `statistics/view/stack_chart_common.cpp@82a1143f…`.


## Deterministic statistics stack/pie batch 5,685–5,690

The remaining statistics/view blobs are read/decomposed: stack geometry/index mapping, filtered pie-percentage rounding, and stack-linear local zoom/pie transition/selection. They remain mapped-open to the canonical Analytics/Chart owner. Canonical code must fail closed for <2 points, zero-width ranges/denominators, invalid indices/line lengths, zero totals and stale/racing zoom/filter/hover state. Accounting: **5,690/16,123 read; 10,433 unread; 15,844 unknown; 0 omitted**. First unread: 5,691 `statistics/widgets/chart_header_widget.cpp@b8754ceb…`.


## Deterministic statistics widgets batch 5,691–5,696

Statistics widgets are exact-blob read/decomposed: responsive chart header, line-filter controls with last-visible-line guard, and localized/currency point details with zoom/cache/ripple state. They map only to canonical source-neutral Analytics/Chart + existing design-system controls. Empty/mismatched data, invalid indices, RTL/long text, currency precision, a11y/focus/theme/DPR and teardown remain production gates. Accounting: **5,696/16,123 read; 10,427 unread; 15,844 unknown; 0 omitted**. Next order 5,697 must be rebound from the accepted recursive tree before recording its path.


## Deterministic PCH + storage security/migration batch 5,697–5,701

Order 5,697 maps C++ PCH/platform composition to canonical cross-platform build evidence. Orders 5,698–5,701 map critical durable local-file protocol, Argon2id/scrypt/legacy passcode KDFs, encryption/signature/version checks and versioned settings/session migrations to canonical Persistence + Local Security/Key Protection + Settings Migration owners. Invalid/over-cost KDFs, corrupt/truncated/unknown blocks, interrupted writes, wrong passcodes and legacy/account/restart recovery must fail closed. This does not close the existing Local Passcode/App Lock/Wallet key-protection responsibility. Accounting: **5,701/16,123 read; 10,422 unread; 15,844 unknown; 0 omitted**. First unread: 5,702 `storage/download_manager_mtproto.cpp@d0007eb1…`.


### TDRP Revision 9 source read 5702-5711 — storage transfer lifecycle

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,702-5,711 cover adaptive remote download scheduling, generic cache/file loader lifecycle, part-based download/resume, HTTP(S)-only web transfer and multipart media upload. Canonical mapping is the existing source-neutral messaging attachment/file-transfer owner plus platform download/save adapters; MTProto/DC/CDN details are protocol-specific and must not create a Telegram runtime. Required parity includes cancel/teardown/account-switch stale-result fencing, partial resume, integrity/reference refresh, redirect downgrade/scheme protection, TLS/auth/disk failures, adaptive concurrency, removed-session resend, transcode/archive cancellation, upload progress/finalization and restart/recovery. Accounting: 5,711/16,123 read, 10,412 unread, 15,844 unknown, 0 omitted; reading alone closes no unknown. First unread: 5,712 `Telegram/SourceFiles/storage/localimageloader.cpp`.

### TDRP Revision 9 source read 5712-5721 — media preparation and durable serialization

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,712–5,713 cover source-neutral media-send preparation and worker/result lifecycle; 5,714–5,717 cover local settings/theme/language/update persistence, legacy encrypted migration and bounded serialization primitives; 5,718–5,721 cover versioned document/media and user/chat/channel projection persistence with legacy location migration and fail-closed corruption handling. Canonical mapping stays with existing messaging Attachment/media-preparation, Persistence/Settings/Local Security/Updater/Theme/Localization, messaging data persistence, and Identity/Profile/Conversation owners. Telegram/MTP file formats and flags are migration inputs only and must not become a parallel runtime or product UI. Required closure includes cancel/account-switch/teardown stale-result fencing, malformed media/encode/archive failures, wrong-key/tamper/truncation/future-version handling, interrupted atomic writes, legacy migration, cache-vs-authority reconciliation, reload/restart and canonical UI/a11y evidence. Accounting: **5,721/16,123 read, 10,402 unread, 15,844 unknown, 0 omitted**; reading alone closes no unknown. First unread: **5,722 `Telegram/SourceFiles/storage/storage_account.cpp@5888ffa1164327b7170a2a3d8df7793a3fa467da`**.

### TDRP Revision 9 source read 5722-5729 — account/domain storage and local security

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,722–5,723 cover encrypted account-scoped drafts/cache/settings/trust/payment/webview/bot/wallet persistence; 5,724–5,725 cover bounded cloud-blob download/extraction; 5,726–5,727 define passcode wraps, bounded memory-hard derivation, secret cleansing, fresh verification tokens, App Lock mutation, crash-safe committed generations, legacy migration and wallet keyring protection; 5,728–5,729 route shared-media/profile-photo cache queries and invalidation. The local-security rows strengthen but do not close the existing Local Passcode/App Lock/Wallet key-protection responsibility. All remain mapped-open to canonical source-neutral owners; no Telegram storage/security runtime or UI family is permitted. Production evidence must cover wrong/unsupported KDF, stale verification, passcode/App Lock mutation, crash/restart migration, wallet corruption/absence, account fencing, delayed-write teardown, cloud/archive failure, cache invalidation and canonical UI/a11y. Accounting: **5,729/16,123 read, 10,394 unread, 15,844 unknown, 0 omitted**. First unread: **5,730 `Telegram/SourceFiles/storage/storage_file_lock.h@181b5bc063827458c4e59a0daa167355f835ae0b`**.

### TDRP Revision 9 source read 5730-5740 — file locking, archive/media preparation and sparse indexes

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,730–5,732 define exclusive storage locking on POSIX/Windows, including conflict-process handling; 5,733–5,734 define folder/multi-file ZIP preparation with symlink exclusion, progress/cancel, size/disk failure and stale temporary cleanup; 5,735–5,736 cover Composer/editor media drag/drop, MIME/limit/decode and asynchronous preview preparation; 5,737–5,740 define shared-media categories and sparse message-id range merge/query/count/invalidation. All remain mapped-open to canonical Persistence/platform adapters, Attachment/Media Editor and Shared Media/index owners. Required evidence includes cross-platform lock conflict/retry, archive cancellation/limit/disk failure/cleanup, malformed media and stale async results, and sparse pagination/count consistency. Accounting: **5,740/16,123 read, 10,383 unread, 15,844 unknown, 0 omitted**. First unread: **5,741 `Telegram/SourceFiles/storage/storage_user_photos.cpp@232befde40c5edea34286105374058fcc66ab658`**.

### TDRP Revision 9 source read 5741-5754 — user-photo cache, streamed download and operator support

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,741–5,742 cover profile-photo history cache list/count/paging; 5,743–5,744 cover bounded aligned streaming-part download with duplicate suppression, cancellation and completion; 5,745–5,754 cover operator/support template autocomplete/search, contact confirmation, shortcut navigation, occupation leases, support-info editing, history preload, fast-button preferences and template update/reload. Support-specific code is conservatively mapped-open pending explicit applicability evidence rather than silently omitted. Required evidence includes profile count/paging consistency, part failure/cancel/staleness, support account/history fencing, edit request replacement, occupation expiry/race, preload retry and malformed/update-failed template recovery plus canonical keyboard/focus/a11y. Accounting: **5,754/16,123 read, 10,369 unread, 15,844 unknown, 0 omitted**. First unread: **5,755 `Telegram/SourceFiles/tde2e/tde2e_api.cpp@efe4edc5eef735c954b39d7a927722a7e8f9214c`**.

### TDRP Revision 9 source read 5755-5758 — call end-to-end encryption

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,755–5,758 own call E2EE temporary/private key lifecycle, one-shot ECDH envelopes, encrypt/decrypt callbacks, participant/state blocks, inbound/outbound queues, subchain reconciliation, failure terminality and emoji verification hash, with protocol conversion as an adapter. They remain mapped-open to canonical Calls E2EE/Crypto + Call Session owners; transport TLS is not a substitute. This implementation batch also hardens the canonical Wallet ledger so reuse of an idempotency request id must match operation kind, accounts, normalized currency, amount and reference; conflicting credit/transfer/refund reuse fails closed with `RequestConflict` and unit coverage. Broader wallet rows remain open pending full production/evidence closure. Accounting: **5,758/16,123 read, 10,365 unread, 15,844 unknown, 0 omitted**. First unread: **5,759 `Telegram/SourceFiles/test/README.md@429b4f3d83ec6fa6aa71203624a894d1d7973d35`**.

### TDRP Revision 9 source read 5759-5771 — professional task-test harness

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,759–5,771 define the upstream task-test authority: pure bounded readiness; exact object/action identity; assertions outside waits; diagnostic timeouts; precondition-only N/A; packed-scenario teardown; protected fixture secrets; deterministic activation/focus and animation timing; exact dialog-shell interaction; visual capture of real paint owners with DPR/scale boundaries; and privacy-safe clipboard classification/retry without logging foreign clipboard contents. These map to Fabushi's professional cross-platform GitHub Actions test/evidence harness, not to production Telegram UI. The same descendant repairs current-head authority failures by correcting order 5,739 to accepted blob `cc0e3ed6b2c763326c2c16635a7671a747ef506b` and refreshing the strict Grok semantic-adaptation target for `source/electron-main/attachments/attachments.ts` to production blob `9fb1ed46b2ae623e7fa781d6d3ab47f465f4a332`; no gate is weakened. Orders 5,772-5,773 cover the Windows console-lock professional-test contract: the implementation reads process-session/WTS metadata and classifies locked/unlocked/unknown, while the header fixes the public NotApplicable/Unlocked/Locked/Unknown state model, makes `locked()` true only for Locked, and requires `ConsoleLockGate` to remain empty for Unknown and NotApplicable. Only a positive locked reading gates; documented legacy-Windows reversed flags remain unknown; diagnostics never read or print user/domain/window-station fields; non-Windows is explicit N/A. This maps to the canonical professional cross-platform test/evidence harness and creates no production Telegram UI. Accounting: **5,773/16,123 read, 10,350 unread, 15,844 unknown, 0 omitted**. Reading alone closes no unknown. First unread: **5,774 `Telegram/SourceFiles/test/test_corner_patch.cpp@a34785a050cdeea405c3948345704d2e19395843`**.



### cf478d37 live-authority rebaseline

Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@cf478d37c8f57df831cdedb5621e1cf2ef069a0f (root tree fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,793; first unread=5,794 Telegram/SourceFiles/test/test_log.cpp@17c72a8ae1bc672e7c380b855887b859b78fabec; unread=10,332; unknown=15,846; omitted=0. Changed accepted-prefix orders 11/30/31/92/4599/5759/5764/5765 were explicitly re-read; lib_ui is now order 6,589; runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 are historical-only for 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e + b4d9f7f8cf580eefd553c5ce105f1d3e682de87f.

Direct delta: channel_earn.style replaces static negative placeholder margins with floating placeholderShiftLeft; lib_ui 91ff446 implements horizontal interpolation in InputField and MaskedInputField. Canonical TextField remains the sole product owner. Orders 5,774-5,793 were identity/order reconciled; reading alone closes no unknown.


### Revision 9 live read-through note: 5794-5801

Orders 5,794-5,797 preserve professional evidence-log one-line/completion-forgery integrity plus an independent raw-byte oracle. Orders 5,798-5,799 preserve complete mapped-target/viewport capture readiness. Orders 5,800-5,801 preserve the bounded reversible not-marking-read evidence lever. These are test/evidence responsibilities only, create no second product owner, and close no unknown. The new 811b83a1 readability delta is order 5,951 and remains unread/unknown.


### Revision 9 live read-through note: 5802-5805

Orders 5,802-5,803 close the exact-blob read/decomposition of popup/context-menu professional evidence semantics: same-turn fresh-menu identity, explicit refusal taxonomy, isolated QAction queued-callback delivery, prepared-frame capture and lock/teardown behavior. Orders 5,804-5,805 close the SentMessageWatcher client→server id reconciliation contract with stable-history/candidate fencing and a five-second diagnostic probe throttle. These are test/evidence responsibilities only and close no unknown. First unread is now 5,806 `test_notify_override.cpp`.

### Revision 9 live rebaseline/read-through note: b0d1fe5e / 5166 / 5806-5813

The 811b83a1→b0d1fe5e upstream delta changes only order 5,166 `media_view_overlay_widget.cpp`: the old `forbidsSaving()` control-refresh branch hid controls and could call `DocumentSaveClickHandler::Save(...ToCacheOrFile)` when media was not loaded; b0d1fe5e removes that viewer-driven automatic persistence path. The new blob `e35f67119323850a3cea450548566186c8a2231c` was re-read and the mapped-open MediaViewer lifecycle responsibility now explicitly requires protected/forbids-saving media not to be auto-persisted as a rendering/control-refresh side effect. Path order and total counts are unchanged. Separately, orders 5,806–5,813 are read-complete/responsibility-decomposed professional test harnesses for reversible notify overrides/no-server-write locality, document-open handoff inspection without OS launch, SeparatePanel pointer/liveness evidence, and post-paint temporal sampling. They add no product owner, fake fallback, unknown closure or release credit. First unread is 5,814 `test_probe.cpp`; read-through is 5,813/16,125, unread 10,312, unknown 15,846, omitted 0.


### Revision 9 live read-through note: 5814-5821

Orders 5,814–5,821 are exact-blob read-complete/responsibility-decomposed professional evidence harnesses. Probe preserves mark-bounded, keyed issue→answer correlation with explicit ambiguity/outstanding states and positive controls; controlled RPC fixtures reject malformed/truncated/trailing/wrong-constructor buffers before live parser delivery; retry evidence distinguishes 500 retry registration from 400 fail/unregister and unknown-request controls while logging only code/type/constructor; Runner preserves bounded stage/watchdog execution, explicit N/A, QPointer/paint readiness, exactly-once onFinish teardown, disposable-copy markers, completion drain and post-quit fuse behavior. These rows add no product owner or fake fallback and close no global unknown. First unread is 5,822 `Telegram/SourceFiles/test/test_scenario.cpp`; read-through is 5,821/16,125, unread 10,304, unknown 15,846, omitted 0.

### Revision 9 live read-through note: 5822-5824

Orders 5,822–5,824 are exact-blob read-complete/responsibility-decomposed professional evidence code. The checked-in scenario slot is deliberately no-op/test-only. The secrecy oracle is fail-closed over accepted launch identities, controls, bidirectional MTP headers and canaries; distinguishes client-written plain/Send from report-only Recv, withholds unsafe sites, narrowly handles declared computed fields, emits only public evidence and rescans its own rows. Missing/foreign/unreadable/control/canary/input failures are undecided rather than clean. These rows close no global unknown and create no product owner. First unread is 5,825 `Telegram/SourceFiles/test/test_style.cpp`; read-through is 5,824/16,125, unread 10,301, unknown 15,846, omitted 0.

### Revision 9 live read-through note: 5825-5830

Orders 5,825–5,830 are exact-blob read-complete/responsibility-decomposed UI evidence harnesses: stable-window style/baseline verification, real InputField text-drop delivery/refusal/ancestor shielding with no clipboard-content access, and narrowly-scoped U+00A0/U+202F text-read normalization with deliberate negative controls. They create no production owner and close no global unknown. First unread is 5,831 `Telegram/SourceFiles/test/test_toast_capture.cpp`; read-through 5,830/16,125, unread 10,295, unknown 15,846, omitted 0.


### Source closure through 5,848

Under live `65e23ba7` authority, orders **5,837–5,848** were completely read and exact-blob dispositioned. Accounting is now **5,848/16,125 read, 10,277 unread, 15,846 unknown, 0 omitted**; these test/evidence-only rows do not reduce global unknown. Fresh exact-head Actions are required.


### Source closure through 5,859

Under live `65e23ba7` authority, orders **5,849–5,859** are now exact-blob read-complete/responsibility-decomposed. Tests 5,849–5,854 are professional evidence-only responsibilities; Tray 5,855–5,859 remains mapped-open production work pending canonical source-neutral owner/composition and exact-head evidence. Accounting: **5,859/16,125 read, 10,266 unread, 15,846 unknown, 0 omitted**. First unread is 5,860 `ui/boxes/about_cocoon_box.cpp@4aab8e79…`. Unknown is not reduced by reading alone.


### Source closure through 5,871

Orders **5,860–5,871** (`ui/boxes`) are exact-blob read-complete/responsibility-decomposed. They remain mapped-open product/UI responsibilities; reading does not reduce unknown. Current accounting: **5,871/16,125 read; 10,254 unread; 15,846 unknown; 0 omitted**. First unread 5,872 `choose_font_box.cpp@56a7386c…`. Canonical owner/applicability evidence remains mandatory.


### Source closure through 5,909

The deterministic live prefix is exact-blob read-complete/responsibility-decomposed through **5,909**, completing `ui/boxes/**`. Accounting is **5,909/16,125 read; 10,216 unread; 15,846 unknown; 0 omitted**. First unread is 5,910 `ui/cached_round_corners.cpp@7da69a22…`. The predecessor d52b8c5 authority failure identified a stale `upstream.lock.json.source_disposition_evidence.read_through`; this descendant fixes that data contract without modifying or weakening the validator. Canonical production owner, UI/UX, backend/platform dependency and same-head evidence remain mandatory.


### Source closure through 5,980

Orders **5,910–5,980** are exact-blob read-complete/responsibility-decomposed. Accounting is **5,980/16,125 read; 10,145 unread; 15,846 unknown; 0 omitted**. First unread is 5,981 `ui/color_contrast.cpp@f981e717…`. Parent `9c969d1` repaired the validator-required narrative tokens; this batch preserves that exact contract. Canonical production owner, UI/UX, backend/platform dependency and same-head evidence remain mandatory.


### Source closure through 5,991

The deterministic prefix is read/decomposed through **5,991**. Color contrast/serialization maps to canonical semantic-theme utilities; button busy/context-menu/two-label behavior maps to canonical Button/IconButton/ContextMenu without source-named components. A visual disabled state does not replace command-level duplicate-execution fencing. Reading closes no unknown. Accounting: **5,991/16,125 read; 10,134 unread; 15,846 unknown; 0 omitted**.


### Source closure through 6,001

The deterministic prefix is exact-blob read/decomposed through **6,001**. Call action/mute UI maps to existing Human call + canonical Button/IconButton owners; service check maps to Checkbox/theme; compose-AI/large-paste maps to canonical Composer/attachment/AI owners; custom emoji Toast maps to Toast + emoji/media resolution. The current production slice canonicalizes Human call buttons and puts duplicate mute refusal plus rollback in the command owner, not presentation state. Unknown is unchanged because the complete responsibilities are not yet verified. Accounting: **6,001/16,125 read; 10,124 unread; 15,846 unknown; 0 omitted**. First unread is 6,002 `ui/controls/delete_message_context_action.cpp@9cecf76e…`.


### Source closure through 6,011

Orders **6,002-6,011** preserve delete TTL countdown/user-vs-expiry callback separation, aggregate download progress/finished navigation projection, dynamic image strip pointer-intent + cyclic keyboard selection, and emoji loading/panel geometry/suggestion lifetime semantics. Canonical owners are ContextMenu/Message lifecycle, Downloads/Resource, source-neutral Picker/Avatar rows, and IconButton/Composer/TextField/Popover. Exact applicability, domain ownership and executable evidence remain open, so global unknown stays **15,846**. Accounting: **6,011/16,125 read; 10,114 unread; 0 omitted**; first unread 6,012 `ui/controls/feature_list.cpp@8c3bf36c…`.

### Source closure through 6,032

Orders **6,012-6,032** are exact-blob read/decomposed for feature/detail rows, filter/share header state, invite-link actions/ContextMenu, jump-down unread projection, labeled emoji Tabs, location-picker service/UI boundaries, participant LoadingState skeletons and Popover lifecycle. All map to source-neutral canonical owners; reading does not reduce unknown. The descendant Human-call shipping surface also serializes camera/screen-share mutations through one `video-media` command fence with canonical pending projection, rollback and capture teardown. ForceMuted/RaisedHand/scheduled/audio-reactive call presentation remains mapped-open. Accounting: **6,032/16,125 read; 10,093 unread; 15,846 unknown; 0 omitted**. First unread 6,033 `ui/controls/round_video_recorder.cpp@b52b73bd…`. No baseline/release credit is granted.

### Source closure through 6,041

Orders **6,033-6,041** are exact-blob read/decomposed for round-video recording/encoding, sender identity, the primary Composer send-state control and silent-send accessibility. Round-video remains mapped-open as a real device/media/codec lifecycle, not a cosmetic widget. The descendant Composer now also removes its raw HTML send button and reuses canonical `SandIconButton`. Accounting: **6,041/16,125 read; 10,084 unread; 15,846 unknown; 0 omitted**. First unread is 6,042 `ui/controls/stars_rating.cpp@7b7dce6e…`.

### Source closure through 6,052

Orders **6,042-6,052** are exact-blob read/decomposed for Stars rating, source-neutral tabs/subsection reorder and swipe gesture/scroll ownership. The existing canonical `SandTabs` owner is extended with optional overflow visibility, context-menu requests, locked reorder boundaries, pointer cancellation/edge-scroll and Alt+Arrow keyboard reorder; no Telegram-derived Tabs owner was added. Stars account/reputation truth, complete Avatar/Badge subsection composition and the navigation/conversation swipe owner remain mapped-open. Accounting: **6,052/16,125 read; 10,073 unread; 15,846 unknown; 0 omitted**. First unread is 6,053 `ui/controls/tabbed_search.cpp@ca207d0b…`. Reading alone does not reduce unknown.

### Source closure through 6,068

Orders **6,053-6,068** are exact-blob read/decomposed for grouped search/category picking, detail rows and transient tooltip lifetime, title/status chrome, localized fixed-precision amount input/IME, ephemeral-media countdown presentation, and Avatar/profile-media acquisition/upload/privacy/streaming lifecycle. The existing canonical `SandTooltip` gains optional auto-dismiss, outside-press/Escape dismissal and focus-return policy with a focused contract test; no Telegram-derived Tooltip owner was added. Search/Picker shipping composition, full profile-media ownership, monetary-domain applicability and authoritative ephemeral-message lifecycle remain mapped-open. Accounting: **6,068/16,125 read; 10,057 unread; 15,846 unknown; 0 omitted**. First unread is 6,069 `ui/controls/who_reacted_context_action.cpp@6851dee0…`. Reading alone does not reduce unknown.

### Source closure through 6,084

Orders **6,069-6,084** are exact-blob read/decomposed across who-read/reaction context actions, platform-obsolescence and screen-reader-mode Banners, country selection, dynamic media thumbnails, animated text, and credits/commerce graphics/style. Canonical owners remain source-neutral and all rows remain mapped-open unless independently closed; reading does not reduce unknown. Accounting: **6,084/16,125 read; 10,041 unread; 15,846 unknown; 0 omitted**. First unread is 6,085 `ui/effects/drifting_particles.cpp@ed119cf9…`.


### Source closure through 6,104

Orders **6,085-6,104** are exact-blob read/decomposed across decorative particles, reaction/custom-emoji fly overlays, fireworks/glare/LoadingState skeletons, Composer-to-Transcript message-send transitions, Stars particles/Story outlines, Premium commerce styling and an optional GPU 3D commerce cover. The source responsibilities require pause/teardown correctness, reduced-motion/power-policy handling, target-identity fencing, theme/DPR/RTL/responsive behavior, resource completion recovery and GPU capability fallback. They map only to source-neutral canonical animation/LoadingState/Transcript/Resource/Avatar/Status/commerce/design-system owners; no Telegram-derived parallel visual runtime is accepted. Accounting: **6,104/16,125 read; 10,021 unread; 15,846 unknown; 0 omitted**. First unread is 6,105 `ui/effects/premium_3d_mesh.cpp@b8541b31…`. Reading alone does not reduce unknown or grant implementation/release credit.


### Source closure through 6,125

Orders **6,105-6,125** are exact-blob read/decomposed across validated 3D asset loading, GPU support gating, reactive credits/limit/subscription presentation, coin/diamond RHI renderers, promo particle strategies and the interactive Premium Star lifecycle. Domain truth stays in canonical commerce/credits/gifts/subscription owners; graphics reuse canonical controls and the thinnest visual/GPU adapters. Malformed assets, unsupported RHI, shader/buffer/pipeline failures, power-saving/reduced-motion, pause/resume and partial-init teardown remain explicit failure/lifecycle obligations. Accounting: **6,125/16,125 read; 10,000 unread; 15,846 unknown; 0 omitted**. First unread is 6,126 `ui/effects/premium_star_model.cpp@499d751a…`. Reading alone grants no implementation or release credit.


### Source closure through 6,145

Orders **6,126-6,145** are exact-blob read/decomposed across Premium Star assets/particles/GPU renderer, colored collectible decoration, responsive commerce top-bar capability fallbacks, reaction fly presentation, Checkbox/Avatar selection rendering, scroll-edge shadows, and send-action status animations. Send-action types include record/upload/round/speaking/choose-sticker with typing fallback; same-family restart and speaking finish/restart transitions remain visual projections over authoritative conversation state, and animation-disabled mode resolves to static frames. All responsibilities map to source-neutral canonical owners; reading does not reduce unknown. Accounting: **6,145/16,125 read; 9,980 unread; 15,846 unknown; 0 omitted**. First unread is 6,146 `ui/effects/shake_animation.cpp@38fbd6f0…`.


### Source closure through 6,165

Orders **6,146-6,165** are exact-blob read/decomposed across shake/LoadingState skeleton feedback, snowflake/star-burst decoration, the message-removal dissolve capture/collapse/GPU compute lifecycle, disclosure-arrow affordance and TTL timer-icon projection. The dissolve path is explicitly presentation after authoritative deletion: it capability-gates power/RHI/compute, pre-captures exact Transcript items, reconciles collapse gaps and scroll baselines, bounds GPU particles/frame delta, and tears down pending/per-item resources; it never owns deletion truth. Accounting: **6,165/16,125 read; 9,960 unread; 15,846 unknown; 0 omitted**. First unread is 6,166 `ui/effects/unique_gift_message_bubble.cpp@22802623…`.


### Read-through 6166-6171

Accepted Telegram authority remains `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`. Orders **6,166-6,171** are exact-blob read and responsibility-decomposed: unique-gift message bubble geometry maps to canonical TranscriptEntry/Avatar plus Gift/Commerce presentation; upload progress lifecycle maps to canonical Resource/attachment upload + Status/Progress/IconButton; voice-once particles map to canonical voice-message playback presentation with reduced-motion/teardown policy. Reading alone does not reduce unknown. Accounting after orders 6,166-6,171 was **6,171 / 16,125 read**, **9,954 unread**, **15,846 unknown**, **0 omitted**. The next unread order was **6,172** `Telegram/SourceFiles/ui/empty_userpic.cpp`.

### Read-through 6172-6182

Accepted Telegram authority remains `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`. Orders **6,172-6,182** are exact-blob read and responsibility-decomposed: fallback identity presentation maps to canonical Avatar/semantic Icon/Resource owners; filter-icon selection maps to canonical Picker/Popover/Menu/IconButton with the existing conversation-filter domain owner; grouped-media geometry maps to the canonical TranscriptEntry/Resource layout utility and must remain deterministic and single-owned. Reading alone does not reduce unknown. Accounting after orders 6,172-6,182 was **6,182 / 16,125 read**, **9,943 unread**, **15,846 unknown**, **0 omitted**. The next unread order was **6,183** `Telegram/SourceFiles/ui/image/image.cpp`.

### Current read-through 6183-6191

Orders **6,183-6,191** are exact-blob read and responsibility-decomposed across the image/resource base layer: prepared image cache keys and DPR transforms; typed download/image locations with versioned serialization, cache-key derivation and file-reference refresh; protocol-to-resource factories including progressive/cached/in-memory/web/video forms; bounded local-image decoding; and sanitized SVG preview rendering with explicit byte/dimension limits. These map to the existing canonical `Resource`/attachment/media owner rather than a second download/cache owner. Reading does not reduce `unknown`. Current accounting is **6,191 / 16,125 read**, **9,934 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,192** `Telegram/SourceFiles/ui/item_text_options.cpp`.

### Read-through 6192-6202

Orders **6,192-6,202** are exact-blob read and responsibility-decomposed across conversation-aware text option projection, single-window layer-stack lifecycle, canonical context-menu icon semantics, new/attention Badge projection, and local passcode-strength/transliteration helpers. These responsibilities map to existing source-neutral TranscriptEntry, Dialog/Popover, ContextMenu/Menu/Icon, Badge/Status and security/passcode owners. Sensitive passcode candidates must remain ephemeral and unlogged. Reading alone does not reduce unknown. Current accounting is **6,202 / 16,125 read**, **9,923 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,203** `Telegram/SourceFiles/ui/peer/color_sample.cpp@d11ac016c6e318786c3a1206efa415c91928ee21`.

### Read-through 6203-6214

Orders **6,203-6,214** are exact-blob read/decomposed across color/profile selection, video Avatar Resource playback, power-saving/reduced-motion policy, resize lifecycle, bounded row-scroll render caching and SearchField query/a11y composition. All map to existing source-neutral canonical owners and remain mapped-open. Reading alone does not reduce unknown. Accounting: **6,214 / 16,125 read**, **9,911 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,215** `Telegram/SourceFiles/ui/text/format_song_document_name.cpp@d5a77025b392b8385bff3268be1f7465d91d48a2`.

### Read-through 6215-6224

Orders **6,215-6,224** are exact-blob read/decomposed across media naming, locale/value/currency/credits formatting, custom-emoji animation lifecycle and canonical rich-text parser presets. All remain mapped-open; reading alone does not reduce unknown. Accounting: **6,224 / 16,125 read**, **9,901 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,225** `Telegram/SourceFiles/ui/top_background_gradient.cpp@7325d188f856d76f08c67a2bdb43ce1f50eb94b7`.


### Read-through 6225-6239

Orders **6,225-6,239** are exact-blob read/decomposed across commerce/profile top-gradient pattern rendering, unread/peer Badge precedence and animation/power-saving behavior, unread counter formatting, Avatar/Resource cache invalidation and fallback shapes, canonical vertical-list composition, and WebView theme/zoom/style serialization plus attribute/script escaping. All remain mapped-open and source-neutral; build-only `ui_pch.h` is recorded without claiming product closure. Reading alone does not reduce unknown. Accounting: **6,239 / 16,125 read**, **9,886 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,240** `Telegram/SourceFiles/ui/widgets/chat_filters_tabs_mode.h@f5d37194063110eb2ba8698f5b714e8a7ebe37ea`.


### Read-through 6240-6244

Orders **6,240-6,244** are exact-blob read/decomposed across persisted chat-filter Tabs presentation mode, canonical Tabs/Badge/ContextMenu composition, locked-range handling, custom-emoji pause, and drag-reorder lifecycle with pinned intervals, threshold start, edge auto-scroll, cancel/apply convergence and stable index remap. These remain mapped-open and must reuse existing source-neutral Tabs/ContextMenu/Badge/Status plus the single canonical ordering state owner. Reading alone does not reduce unknown. Accounting: **6,244 / 16,125 read**, **9,881 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,245** `Telegram/SourceFiles/ui/widgets/chat_filters_tabs_strip.cpp@750feab6cbf3b30a3b1e2bc946d85eaed4a4382c`.


### Read-through 6245-6251

Orders **6,245-6,251** are exact-blob read/decomposed across the shipping chat-filter Tabs composition, premium lock/menu/edit/remove/mark-read and saved ordering lifecycle, plus canonical Color Picker/validated field synchronization and Continuous/Media Slider pointer-wheel-keyboard-a11y progress/finished semantics. These remain mapped-open and must reuse source-neutral Tabs/ContextMenu/Badge/Picker/TextField/Slider and existing filter/settings/media domain owners. Reading alone does not reduce unknown. Accounting: **6,251 / 16,125 read**, **9,874 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,252** `Telegram/SourceFiles/ui/widgets/cross_fade_label.cpp@fb094439ceebc0ad1b45db68ebf08d31450464e3`.


### Read-through 6252-6261

Orders **6,252-6,261** are exact-blob read/decomposed across cross-fade text motion, Discrete/Settings Slider selection timing and ripple state, expandable participant Checkbox/Avatar collection semantics, phone/country/username masked-input normalization including Windows IME re-entry fencing, and count-aware time-part placeholders. These remain mapped-open and must reuse source-neutral Status/text motion, Tabs/Slider, ParticipantRow/Checkbox/Avatar, and TextField owners. Reading alone does not reduce unknown. Accounting: **6,261 / 16,125 read**, **9,864 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,262** `Telegram/SourceFiles/ui/widgets/glare_tooltip.cpp@b38e44a0769199bf65b5d584181e3b4f24d716aa`.


### Read-through 6262-6269

Orders **6,262-6,269** are exact-blob read/decomposed across glare Tooltip tracking/timer/teardown and motion, gradient Button glare/ripple/cache behavior, equal-width horizontal Button layout, and LevelMeter value projection. These are presentation responsibilities over existing source-neutral Tooltip/Button/layout/Status owners and must honor reduced-motion and lifecycle teardown. Reading alone does not reduce unknown. Accounting: **6,269 / 16,125 read**, **9,856 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,270** `Telegram/SourceFiles/ui/widgets/marquee_label.cpp@ddb4fac4dc2eb2f4d990b283aad018416ce7d1d2`.
Orders **6,304-6,315** are exact-blob read/decomposed across Wallet card pointer-follow motion and DPR-aware gradient caches, a capability-restricted Wallet-to-chat/session Show adapter, collectible metadata/artwork priority lanes with a 10-chain budget, 60-second deadline, generation fencing, Gift-to-web fallback, cancellable Resource loading and preload-window projection, plus the encrypted transfer-comment reveal lifecycle with account/session/app-lock/sleep fencing, bounded identity resolution, key authorization, revision/scope cancellation and plaintext wipe on reset. These remain mapped-open to the existing source-neutral Wallet/Payments/Commerce, Resource/file-transfer/cache, Session/Navigation, Security/Key Protection, Theme, motion and canonical design-system owners; no Telegram-derived parallel Wallet UI/runtime is introduced. Reading alone does not reduce unknown. Accounting: **6,315 / 16,125 read**, **9,810 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,316** `Telegram/SourceFiles/wallet/wallet_content.cpp@6aa42680fee3870ae9969104fdb5872c309e8ef2`.
Orders **6,316-6,319** are exact-blob read/decomposed across the full Wallet content/composition monolith and its public seam plus the durable custody store. The source proves exact-dependency/revision fee quotes, per-press frozen recipient/amount/comment/link expiry, key-ladder and vault-epoch fencing, bounded signing readiness, QuoteExpired retries only before anything leaves the device, SubmissionUnknown operation handoff rather than blind resend, bounded recipient/name/address lookups, secure phrase/restore/import/conflict/backup/rotation lifecycles, polling/teardown and list/scroll-anchor behavior. Custody persistence is v4, separates immutable anchor and current signing keys, retains pending rotation/new-key/awaiting-server reconciliation, validates old/broken data fail-closed and centralizes secret-reference removal. All remain mapped-open to existing source-neutral Wallet/Payments, Security/Key Protection/Custody, storage, Session/Navigation, Resource, Motion and canonical design-system owners; no Telegram-derived parallel owner is introduced. Reading alone does not reduce unknown. Accounting: **6,319 / 16,125 read**, **9,806 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,320** `Telegram/SourceFiles/wallet/wallet_diamond_flight.cpp@9444c0eac18a5c625aad9a2671bbd319974129fb`.
Orders **6,320-6,323** are exact-blob read/decomposed across the send-diamond animation and the Wallet engine/host bridge. Motion semantics include a dynamic target, target-loss cancellation, exact single landing/finish, reduced-motion and power-saving suppression, loop continuity and bounded dirty-region repaint. Engine semantics include strict provider-origin/path enforcement, Accepted/Rejected/Uncertain routed submission, a 40-second ambiguous-broadcast wait that preserves late MTProto settlement, durable send-journal compare-exchange, account/clear-epoch protected-secret fencing, exact failed-call secret cleanup, stale private-result rejection, serial worker ownership and teardown that wakes blocked calls before shutdown while relying on durable journal recovery on restart. These remain mapped-open to existing source-neutral Wallet/Payments, Network, Vault/Key Protection, durable Storage/Journal and Motion owners. Reading alone does not reduce unknown. Accounting: **6,323 / 16,125 read**, **9,802 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,324** `Telegram/SourceFiles/wallet/wallet_fiat.cpp@6e16d23676a8d24ff840c71f88310cb21d540ce4`.
Orders **6,324-6,325** are exact-blob read/decomposed across Wallet fiat/rates formatting: unavailable-rate ellipsis, currency-specific exponent/decimal/grouping, approximate U+2248 output, tiny positive-value significant digit projection, localized currency names, stable top-currency ordering from the active currency plus cloud/system/input-method/phone-country locale signals, and minor-unit nanos. They remain mapped-open to source-neutral canonical Money/Currency formatting and Wallet rates owners.

Orders **6,326-6,339** are exact-blob read/decomposed across Wallet key protection, funding/on-ramp, palette replacement, singleton panel lifecycle, recovery phrase shares, live fiat-rate refresh, and sending effects. Existing-owner-first production work at parent exact HEAD `882728682058ea6235cf625c7e8a33f851fad1a3` extends `native/mahayana-messaging/src/wallet.rs`, `engine.rs`, and `service.rs`: provider-neutral authenticated key wrapping with epoch fencing and zeroized secret bytes; fixed-field recovery seed/XOR shares plus authenticated holder encryption; 5-minute rate refresh / 30-second retry state; funding-request generation fencing and optional base-currency fallback; singleton panel state; deterministic clock/glare math; native/server-authoritative runtime projection. Exact-head evidence: Desktop Chat Parity run `38016864742` completed success; Rust desktop runtime run `38016864748` bound the same SHA, canonical messaging tests succeeded and artifact `11656926928` has digest `sha256:cb27c66d4893776f5105741254f86df84d25a707ff7cd9f256e708c76676b0fa`. These rows intentionally remain mapped-open: no concrete platform key-protection adapter, real funding/rates service adapter, wallet shipping renderer/panel consumer, wallet-specific palette consumer, recovery-share product entrypoint, or sending-effect paint consumer has exact-head production evidence yet. Reading therefore does not reduce global unknown. Accounting: **6,339 / 16,125 read**, **9,786 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,340** `Telegram/SourceFiles/wallet/wallet_session.cpp@7000a36395819deea9ca8ba0cce227ff0e535f32`.


### Read-through 6340-6343

Orders **6,340-6,343** are exact-blob read/responsibility-decomposed across the Wallet Session monolith/public state contract and the real-time Wallet stream. Existing-owner-first production work extends only canonical `native/mahayana-messaging` Wallet/ConnectedApp owners: parked-wallet discovery, panel-scoped legacy-balance lookup, durable connected-app recovery fallback separation, and a source-neutral live-stream epoch/backoff/keepalive/renew/coalesce/history-recheck state machine. The stream still lacks a real Fabushi Host/network service adapter and 6,340-6,341 retain substantial mapped-open custody/history/collectibles/preview/send-recovery/rotation/TonConnect/UI responsibilities, so none of these rows is marked verified and global `unknown` is unchanged. Accounting: **6,343 / 16,125 read**, **9,782 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,344** `Telegram/SourceFiles/wallet/wallet_ton_connect.cpp@60bf68a9…`. Fresh exact-head GitHub Actions are required for the accounting descendant.

### Read-through 6344-6353

Orders **6,344-6,353** are exact-blob read/responsibility-decomposed across the connected-app session/key/connect/disconnect lifecycle, canonical connect/settings UI projection contract, durable claim journal, transaction emulation, and deep-link/start-param boundary. Existing-owner-first production descendants already implement transport message identity, staged claim recovery, source-neutral connect/key/disconnect state, emulation parsing and start-param security/encoding inside canonical native/mahayana-messaging owners. The connect/settings UI projection, real service/navigation composition and complete request-family closure remain mapped-open; no row is promoted to verified and reading does not reduce global unknown. Accounting: **6,353 / 16,125 read**, **9,772 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,354** Telegram/SourceFiles/wallet/wallet_ton_connect_request.cpp@496881cf….

### Read-through 6354-6357

Orders **6,354-6,357** are exact-blob read/responsibility-decomposed across the full connected-app request scheduler/Flow contract and its request-confirmation projection. Existing-owner-first production descendants now cover transport message identity, active/silent/waiting arbitration, durable claim recovery rebuild, recovery polling/submission settlement, flow-completion ownership fencing and Recovery::Offer fallback through canonical native/mahayana-messaging ConnectedApp owners. Remote pending/service composition, closed-session late context, wallet resolve/polling, cryptographic decrypt/sign/encrypt, fee/emulation send integration, Recovery::Answer re-encryption and shipping request UI/a11y/visual evidence remain mapped-open. No row is promoted to verified and reading does not reduce global unknown. Accounting: **6,357 / 16,125 read**, **9,768 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,358** Telegram/SourceFiles/wallet/wallet_transfer_messages.cpp@69cfdacd….



### Revision 9 source closure through 6,361

Orders **6,358-6,361** are exact-blob read/responsibility-decomposed for optimistic outbound transfer-message reconciliation and the durable submitted-transfer store. The canonical Wallet journal already owns handoff/lookup/terminal recovery; this descendant additionally fail-closes noncanonical source, destination and collectible addresses through the existing wallet-address validator. The Message owner still lacks the complete per-draft server-id floor, transport-identity and identical-twin adoption/refusal contract, so 6,358-6,359 remain explicitly mapped-open rather than receiving false implementation credit. Accounting is **6,361/16,125 read; 9,764 unread; 15,846 unknown; 0 omitted**. First unread is 6,362 `Telegram/SourceFiles/wallet/wallet_unlock.cpp@5fbaea9d950772164094e7477e5f786f9a78279f`.


### Revision 9 read-through 6362-6367

Wallet unlock, user-address and vault files are exact-blob read/responsibility-decomposed. Descendant `cb6d56174fb3e3f34c99e7bed3156c3b1e8d523d` closes one security gap: unlocked Wallet projection is runtime-only across serde/restart. Full grant/retention/platform-factor/address-service/multi-account vault lifecycle remains mapped-open. Accounting: 6,367/16,125 read; 9,758 unread; 15,846 unknown; 0 omitted. First unread: 6,368 `Telegram/SourceFiles/webauthn/cable.h@6909eb913dbb41765ebbaeecb2072f130f868655`.


### Revision 9 read-through 6368-6377

The first CaBLE/WebAuthn tranche is exact-blob read/responsibility-decomposed. Existing Fabushi WebAuthn provider/signer/proxy is the canonical owner; full BLE/QR/tunnel/Noise/CTAP and ceremony UI parity stays mapped-open. `94aaee4` already closes one fail-open boundary by rejecting unknown ceremony kinds instead of treating them as login/get. Accounting: 6,377/16,125 read; 9,748 unread; 15,846 unknown; 0 omitted. First unread: 6,378 `Telegram/SourceFiles/webauthn/org.bluez.xml@245e0b9bc27e1cfbb31bd78d750ae189adf5f0cc`.


### Revision 9 read-through 6378-6380

BlueZ interface and common WebAuthn security-key/CaBLE orchestration are exact-blob read/responsibility-decomposed. Existing Fabushi WebAuthn provider/signer/proxy remains the canonical owner; device/PIN/cancellation/fallback/platform parity remains mapped-open. Accounting: 6,380/16,125 read; 9,745 unread; 15,846 unknown; 0 omitted. First unread: 6,381 `Telegram/SourceFiles/window/main_window.cpp@ae2b4949734e6c8826310c96b65f51b630ab600d`.


### Revision 9 read-through 6381-6388

Main window and first notification-manager tranche are exact-blob read/responsibility-decomposed against existing Electron main/window-state/window-chrome/dock-badge/OS-notification owners. Existing geometry/focus/maximized/badge and scoped notification action behavior is retained; close-to-tray/background, privacy-aware OS title, full canonical conversation notification mute/privacy/grouping/device-delay and fallback/resource lifecycle remain explicitly mapped-open. Accounting: 6,388/16,125 read; 9,737 unread; 15,846 unknown; 0 omitted. First unread: 6,389 `Telegram/SourceFiles/window/section_memento.h@d139928a9996a6333e541a5db9c77e64056b41fd`.


### Revision 9 read-through 6389-6392

Section navigation/workspace contracts and draw-to-reply media flow are exact-blob read/responsibility-decomposed. Qt widget/slide/painting is not copied; routing, memento identity, focus/search/permission/theme behavior and async draw-to-reply target/payment/ephemeral safeguards map into existing ProductShell/ConversationWorkspace/Search/Permissions/Composer/Resource/Message owners. Accounting: 6,392/16,125 read; 9,733 unread; 15,846 unknown; 0 omitted. First unread: 6,393 `Telegram/SourceFiles/window/themes/window_theme.cpp@2ead4b28873908361728df3f622318ed073cdee2`.


### Revision 9 read-through 6393-6412

Theme runtime/editor/preview/chat/cloud/embedded/accent/name-generation files are exact-blob read/responsibility-decomposed. Existing SandThemeController and the Fabushi Design System remain the only app/theme visual authority; Telegram Qt palette/theme-format rendering is not ported. Applicable bounded parsing/resources, transactional test→keep/revert, async generation fencing, per-conversation/cloud theme state, accessibility/contrast and editor lifecycle remain mapped-open against existing Theme/Settings/Conversation/Resource owners. Accounting: 6,412/16,125 read; 9,713 unread; 15,846 unknown; 0 omitted. First unread: 6,413 `Telegram/SourceFiles/window/window.style@a91951fc42761db61f7f7718f89e226fc9d3b5c2`.


### Revision 9 read-through 6413-6423

Window style/adaptive/chat preview/chat switch/connection/controller files are exact-blob read/responsibility-decomposed. Current Fabushi already has canonical responsive shell, ConversationWorkspace/List/navigation, CoordinatorConnectionController and Electron window owners, so no Telegram window runtime is introduced. Full adaptive columns, accessible chat preview/switch, proxy/retry status, multi-account/separate-window and security/layer sequencing remain mapped-open where current behavior is partial. Accounting: 6,423/16,125 read; 9,702 unread; 15,846 unknown; 0 omitted. First unread: 6,424 `Telegram/SourceFiles/window/window_filters_favorite.cpp@c51e19763a7939fe908fb8b9fc3eb5f73735674f`.


### Revision 9 live read-through note: 6424-6437

Orders **6,424-6,437** are exact-blob read-complete and responsibility-decomposed on accepted `28ac576967a1026ecc89a927360fa0e138d7c88c`. Favorite/filter files preserve validated source-neutral shortcut/deep-link cancellation, collection selection/reorder/premium-lock, unread/mark-read, drag-hover and keyboard/screen-reader semantics. History-hider files preserve exactly-once overlay dismissal. Lock files preserve passcode flood/busy/async derivation, OS-unlock cooldown/lifetime and terms-consent security semantics. Main-menu/helpers preserve multi-account unread, archive lifecycle, profile/status, Contacts/Calls/Wallet/Settings navigation, theme/scale, owned-room enumeration and lifetime-bound dynamic Mini App/Plugin resource projection. All remain **mapped-open** to existing canonical ProductShell/Account/Conversation/ReadState/Security/Settings/Marketplace/MiniApp/Resource owners; no Telegram runtime/sidebar/menu/security registry is introduced. Reading alone closes no unknown. Accounting: **6,437/16,125 read; 9,688 unread; 15,846 unknown; 0 omitted**. First unread: **6,438 `Telegram/SourceFiles/window/window_media_preview.cpp@4fe0e7f11298b0b56c99c47e43c27c6fd47d0282`**. Fresh descendant exact-head Actions are required.

### Revision 9 live read-through note: 6438-6448

Orders **6,438-6,448** are exact-blob read-complete and responsibility-decomposed on accepted `28ac5769`. They cover media-preview resource/playback lifetime, context-sensitive canonical conversation commands, temporary restore shells, durable multi-window/account/conversation restore, section fallback semantics, and detachable-window route/security identity. All remain **mapped-open**. Current `source/electron-main/window-state-persistence.ts` is explicit partial parity: it persists one main-window placement but not durable account/conversation multi-window restore. Reading alone closes no unknown. Accounting: **6,448/16,125 read; 9,677 unread; 15,846 unknown; 0 omitted**. First unread: **6,449 `Telegram/SourceFiles/window/window_session_controller.cpp@df216237e47d22c200a51b2c7fda31175682d3c5`**.

### Revision 9 live read-through note: 6449-6458

Orders **6,449-6,458** are exact-blob read-complete and responsibility-decomposed. The large SessionController is decomposed into canonical Navigation/Conversation/Search/Window/Resource/Permissions/Calls/Settings/Theme/Story owners rather than ported as a new monolith. The range also captures typed deep-link intent, setup-email enrollment/verification with flood/expiry handling, canonical motion/adaptive-top-bar semantics, and passcode unlock exactly-once convergence across windows. All remain **mapped-open**. Accounting: **6,458/16,125 read; 9,667 unread; 15,846 unknown; 0 omitted**. First unread: **6,459 `Telegram/Telegram.plist@d156eda29c86bbd11b3c7757b9d419e2c863ea8b`**.

### Revision 9 live read-through note: 6459-6483

Orders **6,459-6,483** close the upstream macOS bundle plist, helper/app entitlements, icon-catalog manifest and every referenced 16/32/128/256/512 1x/2x icon blob by exact identity. The applicable responsibility is bundle/privacy/protocol/signing/sandbox/icon coverage under **Fabushi** identity; Telegram/TON schemes and Telegram artwork are explicitly not migrated. Current `desktop/package.json` already owns `com.ombhrum.fabushi`, the `fabushi` scheme, Fabushi icon, forced signing, notarization, and microphone/camera usage; current MAS entitlements own sandbox/network/user-selected/mic/camera. Downloads/bookmarks/location remain applicability-open and are not added without a shipping need. Accounting: **6,483/16,125 read; 9,642 unread; 15,846 unknown; 0 omitted**. First unread: **6,484 `Telegram/ThirdParty/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37`**.

### Revision 9 live read-through note: 6484-6505

Orders **6,484-6,505** close the root ThirdParty gitlink identity layer: exact pinned commits and `.gitmodules` repositories are mapped to source-neutral capability/implementation roles. This includes generic C++ support (GSL/expected/range-v3/TooManyCooks), math/Markdown/QR/syntax/image/language/spell utilities, Linux IME/portal adapters, CBOR/FIDO2/passkey dependencies, compression/hash/password-strength primitives, and `tgcalls`. The root pin does **not** close any nested recursive source; all nested entries remain independently unread/unknown until their later deterministic orders. No Telegram runtime/library is automatically retained. Accounting: **6,505/16,125 read; 9,620 unread; 15,846 unknown; 0 omitted**. First unread: **6,506 `Telegram/build/build.bat@3e5f698f99c5944e237c514d60124bc25489ea76`**.

### Revision 9 live read-through note: 6506-6541

Orders **6,506-6,541** close the root build/release tooling source read. Product-relevant responsibilities include fail-fast release prerequisites, version/channel/artifact collision fencing, macOS preserved-toolchain and universal-resource checks, Windows/Linux packaging, App Store/GitHub publication, source tarballs including submodules, symbols, installer/upgrade semantics, reproducible dependency preparation, canary positive+negative signature fixtures, ES256 Key Vault signing with manifest-key-coordinate verification, and digest/dedupe/retry semantics for spellcheck dictionary distribution. Telegram product/private paths are not targets; equivalent Fabushi guarantees must live in existing GitHub Actions/electron-builder/updater/platform/resource owners. Accounting: **6,541/16,125 read; 9,584 unread; 15,846 unknown; 0 omitted**. First unread: **6,542 `Telegram/cmake/binobj2obj.py@3683a6dd3a72706ee382ea415d6e9cfaae84d8ce`**.

### Source closure through 6,574

Orders **6,542-6,574** are exact-blob read-complete/responsibility-decomposed for CMake/build-runtime composition. They preserve source-neutral responsibilities for deterministic code/resource generation, updater trust-root and signed-manifest embedding, FIDO2/WebAuthn, Calls/media composition, Wallet/Payments, password-strength dictionaries, Localization, data export, protocol/session maturity, Apple Swift runtime packaging and production-code updater tests. No Telegram-specific runtime or second product owner was introduced. Accounting: **6,574/16,125 read; 9,551 unread; 15,846 unknown; 0 omitted**. First unread is 6,575 gitlink `Telegram/codegen@dbd1d53137cd581cfdbeebe93bcaf54abf55ec74`. Reading/decomposition grants no implementation or release credit; same-head Actions and production/release evidence remain mandatory.


### Source closure through 6,591

Orders **6,575-6,591** are exact root-entry read-complete/responsibility-decomposed. The pinned `Telegram/codegen` child tree was inspected for tokenizer/UTF-8, emoji compatibility, localization and style-generation responsibilities; configure wrappers preserve fail-closed target/toolchain/credential propagation; direct library gitlinks map to existing runtime/concurrency/media/QR/reactive/spellcheck/storage/protocol/translation/design-system/call/webview owners. `Telegram/create.bat` is explicitly classified development-only non-applicable after reading, so it closes exactly one unknown without counting as omitted. Accounting: **6,591/16,125 read; 9,534 unread; 15,845 unknown; 0 omitted**. First unread is 6,592 `Telegram/shaders/argb32.frag@4b4deb01e408b5f441808dc104488e3b8eeabfe2`. No mapped-open row receives implementation or release credit.


### Source closure through 6,626

Orders **6,592-6,626** are exact-blob read-complete/responsibility-decomposed for GPU shader contracts. The tranche preserves ARGB/NV12/YUV420 conversion, OpenGL/Vulkan coordinate-origin handling, blur/dither, premultiplied alpha, rounded/fade/shadow composition, PiP nine-slice shadow behavior, premium time/night/alpha material inputs, and seeded/timestep-bounded particle lifecycle. These are mapped to source-neutral Fabushi rendering/Resource/Call/visual-effect owners; Telegram branding and pixel design are not carried over. Accounting: **6,626/16,125 read; 9,499 unread; 15,845 unknown; 0 omitted**. First unread is 6,627 `changelog.txt@bf82383b02717265acac50035c2ce3000afdac7a`. All shader rows remain mapped-open until shipping consumers, focused tests and cross-platform visual evidence exist.


### Root source closure complete at 6,653

Orders **6,627-6,653** close the remaining root entries: release changelog/build helpers, credential/build documentation, AppStream screenshot assets, WEB proxy design and acceptance contracts, Linux XDG/AppStream/DBus/Snap shipping integration, Wallet/thread/shared-media/navigation task evidence, and static-analysis/edit-rule tooling. Root accounting is now **6,653/6,653 exact-read**, while the recursive census remains **6,653/16,125 read; 9,472 unread; 15,845 unknown; 0 omitted**. The next recursive identity is order 6,654: `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37`, mount `Telegram/ThirdParty/GSL`, path `.clang-format`, object `c12d3bf2994fd5a083c04025d355a5ab4b3f6802`. The existing validator intentionally forbids crossing the root prefix without an explicit component identity schema, so schema/validator extension is the next required gate rather than a bookkeeping bypass.

### Recursive component read-through 6,654-6,697 — Microsoft/GSL

Orders **6,654-6,697** exact-read the pinned `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37` component mounted at `Telegram/ThirdParty/GSL`. The 44 entries cover repository/build/CI metadata, license/security/provenance, header-only safety contracts (bounds and extent checks, non-null ownership, checked narrowing, byte/string boundaries, scope cleanup), and focused fail-closed tests. Applicable semantics map to existing Fabushi Build/Quality/Provenance and Rust/TypeScript/platform-boundary owners; no Telegram/GSL-specific runtime is introduced. Reading does **not** reduce `unknown`; production/shipping evidence remains required. Accounting: **6,697/16,125 read; 9,428 unread; 15,845 unknown; 0 omitted**. Next: **6,698 `Telegram/ThirdParty/MicroTeX::.github/workflows/ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764`**.

### Recursive component read-through 6,698-6,714 — MicroTeX entry/build contracts

Orders **6,698-6,714** exact-read the pinned `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f` build/entry contract under `Telegram/ThirdParty/MicroTeX`. The applicable product behavior maps into the existing ConversationWorkspace/Transcript math owner and existing Build/Resource/Release provenance owners. Current production work adds a 32 KiB pre-parse formula bound and bounded 32 MiB LRU markup cache without retaining the C++/Qt/GDI runtime. Reading does **not** reduce `unknown`; current exact-head GitHub Actions evidence is still required. Accounting: **6,714/16,125 read; 9,411 unread; 15,845 unknown; 0 omitted**. Next: **6,715 `Telegram/ThirdParty/MicroTeX::readme/example_bw_false.svg@9e812c823d3cbe06ab43ee77f10b6122bd638dd`**.


### Recursive component read-through 6,715-6,740 — MicroTeX reference/golden/resource manifest

Orders **6,715-6,740** exact-read the pinned MicroTeX reference SVG/PNG outputs, executable sample corpus, golden rendered samples, TeX logo, resource-root sentinel and `RES_README` font/license/language resource manifest. Applicable semantics map to the existing canonical ConversationWorkspace/Transcript math visual-regression owner plus Resource/Release provenance owners. These assets are regression/provenance authority only; they do not justify retaining or shipping a MicroTeX/Telegram renderer. Reading does **not** reduce `unknown`; exact-head production/resource/visual evidence remains required. Accounting: **6,740/16,125 read; 9,385 unread; 15,845 unknown; 0 omitted**. Next: **6,741 `Telegram/ThirdParty/MicroTeX::res/SAMPLES.tex@e03f0088e3fba1d225b90d06d586f32c5f70310b`**.

### Recursive component read-through 6,741-6,800 — MicroTeX multilingual formula resources

Orders **6,741-6,800** exact-read the executable formula sample corpus, compiled font-resource manifest, Cyrillic license/mappings/symbols/font metrics/fonts, Latin/math font resources and licenses, and the first Greek font/metric resources. Applicable responsibilities stay in the existing ConversationWorkspace/Transcript math owner plus Resource/Packaging/Release provenance owners: multilingual Unicode-to-symbol/formula mapping, deterministic glyph metrics/kerning, explicit resource fallback, packaged resource completeness, license/notice provenance, and rendering regression. Reading does **not** reduce `unknown` and provides no verified credit. Accounting: **6,800/16,125 read; 9,325 unread; 15,845 unknown; 0 omitted**. Next: **6,801 `Telegram/ThirdParty/MicroTeX::res/greek/fcmbpg.xml@dbedd0385cdb004569e0d69b2567a1fa6c6cad5f`**.

### Recursive component read-through 6,801-6,830 — MicroTeX Greek resources and atom safety

Orders **6,801-6,830** exact-read the rest of the Greek glyph resources plus the first atom/rendering implementation slice. Applicable atom responsibilities include wrapper-chain depth bounding, copy-before-mutation for shared cached atoms, exceptional-path state cleanup, bounded matrix/column-spec expansion, finite user length handling, TeX operator/glue/kerning isolation, delimiter/matrix/layout semantics, and deterministic multilingual glyph resources. These map to the existing canonical KaTeX math owner and Resource/Release provenance owners. The shipping owner now sets explicit `maxExpand=1000`, `maxSize=1000em`, and `trust=false` for strict and recovery renders, with a focused Desktop Chat Parity contract; exact-head verification is still required. Reading does **not** reduce `unknown`. Accounting: **6,830/16,125 read; 9,295 unread; 15,845 unknown; 0 omitted**. Next: **6,831 `Telegram/ThirdParty/MicroTeX::src/atom/atom_space.h@3ba203a52a1fcff831a1137dfa6c3e35927fb0d7`**.

### Recursive component read-through 6,831-6,852 — MicroTeX bounded layout/parser safety

Orders **6,831-6,852** exact-read the pinned MicroTeX unit/color, box/layout, formula/environment and glue implementation contracts. Invalid fixed-table indices and non-finite numeric input must fail closed; layout/repeat/matrix growth is bounded; partial-parse and exceptional paths preserve deterministic later renders. These responsibilities stay in the existing canonical ConversationWorkspace/Transcript KaTeX owner plus Build/Release provenance and do not justify a MicroTeX runtime. The canonical owner adds fail-closed non-finite custom budgets and focused output/cache lifecycle tests. Reading does **not** reduce `unknown`. Accounting: **6,852/16,125 read; 9,273 unread; 15,845 unknown; 0 omitted**. Next: **6,853 `Telegram/ThirdParty/MicroTeX::src/core/localized_num.cpp@4f8e61fcc0f6f3c6d361cd835b4ceaa3a4ae5b83`**.

### Recursive component read-through 6,853 — MicroTeX localized decimal input

Order **6,853** exact-reads `Telegram/ThirdParty/MicroTeX::src/core/localized_num.cpp@4f8e61fcc0f6f3c6d361cd835b4ceaa3a4ae5b83` from pinned `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`. Its applicable responsibility is deterministic numeric input normalization: U+066B becomes the ASCII decimal point and the twenty accepted Unicode decimal blocks map digit-for-digit to ASCII while every unrelated code point remains unchanged. The existing canonical ConversationWorkspace/Transcript KaTeX owner performs that normalization before both strict and recovery renders, and the focused Desktop Chat Parity contract covers all twenty blocks, unchanged non-target text, and identical strict/recovery input. No MicroTeX parser/runtime is retained. Reading does **not** reduce `unknown`; descendant exact-head GitHub Actions remain required. Accounting: **6,853/16,125 read; 9,272 unread; 15,845 unknown; 0 omitted**. Next: **6,854 `Telegram/ThirdParty/MicroTeX::src/core/macro_def.cpp@0ccf9fe8b4028f97c62834555d2a3d344d0f8726`**.

### Live rebaseline 28ac5769 → 6fed91ff

Accepted upstream advanced by four commits to `6fed91ffab9861771a75f29df65031b3c80b6941` / root tree `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6` without adding or removing a root path. Ten root blobs changed at deterministic orders **3,155, 3,201, 3,202, 3,766, 4,521, 4,522, 5,166, 5,943, 6,541, 6,627**. Each changed blob was re-read against its predecessor object and reconciled through the explicit source-disposition rebaseline record. Version/resource/build changes preserve source-neutral release-version coherence. The history-photo delta adds owned enlarge eligibility/hit-testing plus press/release ripple lifecycle and keeps sponsored/editor/small-media exclusions; it maps to the existing Transcript/media interaction owner rather than a new media root. The media-viewer delta explicitly claims Save/SaveAs at the application event-filter boundary so competing shortcuts cannot consume Ctrl+S; it maps to the existing canonical media-viewer shortcut owner. The new style opacity is presentation-only under the Fabushi design system. The 7.3.1 changelog adds wallet Windows Hello/passcode, copy-restricted download, custom-theme date-color and photo-editor Link regression cues; those cues remain open until their existing Fabushi owners have shipping evidence. Recursive totals/order are unchanged, so read-through remains **6,853/16,125**, first unread remains **6,854**, unread **9,272**, unknown **15,845**, omitted **0**.


### Read-through 6862-6872 — MicroTeX font lifecycle

Exact component blobs cover Unicode classification, glyph metrics/extensions, safe missing lookup, font/style/symbol registration, global sizing reset boundaries and build composition. Fabushi keeps one source-neutral math owner: the existing Transcript math wrapper and shipped KaTeX dependency plus Build/Release provenance. No MicroTeX font runtime is ported. These rows remain mapped-open until focused production behavior and descendant exact-head Actions verify applicable fallback/isolation/build contracts. Accounting: **6,872/16,125 read; 9,253 unread; 15,845 unknown; 0 omitted**; first unread is **6,873 `Telegram/ThirdParty/MicroTeX::src/fonts/tex_font.h`**.


### Read-through 6873-6892 — MicroTeX rendering boundary

TeXFont/graphics/resource/platform/render sources are exact-read. Existing-owner-first maps them to the sole Transcript math wrapper, Electron/DOM rendering boundary and Build/Release provenance. Applicable invariants include clamping non-finite color inputs, saturating huge raster bounds instead of integer overflow, restoring transform/color after draw failures, deterministic font/resource lookup failure and single init/release ownership. Platform-specific Cairo/GDI/Qt/Skia code is evidence, not target architecture. Accounting: **6,892/16,125 read; 9,233 unread; 15,845 unknown; 0 omitted**.


### Revision 9 exact read-through 6893-6900 — MicroTeX builtin resources and font provenance

Orders **6,893-6,900** are exact-blob read/decomposed for Unicode/codepoint→formula normalization, symbol/text mappings, TeX spacing/layout constants, builtin symbol taxonomy, Meson resource closure, CTAN font provenance, and the first `bi10`/`bx10` font metric/style definitions. Existing-owner-first keeps one canonical Transcript math/KaTeX owner plus Build/Release provenance; no MicroTeX parser/font/resource runtime is introduced. These responsibilities remain **mapped-open**: source reading does not establish production equivalence, and `unknown` therefore stays **15,845**. Accounting: **6,900/16,125 read; 9,225 unread; 15,845 unknown; 0 omitted**. First unread: **6,901 `Telegram/ThirdParty/MicroTeX::src/res/font/cmbsy10.def.cpp@6e72926a0156f2b06033d9995c7653b5d76b2a10`**.


### Revision 9 exact read-through 6901-6910 — MicroTeX math font metrics and fallback graph

Orders **6,901-6,910** exact-read `cmbsy10`, `cmbx10`, `cmbxti10`, `cmex10`, `cmmi10` (+ unchanged compatibility variant), `cmmib10` (+ unchanged compatibility variant), `cmr10` and `cmss10`. The files carry glyph metrics plus family/style fallback edges, larger-glyph and delimiter-extension chains, ligatures, kerns and skew metadata. Existing-owner-first keeps the sole AssistantMath/KaTeX shipping owner and Build/Release provenance; no MicroTeX font runtime or second font truth is added. All remain **mapped-open**, and source reading does not reduce `unknown`. Accounting: **6,910/16,125 read; 9,215 unread; 15,845 unknown; 0 omitted**. First unread: **6,911 `Telegram/ThirdParty/MicroTeX::src/res/font/cmssbx10.def.cpp@8b1c030f0123c71809eda15a48314298475300e7`**.


### Source closure through 6,921

Orders **6,911-6,921** are exact-blob read-complete/responsibility-decomposed for MicroTeX sans-bold/sans-italic, math-symbol, text-italic/mono, double-struck, Euler Fraktur, base italic metrics and the exact font Meson build closure. Metrics, family/style fallback, larger-glyph, ligature, kern/skew and math-alphabet semantics remain mapped-open to the existing canonical AssistantMath/KaTeX dependency interface plus Build/Release provenance owner; no second MicroTeX/font runtime is introduced. Accounting: **6,921/16,125 read; 9,204 unread; 15,845 unknown; 0 omitted**. First unread is 6,922 `Telegram/ThirdParty/MicroTeX::src/res/font/moustache.def.cpp@629e014e…`. Reading grants no implementation, verification, baseline or release credit.


### Source closure through 6,932

Orders **6,922-6,932** are exact-blob read-complete/responsibility-decomposed across moustache extensible delimiter pieces, AMS symbol/blackboard fonts, roman/sans compatibility variants, RSFS script kern data and special/sans glyph metrics. They stay mapped-open to the existing AssistantMath/KaTeX dependency interface plus Build/Release provenance owner; no second font runtime is introduced. Accounting: **6,932/16,125 read; 9,193 unread; 15,845 unknown; 0 omitted**. First unread is 6,933 `Telegram/ThirdParty/MicroTeX::src/res/font/stmary10.def.cpp@c687e395…`. Reading grants no implementation or release credit.


### Source closure through 6,948

Orders **6,933-6,948** are exact-blob read-complete/responsibility-decomposed across StMary/mono typography, font/symbol definition macros, dependency registries, resource Meson closure and AMS/base/StMary/special symbol mappings. They remain mapped-open to the existing AssistantMath/KaTeX dependency interface plus Build/Release provenance owner; no second registry or MicroTeX runtime is introduced. Accounting: **6,948/16,125 read; 9,177 unread; 15,845 unknown; 0 omitted**. First unread is 6,949 `Telegram/ThirdParty/MicroTeX::src/samples/gtkmm_main.cpp@d491af61…`. Reading grants no implementation, verification, baseline or release credit.


### Source closure through 6,962

Orders **6,949-6,962** are exact-blob read-complete/responsibility-decomposed for MicroTeX GTK/memory-check/Qt/Skia/QML/Win32 samples. Platform demo UI is not copied; applicable single-owner init/parse/render/export/failure/teardown semantics map to existing AssistantMath/KaTeX, test and Build/Release owners. Accounting: **6,962/16,125 read; 9,163 unread; 15,845 unknown; 0 omitted**. First unread is 6,963 `Telegram/ThirdParty/MicroTeX::src/utils/dict_tree.h@314b8eca…`. Reading grants no implementation or release credit.


### Source closure through 6,976

Orders **6,963-6,976** are exact-blob read-complete/responsibility-decomposed for MicroTeX utility and dependency boundaries. Binary lookup must remain one-past-end safe; byte ctype classification must use unsigned-byte domain; malformed UTF/numeric/resource input must fail locally; typed error distinctions and dependency provenance remain explicit. C++ utilities are not copied. Accounting: **6,976/16,125 read; 9,149 unread; 15,845 unknown; 0 omitted**. First unread is 6,977 `Telegram/ThirdParty/QR::Readme.markdown@66292b4b…`.

### Source closure through 7,028

Orders **6,977-7,028** are exact-blob read-complete/responsibility-decomposed for the pinned Nayuki QR component across C/C++, Java/fast-Java, Python, Rust/no-heap, Rust and TypeScript/JavaScript implementations, tests, demos and package/build inputs. Applicable invariants include versions 1-40, four ECC levels, numeric/alphanumeric/byte/Kanji/ECI segmentation, bounded capacity/version selection, optional ECC boost, mask selection/penalty scoring, Reed-Solomon block/interleave behavior, bounded module access, typed too-long/invalid-input failure, no-heap caller-buffer contracts and output border/scale bounds.

Existing-owner-first audit rejects a new QR product/state subsystem: RemoteControl already owns pairing/manual code, auth, expiry, status and persistence; SharedRoom owns share URL/expiry/room lifecycle; DeepLink owns canonical URL parsing. QR is mapped-open as a stateless representation/codec boundary behind those owners plus canonical Resource presentation. Reading grants no implementation, verification or unknown-closure credit.

Accounting: **7,028/16,125 read; 9,097 unread; 15,845 unknown; 0 omitted**. First unread is order 7,029 `Telegram/ThirdParty/TooManyCooks::.clang-format@b484c8323870e494c8031a35ddac49aa114052b3`.

### Source closure through 7,056

Orders **7,029-7,056** exact-read TooManyCooks formatting, CI, project authority and package/build contracts. The source authority requires sanitizer/platform/topology coverage, cold task semantics, executor/priority inheritance and resume affinity, mandatory await/detach linear lifetime, fork/join lifetime, coroutine-lambda capture safety, work-stealing/topology behavior and build-option consistency. These remain mapped-open to existing Fabushi Coordinator/Host/Runner/ConversationActor/CapabilityBroker plus Build/Release owners; no TMC provider/runtime is introduced. Reading closes no unknown.

Accounting: **7,056/16,125 read; 9,069 unread; 15,845 unknown; 0 omitted**. First unread is order 7,057 `Telegram/ThirdParty/TooManyCooks::include/tmc/all_headers.hpp@9f14633b57c5d4b319d1309b4bc6db643bc1822d`.

### Revision 9 deterministic read-through — TooManyCooks 7,057–7,154

Pinned authority `tzcnt/TooManyCooks@b86af81982860c96295a7e95e4c60cb335615cef` is now exact-read through all **126** non-directory entries. Orders **7,057–7,154** revalidate the prior temporary 7,057–7,064 read and cover all remaining runtime headers, implementations and package inputs.

Existing-owner-first audit found real shipping candidates: `GatewayHostSupervisor`; `HostRunnerComposition`; the frozen Host→Runner bridge; `RoutedProviderCancellation/RoutedProviderTaskRegistry`; `TurnSettlement`; canonical Mahayana messaging; `PressureCpuProfiler`; and existing GitHub Actions Build/Release owners. Applicable/open behavior includes executor/priority restoration, awaitable/fork/join lifetime, UAF-safe settlement, atomic wait/wake and teardown, foreign callback lifetime/cancellation, queue close/reclamation, CPU quota/topology/work-stealing capacity behavior, and sanitizer/fuzz/coverage/build-option consistency. Exact hwloc pinning is not presumed complete without a dedicated shipping worker-pool owner. vcpkg/CMake/version descriptors are build/provenance inputs, not a new product owner.

No TMC/Telegram runtime/provider or duplicate Identity/Conversation/Message/Search/Settings/Marketplace is introduced. Accounting is **7,154/16,125 read; 8,971 unread; 15,845 unknown; 0 omitted**. Reading closes no unknown. Next deterministic entry is **7,155** `Telegram/ThirdParty/cld3::.github/workflows/main.yml@b26ff5ec0300683c819c0c7f17edfc21bfdd0fc3` in `google/cld3`.

### Revision 9 deterministic read-through — CLD3 7,155–7,241

Pinned `google/cld3@b48dc46512566f5a2d41118c8c1116c4f96dc661` is exact-read through all **87** non-directory entries. The component implements local language identification: bounded input selection; UTF-8/interchange repair and script spans; character n-gram + relevant-script features; embedded feed-forward neural inference; `language / probability / is_reliable` results; multilingual, empty, invalid-UTF8 and script tests; generated model parameters; and build/binding/license inputs.

Consumer tracing in accepted Telegram source finds `Platform::Language::Recognize` used by translation offer/skip policy, chat translation tracking, AI compose default source language, URL translation and Linux translation-provider flows. Current Fabushi shipping tree contains no `language/translate/spellcheck/i18n` path. Prior dossiers only define a **Canonical Translation service gateway** as mapped-open. Therefore no existing production owner can honestly be claimed.

Owner-absence decision: reject Search, Settings/Localization and Message as detector owners because they consume/filter/project language state but should not own classification. If implementation proceeds, create only an approved minimal source-neutral `TextLanguageClassifier` contract behind the canonical Translation gateway; it must be local/private by default, typed unknown/unreliable on bad/short/mixed input, bounded for long text, model/version/license pinned, and covered by multilingual + Unicode/property tests. This paragraph is an ADR requirement/proposal, not approval to add a CLD3 runtime/provider.

Accounting: **7,241 read; 8,884 unread; 15,845 unknown; 0 omitted**. Next is **7,242** `Telegram/ThirdParty/cmark-gfm::.editorconfig@12032e647502de7a91f241f75fcd7666be375875`.


### Revision 9 deterministic read-through — kcoreaddons file/format/Search/job tests 9,141–9,148

Orders 9,141–9,148 were read by exact `KDE/kcoreaddons@fd84da51b554eac25e35b1e3f373edaab3029b15` blob identity. They recover collision-safe filename/resource discovery, locale-aware byte/value/duration formatting, Unicode/RTL/separator fuzzy matching and ranking, and asynchronous job result/progress/cancel/destroy/exactly-once lifecycle semantics. Existing-owner audit maps these into canonical Resource/localization, Command Palette Search and Task/Coordinator/Host owners. The Search owner already implements source-neutral Unicode normalization and fuzzy scoring; this descendant adds focused RTL/diacritic/separator/non-match coverage. No KDE runtime, parallel Search or KJob-style Task runtime is introduced. All eight source rows remain mapped-open; reading and a focused test do not mark the broader responsibility verified.

Accounting is **9,148/16,125 read; 6,977 unread; 15,845 unknown; 0 omitted**. First unread is order 9,149 `Telegram/ThirdParty/kcoreaddons::autotests/klibexectest.cpp@6bc9bd18ab8f62711448a26b7057b9c813281f87`. Fresh descendant exact-head GitHub Actions are mandatory.


### Revision 9 deterministic read-through — kcoreaddons platform/plugin contract tests 9,149–9,168

Exact reads cover packaged helper discovery, typed open-file diagnostics (including explicit Windows unsupported behavior), shell-safe macro interpolation, non-null memory telemetry, canonical/slow network-path and symlink policy, OS release metadata, and plugin factory/loader failure/lifecycle behavior. These responsibilities are mapped existing-owner-first into Host/Runner, Resource/platform, Settings, updater/diagnostics, and the canonical Plugins/MCP/Marketplace runtime. No KDE process helper, second filesystem policy, or second plugin loader is introduced. The upstream tests are behavioral authority only; these 20 rows remain mapped-open.

Accounting is **9,168/16,125 read; 6,957 unread; 15,845 unknown; 0 omitted**. First unread is order 9,169 `Telegram/ThirdParty/kcoreaddons::autotests/kpluginmetadatatest.cpp@2a81878f535ed71fcf76688acf413a333ddff214`. Fresh descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — kcoreaddons plugin/process/platform/shell contracts 9,169–9,178

Exact reads cover plugin metadata identity and locale/filter behavior without executing plugin code; process enumeration and invalid-PID behavior; stdout/stderr/exit channel lifecycle; random-string uniqueness and deterministic test-only shuffling; typed runtime-platform labels; shared-cache insert/overwrite/read semantics; and cross-platform shell quoting/splitting with explicit meta-character rejection. Existing-owner audit maps these into canonical Plugins/MCP/Marketplace, Electron/Host process diagnostics, Host local-exec/Runner + permission gating, platform capability, and domain cache/persistence owners. The canonical plugin manifest now carries normalized optional publisher-declared license metadata and exposes it through local plugin/read; this is a real production increment, not a second plugin loader. Shell-string safety and the broader metadata/process/cache responsibilities remain mapped-open.

Accounting is **9,178/16,125 read; 6,947 unread; 15,845 unknown; 0 omitted**. First unread is order 9,179 `Telegram/ThirdParty/kcoreaddons::autotests/kstringhandlertest.cpp@cceb3d1651308336a0c89f9fb900a1542d170230`. Descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — kcoreaddons text/resource/identity/plugin fixtures 9,179–9,197

Exact reads cover multilingual string wrapping/elision, plain-text linkification and HTML-injection rejection, URL/clipboard resource metadata and local-vs-canonical URL preference, local OS user/group identity and invalid-safe access, plus static/dynamic, metadata-less and versioned plugin factory fixtures. Existing-owner audit keeps these in canonical Transcript/external-link, Resource/attachment transfer, platform/Computer identity and Plugins/MCP/Marketplace owners. Transcript text/rich-text shipping projection uses React nodes rather than raw HTML and normalizes external HTTP(S)/mailto links; a descendant focused contract adds the upstream malicious URL forms and forbids `dangerouslySetInnerHTML`. This closes a concrete security replacement, not the broader formatting/linkification responsibility.

Accounting is **9,197/16,125 read; 6,928 unread; 15,845 unknown; 0 omitted**. First unread is order 9,198 `Telegram/ThirdParty/kcoreaddons::cmake/FindFAM.cmake@fc8fc967758a27a7e38e9b98efbf5d629d65e0a0`. Descendant exact-head GitHub Actions are mandatory.

### Revision 9 deterministic read-through — kcoreaddons build/platform provenance 9,198–9,201

Exact reads cover optional FAM file-watch and FreeBSD procstat dependency discovery, docs-only deprecated API preprocessing, and framework platform/release/library metadata. Existing Fabushi build, platform-capability, process-diagnostics, file-watch and packaging owners remain canonical; no optional KDE-era dependency is adopted merely because upstream can discover it.

Accounting is **9,201/16,125 read; 6,924 unread; 15,845 unknown; 0 omitted**. First unread is order 9,202 `Telegram/ThirdParty/kcoreaddons::poqm/af/kcoreaddons5_qt.po@cfda626078de334eeecd25fa310e8fffd4b513cd`. Descendant exact-head GitHub Actions are mandatory.
