# OCRM Technical Dossier — Unit 10: Channels & Outbound Delivery (`OCRM-UNIT-CHANNELS-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M14/M15 (Channel Ingress, Deduplication, Thread Routing & Outbound Delivery Subsystems)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
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
   - **Rust Owner**: `source/host/src/connectors/channel_delivery.rs:split_channel_message`.
   - **Oracle**: `OCRM-CHANNEL-O06`.

6. **`OCRM-RESP-CHANNEL-ACCESS-CONTROL-001`**:
   - **Behavior**: Channel sender access control list (ACL) evaluation, wildcard resolution, normalized candidate matching, and webhook HMAC signature replay protection.
   - **Rust Owner**: `source/host/src/connectors/channel_delivery.rs:ChannelAclGate`.
   - **Oracles**: `OCRM-CHANNEL-O11`, `OCRM-CHANNEL-O12`.

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
