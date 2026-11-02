# FBCP native Human-message reconciliation

Status: active  
Spec ID: FBCP-NMR-001  
Updated: 2026-11-03
Project: FBCP-001 / TDRP-001  
Task: 20bf9475-a885-4314-b509-bea53ef8849f, rounds 1–28

## Context and goal

At desktop `b93357e15d3a8b2b33e069b97cf2eea0024eabff`, the shipping Host Session owner sends and synchronizes Human messages through `FabushiNativeMessagingClient` and `/api/social/messages`. `ProductionSessionWorkers::materialize_remote_human_message` accepts a row whenever the local account is either sender or recipient, without requiring the other participant to match the selected Human conversation. It also reconciles by client nonce alone. A foreign conversation row or a peer-controlled request ID can therefore contaminate the selected transcript or acknowledge a local pending send. Fix this existing production boundary and its behavioral regression coverage.

This batch preserves the current Session/Transcript authority and Fabushi native service contract. It does not create another message owner, switch service protocols, retire legacy Telegram storage, or claim complete migration/packaged acceptance. Known history-overlap refresh and broader legacy-runtime retirement remain separate open work.

## Requirements and ownership

- NMR-01: Before an individual remote row causes resource download, transcript mutation or outbox settlement, require exactly the current local-account/peer pair, in either direction. Reject unrelated, self-only or conflicting participants. Permission denial/error must leave that row's prior local state unchanged.
- NMR-02: Only a local-authored remote message may match a local-authored pending/accepted entry through `clientRequestId`/`clientNonce`. A peer's request ID is not a local optimistic identity. Preserve legitimate incoming messages independently, including a peer request ID equal to a local pending ID.
- NMR-03: Before treating a remote row as acknowledgement of a persisted local send, require the same local author, peer, text, reply identity and dispatched resource identities available in the existing durable outbox contract. Missing or conflicting dispatch facts fail closed and keep the pending send recoverable. Identical replay remains idempotent and preserves stable transcript IDs/order.
- NMR-04: A previously persisted incoming row carrying a historical `clientNonce` must not deduplicate a new local send or be overwritten by its acknowledgement. Keep stored user content; normalize only affected projection metadata or scope matching by author. No destructive database migration or broad data deletion.
- NMR-05: Existing outgoing send/ambiguous-dispatch recovery, resource reuse, event publication and restart behavior remain on the same Session storage and service owner. Agent entries and their Coordinator/Host/Runner lifecycle are unaffected.
- NMR-06: Use the existing `session_gateway_contract` loopback service/production-worker harness for permission, collision, conflict, replay and restart cases. No string-only assertion may stand in for the behavioral oracle. Execute compilation and these tests only in GitHub Actions bound to the exact batch commit.
- NMR-07: The shipping AppHost composition must compile against its declared dependencies and the existing Host handoff types. Preserve optional self identity instead of fabricating zero, and keep live authorization checks at the existing provider rather than adding a stale cached authorization field. This compatibility repair does not authorize the legacy provider as the final native messaging owner; caller/data-safe retirement remains open.
- NMR-08: The FMEP-SOURCE-01 accepted-source checker must compare exact recursive identities and explicit source-to-responsibility relations. A responsibility may name additional exact source identities; no unrelated existing responsibility ID may close a product source. Historical unknown/unread counters cannot substitute for inventory membership, and a moving upstream branch cannot invalidate the explicitly frozen U. Source accounting alone cannot grant product/release ACCEPT.
- NMR-08a: Each nested repository must match the URL in its exact parent's `.gitmodules` as well as the parent/path/commit gitlink. A fork containing the same commit cannot silently replace original provenance. The accepted helper and affected schema agree on uniqueness of evidence, responsibility IDs and additional source identities.
- NMR-08b: The original-host cppgir acquisition must retain the exact `.gitmodules` declaration and the pinned original license bytes for cppgir and its declared expected-lite child, with Git blob identity, length, SHA-256 and durable Actions artifact paths. The declared expected-lite URL is the original `martinmoene/expected-lite` path; a matching commit fetched via a renamed repository must not silently replace that declared provenance. Verify provenance and saved source bytes on exact-H Actions; this is acquisition evidence only, not legal approval, semantic review, complete-source closure or product acceptance.

## State, interfaces and failure behavior

The flow remains production renderer command → Session production workers → native authenticated HTTP client → existing transcript/outbox persistence → gateway/renderer projection. Fabushi service IDs stay authoritative remote IDs; the persisted local transcript/outbox provides stable local entry identity and retry state. Nonces are account-author scoped deduplication keys, never authorization credentials. The authenticated local account and selected Human peer determine ingress eligibility. Resource identity comparison uses the persisted dispatched attachment/resource metadata rather than downloading an untrusted row first.

No service API or public message schema is intentionally changed. Incoming request provenance may be retained separately from `clientNonce` using existing extensible entry metadata if needed. No body, token or private credential is added to logs. Apply admission per row; retain existing retry/history orchestration and partial progress for independent valid rows. A rejected row returns the existing error path rather than pretending the synchronization succeeded.

NMR-09: Distinguish deterministic participant/acknowledgement rejection from resource or persistence failure during ordered replay. A known rejected row may be skipped while preserving independent valid progress. If durable-state access or materialization fails, stop before persisting later rows so a later known remote ID cannot hide the unresolved gap on the next sync. A temporary resource failure followed by restart must retry the missing row, preserve its resource bytes, and materialize the remaining page in the original order without duplicates.

Consider swapped/foreign participants, incoming/outgoing nonce collisions, same nonce with different text/reply/resources, resource-upload interruption, duplicate acknowledgement, retry after ambiguous dispatch, restart and historical incoming metadata. Preserve current cancellation and event-consumer boundaries; this batch must not claim to repair unmodified cancellation/overlap behavior.

## Verification and batch acceptance

| Criterion | Required evidence |
| --- | --- |
| NMR-AC-01 | Behavioral tests prove foreign conversation rows cause no resource access or transcript/outbox settlement. |
| NMR-AC-02 | A peer nonce collision produces an independent incoming entry and preserves the pending local send, before and after reload/restart where applicable. |
| NMR-AC-03 | Conflicting local acknowledgements fail closed; equivalent acknowledgement and replay preserve stable ID/order and recover pending state. |
| NMR-AC-04 | Existing ambiguous-dispatch recovery and relevant Human message integration contracts remain successful at exact H. |
| NMR-AC-05 | Production callers and state ownership are recorded in the existing owner matrix/RTM/ledger; source review, implementation, execution and independent acceptance stay distinct. |

Actions reuses the existing native-service contract workflow and Host test harness, with a focused batch job. Record selected commands, run/job/artifact and exact checkout. If inherited compilation, configuration or service gaps prevent execution, preserve failure evidence and repair only justified scope; never substitute a local run or mark the batch verified. Final service/package/UI/security/platform/Bot acceptance remains governed by FBCP-001 and TDRP-001.

### Round 2 recovery contract

Run `41381218656` at `c749a7b2e05db641266e966ccc27a23834be164b` ended with two distinct failures. AppHost job `125072247204` rejected its stale lock under `--locked`; Human job `125072247604` accepted the Host lock and compiled the library, then failed with E0599 because the new legacy-nonce fixture called nonexistent `react_human_message` instead of the existing `toggle_human_message_reaction`. No Human behavior test executed. The subsequent unconditional-on-failure `generate-lockfile` diagnostics are not acceptance and their unrelated registry upgrades must not be adopted.

- NMR-10: Repair only demonstrated missing dependency edges, preserving manifest constraints and every existing registry package version/checksum. The AppHost lock must include the current network/runtime manifest edges to already locked `rustls`, `serde_json`, `sha1 0.10.7`, `webpki-roots 0.26.11` and `serde`. Keep the already accepted `source/host/Cargo.lock` unchanged unless new causal evidence proves a necessary correction.
- NMR-10 diagnostic qualification: On later exact-H failures, resolve from the existing lock only when the affected Cargo log reports a lock-file error. Use Cargo metadata to retain existing version selections and preserve its diff as diagnostic-only evidence; compiler, test, count, service and packaging failures must not trigger `generate-lockfile`. All acceptance commands retain `--locked`; a diagnostic never reruns or changes a failed verdict.
- NMR-11: Correct the fixture to the existing production reaction entry without adding a second runtime method or weakening its content, nonce, persistence, restart and HTTP assertions. Classify this failure as `TEST_FLAW`; retain the failed run.
- NMR-AC-06: After non-force compare-and-swap publication and remote content/SHA readback, both `native-message-reconciliation` and `native-message-shipping-compile` must terminate successfully on the same exact H with `--locked`. The Human log must show all ten selected behavior tests actually ran with zero failures/ignored cases. A successful diagnostic/upload, skipped test or old-H result is insufficient. Dependent message batches remain gated until these conditions hold; loopback/compile success does not grant final service, UI, package or migration acceptance.

### Round 3 shipping recovery contract

Run `41392023824` is bound to `3ccbf37dbe8b648c93c7c8c5e35aab9438cb3e23`. Human reconciliation executed all ten cases successfully; shipping compilation failed with a missing direct `base64` dependency and an unmapped `NetworkError` at the proxy boundary. Protocol/TCP behavior was skipped and remains unproven at that H. Adjacent shipping callers must also obey NMR-07's optional-identity/live-authorization handoff.

- NMR-12: Declare the existing workspace `base64` dependency at its actual Native credential consumer, add only required edges to already pinned packages, and map proxy bootstrap errors through the existing `network_failed` response. Invalid proxy input must preserve the previous projection and must not disclose its secret.
- NMR-AC-07: At the repaired exact H, require NMR-AC-06 plus actual nonempty protocol/TCP behavior runs, all four proxy-boundary cases, and the Native credential schema case covering valid/invalid storage-key decoding. Every selected test must pass without ignored cases. Compatibility evidence does not qualify legacy Telegram as a final shipping owner or authorize its data deletion.

Run `41394020435` at `93b5977f99115bd41a9ff30441e0b22696797364` compiled the actual AppHost and executed 114 protocol cases, with 112 passed and two implementation failures. The complete failure artifact was recovered and digest-verified by run `41395252062`; TCP, proxy and credential tests were not reached. Preserve these failures and repair their implementation boundaries before advancing the message batch.

- NMR-13: Bound the complete encrypted packet, including its 24-byte outer header and encrypted inner header/body/padding, to the fixed-U receive ceiling of 16 MiB. Check before parser allocation and before decryption, including directly constructed public envelopes. Use checked length arithmetic and preserve existing authentication, session, parity and error contracts. Applying the same budget to encoding and quick-ack generation is a target defensive boundary; it is not a claim that fixed U has an identical send-side check.
- NMR-14: Before admitting special-config endpoints to the retry queue, accept only ports 1 through 65535 for both Plain and Secret forms, matching the existing target runtime boundary. Preserve the TL parser's signed value semantics and the source expiry/prefix/IP/secret policies. This port filter is target hardening, not an assertion that fixed U's selector already filters these ports.
- NMR-AC-08: Keep the two original regression assertions, add complete-packet overhead and authenticated parser-bypass negatives, round-trip the largest aligned allowed packet, and cover Plain/Secret port limits with ordering, exact secret retention, inclusive expiry and prefix rejection. Run the complete protocol suite and the TCP suite including its explicit live case at the repaired H; zero ignored tests remain required. Bounded failure diagnostics must not weaken the rejection oracle.

### TCP fixture recovery at the same message gate

Run `41395923359` at `d00fdb48200b105b7aea9f2cb865cf4e7b482b2d` passed all ten Human and four Agent nonce cases and compiled AppHost, but its protocol/TCP step was still nonterminal when inspected. Source review found two deterministic test defects: the unsolicited quick-ack peer waits for a client marker while its client calls receive before send, and the plaintext peer incorrectly uses the client-only transport adapter as a server. Two directory fixtures also close after the connection prefix before consuming the client's frame. These findings are `TEST_FLAW`; they do not establish any unobserved protocol/TCP pass or authorize a product transport change.

- NMR-15: Loopback peers must implement the server side of the wire contract: consume the single client initialization marker and complete request frame, then send a response frame without another client marker. The unsolicited quick-ack case must prove the preceding request did not request an acknowledgement and must still require the exact `UnexpectedQuickAck` error. Directory-ID cases must consume and verify the encrypted request before closing. Give all loopback socket reads/writes explicit bounded deadlines; timeout or peer panic fails the test.
- NMR-AC-09: Keep all original TCP cases and the live-session case, preserve payload, padding, message-ID, directory-ID, encryption-stream and refusal assertions, and execute the complete suite with zero failed or ignored cases in exact-H Actions. Existing NMR-AC-06/07/08 remain mandatory. A bounded timeout is non-passing and must preserve its failure log; it cannot be substituted for the required response or rejection.

Run `41397871884` at `b13e3d8d072ef5d58f466c55bb273ff502ca4463` and run `41398061600` at `fb4df3898ae31b04bc64adde1944067154ae3678` each passed AppHost compilation, 118 protocol cases and eight TCP loopback cases. Their live TCP case failed in the production plaintext decoder: a declared 80-byte body arrived with respectively 192 and 144 post-header bytes. This is `IMPL_BUG`; the extra received tail is not a truncated declared body.

- NMR-16: Match fixed-U `parseNotSecureResponse` by requiring a positive four-byte-aligned declaration that fits the received body region, and pass only that declared span to the TL handshake parser. Account for the entire actual post-header region, including extra tail, within the existing 16 MiB receive budget before copying. Preserve zero auth-key ID, message-ID, nonce, parser and state-machine checks. The outgoing 0–252-byte random padding is included in its declaration and must not become an invented incoming-tail cap.
- NMR-AC-10: At repaired exact H, execute all protocol cases including declared 80/actual 192, 1–3-byte excess tails, bounded larger tails, truncation/invalid headers, combined declared/received padding and actual-region overflow. A tail must not complete a truncated nonce or replace a wrong nonce; rejection leaves the handshake awaiting resPQ. Execute the full TCP suite including the live case and all NMR-AC-06/07/08/09 gates. Also execute all six response-normalization unit cases with the corrected Latin-1 oracle: U+00E9 survives as E9 and a supplementary-plane character maps its two UTF-16 units to two question marks. Authored assertions, skipped steps and successful diagnostic lock generation are not passing execution.

## Rollout, recovery and provenance

Deliver on the existing checkpoint migration line with non-force compare-and-swap and remote content/SHA verification. Existing persisted data and account credentials remain intact. Rollback is a normal revert of admission/projection changes; it does not delete messages/resources or discard failed-send retry evidence. A rollback reopens the security defect and requires explicit release assessment.

Normative references: root `AGENTS.md`; FBCP/TDRP `SOURCE_OF_TRUTH.md`; `docs/specs/fabushi-bot-communication-platform.md` §§6,7,8.3,12; `docs/specs/telegram-desktop-rust-equivalence-migration.md`; `docs/specs/fabushi-deferred-ci-migration.md` §§12–13. Fixed research U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; exact source objects and inspected ranges are recorded in batch evidence without implying whole-file or whole-upstream review. Source lock/inventory drift and review checker contracts are separately reconciled without promoting source completeness.

## Compliance record

Message-contract gates remain pending exact-H Actions terminal protocol/TCP and proxy/credential outcomes. NMR-08b source-byte retention requires its own exact-H Actions execution and separate legal review; it does not change the product acceptance state. Full FBCP/TDRP acceptance remains false. The task evidence records outcomes and remaining gaps after execution.


### Round 4 live TCP protocol repair (NMR-17)

Run `41400057867` at `f3b00a552dbc14536a26c7b19492319d3445fd1c` executed Human 10/10, Agent nonce 4/4, DNS/special-config 6/6, protocol 122/122, and eight of nine TCP cases; its real-service `live_session_exchanges_encrypted_ping_and_service_rpc` failed with `InvalidServiceMessage`. Proxy and credential tests did not execute. The fixed-U `mtproto/scheme/mtproto.tl` type `future_salts` encodes `salts:vector<future_salt>` as a bare signed element count followed by bare 4/4/8-byte elements, not the boxed vector/element constructors previously assumed by the parser and its fixtures.

- NMR-17: Correct the existing `telegram-protocol` `parse_future_salts` owner to decode bare vector/count and 16-byte elements, keep bounded allocation, reject negative and oversized counts, truncation and trailing bytes, and preserve signed values. Use the existing native TCP session and RPC result route; do not introduce a second protocol owner.
- NMR-AC-11: Run both library parser fixtures and the entire protocol-contract test suite plus all nine TCP tests including the live service against the **same repaired exact H**. Preserve DNS six, proxy four, Native credential one, Human ten and Agent nonce four as mandatory actual-execution gates. A prior successful subset, skipped step, failed live RPC or passing post-failure diagnostic does not close the batch.

### Direct-message reaction summary admission (NMR-18 through NMR-20)

Prerequisite: the original message-contract gate is executed at exact H `68d4f07688741e8f54b35f3746a087f7fdd12108`, run `41402071662`: both required message jobs completed successfully, with Human 10, Agent nonce 4, normalization 6, bare parser 2, protocol 122, TCP 9 including live service, proxy 4 and Native credential 1 actually passing. This permits the bounded dependent batch below; it does not grant full product or source acceptance.

The existing Human direct-message path currently expands each remote reaction count into transcript entries. A `u64` count is externally supplied, and the same summary reaches message ingress and reaction-toggle settlement. Keep the existing Session/Transcript owner, native HTTP client, durable SQLite row, transcript event publisher and renderer projection. This batch adds no schema, store, module, route or UI control.

- NMR-18: A Human direct-pair summary permits only `(count, reactedByMe)` values `(0,false)`, `(1,false)`, `(1,true)` and `(2,true)`. Reject `(0,true)`, `(2,false)` and every count above two, including `u64::MAX`. Zero/false is an explicit compatibility representation; the current backend normally omits an emoji with no rows.
- NMR-19: Reject duplicate emoji strings in one summary. Equality is exact string equality; do not normalize, merge, trim or replace Unicode, and do not introduce a cap on the number of distinct emoji. Apply this only to the existing native Human direct-pair contract, not Telegram groups/channels or other multi-party models.
- NMR-20: Validate message summaries after participant/sender admission but before the peer-author early return, any resource access, overlap selection or durable message mutation. The shared materializer must also reject malformed send acknowledgements while preserving its existing durable pending/error recovery. Validate reaction-toggle POST summaries immediately after the native response and before legacy nonce changes, durable update or events. Project each valid summary item into at most one local and one peer entry, independent of the numeric count magnitude. Preserve the received summary and use the existing write-then-publish route.

