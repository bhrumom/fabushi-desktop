# Public device join ingress admission (DJL-v1)

This is a bounded production contract under the existing Gateway and
`OCRM-RESP-DEVICE-PAIRING-JOIN-CODE-001` responsibility. GatewayServer remains
the ingress owner; DevicePairingOwner remains the only durable code/token
owner. No Agent runtime, authority store, scheduler or public subsystem is added.

## Authority and actual missing link

Fixed U is `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`,
tree `90c8aab39972da11cfad220361781d49e1bc6700`. Target discovery base is
`d53608f0823dbc15e2f2ab992eb31898fb462dc3`, tree
`7bc2315bb94b9a0638b7328e9999333697e91c78`. Its Gateway blob is
`916c1c1ac015699a2814647d86ae2dbb754cf61d`, including the incoming native
channel-credential gate. Its pairing owner includes DJC-v1, blob
`d82db64de7077e20b75a99366a21dc858a0984e7`; do not overwrite that transaction.

The current accept loop discards its kernel-provided peer address. Public join
requests reach optional bearer authentication and later redemption without an
ingress failure budget. Installing a check only at the late redemption branch
would still let a blocked caller reach pairing authentication/storage first.

Required fixed-U sources are:

| Source | Exact blob | Read/disposition |
| --- | --- | --- |
| `src/gateway/device-pairing-join-http.ts` | `44065a704fd65fc5ecdd9a4088aa55961791cf2e` | Complete public handler: serialized check, miss accounting, reset and response |
| `src/gateway/auth-rate-limit.ts` | `ee5bcdb23e3bad6dee0cdfbf955196bf63906d90` | Complete limiter: defaults, strict policy, bounds, eviction, expiry and disposal |
| `src/gateway/rate-limit-attempt-serialization.ts` | `8d121b586c078f62e3f9ce5b337a0a0d4f65da57` | Complete per-identity serialization wrapper |
| `src/plugin-sdk/keyed-async-queue.ts` | `5dcd994ba177bad05fa8aea00de0e205645a6b27` | Complete reservation, failed-settlement and current-tail retirement contract |
| `src/shared/deferred.ts` | `1d2a71bd573394a16409cc3a013d34264437a377` | Complete promise helper; not a new Rust product runtime requirement |
| `src/gateway/server-runtime-state-prepare.ts` | `ef3a0d3227218670b1a4fd4db387340ae934840b` | Targeted production policy/construction/transport call chain, not full-file read |
| `src/gateway/server-http-request.ts` | `b47c34d3d5f9da4693166a49882cdcfa42de8d78` | Targeted prepared-attribution and public namespace consumer, not full-file read |

The actual U composition creates `browserAuthRateLimiter` from the existing auth
policy with `exemptLoopback: false`, then passes it as `joinRateLimiter`.
The public handler uses the dedicated device-join scope and serializes an entire
attempt for one prepared identity. This contract implements the currently
missing direct-socket default policy; it does not claim full proxy/configuration
compatibility or promote historical reading facts.

## DJL-01: trusted identity and existing entry

Carry the accepted socket IP through both plain and TLS connection paths.
Normalize IPv4-mapped IPv6 to the same IPv4 key; source ports do not distinguish
clients. Host, Forwarded, X-Forwarded-For, X-Real-IP, authorization and Tailscale
headers cannot supply an identity. Current Fabushi has no trusted-proxy policy;
proxied clients therefore share the proxy socket's budget.

After the existing Origin and Host checks, recognize the existing `/j`
namespace independently of method using the existing path parser. Handle it
before optional bearer authentication, nonce/revision handling and command
dispatch. A public join code is the credential for this route; optional headers
must neither grant authority nor cause a blocked request to access another
owner. Keep every non-join authentication, role/scope, Memory and Channel gate.

Invalid method, query (including an empty query marker), empty/invalid/slashed
shortcode produces opaque 404 without calling authentication or redemption.
Preserve the current context-prefix path rule and valid shortcode validation.
All responses produced by this join handler use `Cache-Control: no-store`.
Earlier Origin/Host refusals retain their existing behavior and are not claimed
as part of that response guarantee.

## DJL-02: default policy and result accounting

Apply the real strict-loopback defaults: ten misses in a 60-second rolling
window earn a 300-second lockout. The tenth miss is still 404; following
attempts are 429 with `{"error":"rate_limited"}` and a positive, ceiling-second
`Retry-After`. Denied checks do not extend the earned deadline. At the exact
window cutoff the old attempt expires; at the lockout deadline clear the served
history before admitting and recording a fresh attempt.

