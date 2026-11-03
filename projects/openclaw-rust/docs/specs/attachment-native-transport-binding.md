# Attachment native transport binding — MAB-v1

Status: implementation contract; exact-H execution and acceptance pending.  
Project: OCRM-001, existing M01/M04/M23 desktop transport and attachment owners.  
Implementation base: `f986b457e77d106638f02e8ff23f6b15d3eb792c`. Narrow source
discovery: `961d3ca65fc72cacd38eb0e81da768a44d6a1bc2`; the relevant Electron
sources are unchanged between these subjects. Fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.

First integration base: `6273f9252f308355526c68e833da8dea194bb6cf`. Its accepted-turn
Rust cancellation changes and workflow controls are retained; the relevant
Electron account, MainData and attachment source blobs remain unchanged.
Reviewed integration base: `5f8c5ab76fee0cef28ebe43dc7250a21e5eec0b5` preserves
the incoming Rust MCP uninstall custody, checkpoint publication, literal source
observation corrections and their complete workflow controls. Relevant Electron
blobs remain unchanged. The equivalent CGWA reopen-oracle repair was already
published in `31cac7c95ec0297f7dee72d4bc40dad56aee4f01`; adopt that exact test
blob instead of publishing a duplicate local correction. Neither incoming
source nor historical evidence is reclassified as this batch's implementation.
Integration parent `d95efb8a690750ad8dc78ec375ec5ff2a7ad2660` adds only the
existing inventory index/manifest and three generated shards. Those bytes are
retained; all 17 declared review paths were unchanged in the integration
snapshot `31ee38b019dd5e126887fe045d1af15550079983` before the recovery below.

## Problem and decisions before implementation

The real attachment binding retains `CoordinatorMainLegs.legs`, whose methods
select the current client on each invocation. A preview/download may probe on
account A and read subsequent chunks on B. The native staged-file commit may
read under A, cross an asynchronous account switch, then upload through B.
Copying the dynamic method table does not capture its original client.

**MAB-DEFECT-01 — FIX / IMPL_BUG:** bind each remote attachment operation to the
native account/runtime lifetime and exact MainData client present at entry.
Check at actual request posting, reply acceptance and native continuation.
Do not reinterpret renderer account labels or presentation freshness as authority.

**MAB-DEFECT-02 — FIX / IMPL_BUG:** synchronous runtime construction and port
adoption call external collaborators before publishing their candidate. Those
callbacks can retire, dispose or supersede the candidate. Recheck ownership
after construction/callbacks; dispose only the owned stale candidate and retain
the existing confirmed-exit/blocked discipline before a successor can launch.

**MAB-DEFECT-03 — FIX / IMPL_BUG:** independently capturing an active account
and an arbitrary current port cannot prove they belong together when this
runtime's adoption failed or a callback installed another client. The account
binding exposes only an opaque `runtimeIdentity: object`; the production
provider joins that exact returned runtime to its own latest port ownership
handle, and requires that handle's `assertCurrent` before capturing a session.
Use a local WeakMap of runtime-to-handle accessors for this composition, not
another account/port state authority. This decision precedes its implementation.

**MAB-DEFECT-04 — FIX / IMPL_BUG:** production account detach still invoked a
global `revokeMainDataPort()` before calling the runtime's scoped disposal.
If another port had replaced its adoption, that callback could close the
foreign client first. Pass the exact detached `CoordinatorRuntime` to the
existing `revokeMainDataPort(runtime)` collaborator; production resolves only
that runtime's own handle. Keep prepare → scoped revoke → stop/confirmed-exit
ordering. Existing zero-argument observer callbacks may ignore the argument;
the production adapter must use it. No global cleanup is substituted for an
unavailable handle. This decision precedes the callback and composition repair.

Account departure preparation still uses the old shared MainData port to
clear Host settings. Retiring attachment work must not close that port early
or prevent its ordinary cleanup requests. Guard denial is a request result,
not transport failure. Existing Rust Session, Attachments and Gateway owners
retain business policy, authorization, paths, byte admission and persistence.

## Existing ownership and production consumer chain

`ProductionRenderer` and its real `SpreadsheetViewerProvider` call preload
`readAttachmentBytes`/`downloadAttachment`; MainEdge forwards their existing
payloads to the production attachment binding and `createAttachmentEdgePort`.
That edge performs probe/chunk/dialog/write operations over the existing
MainData client and registered Rust methods. Renderer generation checks alone
cannot prevent the native side from reading a new account's bytes.

`CoordinatorAccountRuntime` remains the account/runtime activation owner;
`CoordinatorMainLegs` owns the current native port; `MainPortClient` owns its
queue and pending replies. The attachment adapter composes their opaque
lifetime checks. No credential lookup, account network call, second account
store, scheduler, permission owner or Rust business implementation is added to
Electron. This is a necessary thin transport contract for the Rust migration.

