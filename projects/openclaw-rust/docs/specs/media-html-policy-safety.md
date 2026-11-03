# HTML policy inspection safety and explicit loading — HHP-v2

Status: implementation contract; source review and exact-H Actions pending.
HHP-v2 continues the recovered HHP-v1 candidate without rewriting its original
source-reading facts. Current integration base is
`8cf6143501b8c69b051d98df05f3ed523b364268`; its relevant policy, web loader,
service and existing contract-test blobs are unchanged from HHP-v1's base.
The separately frozen media cleanup candidate is preserved by root integration.
Initial implementation base H `524889b3511f220233ee4948a81ddefc161d40b4`, tree
`7897575c3d087517c075cc1aae31e06b605be793`. Fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.
The initial HHP integration plan used the published media cleanup H
`cb62ae7c22a8264c49e90443e41cb6ad3c9b9a5d`, tree
`76767b81a11ccc00636bdf7ef4579c9229d27392`. The final immutable review manifest
binds the current integration base and full candidate. Preserve later media
platform corrections and concurrent Codex workspace binding; do not restore
an older owner, workflow or ledger snapshot. Both original media command
groups and their strict platform selections remain required.

## Problem and selected contract

The existing Rust `host_read_media_policy` owner indexes an arbitrary UTF-8
string at byte 8192 before inspecting HTML shape. A valid multibyte character
can cross that index and panic. Its byte window also differs from the fixed
upstream's 8192 UTF-16 code-unit window, and the current header expression
omits the upstream word boundary. The same owner selects provenance rows for
deletion on every filesystem inspection error, including errors that provide
no evidence that the file is absent or invalid.

Fix these necessary public policy contracts in the existing owner. The
generated-HTML producer, durable marker and subsequent read consumer remain
unimplemented in the real SendMessage/Session chain. A generic upload, local
path, tool argument, caller boolean, successful send or checkpoint must not
acquire generated-origin authority as part of this correction.

Source decisions: **HHP-F01 FIX / IMPL_BUG** for total bounded HTML shape
inspection; **HHP-F02 FIX / IMPL_BUG** for retaining uninspectable provenance
rows. The adjacent missing HTML header boundary is part of HHP-F01's complete
shape predicate, not an additional permission or parser owner.

## Fixed source and actual reachability

Fixed-U `src/media/host-read-media-policy.ts`, blob
`dfe3731a67d3f2d320c0bb67c3aa747cabdee909`, was read completely. Its shape
predicate trims ECMAScript leading whitespace, examines the first 8192 UTF-16
code units, and accepts a case-insensitive doctype/html header with the `iu`
word boundary or a closing html/body tag within that window.
`src/media/generated-html-provenance.ts`, blob
`d00a635b7ee764126236a7f7c4c32969acf3f893`, was read completely; pruning
retains rows on non-missing inspection errors. Its path-guard dependency uses
the actual locked `@openclaw/fs-safe@0.27.0` archive (SHA-256
`02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`).
The reviewed `package/dist/path.js` contract defines missing as `ENOENT` or
`ENOTDIR`. This bounded dependency review is not full native/legal closure.

Target policy blob `9901ae7acfbd9e7bb80d9bc8dd155435d86739a7` and existing
test blob `0cd4fd9d9c0e513308c005a38b10aef84c0d4acd` were read completely.
The policy has the existing `assert_host_read_media_allowed` consumer and a
direct Attachments service dispatch arm. That arm supplies no HTML trust or
explicit publication. The registered external Gateway table does not expose
that helper as a generated-HTML product flow. The provenance row helper has
no production caller. The real Runner SendMessage sink persists generic
attachments under Session custody, without producing a trusted HTML marker.
These limits remain part of acceptance and cannot be closed by helper tests.

## Requirements and ownership