Invalid count errors use `remote Human reaction summary is inconsistent with direct participants`; duplicate errors use `remote Human reaction summary contains duplicate emoji`. A reaction-toggle rejection must preserve the entire prior row and publish no event for that target. The separate send POST still persists its legitimate outbox state before dispatch; this rule must not remove that state.

NMR-AC-12: Add three real Host/SQLite/loopback behaviors to the existing ten `shipping_human_` cases. Prove all allowed representations and exact Unicode preservation across reopen; reject the invalid matrix before resource access while allowing independent valid progress and traversing a full 200-row page past a malformed known ID; reject reaction-toggle responses without target-row/event changes, reopen with a pure read, and recover through a matching valid response and durable event. Use the existing bounded Human fixture and filtered mutation subscription, then unsubscribe. The exact-H selector must execute 13 Human cases plus the existing 4 Agent nonce cases. Retain locked AppHost, normalization 6, bare parser 2, protocol 122, TCP 9 including live, proxy 4 and credential 1 gates. Authorship or prior-H subsets do not qualify this new batch.

Provenance: `bhrumom/fabushi@edde714def34e6eb5ae133dba20790c80a355c85`, `fabushi/web/src/handlers/friends.js` blob `e3c6461a01181eb66f1dccbb3845fea530ec072e`: `handleSetDirectMessageReaction` authorizes the authenticated sender/recipient and writes only that user; GET and POST use `loadReactionMap`. Migration `fabushi/web/migrations/20261005_direct_message_remote_semantics.sql`, blob `048c6f922b726460ba7eea8c4ef5551b62b2a4ca`, uniquely keys message/user/emoji. The two-person bound follows the normal authorized write path and that key. The SQL table has no participant CHECK and the aggregator does not filter corrupted third-party rows, so malformed stored/service results still require client rejection. This source commit is inspected backend code, not proof of its live deployment.

Known-row refresh, complete history-range coverage, GET/POST freshness races, monotonic read receipts, packaged UI and full service acceptance remain separate work. Rollback is a code-only revert of these guards/projections and tests; there is no data deletion or schema migration, and invalid responses do not overwrite prior durable reaction state.

### Round 5 incremental prerequisite — known message state refresh (NMR-21a)

Following successful prerequisite run 41402071662 at exact H
68d4f07688741e8f54b35f3746a087f7fdd12108, the existing native
Session/Transcript owner must refresh known authorized remote Human message
rows, rather than using an overlap ID to skip state changes. An omitted
reaction field in a partial snapshot preserves the last durable reaction
state; an explicit empty reaction list clears it. Missing or older read
timestamps cannot regress a newer valid stored receipt. Preserve local
metadata, canonical identity and forward provenance during refresh.
The native HTTP client also exposes the server's positive safe-integer
afterId lookup and rejects missing success envelopes.

NMR-AC-13a: the real Host/SQLite/loopback fixture refreshes one known ID
with a partial reaction snapshot and then an explicit empty list. Reopen
proves one stable canonical ID and monotonic read receipt. The additional
shipping_human_known_refresh_preserves_omitted_reactions_and_clears_explicit_empty
test must actually execute on the new exact H with the previous 13 Human
tests, 4 Agent nonce cases and shipping compile/protocol/TCP gates.
This first increment does not close durable history-range coverage,
same-conversation reaction concurrency, POST restart recovery, saturated
time-cursor handling, full product acceptance or independent ACCEPT.

### Round 6 prerequisite recovery — canonical acknowledgement oracle

Run `41405385707` at `849f91588047e7ee856334994f485ccb2744bb7c`
ended with Human 13 passed / 1 failed / 0 ignored; Agent nonce did not run.
Shipping compilation, normalization 6, bare parser 2, protocol 122, TCP 9
including the live service, proxy 4 and Native credential 1 all passed.
The failure is `TEST_FLAW`: after correctly rejecting a missing/changed
nonce and proving unchanged durable state across restart, the recovery
assertion still expected the pre-refresh row byte-for-byte. NMR-21a
requires a valid matching acknowledgement to refresh the service fields.

NMR-AC-13b: retain all complete-row rejection/restart assertions for both
nonce negatives; explicitly expect the valid service timestamp, author
name and empty reaction projections on recovery, and reopen once more to
prove that complete refreshed row is durable. Do not change admission or
production refresh semantics to satisfy the stale oracle. Before dependent
history/reaction implementation, both required jobs must pass on the new
exact H with Human 14 and Agent nonce 4 actually executed, plus the existing
shipping/protocol/TCP gates. Preserve the failed run as evidence.

### Round 7 prerequisite recovery — history refresh fixture protocol

At exact H `63da06b025fdf57088b432602fa35e3761d62c9b`, run
`41409695104`, Human job `125159664070` executed 16 tests: 14 passed,
2 failed, 0 ignored. Both failures are `TEST_FLAW`: the valid-reaction
fixture and 401-message history fixture close before the production
post-coverage latest-snapshot GET. Their old request-prefix assertions
also omit the required `afterId=0` query. The full job log retains the
HTTP request failure; this is not evidence of a service outage or a
production admission defect. Agent nonce execution was skipped.

NMR-AC-14: script the real ascending-ID coverage requests and the final
latest-snapshot request explicitly, assert their different query shapes,
and preserve every Unicode, content, stable-ID, order and pure-reopen
assertion. Keep all 16 Human cases and require both message jobs to pass
at the repaired exact H before dependent history/reaction implementation.
Do not remove the refresh, ignore HTTP failures or adopt diagnostic
lockfile upgrades to make the fixture pass.


### Round 7 bounded history and reaction acknowledgement (NMR-22)

On the current Session owner, an afterId page is bounded to at most 200
messages. A response exceeding that ceiling must fail before transcript or
coverage writes. A malformed persisted coverage object must fail closed rather
than silently restarting at zero. Each synchronization invocation is capped at
32 HTTP history requests, including its terminal short/empty request; a
latest-snapshot refresh is allowed only if the call has spare budget. A
successful reaction POST is not a durable settlement until the backend
explicitly returns the reaction summary and the local reactedByMe state equals
the staged desired state. On missing or stale acknowledgements retain the
persisted intent and original entry for identical-state retry after restart.

NMR-AC-15: the Host/SQLite/loopback cases prove oversized history and corrupt
checkpoint rejection without partial writes or unauthorized network requests,
and stale/missing reaction replies preserve intent and exact POST body across
restart. The focused gate expands to 20 Human behaviors and preserves four
Agent nonce cases. A separate saturation/32-page boundary, UI page-cache and
late-ack competition batch remains required, as do service deployment and
independent full acceptance. Old-H success is not evidence for this H.

### Round 10 history, settlement and consumer continuity (NMR-23 through NMR-28)

The starting candidate is `d24783f4c000f765c8f65b58517f25753c5a20a3`,
run `41421314346`. Its run and job state must be read directly; a queued run
has no passing evidence. Preserve the complete message-job logs and actual
test counts. Repair a failed public contract before advancing dependent
consumers. The following behavior is part of the existing Session/Transcript
and Electron/renderer projections, without a new domain owner or data store.

- NMR-23: Bind an ascending history response to the requested `afterId`,
  `historyCursor` mode and page boundary. A missing, contradictory or
  non-progressing cursor cannot advance durable coverage. Exactly 6,400 rows
  in 32 full 200-row pages consumes the call budget without a 33rd request;
  after restart, resume from the persisted highest covered ID, including the
  final short/empty page. Duplicate rows do not change canonical identity or
  order. Page application must not reread and parse the entire persisted
  transcript for each row. A page-local snapshot may be reused only while
  the existing conversation mutation lock excludes competing staging and
  settlement writes; update that snapshot after each durable write. Retain
  per-row admission/resource checks and advance coverage only after the page
  has been applied successfully. The 6,400-row fixture retains its bounded
  inter-request deadline so repeated full-history work is not hidden by a
  relaxed timeout. A response contract test is not proof of Worker deployment.
- NMR-24: History GET application and reaction read/intent/POST/settlement
  for one Human conversation use the existing conversation mutation lock.
  Two overlapping toggles must derive their desired states serially from
  durable state; an intervening stale GET cannot undo the settled reaction.
  Failed or mismatched acknowledgements retain the existing durable intent
  for identical-state retry across restart. Other conversations remain
  independent and all existing authorization checks stay in force.
- NMR-25: A send POST may remain in flight while history discovers its
  accepted canonical message. Reconcile its eventual success or transport
  failure under the same existing conversation lock. A valid late reply
  must still pass participant, nonce and immutable dispatch-identity checks,
  but must not overwrite newer persisted receipts or reactions. A transport
  failure after a matching durable send settlement must return that settled
  entry without restoring failed/pending state. An unmatched send retains
  ordinary durable error/retry behavior. A malformed/conflicting success
  response remains an error, even if a row already exists; do not hide a
  semantic admission failure as a successful transport recovery.
  Bind dispatch to the Host's authenticated Human identity before any message
  or resource POST and recheck the request credential before consuming its
  response; a same-issuer account change must not send or settle another
  account's content, including through late-transport recovery.
  Compare receipt freshness as parsed RFC3339 instants, including offsets and
  fractional seconds; lexical string ordering cannot establish monotonicity.
  An invalid nonempty incoming receipt must fail admission before resource
  access or any durable row/coverage change. A valid receipt may repair a
  malformed legacy stored value; omitted receipts preserve stored state.
- NMR-26: `ProductionRenderer` keeps one account/conversation-scoped
  Transcript cache. Initial, latest-tail and older-page responses merge by
  canonical entry ID and canonical order, preserving already loaded older
  pages and live appends. Events received after a page request began take
  precedence over that response for the affected entry. Refreshes of older
  messages reach their cached page rather than requiring the message to be
  in the tail. Clear, conversation deletion, account change and superseded
  request generations fence every in-flight page result; they cannot
  repopulate cleared or foreign state. A failed load preserves valid cached
  content and remains retryable. Existing Agent entry and streaming behavior
  continues through the same cache and component roots.
- NMR-27: The shipping Electron carrier forwards the existing transcript
  page cursors and native Human commands/events unchanged through the
  canonical Host bridge. Its tests exercise actual carrier methods with
  controlled bridge responses: pagination, stable IDs, edits/deletes,
  reactions, read receipts, error propagation and subscription disposal.
  Where a contract is deliberately unsupported, retain the existing explicit
  error; do not invent fallback success or a second HTTP/store owner.
- NMR-28: Reuse `native-service-contracts.yml` for focused renderer and
  Electron behavior gates. Bind checkout, commands, logs and machine-readable
  results to the exact H; require nonzero executed tests and fail on missing,
  failed or skipped selected cases. Preserve the Human/Agent and shipping
  compile gates. All executable checks remain GitHub Actions only.

| Criterion | Required evidence |
| --- | --- |
| NMR-AC-16 | Actual Host/SQLite/loopback execution proves the 6,400/32-page restart boundary, cursor rejection, concurrent reaction ordering, delayed successful and failed send settlement, and pure-reopen persistence. Existing security/nonce negatives continue to execute. |
| NMR-AC-17 | Renderer behavior tests exercise initial/older/tail merges, event-versus-page races, old-entry refresh, duplicate IDs, clear and account/conversation fencing, failed-load recovery and existing Agent state preservation. Carrier behavior tests prove the unchanged command/page/event boundary and error/disposal behavior. |
| NMR-AC-18 | Both required native message jobs and the focused UI/Electron jobs terminate successfully at the frozen candidate, with full logs and actual nonzero passing case counts. Prior H, unexecuted authored cases and fixture-only evidence cannot close the new candidate. |
| NMR-AC-19 | Separately bind the compatible backend source commit to an observed production Worker version/deployment record and execute authenticated `afterId=0` ascending multi-page traversal against that deployment. Record request mode/cursor/order/uniqueness and authorized participant scope without exposing credentials or private message bodies. Missing deployment identity, compatible source, authorized account or real pages remains not accepted; mocks and a backend CI pass do not qualify. |

Rollback is a normal code revert on the same owners; no user database,
credentials or resource deletion is authorized. These gates qualify this
batch only. Packaged visual/a11y/platform evidence, source/legal closure,
complete Bot regression, upgrade/rollback, soak and independent ACCEPT remain
required for final FBCP/TDRP acceptance.

### Round 12 credential-rotation oracle recovery (NMR-29)

Run `41428797981` at `34b71fbd2b76a31380406c73c7b4a915f21a40e7`
executed 32 Human cases: 31 passed, one failed, none ignored. The failed
midflight-rotation case expected the word `credential` for every rejection;
the invalid issuer instead correctly failed the existing approved-origin
check with `Fabushi messaging API must use an approved Native issuer origin.`
This is `TEST_FLAW`, not permission to change that production refusal.
The response-body rotation/restart case passed. Agent nonce did not run.
Shipping, renderer and Electron jobs passed; retain that exact-H evidence.

- NMR-29: A reaction response must remain bound to its dispatched Native
  account, device, token, session, expiry and issuer through response-body
  consumption. A changed or invalid credential must leave the prior durable
  entry unchanged and preserve the staged desired-state intent. Reopening
  with the original valid account must replay that same desired state once.
- NMR-AC-20: Keep all six rotation variants, the streaming-body variant,
  complete durable-state assertions and restart replay. For invalid issuer,
  assert the exact approved-origin refusal; other rotations retain the
  credential-refusal oracle. The existing Human gate must require all 32
  cases with zero failures/ignored tests, followed by four Agent nonce cases
  on the repaired H. Its previous count of 30 is stale and must not reject
  the two newly authored behaviors after they pass. No assertion is removed,
  no failing case is excluded, and diagnostic dependency upgrades are not
  adopted. Full service/packaged/independent acceptance remains open.

### Round 13 delayed-settlement evidence and rejection provenance (NMR-30)

Run `41432042504` at `1f5bd31d54c678a619f88dd4529834129bf11ff7`
completed with 32/33 Human cases passing. The new delayed-lock case passed,
but the older midflight send case failed because the later credential check
replaced its transport's explicit `changed during Human request` rejection
with `changed before Human settlement`. The resulting fixture timeout was
after that assertion aborted recovery; it is not a production timeout.
Shipping, renderer and Electron jobs passed; Agent nonce was skipped.

- NMR-30: Preserve an explicit HTTP/admission rejection and its causal error
  when a send rejoins the Session lane. Successful replies and ambiguous
  transport failures must still acquire the existing conversation lock and
  compare every private credential field with the pre-dispatch fingerprint
  before reading or returning a durable settlement. Token-only changes count
  as a different credential. Do not expose the fingerprint, bearer or session
  identifier through renderer state, transcript metadata or logs.
- NMR-AC-21: Keep all 33 Human integration behaviors, including successful
  and disconnected delayed sends, pending intent, same-nonce retry and pure
  reopen. Add deterministic private-owner concurrency coverage: hold the
  real conversation mutex, provide an already completed response, rotate the
  credential before allowing settlement, and require the exact post-lock
  refusal for both success and transport ambiguity. Also prove explicit
  rejections retain their original error and unchanged credentials proceed.
  A scheduling delay alone is not evidence that an HTTP body was consumed.
  Execute these private-owner tests plus Human 33, Agent nonce 4 and the
  existing shipping/UI/Electron gates only in exact-H GitHub Actions.

The helper remains private to the existing Session owner; this introduces
no new public command, event, schema, credential owner or persistent field.
Rollback is a normal code revert without deleting user data. Historical
passed scopes remain recorded against their original SHA, and final
production-service, source/legal, package and independent acceptance stay
open.

### Round 14 Native startup isolation and reversible legacy retention (NMR-31–33)

At base `50b0a2048adce13f2281143e6c3f7449d56344d5`, Electron's
`FabushiAuthService.writeHostCredential` always obtains a Telegram storage
key and reads optional Telegram application credentials. On Unix with
`FABUSHI_PRODUCT_MODE=1`, the Production AppHost opens a persistent legacy
client when that key is present, restores its proxy state and hands its numeric
client ID to `HostCreateConfig`. Host then registers a Telegram conversation
provider. An unrelated corrupt legacy database or application secret can
therefore prevent ordinary Fabushi startup. The lease is fixed at construction
and is not rebound by the Native account owner's later logout/account switch.

The target for this bounded batch is the existing Electron Native credential
owner plus the shipping AppHost composition. Native Human messaging remains
Session/Transcript → `FabushiNativeMessagingClient` → Fabushi service;
Agent/FeatureHost and the existing account, resource, permission and search
owners remain in place. No new owner, protocol, persistent schema or renderer
flow is introduced.

- NMR-31: Ordinary Native credential creation, refresh and account switching
  must not read, create, rotate, delete or send Telegram storage/application
  secrets. Preserve the existing Native v1 descriptor's authentication fields,
  atomic replacement and private permissions. Existing secure-store keys stay
  untouched, including malformed legacy values; they are retained data, not
  inputs to Native authentication.
- NMR-32: Every existing AppHost constructor composes the existing Host with
  no Telegram lease/client-ID/self-ID. It must not open, create, migrate,
  decrypt or delete any legacy database, including when an older Native v1
  descriptor still contains valid optional Telegram fields. Existing legacy
  auth methods fail through the unavailable-client error and cannot lazily
  instantiate a provider. Native login, logout, issuer/account isolation,
  Agent roster and restart continuity remain functional. Product settings
  persist as before; no Telegram proxy publication is performed. This does
  not claim that native-service proxy support is implemented.
