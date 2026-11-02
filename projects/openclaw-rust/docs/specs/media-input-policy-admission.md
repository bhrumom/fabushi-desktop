# Media input policy admission — MIP-v1

Status: bounded production correction; exact-subject Actions and full media
acceptance remain pending. This contract belongs to
`OCRM-RESP-MEDIA-INPUT-FILES-EXTRACT-001` / `OCRM-UNIT-MEDIA-014`.

## Authority and defect

Fixed U remains `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`
(tree `90c8aab39972da11cfad220361781d49e1bc6700`). The target baseline is
`3c4bf6963cb30ecf01eb34b905315074886e1972` (tree
`313daf015446a9abd40a8a6f1a5d7a5892542bf6`).

At that baseline, both file-extraction RPC branches deserialize explicit
`limits` with `unwrap_or_else` and replace any invalid policy with defaults.
For example, a malformed PDF field silently discards an otherwise valid
`allowUrl=false`, empty MIME set, byte cap or zero character cap. The shared
base64 size estimator also subtracts padding without a lower bound: `=` and
`==` can panic before the decoder rejects them.

The fixed-U bounded review covers the resolved limits/resolver,
`readInputSource`, and file/image extraction callers in
`src/media/input-files.ts` (blob
`64bca417aa7b46c02d11d81f1b8a5b6f906137cd`), plus the imported estimator and
canonicalizer in `packages/media-core/src/base64.ts` (blob
`31eead019c4aeecd081ba7836e7ec408a367ae1c`). The package export is recorded by
`packages/media-core/package.json` (blob
`88490cbd64f5cddd4181f053bb029da78d272885`). Upstream uses supplied resolved
limits and clamps the estimated decoded length to zero before invalid-data
rejection. Its base64 controls are in `packages/media-core/src/base64.test.ts`
(blob `337c775bd1723f7bfba87e3d0275a81798857fe9`) and the base64 section of
`src/media/input-files.fetch-guard.test.ts` (blob
`645719e0cc5822b85f4431f1ef55899e2aafb0c2`). These are bounded contract and
implementation reviews at current U; they do not relabel historical reads or
establish whole-unit source closure.

## Existing production owner and behavior

The producer is an existing attachment gateway call. `AttachmentsService`
owns JSON admission; `input_files` owns extraction and the shared file/image
source decoder; URL acquisition stays in the existing `media_fetch` owner.
The consumer is the existing RPC result. No new module, persistent state,
scheduler, permission authority, or runtime is introduced.

| ID | Required behavior and observable result |
| --- | --- |
| MIP-01 | `extractFileContentFromBuffer` and `extractFileContentFromSource` accept omitted or JSON-null `limits` as the existing default-policy request. Every other value must be a JSON object that deserializes as complete `InputFileLimits`; scalar, sequence or malformed explicit values return `Invalid InputFileLimits` before payload decoding or URL processing. |
| MIP-02 | Valid resolved limits pass unchanged to the existing extractor. `allowUrl=false`, an empty MIME set, a restrictive byte cap and `maxChars=0` remain effective. Partial configuration continues through the existing `resolveInputFileLimits` API. |
| MIP-03 | One- and two-padding-only base64 (`=` and `==`), including surrounding whitespace, reaches invalid-data rejection without arithmetic panic or underflow-generated size errors, through both file and image extraction. Other malformed inputs retain the upstream size-before-validation order; valid padded data retains its byte-cap behavior. |
| MIP-04 | Rejection produces no file write, staging, upload, provider invocation or network request. The operation is synchronous and has no admitted durable work to recover after cancellation or restart. Existing successful extraction and downstream network lifetime rules are unchanged. |

The image RPC already rejects invalid resolved limits. This batch preserves
that behavior and applies the shared estimator correction to its source path.
Resolved policy validation does not add a caller authorization grant.

## Verification selection and limits

Extend the actual `AttachmentsService::dispatch_gateway` fixtures in
`source/host/tests/openclaw_input_files_contract.rs`; do not replace the owner
under test with a parser mock. Required controls include malformed scalar,
container, missing-field and nested-field policies; invalid payload/URL
sentinels proving policy error precedence; omitted/null defaults; valid
restrictive policies; and malformed/valid padded file and image sources.
Existing controls remain selected. Execute the affected integration target in
the existing `rust-desktop-runtime.yml` scoped Actions lane at frozen H and
retain its command log, exit status and exact-subject receipt. Also select the
same target in the existing Linux native-path job, reusing its compiled Host
cache, with a separate receipt that requires the named MIP controls. This
provides direct Linux evidence while the earlier CWR macOS job has no assigned
runner; the queue cause is unexposed. Linux success does not substitute for
macOS Host execution or other platform acceptance. No new job, workflow,
trigger, or local test/build/lint/generator/schema execution is required.

This closes JSON policy admission and padding arithmetic only. It does not
establish full URL hostname-allowlist/redirect/cancellation parity, real PDF
extraction, real HEIC conversion, or accepted-turn multimodal composition.
`BL-MEDIA-INPUT-FILES-CHAT-WIRING-001`, full source authority, real consumer
acceptance and all global completion gates remain open. In particular, the
existing HEIC fixture uses a placeholder conversion and is not real-format
compatibility evidence. No verification, acceptance or global flag may be
promoted merely because this scoped target passes.

## Review decision before correction

MIP-ORACLE-01 — **FIX / TEST_FLAW (evidence-scope overclaim)**. Two independent
readers of frozen tree `1abbe9e5db9d58555e9b47a46b29a59ef1a9c980` identified
that the general phrase "padding-only" in MIP-03 and its evidence could imply
an invalid-data error for `====` at `maxBytes=0`. Both Rust and fixed U instead
estimate one byte and reject the size before decoding. Fixed-U
`base64.ts::estimateBase64DecodedBytes` clamps the subtraction, while
`input-files.ts::readInputSource` preserves this size-before-validation order.
The broader promise was introduced by this batch, not required by the user or
upstream. Retain the production arithmetic and all existing assertions;
state the one-/two-padding underflow guarantee precisely and add explicit
longer-padding size-error controls. This does not waive broader base64 parity.
No executable check has run or failed; the retained positive evidence is the
independent source review of policy admission, ownership, valid byte caps and
Actions wiring. The overbroad receipt/spec claim is invalidated. A fresh
focused review must inspect the corrected description and added oracle.
