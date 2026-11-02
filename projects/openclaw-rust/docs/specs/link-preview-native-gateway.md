# Existing desktop link-preview native Gateway — LPG-v1

Status: implementation contract; exact-H Actions and full UI/platform
acceptance remain required. Discovery baseline H is
`4e90f031aeeedb50efe4e265da084b99901478cd`, tree
`92ae0ff8441916395a33cdb38b31545c2d28e23c`. Fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`. Integrate after the concurrent
MHE-v1 correction without changing its owner or evidence boundaries. The
first implementation parent is H `55c23059ed28c34a087ec91eda17ae8376b75acb`,
tree `4cccb93b4d35357de2dc18f58be3bca0b474407a`; its media diagnostics and
inherited portable channel recovery remain intact. Concurrent MURL-v1 advanced
the integration parent to H `181cf49d4e14f08c1fc5daa9587928164ad09a12`, tree
`399d7bc03165fd49abffebfa4f149d1fd1c09e44`; preserve its input policy and
redirect controls along with the earlier batches. The next integration parent is
H `1b9803de2283322a7809dc626a5ca35a52eb834e`, tree
`7ffa987205bbc71af80114f6c15267937f2cf09a`, which carries the same workflow
admission correction described below and its original MURL decision record.
Further integration reached H `f890002148f122aacdfc78fe39530a7e9c1bef6e`,
tree `81f60c54ccf1c88698db3df41e3071445aa5b63b`, preserving its typed
Automation admission and actual app checks. Its WIP publication is not
runtime approval. Reuse that Linux app target/cache and the already selected
MCP composition target; do not duplicate their owner or test executions.
The delivery parent is H `e96c39ebe5b1c3fc318b20579bc38595d4d588a3`, tree
`2b49afdaa8d83e9b3503e9f33d71ac91ed5f68c1`. Preserve its provider-bound
desktop bridge and source-observation materializer, including their strict
pending reverse-link and historical-reading gates. No source count or
global flag is manually promoted by this preview batch.

## Current user path and missing connection

The desktop URL card calls `DesktopBridge.getLinkMetadata(url)` through
preload/main-edge. `createProductionAttachmentGatewayBinding` supplies the
native `fetchLinkMetadata` leg and constructs `cacheDir` from Electron's
`userData`; the renderer supplies the URL, not that path. Coordinator MainData
forwards the named method through its existing Host HTTP client.

At the baseline, `/api/fetchLinkMetadata` is absent from the Host method
registry and explicit extension list. `gateway_server::is_gateway_command`
therefore returns 404 before the existing `call_host_lane` →
`dispatch_gateway_call` → `AttachmentsService::dispatch_gateway` path can run.
The service already owns `fetch_link_metadata`, cache admission and
`safe_link_preview_fetch`; LPIP-v1 has corrected mapped-IP classification.
The earlier LPIP service tests do not prove this outer connection.

The current URL-card resource records this RPC rejection as `failed`; the
missing route prevents enrichment through this path even though the service
and cache owner already exist. No renderer state machine is replaced.

Decision **LPG-01 — FIX / IMPL_BUG**: restore this existing desktop behavior
through the existing registry, Coordinator carrier and Host native authority.
No new fetch/cache/runtime owner, generic method prefix or external media API
is required. Low-level media-fetch and extraction methods remain outside this
batch; their broad policy and real consumers are still open.

## Contract

| ID | Required behavior and oracle |
| --- | --- |
| LPG-01 | Register exactly `fetchLinkMetadata` as a Fabushi native attachment extension owned by `HostGatewayOwner::Attachments`. Preserve the frozen Grok and OpenClaw method lists; unknown neighboring names stay unknown. |
| LPG-02 | The existing Coordinator carrier admits this method only from MainData. Data/Control cannot supply a cache directory through the raw RPC leg; the existing renderer URL-only DesktopBridge path continues through Electron. Other methods retain their current decisions. |
| LPG-03 | Before parsing/dispatching preview arguments, reuse the Host's existing native control-plane authority: configured shared-secret authentication or the existing trusted loopback configuration, no paired-device authorization, absent/`operator` role and absent/`operator.write` scope. Missing/wrong configured secret remains 401; restricted, unknown or empty role/scope and recognized node/operator device tokens are denied before owner access. Existing Origin/Host checks remain. |
| LPG-04 | An authenticated native HTTP request reaches the actual AttachmentsService exactly once. A fresh public or mapped-public cache entry returns unchanged metadata. A prohibited mapped-private URL returns null before reading its fresh cache; cache bytes remain unchanged. No fake metadata policy or mock service response may establish this positive. |
| LPG-05 | Malformed JSON, unknown method neighbors and denied identities cause no AttachmentsService call or cache mutation. Existing service argument-error behavior is retained. A valid native call after each refusal still reaches the same owner. |
| LPG-06 | No persistent accepted job or second permission/cache truth is introduced. Connection/server ownership and shutdown remain with the existing Gateway/Coordinator. Restart reuses the existing cache policy; rollback removes the named exposure while retaining LPIP's address refusal. |
| LPG-07 | Dispatch this one native preview method using the same production `Arc<AttachmentsService>` on the existing HTTP connection execution path before `call_host_lane`. A closed Host-lane channel must not prevent a valid cached preview or private-address refusal; ordinary methods keep their existing Host-lane dispatch. Cold preview network work must not be enqueued on the single Host owner loop. |
| LPG-08 | Preserve serial cache read, fetch and publication within the existing AttachmentsService using one preview mutex shared by both service preview entry points. Concurrent native HTTP calls must not overlap the final-file cache writes; unrelated attachment methods and the Host lane do not take this mutex. No cross-process cache or parallel cold-fetch guarantee is introduced. |

The native caller remains responsible for its cache-directory selection.
This batch does not grant renderer, paired device or read-only credentials
authority to choose Host filesystem destinations. It reuses the existing
native authority rather than adding a caller-supplied trust flag or bypass.

## Implementation, review and evidence

Root serializes `host_gateway_api.rs`, `gateway_server.rs`, Host app composition, Coordinator
`carrier.rs`/`main.rs`, workflow and traceability changes. Use the existing
registry macro's separate native categories and existing carrier owner; no
new domain module or ADR is necessary.

Extend `gateway_server_contract.rs` with real loopback HTTP → actual service
fixtures, fresh cache data and real DevicePairingOwner-issued node/operator
tokens. Assert status, exact dispatch counts, metadata and unchanged bytes;
keep ordinary authorized-method controls and all existing native MCP tests.
Extend the existing Coordinator production-protocol fixture to prove Data
refusal and MainData forwarding with unchanged arguments/results; its fake
HTTP peer proves transport decisions only, not the attachment implementation.
Keep an explicit Control-carrier predicate oracle as well.

Use existing Actions jobs/caches for Host Gateway contracts and Coordinator
contracts. macOS consumes the already selected Gateway and Coordinator logs
without rerunning those targets. Linux adds the two existing Gateway targets
and the existing Coordinator contract/production-protocol targets using the
Host target directory/cache. The app manifest is portable and its closed-lane
fixtures need neither desktop initialization nor accounts; run the three
app preview tests in that Linux job as well to verify the changed production
composition while macOS remains queued. Retain the separate macOS gate.
Require the nine new Host positives, the new
carrier predicate positive and the existing shipping Coordinator test once
each; that binary test now also asserts native preview routing after joining
its controlled HTTP peer. Add closed-Host-lane controls at the actual app
composition helper to the already compiled macOS app-test lane and the
Linux app test binary; library and Coordinator results do not replace these
app controls, and Linux execution does not establish macOS/platform acceptance.
Require exact H/tree/U, complete commands,
exit codes and retained logs/artifacts. Freeze the complete candidate and
perform fresh independent source review before push/readback. No local or
htch-runtime execution is permitted.

Authority is the existing Fabushi producer and Host ownership contract,
plus LPIP-v1's fixed-U address-policy corroboration. This is OCRM R24/AC24
preservation of an existing product path, not new source read credit or a
whole-unit verification promotion. Real internet fetch/redirect/DNS pinning,
full desktop rendering/timing, other media methods, remote-host cache-path
portability, platform packages and G01–G12/AC01–AC24 remain open.

## Review correction before integration

**LPG-LANE-01 — FIX / IMPL_BUG (P1).** Frozen tree
`d6b5e09f1eb789b3d9c4be6fdcd5a40263890daa` exposes the method but leaves
`UnifiedGatewayApi::call` falling through to the single Host lane. A cold or
expired cache can then hold all later Host-lane requests behind sequential
DNS/HTTPS preview work, including shutdown input. Fresh-cache HTTP fixtures
alone cannot decide this property. Share the existing AttachmentsService Arc
into that API and route only this method on the already-owned connection
execution path. Preserve ingress authority, error mapping and ordinary
fallback dispatch. Add a causal closed-lane app oracle and obtain fresh
focused review; no new worker pool, queue or cache owner is authorized.

**LPG-CACHE-01 — FIX / IMPL_BUG prevention for the lane correction.**
The current preview writer truncates and writes the final cache pathname.
Moving calls from the serial Host lane onto HTTP workers without retaining
preview serialization would allow readers to see partial JSON and concurrent
writers to leave mixed bytes. Use a single mutex in the existing service and
a shared private helper covering read, fetch and publication. Preserve the
current serialization, ownership and cache format; do not add a keyed lock
registry or independently redesign atomic file replacement in this batch.
The focused review must inspect both service entry points and guard lifetime.

**LPG-CI-01 — FIX / TEST_FLAW in the incoming workflow.** Exact parent
`181cf49d4e14f08c1fc5daa9587928164ad09a12` runtime run `41526525909` failed
workflow validation before creating jobs or artifacts: its two media steps
declare both `NO_PROXY` and `no_proxy`, which Actions rejects as duplicate
mapping keys (original lines 381 and 1397). Retain one YAML environment key
and export its lowercase alias inside each bash command. Preserve every
inherited positive name and assertion. That parent has no runtime execution
evidence from this failed run; the corrected exact commit requires Actions.

Fresh independent security/lifetime and integration/reuse/performance source
reviews of tree `c52539d6de97c0a9144230b94e48c0af662cb138` covered all 14
changed paths and relevant callers/consumers and initially returned no findings.
The later concrete MCP oracle finding below supersedes that verification-
integration verdict; the production conclusions remain. Both used explicitly
configured `xhigh`; the reuse/performance lens
used `gpt-6.1-sol` under AGENTS. The later `1b9803d` reconciliation retains
identical production, test and workflow blobs; only lineage/review text and
the incoming MURL record differ. This is source-review evidence, not runtime,
macOS, cold-network, cancellation/draining or whole-product acceptance.

**LPG-MCP-ORACLE-01 — FIX / TEST_FLAW (existing shared composition test).**
The source-shape oracle `desktop_mcp_dispatch_precedes_host_fallback` accepts
only a direct three-argument `call_host_lane` tail. LPG-07 intentionally puts
the exact four-argument preview-or-lane helper at that tail, leaving the
preceding returned MCP dispatch unchanged, so the old oracle would reject the
valid production composition. Accept exactly either the original call or the
new helper with `&self.host_tx`, `&self.attachments`, `method`, `args`.
Retain the MCP owner, argument, return/error mapping and adjacent ordering
checks. Run all original ignored-result, wrong-owner, dead-delegate,
early-fallback, comment-only and test-only negatives against both forms;
add wrong helper/host/service/method/arguments/arity negatives. Require the
two MCP positives from the existing exact-H composition command on both
platforms. This repairs a stale source-shape instrument; it does not change
MCP behavior or substitute source assertions for the actual app helper tests.

Fresh focused review of the complete 15-path tree
`a8f5eee193115bec852ad16e958dc5725e652d44` found no remaining actionable
source issue after the MCP oracle amendment and incoming Automation merge.
Actual f890 Actions then identified a separate pre-admission fixture error;
its narrow correction and retained failed evidence are recorded as
`AUTO-ADMISSION-ERROR-ORACLE-01` in the existing cron/session spec. Neither
source approval nor the incoming media positives establish a new-H pass.
