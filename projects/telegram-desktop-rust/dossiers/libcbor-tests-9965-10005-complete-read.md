# libcbor tests — deterministic orders 9,965–10,005 complete read

Authority: `PJK/libcbor@170bee2b82cdb7b2ed25af301f62cb6efdd40ec1`, mounted at `Telegram/ThirdParty/libcbor`.

The exact recursive identities were generated and range-attested at `f936a5e200349d31bff3a64e9659187fb7680947` in run `38118212864`, source-authority job `114407396444`, artifact `11693652953` (digest `sha256:e9307971683ac2f1920de0aed9d9cd9f9be0a4dbfd0f2d31910459cd80878970`). That job later failed only on the strict `inventory/index.json.note` wording contract, before current-window materialization. The source identities themselves were attested, and every file in this range was read directly from the exact upstream commit; binary `.cbor` fixtures were fetched as exact base64/raw-byte content rather than treated as text or omitted.

This component tail is primarily adversarial test authority. It verifies typed incomplete/malformed/memory/syntax errors, exact error positions, exact streaming `required` bytes, reserved encodings, allocation failure at individual nested-copy/build stages, reference-count stability after failed operations, no-space and recursive serialization overflow, `SIZE_MAX` add/multiply/reallocation boundaries, parser nesting beyond `CBOR_MAX_STACK_SIZE`, invalid/truncated UTF-8, indefinite collection state machines, and randomized/fault-injected decoding. These responsibilities map into the existing Coordinator WebAuthn provider/signing lifecycle plus canonical Account/Auth security and Build/Quality/Release; they do not justify a second CBOR/FIDO runtime.

At `f936a5e…`, the independent Mahayana Coordinator contract step already passed with the bounded WebAuthn helper-output hardening present. Final verification still requires the whole descendant exact-head Rust workflow to succeed after the source-authority narrative fix.

All 41 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`. Order 10,005 closes the pinned libcbor component; order 10,006 begins Yubico/libfido2.
