# Unicode-safe existing media references — MRU-v1

Status: implementation contract; exact-H Actions and full acceptance pending.
Base H `1573db1356d4e3531efd559ca3718abc4d7eeb22`, tree
`5ebf3f2d98e0ccc9713aeeefd6bc6580e949f548`. Fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.
The first integration parent is `f5f6d88fd5e1f96e27558d2a17e490964c6d57a5`, tree
`0ba6b18f1f23b4fd634b6d26755e48c321f4ce87`; preserve its independent
Automation fresh-custody implementation and gates without claiming acceptance.
Publication resumes on `42ee1165d216a08361e6a1c7f9fe29bd609c8b1c`, tree
`d119b337cf4f1d00ab00bb26144998e2c378ee86`. Preserve the incoming Memory
selector implementation, required positives and evidence references alongside
MRU, including its shared workflow/RTM additions. The preceding preview
fallback oracle repair, prior-batch evidence and temporary configuration
Git-object carrier remain unchanged; the carrier supplies no product validation.

## Problem, authority and ownership

The existing Rust `media_reference` owner classifies ASCII scheme prefixes by
indexing arbitrary UTF-8 strings at byte 5, 7 and 8. A valid string can have a
code point across those boundaries. Examples include `file:/é.png`,
`file://é.invalid/a.png`, Windows paths and ordinary non-ASCII local paths.
The slice can panic before the existing resolver returns its normal result.
The same defect occurs in normalization and inbound-URI prefix detection.

Decision **MRU-01 — FIX / IMPL_BUG**: make these prefix decisions total for
valid Rust strings, retaining exact ASCII case-insensitive scheme recognition
and the existing URI, local-path, size and upload policies. Repair the existing
owner; do not catch panics, add a second parser or add a new Gateway method.

Current production binding is `ProductionXuserHost::resolve_attachment` →
`AttachmentsService::resolve_channel_attachment` → channel attachment resolver
→ `classify_media_reference_source`. `SandXuserEntryPublisher::inline_images`
uses that callback and its existing per-room worker. An unresolved image is
omitted while publishable text and later entries continue; a panic instead
terminates that worker. The sender map retains its closed sender.

This is a necessary public parsing/attachment callback contract, not proof
that a fresh desktop send supplies arbitrary images to this sharing path.
Current fresh group/mirror producers require further attachment projection
review; no new producer or sharing protocol is added here. Tests must identify
their controlled relay boundary and must use the actual production callback
and actual AttachmentsService for the image-resolution decision.

Fixed-U prefix authority was read directly through the GitHub file API:

| Source | Exact blob | Relevant contract |
| --- | --- | --- |
| `src/media/media-reference.ts` | `b94761535f4e5e6521e7b203c7e3792a71ea2c04` | Scheme classification uses anchored ASCII regexes; Windows/data policies remain distinct. |
| `src/media/inbound-media-uri.ts` | `81ca45eb0accaa72a31cc4c0e718ac7e72121472` | Normalization and inbound recognition use regexes; arbitrary Unicode is not an exceptional condition. |
| `packages/net-policy/src/url-protocol.ts` | `baa01d1b7fefc35e0f2f9fee2dacb7d94bd50b67` | HTTP prefix detection is anchored `https?://`, case-insensitive. |

These source reads establish the bounded prefix contract. Other reference
resolution dependencies, realpath/store behavior and the whole media unit
remain unaccepted. Preserve historical reading observations; do not relabel
the old ad8d dossier or upgrade source, verification or global flags.

## Requirements and causal oracles

