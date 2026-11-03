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
  local-Human message from SandAgentDb. A retryable unsent row must be a Human
  message with `human-message:<nonce>` identity and pending/dispatching state.
  A valid already settled canonical row returns unchanged without another
  upload or POST. Reject absent, duplicate, foreign, malformed or conflicting
  identities before HTTP or creating/replacing an intent. Preserve all data.
- HRR-03: Ordinary retry reuses the actual durable nonce, text, current reply,
  composed time, attachments and uploaded dispatch resources through the
  existing append/settlement implementation. Recheck existence and source
  ownership under its staging lane. Keep the existing release-before-POST and
  credential-fenced settlement boundaries; no conversation lock spans the
  explicit retry's HTTP wait. Rapid same-target retries share a short-lived
  in-process retry lane, while persisted/server idempotency remains authority.
- HRR-04: Source-owned forwarding must never become ordinary resend, including
  explicit send of a marked row whose forward context was lost. Use the existing
  fail-closed forward-ownership classifier. When its original readable context
  and current account/issuer identify the original batch/target, retry only that
  target through `deliver_forward_batch`, preserving source authorization and
  pinned bytes. Missing, changed, unsupported or unresolved source ownership
  remains a visible retained failure; generic resend cannot repair it by dropping
  provenance. Existing source-batch ordinary-sync recovery remains authoritative.
- HRR-05: The Human Retry UI sends only the two durable identities, never calls
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
- HRR-07: The existing Human Session owner must bind a readable persisted
  conversation to its canonical participant-pair ID before it authorizes retry,
  send, transcript/history/search, reaction or resource materialization. Use
  the same order-independent ID algorithm as creation; check cached and cold
  owners without replacing metadata, user content or pending work. Cold
  admission retains the validated read-only opener. Recheck identity after
  service waits before settlement side effects. A private structural-only
  forward-journal reader may preserve the original request reservation and
  account/issuer-qualified denial after source authorization changes; it must
  not grant product read/send/resource authority. Valid participant reordering
  or title edits remain compatible. This reconciles the independently committed
  NMR-CONVERSATION-IDENTITY-01 design at `f3467c39e1258e2562ddbd7639c2617460a36f46`
  with this branch's newer cold-store protection; it does not import that
  branch's separate event-feed implementation or declare it accepted.

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
- HRR-AC-04: Real Session/SQLite and gateway HTTP fixtures refuse a changed
  participant pair under the old ID in cached, fresh and restarted owners,
  preserve stored bytes/rows, produce no unsafe network work, and recover after
  valid metadata restoration. Include reversed-pair/title positive controls,
  retained forward denial and a service response held across identity change.
  Actual-root UI negatives cover selection A-to-B-to-A, account replacement,
  late error/success, a same-nonce foreign-author response and an unstaged
  retry whose request waits for readiness. Original and newer composer drafts
  remain unchanged. Preserve all prior cases and exact-H Actions evidence.

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

### HRR-06 local retry queue continuation

An explicit local pre-admission retry may coexist with a newer composer draft or queued send. When that retry fails or its presentation selection expires, the existing ComposerSubmissionQueue must advance eligible queued successors within the same account lifetime without reconnecting, retrying the failed record, clearing the newer draft, or allowing a later submission to overtake it. Account retirement must not dispatch the retired queue. HRR-AC-04 includes actual-root failure and selection-expiry scenarios with a held Host reply, one successor dispatch and preserved original intent.

### HRR-07 retained forward authorization across service waits

For a source-owned forward, the original source identity and policy as well as the destination identity remain required after each resource or message response wait, before later network work or successful settlement. Reauthorize the original source context under its account/issuer without introducing a lock held across HTTP. If permission is lost, preserve the batch, target nonce and pinned bytes, record the existing durable denial, and permit the same request to recover after source restoration and restart. HRR-AC-04 includes source mutation during upload and message-response waits; these are distinct from destination mutation tests.

Local pre-admission retry receipts must satisfy the existing raw canonical Human settlement parser before display projection supplies defaults, and must correlate the captured author and original nonce. Missing or malformed canonical ID, remote identity, role or delivery cannot consume the failed row or newer draft. An eventual valid ordinary transcript event may still settle the already admitted Host work.

### HRR-08 partial attachment upload recovery

The existing `remoteDispatchAttachments` field is an ordered, confirmed prefix of
the immutable pending attachment list, including the already supported complete
list. After the native client qualifies each successful resource response and
the current destination/source authority is rechecked, persist that prefix on
the same pending row before the next upload. A later upload failure or process
restart must retain the original nonce, content, reply, attachment order and
confirmed resource IDs. Retry uploads only the remaining suffix and does not
reopen or reupload confirmed ordinary attachments. The existing forward owner
still validates its complete pinned snapshot before any new resource I/O.

Malformed, overlong or mismatched stored prefixes fail closed without new upload
or message I/O; they must not be treated as an empty list. All existing account,
issuer, permission, credential, settlement and release-before-message-POST
boundaries remain in force. Receipt persistence is durable recovery state, not
message settlement or permission to dispatch after revocation. History replay
and acknowledgement admission may settle the original intent only after all
original attachments have qualified receipts; an echo containing only the
confirmed prefix cannot discard the unsent suffix. A resource
accepted remotely without a qualified response remains an explicit ambiguity;
this change does not invent resource-service idempotency or an unseen receipt.

HRR-AC-05: Real Host/HTTP/SQLite cases must fail a later upload after one receipt
is persisted, restart the actual Session, remove or change the already uploaded
ordinary source path, then prove suffix-only upload, original nonce/reply/order,
one canonical message and no HTTP on a settled retry. Include malformed,
overlong and wrong-order prefix negatives with retained rows and no resource or
message dispatch, incomplete-prefix history settlement refusal, and the existing
forward snapshot/permission regressions.
No storage migration is required; old complete receipt arrays remain compatible.
