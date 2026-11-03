# Media file URL admission — FUA-v1

Status: implementation contract; source and runtime acceptance are separate.

## Authority and problem

This contract extends `OCRM-RESP-MEDIA-WEB-PIPELINE-001` in the existing Rust
Attachments owner. It follows the main migration specification §17 and preserves
the HHP-v2/IPB-v1 implementation published at target base
`947e980bd3c9dcb06535127f5cedfd193b83f0fc` (tree
`686109128a616195e65d1c9018ccd5b0b9d2cc8a`). Fixed OpenClaw U remains
`1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.

At that target, `web_media::file_url_to_path` rejects a literal `attacker` host
and a four-slash prefix, then accepts the native URL conversion or manually
strips the failed URL into a local path. On Windows the locked `url` 2.5.8
converter accepts other hosts as UNC paths. On Unix the fallback turns a remote
authority into a local first path component. The loader checks the original
string for network paths but does not check its resolved path before
canonicalization or reading.

Fixed U's `src/media/web-media.ts` delegates file URLs to the actual
`@openclaw/fs-safe/advanced` 0.27.0 dependency and wraps any conversion failure as
`invalid-file-url`. Its packaged `safeFileURLToPath` permits only empty or
localhost hosts before native conversion, rejects encoded path separators, and
checks the result for prohibited Windows paths. No rejected URL is recovered by
discarding its authority. This is a required admission behavior, not a new
source-tool feature.

## User behavior, scope and owner

An admitted local media file URL must identify the same native file and preserve
the loader's existing byte/format/error behavior. An unsupported remote file URL
must be refused before the local filesystem boundary, including when roots or
the existing sandbox flag are supplied. The producer is a caller's media
reference; the consumer is the existing bounded media loader and direct
`AttachmentsService::dispatch_gateway("loadWebMedia", ...)` arm.

There is no new runtime, public API registration, permission owner, persistence
schema, cache, background task or dependency. The public Gateway registry does
not currently expose `loadWebMedia`; this repair keeps that boundary unchanged.
The existing channel attachment resolver is a separate consumer and is not
rerouted through this loader.

This batch does not establish authentic default/session roots, an authorized
safe-reader capability, file-identity/race closure, generated HTML provenance,
full Node/Rust URL or filesystem-alias parity, codec parity, public HTTP/UI/
Runner/channel wiring, or a physical/network device acceptance. Those remain
applicable work. In particular, rejecting a remote file URL does not grant
permission to read an arbitrary local file.

## Requirements

| ID | Required behavior |
| --- | --- |
| FUA-01 | Use structural URL parsing. Accept only the file scheme with empty/localhost host. Reject every other hostname or IP authority on every supported OS as `WebMediaError::LocalAccess` with code `invalid-file-url`. Do not use a host-name denylist. |
| FUA-02 | Preserve URL parse and native file-path conversion failures as `invalid-file-url`. Remove the permissive slash-stripping/lossy-decoding fallback. Support legitimate native absolute, single-slash, mixed/uppercase and localhost file URL forms through the real parser. |
| FUA-03 | Reject prohibited raw and resolved network paths before canonicalize, metadata, open or read. File URL rejection retains `invalid-file-url`; raw/workspace-derived prohibited network paths retain `network-path-not-allowed`. Neither roots nor the existing sandbox flag may bypass URL admission. |
| FUA-04 | Preserve exact native paths, Unicode, spaces and literal percent signs for supported local URLs. Retain encoded slash/backslash refusals. Use platform-native fixtures rather than a Unix path as a Windows positive. Keep byte limits, allowed-root checks, explicit HTML policy and image handling intact. |
| FUA-05 | The actual direct Attachments service propagates admission errors and still returns the correct bytes and file name for admitted native fixtures. Refusal creates no durable state; a later valid call works normally. Public Gateway admission remains unchanged. |
| FUA-06 | Keep source reading, implementation, wiring, execution and acceptance distinct in the existing OCRM row/RTM. Historical reading and failed receipts remain intact; no global flag or full-unit verification is promoted by this bounded repair. |

## Architecture and implementation boundary

Repair `web_media.rs::file_url_to_path` in place with the existing `url::Url`.
Require its parsed scheme/host, reject encoded separators in the path, require
native conversion success, and reject a prohibited converted network path.
Do not replace the local loader with a source-specific runtime or reuse the
different channel parser in a way that changes its typed error contract.

The loader's reference cleanup, URL/workspace resolution and final path guard
form a pure admission phase. A private, synchronous local-reader callback may
mark the one transition from that phase to the existing canonicalize/bounded-
read/format body. Production always supplies the existing local body; callers
cannot inject a reader through a public option or Gateway argument. The seam
allows negative tests to count boundary entry without contacting a share. It
is not a permission capability or a new authority to skip roots.

Keep all actual filesystem operations in the local body after admission. Keep
that body's cap, root, MIME, HTML and image behavior unchanged. No cancellation
protocol, transaction or restart recovery is added: a refusal is synchronous,
has no persisted effect and does not poison a later ordinary load.

On Windows, a workspace already expressed as a rooted native `VerbatimDisk`
path (for example after canonicalization) remains a local workspace. The final
guard must distinguish it from UNC/VerbatimUNC and device/network namespaces
without broadening the existing raw-path contract. Pin that distinction with
a native Windows boundary-entry positive and retained UNC negatives.

## Verification and acceptance

All executable checks run only in GitHub Actions at a frozen exact H. Extend
the existing three-platform media capture, retaining all HHP-v2/IPB-v1 and
cleanup controls. Do not execute product code, parsers, tests, generators or
validators locally or on htch-runtime.

| Oracle | Causal evidence required |
| --- | --- |
| FUA-O01 | Pure conversion rejects unrelated hosts, localhost suffixes, IPv4/IPv6 authorities, mixed-case schemes and malformed/conversion-invalid URLs with the exact code. No actual share is contacted. |
| FUA-O02 | The production admission/reader boundary is entered zero times for remote/malformed/encoded-separator references with default options, explicit roots and sandbox flag variants. Workspace-derived prohibited network paths also stop before the reader. A valid native path enters once with the exact path/options; reader errors propagate without retry. These are boundary-entry measurements plus a source review that all filesystem operations are inside the body, not measurements of arbitrary OS syscalls. |
| FUA-O03 | Native temp fixtures on Linux, macOS and Windows preserve exact path, bytes and Unicode/space/percent filename through ordinary, single-slash uppercase and localhost URL forms, actual load and service dispatch. Byte-cap refusal and subsequent valid recovery remain effective. |
| FUA-O04 | A Unix fixture reachable by the old authority-stripping fallback is refused with roots still active, and via the real service. Its legitimate local URL still loads unchanged bytes. This is a local alias test, not a network probe. |
| FUA-O05 | Actual service parse/refusal behavior and the current finite Gateway command admission stay distinct. No newly public loadWebMedia method is introduced to make the tests pass. |

Restoring a one-host denylist must fail unrelated-host controls; restoring the
fallback must fail the Unix alias; accepting Windows UNC conversion must fail
native Windows converter controls; moving admission after the reader must fail
the zero-entry controls; rejecting all URLs must fail native positives.

Before bounded acceptance, require source-review convergence and exact-H native
receipts with fixed U, matching input blobs, raw-log hashes, command/capture
exits, positive-name uniqueness and all selected assertions. Missing/queued OS
jobs remain unverified. This batch cannot satisfy full G01–G12/AC01–AC24,
independent ACCEPT or shipping-SHA acceptance by itself.

## Release, rollback and provenance

The change is a stateless Rust implementation repair on the existing migration
branch. Rollback is an ordinary scoped revert preserving unrelated incoming
work; there is no data migration. Existing typed errors provide the observable
refusal, without adding logs containing file contents or credentials.

- Fixed-U `src/media/web-media.ts`, blob `f48bca504f6fb77c3d094864eb656cb2b1bf25e2`, URL admission and resolved-path/read ordering.
- Fixed-U `src/media/web-media.file-url.windows.test.ts`, blob `bdaedcde111ea2880b96d9afa547a9c1d2a36b42`, native uppercase single-slash control and refusal cases.
- Fixed-U package/lock binding: `@openclaw/fs-safe` 0.27.0, archive SHA-256 `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`; packaged `dist/local-file-access.js` SHA-256 `7530a957ca9d56f56deba8da1dd06be03c894c6f6472bc8571080937b57bfda8`. Review is scoped dependency-contract/actual-use review, not a claim of full package reading or generator closure.
- Target host and App locks both bind `url` 2.5.8, checksum `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`. Version-source commit `d6ea13c5f8e7e6e627f6390161b3e185bda5e5ce` documents Windows UNC conversion; exact crate archive acquisition remains separate.
- Existing `media-store-read-scope.md`, `media-html-policy-safety.md` and `media-image-probe-boundary.md` retain their independent authority and open scope.

## Locked-parser normalization correction before combined BR publication

This pre-implementation amendment is based on current owner commit
`3647531d1d47d1c690b98a06f0156ae2734ff7ad`, inherited at
`073b98b4826c8125d0a42d15f8ccdbfebb5eb488`. It enforces FUA-01 through
FUA-04 with the actual locked `url` 2.5.8 archive, SHA-256
`ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`.
The archive source is the authority for these library semantics; no executable
reproduction has been performed locally or on htch.

The parsed host alone is insufficient: `parser.rs` 1344–1363 reports
`FileWithHostAndWindowsDrive` then clears a nonempty host; 1371–1378 removes
initial empty file path segments without a violation. The Windows converter
`lib.rs` 3128–3202 can reach its absolute-path assertion on a bare drive.

- Retain the parser's original-host violation through its existing parse-options
  callback and refuse it as `invalid-file-url`. Ordinary empty/localhost native
  URLs, mixed-case schemes and valid single-slash forms remain accepted.
- Preserve the existing all-platform four-slash refusal, including equivalent
  mixed slash/backslash and localhost forms. A private prefix/component guard
  may observe the original leading file-path separator shape while ignoring
  exactly the tab/CR/LF characters ignored by the URL parser. It is not another
  URL parser or a blanket ban on ordinary interior separators. Query/fragment
  bytes do not become path admission input.
- Before Windows native conversion, refuse bare path/drive forms lacking a root
  segment. No assertion/panic, implicit current-drive path or successful empty
  result may replace `invalid-file-url`. Actual rooted native positives must
  still work.
- Keep the existing private no-I/O reader boundary and add these cases to its
  zero-entry controls under default, rooted and sandbox option variants. Keep
  every pre-existing four-slash/authority/encoded-separator assertion. Exercise
  legitimate localhost, percent/unicode names and query/fragment forms through
  real native file/service consumers; no live share/device negative is allowed.

This retains the existing stricter all-platform leading-separator policy;
it does not claim equivalence for all POSIX double-root spellings. Strict
pathname UTF-8/percent decoding, encoded drive-letter behavior, Unix terminal
colon spelling, broader dot-normalization parity, Windows ADS/NUL/device opens,
and existing capability/identity/transport obligations remain applicable
followup work. Neither this correction nor BR's generic reader promotes the
complete FUA/media row or any global acceptance flag. Every native execution
and exact current-H input/log/exit observation remains required in Actions.

## Native Windows rooted-drive oracle correction — FUA-WIN-ORACLE-01

At e19 runtime `41571103928`, Windows job `125703932910` executed the
web-media target with 22 passed and one failed test. The rooted-drive loop
incorrectly expected `file:///C|/` to normalize under locked `url` 2.5.8.
Its empty-host parsing leaves the pipe drive unchanged and native conversion
refuses it; FUA-02 requires preserving that refusal. The original log did not
print the loop value, so this cause is an exact-source inference from the
checksum-bound parser/converter, not a per-iteration runtime observation.

