# FBCP native Human-message reconciliation

Status: active  
Spec ID: FBCP-NMR-001  
Updated: 2026-11-02  
Project: FBCP-001 / TDRP-001  
Task: 20bf9475-a885-4314-b509-bea53ef8849f, rounds 1–3

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

Run `41400057867` at `f3b00a552dbc14536a26c7b19492319d3445fd1c` executed Human 10/10, Agent nonce 4/4, DNS/special-config 6/6, protocol 122/122, and eight of nine TCP cases; its real-service `live_session_exchanges_encrypted_ping_and_service_rpc` failed with `InvalidServiceMessage`. Proxy and credential tests did not execute. The fixed-U `mtproto_api.tl` type `future_salts` encodes `salts:vector<future_salt>` as a bare signed element count followed by bare 4/4/8-byte elements, not the boxed vector/element constructors previously assumed by the parser and its fixtures.

- NMR-17: Correct the existing `telegram-protocol` `parse_future_salts` owner to decode bare vector/count and 16-byte elements, keep bounded allocation, reject negative and oversized counts, truncation and trailing bytes, and preserve signed values. Use the existing native TCP session and RPC result route; do not introduce a second protocol owner.
- NMR-AC-11: Run both library parser fixtures and the entire protocol-contract test suite plus all nine TCP tests including the live service against the **same repaired exact H**. Preserve DNS six, proxy four, Native credential one, Human ten and Agent nonce four as mandatory actual-execution gates. A prior successful subset, skipped step, failed live RPC or passing post-failure diagnostic does not close the batch.