## Required behavior and shared interface

| ID | Contract |
| --- | --- |
| MAB-01 | The existing account owner exposes `captureCurrentBinding(): CoordinatorAccountBinding \| null`, where the binding has readonly opaque `runtimeIdentity: object` and synchronous `assertCurrent(): void`. The identity is the exact returned runtime reference, without exposing its methods or an account slot through this interface. It is available only for the fully published active native-authorized runtime, remains native-only and requires no network/credential refresh. |
| MAB-02 | Each activation has one stable, permanently retireable lifetime. Observation of a different slot or logout retires it before callbacks or queued cleanup; detach/disposal also retire it. A→B→A cannot revive an old binding. Same-account status refresh preserves it. Unstarted, constructing, inactive, refused, blocked, disposed and retired-for-cleanup states cannot furnish a usable capture. |
| MAB-03 | The existing MainData owner exposes `captureCurrent(assertLifetime): CapturedCoordinatorMainSession \| null`, containing captured `legs` and synchronous `assertCurrent`. It pins one client and checks identity, owner disposal, synchronous client phase and the supplied opaque guard. Awaiting-ready remains eligible; settled/replaced clients never revive. |
| MAB-04 | The client carries the guard in its real queued and pending records. Check on call admission, immediately before posting including ready-flush, and before accepting a reply. Guard failure settles only that request; remaining queued cleanup requests continue. Do not convert guard denial into protocol breach/post failure or close the shared port. Preserve existing post/breach/close failure settlement and unmatched-reply behavior. |
| MAB-05 | Runtime construction reserves an unpublished lifetime/attempt before external callbacks. Reentrant start cannot create another concurrent launch. After construction, recheck attempt ownership, permanent retirement and disposal before activation or renderer delivery. A stale returned runtime is owned until disposal/confirmed exit; timeout or failed exit remains blocked, eventual exit may recover only the latest account, and disposal stays terminal. |
| MAB-06 | MainData adoption uses an exact attempt identity. Revoke/dispose/nested adoption during hello or prior-client close cannot resurrect an outer candidate. Listener/start failure settles only the candidate; late settlement cannot clear a successor. Failed hello preserves a still-valid prior client. `adoptPort` returns an optional exact-client ownership handle with synchronous `assertCurrent()` and idempotent `revoke()`, so stale runtime cleanup cannot revoke another client's port. |
| MAB-07 | Production Coordinator service exposes typed `captureMainDataSession()` and resolves it lazily for attachments. It joins the active account binding's runtime identity to that runtime's own latest adoption handle, then captures the exact MainData client with both lifetime assertions. Missing, stale or foreign ownership refuses capture. Production runtime disposal withdraws only its own adopted MainData identity before awaiting process exit. Existing normal departure preparation and its shared Host-settings cleanup ordering remain intact. |
| MAB-08 | Each `readText`, `readBytes`, `download`, `commitStaged` and remote link-metadata operation captures once. Retain that capture across the entire operation; never capture again after an await or retry on a successor. Check after async replies, before consuming/copying bytes, and before reporting success. |
| MAB-09 | Download checks after the dialog before opening with `w`, after open, after each write and final close; all failures retain reachable handle close and existing failure cleanup. Zero-size and final-chunk cases cannot bypass retirement. Commit captures before the first native file read, checks before upload and after reply, and preserves original staged files on refusal. Already admitted Host/OS effects are not claimed rolled back. |
| MAB-10 | Local image preview snapshots an optional remote fallback before local I/O; absence of account/port must not prevent a successful local read. A failed local read cannot acquire a newly current remote account. Recheck remote-derived results. Local stage/discard and audio/video URL construction retain existing behavior and do not require account/network access. |
| MAB-11 | Keep public IPC/Rust method envelopes, paths, limits, existing error/telemetry boundaries and controlled valid behavior. Lifetime errors use bounded fixed text, without credentials or attachment payloads. No automatic replay or fake acknowledgement follows retirement. |

`CapturedCoordinatorMainSession` and its exact-port ownership handle are local
transport contracts, not wire schemas. The root integrator owns their shared
definitions and composition. The client may expose a narrow guard-bound method
table and synchronous live-phase assertion internally. Ordinary dynamic legs
used by resync and account cleanup remain unguarded by the attachment lifetime.

## Implementation boundaries and independent work

Allowed owner changes are account runtime and its focused contracts; MainData
legs/client and their focused contracts; the production Coordinator provider
and service interface; attachment edge/binding and a focused production-binding
contract. Existing Coordinator runtime tests supply restart/exit controls.
Root serializes this spec, production composition, test build selection and the
existing `.github/workflows/rust-desktop-runtime.yml` receipt. No lock,
inventory, ledger, RTM, global flags or upstream read facts are promoted.