Keep that spelling as an explicit Windows `invalid-file-url` negative. Retain
all existing refusal assertions and the three valid colon/encoded-colon root
positives; use the supported `file:/C|/` spelling to preserve the normalization
positive and include each fixture value in failure diagnostics. Keep the same
test name, complete selected count and strict capture. Production admission,
fallback removal, dependency locks, BR/HHP/IPB and all broader compatibility
obligations remain unchanged. The corrected exact-H Windows result is required;
source inference alone is not a passing result or a permanent scope exemption.

## Windows rooted drive-pipe recovery — FUA-PIPE-01

The preceding FUA-WIN-ORACLE-01 section is retained as the historical decision
published at `78aa967cc9400bb5c207441b2a78dbf7021d596c`, under the then-unchanged
native converter. This amendment supersedes only its refusal of
`file:///C|/`: source-supported compatibility now requires the guarded Windows
normalization below. FUA-02 still propagates failures after that normalization;
there is no recovery after a failed conversion or rejected authority. Preserve
the incoming `file:/C|/` positive as well as all other still-valid controls.

The exact-H Windows job
[`41571310630/125704599418`](https://github.com/bhrumom/fabushi-desktop/actions/runs/41571310630/job/125704599418)
at `3438eb5bac755c399c9ec686414bbb4c0816888d` failed
`test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references` at line
253: `rooted drive remains valid` returned `invalid-file-url` with
`File URL cannot be converted to an absolute local path`. The web contract
reported 22 passed/1 failed, cargo exit 101 and capture exit 0; the other 11
media capture cases were accepted. Preserve this failed receipt/artifact
`13201906203`; it is not an environment failure or acceptance evidence.

The old panic omitted the reference. Static tracing of the actual locked
`url-2.5.8.crate` identifies the third positive, `file:///C|/`: `parse_file_host`
returns the empty-host flag, `parse_file` adds a slash before the still-leading
slash, and `parse_path` consequently misses its first-segment drive-letter
normalization. Removing leading empty segments afterward leaves `/C|/`; the
Windows converter rejects its literal pipe. This is a source-traced trigger,
not a reference printed by the failed run. The same converter already supports
`C%3A`, so encoded-colon conversion is not the repair.

Fixed U calls `@openclaw/fs-safe` 0.27.0 `safeFileURLToPath`; its exact source
`openclaw/fs-safe@67dee5ef4c03655a2aa98893539475ad041cb61b`,
`src/local-file-access.ts`, applies Node URL/native conversion after host and
encoded-separator admission. Fixed U permits Node 24.16.0. That Node version's
bundled `deps/ada/ada.cpp` sends an empty file host through path-start and
normalizes a first literal ASCII drive-letter pipe to a colon; its
`lib/internal/url.js::getPathFromURLWin32` then yields the rooted native path.
The asserted positive is a source-supported compatibility requirement.

- **FUA-PIPE-01:** On Windows only, after every existing scheme, original-host,
  leading-separator, encoded-separator and bare-root guard, normalize exactly
  the first parsed `/[A-Za-z]|/` component to `/[A-Za-z]:/` and delegate to the
  existing `Url::to_file_path`. Keep final network-path refusal. Do not decode
  another spelling, recover a failed authority, build a native path manually,
  add a dependency/owner, or change Unix behavior.
- **FUA-PIPE-02:** Retain all four pre-78aa rooted-drive positives plus incoming
  `file:/C|/`. The incoming `file:///C|/` refusal is superseded by the guarded
  compatibility implementation and its positive; retain every other negative.
  Include the reference in rooted-drive failure diagnostics.
  Pin refusal of encoded pipes, non-letter/long drive tokens, bare drives,
  prohibited leading separators and encoded separators before the private
  reader; roots and sandbox variants do not bypass admission.
- **FUA-PIPE-03:** Extend the existing Windows native temp-file and actual
  Attachments service fixture with literal-pipe drive URLs, including localhost
  and query/fragment variants. They must retain exact Unicode/space/percent
  filenames, bytes, explicit HTML behavior, cap refusal and fresh recovery.
  A synthetic path-only assertion is insufficient consumer evidence.

Focused recovery must run in Actions on the new frozen H: the changed URL
admission owner controls and full `openclaw_web_media_contract` with positive
name/summary/exit evidence, retaining still-valid unrelated case evidence at
its original subject. Do not rerun unchanged inputs merely to select a green
attempt. Broader percent-decoding, encoded drive-letter spelling, dot-path
normalization, namespace/device policy and all other deferred/full-media
obligations above remain open; this repair does not claim their closure.