- NMR-33: Retention is a code-only, reversible composition transition, not a
  data-format conversion. Keep `feature-host/telegram/<account-hash>.sqlite3`,
  its existing auxiliary files and secure-store entries intact. Do not import
  unverified legacy messages/auth into Native identity or transcript state.
  Rollback is an ordinary reviewed revert restoring the old code and its
  unchanged paths/keys/formats; it can reintroduce the old startup/lifetime
  defect. Before actual data migration or deletion, require an account-bound
  versioned conversion, backup, interrupted-write recovery and explicit scope.
  Legacy crates, direct FFI client injection and default link dependencies are
  separately open retirement work and are not declared absent by this batch.

NMR-AC-22: In Actions, exercise the actual Electron account service against
synthetic Native login/refresh credentials and an instrumented secure store.
Prove the descriptor contains only Native auth fields, malformed/existing
legacy secrets do not block login and all legacy key bytes are unchanged;
prove ordinary logout/refresh behavior is preserved without exposing secrets.

NMR-AC-23: In Actions, execute Production AppHost construction with old-format
Native credentials carrying a Telegram key, an existing real legacy SQLite
fixture, a corrupt legacy file and a missing legacy directory. Prove Native
authentication and Agent create/list survive account/issuer changes and
process restart, the legacy auth entry refuses calls, no absent directory is
created and retained files remain byte-identical. A retained encrypted fixture
must remain readable through the original storage API after AppHost shutdown.
Require actual behavior rather than source-string matching. Preserve the
existing Native account handoff cases and the Human/Agent/UI/Electron gates.

NMR-AC-24: Compile the actual `source/host/app` shipping binary and execute
the selected new tests at one frozen exact H. Persist command/log/count/source
binding as Actions artifacts. Publish the code by non-force expected-SHA CAS
and read back its complete remote SHA, parent/tree and changed blobs. Final
source/legal, supported-platform package, real service, upgrade/rollback,
full Bot regression and independent ACCEPT remain unqualified. Backend
PR #137 may only be inspected until explicit merge/deployment authorization;
no mock or old Worker artifact substitutes for production `afterId=0` evidence.

### Round 15 desktop standalone-messaging boundary (NMR-34–35)

At base `c15dfbd9631f8b9fb6920cb5f80b9aea35210d07`, the actual Human
Composer and history requests use the existing Session/Transcript native HTTP
owner. AppHost nevertheless exposes `feature.messaging.execute`,
`feature.messaging.blob.read`, `feature.messaging.access.issue`, and the generic
`feature.execute` variant `messaging.execute`. Those routes reach a separate
FeatureHost MessagingService snapshot/search/blob/access authority. The generic
variant branches before ordinary feature authorization. This is callable
production composition, not evidence that the Human Composer uses it.

- NMR-34: In the existing FeatureHostController, reject all standalone messaging
  execution, blob reads and access issuance when `HostMode::Production` and
  `SurfacePlatform::Electron` are selected. Check before envelope parsing,
  authentication refresh, messaging locks, state-store/blob/access I/O or
  MessagingEvent publication. Both explicit AppHost methods and generic typed
  commands must reach the same refusal. Use the existing contract-error path:
  `standalone messaging is unavailable in the desktop host; use the Session/Transcript owner`.
  Do not return accepted/empty-success, synthesize Native IDs, silently import
  state, or send through a second HTTP owner. Keep the working Human/Agent
  Session/Transcript path, Agent control, Mini Apps, Settings and other feature
  commands unchanged. Explicit Test mode and non-Electron platform composition
  retain their current behavior; their separate authority/acceptance remains open.
- NMR-35: The refusal is a reversible code transition. Preserve account-scoped
  `runtime/accounts/<account-hash>/agents/_messaging/{snapshot.json,blobs}` and
  the existing unscoped `runtime/agents/_messaging/access.json`, including
  incomplete uploads, corrupt or unknown files and other accounts. An absent
  messaging directory must remain absent. Login/logout, account/issuer changes,
  invalid requests and process restart cannot reopen or overwrite those stores.
  The reserved `_messaging` directory is not an Agent automation owner. Existing
  automation loading, owner overrides, desired writes and stale-record cleanup
  must exclude it, so valid, corrupt and unknown nested `automation.json` files
  and their companion files remain untouched. Keep ordinary Agent automation
  create/update/delete behavior working. This fixes design-review R15-D1 at the
  existing owner boundary without changing general Agent-ID validation.
  Preserve an existing automation folder outside `_messaging` that explicitly
  declares that reserved owner. Skip both reserved-owner records and records
  targeting such a folder when merging the old account-wide `automations.json`;
  keep the original archive and reader intact. Desired-write collisions with
  these retained markers must fail before cleanup or writes, never overwrite
  the marker or pretend the conflicting record was persisted.
  `AutomationUpsert` must persist its candidate map before committing the
  active map or publishing an event; a collision cannot leave an executable
  in-memory task after returning an error. Ordinary legacy archive records
  must still migrate and appear through the real authenticated public list.
  Keep the original readers and formats available for a reviewed code revert;
  no data conversion, deletion or automatic fallback is authorized. Any future
  import needs versioned account/ID/permission mapping, backup and interrupted
  migration recovery before the old data can become canonical state.

NMR-AC-25: Exercise the real Production AppHost public dispatch for all three
explicit methods and both wrapped/unwrapped generic command shapes, with valid
and malformed envelopes and existing/absent/corrupt retained storage. Prove
the exact refusal for valid outer RPC/command fields; invalid outer fields may
retain AppHost's existing InvalidRequest response. Check no MessagingEvent
immediately after each refusal, before any account-boundary queue reset. Prove
no messaging-directory creation and
byte-identical retained files across account changes, logout and subprocess
restart. Native account projection and real Agent create/list are positive
controls. Add direct FeatureHost method/typed-command tests and original
snapshot/blob/access-reader rollback controls using existing dependencies.
Include nested valid/corrupt automation files under `_messaging`, reject that
reserved override owner, and exercise ordinary Agent automation persistence.
Also retain external reserved-owner markers/companion files and the real old
`feature-host/accounts/<account-hash>/automations.json` archive. Prove neither
reserved nor same-destination conflicting archive records become active, and
direct desired-write collisions fail without changing existing file bytes.
Require a public Upsert collision followed immediately by an unchanged active
list/no change event, and an ordinary legacy record as a positive import control
across login, logout, account/issuer changes and process restart.

NMR-AC-26: Reuse the existing Actions workflow and actual shipping AppHost
compile. Require nonzero executed new owner cases and the expanded existing
account-handoff target, preserving Human33, Agent4, private settlement3,
composition11, UI16, carrier10/auth8 and protocol/TCP gates. Bind all selected
results to exact H and retain any failed or queued predecessor evidence. The
current source-review schema/validator must still accept the same canonical
ledger; no source-review state or source-completeness claim is introduced.

The old `mahayana-ffi` force-link exports and explicit Host provider injection
are separate audited paths. Current Electron packaging builds `source/host/app`
and does not select the old FFI crate; unconditional Telegram dependencies in
AppHost/Host and non-desktop callers are not declared retired. No backend
mutation, merge or deployment is part of this batch. Full FBCP/TDRP source,
service, platform/package, Bot, migration and independent acceptance stay open.

### Round 15 recovery extension — direct Host and C ABI handoff (NMR-36)

At `98e51b31b5a296065190f665696b2ffe11a3efe1`, run `41443460672`
completed successfully. Human33, Agent nonce4, settlement credential3,
composition11, startup/retention6, FeatureHost boundary7, normalization6,
future-salts2, protocol122, TCP9, proxy4, Native credential1, UI16 and
Electron carrier10/auth8 actually ran. These results qualify that bounded
candidate only. The old `41438907013` Rust jobs remain cancelled/not-run.

The next proven entry is `mahayana-ffi::mahayana_runtime_create` decoding
arbitrary JSON into `HostCreateConfig`, then `MahayanaHost::create` registering
`TelegramConversationProvider` in `build_runtime`. Direct Rust callers have
the same injection path. Current Electron and mobile AppHost construction
already supply neither legacy field; the inspected CLI constructor also uses
the default empty handoff. External caller completeness and platform package
acceptance remain open, rather than inferred from these inspected consumers.

- NMR-36: The existing `MahayanaHost::create` must reject either non-null
  `telegramClientId` or `telegramSelfUserId` before product-origin resolution,
  credential/client construction, filesystem access, plugin discovery or
  runtime initialization. Zero, maximum and self-only values are legacy
  handoff attempts too; do not silently ignore them or synthesize a Native
  identity. Preserve both optional fields and camelCase JSON compatibility
  so old callers receive an explicit error:
  `legacy Telegram handoff is unavailable in the Fabushi host; use the native Session/Transcript owner`.
  Remove the old provider registration from this Host composition. The
  test-support constructor must enforce the same refusal before its injected
  engine is used. Configurations without either field retain their existing
  Agent, social, Mini App, model, approval and runtime behavior.
- NMR-36a: The C ABI keeps its symbols and existing creation failure contract
  (zero handle and thread-local last-error). A rejected legacy handoff must
  not allocate or invalidate a runtime handle, damage a previously created
  native host, or touch provided absent/existing data/session/surface paths.
  Retain original Telegram client/storage APIs, encrypted databases, sidecars,
  credentials, unknown files and readers for a reviewed code rollback. This
  change performs no data conversion, cleanup, replacement login or automatic
  fallback. Unconditional Cargo dependencies and force-linked legacy client
  exports are explicitly still open; refusal is not dependency retirement.

NMR-AC-27: Direct Host behavior tests cover the handoff value matrix, absent
roots, existing opaque/corrupt data byte preservation, compatibility decoding,
test-support refusal and native status/ready/Agent lifecycle positive controls.
C ABI tests exercise real exported functions for rejection, error ownership,
success after a prior error and survival of an existing native handle. Include
the existing unknown-handle/error and remote-Agent refusal controls. Run both
test targets and the actual desktop AppHost compile in the existing exact-H
Actions gate, retaining all NMR-AC-26 gates. The source schema/validator must
continue accepting the single candidate ledger without promoting source or
product acceptance. Record unchanged old data and a normal reviewed code
revert as the rollback boundary; real platform/package and deployment evidence
remain separate final gates.

### Round 16 AppHost production dependency and package boundary (NMR-38–39)

Prerequisite `adf15caadac5775fdd8582227cad3bbf9c93cf0f`, run
`41447812811`, finished successfully. Complete logs prove Human33, Agent4,
private credential3, composition11, Native6, FeatureHost7, Host6, C ABI7,
normalization6, future-salts2, protocol122, TCP9 including the live case,
proxy4, Native credential1, UI16 and Electron carrier10/auth8. The generic
Host direct-dependency negative also passed. These are bounded execution
results, not source completeness, backend production or release acceptance.

At that base AppHost's private Telegram lease is always absent, but unreachable
authentication/proxy forwards still require a normal `mahayana-telegram`
dependency. Electron stages four executables, while `extraResources` copies
all of `resources/bin`, including any stale compatibility shared library.
The separate CLI really consumes the generic `mahayana-ffi` ABI, so deleting
that crate or its exports is not part of this batch.

- NMR-38: Remove the impossible AppHost lease state and unreachable Telegram
  calls from production composition. Preserve the twelve recognized legacy
  auth methods as explicit refusals with their existing Operation error.
  Preserve `telegramRegistration`'s prior `firstName` validation precedence;
  unknown methods must retain InvalidRequest. Return the original request ID,
  no result and no new event. Keep Native auth/Agent behavior unchanged.
  ProductSettingsOwner still persists before returning the exact existing
  unavailable proxy-publication envelope; do not claim Native proxy support.
  Keep old proxy projection types and their compatibility tests only under
  `cfg(test)` with a dev dependency. Normal/build dependency paths from the
  actual desktop root must exclude the old Telegram adapter/core/runtime.
  Retain the original storage dev reader and all old data, keys and formats.
- NMR-39: The existing Electron package owner copies only its four declared
  host/coordinator/box-executor/network-worker executable names (including
  the corresponding `.exe` forms for Windows) from `resources/bin`. A stale
  FFI library, another executable or a nested file must not enter that package
  resource slot. Do not delete or rewrite the source staging directory or user
  data. Keep all four ordinary executables and their bytes/permissions. The
  generic CLI ABI and direct Telegram exports stay available to their explicit
  non-Electron consumers until separately qualified retirement.

NMR-AC-28: Expand the real Production AppHost account/issuer/restart fixture
with all twelve legacy routes and malformed parameter envelopes. Check exact
response IDs, no result, original errors, unknown-method refusal, unchanged
retained SQLite/sidecar bytes, and original encrypted-reader rollback control.
Exercise Settings persistence and the unavailable publication envelope before
and after restart. Keep the existing six process scenarios and all prerequisite
gates. The eleven pure composition/proxy cases remain compatibility tests and
are not evidence of an active legacy network carrier.

NMR-AC-29: In the existing Actions shipping job, inspect normal/build dependency
edges from `source/host/app/Cargo.toml`, build its actual executable with the
locked inputs, and save its hash, symbol inventory and dynamic-library list.
Reject direct legacy Telegram C entry symbols and old runtime shared-library
loads. The dev/test dependency graph is deliberately separate. A lockfile may
retain the same package edge for dev use; do not regenerate unrelated versions.

NMR-AC-30: In Actions, exercise electron-builder's actual locked resource-copy
implementation with the production FileSet and synthetic staging directories.
Prove the four ordinary files are copied byte-for-byte and excluded FFI,
unrelated and nested files remain absent at the destination and unchanged at
the source. Test Unix and Windows filenames. Retain the normal Electron compile
and carrier/auth cases. This is package-copy boundary evidence, not a signed
installer, all-platform runtime or production deployment claim.

Rollback uses an ordinary reviewed code/config revert. There is no data schema
change, conversion, cleanup, new owner or automatic legacy fallback. The old
reader may modify SQLite setup when opened, so rollback controls use synthetic
or explicitly authorized copies only; this audit never opens real user stores.
Full supported-platform packaged execution, external ABI consumers, Native
proxy support, complete source/legal closure, backend authenticated pagination,
Bot regression, upgrade/rollback, soak and independent ACCEPT remain open.

### Round 17 account matrix and actual AppHost resource-copy evidence

NMR-AC-28 remains required in full: extend the existing six process scenarios,
not a parallel account harness. Exercise each of the twelve legacy methods
with its valid historical parameters and malformed JSON parameter envelopes;
preserve registration's first-name validation priority, exact response IDs,
error types/messages, no result and no new events. Cover every existing proxy
Settings mutation, compatibility toggles and invalid input, persistence before
unavailable publication, failed persistence, and complete state after restart.
Original encrypted SQLite/sidecar bytes and the rollback reader remain controls.

- NMR-40: Reuse the locked production electron-builder FileSet and copy
  implementation with the actual AppHost executable built in the same Actions
  job. Use an isolated temporary staging directory containing that binary and
  explicit obsolete FFI, unrelated and nested-file controls. Prove that the
  copied package resource slot contains only that real executable, preserves
  its bytes and executable mode, and leaves every staging input unchanged.
  Never create substitute coordinator/box/network executables or label this
  single-resource experiment a complete application package.
- NMR-AC-31: The ordinary resource-copy test target retains its three fixture
  cases. A separate, explicit actual-binary invocation requires the binary,
  exact target SHA and evidence destination; missing or malformed inputs fail
  instead of silently skipping. Save a manifest of source/destination SHA256,
  size/mode and excluded controls, the copied real binary and the complete log
  in the existing shipping artifact. Bind the source hash to the AppHost build
  hash already produced by that job, then inspect the copied file's symbols
  and linked libraries against the same legacy-export/load negatives. Execute
  these checks only in Actions with locked dependencies. No signing, release,
  publication, source staging cleanup or user-data mutation is authorized by
  this bounded verification extension.

The retained generic C ABI has two observed in-tree executable consumers:
`mahayana-cli/src/main.rs` and `mahayana-cli/src/bin/mahayana-gateway.rs`.
Keep their create/execute/receive/interrupt/approval/close/string-free routes
and the existing direct-Telegram export retention until the complete caller
and data-retirement conditions are separately proved. External ABI consumers
and complete signed supported-platform packages remain unverified.

#### Settings removal regression: NMR-41 / NMR-AC-32

The account matrix source review found an inherited data-loss defect in
`ProductProxySettings::apply(Remove)`: decrementing preference indices before
removing the deleted index also discards a surviving adjacent endpoint's
preference. With list `[A,B,C]` and preferences `[0,1,2]`, removing A currently
returns `[1]` instead of `[0,1]`. The complete fixed-U
`core/core_settings_proxy.cpp` blob `18e7ab8ce60865409dbaff267ee192e272622d26`
has the same order in `SettingsProxy::removeFromList`; this is a deliberate
Fabushi correctness repair to that inherited behavior, not a claim that the
fixed-U implementation already preserves the intended identity invariant.

- NMR-41: In the existing Settings owner, discard only the removed endpoint's
  old preference index, then rebase indices after it. Preserve the identity
  and relative priority of every remaining preferred endpoint, selected proxy,
  mode, publication policy and transactional persistence. No schema/data
  conversion or new state owner is introduced. Already-lost preferences cannot
  be reconstructed automatically and must not be invented on reload.
- NMR-AC-32: Add one owner-level table case covering all complete three-endpoint permutations plus empty/singleton orders and
  each first/middle/last removal, plus repeated removal/no-op. The real AppHost
  dispatch matrix must compare the entire snapshot and persisted document
  through adjacent-removal cases and process restart. Retain all prior eleven
  composition cases; the exact-H expected composition count becomes twelve.
  Classify the finding as an implementation defect found by source review;
  do not claim a pre-fix execution failure. The existing earlier insertion/
  removal case missed this because its removed slot had no preferred neighbor.

### Round 19 direct Host approval observation (NMR-42 / NMR-AC-33)

At `5281b87d878c9223dc9cd1c100c0f56186c81173`, the two in-tree CLI
executables have moved from the retained generic C ABI to the existing
`MahayanaHost`. The earlier C ABI consumer description above is historical;
the C ABI remains retained for unverified external callers and rollback.
Do not delete exports, legacy readers or user data as part of this batch.

- NMR-42: CLI/gateway creation, execution, receive, interrupt, approval and
  close continue through the same canonical Host. Gateway approval choices
  are exactly `accept`, `acceptForSession`, `decline` and `cancel`. Unknown
  choices must fail before calling the runtime approval method. This is an
  observation/oracle repair, not a new approval owner or a change to those
  production choices.