The production `createRuntime` composition retains the exact ownership handle
returned by its own `onMainDataPort` adoption, including later same-account
relaunches. Its disposal withdraws that owned client without blindly revoking
a client installed by a reentrant collaborator. Account owner stale-candidate
cleanup awaits that returned runtime's disposal; it must not replace failed
cleanup with an unconditional inactive state.

Native filesystem seams used to pause an operation in tests remain thin ABI
ports whose shipping defaults call the existing Node filesystem functions.
They carry no duplicate attachment policy or fake persistence implementation.
Production code is not validated by running those seams in the local workspace.

## Causal Actions verification and acceptance

Use actual account runtime, MainData owner/client and production attachment
adapter/edge. Control only native process/port carriers, dialog and causal I/O
barriers; use real temporary-file bytes for download and staged-file checks.
At least one composed contract must use the production Coordinator service's
capture wiring. Preserve the original port-failure and runtime restart tests.

Required oracles: same-account refresh; immediate retirement while real cleanup
is held on `setHostSettings`; A→B→A; guarded and ordinary requests sharing one
ready-flush; posted and later-continuation stale replies; same-account client
replacement and synchronous settled phase; runtime construction reentrancy and
confirmed/failed/timed-out stale exit; port adoption reentrancy/listener/start
failure; real multi-chunk read and download positives plus switching negatives;
retirement during dialog/final write/final close and zero-size download;
native staged read before upload; offline local image/stage/discard and stale
remote fallback. Prove no successor request, mixed bytes, stale success or
lost cleanup, rather than merely asserting a helper's return value.

Freeze a complete immutable source scope and obtain independent general/
lifetime/security and `gpt-6.1-sol` reuse/structure/performance review at xhigh.
Run only affected existing GitHub Actions lanes and caches at exact H. Explicitly
bundle and execute the new contracts: their source presence is insufficient.
Require named positives once, completed non-skipped TAP totals, command and
capture status, H/tree/U and input/raw-log identities. Pending jobs, historical
CAP/MHT results and controlled native carriers are not current platform proof.

## Recovery, release and retained scope limits

No durable format or public RPC change. Retired operations return the existing
failure shape and retain source drafts/files; a later explicit operation may
capture the then-current owner only under its existing authorization. A Host
mutation or OS write accepted before retirement may have an unknown result;
there is no implicit retry or new receipt store. Rollback must retain lifetime
fencing or disable the affected remote operation rather than restore cross-port
continuation.

The immediate account fence starts at the native owner's observation, not at
unobserved token expiry. Same-account restart may still use the exact old live
port until actual settlement/replacement; this batch does not promise refusal
at restart invocation. `sand-media` streaming, atomic download overwrite,
bounded/no-follow legacy import, cold-start offline identity, full Rust composer
stage/commit/discard, its payload mismatch, Agent/Human target admission and
durable retry receipts remain separate open responsibilities. Real Electron,
provider/device/platform and all G01–G12/AC01–AC24 acceptance remain open.

## Source references

The narrow source review at the discovery subject read complete account runtime,
MainData legs/client, Coordinator production/root providers, cleanup/runtime/
launcher/resync, attachment adapter/edge/manager, existing port/runtime contracts
and spreadsheet viewer. It read bounded production-root, entry/MainEdge/preload,
renderer and workflow/build-consumer slices; these are not full-file read claims.
Reuse is justified by unchanged exact blobs at the implementation base. No
upstream full-file read or existing machine-ledger state follows from this native
bridge repair. Authority remains OCRM main Spec §17 and the existing account
transition, transport settlement and CAP contracts.

## Independent review recovery

Round 1 reviewed tree `31f2993d85a009d4d3e5ee49b9246cd6c161c3e5` against
`5f8c5ab76fee0cef28ebe43dc7250a21e5eec0b5`, complete 17-path patch SHA-256
`175e6ce80e7181a75645f6377b6dc92a2b793a4722bde6bb258192ad34db4d9c`.
The fresh configured-model general/lifetime/security reviewer and
`gpt-6.1-sol` reuse/structure/performance reviewer both ran at xhigh. The latter
returned no findings; the former identified the evidence defect below and no
other concrete product defect. These are source observations, not CI approval.

