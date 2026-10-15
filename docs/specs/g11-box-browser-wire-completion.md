# G11 Box browser wire completion

Status: active; owner: existing BoxExecRuntime and Host browser driver;2026-10-15.
Parent: fdn-g11-host-browser-actions.md HB02/06; FDN-001 R12/13/19/20.

## Context, goal and non-goals
Actual Actions38697503722 starts/authenticates the real canonical daemon but the
first production accessor shell RPC fails. Source proves Host wraps Exec requests
in a Connect envelope while daemon decodes the whole body as bare protobuf. The
daemon also omits frozen Write tag3 and denies browser files under/tmp because
they escape workspace. Complete these existing owners, not a parallel browser or
RPC service. No arbitrary filesystem roots, credential export, installed personal
browser, external Box provider or packaged/extension claim.

## Requirements
- BW01: Exec accepts exactly one bounded uncompressed Connect data envelope when
  Content-Type is application/connect+proto; reject flags, length mismatch,
  trailing/multiple frames. Preserve existing bare application/proto compatibility.
  Body limit16MiB; authenticate existing bearer before dispatch. Preserve response
  Connect stream, IDs, errors and lifecycle. Do not unwrap Control unary messages.
- BW02: Add existing frozen Write tag3 request/result. Args path1/toolCallId3/bytes5;
  result success1/error5/rejected6. Reuse workspace identity/path authority.
  Writes at most8MiB, require existing parent, reject symlink/escape/nonregular
  targets, use mode0600 staging, sync and atomic replacement, remove own staging on
  failure. On Unix anchor traversal/rename to workspace directory descriptors and
  reject symlink components so concurrent symlink replacement cannot redirect.
  Other platforms explicitly reject unsupported secure writes for this slice.
- BW03: Browser runtime driver/results/views/snapshots live under the existing
  authorized /workspace/.sand-browser root, not a new daemon filesystem exception.
  Keep directory private0700. Old ephemeral claims are not migrated: require fresh
  list/claim; no credentials are moved or broad filesystem access granted.
- BW04: Actions unit/protocol tests cover framed success and malformed envelopes,
  frozen Write tags, actual file bytes, overwrite, symlink/escape/size rejection,
  no orphan staging. Full existing daemon/Host tests plus actual12 browser cases
  through production accessor/daemon are mandatory on exact PR and merged main.
- BW05: Actual framed RPC now executes navigation, but run38698699437 shows no
  screenshot bytes: daemon Read always lossily decodes binary as UTF8. When no
  encoding hint is supplied, return frozen Data field5 for invalid UTF8 or NUL
  bytes; retain Content for normal text and explicit text encoding behavior.
  Reject line ranges for binary data. Bound files to8MiB before allocation and
  return a structured invalid-file response on excess. Verify exact binary bytes
  and range/size rejection, plus real PNG retrieval through the shipping client.

## Architecture, strategy and interfaces
Host generated_production.rs remains the client wire authority. BoxExecRuntime
server.rs remains request dispatch/filesystem owner. Existing driver constants and
STATE_DIR select the confined browser runtime path; Host browser executor still
uploads/executes/reads via normal production resource APIs and safety callback.
Spec -> strict envelope decode + existing Write case + confined browser paths ->
focused Actions protocol regressions -> real browser/Rust/Box matrix -> integrate.
Test-only Actions adapter follows production paths and must not translate around
or hide unsupported RPCs. No new schema, auth system or external credential.

## Failure, acceptance and delivery
AC1: BW01–05 focused/whole-owner CI pass; actual direct and Rust/Box12-case suites
pass with actual screenshots and hashes. AC2: exact-main evidence recorded.
External hosted Box, signed extension/native messaging and packaged release remain
independent. Any missing/failed case blocks; do not weaken workspace guard.
Rollback pauses browser dispatch and retains old ephemeral state; never restore
claims by URL or import personal profiles. Record source/run/count/exit, reject
categories and synthetic fixture observations only, no bearer token/profile data.

## Provenance
Actual failure38697503722, Host generated_production.rs encode_exec_server_message /
send_connect_server_stream and daemonserver.rs frozen enum/route/resolve_path read
from current canonical main. User requests completing real G11 environment and all
remaining conditions; this repair is necessary to run the existing production path.