- NMR-AC-33: The existing gateway mock must record `resolve_approval` calls
  independently from ordinary `execute` commands. Each valid choice must
  produce exactly one call with the expected request ID and choice; invalid
  choices must produce zero approval calls and zero ordinary commands.
  An empty ordinary-command queue alone cannot prove zero approval dispatch.
  Retain the existing two approval tests and execute them at the frozen H
  alongside the three direct CLI cases, five gateway executable cases and
  the CLI/gateway normal/build dependency negatives in the existing shipping
  job. Classify the source finding as `TEST_FLAW`; do not invent an earlier
  failing execution or alter production behavior to satisfy the assertion.

All checks run only in GitHub Actions. Preserve the preceding exact-H evidence
and record the corrected oracle's own execution separately. These bounded
checks do not qualify external ABI consumers, signed installation packages,
production backend pagination or independent full-product ACCEPT.

### Round 19 final macOS update bytes (NMR-43 / NMR-AC-34)

The existing macOS release owner runs electron-builder before separately
stapling the DMG. Its metadata gate only checked a ZIP name and the presence
of any blockmap. Read-only inspection of real run `41327420143`, package
artifact `13093546572`, found the DMG ZIP-entry size `174156394` while the
same artifact's `latest-mac.yml` declares `174154251`. The actual source
checkout in that historical job is `9d16483f70df9af06db97b17bb02a24dc507e459`,
distinct from the workflow run head. This proves stale size metadata in that
artifact; it does not prove any published release or installation succeeded.
The packaged acceptance step failed and final checksums were skipped.

- NMR-43: Extend the existing `desktop/scripts` packaging owner, using the
  locked electron-builder blockmap implementation and YAML dependency, to
  reconcile `latest-mac.yml` and both external blockmaps after DMG stapling.
  Bind the exact expected release version, canonical arm64 ZIP/DMG filenames,
  real regular files inside the release directory, SHA512 and byte sizes.
  Preserve the ZIP and DMG bytes. Regenerate each sidecar from those final
  bytes; refresh each file's hash/size and the legacy top-level ZIP hash/path.
  Do not reuse a nonempty stale sidecar. Preserve unrelated release fields.
- NMR-AC-34: Before any write, reject missing/duplicate/wrong-version entries,
  unsafe paths, symlink/non-file inputs and malformed metadata. A read-only
  verify mode must reject stale size/hash or a blockmap that does not describe
  the current bytes. Use the locked builder's actual generator in behavioral
  tests: append synthetic ticket bytes after initial metadata creation,
  demonstrate rejection, reconcile, and verify exact final size/hash and
  regenerated sidecar bytes while proving the package inputs unchanged.
  Cover an independently stale blockmap, missing assets, duplicate metadata,
  version mismatch and path traversal; failed preflight leaves metadata and
  sidecars untouched. Reconciliation is retryable after interruption; only
  successful final verification allows the existing release flow to proceed.

Wire reconciliation immediately after the existing notarize/staple step and
before packaged acceptance/checksums/upload. Save an exact-source JSON receipt
with final asset and sidecar digests alongside existing release evidence.
Run the focused behavioral tests in the current Native service Actions
workflow with `npm ci` and the existing lock. No dependency upgrades, new
updater, signing request, package publication or release dispatch is part of
this repair. Actual signed package generation, installation, update/rollback
and independent ACCEPT at the final shipping H remain separate open gates.

The same Actions batch also downloads immutable historical artifact
`13093546572`, verifies archive digest
`f17fc29681164e69507d24201c57a58c84651d66a65cc3d0ebf1faeeee0be225`, and
extracts only its five declared package/metadata members into a temporary
directory. Its original DMG must fail specifically with an asset size/hash
mismatch. Reconcile that temporary copy, verify it again, and compare original
ZIP/DMG SHA256 and sizes. Record the historical package source separately from
the current checker H; never label historical bytes a newly built candidate.
No installer execution, publication or signing occurs in this experiment.

#### Round 20 cross-platform blockmap verification recovery

Run `41465224038` at `151d3ffe6382219dab98fb317e59fe5faa8cea01`
passed all 15 metadata cases but the historical-artifact experiment stopped
at `MACOS_METADATA_BLOCKMAP_MISMATCH`, before reaching the stale DMG asset.
The digest-verified historical ZIP sidecar has gzip OS byte `0x13` (macOS),
while the verifier compared the complete compressed byte stream produced
on its current platform. The archive and current candidate share the same
locked app-builder-lib 26.15.3 dependency. Gzip envelope/compressor bytes
are not the block-map content contract; this is an implementation/oracle
gap in cross-platform verification, not permission to accept stale maps.

NMR-43 recovery: compare the bounded decompressed content of each supplied
sidecar with the locked generator's expected content. A different valid
gzip envelope or compression level may represent the same map. Invalid,
truncated, checksum-corrupt, oversized decoded or different map content
must still fail closed. Limit each decoded map to 64 MiB. Preserve the
read-only verify boundary and report hashes/sizes of the actual accepted
sidecar bytes, not those of a temporary recompression. Reconciliation
continues to replace sidecars with maps generated from final package bytes.

NMR-AC-34 recovery adds cross-platform gzip/receipt regressions, content
tampering and bounded decode negatives to the existing behavioral suite.
The stapling case must retain an equivalent differently encoded ZIP map
while the stale DMG still fails specifically with asset size/hash mismatch.
The same digest-pinned historical experiment must then repair and verify
both maps with unchanged ZIP/DMG hashes. Preserve the failed run/artifact;
do not loosen its required stale-DMG rejection or infer installation,
signature validity, update/rollback or release acceptance from this repair.

### Round 19 gateway lifecycle compilation recovery (NMR-44 / NMR-AC-35)

Run `41460570824` at `5281b87d878c9223dc9cd1c100c0f56186c81173` failed
shipping compilation with Rust E0004: `RuntimeProjection::project` omitted
the existing `RuntimeEvent::TurnStateChanged` variant. This is an interface
integration implementation defect; the diagnostic lock regeneration is not
the cause and its unrelated dependency upgrades must not be applied.

- NMR-44: Handle the lifecycle variant explicitly at the existing gateway
  projection boundary. The present v1 gateway event schema has no lifecycle
  event and the v2 wrapper reuses it. Preserve that schema and the canonical
  raw RuntimeEvent authority. Lifecycle-only events produce no fabricated
  text/reasoning/completion, no replay sequence increment, no new turn mapping
  and no premature cleanup. Existing prompt admission establishes operation
  ownership; actual message/operation events remain its gateway consumers.
  A queued terminal lifecycle event after interrupt must not resurrect a
  forgotten operation or replace the ownership of a subsequent run.
- NMR-AC-35: Add behavioral projection cases for all existing lifecycle
  states, stable session/run ownership and replay sequence, real completion
  and failure as the only gateway terminal sources, and late lifecycle after
  interrupt followed by a new run and approval. Keep the four canonical
  approval decisions and independent call observations from NMR-42.
  Execute the entire gateway library suite, including existing replay and
  v2 durability tests, together with direct CLI3/gateway executable5 and all
  previously skipped shipping dependency/protocol/TCP/package gates at the
  repaired H. No catch-all arm or relaxed compile/assertion gate is allowed.

This is a compatibility recovery for the existing gateway wire contract.
Exposing full native lifecycle state through a future versioned gateway
contract remains separate unfinished work; no event-schema completeness,
full Bot regression or independent ACCEPT is claimed here.

### Round 20 persistent gateway approval fixture recovery (NMR-45 / NMR-AC-36)

Run `41465224038` at `151d3ffe6382219dab98fb317e59fe5faa8cea01`
passed CLI3, then gateway executable cases finished 4 passed / 1 failed.
`repeated_approval_event_does_not_duplicate_peer_lock` failed on its first
issuance (expected one, observed zero), before the duplicate assertion.
Both approval fixtures construct the real PersistentServerRequestRegistry
with Default; this loads the same default gateway-state-v1.json file.
The earlier fixture leaves approval-1 pending, so the later fixture correctly
recovers and suppresses it. This is TEST_FLAW, not a demonstrated defect in
production duplicate-request handling. Full gateway library and later
protocol/TCP/resource-copy steps did not execute in that failed job.

- NMR-45: Give each gateway approval behavior test its own UUID-named
  temporary directory and an explicit persistent registry file within it.
  Retain the real persistence implementation, production Default path and
  duplicate handling. Do not change global home/environment settings or
  delete shared state. Cleanup removes only the fixture's own directory.
- NMR-AC-36: Preserve both original approval tests and the five-case
  executable gate. Prove initial issuance once, duplicate issuance zero
  with unchanged persisted state, and reopen from the same explicit file
  retaining the same server-request ID and suppressing a repeated event.
  An independent fixture with the same approval ID must issue normally.
  Run the direct CLI3, executable5 and full gateway library9 suites, the
  normal/build dependency negatives and all previously unexecuted shipping
  gates on the repaired exact H. No production approval semantics, public
  schema or lockfile dependency change is authorized by this fixture repair.

Keep the failing log and causal assertion. Tests and all executable checks
remain Actions-only. This recovery does not qualify real approval-service,
external FFI, complete Bot, signed-package or independent migration ACCEPT.

### Round 26 forward account and dialog lifetime (NMR-46 / NMR-AC-37)

At `a1196f00230f3307ce0bd9c704ef5c4a7c9a4044`, the canonical renderer
opens `ForwardMessageDialog` for a Session/Transcript message. Search has a
query generation, but submit settlement/error/finally has no lifetime fence;
the account reset does not close the forward target, and the post-forward
roster read can publish after an account change. A late A-account response
can therefore mutate or close B's dialog or restore A's conversation roster.
This is a source-observed implementation defect, not a claimed pre-fix run.

- NMR-46: Keep the existing renderer account generation as authority. Bind
  every forward target, search, submit and completion callback to that
  generation and the live dialog lifetime; bind the subsequent roster read to
  the same account generation even after a successful dialog closes. Account change
  or logout invalidates the target synchronously, including A→B→A reuse of
  identical conversation/entry IDs. Closing, replacing or unmounting a dialog
  invalidates its pending searches and submissions. Stale success, rejection
  and finally callbacks must cause no notice, selection, roster, error or
  busy-state mutation in the new scope. A query change must not invalidate a
  valid in-flight submit; rapid double submit must dispatch only once. Current
  partial failure keeps the same request identity and only failed selections
  retry. Same-account credential refresh must preserve the current lifetime.
- NMR-AC-37: Exercise the actual React dialog in Chromium with controlled
  delayed search/forward promises, including account switch/logout and
  A→B→A, close/reopen, source replacement, unmount, old success/failure/finally,
  partial failure/retry and current-success positive controls. Exercise the
  renderer's actual roster-settlement boundary with a delayed read across an
  account change and the root's synchronous layout-unmount generation fence
  before passive effects. Preserve Transcript16 and compile the actual renderer.
  Capture keyboard/focus and light/dark narrow/wide dialog evidence using the
  existing styles and components; publish logs and screenshots at exact H in
  the existing Actions UI job. No local executable validation or new test
  dependency is authorized. These are focused UI/contract checks, not complete
  packaged visual/accessibility, real service or independent ACCEPT evidence.

Session/Transcript remains the only message writer. Invalidating a renderer
lifetime suppresses stale projection; it does not cancel or undo an already
dispatched durable send. No database/key migration or data deletion occurs;
rollback is an ordinary reviewed code revert. Whole-batch forwarding intent
durability before the first destination dispatch, restart recovery of an
unsubmitted destination, and remote forward provenance are separate known
gaps. Backend #137 remains read-only, and all release/independent ACCEPT gates
remain blocked. Preserve U=`f985723b552b30fe1c17d377d57f44e61a15a8c4`.

### Round 27 durable forwarding batches (NMR-47 / NMR-AC-38)

Prerequisite recovery: exact `2a00708897e17c76194e0811a083ef6cd315d674`
run41500770749/UI125467351076 compiled the renderer and passed Transcript16
and settlement5, then all twelve Chromium cases timed out awaiting the fixture
mount, before reaching behavior assertions. Preserve that `TEST_FLAW` evidence;
it does not prove twelve product failures. Use a Vite-root module entry that
imports the external fixture through Vite's module resolver, retain complete
Vite/page/network diagnostics and bootstrap screenshots, and rerun all twelve
actual behavior cases. The query-change case must change the query during an
in-flight send; the positive control must exercise real keyboard focus/selection
and capture four actual theme/viewport screenshots. These strengthen the
declared oracle; they do not grant packaged visual/accessibility acceptance.

At `2a00708897e17c76194e0811a083ef6cd315d674`, Session persists an individual
destination only when its turn in the forwarding loop begins. Process exit
after the first dispatch loses every later destination and the renderer's
request identity. Reopening a destination outbox cannot reconstruct that
missing user intent. This batch closes that Session-owned durability gap.

- NMR-47a: Persist one versioned whole-batch record in the existing source
  conversation's SandAgentDb before any destination dispatch. Bind the record
  to the active Human account and Native issuer (never bearer/session secrets),
  request ID, source conversation/entry, normalized privacy, immutable submitted
  text/attachment/provenance snapshot, ordered deduplicated destination IDs and
  their existing deterministic nonces. No second database, queue owner or
  message writer is introduced. An absent field means no pre-existing batches;
  malformed/unknown-version state fails closed and is not overwritten.
- NMR-47b: Serialize batch staging/progress with a dedicated key in the existing
  Session lock registry. A short account/issuer staging lane checks existing
  durable source journals so the same actor cannot rebind a request to another
  source, including a source whose membership was later removed. Release that
  lane before network work. Do not hold a source conversation mutation guard while
  acquiring destination mutation guards. Every progress write must succeed
  durably; a failed write stops further dispatch. A crash after destination
  settlement but before batch progress is stored reuses the destination's
  canonical nonce/outbox settlement, never creates another message.
- NMR-47c: Identical retries and an original-destination subset retain the
  initial batch identity and snapshot. Reject nonce reuse for another source
  entry, privacy choice or expanded destination set before dispatch. Source
  edits do not rewrite the accepted payload. Before unfinished work, recheck
  source membership, source existence/forwardability/protection and destination
  membership through the existing owners. Removed/protected sources, malformed
  state, account/issuer changes and missing credentials cannot authorize sends.
  Preserve records and outstanding destinations for recovery; credential
  refresh for the same account/issuer remains recoverable.
- NMR-47d: The existing `syncHumanConversation` and startup/refresh
  `syncHumanConversations` path discovers and resumes persisted incomplete
  batches, including destinations that had no individual outbox entry before
  process exit. Use the existing append/HTTP/resource/event path and per-send
  credential settlement fencing. Already settled destinations stay idempotent;
  independent destination errors are durable, visible in the existing result
  envelope and retryable. Preserve original destinations even when the UI
  retries only a failed subset. Limit each batch to the existing 32 unique
  destinations. No backend API, public command or service schema changes.
- NMR-AC-38: At exact-H Actions, exercise the actual Session/SQLite/loopback
  and gateway path: durable complete intent before the first POST; process
  termination with an unsubmitted destination and restart through ordinary
  sync; ambiguous accepted first POST and idempotent replay; immutable source
  snapshot; changed source/privacy/expanded-destination nonce negatives;
  missing/protected source and account/issuer rejection; concurrent/repeated
  submit; failure of durable staging/progress before further dispatch; and
  preservation of existing message/Agent/credential contracts. Keep the actual
  renderer, Transcript16, settlement5, Chromium12 and shipping gates. Tests
  run only in GitHub Actions; local source reading/diff preparation is allowed.

Rollback retains the additional metadata and all transcript/resource rows;
older code ignores the batch metadata and must not delete it. Reapplying the
change can resume the same identities. An older binary cannot recover a
never-submitted destination, so rollback is not full batch-recovery acceptance.
Remote provenance fidelity, full source/legal closure, real production
service/pagination, supported signed packages, Bot regression and independent
ACCEPT/release gates remain open. Backend PR #137 is not modified or deployed.

### Round 28 forwarding crash and persistence qualification

At `65253740c033508ab08a4b500449a5b56727e940`, run41504939510
executes Human33 successfully. The following owner step selects only the one
`sharebox_shipping_tests` case while asserting three; the two journal-format
tests were never selected. Chromium executes twelve cases with eleven passing;
its positive control fails when the fixture calls the absent `setTheme` method.
Shipping compiles the actual AppHost and passes composition12, Native7,
FeatureHost7, Social9, HTTP-owner6, Host6, C ABI7 and CLI3. Its gateway executable
passes all five cases but the workflow asserts three, preventing gateway
library/protocol/TCP/package-copy execution. Correct that stale count to five;
these later checks remain unexecuted at the base, not passing evidence.
These are `TEST_FLAW` outcomes. Preserve all three failures, select the owner and
journal suites explicitly, and install the existing runtime theme owner in the
fixture so the four declared theme/viewport screenshots test actual palettes.

Source review also identifies two NMR-47 implementation defects. Different
batches can load the same source-journal array before `set_metadata` rereads the
metadata row, then overwrite another batch with a stale merged array. A pending
batch with a deleted/protected source currently aborts unrelated ordinary sync.

- NMR-47e: Journal read/merge/write must compare the exact observed key value
  and atomically compare-and-set the containing SQLite metadata row. An absent
  key differs from null or malformed data. A conflict or storage error stops
  later dispatch without overwriting another batch; recovery reloads durable
  state. Preserve existing unconditional metadata callers and notification
  semantics. Test the stale-observation interleaving with independent handles
  to the same real SQLite file, including no-change and unrelated-key controls.
- NMR-47f: Distinguish source policy denial from storage/decoding failure.
  Initial admission still rejects an ineligible source. For an existing batch,
  persist the denial on every unfinished target, preserve completed targets and
  original intent, and continue independent ordinary synchronization without
  sending denied content. Reauthorization after recovery may resume that same
  identity. Storage/corruption failures and failed denial persistence remain
  errors; they cannot be swallowed as policy refusal.