| ID | Required behavior |
| --- | --- |
| MRU-01 | Normalization, inbound parsing and source classification return normally for valid UTF-8, including code points crossing bytes 5, 7 or 8. Preserve non-prefix text and ordinary ASCII behavior. |
| MRU-02 | Retain case-insensitive file/http/https/data/media flags, explicit data policy and Windows-drive recognition. A Unicode lookalike or partial scheme must not acquire another scheme's authority. |
| MRU-03 | Preserve Unicode inbound IDs, canonical and legacy MEDIA prefixes, and all existing wrong bucket, credentials, query, fragment, nested/encoded slash, backslash, NUL and traversal refusals. No filesystem scope expansion. |
| MRU-04 | The actual AttachmentsService preserves Unicode filename, bytes and MIME for a valid local image and omits unresolved or remote-authority file references through the existing resolver. Auth and Box execution are not needed. |
| MRU-05 | The actual ProductionXuserHost callback and ordered publisher handle an unresolved Unicode reference without terminating their room worker. A subsequent valid local image and text publication reach a controlled relay in order with exact expected content, bytes, MIME and alt text. The fixture must not replace the image callback with a mock lookup. |

The parser remains stateless. Existing Attachments/CrossUser owners retain
storage, read limits, per-room sequencing and result handling. This fix adds no
accepted job, retry state, cancellation policy or restart migration. A new
owner instance continues to resolve unchanged stored references. Existing
publisher cancellation/drain and real-backend behavior remain separate gates.

## Implementation and verification

Root owns `media_reference.rs`, workflow, this contract and additive evidence
references in the existing OCRM ledger/RTM. An independent worker may add tests
under the existing `cross_user_sharing::production` test module; it may not
change production behavior there. Keep the existing channel/media-reference
contracts and test the complete changed prefix functions plus their consumers.

Reuse the existing Linux and macOS Host workflow/cache. Run the existing media
reference and channel attachment targets and a focused actual production-owner
test; retain complete command exits, logs, exact H/tree/U and required positive
names. Missing/duplicate positives fail. No local or htch-runtime validation.
Freeze the complete changed scope and obtain independent source review before
publication; exact-H Actions are required before claiming this contract passes.

Acceptance is limited to MRU-01..05 and the stated callback boundary. Real
sharing accounts, fresh UI attachment projection, other media readers, platform
packages, performance/soak and G01–G12/AC01–AC24 remain open. Rollback must not
silently reintroduce unchecked UTF-8 indexing; a release rollback uses the
normal prior-artifact boundary and preserves outstanding acceptance records.

## Shared Linux verification recovery

**CI-LINUX-STORAGE-01 — FIX / TEST_FLAW (infrastructure/evidence).** Runtime
run `41529468771`, H `1573db1356d4e3531efd559ca3718abc4d7eeb22`, Linux job
`125565630761` ended with failure at `2026-11-02T21:38:11Z`. Its public run
annotation reports `System.IO.IOException: No space left on device` for
`/home/runner/actions-runner/cached/2.338.2/_diag/Worker_20261102-213435-utc.log`,
followed by an unhandled runner exception. The job's Automation step retained
a stale in-progress marker; that marker does not override terminal job state.
The log endpoint returns BlobNotFound and no SQLite job artifact survived.
Native/media/CWR step-success metadata is retained as limited evidence, not
as audited named-positive receipts. App and LPG execution remain unproved.
Renderer's 104 tests and source authority's 33/21 checks have their own retained
artifacts; they do not establish the missing Rust results.

The precise disk consumer is unproved. The Host manifest has no compact test
profile, while this job now links multiple contract targets and the actual App.
Use job-scoped `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0`,
retaining test/debug assertions, overflow checks, target selection and every
oracle. Record `CARGO_INCREMENTAL=0` explicitly; rust-cache already disables
incremental builds. Keep the same workspace, cache action and shared key:
the action includes `CARGO` variables in its environment-key inputs, so the
new profile is distinguished without deleting unrelated caches or artifacts.
This reduces known owned build-output cost; it is not proof that build output
was the only exhausted-disk cause.

Record exact subject/profile and filesystem capacity before execution and after
the native/media/channel stages, then upload that completed evidence before
entering the later Automation/App work. Retain the final full artifact upload.
Do not delete runner-wide software, clear shared caches, remove assertions or
rerun the unchanged failed job. The next changed exact-H execution is the
focused recovery campaign. A repeated exhaustion requires fresh storage
evidence assessment; it is neither a passing test nor permission to retry
unchanged. Fresh independent review must cover this two-file recovery delta
with the complete frozen MRU scope as context.