**MAB-REVIEW-R1-F1 — FIX / IMPL_BUG (P2, evidence capture):** the new
`mab_contracts` step inherits Bash `-e`; `set -uo pipefail` does not disable it.
A failing Node command or `tee` can exit before both `PIPESTATUS` members are
saved. Keep the gate fail-closed and bracket the actual shared capture wrapper's
pipeline with `set +e`, immediate status capture, then restored `set -e`.
Persist command/capture exits before returning failure. Exercise that same
wrapper in Actions under explicit inherited `-e` with nonzero Node/zero capture
and zero Node/nonzero capture; both must preserve their actual statuses and
return failure. Retain the normal both-zero production command and require all
three outcomes in the exact-H receipt. No local reproduction is authorized.
This decision is recorded before the wrapper and control implementation.

## Exact-subject Actions recovery

**MAB-CI-R1-F1 — FIX / TEST_FLAW (authored test source):** Runtime run
`41555028047`, attempt 1, renderer job `125651721307`, subject
`e2b6b850584045254a574a7847623939acf8d4cb`, failed the existing Electron build.
esbuild reported `Unexpected ")"` at
`source/electron-main/adapters/attachment-gateway.contract.test.ts:504:1`.
The plain `rejectReadAtBoundary` function declaration ends with the callback
delimiter `});`. Correct only that function's closing delimiter to `}`; retain
all four boundary scenarios, assertions and test selectors. The MAB and prior
Plugin Skills tests were skipped, and the always-run receipt failed closed;
neither suite is approved for this H. The completed shipping-bundle build
substeps do not establish the unexecuted lifetime controls.

This decision precedes the source correction. Freeze the complete MAB scope
again, obtain fresh focused review of this fix and adjacent callers, then use
the existing Actions build, 85-name MAB/capture controls and retained Plugin
Skills suite on the new exact H. Do not rerun the unchanged failing subject,
remove assertions, or infer a runtime defect or platform acceptance from this
test-source failure. Prior CAP evidence keeps its original subject.

**MAB-CI-R1-F2 — FIX / TEST_FLAW (same-class source inspection):** inspecting
the remaining plain helper declarations before the recovery found the same
callback closing delimiter at the end of `rejectDownloadIo` (original line
853). Actions stopped at the earlier parse error; this second defect is a
direct source observation, not another observed CI failure. Correct this
helper's final `});` to `}` as well. Preserve all four actual-file I/O boundary
controls and their exact assertions. Include both delimiter fixes in the same
focused review and recovery execution; do not spend another run rediscovering
an already confirmed syntax defect. This decision precedes the second edit.

**MAB-CI-R2-F1 — FIX / IMPL_BUG (native staging default):** on
`150df821a8a341a33053dbb9c01c169f5d73763c`, Runtime `41555913691`, attempt 1,
renderer job `125654628782`, the shipping/contract build and 115 retained
Plugin Skills/model/MCP tests passed. MAB executed all 85 tests: 81 passed,
four failed, none skipped/cancelled/todo. All four failures occur at actual
`stageBytes` setup before the staged-read/upload continuation assertions.
The normal command captured Node/tee exits `1/0`; both inherited-errexit
controls retained their required `7/0/1` and `0/1/1` outcomes.

CI used Node `v24.22.1`. Its [official Web Crypto source][MAB-NODE-WEBCRYPTO]
requires the Crypto receiver for `randomUUID`; the existing native edge calls
`(deps.randomUUID ?? crypto.randomUUID)()` without that receiver. Production
composition supplies no override. Actual TAP proves staging returned failure;
the precise invalid-receiver exception is a source/runtime inference because
the edge catches it and the failed assertion did not print that exception.
The [same Node version's module export][MAB-NODE-CRYPTO] exposes the underlying
freestanding native function. Import `randomUUID` from `node:crypto` and use it
as the existing fallback. Keep the injected override, source naming, byte
limits, real filesystem work, error/cleanup and account behavior unchanged.
Do not inject a fixture UUID or install a global Crypto shim to hide the defect.

The independent campaign-cap/directness assessment classified the subject as
reachable and this failure as an implementation defect, not an environment
block. The initial syntax campaign and focused recovery used the default
two-campaign allowance; an unchanged rerun is forbidden. Retain the 81 MAB,
115 Plugin and capture-control positives under their actual H, including the
25 MAB and 14 Plugin input-hash matches. After this narrowly scoped production
fix and fresh source review, the next direct instrument is the existing
exact-H Actions renderer build and real staging/continuation controls, with
their unchanged complete MAB/capture oracles. No assertion is removed, and
neither the earlier successes nor the campaign limit supplies new-H acceptance.
This recovery decision is recorded before editing the production fallback.

[MAB-NODE-WEBCRYPTO]: https://github.com/nodejs/node/blob/v24.22.1/lib/internal/crypto/webcrypto.js
[MAB-NODE-CRYPTO]: https://github.com/nodejs/node/blob/v24.22.1/lib/crypto.js
