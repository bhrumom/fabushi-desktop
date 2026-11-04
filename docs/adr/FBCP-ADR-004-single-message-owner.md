# FBCP-ADR-004: Single message state owner; shared pure messaging domain policy

Status: accepted (decision delegated by the user on 2026-11-04 under FMEP-001 r3 §14 FMEP-STATE-02)
Date: 2026-11-04
Supersedes: none. Refines FMEP-OWNER-02 for Message/Conversation/Permission.

## Context

At base `5682bec433fc5f42051db978ed627754be244d3f` two writers own the same Human message semantics:

| Path | Entry | Persistence | Reach |
| --- | --- | --- | --- |
| Desktop Session/Transcript | `ProductionRenderer` → Session gateway → `ProductionSessionWorkers` (`source/host/src/extensions/session/production.rs`) | SandAgentDb SQLite, forward/reply journals, Fabushi service (`/api/social/messages`) | Desktop production, Actions-qualified, sole desktop owner |
| FeatureHost standalone messaging | `execute_messaging_sync` → `MessagingService::handle` → `MessagingEngine` (`native/mahayana-messaging`) | whole-file `<account>/_messaging/snapshot.json` + blobs | Rejected on Electron production by `ensure_desktop_messaging_owner`; reachable on iOS/Android and tests |

The desktop path already consumes pure helpers from `fabushi_messaging_core` (`recipient_search_authorized`, `RecipientAuthorizationInput`, `ForwardPrivacy`, `ConversationPermissions`, `MemberRestrictions`, `query_matches_fields`, `visible_forward_origin_author`). From 05:54–08:02 on 2026-11-04 roughly 3,800 lines of new Telegram-derived behavior landed only inside the stateful `MessagingEngine`, which the desktop cannot reach.

## Options considered

1. **Make `MessagingService` the desktop owner.** Rejected: whole-file JSON snapshot persistence with a process-global lock is not a durable multi-window/multi-process store; it has no server-authoritative sync with the Fabushi service; switching would discard the qualified desktop journals, SQLite CAS, restart and recovery behavior, and require a user-data migration from SQLite to JSON in the wrong direction.
2. **Keep both as peers.** Rejected: violates FMEP-OWNER-02 (one authoritative writer per semantic) and doubles every feature.
3. **Selected: one state owner, one shared pure domain policy library.**

## Decision

1. **State/persistence/sync authority** for Identity-bound Conversation and Message state is the Session owner (`ProductionSessionWorkers` + its SQLite/journals) backed by the Fabushi service as the remote authority. It is the only writer. Mobile embeds the same host composition and must converge on it (see Migration).
2. **`native/mahayana-messaging` (`fabushi_messaging_core`) is the canonical *pure domain policy* library**: permission/eligibility/authorization decisions, send-menu applicability, forward privacy/attribution, formatted-text transforms, search matching, paid-send planning and similar deterministic rules. Functions take explicit inputs and return decisions; they do not own storage, clocks, IDs or IO.
3. **`MessagingEngine`/`MessagingService` stateful paths are deprecated as owners.** Existing behavior is preserved for mobile until migration; no new capability may be implemented *only* there. New rules are written as pure functions in `fabushi_messaging_core` and called both by Session (desktop) and, until migrated, by the engine.
4. **Ledger rule.** A responsibility counts as desktop-implemented only when the Session/Transcript production path calls the pure function (FMEP-STATE-02). Engine-only implementations stay `mapped` with `[FMEP-STATE-02 domain-only wiring gap]`.

## Consequences / required follow-up

- **Extraction batch (next):** expose `MessagingEngine::resolve_destination_send_menu_details`, `resolve_destination_paid_restriction`, destination authorization and `FormattedText::prepend_plain_text` rules as pure functions over explicit inputs, keep the engine delegating to them (no behavior change), and call the applicable ones from `ProductionSessionWorkers` forward/send paths with contract tests.
- Topic/thread rules (`SHARE-THREAD-DESTINATION`, `SHARE-TOPIC-ELIGIBILITY`) stay `mapped` until the desktop Session owner supports Group/Topic conversations; they are not wired into a Human-only path artificially.
- **Mobile migration:** a versioned importer reads `_messaging/snapshot.json` once into the Session store (idempotent, keyed by message ID + revision), keeps the snapshot read-only for one release as rollback, then removes the standalone write path. Until then the snapshot remains the mobile store; no deletion of user data.
- Desktop production keeps rejecting standalone messaging (existing guard and tests remain).

## Verification

Extraction must prove identical engine outputs before/after (existing `native/mahayana-messaging/tests/*` unchanged and passing) and Session-side contract cases for each wired rule, executed in GitHub Actions. Local runs are `local-feedback` only.
