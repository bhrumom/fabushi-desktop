# First-party Mini App publication from existing Plugins

Status: active implementation slice. Date: 2026-10-13. Owner: existing Plugins
surface / native MCP-Marketplace edge / existing account authentication service.
Parent: FPM-001, desktop-catalog-package-installation.md.

User requests completing first-party Global Dandelion publication. Installed
Fabushi was switched by the user from CI account to real account bhrum108.
Current Plugins lacks canonical namespace/release publication UI. Do not read,
export or impersonate its session; use existing native account service internally.

## Requirements / interfaces

- MP-01: same Plugins dialog contains first-party publisher controls, not an
  untrusted mini-app page or second store. Optional typed marketplace facet lets
  older bridges report unsupported instead of pretending publication succeeded.
- MP-02: native handlers require exact trusted main-window sender and current
  Fabushi auth service. Backend resolves genuine account and namespace ownership.
  Token never returned to Renderer, logs, file, artifact or agent. No reviewer
  credential or approval method in this bridge.
- MP-03: namespace claim is an explicit user intent. Package selection uses OS
  dialog only; renderer supplies no filesystem path, download URL, account ID or
  key. Native edge bounds regular-file size to25MiB and fences account/token change
  during selection and each mutation; no retry of uncertain writes.
- MP-04: the exact selected .fabapp bytes and SHA256 are displayed as the user's
  publication candidate. Native publisher does not execute the package. Create
  immutable web-miniapp draft, upload exact bytes to existing artifact CAS, submit
  review with honest preview/offline-workbench/native-transfer-unavailable metadata.
  The current package ships MIT OR Apache-2.0 notices. Server scan/sign/review and client install
  acceptance remain separate, never automatic UI approval.
- MP-05: first-party metadata/namespace fields bounded and validated. Backend
  public descriptors contain no paths/session/secret. Errors are structured safe
  states; cancellation preserves state, account departure fails closed.

Data flow: Plugins publisher intent -> typed optional marketplace bridge ->
trusted native account/file edge -> existing /v1/extensions publisher APIs.
Identity namespace=bhrumom/name=global-dandelion/kind=web-miniapp, appId
org.fabushi.global-dandelion, version0.2.1 for the current reviewed candidate.
This narrow first release does not invent a generic publisher/code-upload service.
The prepared signed package digest is pinned to the reviewed delivery evidence;
other package/version requires a separate reviewed release update.

### Native network adapter

Signed candidate0bf recovered real bhrum108 in the normal UI, but owned-namespace
GET timed out in the Node fetch path. Production native ports inject Electron
net.fetch into the existing account and publisher owners, using the installed
client's OS network/proxy stack. Recipient stays the fixed first-party API;
redirects remain refused, ambient cookies omitted, and TLS validation enabled.
Headless ports retain injected/global fetch, with no second auth owner. No
uncertain writes are retried. The cause/fix still requires actual UI verification.

Verification exclusively Actions: trusted sender rejection, no renderer-supplied
path/token, file bounds, token change, uncertain-write/no-retry, exact headers/bytes,
namespace ownership rejection, draft/upload/submit order; frontend accessible
unsupported/busy/failure/review-pending states and existing Plugins regressions.
Then packaged first-party UI must enroll actual account and submit actual signed
candidate. Native service activation and final approved install/offline runtime are
independent gates. No source/local build/test. Rollback removes this optional UI
facet without modifying existing connectors/skills or immutable server releases.

AC-1 MP-01..05 exact-head CI. AC-2 actual account owns namespace, actual signed
artifact uploaded and pending-review state observable. AC-3 separate reviewer scan,
release signature/approval, catalog and packaged install acceptance. All pending.
References: Global Dandelion PR2 fce746b, backend production signing run38366572165
(packageSHA afc45ad273e4965fc272a89e775291444c683513ecbc3fa31769858431e8a6b6).

## Architecture exception and protected-source evidence

The user's explicit request to publish a Fabushi offline mini app supersedes the
historical CONN-009 prohibition only for this bounded publisher feature inside
the existing Plugins dialog. It does not authorize a separate Mini Apps shell,
parallel Host, renderer-owned install ledger, or approval bypass. Existing frozen
reference inventory and reference blob hashes remain unchanged.

Five mapped modules require evidence-locked semantic adaptation: preload adds
the optional narrow three-intent facet; MCP OAuth native composition injects
existing authentication and safe OS file selection; production services forwards
Electron dialog without creating a manager; Plugins desktop surface mounts the
same panel during catalog loading/failure; Plugins view accepts one optional
presentation slot. Each adaptation retains original behavior/evidence and adds
this spec and the signed-candidate publisher contract workflow. Updating a target
hash records inspected source, not passing CI or independent review. All five
must re-pass strict architecture and exact-source packaged acceptance.
# Native error redaction

All publisher IPC failures return only a fixed safe error code. Native account,
file-picker, filesystem and stream exceptions must not send paths, credentials,
upstream diagnostics or selected file contents to Renderer. HTTP failures retain
only the numeric status; network uncertainty still forbids automatic write retry.

## Focused publisher verification plan (2026-10-14)

MP-T-EDGE-01 (MP-03/05): injected ports cover cancelled selection with no writes,
empty/25MiB/25MiB+1 byte selection limits (the inclusive limit must proceed to
digest validation), disposal and account/token changes during selection, response
reading and each draft/upload/submit boundary. Assert no next mutation or retry
after a fence fails, and no duplicate publication while one selection is pending.
MP-T-BOUND-01 (MP-04/05): validate namespace syntax/128-character boundary,
bounded response streaming and cancellation, malformed JSON/safe HTTP errors,
immutable-release conflicts and frozen states using the existing injected fetch.
Use the immutable signed fixture where publication must reach mutations; never
fake its digest or production approval. Execution is GitHub Actions-only and
results remain pending until exact-source evidence exists.

The publisher port exposes bytes, not file metadata. OS regular-file, symlink and
pre-read size checks belong to the existing native file-selection adapter. A
follow-up in that owner must test a cancelled dialog, directory/special/symlink
selection and stat/read-size or file-change races with temporary fixtures on an
Actions runner, preserving the 25MiB bound before reading. These byte-port tests
do not satisfy that OS boundary or packaged first-party UI acceptance.
The actual adapter is `source/electron-main/adapters/mcp-oauth.ts`'s
`selectPackage` closure: O_NOFOLLOW/open, handle.stat().isFile(), bounded read and
post-read size comparison. Test that closure through its existing composition
ports first; if unavailable, propose a narrow helper in that same adapter owner,
not a new publisher/file service. The namespace fixture also rejects trailing LF:
the current JavaScript `$` anchor may accept that terminator. Keep the strict
assertion and record any Actions failure for the existing publisher owner rather
than silently relaxing namespace syntax.