| ID | Required behavior and causal oracle |
| --- | --- |
| HHP-01 | HTML shape inspection returns normally for every valid Rust string. Match the upstream's leading-whitespace and 8192 UTF-16 code-unit window. Boundary controls include ASCII, two/three/four-byte Unicode, a supplementary scalar crossing the UTF-16 limit, and a closing tag just within/after the window. |
| HHP-02 | Retain case-insensitive doctype/html header and closing html/body recognition. Require the upstream header word boundary; reject `htmlish`, `html_` and digit continuations, including the Unicode `iu` word-fold cases. Keep MIME, byte-hash/size, explicit-publication and missing-trust decisions in their existing owner. Exercise the actual public assertion path with authorized/unauthorized, malformed, hash/size and long Unicode controls. |
| HHP-03 | Provenance stale selection distinguishes observed invalid kind/link and confirmed missing path (`NotFound` or `NotADirectory`) from uninspectable errors. Other errors retain the original row. Selected rows preserve the complete input snapshot and order. The helper performs no database mutation, content publication or trust grant. |
| HHP-04 | Use real owned filesystem fixtures for ordinary files, missing leaf, a regular-file parent (`ENOTDIR`), directory and available Unix symbolic/hard links. Unix permission-denied and symlink-loop controls must actually encounter the required errors and retain rows, then prove ordinary recovery after repair. No skipped fixture may be reported as a positive. |
| HHP-05 | Preserve all existing three provenance tests and the existing host-read format regression. Add no parallel store, scheduler, permission truth or synthetic generated producer. Existing wiring blockers and source-reading history remain unchanged. |

The public interfaces remain `has_html_document_shape(&str) -> bool`,
`assert_host_read_media_allowed(...) -> Result`, and
`filter_stale_provenance_rows(&[GeneratedHtmlProvenanceRow]) -> Vec<_>`.
Only `host_read_media_policy.rs` owns these decisions. The bounded shape scan
may discard a dangling surrogate half at the UTF-16 boundary because none of
the ASCII-shaped tokens or word boundaries can be created by that half.
It must not allocate or scan the unbounded suffix to recognize a tag beyond
the permitted window. Leading-whitespace scan retains the upstream semantics.

The helper remains synchronous and keeps no pending job or new durable state.
Cancellation, database generations, conditional deletion of a newer marker,
lost commit acknowledgment, marker migration and real shutdown integration
require the missing production contract; this patch promises none of those
effects. Re-running the read-only selector after an inspection problem is
repaired evaluates fresh filesystem evidence. A retained row is never by
itself permission to read the file.

## Implementation and verification

Root records this contract before code. An isolated owner worker may edit only
the existing policy and provenance test target. Root serializes the existing
Runtime workflow, narrow OCRM/RTM evidence updates and cloud delivery after
the separately frozen media cleanup batch. No dependency or public schema is
needed. Preserve the historical dossier and literal reading observations.

Reuse the existing affected Host CI/cache and the complete
`openclaw_html_provenance_contract` target, plus the existing host-read format
admission regression. Extend the existing `capture_media_cleanup.py` with
the HTML policy and format groups, the actual web-media consumer group and
the adjacent IPB image-probe group, while retaining its two MSL commands and
all their strict names. The existing three media platform lanes provide the execution
entry; no second capture framework or workflow is needed. Retain exact
H/tree/U, command/capture exits, raw logs
and hashes, required named positives, platform/toolchain and source inputs.
Missing, duplicate, failed, ignored, zero-test or incomplete results must fail
the receipt. Unix error fixtures run on the actual unprivileged Actions
runner; lack of the expected real error is a visible fixture failure, not a
mock compatibility claim. Windows ordinary/ENOTDIR execution and Windows
reparse/hardlink provenance acceptance are distinct scopes.

Freeze the complete changed scope and obtain independent general/security
and pinned structure/reuse/performance review before publication. Execute
validation only in GitHub Actions, never locally or in htch-runtime. This
public-contract correction has no data migration or background work. Release
rollback uses the normal prior-artifact boundary; it must not make HTML trust
optional or reinterpret inspection failure as proven absence.

## Acceptance and remaining work

HHP-01..05 require reviewed implementation and actual exact-subject receipts.
No such result exists at contract creation. The existing responsibilities
`OCRM-RESP-MEDIA-HOST-READ-POLICY-001` and
`OCRM-RESP-MEDIA-GENERATED-HTML-PROVENANCE-001` keep their existing wiring,
verification and acceptance state until the corresponding evidence and scoped
authority exist. All eight global completion flags remain false.

The real trusted producer, current read capability, captured Session owner,
durable marker transaction, conditional stale-row deletion, cancellation and
shutdown fencing remain required follow-on implementation. Other host-read
decoder equivalence, platform filesystem coverage, actual UI/channel sends,
performance/soak, package/release and full G01–G12/AC01–AC24 remain open.

References: OCRM `SOURCE_OF_TRUTH.md`; main migration spec §17; existing
`media-store-read-scope.md`; `attachment-content-publication.md`;
`media-reference-unicode-safety.md`; and the historical provenance dossier.

## HHP-v2: explicit HTML through the existing real media loader

