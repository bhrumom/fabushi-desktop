# Human message retry through the existing Session owner

Status: active / bounded implementation contract; full migration unaccepted
Spec ID: NMR-HUMAN-RETRY-01
Parent: FBCP-001 r8 / TDRP-001 r10 / FMEP-001 r2
Discovery base: afb34b49c5964524a887088f1095070b75cfdf8b
Fixed U: f985723b552b30fe1c17d377d57f44e61a15a8c4

## Problem and ownership

The current `ProductionRenderer::resendFailedSend` makes a fresh nonce, drops
the reply reference and sends projected attachment paths through the staging
commit bridge again. An ambiguous accepted send can become a second message;
already committed attachments cannot be committed through that bridge again.
NMR-51 records this dependent consumer gap. The canonical durable intent is
already in the existing Human Session/SandAgentDb, including uploaded resource
identities, original nonce, content, reply and dispatch errors.

Keep the existing Retry action, ConversationTranscript, account lifecycle,
Session gateway, native client, transcript events and acknowledgement projection.
No new message store, queue, resource owner or service endpoint is introduced.
Agent retries and all Agent/Coordinator/Host/Runner/Tools/approval/task/artifact/
Plugins/MCP/Computer/Automations capabilities retain their existing behavior.

## Command and invariants

- HRR-01: Add the explicit authenticated Host command `retryHumanMessage`
  with exactly `conversationId` and `clientNonce`, both nonempty unpadded
  strings; nonce maximum is 200 UTF-8 bytes. Reject extra payload, sender,
  attachment, reply or forward parameters. Admit only the explicit command in
  the existing gateway and Coordinator contract. The renderer cannot provide
  replacement content or reconstruct durable resource/reply state.
- HRR-02: Read the current authorized conversation and uniquely matching
  local-Human message from SandAgentDb. The two distinct unpadded participants
  must agree with its existing deterministic direct-conversation ID; persisted
  dispatch resources must satisfy the existing resource identity/size boundary.
  A retryable unsent row must be a Human
  message with `human-message:<nonce>` identity and pending/dispatching state.
  A valid already settled canonical row returns unchanged without another
  upload or POST. Reject absent, duplicate, foreign, malformed or conflicting
  identities before HTTP or creating/replacing an intent. Preserve all data.
- HRR-03: Ordinary retry reuses the actual durable nonce, text, current reply,
  composed time, attachments and uploaded dispatch resources through the
  existing append/settlement implementation. Recheck existence and source
  ownership under its staging lane. Keep the existing release-before-POST and
  credential-fenced settlement boundaries; no conversation lock spans the
  explicit retry's message POST wait. Resource preflight/upload keeps the
  existing serialized staging boundary; this batch does not claim concurrent
  resource upload and conversation mutation. Rapid same-target retries share a short-lived
  in-process retry lane, while persisted/server idempotency remains authority.
- HRR-04: Source-owned forwarding must never become ordinary resend, including
  explicit send of a marked row whose forward context was lost. Use the existing
  fail-closed forward-ownership classifier. When its original readable context
  and current account/issuer identify the original batch/target, retry only that
  target through `deliver_forward_batch`, preserving source authorization and
  pinned bytes. Missing, changed, unsupported or unresolved source ownership
  remains a visible retained failure; generic resend cannot repair it by dropping
  provenance. Existing source-batch ordinary-sync recovery remains authoritative.
- HRR-05: The Host-admitted Human Retry UI sends only the two durable identities, never calls
  attachment staging or makes a new nonce, and never clears or replaces the
  current composer draft. It keeps one in-flight UI attempt per account lifetime,
  conversation and nonce. Restore a visible failed state on current failure;
  apply only a matching authoritative result through existing transcript and
  acknowledgement owners. Host-persisted pending `lastDispatchError` projects
  as failed for local Human rows after reload/restart; actual dispatching stays
  pending and a canonical sent row never becomes failed from an old error.
