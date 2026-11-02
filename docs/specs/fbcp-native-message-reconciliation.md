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
