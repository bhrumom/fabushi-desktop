# Plugin SDK Channel Feedback, Inbound Debouncing & Ingress Coordination Dossier

## 1. Scope & Source Attribution
- **Target Files (8 files in `src/plugin-sdk/`)**:
  - `src/plugin-sdk/channel-feedback.ts` (71 lines)
  - `src/plugin-sdk/channel-inbound-debounce.ts` (5 lines)
  - `src/plugin-sdk/channel-inbound.test.ts` (300 lines)
  - `src/plugin-sdk/channel-inbound.ts` (335 lines)
  - `src/plugin-sdk/channel-ingress-runtime.test.ts` (90 lines)
  - `src/plugin-sdk/channel-ingress-runtime.ts` (61 lines)
  - `src/plugin-sdk/channel-ingress-test-runtime.ts` (7 lines)
  - `src/plugin-sdk/channel-join-intro-runtime.ts` (8 lines)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugin-sdk).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGIN-SDK-CHANNEL-INBOUND-INGRESS-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 WhatsApp ACK Reactions & Tool Emojis
- **WhatsApp ACK Decision Gate**: Verifies direct vs group mode (`always`, `never`, `mentions`), checking mention status and group activation flags.
- **Contextual Tool Status Emojis**: Maps tool classification tokens (`BUILD_TOOL_TOKENS`, `CODING_TOOL_TOKENS`, `CONCIERGE_TOOL_TOKENS`, `DEPLOY_TOOL_TOKENS`, `WEB_TOOL_TOKENS`) to distinct UI emojis (`💻`, `🔨`, `🚀`, `🔍`, `🌐`, `⚡`).

### 2.2 Ingress Lifecycle Fan-in & Coordination
- **Multi-Source Claims**: `fan_in_channel_ingress_lifecycles` coordinates multiple durable message ingress claims into a single execution lifecycle.
- **Cascade Error Release**: If an adoption throws midway through a chain of lifecycles, all remaining unadopted claims are cleanly notified via `on_failed` with the active error.
- **Heartbeat Interval Resolution**: Selects the minimum positive finite interval from all contributing lifecycles.
- **Claim Finalization**: Exposes unified `settle`, `abandon`, and `cancel` semantics with once-only idempotency guards.

### 2.3 Identifier Authentication Levels
- **Hierarchy Comparison**: Strict ordinal comparison across `Mutable < Asserted < Authenticated < Verified < Signed`.

### 2.4 Inbound Debouncing, Envelopes & Room Joins
- **Inbound Debouncer**: Prevents duplicate processing by checking elapsed duration since the last message from a given sender key.
- **Envelope Serialization**: Standardizes format `[{channel}] {from} ({timestamp_ms}): {body}`.
- **Room Join Outcome**: Emits join confirmation text including inviter metadata.

## 3. Rust Equivalents Implemented
- [`source/host/src/extensions/plugin_sdk/channel_feedback.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_feedback.rs)
- [`source/host/src/extensions/plugin_sdk/channel_ingress.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_ingress.rs)
- [`source/host/src/extensions/plugin_sdk/channel_inbound.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/channel_inbound.rs)

## 4. Contract Test Verification
- [`source/host/tests/openclaw_plugin_sdk_contract.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/tests/openclaw_plugin_sdk_contract.rs):
  - `test_ocrm_plugin_sdk_channel_feedback_inbound_ingress`

## 5. Audit Status
- Mapped Files: 8 added (Total mapped: 793 / 58,750).
- Unclassified Remaining: 57,957.
- Global completion flags: all 8 remaining `false` (fail-closed).

## 6. Sender-normalization boundary (2026-10-29)

The locked `src/plugin-sdk/allow-from.ts` oracle normalizes the sender value
through string coercion, trimming and lowercase comparison before applying the
allowlist. The Host ingress owner must preserve that boundary for direct-message
allowlist decisions: a sender handle with transport whitespace is equivalent to
the same trimmed handle, while an empty sender remains denied and wildcard
semantics remain explicit. This is a local ingress authorization hardening; it
does not register an external channel or establish production channel delivery.

The locked `channel-ingress-runtime.ts` lifecycle also requires a single
terminal handoff: repeated settle, abandon, cancel, or direct lifecycle
callbacks must not release the same durable claim twice. If adoption fails at a
claim, that claim and every later unadopted claim receive the failure cleanup,
while the original adoption error is preserved. The Rust fan-in owner now keeps
that once-only fence in its state machine; this remains scoped lifecycle
evidence and does not claim external channel production wiring.

## Fixed-U production idle handoff — 2026-11-02

Contract `OCRM-CHANNEL-IDLE-HANDOFF-v1` uses U
`1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`, and target base
`81d48636b06d20f8bec551cdc334fe40f31e614b`. The historical 61-line SDK
subject above is preserved as history; it is not this fixed-U implementation.
The independent owner investigation read these exact files completely:

| Fixed-U source | Blob | Lines |
| --- | --- | --- |
| `src/plugin-sdk/channel-ingress-runtime.ts` | `1b1817a655b06c0f5d2a3b7438ddf35dcfe82719` | 1–358 |
| `src/channels/message/ingress-monitor.ts` | `9d37fc03b0807d9f3b9f06e69662a7ac2d631a41` | 1–743 |
| `src/channels/message/ingress-monitor-admission.ts` | `0e83828d0d45af2d3e17d7abb7f8ea470c3f015c` | 1–235 |
| `src/channels/message/ingress-monitor-tasks.ts` | `f8c4bdb6c6374a01c282a8cd4ffb24c5416c59ba` | 1–80 |
| `src/channels/message/ingress-monitor-types.ts` | `d153c0803b67a2b2d3cc7128906c0aaef0e440d6` | 1–137 |
| `src/channels/message/ingress-retry-policy.ts` | `92cb60517f91bd9fa134a44e1953aec6904c3409` | 1–179 |
| `src/plugin-sdk/channel-ingress-runtime.test.ts` | `1621195d008ccda9aeb6d2bfc380fd2db31025ef` | 1–283 |
| `src/channels/message/ingress-monitor.test-harness.ts` | `10d2ff59597795a3b1f37540df9d0f5a274055a7` | 1–84 |

`ingress-monitor.test.ts`, blob `7f2d1dffce0db14013bc1991631cde4f2a210de3`,
was read only at 1–58, 114–194, 290–332, 514–532, 624–687 and 903–980.
It remains a partial read. The integrator separately re-read monitor 390–535
and SDK 1–140 from the same fixed GitHub blobs. The monitor's coalesced idle
wake, final repump and active ownership fences establish the relevant invariant:
new ingress cannot be stranded by an old drain returning. Its durable queue,
plugin-instance authority, admission/transport acknowledgment and stop custody
are broader contracts, not supplied by this bounded Rust queue repair.

The actual Host `wakeForInbound` enqueues and begins revival under one
`BackgroundWakes` mutex. Its worker previously consumed an empty batch and
cleared revival under two separate locks. The same queue owner now supplies an
atomic take-or-end operation; production drain control returns on idle without
clearing a successor's ownership. Nonempty pause exits keep their prior cleanup.
Real owner mutex-ordering tests and a production-control test pause a predecessor
after idle release, admit a successor, then verify the predecessor cannot erase
that claim. They are authored tests until exact-H Actions executes them.

Append failures still lack typed known/unknown commit outcomes; deferred batches
lack autonomous inbound rearm; detached workers lack a dispose/recreate generation
fence. Persistent ingress, transport receipt, actual provider delivery and restart
acceptance remain open. No machine inventory, historical read state, scoped
authority approval or global completion flag is promoted by this source review.