Serialize check, validation/redemption and outcome accounting for one peer,
including concurrent requests with different codes and source ports. Check the
clock after waiting for that peer's turn and record a new failure at settlement,
not at queue insertion. Release the peer lock before writing the network reply.
Different peers must not wait behind a global map lock held across SQLite or
socket operations.

Preserve U's per-identity reservation order: once two attempts have reserved a
place, the later reservation cannot overtake the earlier one. This orders queue
reservations, not bytes arriving on different sockets or thread scheduling before
reservation. Use private turn state within the existing Gateway limiter; do not
add a second scheduler. Refusal, stop and unwinding must settle or invalidate the
reserved turn and wake its waiters without admitting through poisoned state.
Counter exhaustion must fail closed rather than wrap and duplicate a live turn.

Successful `Some(Value)` resets only that peer. Retain DJC's legacy arbitrary
JSON success contract, including scalar/null payloads; this is not a claim to
implement U's stricter setup decoder. `Ok(None)` records a miss. An admitted
storage error keeps its actual error response and does not become a synthetic
miss or success reset. A blocked request returns 429 before any authentication,
authorization or redemption callback, even if storage would be unavailable.
No automatic retry or replay is added.

Use Rust's monotonic `Instant` for this transient elapsed-time policy. U uses
its Gateway scheduler's wall clock; exact clock-jump equivalence is not claimed.
Keep DevicePairingOwner's existing durable expiry clock and commit rules intact.

## DJL-03: bounded ownership and capacity

Keep private state in the existing Gateway server generation, shared with its
connection workers. Bound tracked keys at 10,000 and each peer's retained
failure history at the policy threshold. Expire state lazily during access or
capacity admission; do not introduce a pruning thread, timer or scheduler.

Never evict an unexpired lockout or an entry with a live/queued attempt owner.
An old guard must not finish into an orphaned entry while a replacement admits
the same peer under a fresh budget. Reclaim expired idle entries first and
then idle unlocked entries if capacity requires it. If no entry is safely
reclaimable, refuse a new peer with opaque 429 and positive Retry-After before
owner access; tracked peers retain their own budget. The additional bound on
live serialization ownership is target hardening, not a claim of identical
U overflow timing. Poisoned or unavailable admission state must fail closed.

## DJL-04: stop, disconnect and restart

Use the existing stop signal. Check it before admission and after acquiring a
queued peer's turn; a queued request must not newly enter the owner after that
stop observation. Return an explicit no-store unavailable response without
recording a fake miss. An operation already entered before stop may complete;
account its real outcome and do not claim rollback when response delivery fails.

A new Gateway generation starts with a fresh transient budget. It must not
clear or reuse old generation state while old workers still retain it, and it
must not clear durable join rows or the paired-device revision. Existing server
shutdown joins only the listener: full connection draining is not claimed.

## DJL-05: executable oracles and integration

Use deterministic private state/clock controls for exact window, lockout and
ceiling boundaries, success isolation, mapped addresses, bounded capacity,
locked/live non-eviction and poison refusal. Use causal synchronization for
per-peer serialization, independent-peer progress and queued stop admission;
sleep alone is not an admission witness.
Require a causal reservation-order control with multiple queued attempts and
mixed success/failure outcomes; a mutual-exclusion-only assertion is insufficient.

The actual HTTP contracts must run the real Gateway and file-backed pairing
owner. Require the default ten-miss threshold, no-store/Retry-After, malformed
namespace handling, forged-header/port budget continuity, and 429 precedence
with zero authentication/redemption callbacks and an independently unburned
valid code. Require success reset, unchanged stored rows on refusal, actual
storage-error accounting, concurrency at the last available attempt, restart
generation isolation and no automatic redemption replay after lost response.
Use only owned temporary fixtures; retire request/API owners before successful
cleanup and retain unresolved fixtures on failure.

Keep DJC8 and DPR20, the existing public join and TLS controls, Memory and the
latest Channel authorization controls selected in the existing Actions lanes.
The root integrator serially adds affected commands and exact positive names,
single complete summaries, zero command/capture admission, H/tree/U/run/attempt,
source/input hashes and raw-log hashes. A successful unrelated target cannot
stand in for an omitted new case. No local or htch executable validation.

## Remaining whole-migration responsibilities

Configurable upstream auth-rate-limit settings, prepared trusted-proxy and
managed Tailscale attribution, other auth scopes, complete setup schema/issuer
UI, real companions and platforms, unknown COMMIT/crash, whole-server draining,
soak/packages/upgrade and final G01-G12/AC01-AC24 acceptance remain open. This
contract does not exempt them from the migration. Scoped source authority,
coverage and all eight global completion flags are unchanged.
