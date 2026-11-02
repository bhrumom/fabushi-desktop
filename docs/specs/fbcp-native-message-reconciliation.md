# FBCP native Human-message reconciliation

Status: active  
Spec ID: FBCP-NMR-001  
Updated: 2026-11-02  
Project: FBCP-001 / TDRP-001  
Task: 20bf9475-a885-4314-b509-bea53ef8849f, rounds 1–13

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



