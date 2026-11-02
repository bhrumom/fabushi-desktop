# Bounded media HTTP error diagnostics — MHE-v1

Status: bounded existing-owner correction; exact-H Actions and whole-media
acceptance remain pending. Responsibility `OCRM-RESP-MEDIA-FETCH-PIPELINE-001`,
unit `OCRM-UNIT-MEDIA-012`. Base H is
`4e90f031aeeedb50efe4e265da084b99901478cd`, tree
`92ae0ff8441916395a33cdb38b31545c2d28e23c`. Fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.

Integration parent advances to
`b5b5737edf2b9108221036368ee113859dfa7364` (tree
`787bf4782245fb03b02871be0e3b2cc0695dad91`), preserving its portable channel
recovery Actions and evidence contract. MHE implementation/test blobs remain
the reviewed candidate; only shared workflow integration and this parent
record change from the first frozen snapshot.

## Defect, authority and ownership

The actual AttachmentsService `readRemoteMediaBuffer` gateway calls
`media_fetch::read_remote_media_buffer_with_headers`; `saveRemoteMedia` calls
`save_remote_media` and `save_response_media`. File/image URL extraction also
consumes the buffer owner through `input_files::read_input_source`.

These are actual service-dispatcher/owner calls, not proven shipping HTTP
routes. Follow-up tracing at base H found that `readRemoteMediaBuffer`,
`saveRemoteMedia` and the input-extraction methods are absent from the outer
Host method registry. MHE-v1 is the necessary shared-owner failure contract;
its fixtures do not establish desktop/HTTP reachability or open those methods.

On a failed HTTP response the buffer owner calls `res.text()` before limiting
the diagnostic, then slices a Rust string at byte 8192. This can consume the
whole body and panic when the cut crosses a Unicode scalar. The save path
reads the success payload budget before diagnosing failure, and supplied
response bytes are converted/formatted in full. Body detail bypasses the
existing media redactor. Decision **MHE-01 — FIX / IMPL_BUG**: correct these
adjacent error branches within this owner, preserve success-body behavior and
use actual response/Gateway oracles.

The bounded current-U review covers these exact inputs:

| Source | Blob | Reviewed responsibility |
| --- | --- | --- |
| `src/media/fetch.ts` | `46efb618af1faa18ff8e5c19000ba93adf8470a0` | `assertMediaResponseOk`, supplied/fetched consumers, status-before-success-limit and release boundary |
| `src/infra/http-body.ts` | `9a50fe13e011589e8930c588eb4dd27c24556190` | Response-reader export |
| `src/infra/http-response-body.ts` | `8307879d96cd2470becc8a821231591a1c5af0a5` | Bounded prefix, normalization and diagnostic formatting |
| `packages/normalization-core/src/response-bytes.ts` | `9a31fdab468a3a858781e0a36e567fb9f83116a3` | Stop at a filled byte budget without another read |
| `packages/normalization-core/src/text-decoding.ts` | `85c128e632a1a97ed1bfa16c2896d25a525eace1` | UTF-8 prefix decoding and incomplete trailing sequence |
| `packages/normalization-core/src/utf16-slice.ts` | `8dc757cc3bf6e8e17e853badf17ca0aa82611ad4` | Surrogate-safe display truncation |
| `packages/normalization-core/src/index.ts` | `93a00e422cc308571d22aa819a2498105329146e` | Exported helper reachability |
| `packages/normalization-core/package.json` | `13606635cecef520cc2a8ebc4e10d7769559f59f` | Root and UTF-16 helper export contract |
| `src/media/fetch.test.ts` | `068ee0f431de2d1310370b6d88267f04ada2f2f0` | HTTP status and token-redacted diagnostics |
| `src/infra/http-body.response.test.ts` | `f32268148f7a57c9131acbc33b739ebe44e4d77a` | Whitespace, UTF-8/UTF-16, exact-budget and truncation controls |

These are scoped implementation/contract reviews, not whole-file read credit,
dependency closure or complete fetch equivalence. Reuse existing Rust standard
IO, UTF-8 decoding and media redaction dependencies. No new runtime, network
policy, storage owner, public option or dependency is required. The separate
`media_store::save_remote_media_for_store` has its own fetch path and is not
an affected caller; its broader contract remains open.

## Required behavior

