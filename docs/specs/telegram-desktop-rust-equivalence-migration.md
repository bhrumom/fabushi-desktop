# Telegram Desktop Capability Research Source Sub-Spec

Status: active  
Spec ID: TDRP-001  
Revision: 4  
Last updated: 2026-10-02  
Parent product: `FBCP-001`  
Parent spec: `docs/specs/fabushi-bot-communication-platform.md`  
Related project: `projects/telegram-desktop-rust`

> Telegram Desktop is a **research source**, not the Fabushi runtime/network/provider. This sub-spec studies the complete frozen Telegram Desktop source to discover capabilities, state machines, edge cases, UX and implementation lessons, then maps each capability into the existing Fabushi architecture.

## 1. Highest-order rules

- Fabushi / PR #20 is the only target architecture.
- Telegram Desktop is not the target product.
- Telegram network is not the target network.
- No Telegram Provider is required by this spec.
- No MTProto interoperability is required by this spec.
- No Telegram account/peer/message identity becomes Fabushi canonical identity.
- Every Telegram capability must first resolve an `existing_owner` in the current Fabushi architecture.
- Only when no suitable owner exists may a minimal `new_owner_proposal` be created.
- C++ source is studied; required product responsibilities are reimplemented in Rust/best-fit Fabushi code rather than shipped from the original C++.
- This is source-informed, not clean-room.

## 2. Fixed research baseline

`telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`

The upstream `dev` branch is discovery-only.

Machine-readable source provenance remains:

`projects/telegram-desktop-rust/upstream.lock.json`

## 3. What must be researched

The source inventory must discover all product capabilities, including at minimum:

- accounts/auth/multi-account
- contacts/identity/presence
- dialogs/folders/archive
- private messaging
- groups/supergroups
- channels/broadcast
- topics/forums/threads
- message variants
- reply/quote/forward/edit/delete
- reactions/polls
- drafts/scheduled/silent send
- search
- media/file transfer/cache/streaming
- voice/video
- calls/screen sharing
- stickers/GIF/custom emoji
- Stories
- notifications
- settings/privacy/local lock
- bots/inline interactions
- Mini Apps/WebView concepts
- Premium/Stars/gifts/business
- payments/security flows where applicable
- export/local data/migration
- themes/language/RTL/IME/accessibility
- desktop lifecycle/install/update/recovery
- protocol/sync concepts that reveal mature communication edge cases

A source-discovered capability not listed above is still in scope.

## 4. Research dossier

Every capability dossier must include:

- capability ID/name
- exact upstream commit
- source references
- user-visible behavior
- state machine
- ordering/idempotency/retry/cancellation
- persistence/restart semantics
- concurrency/thread constraints
- negative/error cases
- security/privacy
- performance/resource constraints
- platform differences
- dependencies/resources
- observed strengths
- observed technical debt
- behavioral oracle
- C++ production responsibilities
- candidate `existing_owner` in current Fabushi
- absorption changes required in that owner
- `new_owner_proposal` only if no valid existing owner
- native-network implications
- test scenarios
- license/provenance

## 5. Owner resolution

For each capability:

```text
research
  ↓
candidate existing owners
  ↓
inspect current exact-head implementation
  ↓
choose existing_owner
  ↓
define absorption_plan
```

If no owner fits:

```text
existing_owner = none
new_owner_proposal = <minimal responsibility>
new_owner_adr = required
rejected_existing_owners = required with reasons
```

A broad new `CommunicationCore`, `TelegramRuntime`, `MessengerSubsystem` or equivalent is prohibited.

## 6. Initial absorption hypotheses

| Telegram capability | Candidate existing Fabushi owner |
| --- | --- |
| Dialog list | sidebar / conversation list |
| Chat/history | conversation workspace / transcript |
| Message variants | transcript model/cards |
| Composer/draft | composer / draft state |
| Reactions | transcript reaction model |
| Group/member | Shared Room / member model |
| Permissions/admin | existing permission + room controller |
| Scheduled send | Automations + composer |
| Attachments/media | attachments / artifacts / resource lifecycle |
| Bot interactions | Agent/composer interaction model |
| Search | existing search/command architecture |
| Settings/privacy | existing settings / permission surfaces |
| Screen sharing | Computer/realtime/call ownership after ADR |
| Notifications | existing desktop lifecycle/notifications |
| Calls | current calls surface if valid; otherwise minimal new call owner |
| Channel/topic | extend room/conversation model before proposing new owner |
| Stories | product shell or minimal Story owner after analysis |

These are hypotheses, not acceptance evidence.

## 7. C++ → Rust meaning

This project no longer means “port Telegram C++ modules into a Telegram clone.”

It means:

1. understand the responsibility implemented by the C++ source;
2. decide whether Fabushi needs that responsibility;
3. find the existing Fabushi owner;
4. implement the required behavior in that owner, preferring Rust for native/runtime/state-machine logic;
5. do not ship the original Telegram/desktop-app C++ implementation as the product owner.

For non-C++ source/resources, use the best-fit language/format.

## 8. Protocol and network research

MTProto/TL/session/update code remains useful research material because it exposes:

- ordering
- replay/duplicate behavior
- gap recovery
- multi-device synchronization
- session failure
- server/client clocks
- file transfer
- reconnect
- durability
- large history behavior

But Fabushi is free to design a better native protocol.

No MTProto compatibility test is an acceptance requirement unless a future separate interoperability spec explicitly adds it.

## 9. P0 output

P0 must produce:

- complete recursive source/dependency/resource inventory
- C++ production responsibility inventory
- full Telegram capability graph
- source → capability research coverage
- existing-owner inventory from current PR #20/canonical architecture
- capability → existing-owner mapping
- absorption plan per capability
- minimal new-owner proposals only when necessary
- native-network requirements discovered from Telegram behavior
- first research dossiers
- ADR backlog
- fail-closed capability ledger

## 10. Acceptance

TDRP-001 is accepted only when:

- research coverage is complete;
- every capability has an owner resolution;
- every applicable capability has an FBCP absorption plan;
- no source area with product relevance is silently ignored;
- C++ production responsibilities that enter Fabushi have Rust/best-fit production implementations or explicit blockers;
- no Telegram clone/provider architecture has been introduced;
- exact-head evidence proves the implemented Fabushi capability behavior;
- licensing/provenance review is complete.

TDRP acceptance does not by itself mean full FBCP product acceptance.

## 11. Execution environment

All executable checks run only in GitHub Actions or `htch-runtime`.

No local build/test/lint/generator/schema/benchmark/fuzz/package/acceptance.