- NMR-AC-38 qualification: Add five `shipping_human_forward_` integration cases
  in the existing Session gateway target. Use parent-owned HTTP barriers and
  an actual child process killed and reaped by the parent, independent SQLite
  reads, nonce-deduplicating accepted-but-unacknowledged requests, ordinary sync
  restart, immutable payload and replay/conflict checks, account/issuer/source
  permission negatives, and SQLite trigger failures at staging and progress.
  No sleep-only or source-string oracle substitutes for these behaviors.
  Require all 38 Human cases, the existing sharebox owner case, the two existing
  journal cases and the new stale-write regression, Agent4, credential3,
  Gateway1, Chromium12 and the affected shipping/Electron/Windows gates in
  exact-H Actions. Authored tests remain `not-run` until execution evidence is
  recorded; no historical statement that the five tests already existed is
  accepted as evidence.

The data format, command/service schema, unique Session/Transcript owner,
Backend PR #137 read-only boundary and all final acceptance blockers remain.

### NMR-47g: source-owned pending forwarding cannot become generic replay

Generic destination replay must preserve ownership when a forward's context
version is unsupported, malformed or missing. Persist an independent
`forwardBatchOwned` marker before resource upload or message POST, including
adoption of older v1 pending rows. Any explicit ownership/version marker stays
under source-batch recovery. For markerless context with source/request fields,
read the referenced source journal directly; a matching request or destination
nonce remains owned even after membership removal. Missing or corrupt source
storage cannot prove legacy ownership. A forwarding-shaped nonce without
usable context must also be checked against the stored source journals; ordinary
reply metadata alone is not a forwarding identity. If no journal matches, require
a strict readable referenced source plus the actual pre-journal nonce contract
`SHA256(request UTF-8 || NUL || destination UTF-8)` before generic replay. Old
and v1 nonces share the same external format, so the prefix alone proves neither
ownership nor legacy compatibility. Missing/replaced-unrelated context plus an
unavailable source remains pending with an explicit unresolved-ownership error.
Preserve genuine readable pre-journal replay, existing pinned-byte guarantees
and NMR-49 reply repair. This compatibility claim binds to the actual 2a007088
legacy path; it does not promise automatic repair of arbitrary lost provenance.

A readable legacy source and correct old nonce are compatibility evidence only.
Before allowing generic replay, reuse `authorize_forward_source` to require
current source membership and an existing settled, unprotected, nonempty source
message. A policy or storage rejection returns an error before any upload or
message POST and preserves the complete original pending row; do not create a
replacement journal or replace its saved payload with newly edited source text.
After permission is restored, the same nonce and original snapshot may resume.

NMR-AC-38 ownership continuation adds one actual killed-process/SQLite/HTTP
case: durable marker before first POST; revoked-source pending rows with future
or missing context version, changed request marker, missing context and retained
nonce; unavailable source combined with missing/replaced-unrelated context;
corrupt source refusal; and a readable genuine legacy control using the real
pre-journal nonce algorithm, with real shutdown/reopen under revoked source
membership, protected-source refusal, and restored-permission delivery of the
original pending text after the source text was edited.
The complete original intent and pending payload must remain retained. Tests
execute only at exact-H GitHub Actions. NMR-50 retains nonblocking batch recovery;
this extension does not replace that work or change public commands/service APIs.

### Round 4 resumed batch: immutable forwarding attachment bytes (NMR-48)

Prerequisite run `41508537631` at `59f5fa24827812f78336c464995d161f403dedfb`
finished successfully: Human38, sharebox1, journal3, Agent4, credential3,
forward HTTP1, renderer16/settlement5/Chromium12, Electron and Windows suites,
actual AppHost/package copy, normalization6, future-salts2, protocol122,
TCP9 including live service, proxy4 and Native credential1. Concurrent source
guard commit `59308cf3cee48213278c52e4ad60bfd9250fb545` is retained.
The existing batch freezes attachment JSON but each destination rereads its
mutable path. Later destinations can therefore receive different bytes after
cache replacement, and source cache loss prevents recovery. This is a
source-observed implementation gap; no pre-fix test failure is claimed.

- NMR-48a: Before accepting a new batch with attachments, preserve an ordered
  byte snapshot in the existing source Session's durable conversation directory.
  Bind paths to the account/issuer/request and record a version, SHA-256 and
  exact nonzero byte size with each attachment. Limit the batch to the existing
  service's six attachments and each file to 32 MiB. Read through a bounded
  handle, require a regular file, flush and atomically commit each snapshot
  before the source journal references it. Retain names/privacy and the same
  Session/native-resource owner. Do not overwrite source/cache/user files or
  erase old journals; interrupted unreferenced snapshots are not accepted sends.
- NMR-48b: All destinations, pending-send replay and ordinary-sync restart
  consume the pinned attachment identity. Validate size and digest on the same
  bounded byte buffer submitted to the existing credential-bound upload; a
  second unchecked path read is forbidden. Verify all target attachments before
  its first resource POST and retain those exact buffers, bounded at six times
  32 MiB; later path changes cannot alter the bytes already admitted for upload.
  Source path replacement/deletion
  cannot change an accepted batch. Missing, corrupt, oversized or nonregular
  snapshots fail before resource/message POST and retain retry/error state.
  Existing valid uploaded resource IDs may still settle an ambiguous send.
- NMR-48c: Old attachment batches without byte evidence stay preserved and
  explicitly fail unfinished delivery rather than fabricate an old digest or
  silently read changed source content. Completed targets and unrelated inbound
  sync remain intact. Text-only version-one batches retain normal recovery.
  Restoring the exact pinned bytes may retry an incomplete new batch; a new
  user request is required when the original legacy bytes cannot be established.
- NMR-AC-39: Execute real Session/SQLite/loopback upload/send/reopen behaviors
  for original-cache mutation and deletion after staging, two destinations with
  identical uploaded payloads, missing/corrupt snapshot refusal and exact-byte
  repair, bounded/nonregular inputs, failed snapshot staging, and legacy
  unpinned recovery. Inspect durable intent before the first destination POST
  and preserve the existing actual process-kill/restart cases. Add focused
  owner tests for atomic retention, path confinement and bounded byte/digest
  validation; run all existing Human/Agent/shipping/UI/Electron/Windows gates
  at the new exact H in Actions. No local executable validation is permitted.

This freezes bytes admitted from the existing authorized source projection;
it does not establish remote forward provenance or prove that an already
corrupt historical cache matched its original server object. Backend PR #137,
real-service authorization/pagination, power-loss/filesystem qualification,
full source/legal, signed-platform packages and independent ACCEPT remain open.
Rollback retains snapshots and journal metadata; an older binary must not be
used to resume newly pinned attachments without the same integrity checks.

### Auxiliary packaged executable source locks (FMEP-SOURCE-01)

The four-file desktop resource boundary also depends on the independent
Coordinator and box-exec-daemon Cargo roots and the Go network-worker module.
At `59308cf3cee48213278c52e4ad60bfd9250fb545`, the two Rust roots have no
tracked Cargo.lock, the Go root has no tracked go.sum, and the release build
runs go mod tidy. The existing fixed Telegram U is unchanged; this batch binds
currently unresolved Fabushi package inputs without claiming source ACCEPT.

1. Capture missing dependency locks/checksums only in exact-H GitHub Actions,
   using the existing Rust 1.97.1 / Go 1.27.1 and the unchanged direct manifests.
   Preserve the original Go manifest and its fixed Tailscale v1.104.1 selection.
   A capture-mode result is review evidence, not a locked-input acceptance.
2. Review the captured inputs, commit them with expected-head CAS, and make the
   existing desktop/release build commands require Cargo --locked and Go
   -mod=readonly. Normal builds must not regenerate or tidy dependency inputs.
3. At the containing pinned H, build the three actual macOS auxiliary binaries,
   retain source/toolchain/normal-build dependency/module, file/hash/symbol/library
   evidence, and reject legacy Telegram or generic FFI runtime dependencies.
   Run the existing owner-focused lifecycle tests; missing inputs or changes to
   tracked locks/checksums must fail. Preserve prior full regression requirements.
4. Do not alter command/service schemas, process lifecycle owners, executable
   override policy, user data, legacy readers, signing or deployment. Registry/Go
   source/legal review, supported-platform signed packages and independent
   acceptance remain separate open gates.

### NMR-47e concurrent completion preservation

Concurrent commit `4f1070d73958a1275160c2590d31fbe6507008ee` adds strict
source-journal recovery scanning and two journal concurrency regressions. Keep
those tests and the scanner when integrating NMR-48. A stale batch snapshot
must not change any durably completed destination back to unfinished, even if
the caller reads a fresh metadata value immediately before CAS. Reject that
regression before writing; preserve complete intent and allow a freshly loaded
batch to advance remaining destinations. The existing two independent-handle
and concurrent SQLite tests are required with the original three and NMR-48's
seven: 12 journal tests, plus Human44 and all retained exact-H message gates.
Human44 retains concurrent `06fb24c4963f35d1450c122e7222ee39e21b4192`'s
actual process-restart corrupted-journal refusal and exact-repair test.
This does not claim cross-source request uniqueness across multiple AppHosts.
Journal load and store also require the containing metadata to be a JSON object,
including when the source owner is already cached. Invalid hex/JSON or a valid
JSON scalar/array must remain a storage error, never absence of a journal; retry
may proceed only after the exact original metadata is restored. Extend the
existing corrupted-journal test across cold and cached owners without removing
its process-kill, unchanged-bytes, zero-network and exact-repair assertions.

### Pending-parent reply identity and restart recovery (NMR-49 / NMR-AC-40)

Current source `59f5fa24827812f78336c464995d161f403dedfb` and its
completed/success run `41508537631` are the prerequisite. The existing UI
delivery decoder does not recognize Host `dispatching` and defaults it to
`sent`, making an in-flight local parent actionable for Reply. A child is
durably staged before dispatch discovers that parent has no remote ID. Parent
settlement currently rekeys only the parent; the child's persisted reply and
the sync pending snapshot still name the old local ID. This is a source-observed
recovery defect; no pre-fix runtime reproduction is claimed.

- NMR-49a: Treat Host `dispatching` as existing UI `pending`, preserving the
  current prohibition on pending/queued/failed message reply actions. Keep
  all existing sent, failed and Agent projections and component roots. Do not
  introduce a new public delivery state, route or message owner.
- NMR-49b: When the existing Session store settles a local Human parent to
  its canonical identity, atomically rekey that parent and rewrite exact old
  `replyToId` references on the same conversation's local-author unsent
  pending/dispatching children. Preserve their nonce, text, attachments,
  forwarding metadata, sequence and other fields. No remote-authored or already
  canonical child is rewritten. Use the existing SQLite owner and an explicit
  transaction; its write wrapper alone is not a transaction. Failed parent or
  child persistence rolls back the whole change and the atomic store method
  publishes no mutation. An outer send may still record its existing pending
  dispatch error; it must not publish canonical identity or child-reference
  changes from the rolled-back transaction.
  After commit, emit the existing deletion/upsert events for every changed row
  so the mounted projection receives the new references.
- NMR-49c: Pending replay must consume each row's current durable reply
  reference after earlier parent settlement, rather than the initial stale
  snapshot. Preserve current account/participant checks, per-conversation
  locking, ordered retries and forwarding-batch policy exclusions. For data
  already orphaned by older code, resolve `human-message:<nonce>` only to one
  canonical sent parent in this same conversation with the matching local
  author and exact nonce. Validate canonical ID and remote ID agreement.
  Normalize both requested and persisted reply identities before idempotency
  comparison and persist the canonical reference before dispatch. Missing,
  foreign, ambiguous or conflicting identity never drops the reply or borrows
  a peer's nonce; retain durable retry evidence and refuse that send.
  Retrying an unsent local row also requires its existing Human author kind and
  exact `human-message:<its own nonempty nonce>` ID. A canonical-looking row
  missing remote identity, or a non-Human row with the same author ID, is not a
  local outbox intent; preserve it without HTTP dispatch or child-edge repair.
  Directly selected canonical parents and historical local aliases must both
  use the existing message-specific canonical positive safe-integer parser for
  remote identity. Do not trim padded strings or accept leading-zero, signed,
  fractional, nonpositive or unsafe numeric aliases. Keep exact canonical string
  and integer equivalence, valid peer replies, and local pending/dispatching or
  remote-less local-only sent parents. Reject malformed remote targets before
  staging or HTTP, retaining complete durable rows through sync and reopen.
  Non-message negative fixtures must be valid persisted entry shapes so they
  actually reach the kind guard rather than disappear during decoding.
- NMR-AC-40: Add actual Host/SQLite/loopback tests for A→B→C staging,
  failure/restart and ordered canonical replay; historical dangling local
  parent references with same-nonce peer and foreign-conversation negatives;
  and a SQLite child-update trigger failure proving parent/children and the
  atomic owner's event stream remain unchanged, followed by recovery after
  reopen through the real send path. Preserve the existing outer pending-error
  event semantics separately from forbidden identity/reference changes. Check
  immutable content/resources/nonces and no duplicate sends. Add a behavior
  regression using the actual renderer delivery projection for dispatching
  and unchanged normal/Agent states. Preserve all existing Human44/Agent4 (including the concurrent source-corruption restart case),
  credential3, forwarding journal/HTTP and UI/Chromium gates. Bind new actual
  counts and complete logs to the new exact H in GitHub Actions only.

This repair changes no server contract, schema or owner and deletes no user
data. A normal code revert retains canonicalized references; it cannot repair
new dangling references created by older code. Full source/legal closure,
real service/Worker recovery, signed packages, complete Bot regression and
independent ACCEPT remain open. Fixed U remains
`f985723b552b30fe1c17d377d57f44e61a15a8c4`.

Delivery constraint: the direct 3.5MB ledger-object request was interrupted and
the expected remote blob is absent. A temporary checkpoint-only Actions job may
apply the exact reviewed NMR row delta to the expected base blob, prove every
other row and all source/acceptance fields unchanged, and create only the exact
unattached Git blob. It may not commit, update refs, push or modify production.
The main integrator must read back that blob, use ordinary expected-head CAS to
integrate the sole canonical ledger, and remove this temporary delivery job.
Existing schema/source and product gates remain required; object creation is
neither validation nor acceptance evidence.

## NMR-50 — Forward recovery must not stall history or cross account boundaries

Discovery base: `b25c2003a8e76779f2535fc3dfa90e4fe5bde149`; fixed U remains
`f985723b552b30fe1c17d377d57f44e61a15a8c4`. This batch extends the existing
`ProductionSessionWorkers` / native messaging / canonical SQLite owners. The
public gateway, journal schema, composition and account authority stay the same.

The current automatic recovery path waits for a batch mutex held across HTTP,
before ordinary history sync can run. A forward already accepted by the service
but awaiting its response therefore blocks the history settlement that could
confirm it. Roster sync also scans all batches again for every friend, multiplying
failed send attempts. Its friends request lacks the full credential fence already
required of message history, and can create old-account conversations from a
response completed after credential rotation. Root review decisions: FIX all
three implementation defects within the existing owner.

### Required behavior

- **NMR-50a:** Explicit forward callers retain serialized, durable request
  semantics. Automatic recovery must skip a batch already held by an active
  executor and continue ordinary history synchronization, including the active
  forward's destination. Acquire the batch lock once; do not release and reacquire
  it between admission and delivery. Lock poison, corrupt journals and storage
  errors remain failures. Skipping an active batch must neither alter its journal
  nor issue duplicate POSTs; later direct/roster sync can resume it after release
  or process restart. Preserve the NMR-25 history/send settlement contract and
  NMR-47/NMR-48 journal CAS, source authorization and attachment guarantees.
- **NMR-50b:** One top-level `syncHumanConversations` performs at most one recovery
  scan. Its internal per-friend history sync must not repeat that scan. Direct
  `syncHumanConversation` continues to recover once. A failed target remains
  durable and retryable on the next invocation; unrelated histories still advance.
- **NMR-50c:** Roster retrieval must bind to the active Host Human account before
  HTTP and verify the entire credential snapshot after consuming the response
  body, including user, device, token, session, expiry and issuer. A mismatch
  rejects the response before conversation creation, metadata updates, events,
  history requests or call sync. Retain the compatible native client API through
  a private bound implementation. Restoring valid credentials must permit normal
  synchronization and persist only the authorized roster.
- **NMR-50d:** A forward awaiting Coordinator readiness has not yet been admitted
  by the Host. Recheck its original account generation and dialog lifetime at
  the actual request-frame boundary, after readiness and immediately before
  pending registration/posting. Closing, unmounting, replacing the source or
  switching accounts before that boundary rejects the old request without a
  mutation frame. Add an optional local call precondition, compatible with
  existing callers; never serialize callbacks into the gateway payload or add
  a second account authority. Once posted, retain the existing durable Host
  intent and response settlement semantics; closing the presentation does not
  cancel, discard or reinterpret an accepted batch.

### Verification and completion

Use the existing real Host/loopback service/SQLite integration harness, executed
only in GitHub Actions at the frozen batch H. A response barrier must demonstrate
that both unrelated and destination history sync complete while a forward POST
response remains held, with one canonical destination message and no extra POST.
Always release the barrier before joining workers on a failing assertion. A
multi-friend roster fixture must count exactly one failed recovery POST per
invocation and show recovery on a later invocation. Wrong-user roster binding
must produce zero HTTP; delayed-body rotation cases must cover all six credential
fields and prove unchanged persisted conversations/metadata and no downstream
request/event, followed by successful retry and reopen.

Exercise NMR-50d through the actual Coordinator client and a controlled port,
including a disconnect/readiness wait, account and dialog retirement, a live
positive control and retirement after posting. Preserve the 16 existing actual
Chromium picker cases and all delivery/Agent projections. A component mock that
registers a forward immediately cannot prove the delayed frame boundary.

Retain all existing Human, forward-journal, Agent and credential regressions and
the actual AppHost compile gate; use the current Actions workflow without a
parallel test framework. Independent code/spec review and exact-H positive
execution evidence are required for this batch. Queued or unrun checks remain
pending. No final source, real-service, supported-platform, UI/accessibility,
soak, upgrade/rollback or shipping acceptance is implied. Rollback is an ordinary
code revert; no user data or journal migration/deletion is required.

