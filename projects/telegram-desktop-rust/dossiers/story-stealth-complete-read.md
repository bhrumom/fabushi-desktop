# Story stealth-mode source-completeness dossier

Authority: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`  
Fabushi mapping baseline: `bhrumom/fabushi-desktop@3bc92400826cc4ca7ac665b467708e22261edc61`  
Status: read-complete / provenance-rebound at d346b42a1d30ef60dc989b6e5191bb8e571f6bd5 / mapped-open / not implemented / not verified

### Rebaseline provenance

The accepted upstream advanced 16 commits from `72b3b71c3d6e450e5ef94a3112dd750a0168aa0b` to `d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`. GitHub exact-file reads at the new commit confirm every source entry in this dossier has the same blob SHA as the prior read. Therefore the semantic read is provenance-rebound rather than inherited blindly; no changed story blob is being treated as read.

## Exact source entries

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/media/stories/media_stories_stealth.h` | `649cfa1eb5abcad11706741753bb70232fe43c7a` | complete | public Story anonymous-view activation/menu/time-left contract | mapped-open |
| `Telegram/SourceFiles/media/stories/media_stories_stealth.cpp` | `8bd47aaf5b5b454a6f2ac0b4641b285646d4a9b3` | complete | entitlement, activation, cooldown/timer, duplicate-request fencing and terminal feedback lifecycle | mapped-open |

Both files were read completely at the accepted upstream commit.

## Responsibilities recovered

### ST-STEALTH-01 — durable anonymous-view mode state and time boundaries

Symbols: `StateValue`, `SetupStealthMode`, `TimeLeftText`.

The transferable contract is an authoritative Story-view privacy mode with two independently meaningful deadlines: active `enabledTill` and reactivation `cooldownTill`. Projection is timer-driven at the exact state-boundary transitions rather than by an unbounded poll. Already-active activation is idempotent from the user's perspective and must invoke the requested continuation without issuing a second activation.

### ST-STEALTH-02 — eligibility, request fencing and activation settlement

Symbols: `StealthModeBox`, `MakeButton`, `AddStealthModeMenu`.

Activation is eligibility-gated. The upstream product distinguishes availability, paid entitlement, active state and cooldown state. A single surface-local `requested` fence prevents duplicate concurrent activation; the fence clears on failed completion and the authoritative state transition to active closes the surface, publishes feedback and invokes the continuation exactly once. Cooldown cannot be bypassed by repeatedly pressing the action.

## Revision 9 traceability

The two Story stealth ledger rows are governed by the existing Revision 9 requirements `TDRP-MOD-01`, `TDRP-MOD-02`, `TDRP-OWN-01`, `G-PRODUCTION`, `G-SECURITY-PRIVACY`, and `G-TEMPORAL`. These IDs bind the mapped privacy lifecycle to source/module completeness, canonical ownership, executable production authorization, privacy/security, and temporal recovery gates; they do not claim an implemented or verified stealth service.

## Existing-owner-first audit

Current Fabushi already has:
- canonical Story identity/privacy/view state in `native/mahayana-messaging/src/story.rs`;
- a payment model containing `PaymentKind::Subscription` and an `Entitlement` type in `native/mahayana-messaging/src/payment.rs`;
- ProductShell overlay/action composition.

However, the current exact-head audit did not establish an executable Story-stealth activation command, persisted `enabled_until/cooldown_until` Story privacy state, entitlement-to-Story-feature authorization, or server contract that suppresses viewer attribution while active. Therefore these files cannot be called implemented. The correct direction is to extend the existing Story/privacy plus payment/entitlement/service owners; creating `TelegramStealthMode`, a second Story store, or a premium-specific Story app would violate existing-owner-first.

## Required server/platform contract and evidence

Before this responsibility can advance beyond mapped:
- canonical Story state must represent anonymous-view activation/cooldown without overloading ordinary visibility;
- the service must authorize the feature against canonical entitlement, enforce cooldown server-side, settle activation idempotently and define multi-device clock/state semantics;
- Story view recording must suppress or transform viewer attribution only under the authorized active mode, with security/privacy tests proving no client-only bypass;
- ProductShell must project active/cooldown/unavailable/upgrade states from authoritative state and fence duplicate activation;
- GitHub Actions must cover deadline transitions, duplicate activation, late/out-of-order settlement, reconnect/restart/multi-device recovery, entitlement loss/expiry and packaged temporal UI behavior.

The existing payment `Entitlement` data type alone is not production evidence that this feature is authorized or deployed. These rows remain mapped-open, with the service/entitlement integration explicitly blocked.

Coverage after these two complete reads: `unread=15,755`, `unknown=15,788`, `omitted=0`, `baseline_ready=false`, `acceptance.accepted=false`.
