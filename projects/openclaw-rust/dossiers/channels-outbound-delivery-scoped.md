# OCRM Technical Dossier — Unit 10: Channels & Outbound Delivery (`OCRM-UNIT-CHANNELS-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M14/M15 (Channel Ingress, Deduplication, Thread Routing & Outbound Delivery Subsystems)  
**Upstream live discovery subject (not authority)**: `openclaw/openclaw@ce908375fed062dd75abab62d57fc840b96a144e` (tree `f2db36d9a98e23753ce1130a84cdae7a4c57488d`); the file/blob rows below retain their historical source-audit identities and require exact-head revalidation before final closure.
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `codex/openclaw-rust-fusion-20261031`
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 core channel ingress, deduplication, thread routing, attachment handling, and outbound delivery files (totaling 1,300 lines) were audited line-by-line and mapped into canonical shards:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0027` | `src/channels/inbound-debounce-policy.ts` | `edc265f0c352c44dcf816f48200c1d65a33d6a14` | 59 | 1..59 (full-file) | `OCRM-RESP-CHANNEL-INBOUND-DEBOUNCE-001` |
| `part-0027` | `src/channels/draft-streaming-chunking.ts` | `ca99866dc4c2b40b7d735f3799e8fee80e24fb5b` | 50 | 1..50 (full-file) | `OCRM-RESP-CHANNEL-STREAMING-CHUNKING-001` |
| `part-0027` | `src/channels/allowlist-match.ts` | `0735987eeec5980109dbec31a98af08c93f5e69c` | 107 | 1..107 (full-file) | `OCRM-RESP-CHANNEL-ACCESS-CONTROL-001` |
| `part-0027` | `src/channels/channel-config.ts` | `871d6288d54e149616b1d657862a78b0bf3b893e` | 146 | 1..146 (full-file) | `OCRM-RESP-CHANNEL-DELIVERY-PORT-001` |
| `part-0028` | `src/channels/thread-bindings-policy.ts` | `849ad93c82d1c07d67dac8679d24b457b2e8f135` | 185 | 1..185 (full-file) | `OCRM-RESP-CHANNEL-THREAD-BINDINGS-001` |
| `part-0028` | `src/channels/progress-draft-compositor.ts` | `6da9c87e6231f48df2852bf2026a18cb7eec3805` | 753 | 1..753 (full-file) | `OCRM-RESP-CHANNEL-ATTACHMENT-RESOLVER-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-CHANNEL-INBOUND-DEBOUNCE-001`**:
   - **Behavior**: Inbound message debouncing gate, immediate bypass for control command messages (abort/stop/status) and media payloads, and channel-scoped timing resolution.
   - **Rust Owner**: `source/host/src/connectors/channel_delivery.rs:InboundChannelDeduplicator`.
   - **Oracles**: `OCRM-CHANNEL-O01`, `OCRM-CHANNEL-O08`.

2. **`OCRM-RESP-CHANNEL-THREAD-BINDINGS-001`**:
   - **Behavior**: Thread binding lifecycle, account-level idle timeout and max age resolution, thread spawn policy resolution (isolated vs fork), and conversation affinity routing.
   - **Rust Owner**: `source/host/src/connectors/channel_delivery.rs:ChannelThreadRouter`.
   - **Oracle**: `OCRM-CHANNEL-O02`.

3. **`OCRM-RESP-CHANNEL-DELIVERY-PORT-001`**:
   - **Behavior**: Channel config entry precedence matching (direct, parent, wildcard fallback), nested allowlist gating, and host channel delivery port lifecycle registration.
   - **Rust Owner**: `source/host/src/connectors/channel_delivery.rs:HostChannelDelivery`.
   - **Oracles**: `OCRM-CHANNEL-O03`, `OCRM-CHANNEL-O04`.

4. **`OCRM-RESP-CHANNEL-ATTACHMENT-RESOLVER-001`**:
   - **Behavior**: Streaming progress draft composition, reasoning/commentary line formatting, and local attachment 50 MiB upload cap enforcement with safe mime resolution.
   - **Rust Owner**: `source/host/src/connectors/channel_attachment.rs:resolve_channel_attachment`.
   - **Oracle**: `OCRM-CHANNEL-O05`.

5. **`OCRM-RESP-CHANNEL-STREAMING-CHUNKING-001`**:
   - **Behavior**: Draft streaming preview chunk size clamping, break preference resolution (paragraph, newline, sentence), and code fence boundary preservation.
   - **Rust Owner**: `source/host/src/extensions/plugin_sdk/channel_outbound.rs:ChannelDraftStreamingChunking::resolve`.
   - **Oracle**: pending; the historical `OCRM-CHANNEL-O06` fixture covers the separate Host transport splitter and is not an oracle for this configuration resolver.

The shared outbound router now invokes `split_channel_message` for every text
delivery before handing chunks to the registered provider. The implementation
uses the conservative 4,000-character Host boundary and reopens fenced code
blocks between chunks. This is a real production call path, so the row is
`wiring=wired` for the separate Host transport-splitting behavior only. It is
not evidence for the upstream `draft-streaming-chunking.ts` responsibility:
that source resolves account/channel configuration and break preferences, while
the Host splitter has no configuration input and only preserves code fences and
the fixed 4,000-character transport boundary. The ledger therefore points the
upstream responsibility at `ChannelDraftStreamingChunking::resolve`, records
`implementation=implemented`, `wiring=not-wired`, `verification=not-run`, and
`acceptance=pending`, and retains the Host splitter as a separate unregistered
mapping candidate until its exact upstream source is re-established.

6. **`OCRM-RESP-CHANNEL-ACCESS-CONTROL-001`**:
   - **Behavior**: Channel sender access control list (ACL) evaluation, wildcard resolution, normalized candidate matching, and webhook HMAC signature replay protection.
   - **Rust Owner**: `source/host/src/connectors/channel_delivery.rs:ChannelAclGate`.
   - **Oracles**: `OCRM-CHANNEL-O11`, `OCRM-CHANNEL-O12`.

`ChannelAclGate` now owns normalized allowlist/denylist evaluation, including
the deny-overrides-allow rule, `*` wildcard, default-deny empty allowlists, and
ordered candidate match metadata (`id`, name/tag/prefixed/slug/localpart
sources). The connector also exposes strict standard HMAC-SHA256 verification
with constant-time comparison. This closes only pure security helpers; the
inbound channel producer, security audit emission, replay nonce/receipt
storage, and production webhook path remain unwired. The legacy contract
fixture's `secret || payload` digest is not treated as HMAC evidence.

The connector now also owns the Rust `InboundChannelDeduplicator` and
`ChannelThreadRouter` contracts: channel cursors and message IDs are admitted
idempotently, and a normalized `(channel, thread)` pair resolves to one stable
conversation scope. These helpers are not marked wired because no production
inbound channel producer or transcript/session consumer has been identified.

### Exact-source production reachability audit — 2026-10-31

The current Fabushi source was rechecked at the fixed OpenClaw subject recorded
by the project source-of-truth. The only production construction found for the
channel delivery owner is `start_production_extensions` in
`source/host/src/host_production_extensions.rs`, where a fresh
`ChannelDeliveryRouter` is wrapped by `HostChannelDelivery`. The router's
provider lifecycle methods are public, but repository search found no production
caller that registers or replaces a concrete `ChannelDeliveryPort`.

The only application dispatch caller is `deliverToChannel` in
`source/host/app/src/main.rs`. It constructs an outbound message and invokes
`HostChannelDelivery::deliver` on a worker thread. This proves an outbound
failure-aware path only; the default router remains empty and therefore returns
the unregistered-provider error for every channel address until an external
provider registration occurs.

No production caller was found for `ChannelAclGate`,
`verify_webhook_hmac_sha256`, `InboundChannelDeduplicator`,
`InboundChannelDebounceGate`, or `ChannelThreadRouter`. No channel webhook
receiver, inbound event adapter, receipt/cursor store, thread-to-session
binding consumer, or turn-dispatch handoff is present in this target slice.
Consequently the four inbound responsibilities remain `wiring=not-wired` and
their acceptance remains pending. The helper implementations are complete as
source-level Rust contract slices, but helper presence, unit contracts, and the
outbound `deliverToChannel` path must not be promoted to production inbound
acceptance.

### Inbound wake path versus thread binding — 2026-11-01

The current Host does have a real inbound wake path, but it is not a thread
binding path. `wakeForInbound` accepts an opaque envelope and queues it under
an agent ID. The revival worker persists channel address, sender, text, and
timestamp into the agent transcript, builds the inbound prompt, and invokes
the existing background turn owner. The coordinator's
`agent-inbound-wake-request` route adds admission and redrive behavior.

Neither side calls `ChannelThreadRouter::resolve_or_create`, carries a
thread identity into `session_conversation_state`, or persists a canonical
thread-to-conversation scope. Therefore this is evidence of a real
producer→transcript→runner chain for generic inbound wakes, while the
thread-affinity producer and session consumer required by
`OCRM-RESP-CHANNEL-THREAD-BINDINGS-001` remain absent. Adding a call at the
opaque `wakeForInbound` boundary without a provider-owned thread identity,
conversation selection contract, and persistence owner would guess the
missing interface and create an unreviewed routing authority.

### Current internal outbound route — 2026-11-02

At target `e70a72574c108f8b7704eb45e5109875a24eddb9`, direct source tracing
finds that the real Runner `ProductionSendMessageSink` appends through Session/
Transcript and sends `deliverToChannel` through `call_host_lane`, while the main
receiver calls a generic dispatcher without that route. The external Gateway
handler exists but is not reached by this internal path. The older textual
presence checks therefore do not establish this shipping Sink-to-delivery edge.
The bounded repair is specified in
`docs/specs/channel-outbound-host-lane.md` (`OCRM-CHANNEL-HOST-LANE-DELIVERY-v1`).
Preserve the existing delivery/router, failure tray and durable wake owners;
queued acknowledgement is not an accepted transport result. The shared concrete
handler, internal main receive-arm wiring and five real-owner oracles are now
authored on integration base `7a0254d4133080877cbfb2d9d4831acb231e673f`, retaining
incoming atomic Gateway/A2A append, the separate test-compilation repair and
the existing MCP account-listing retirement work.
Independent source review and this batch's exact-H execution remain pending.

Fixed-U source for this boundary: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`,
`src/infra/outbound/message.ts`, blob
`5032acef657250cbce6b85435f5215237825163b`, read in full at lines 1–567. Its
Gateway-owned direct branch avoids re-entering Gateway dispatch, and its result
types distinguish delivery outcomes. Its idempotent timeout reattachment does
not authorize replay of the target's String-error transport. This exact file
read does not upgrade the six historical rows above or imported durable-runtime
dependencies to current complete review. Those dependencies, real provider
registration, channel protocols and full acceptance remain open.