The production generic Host dispatch reaches `AttachmentsService::dispatch_gateway`
and its `loadWebMedia` arm, which calls `web_media::load_web_media` and returns
the actual result bytes. This is a necessary direct service contract. These
extension methods are not registered ordinary external Gateway methods; no
frontend, Runner SendMessage or channel consumer has been established for
this call. This batch does not claim those separate product paths wired.

The existing `WebMediaOptions.html_publication` field is currently unconsumed.
The fixed-U caller is `src/media/web-media.ts`, blob
`f48bca504f6fb77c3d094864eb656cb2b1bf25e2`: its explicit/local admission was
read at 650–756, trust routing at 141–259 and the final optional trust projection
at 588–596. These are partial file reads. The policy and provenance source
reads listed above remain complete; they are not widened to the other caller
or dependency files. The exact source reference/read-range assessment is
retained with this batch's evidence, separately from canonical inventory.
Pass the optional string `htmlPublication` from this existing service arm to
the existing loader. Only exact `"explicit"` plus a resolved `.html` or `.htm`
extension (ASCII case insensitive) selects the explicit HTML path. As in the
fixed-U strict equality check, absent, other strings and non-string service
arguments do not opt in. Non-HTML paths retain the existing behavior. A caller
flag is a content-publication request; it does not grant filesystem authority,
enable `host_read_capability`, bypass roots/limits or mint generated trust.

After the existing path checks and bounded actual file read, inspect the same
captured buffer using `media_store::detect_mime_from_buffer(buffer, None)`.
Passing no filename hint is essential: a `.html` suffix must not overwrite
observed PNG/PDF/ZIP or other recognized byte signatures. Pass that observation,
the resolved file path, actual buffer, `text/html` / `document`, no generated
trust and `explicit_html_publication=true` to the existing
`assert_host_read_media_allowed` owner. That owner consumes HHP-01/02's total,
bounded shape predicate. On refusal return `WebMediaError::LocalAccess` with
code `invalid-html-publication` and the existing policy error text. Never fall
through to image loading/optimization after an explicit HTML refusal.

On success return the original bytes, `contentType="text/html"`,
`kind="document"`, the existing original-filename projection, and no
`trustedGeneratedHtmlSource` value. Do not image-optimize, rewrite, persist,
register or render the HTML. The ordinary loader's existing non-HTML and
non-opted-in behavior, permission inputs and error behavior remain unchanged.
The existing detector recognizes a bounded set of binary formats; upstream's
pinned `file-type` coverage must be reconciled separately before claiming full
MIME-equivalence acceptance. This batch establishes the enumerated conflicting
signature controls and content/shape contract, not all-format sniffing parity.

| ID | Required behavior and causal oracle |
| --- | --- |
| HHP-06 | Actual loader accepts explicit `.html`/`.htm` (including uppercase and file URLs), returns original bytes and document MIME/kind, and keeps generated trust absent. Include long Unicode text crossing byte 8192 and valid final-window closing tags. |
| HHP-07 | Actual loader refuses malformed headers, invalid/non-text or empty HTML, and PNG/PDF/ZIP signature conflicts even with a closing HTML tag. Refusal is typed and does not reach image optimization or another fallback. |
| HHP-08 | The explicit request does not bypass existing local roots, byte caps, network-path refusal or missing-file errors. An ordinary non-HTML load, omitted mode and unknown mode retain their established outcomes. Real-file retry after replacement uses fresh bytes. |
| HHP-09 | Real `AttachmentsService::dispatch_gateway("loadWebMedia", ...)` reads owned files, consumes the exact mode, returns exact bytes/MIME/kind/filename with generated trust absent, and returns the same refusal on invalid explicit HTML. No mock data reader or text-only source assertion substitutes for this consumer. |

Use existing `web_media.rs`, `host_read_media_policy.rs`,
`attachments_service.rs`, `openclaw_web_media_contract.rs` and the recovered
`openclaw_html_provenance_contract.rs`; no new module, dependency, storage,
worker, tool schema or permission owner. The operation is synchronous and
read-only: a refused/failed read publishes no result or durable marker, and
an ordinary retry reopens/revalidates the file. It promises no new cancellation
or atomic filesystem-confinement guarantee beyond the existing loader.

Append the complete `openclaw_web_media_contract` target to the same strict
Actions capture together with the recovered HHP policy and format groups.
Keep all existing MSL and web-media assertions. Capture every applicable
named positive exactly once on the existing Linux/macOS/Windows lanes; bind
the added production/test inputs to the current H. Root serializes the
service arm, capture, ledger/RTM, immutable review scope and cloud publication.
Independent review and actual native Actions remain required before even
these scoped contracts can be marked verified.