| ID | Contract and observable oracle |
| --- | --- |
| MHE-01 | An obtained non-2xx response returns `MediaFetchErrorCode::HttpError` with the known remote status before success Content-Length/payload checks. Malformed wire headers rejected before a response exists remain transport failures. Existing 204 behavior is retained. |
| MHE-02 | Both remote entrypoints read/retain at most an 8192-byte diagnostic prefix at the owner's response-reader boundary and stop as soon as the budget fills. They do not await EOF or read a further body chunk to classify truncation. The response is dropped on return, releasing its existing request ownership; this does not claim a bound on transport-internal prefetch. |
| MHE-03 | For supplied failed-response bytes, slice the prefix before decoding or formatting. Body-derived work and allocations are bounded by that prefix; this does not bound bytes already supplied by a caller or the URL portion of the final error. |
| MHE-04 | Decode the prefix as UTF-8, dropping an incomplete trailing scalar when the byte budget fills and replacing malformed complete sequences. Collapse ECMAScript whitespace runs and trim; retain at most 200 UTF-16 units without splitting a surrogate pair. Append `…` when the byte/display budget truncates. Empty or whitespace-only detail omits `body:`. |
| MHE-05 | Apply the existing media redactor to source URL and diagnostic detail. Its currently supported bot-token pattern must not appear intact in the error. This does not claim general bearer/password/secret redaction parity. |
| MHE-06 | Diagnostic read failure keeps the known HTTP status and omits the unavailable snippet. All error paths leave the media store unchanged and propagate through the existing Gateway string-error boundary. |

The service converts the typed owner error to `MediaFetchError: <message>`;
the Host lane's dispatcher would wrap it as its existing internal Gateway
error once reached. This contract
does not change the outer Gateway status to the remote HTTP status. There is
no durable accepted work to resume after these read-only failures. Repeating
a read after restart uses the same existing owner and limits. No file may be
published for a failed response.

## Implementation and verification

Allowed implementation/test paths are `media_fetch.rs` and the existing
`openclaw_media_fetch_contract.rs`. Root serializes this spec, the narrow
existing dossier/ledger/RTM corrections, and current workflow selection. Keep
all prior controls and success payload/MIME/store behavior. Do not add a new
fetch injection, public bypass or second diagnostic owner.

Use actual `reqwest::blocking::Response` objects through an ephemeral local
HTTP server and the current exact `allowed_origins` option. Retain default
loopback refusal. The gateway JSON uses the existing snake_case option names.
For both remote entrypoints, hold the suffix/EOF behind an owned channel gate
after sending exactly 8192 bytes. Require the actual gateway error before
releasing that suffix; always release/join the exact fixture thread, including
failure cleanup. This proves bounded admission rather than a source-string
assertion. A timeout is only a deadlock safeguard, not a passing oracle.

Add controls for a Unicode scalar crossing byte 8192, 199 ASCII units followed
by an emoji at the display boundary, complete malformed UTF-8, whitespace-only
and empty bodies, redacted echoed token, a large declared response length with
a small success cap, malformed supplied headers, diagnostic read failure, and
unchanged store bytes. Retain valid successful fetch/save controls and existing
byte caps. A controlled HTTP fixture is not real provider acceptance.

Select the existing media-fetch target in the already used macOS scoped media
step and Linux Host-cache step, retaining all MIP/LPIP and Automation checks.
Bind actual named positives, exact H/tree/U, commands, exits, source/log digests
and immutable artifacts; missing positives fail. Freeze the full candidate,
obtain independent source review, push/read back and inspect exact-H Actions.
All executable validation remains Actions-only.

Outer HTTP registration and actual desktop/turn consumers, full SSRF/DNS/redirect/proxy policy, per-chunk idle timeout, retries, read-scope
authority, general redaction, MIME/format parity and accepted-turn chat wiring
remain open. Preserve `BL-MEDIA-FETCH-PIPELINE-CHAT-WIRING-001`, historical
source identities, all global flags and final G01–G12/AC01–AC24 requirements.
Rollback must preserve bounded safe diagnostics or disable the affected fetch
entry; no data migration or stored-format change is introduced.

## Review correction

**MHE-RTM-01 — FIX / IMPL_BUG.** Frozen tree
`1d124085130af87b519d5d79c73a3168b6e5bf57` added the existing media-fetch test
path twice to R24/AC24 evidence lists, violating their `uniqueItems` schema.
Remove only the newly duplicated entries and retain the new MHE spec refs;
all original tests, flags and assertions remain. The source-reviewed schema
defect requires this correction and a fresh focused review before Actions.