### Pending failure restart custody — CWR-v1, 2026-11-02

On exact base `0533ca72870a2794c2db32959b9d01427abf325a`, the same production
chain exposes a second defect: recovery uses the fresh-failure producer and
then clears ChannelDelivery on enqueue, although missing provider or upgrade
pause only requeues. CWR-v1 in the existing Host-lane spec now requires exact
persisted-marker admission, full metadata through the existing queue, checked
whole-image store mutations, and settlement only by existing terminal owners.
The Host adapter, store/rearm and manager/health wiring plus real-owner
restart/refusal oracles are authored. Independent frozen review and exact-H
Actions are required before any bounded verification claim. Existing OCRM
rows and RTM retain their verification/acceptance states and all eight false
flags. The fixed-U outbound source read above is reused without upgrading
historical reads or imported-dependency closure. No transport retry, real
provider acceptance, unknown-effect reconciliation or cross-process writer
arbitration is claimed by this batch.

### Inbound debounce implementation slice — 2026-10-31

The existing connector owner now also contains
`ChannelInboundDebouncePolicy` and `InboundChannelDebounceGate`. The policy
resolves a bounded default or normalized per-channel delay, while the gate
returns deterministic `Immediate` or `Debounced { retry_after }` decisions for
caller-supplied observation instants. Control-command and media classifications
bypass the text window; only admitted text updates the per-channel timestamp.
State is bounded and invalid channels/delays fail closed.