- HRR-06: Account/logout/unmount and conversation-selection generations fence
  retries before the actual Coordinator request frame, after each await and in
  deferred React state updaters. A late result/error cannot mutate another view,
  an A→B→A replacement lifetime, roster, notices or drafts. Cancelling UI lifetime
  does not erase a Host-admitted durable send; normal transcript recovery later
  exposes its outcome. Reuse the NMR-50 optional current-scope admission contract
  at the existing client; do not create a second transport or account authority.
- HRR-07: A failure before the initial Host request is a retained Composer
  submission, not yet a durable Host intent. Preserve that existing queue's
  original submission object/nonce/reply/time and each successfully committed
  attachment. Retry resumes only uncommitted attachments and the original
  initial send. The real Coordinator boundary marks an intent possibly admitted
  immediately before posting its frame; once marked, every repeat uses the
  identity-only Host retry. Never infer non-admission from a timeout, retry with
  a fresh nonce, or rebuild payload from a transcript projection. Keep explicit
  retry scope admission across staging, ready waits and queue callbacks, and do
  not restore/clear/replace the user's current draft during the retry. Existing
  Agent submission and retry behavior remains unchanged.

## Implementation and verification

Root integration owns the shared Coordinator/parser/gateway contract, Session
implementation, workflow selection and canonical ledger. The renderer patch may
be developed separately after these command semantics are fixed. Preserve all
concurrent changes by exact path identities and non-force expected-head CAS.

- HRR-AC-01: Existing real Host/HTTP/SQLite fixtures prove attachment+reply
  retry after ambiguous dispatch/restart with the same nonce and previously
  uploaded resource, one canonical message and no duplicate upload. A subsequent
  canonical retry produces no HTTP. Missing/foreign/corrupt/duplicate inputs and
  extra command fields produce no dispatch or replacement row.
- HRR-AC-02: Existing forward fixture proves source policy denial before resource
  or message I/O, retained target identity/error, source-authorized retry through
  the original batch and refusal of ordinary send after context loss. Keep all
  existing forward snapshot, ownership, credential and process recovery tests.
- HRR-AC-03: Tests exercise the actual production retry controller/projection
  and Coordinator port boundary: same identities/no staging, rapid double click,
  malformed settlement, failure then retry, selection/account ABA, queued-ready
  account replacement, late error and deferred updater. Preserve Agent retry
  behavior and all existing renderer/Transcript/Chromium checks. Build the actual
  renderer and Host through the existing workflow; packaged visual/a11y remains
  a separate final gate, not proven by controller tests.
- HRR-AC-04: Actual Composer queue, attachment-commit adapter and Coordinator
  port controls prove partial attachment commit failure and pre-frame disconnect
  can retry the original submission, while any possibly posted send uses only
  durable retry. Verify no duplicate commit of successful attachments, original
  nonce/reply/time/content, current draft retention, queued/double-click behavior
  and account/selection cancellation before and after awaits.

All executable validation is GitHub Actions only on the frozen containing H.
Retain the actual afb34 compile failure (E0382 at the reaction-reopen cleanup),
its correction already integrated at106bb95 and all still-valid scoped results.
Run affected compile/contract/state/restart/HTTP/renderer checks once per H;
record actual selected counts and failures without lowering existing gates.

## Migration, recovery and acceptance boundary

No storage schema or data migration is required. Old pending rows keep their
existing retry data. A malformed legacy/forward record is preserved and reported;
no nonce, source ownership or historical byte proof is fabricated. Rollback is a
normal code revert with all stored messages/resources retained; it reopens the
old UI retry gap and must not be described as equivalent recovery.

Update the existing FBCP-NMR-001 ledger row, owner inventory/matrix, RTM and STATUS
with separate authored/executed/accepted evidence. Unpublished earlier round1
stale-progress patches are superseded by current4f1070/94dc code and are not
replayed. Full source/legal, real service, supported signed packages, complete
UI/a11y/timing/security/performance/soak/upgrade/rollback/Bot regression and
independent ACCEPT remain mandatory and open.