### Resumed round 18 live-head correction — CLI/gateway production features (NMR-CLI-HOST-01 / NMR-AC-CLI-HOST-01)

The resumed read binds to `59f5fa24827812f78336c464995d161f403dedfb`,
preserving all intervening migration work. Run41508537631 passes the current
CLI3/gateway-executable5/library11 suites, but those commands build test
targets. The CLI manifest removed its normal FFI edge without retaining
the Host `desktop-full` and `local-only` features previously supplied by
FFI defaults. Workspace and FeatureHost edges disable Host defaults.

This is a production composition defect. Host's feature-gated native
construction installs NativeEngine/ResponsesModelRuntime and its routing/
Agent-control backend. Without the feature, the fallback still advertises
`mahayana-ai` and `miniapp`, but uses UnavailableAgentBackend. Merely checking
provider names, status, Ready, a test-support backend or a workspace-wide
feature-unified test build cannot establish working native Agent behavior.

- NMR-CLI-HOST-01: Explicitly enable the two existing Host features on the CLI's
  normal dependency. Preserve direct Host ownership, existing CLI/gateway
  configuration, approvals, lifecycle and storage. Do not restore FFI as a
  normal/dev dependency, change any external C ABI, introduce a new provider,
  alter public RPC/model policy or upgrade dependencies.
- NMR-AC-CLI-HOST-01: In the existing shipping job, separately record the CLI-root
  normal/build package and feature graph and build both actual executables
  with the locked ci profile and a single CLI package root. Require both
  Host features and absence of the legacy FFI/Telegram production graph.
  Save both real executables, SHA256/format/defined-symbol/linked-library
  inventories and an exact-source receipt in the same immutable artifact.
- Run each built executable through its existing entrypoint in a bounded
  child process using a synthetic MAHAYANA_HOME and workspace. A managed
  model configured with a plain-HTTP endpoint must be rejected by the native
  model HTTPS policy before any request; the unavailable fallback skips that
  construction and would incorrectly succeed. Also retain successful ordinary
  CLI status/gateway session-list and clean shutdown controls. This is an
  actual production-binary branch oracle, not external model acceptance.
- Strengthen each existing private RuntimeHandle status/Ready control with
  the real `mahayana.mcp.servers` command in its isolated synthetic root.
  NativeAgentBackend returns the empty local server list for a fresh root;
  UnavailableAgentBackend returns an error. This existing capability gives a
  local positive control for the actual native Agent-control backend without
  accounts, remote model calls or an injected EngineBackend. Provider names
  alone remain insufficient. Keep the current3/5/11 cases, durable state/reopen
  checks and all legacy refusal, retained-data and AppHost/Session gates.

All execution remains in GitHub Actions. The initial99 run and the completed59
run remain separate exact-input evidence. External FFI clients, authenticated
model services, full Bot/installed application acceptance and final ACCEPT
remain open; a successful compiler or binary-identity check alone closes none
of those broader gates.

### Resumed round 18 installer receipt placement (NMR-PACKAGE-DELIVERY-01 / NMR-AC-PACKAGE-DELIVERY-01)

The same59 source writes final-byte macOS metadata to RUNNER_TEMP, then adds
that receipt to the installer artifact beside desktop/release patterns.
upload-artifact uses the least common ancestor of input paths. This changes
the installer ZIP member paths, and the JSON receipt is itself outside the
existing promotion owner's exact allowed installer member set. The current
promotion consumer rejects either condition. No new59 signed installer has
been observed, so this finding is a deterministic producer/consumer source
mismatch, not a fabricated release failure.

- NMR-PACKAGE-DELIVERY-01: Keep the installer artifact's existing release-directory ZIP/DMG,
  blockmaps, latest-mac.yml and SHA256SUMS.txt paths. Put the final-byte receipt
  in the existing packaged-acceptance evidence artifact, where mixed evidence
  paths are already expected. Preserve the receipt, its exact source binding,
  all metadata verification and signing/installation gates.
- NMR-AC-PACKAGE-DELIVERY-01: Keep promotion's member whitelist unchanged. Reuse its existing
  six contract tests in the focused Electron job and review both producer
  upload path lists against the consumer. These tests do not execute the
  upload action or qualify a signed installer; final exact-source package
  generation, archive-member readback, installation and promotion acceptance
  remain mandatory before release.

The initial receipt-placement correction above performed no release dispatch,
publication, backend mutation, merge or deployment. The explicitly scoped
follow-on below permits only nonpublishing artifact-only verification.

### Round 18 package-only execution scope (NMR-PACKAGE-DELIVERY-01b / NMR-AC-PACKAGE-DELIVERY-01b)

The existing exact-source macOS packaging owner gains an explicit
`artifact_only` boolean, default false. Its default push/full behavior and
immutable promotion acceptance remain unchanged. This mode supplies actual
signed/notarized installer and installed-copy byte evidence while real account,
Agent and Mini App service acceptance is still unavailable; it grants no
product, release or independent ACCEPT.

- Admit artifact-only only for an explicit workflow_dispatch with
  publish_release, promote_existing_candidate and
  iterate_existing_miniapp_candidate all false. A single preflight parses
  only the canonical empty/true/false boolean input strings and rejects unknown
  values or conflicting artifact-only routes before any packaging or
  production-account work. Full-mode promotion plus publish remains valid
  for the existing promotion owner to qualify.
- All three existing route jobs depend on this preflight. Only its canonical
  output controls execution. Artifact-only skips exactly the production CI
  login, real packaged Agent acceptance and approved Mini App installed
  service/lifecycle acceptance, together with their full-acceptance uploads.
  Their original names and all existing full-mode acceptance commands remain.
- Preserve complete source builds, pinned dependencies, runtime boundaries,
  Developer ID signing, app/DMG notarization and ticket checks, installed
  executable copy identity, packaged worker/Host sibling checks, final-byte
  metadata, blockmaps and installer checksums. Do not launch the candidate
  application, send production login requests or run models in artifact-only.
- Use distinct fabushi-macos-artifact-only and artifact-verification evidence
  names, bind the requested scope and exact source/workflow/run identities,
  and explicitly record productAccepted=false and serviceAcceptance=not-run.
  Keep the installer member whitelist and flat release paths unchanged.
- Execute the preflight's behavioral and actual child-process IO tests and the
  existing promotion consumer tests only in GitHub Actions. The real consumer
  must reject dispatch-only candidate provenance, either skipped full
  acceptance step and the artifact-only archive name. A green artifact-only
  packaging job cannot satisfy immutable main/push/factory/full-step proof.
- After the containing-H policy contracts succeed, dispatch only this
  explicitly nonpublishing artifact-only route at the exact containing source
  SHA. Read full logs and actual archive bytes before claiming any package
  result. Missing signing/runner/build inputs remain explicit failures or
  blockers; no full-mode or promotion gate is relaxed to obtain a result.

This is a scoped CI/package-verification capability under the existing release
owner, not a new product runtime, authority or migration-completion row.
Backend PR137 stays read-only; merge, deployment and public publication remain
outside this batch.

### Native resource upload service limit (NMR-51 / NMR-AC-42)

The original product prerequisite remains exact59f5fa/run41508537631, whose five
applicable message jobs and actual suites succeeded. Resume at exact3c9995a
preserves all later forwarding, byte-snapshot, reply-recovery and source work.
Current `bhrumom/fabushi-backend@687718ce9e0083aa124d29005bcce4051d357919`,
`fabushi/web/src/handlers/friends.js` blob
`6179f3ce8e223f4e0bd9664885871e134f4488a8` permits only nonempty resources of
at most 25 * 1024 * 1024 bytes. Shared Agent videos may be 200 MiB. The current
Native path/buffer upload and forwarding snapshot limits incorrectly permit
32 MiB; the shared download32MiB budget has a different responsibility.

- NMR-51: Use one 25 MiB service upload bound for Native file and byte-buffer
  entrypoints and the existing forward snapshot reader/validator. This corrects
  NMR-48's upload32MiB and six-times32MiB wording to 25MiB and six-times25MiB.
  Preserve nonregular/symlink refusal, path and opened-handle checks, digest
  validation, atomic snapshots, complete-target prevalidation, credential
  fencing and all durable journal/error state. Bound file reads to limit+1.
  Reject empty/oversized inputs before multipart/HTTP, without changing the
  shared Agent video or downloaded-resource budgets, schema, stores or owners.
- NMR-AC-42: Execute a real Human gateway/SQLite recovery case: oversized
  video fails twice with zero HTTP, including after restart, retaining the
  original pending/error/nonce; corrected same input recovers once with the
  same nonce and exactly resourcePOST/messagePOST/resourceGET. Execute reader
  cases with the literal service25MiB boundary, empty/partial-read failures and
  actual limit+1 consumption. Prove the buffer entrypoint rejects before auth
  and HTTP; preserve the existing exact-limit and oversized forwarding oracles.
  All current Human/forward-journal/Agent/credential/shipping/protocol/TCP/UI/
  Electron/Windows gates must succeed at containing H in GitHub Actions.

This is a bounded service contract correction, not full UI retry acceptance.
Current UI retry creates a fresh nonce, omits reply and cannot recommit an
already committed attachment through the staging-only bridge; that dependent
consumer needs its own verified Host-authoritative recovery batch. Existing
UI failure notice/draft restoration was traced, not visually accepted.
Fixed-U source review is limited to `PrepareMediaList` and `Uploader::readDocPart`
sections; no complete upstream-file review, source/legal closure, deployed
service/package/platform/Bot or independent ACCEPT is claimed.

### NMR-48 qualified evidence publication

Append only the verified exact94/run41513767498 NMR-48 evidence to the existing
FBCP-NMR-001 row, preserving concurrent NMR-49/51 fields, all source reviews,
source counters and acceptance state. The existing temporary Actions transport
pattern may create only a Git blob from explicit input/output identities; it
must not create commits or move refs. Root integration reads back that blob,
uses force=false expected-head CAS, and removes the temporary job and input.
This publication does not rerun unchanged product suites or grant acceptance;
current Backend25MiB compatibility remains governed by NMR-51.

## NMR-PAGING-01 — Durable Human history walks and monotonic activity

Resumed round 6 discovery base:
`1b0116fe9609f54f6a0581d26791d5e98bbe43b9`; fixed U remains
`f985723b552b30fe1c17d377d57f44e61a15a8c4`. Preserve the later message,
credential, forwarding, source-review and shipping work. The earlier unpublished
round-6 working tree is not an integration base. This batch adds the remaining
stable-pagination and activity-recovery behavior to the existing Session,
SandAgentDb and unified renderer owners.

- **NMR-PAGING-01a:** Human full reads use finite canonical timestamp, numeric
  remote identity and physical sequence as a deterministic total order. A new
  tail walk captures only ordered physical sequence identities in the existing
  conversation SQLite KV. Its continuation uses an opaque versioned token,
  qualified by conversation and participant identity. Keep at most 16 durable
  walks; eviction, corrupt/missing/duplicate/non-integer indices, bad scope,
  malformed tokens and contradictory legacy anchors fail explicitly. A walk
  never silently resets to the first page. Original Agent sequence pagination
  and legacy callers remain compatible.
- **NMR-PAGING-01b:** Capture, index publication/eviction and the first page are
  one transaction. Every continuation validates the complete index and reads
  current canonical row payloads in that fixed membership order in one read
  transaction. Later insertion, local-to-remote identity settlement and an
  authoritative timestamp correction do not skip or repeat a member. A new
  walk observes the new canonical order. Completed walks remain retryable until
  bounded eviction. Reopen preserves both membership and exact continuation.
  Closed/unavailable storage reports an error instead of a fabricated empty page.
  No transcript payload or second message database is introduced.
- **NMR-PAGING-01c:** Add optional beforeCursor/nextBeforeCursor to the existing
  Human RPC path. Carry them through admission, projection, existing root cache,
  cached-conversation restore, pagination and request-generation fencing.
  Preserve existing ingestEntryUpdate/onPageApplied ownership and all current
  clear/account/ABA behavior. Committed row updates received during a failed
  older-page request remain an overlay until a later successful retry, so a
  stale fetched page cannot undo a reaction or other canonical update.
- **NMR-ACTIVITY-01:** Successful Human row materialization, canonical settlement
  and idempotent replay reconcile metadata from its current value with a finite
  timestamp using max. Failure to persist metadata is an error and retry repairs
  the row/metadata interruption. Agent-to-Human handoff joins the existing
  per-conversation message lane, retains nonce idempotency, and cannot regress
  activity behind newer message state. Preserve the current full-credential
  response/settlement fences, forwarding journal CAS, read-receipt instant
  comparison and every existing mutation entrypoint.
- **NMR-AC-PAGING-01:** Extend the current real Host/SQLite/loopback tests with
  equal-time history/backfill walks, restart/retry, index corruption and bounded
  eviction, a real native settlement moving the anchor timestamp, and blocked
  history plus Agent handoff with monotonic activity and metadata recovery.
  Add real pagination/RPC tests for opaque cursor propagation, failed retry
  overlays and unchanged Agent requests. Retain all current Human44,
  journal12, Agent4, credential3, renderer/projection/Chromium and shipping gates,
  including later concurrent additions. Run only in GitHub Actions at the
  containing exact H; inspect complete logs and actual counts.

Rollback is an ordinary code revert: extra scoped KV indices can remain inert.
Current Human production does not delete or clear physical transcript rows; any
future such entrypoint must invalidate the walks or establish non-reused row
identity before it is exposed. This batch does not claim a changefeed for older
remote edits/reactions, multi-device or cancellation completeness, real Worker
deployment, full source/legal closure, signed platforms, full UI/a11y/Bot/soak,
upgrade/rollback acceptance, or independent ACCEPT.

### Round 7 resumed qualification — existing NMR-22/24/26 boundaries

Resume base: `3c9995a97c59d89824b2f6e0e49e72cef4825439`; retain all later
NMR-25 through NMR-50 contracts and the fixed U. Earlier unpublished preparation
is not authority to restore blocking sends or replace current projection owners.

Before replaying a durable Human reaction intention, require a message whose
`authorKind` is Human and whose author is the authenticated local Human or the
selected peer. Require a canonical positive safe-integer remote ID and exact
untrimmed emoji identity. The selected row must also have exactly
`id = human-server-message:<canonical remoteMessageId>` and `delivery = sent`
before staging, before replay HTTP and after its response. A numeric remote ID
remains compatible when it denotes that same canonical ID. A local outbox ID,
different/padded canonical ID, or missing/non-sent delivery is inconsistent;
preserve its complete row and any existing intention without dispatching or
settling it. Existing canonical peer-nonce compatibility remains unchanged.
Extend the same admission, restart and held-response tables, retaining all old
cases and the numeric positive control; no new suite-count claim is implied.
Reject malformed/foreign/missing/changed targets before
HTTP, preserve the complete row and intention, and publish no mutation. Recheck
the current target and remote identity after HTTP before replacing reaction
fields; retain unrelated current metadata. New toggles use the same admission
before staging an intention. Existing credential/session response fences remain.

Qualify the actual short and empty 32nd pages without a 33rd list GET, partial-page
resource failure followed by checkpoint replay, equal-timestamp `beforeSeq`
stability, and crash recovery when the row was committed but the desired intent
was not yet cleared. Use the existing real Host/HTTP/SQLite/reopen harness.
Remote message identities must be canonical positive safe-integer strings or
equivalent integer JSON numbers before canonical-row lookup or materialization.
Do not normalize padded, signed or leading-zero text into a second identity.
An ascending history page must advance strictly from the requested cursor, even
when its cursor echo and numeric nextAfterId are otherwise valid. Reject malformed,
duplicate, backward and nonadvancing pages before resource access, row publication
or coverage advancement; retain the checkpoint and recover from it after restart.

The existing pagination anchor must remain pending through intermediate React
commits caused by row updates while an older page is in flight. Consume it only
after that page is installed. Human reaction RPC projection must reject obsolete
account generations, superseded mutations and authoritative transcript events,
including deferred React updaters; current failures use the existing notice.
Reuse `transcript-projection.ts`, the current read buffers/cursor cache, and one
Transcript. Preserve all existing tests and validate only in exact-H Actions.
No full source, backend deployment, package, visual/a11y or independent ACCEPT
is implied by these bounded cases.

### Request round 9 resumed qualification — durable coverage and full renderer

Current reviewed base is `afb34b49c5964524a887088f1095070b75cfdf8b`;
later independent commits remain part of the integration base. Fixed U and every
existing NMR contract remain unchanged.

- **NMR-23a:** Persist new `humanHistoryCoverage` values with `version: 1`.
  A structurally valid legacy value without a version is unqualified and starts
  again at `afterId=0`; retain its original metadata and transcript until a
  matching `historyCursor` page has been durably applied. Reject unknown,
  nonnumeric, null or malformed versions and unsafe/noninteger cursor values.
  Validate coverage after the current identity/credential check and before
  replaying that conversation's reaction or pending-send intentions. Preserve
  independent forwarding recovery. A denied old Worker response must leave
  legacy state intact across reopen; a qualified page upgrades the checkpoint,
  and the next reopen resumes from that qualified cursor. Retain the existing
  6,400-row/32-full-page and short/empty-32nd-page tests without duplicate cases.
- **NMR-26a:** Exercise the actual existing `ProductionRenderer` through its
  Coordinator client and feed. Preserve `HumanReactionSettlement`, opaque
  history cursors, failed-page overlays and viewport-anchor ownership. Canonical
  Human empty text or explicit empty attachments clear that field; an omitted
  field retains it. Composed updates and identity remaps must stay consistent
  in root, loaded older-page and in-flight caches. Nonces are qualified by Human
  author/domain; peer or authorless rows cannot remove local pending sends.
  Retain original Agent incremental and authorless echo behavior.