This is implementation/source-contract evidence only. The gate does not own a
transport timer, inbound adapter, session, or turn dispatch. No production
inbound producer or downstream consumer has been identified, so
`OCRM-RESP-CHANNEL-INBOUND-DEBOUNCE-001` remains
`wiring=not-wired`, `verification=not-run`, and `acceptance=pending`.

---

## 3. Falsifiable Test Oracles (`source/host/tests/openclaw_channel_delivery_contract.rs`)

1. **`OCRM-CHANNEL-O01`**: `test_ocrm_channel_o01_inbound_deduplication`
2. **`OCRM-CHANNEL-O02`**: `test_ocrm_channel_o02_thread_routing_affinity`
3. **`OCRM-CHANNEL-O03`**: `test_ocrm_channel_o03_delivery_port_registration_lifecycle`
4. **`OCRM-CHANNEL-O04`**: `test_ocrm_channel_o04_dead_letter_fallback`
5. **`OCRM-CHANNEL-O05`**: `test_ocrm_channel_o05_attachment_50_mib_limit`
6. **`OCRM-CHANNEL-O06`**: `test_ocrm_channel_o06_message_chunking_code_fence_preservation`
7. **`OCRM-CHANNEL-O07`**: `test_ocrm_channel_o07_rate_limiter_backoff`
8. **`OCRM-CHANNEL-O08`**: `test_ocrm_channel_o08_receipt_settlement`
9. **`OCRM-CHANNEL-O09`**: `test_ocrm_channel_o09_dead_letter_replay`
10. **`OCRM-CHANNEL-O10`**: `test_ocrm_channel_o10_reconnection_backoff`
11. **`OCRM-CHANNEL-O11`**: `test_ocrm_channel_o11_webhook_hmac_verification`
12. **`OCRM-CHANNEL-O12`**: `test_ocrm_channel_o12_acl_gating`

---

## 4. Invariant Verification & Global Conservation

- **Exact Mapped Paths**: 118 (cumulative across Units 1–10)
- **Omitted Known**: 0
- **Total Tracked Paths**: 58,750
- **Unread / Unclassified Paths**: 58,632
- **Strict Conservation**: $118 + 58632 = 58750$.
- **All Global Completion Flags**: `false` strictly preserved.
