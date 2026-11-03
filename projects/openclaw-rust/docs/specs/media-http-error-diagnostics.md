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

## Aggregate remote-request deadline — MHT-v1

Implementation base is `1b9803de2283322a7809dc626a5ca35a52eb834e` after MURL
integration. The publication parent advances to
`0d6acd9bf4b9725bc2030c785db5064d277a68cd` (tree
`a7a8f97c57f1b38fc5efecf981b553c443da592d`), preserving the incoming
Automation, model/Config, native preview, Memory and Unicode work. Its shared
workflow and ledger changes are retained without promoting any global flag.
**MHT-01 — FIX / IMPL_BUG**: MHE's bounded diagnostic reader calls
blocking `Read` repeatedly. Locked reqwest `0.12.28` restarts the blocking
client wait for each read, whereas the earlier buffer-error `text()` wrapped
one complete body future. A trickling peer below the diagnostic byte budget
can therefore keep the operation alive beyond the configured timeout. The
save path already had this aggregate-deadline gap; the buffer error path
regressed in MHE. Do not undo bounded diagnostics to restore the old timing.

Reuse reqwest's existing per-request `.timeout(effective_timeout)` on both
`read_remote_media_buffer_with_headers` and `save_remote_media`. Resolve the
same existing `timeout_ms`/30,000 ms default once and apply it to the client
wait and request. This intentionally sets one connection-through-body budget,
including redirects and successful responses. It tightens previous split
header/body/per-read timing; it is not a claim of unchanged timeout semantics.
No new worker, timer, runtime, retry, option or dependency is introduced.

Required behavior:

- MHT-01: trickled non-success bodies cannot reset the total request budget.
  Once the deadline expires, preserve the already known HTTP status and omit
  the unavailable snippet through the existing error/service boundary.
- MHT-02: timely complete/chunked successful responses retain exact bytes,
  metadata and store behavior. An incomplete successful body at the total
  deadline returns the existing fetch error and publishes no partial file.
- MHT-03: retain MHE exact-cap/Unicode/redaction controls, MURL initial and
  redirect admission/caps, byte bounds and unchanged owner state. Dropping
  the response/request-local client retains existing lifetime ownership.

Extend the existing owned HTTP fixture and real service controls. Supply
headers promptly, trickle bytes with observed gaps below the configured
timeout and keep EOF/suffix behind a fixture-owned gate. Require an actual
status-preserving result before that gate opens; a harness receive timeout is
failure. Record progress so a stalled server cannot masquerade as the total
deadline. Release/join every owned caller and server on failure. Retain a
timely multi-chunk success control and unchanged-store failure controls.

The exact dependency API review uses reqwest release
`d97859910c357827ad5993d37ce750ad595f4fff`: blocking response blob
`19fb7fe203fbd86266ad6592a4476a28cb4398ea`, blocking request blob
`7ac08a768fd31c42fed77b6ee3eb787abb6eb6f5`, blocking client blob
`255d9d8aa4fb2318cb923e6ff4e3db994b781b4d`, and async client blob
`d6dc03ad346c5f78a532f07f73ad99867f848145`, and error Display blob
`1128d633e57d6f7f614c331be1cf86f06ea7f548`. The service exposes the existing
body/decode error category; timeout causality comes from actual pre-EOF
settlement with continuing observed chunk progress, not a nested Display
message. This is a timeout/consumer contract
review, not complete dependency source or archive acceptance.

Root owns the production owner, spec/workflow/ledger changes; a separate worker
may edit only the existing media-fetch contract test file against this contract.
Freeze and independently review before publication; require exact-H Actions
named positives and existing media regressions. No local execution. Caller
cancellation, separate header/read-idle budgets, proxy/DNS/IP security,
outer HTTP/desktop consumers, real services and full migration acceptance
remain open. Rollback must retain the aggregate deadline or disable the
affected remote entry; no stored format or data migration changes.

**MHT-ORACLE-01 — FIX / TEST_FLAW.** Independent review of frozen tree
`fc52fc3aa4e954761261693a673bcf5baed715b3` found that the timely-body control
asserted the buffer result's `file_name`/`content_type` keys for both methods.
The existing saver serializes `SavedRemoteMedia` as `fileName`/`contentType`.
Keep those product wire contracts unchanged; assert each method's actual
metadata keys in its own branch, retaining exact byte/size/store assertions.
This source-proven oracle correction requires a fresh focused review and
exact-H Actions execution; it is not a product compatibility change.

**MHT-MERGE-01 — FIX / integration preservation.** The incoming native preview
implementation supersedes the local unpublished LPR prototype; preserve its
single registry/carrier, shared service, Host-lane isolation and preview mutex.
Retain every incoming MRU, Memory, Config, Automation and preview command and
positive-result gate. The overlapping media receipt labels/scopes are the union
of MRU and MHT, with neither one's execution replaced or relabeled. Interrupted
review/large-blob submission is not approval or delivery; freeze the reconciled
tree and obtain fresh review. The large ledger blob is delivered only through
the repository's existing exact-object Actions carrier after an unconfirmed
large connector submission; that carrier is not product verification.