- **NMR-26b:** Delayed send and reaction results may only update their original
  account lifetime and conversation, and cannot overwrite a newer canonical
  feed result. An initial clear starts a fresh read while fencing the old one.
  An offline queued send drains exactly once only after the existing transport
  state becomes connected; repeated connected events do not redispatch it.
  Reset the existing queue and journal generation on account changes, cancelling
  obsolete renderer completions and releasing the old per-conversation queue lane;
  already-dispatched Host requests retain their durable settlement. A roster
  refresh failure after a canonical send receipt reports a scoped notice and
  cannot turn that send into a failure or restore its draft.
  Retain all current dialog, pagination, Electron and shipping gates. Add eleven
  real Chromium renderer scenarios, with no retries or skipped cases, complete
  JSON result assertions and inspectable screenshots/traces. Pointer-based
  reaction scenarios must hover the message action anchor and assert its
  toolbar is visibly rendered (opacity `1`) and can receive pointer events
  before performing the real click; keep the request-count and stale-receipt
  oracles intact. Controlled
  browser/service fixtures do not constitute deployed-service, signed-package,
  visual/a11y or independent acceptance.

All new and retained behavior is qualified only by GitHub Actions at the exact
containing commit. Preserve the observed afb reconciliation compile failure
(run41522324008/job125541192099, E0382) and repair only the incorrect shutdown
receiver in its restart test before executing the new candidate.

### Retained direct FFI wrapper qualification

The standalone legacy Telegram C wrappers remain a compatibility surface.
They do not regain an AppHost or CLI production registration. Extend the
existing `providers/telegram-runtime/tests/runtime_ffi.rs` target to call the
actual exported C create, persistent-create, execute, close and free functions.
Exercise owned response allocation/free, ordinary and persistent close/reopen,
and the existing null/UTF-8/key-length/closed-handle rejection contracts. Use
only correctly allocated caller buffers and synthetic temporary stores/keys;
never fabricate dangling pointers or open original user stores for a probe.

Run these integration cases only at the containing-H Actions. Preserve the
explicitly ignored live-DC case as unexecuted service coverage; require exact
pass/ignored counts for the non-live target. This qualifies the C wrappers and
rollback fixture, not a final staticlib/cdylib producer, export table or external
installed consumer. Preserve all original data, readers, public ABI signatures,
the generic Host C ABI7 gate and complete release/independent-ACCEPT blockers.

## NMR-UI-01 — Existing renderer author and asynchronous settlement

Integration authority is the live checkpoint, including
`afb34b49c5964524a887088f1095070b75cfdf8b` and later concurrent work.
The earlier unpublished round8 files are a candidate delta, never a replacement
for newer opaque history walks, pending reply recovery, current reaction
settlement, source reviews, Host gates or other product capabilities.
The original exact5d/run41413368942 and exact59f/run41508537631 message gates
succeeded; they do not execute this UI extension.

- **NMR-UI-01a:** Preserve explicit Human author identity from canonical Host
  rows through the existing renderer model, cache, page/event merge and
  acknowledgement/queue/cancel/retry paths. Distinct Human message IDs may
  correlate by nonce only for the same explicit Human author and user role.
  Peer, assistant and unscoped Human rows cannot settle a local intent.
  Bind optimistic Human rows to the authenticated native identity. Preserve
  the existing Agent fragment, attachment echo and tool-turn behavior.
- **NMR-UI-01b:** Reuse the current account/conversation generation and mutation
  fences. Keep per-operation read/receipt overlays where needed so a delayed
  successful send/reaction result cannot undo a newer canonical feed update.
  A late semantic error after canonical send remains an error without
  demoting sent delivery or restoring an unsent draft. Clear only the original
  submitted draft identity. Logout, A-to-B-to-A, transcript clear and root
  disposal cannot permit an old callback to change cache, delivery, draft,
  roster or a new lifetime's busy state. Switching conversations in one valid
  account settles the originating cache; its pending send must not disable
  another conversation or release that conversation's non-send busy operation.
- **NMR-UI-01c:** Apply canonical Human empty text, resources, rich text and
  reactions as authoritative values through initial/older/full snapshots and
  mutation overlays. Retain existing opaque cursor/current-payload behavior,
  failed-page retry overlays and commit-time viewport anchoring. Do not use
  Agent attachment-fragment fallback to restore removed Human content.
- **NMR-AC-UI-01:** Preserve every current projection/Host/cursor/RPC/Forward/
  Electron/shipping gate and extend only the missing behavior tests. Execute
  the actual ProductionRenderer with controlled external coordinator/Desktop
  boundaries in Chromium: source-author negatives, late success/refusal,
  canonical empty fields, overlapping reads, inactive cached rows, account
  replacement/return, clear/disposal, Bob composer/draft positive control
  while Alice waits, and retained Agent tools/attachment controls. Capture
  complete logs, request/error diagnostics, DOM and screenshots at frozen H.
  Authored counts are not execution evidence. No local executable validation.

Use the existing Session/Transcript durable owner and renderer projection.
No new message database, command schema, native edit/delete service, user-data
deletion, backend PR mutation or deployment is introduced. Full source/legal,
real-service/current-active deployment, signed packages, full visual/a11y/Bot/
performance/soak and independent ACCEPT remain open.

### Round8 ledger object transport

Large full-ledger connector writes were interrupted and the proposed object
was not available by its Git SHA. Reuse the repository's exact-checkout
unattached-blob transport pattern for this batch's single existing
`FBCP-NMR-001` row. A temporary Actions job may append only the explicit
reviewed arrays and note from a frozen manifest, while proving all old array
items, scalar/source/legal/acceptance fields, other rows and top-level source
review data are unchanged. It records the exact input, operations and output
blob identities. It cannot create commits or move refs, and grants no test,
source or product acceptance. Root independently reads the complete input and
output, reconciles the latest canonical ledger and publishes with
`force=false` plus `expected_sha` CAS. Remove only this batch's temporary
job/manifest after integration. This transport does not replace the sole ledger
or alter the product validation requirements above.

### Resumed round 24: concurrent Human creation safety (NMR-CREATE-01 / NMR-AC-CREATE-01)

Integration base `2cda05eb62f049b32e8913fe464d3ee955c681f5` preserves the later
forwarding, canonical history and reaction changes above. The real Coordinator/HTTP gateway permits overlapping
Session calls. Two first discoveries of the same participant pair can both
observe an absent canonical `store.db`; the slower creator may then find the
other creator's metadata, interpret `initialize_metadata_if_missing == false`
as ownership of a failed directory, and recursively remove that shared path.
The initializer also returns false for a busy write. Neither result grants
permission to delete the conversation. This is a source-proven defect, not a
claim of an observed production data-loss incident.

- NMR-CREATE-01a: Use the existing Session per-conversation mutation lock before the
  first existence/open/initialization decision, through valid metadata and
  owner publication. Same-pair creation and ordinary mutations share that
  lane; a different pair must still progress. Do not introduce another owner,
  database, global network lock or public command/schema.
- NMR-CREATE-01b: Reuse an existing valid Human conversation's canonical identity,
  participants, title and durable transcript/outbox/checkpoint state. A losing
  creation/open attempt must not replace or close an already published owner.
  Initialization refusal, busy or error must preserve the directory, database
  and unrelated files. No recursive deletion is permitted as initialization
  cleanup. Absent metadata in a truly empty schema-only store after interrupted
  initialization may be retried through this same owner; malformed or incompatible metadata fails closed
  without overwriting it or reporting a successful conversation.
- NMR-CREATE-01c: Reconcile with the current FBCP-HUMAN-CREATE-01/03 task:
  SQLite must atomically establish that metadata, other KV data, transcript
  rows and blob rows are absent before inserting Human metadata. A populated
  store with missing identity is an integrity refusal, never permission to
  adopt its messages. Use the existing SandAgentDb owner and leave general
  Agent metadata initialization consumers compatible. Distinct worker/SQLite
  owners must preserve the first committed identity and metadata; a per-instance
  mutex alone is not cross-handle authority.
- NMR-CREATE-01d: Creation's store opening and reads must not quarantine, reset
  or salvage an existing corrupt/NOTADB store into an apparently empty one.
  Preserve the original database and sidecar contents on refusal, do not cache
  a successful owner, and retain valid WAL-backed committed identity. Use the
  current SandAgentDb and recovery options without changing general Agent
  recovery consumers. This binds the current FBCP-HUMAN-CREATE-04 amendment.
  Keep automatic corruption recovery disabled on the existing Human opener.
  Before writable SQLite work, reject missing/empty main files accompanied by
  any nonempty retained `-wal`, `-shm` or `-journal` without changing files;
  their dedicated recovery remains open. A genuinely empty main without
  nonempty sidecars remains creatable. Reject a retained main shorter than the
  full 100-byte SQLite header before opening it.
  Perform format admission through SQLite's private-cache, immutable read-only
  connection and header-only `PRAGMA schema_version`, discarding its value
  and propagating every query error. Do not open/read/close a raw database
  file: that can release another connection's process-wide POSIX locks. Keep
  SQLite's default VFS and inode/FD lifecycle; do not add `nolock` or
  `unix-none`. This probe is not the current schema, WAL, authorization or
  migration-version authority. After format admission, a nonempty WAL/rollback
  journal remains with the existing WAL-aware owner; otherwise retain immutable
  `quick_check` before journal-mode/schema writes, including the readable-schema
  but corrupt-data-page DELETE-journal case. Keep all ten existing corruption
  variants, byte-preservation/repeat/reopen checks and zero-length positive.
  Add missing/empty-main-with-sidecar refusal, actual retained-WAL/reopen and
  external-process lock-contention controls. These controls qualify the default
  SQLite/VFS owner path, not arbitrary custom VFS or metadata/open races.
  Arbitrary hot/corrupt-sidecar recovery remains separately unqualified. The
  unpublished readonly_shm probe stays withdrawn: locked SQLite 3.51.3 has a
  known corrupt-WAL read-overrun in that path. No dependency upgrade or complete
  WAL/crash acceptance is claimed.
- NMR-AC-CREATE-01: Add focused real-SQLite behavioral cases in the existing Host
  owner tests, selected explicitly by the Actions reconciliation job. Force
  the same-pair scheduling interleaving with owned barriers/channels, exercise
  another pair while one creation is paused, and verify repeated/restarted
  creation retains metadata, transcript, pending state and the live owner.
  Exercise initialization refusal and storage failure at the initializer
  rather than accidentally at schema opening; preserve a sibling sentinel and
  prove retry after an empty initialization failure. Test incompatible stored
  metadata refusal. A bounded wait may diagnose deadlock but arbitrary sleeps
  or probabilistic thread collisions do not prove this regression. Record
  exact selected/executed counts, with zero ignored or skipped cases, at the
  containing H. Add distinct-worker/SQLite-handle first-writer controls and
  missing-metadata KV/transcript/blob refusal cases with unchanged durable rows.
  Retain current Human67/resource3, sharebox1, journal12, Agent4, credential3 and
  gateway1 contracts and all applicable shipping/UI/Windows evidence.
  The existing selected ambiguous-dispatch/restart contract must enter through
  the shipping `createHumanConversation` gateway command, then retain its actual
  Native HTTP, SQLite, pending/settled event and single-message recovery oracles.
  The current upstream reconciliation job already uses Ubuntu at the same H.
  Select creation14 and atomic-storage3 in that existing job, retaining Human67,
  the current selected gateway/retry contracts and all other checks. Require
  actual success for both retained-WAL and independent-process lock cases;
  the child lock checks must not inflate the outer test count.
  macOS shipping and full supported-platform acceptance remain independent gates.
  Preserve concurrent106bb95b's E0382 fixture repair: cleanup shuts down the
  restarted runtime after the original was dropped. That exact H passed
  Human62 and its retained reconciliation gates; it does not execute this
  batch's new creation12/storage3.

Tests and builds execute only in GitHub Actions. Test-only scheduling seams
must not become public product APIs or global mutable state. No data migration
or deletion is required. Rollback keeps all persisted rows and files, but an
older binary has the known creation race and is not a safe recovery proof.
This batch qualifies the shared Session mutation lane and the SQLite atomic
initialization boundary across distinct owners. Whole independent AppHost process
lifecycle and complete crash/release acceptance remain open conditions. Fixed U, full source/legal closure, current
production backend pagination, formal publisher, real signed package
installation/upgrade/rollback, Bot regression and independent ACCEPT remain
unchanged and blocked.

### Desktop resource-owner recovery qualification (NMR-HTTP-OWNER-01 / NMR-AC-HTTP-OWNER-01)

The containing base already rejects desktop generic HTTP `messages` and
`message-resources` at the existing ProductClient boundary. Keep that behavior
and its current error contract. Native Session upload/download remains the
authorized message-resource path; non-desktop ProductClient and MiniApp
package-resource paths retain their current owners.

- NMR-HTTP-OWNER-01: Qualify both actual desktop HTTP dispatch surfaces, including a
  forged request `sessionOwned:false`, UUID resource descendants, each admitted
  HTTP verb, and authenticated/unauthenticated requests. The composition, not
  the request body, chooses Session ownership. Reject before credential/network
  work; never publish rejected payloads as chat/history events. Real 307/308
  resource redirects must remain unfollowed, with contacts and non-desktop
  resource HTTP positive controls.
- NMR-AC-HTTP-OWNER-01: Extend the existing six HTTP-owner cases and seven Native AppHost
  cases without inflating top-level counts. In the real AppHost child/restart
  fixture, switch account `123 -> 456 -> 123 -> logout`, verify Agent isolation
  and restoration, and compare retained Social SQLite plus exact credential
  bytes after every refusal. Keep all current alias cases, URI normalization,
  existing message/Agent/MiniApp behavior and zero-network controls. These
  tests execute only in containing-H Actions and do not prove a real backend
  cross-account vulnerability, service readiness or final acceptance.

### Round10 Transcript pagination extension to NMR-26a / NMR-AC-17

This bounded extension retains the existing NMR-26a/26b renderer requirements
and their separate scenarios. It adds eight pagination cases in
`ProductionRenderer.transcript.behavior.test.mjs`; it does not replace the other
renderer, cursor, forwarding or reaction gates.

The existing ProductionRenderer cache and pagination controller remain the only
owners. A failed older-page request preserves its cursor and current live-row
overlays, stops automatic scrolling and find-driven repeats, and exposes a
generic accessible alert with an explicit keyboard-operable Retry. Retry uses
the existing single-flight request and the same opaque `beforeCursor` with its
`beforeSeq`; it must not disclose a raw transport error. After initial-load
failure and Retry, bind the controller and Find to the actual replacement DOM
viewport. A row commit while a prepend is pending must not consume the existing
`pendingAnchor.ready` anchor before its accepted page is applied.

| Case | Actual-root behavior |
| --- | --- |
| UI-NMR-OLDER-RETRY-001 | Two failures preserve live overlays and exact opaque continuation; reselecting the active conversation preserves the visible failure without another initial or older read; automatic retry pauses, keyboard/double-click Retry stays single-flight; cache A→B→A restores the next continuation. |
| UI-NMR-INITIAL-RETRY-VIEWPORT-001 | Initial-error Retry replaces the real viewport; both ordinary prepend and an intermediate older-row commit retain the visible anchor. |
| UI-NMR-REFRESH-PAGE-RACE-001 | Real reconnect drives roster/sync/full read; live edits/appends win over stale reads and a retired older-page result cannot revive its continuation. |
| UI-NMR-CLEAR-PAGE-001 | Clear invalidates pending reads and permits a fresh initial read; a late old page cannot repopulate the cleared scope. |
| UI-NMR-ACCOUNT-PAGE-001 | A new account using the same conversation ID owns a separate opaque walk; the previous account's late failure cannot poison the current request. |
| UI-NMR-CONVERSATION-PAGE-001 | Leaving a conversation discards its pending result; returning restores its committed cache and continuation. |
| UI-NMR-UNMOUNT-PAGE-001 | Actual root/client disposal closes ports; late failure cannot mutate UI or start another read. |
| UI-NMR-FIND-PAGE-FAILURE-001 | Actual Find of an unloaded hit stops after one failed continuation, including Next match; explicit Retry remains usable. |

The fixture mounts the actual production bootstrap in StrictMode, real
Coordinator port broker/client and native transcript event feed. Only external
desktop/service DTOs and deferred replies are controlled. Undeclared boundary
calls, browser errors and unsuccessful resource requests fail the case; no
copied cache model or permissive boundary proxy is allowed. Preserve complete
per-case request/port/error JSON and screenshots in the exact-H UI artifact.
Require exactly eight executed/passed, zero failed/cancelled/skipped/todo in
GitHub Actions, alongside all inherited gates. Preparation and source review
are not execution or full visual/accessibility/packaged acceptance.

Original d247/run41421314346 is retained as failed (20/22 Human passed); our
9d73/run41423909766 passed Human30/Agent4 and shipping but exposed a renderer
telemetry compilation defect subsequently fixed by a concurrent commit.
Exact106bb/run41523018920 separately passed Human62, Transcript29, cursor17,
settlement5, forward Chromium16 and Electron/Windows subsets; Shipping was
still queued at that observation. These results cannot qualify a later H.
The sole round10 evidence file keeps exact SHAs, counts, job logs and artifact
identities. Fixed U, source/acceptance fields and every full migration gate remain
unchanged.

### NMR-FFI-LIVE-01 — Retained Rust runtime production transport check

The non-live retained C-wrapper job above deliberately leaves the existing
`live_runtime_bootstrap_keeps_an_encrypted_transport_session` case ignored.
Run that unchanged case separately in GitHub Actions at its containing exact
H, using Rust 1.97.1 and the workspace's locked dependency graph. The scoped
workflow checks out `github.sha`, records the source/test/lock identities and
complete command/log, and executes only that named case with
`--ignored --exact --nocapture --test-threads=1`. Require its actual named
success and exactly 1 passed, 0 failed, 0 ignored and 15 filtered out; preserve
failure output and classify any failure before a focused repair or retry.

This invokes the existing Rust client owner: create → bootstrap production DC2
with a fresh MTProto auth key → encrypted ping → connected status → close.
It needs no account, API credentials, user data or device; runner network,
clock and randomness remain real inputs. It does not invoke the exported C
wrappers, restore a persisted transport, log in, send messages, qualify an
installed consumer, or restore the retired Telegram production registration.
Keep the ordinary 15-pass/1-ignored C compatibility gate, native shipping and
all current message gates intact. Store exact run/job/artifact identities;
retain all final service, package, migration and independent-ACCEPT gaps.
The one-purpose workflow may be removed after its immutable evidence is saved.

### Native platform response binding (NMR-NATIVE-RESPONSE-01 / AC-NATIVE-RESPONSE-01)

At exact `19df1199779fa7fd66547ba095c277e0306d4c93`, the existing
ProductClient native transport checks the active credential before dispatch,
but returns a bare HTTP response. Generic platform requests and get_json/
post_json publish a completed body without comparing that dispatch binding to
the current credential. Native plugin multipart publication has the same
completion gap. A response delayed across logout, account/issuer replacement
or credential rotation can therefore publish data or an error from an obsolete
session. The existing auth_status method already uses NativeCredential's
complete same_binding comparison; this batch reuses that authority.

- NMR-NATIVE-RESPONSE-01: Retain the exact NativeCredential used for authenticated dispatch
  until the bounded response body has finished. Immediately before returning
  success data or a decoded HTTP error, reload the existing credential owner
  and require the complete same_binding predicate. Logout, changed token,
  account, issuer, device/session binding or invalid credentials must reject
  the stale result with a fixed nonsecret error. Use the same completion check
  for platform requests, get_json, post_json and native multipart publication.
  Do not add a credential store, cache, refresh/retry loop, alternate identity
  owner or new public schema. An anonymous request must remain anonymous and
  must not acquire an unrelated active credential. Legacy non-native behavior,
  path/owner/origin checks, redirect refusal and body limits remain intact.
- AC-NATIVE-RESPONSE-01: Add GitHub-Actions-only behavior contracts using real loopback HTTP
  and existing private native credential files. Hold the body after headers,
  mutate or remove the credential, release the body and require rejection
  without exposing the obsolete response/token in the returned data or error.
  Cover account/issuer replacement, logout, token and session/device rotation,
  success and HTTP-error bodies; include unchanged-credential and anonymous
  positive controls, plus real multipart publication. Prove a single request
  and no replay of a potentially side-effecting POST. Preserve the six desktop
  HTTP-owner cases and existing native handoff cases, and run the affected
  crate/actual shipping compilation at the containing exact H.

This completion fence suppresses obsolete local results; it cannot undo a
server operation already dispatched. Existing Session/Transcript message
settlement and the new forwarding journal remain separate existing owners,
unchanged by this transport repair. Backend PR #137 remains read-only.
Official publisher, real signed platform packages, complete fixed-U source/
legal coverage, real-service acceptance and independent ACCEPT remain blocked.


## NMR-REG-TRM-01 — Restore existing Transcript recovery regression registration

Exact discovery H `63c59566aae7fa9b424a364df1429176c792d249`, job
`125535006727`, exposes two `unnameable_test_items` warnings in
`source/host/src/transcript_mirror/conversation_state_binary.rs`
(blob `d0cd9c66557514073bdc0e6ada670e52038df70b`). The existing owner is
`ProductionTranscriptCheckpoint::from_state_bytes` through the current binary
state decoder; no new state, storage, wire format or runtime owner is introduced.

- Register the existing compaction, token/timing recovery and stale-token removal
  tests independently at their current module level. Move only the misplaced
  closing brace; preserve every original assertion, fixture and production byte.
- Preserve the existing recovery oracles: token used/max `120/1000`, turn
  duration/timestamp `25/200`, and removal of field 5 without re-encoding or
  reordering the archive and turn field bytes. This is a `TEST_FLAW` repair,
  not a new product feature or proof of all Agent recovery.
- In the existing Ubuntu reconciliation job, run the real locked Host library
  target `conversation_state_binary::tests`; require `3 passed; 0 failed;
  0 ignored` and actual `... ok` records for both formerly unregistered tests.
  Add the exact source path to the workflow trigger. Retain all Human/resource,
  journal, Agent, credential, HTTP, UI and shipping gates and their current counts.
- Execute only in GitHub Actions at the containing H. Existing NMR51 selected
  execution remains bound to H63; this repair does not retroactively execute the
  missing tests there or unblock dependent Human UI before the required message
  jobs succeed. The one-brace repair is published at
  `be660661a4935a590cc6e5074ba3224074493401`; its actual CI selection is published
  at `db6d7f427ba25d7f90d6470a3ff69574bab5c034` (run `41526479236`, attempt 1).
  These publication identities are not execution results. Record registration
  and actual execution separately in the existing RTM/ledger and task evidence;
  do not promote source/product/ACCEPT.

Rollback is an ordinary test/workflow revert; no user data or production format
migration is involved. All full source, real-service, supported-package, UI/Bot,
upgrade/rollback and independent acceptance requirements remain open.


### Recovery round15: strict retained-source discovery (NMR-DATA-ADMISSION-01)

The existing Session gateway and `ProductionSessionWorkers` retain authority
for Native forwarding intent, source journals and destination transcript/outbox.
Current Human creation, explicit retry, reply/history/resource contracts and
generic Agent recovery remain with their existing owners.

- NMR-DATA-ADMISSION-01: Initial request-ID discovery and ordinary restart
  recovery enumerate all retained source journals before dispatch. Missing,
  unreadable or invalid source identity is reported; it cannot release an
  old request ID for another source or silently authorize a second send. Keep
  durable own-account revoked-source denial, other-account journal isolation,
  ordinary roster compatibility and existing metadata/SQLite owners.
- FBCP-HUMAN-CREATE-04 follow-up: Actual f544 Actions exposed deletion of
  retained WAL/SHM/journal when a 50-byte NOTADB main reached the sidecar bypass.
  Preserve the current nonempty-short-main refusal and SQLite immutable
  negative main-format probe. Preserve the peer's PRIVATE_CACHE setting and propagation of all
  schema_version errors; no corruption-error exception is introduced. The probe does not establish WAL integrity
  or current source identity. Before any SQLite open, a missing or zero-byte
  main with any known -wal/-shm/-journal entry refuses, including empty or
  symbolic-link entries. Zero-byte main with no sidecars and a genuinely
  missing store with no sidecars retain existing creation/recovery behavior.
  Keep all current creation14/storage3 cases, including real retained-WAL and
  cross-process lock controls, six-byte and truncated/full-header negatives,
  healthy schema-only recovery and the unrelated-sentinel positive.
  Zero/missing-main and singleton-sidecar variants extend the existing
  first-call/retry/restart byte-preservation case without increasing its count.
- NMR-AC-DATA-ADMISSION-01: Extend the real process/HTTP/SQLite recovery fixture
  with source policy loss, another account, invalid metadata, missing DB and
  plain no-sidecar NOTADB cases. Require retained bytes, no dispatch on failed
  enumeration and no replacement source journal. Preserve all current gates,
  including concurrent Human retry cases; Human68 is current67 plus this one
  strict recovery case. Exact containing-H Actions remain required.

The unpublished broader admission prototype is withdrawn: its readonly_shm
preflight is unsafe with pinned SQLite3.51.3 corrupt-WAL handling, and an
independent main-file open/close risks same-process POSIX locks. This batch adds
neither path, FFI, dependency upgrade nor alternate initializer. The existing
immutable probe uses SQLite's standard VFS lifecycle and is only a conservative
negative check; it does not accept arbitrary corrupt WAL, hot journals or crash
states. Multi-AppHost forward uniqueness, complete source/legal accounting,
live Worker/afterId=0, supported signed packages, Bot regression, upgrade/rollback
and independent ACCEPT remain open.

### Request round9: bounded automatic Human history continuation

At the inspected production chain `7d57aa4393c4f6c13bc6129255e2bec8173ed5ed`,
one Host sync persists at most 32 ascending pages. It returns transcript rows;
the Human roster has no coverage projection. Local transcript paging does not
perform service sync. A stable connected renderer therefore requires a later
focus, reconnect or explicit operation to resume an incomplete checkpoint.
The existing 6,400-row/restart case establishes later-call recovery, not
automatic completion without those triggers.

- **NMR-HISTORY-CONTINUATION-01:** Keep Session/Transcript's versioned
  `humanHistoryCoverage` as the only durable cursor authority. Expose an optional
  `historySync: { afterId: number, complete: boolean }` projection on existing
  authorized Human roster summaries. `afterId` is a nonnegative safe integer.
  Normalize an absent/null checkpoint or a structurally valid legacy checkpoint
  to `0,false` using the same parser as sync. Missing or invalid optional progress must not authorize an
  automatic request; a corrupt checkpoint still makes actual sync fail before
  that conversation's reaction/pending-send replay. Reading roster progress
  must not perform HTTP, mutate metadata or create a second owner.
- **NMR-HISTORY-CONTINUATION-02:** After a successful connected Human roster
  sync, consume incomplete progress through the existing
  `syncHumanConversation` and `listHumanConversations` commands. Each Host call
  retains its 32-page/200-row budget and existing array reply. Yield between
  bounded turns and give each eligible conversation a turn before repeating
  one. Continue only after observed cursor advance or terminal completion.
  A failed request, removed/malformed progress or nonadvancing cursor stops
  automatic work for that conversation until an existing later refresh trigger;
  other eligible conversations may continue. Do not turn a failed initial
  sync/fallback read into an automatic retry loop.
- **NMR-HISTORY-CONTINUATION-03:** Reuse the existing roster attempt, account,
  transport and renderer lifetime fences. Replaced attempts, disconnect,
  logout/account change, Host/transport replacement and unmount prevent another
  request or stale projection. The existing Native client binds each request
  and its complete response to its immutable origin and private credential;
  issuer changes remain that Host authority, not a new renderer identity field. An already dispatched Host operation may finish its durable
  work; suppressing a renderer continuation must not undo it. Project admitted
  updates through the existing transcript feed/cache and preserve optimistic
  sends, reactions, older pages and Agent behavior. A legacy Host with no
  optional progress keeps the existing explicit-refresh behavior.
- **NMR-AC-HISTORY-CONTINUATION-01:** In exact-H Actions, extend the real
  Host/SQLite/HTTP 6,400/32 case to read roster progress before and after budget
  exhaustion, reopen and terminal completion. Retain cursor-denial, damaged
  checkpoint, reaction, delayed receipt, forward and retry negatives. Exercise
  the actual ProductionRenderer with multiple continuation turns and two
  pending conversations without focus/reconnect/manual sync, plus
  failure/no-progress and account/transport invalidation. Keep strict nonzero
  counts, zero retries/skips and all existing UI/Electron/shipping gates.

This is a continuation of the existing initial history traversal. It does not
establish an old-message changefeed, outgoing read acknowledgement, arbitrary
multi-process cancellation, real authenticated Worker compatibility or final
source/platform/independent acceptance. Reverting the new optional projection
and renderer consumer leaves the existing durable checkpoint and explicit sync
recovery intact; no user data or migration reader is deleted.

## NMR-FORWARD-FEEDBACK-01 — Named feedback on the existing Human forward path

This bounded extension uses fixed U `f985723b552b30fe1c17d377d57f44e61a15a8c4`,
`chat_helpers/share_message_phrase_factory.h/.cpp` and the already reviewed
`share_box.cpp` completion consumer. Preserve NMR-46/47 account, intent,
permission, durable journal and failed-target retry contracts.

- The existing ForwardMessageDialog keeps recipient labels within its current
  account/source lifetime. A search may hide an already selected destination
  without discarding its label. The first submitted immutable intent freezes
  only selected destination labels; later rename/search results and consumer
  mutation cannot change labels used by that intent's retries.
- Only validated completed destinations reach the existing onSettled callback.
  Pass a new presentation Map for that attempt; never add names to service
  requests, persisted messages or a second owner. Reuse the existing
  ProductionRenderer -> settleForwardProjection -> ConversationComposer notice.
- One or two fully confirmed destinations with nonempty known labels use their
  names. More destinations or missing labels use truthful counts. Mixed,
  unconfirmed and empty results must not become an all-success message.
  Refreshing the roster may fail without undoing a confirmed forward result.
- Account changes clear existing notices. Old account/source/closed-dialog
  callbacks, including A-B-A, cannot publish names, close a replacement dialog
  or repopulate its roster. Ordinary code revert rolls back this presentation
  extension without touching the accepted Host intent or user data.
- Saved/self, premium-dependent suppression and session-window navigation remain
  explicitly unclosed: the current direct-pair owner supplies no canonical self
  destination. This extension cannot mark either complete phrase-factory source
  review, whole sharing parity, independent ACCEPT or release accepted.

NMR-AC-FORWARD-FEEDBACK-01 requires exact containing-H Actions: settlement9,
actual dialog Chromium17 and actual ProductionRenderer13, while preserving all
current compilation, cursor, retry, Transcript and Host gates. Keep all previous
cases and normal pointer actions. The new root cases are
UI-NMR-NAMED-FORWARD-001 (accepted notice, roster failure, account clear) and
UI-NMR-RETIRED-FORWARD-NAMES-001 (delayed reply after A-B-A); dialog coverage includes
hidden selections, rename/retry, callback Map mutation and lifetime replacement.
Authored cases are not execution evidence. Use the existing canonical
`TDRP-R9-SHARE-FORWARD-PRIVACY-001` ledger responsibility and the Round30 receipt.


## NMR-ELECTRON-01 — Actual Human IPC and process restart qualification

Extend the existing Electron e2e owner and production startup chain after the
renderer contract fixes. Keep the existing Plugins Close pointer cases and
native-message-electron Node/carrier/auth/update contracts. The browser-only
external-port fixture is not an Electron IPC substitute.

- Start the real desktop entry, production renderer/preload, MessageChannelMain,
  utility-process carrier, Rust Coordinator and AppHost/Session Native client.
  Control only a loopback auth/social HTTP service and test-owned user directory.
  Return the current bounded, explicit-zero historyCursor contract and canonical
  author/recipient/clientRequestId identities; unexpected requests fail visibly.
- A canonical Human send must remain unique and display its full persisted row
  after closing and reopening Electron with the same test user directory while
  remote history is unavailable. Assert actual IPC requests, user-visible state,
  persisted local recovery, and a new positive operation after service recovery.
- A remote-committed send whose response is withheld must recover after the old
  Electron lifetime ends and the same directory is reopened. Reuse the original
  nonce, settle one canonical row, preserve newer draft state and reject any late
  old-lifetime UI mutation. Record the actual shutdown mode; graceful reopen is
  not a process-kill claim. The fixture's idempotent server state is not another
  product message owner.
- Capture exact H, full Electron/Host output, HTTP request/response/release order,
  page errors and console, DOM and screenshots, trace/video and exact two-case
  results with zero retries/skips. No local executable validation. These are
  controlled Electron/HTTP/SQLite process tests, not deployed-service, complete
  visual/a11y, OS keychain, signed-package or independent product acceptance.

The actual production behavior, persistence owner and public schema stay in the
existing Session/Transcript path. If the tests expose a production defect, fix
that owner in a separately reviewed narrow delta before claiming a pass.


## NMR-DRAFT-01 — Persist original Human submission draft ownership

Keep the existing account-scoped composer-drafts V1 owner. Persist optional
per-conversation submission identity only: nonce, local Human author, original
draftId and original recoveryId. Session/Transcript remains the message,
canonical state, retry and outbox authority. Equal text or resources never
establish ownership.

Existing V1 draft/recovery content remains readable without a binding and cannot
be retired by matching canonical text. Malformed or duplicate new binding
metadata must preserve otherwise valid content. A recovery draft has a stable
identity; pending bindings remain durable when visible text is cleared or replaced.

The existing draft store adds identifyRecovery, bindHumanSubmission,
settleHumanSubmission and forgetHumanSubmissions. A repeated nonce retains its
first draft/recovery IDs. The awaited binding write completes before the existing
Human send/retry owner can post; failure reaches the recoverable send error with
zero message RPC. Failure and cancellation recover the complete original Composer
input, including richText, replyToId and isFork, rather than rebuilding it from
the Transcript projection.
Restored reply consumption requires an explicit current account/conversation
scope before either a selected target or draft fallback is used. Its visible
reply target and posted request agree; explicit Cancel reply clears the stored
recoverable target without discarding the other input. Agent fresh-nonce retry
uses its original queue input and confirmed per-file commit progress in the
existing scope; Human posted-frame retry keeps its original nonce. These local
progress checks do not qualify cross-process partial upload resume.

Canonical local-author settlement compares scoped account/conversation/author/
nonce and current IDs, retires only matching draft/recovery values and removes
the binding. Initial/history/feed/receipt observations are idempotent. Failure to
persist a settlement retains a safe retry binding without restoring old A over
new B. Conversation clear forgets bindings without claiming Host cancellation.

Restore merges edits by conversation and field while preserving untouched disk
records and forget tombstones. Account ABA and disposal fence asynchronous effects.
A disposed owner cannot clear or write a snapshot after its delayed read returns;
not-yet-entered writes are suppressed. Writes already admitted to the existing
main-process SandClientPersistenceStore follow its single read/write/remove chain,
so a replacement renderer's read is ordered after that write. Do not introduce
another storage owner or claim that dispose retracts an already-issued IPC.

An indexed persistence file read may return absent only for ENOENT. Other read
errors must reject through the existing persistence IPC so draft restore can
block writes to unknown state; EIO/EACCES must not become an empty account and
overwrite recoverable content. Directory enumeration and indexed-file stat
follow the same rule: do not cache an empty or partial index after an I/O refusal.
Delete the in-memory key only after its backing remove succeeds, so a failed
remove remains retryable. Preserve valid-file and genuine-missing behavior.

ProductionRenderer uses the common canonical observer, re-observes cached rows
after draft restore, and clears Composer only in the current lifetime when no
newer draft or recovery exists. Preserve current Human original-nonce retry,
partial attachment commit progress, Agent approval lifecycle and all existing
projection, pagination and forwarding owners.

Actions must execute the actual-store cases for failed/held binding writes,
complete restart, malformed metadata, field merge, duplicate settlement, account
ABA/disposal and same-text replacement IDs; exercise the real persistence owner
for read failures and admitted-operation ordering. Actual Chromium exercises the
production root and bridge; the Electron held-receipt case must assert original
A retirement before any new edit, create a new B identity through real
A-to-other-to-A editing, and retain B across the next real process reopen.
Authored/static evidence is not execution, deployed-service or independent ACCEPT.
