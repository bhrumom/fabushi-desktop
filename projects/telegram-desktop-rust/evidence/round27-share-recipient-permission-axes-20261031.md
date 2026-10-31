# Round 27 — Share recipient typed permission axes (2026-10-31)

Status: implemented / exact-head execution pending

## Live authority

- Fabushi main observed before implementation: `cfef45568293a22deb32a6cdef17916f3ec19648`.
- PR #178 branch pre-implementation head: `012150fa9eb961d934b3838ad9dc34fdaf3c336b`.
- Upstream source authority inspected: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`, `Telegram/SourceFiles/boxes/share_box.cpp`.
- Upstream behavior inspected in full source: forwarding checks `requiredSendRight()`, separately requires `ChatRestriction::SendInline` where applicable, rejects game delivery to broadcast destinations, and another share path requires `ChatRestriction::SendOther`.

## Existing-owner resolution and implementation

No new owner was introduced. The existing canonical owners were extended:

- `native/mahayana-messaging/src/conversation.rs`: `ConversationPermissions` now owns `can_send_other`, `can_send_inline`, and `can_send_games`. Missing fields in legacy persisted records deserialize false; fresh default permissions allow the capabilities.
- `native/mahayana-messaging/src/community.rs`: `MemberRestrictions` now owns matching `send_other`, `send_inline`, and `send_games` restriction axes.
- `native/mahayana-messaging/src/service.rs`: `actor_can_send_to_recipient` projects both Conversation allow axes and Community restriction axes into the existing `recipient_search_authorized` pre-exposure boundary instead of hardcoding the typed axes false.
- `native/mahayana-messaging/tests/m5_contacts_groups_contract.rs`: focused contracts cover allowed/denied typed recipient exposure, restricted-member denial, and legacy persistence fail-closed behavior.
- `.github/workflows/rust-desktop-runtime.yml`: exact-head messaging evidence manifest now names `share-recipient-send-other-inline-game-permission-axes`.

Production implementation commits in this branch include:
- `4e66954b7fb6859108f0086c57eaf824d8a29378`
- `fc792c61f29aa5d6b415bf99ede958ea60d6a0d0`
- `0c5bf2551a6c730eedfc1ea19bab81738b9a9103`
- `1e3a85488cb93fbe65657a15a817c474d98014da`
- `217fa8cef686e60a5bba565a95419be6d6711046`
- `7e8f68d22e6220444061f54f1aca5aaf698b7b03`
- `82127ec395c9ab6a13b68b235267864e0659bd2b`
- `38bc48076d3e17fded451f7e869e5f7e5685369e`

The oracle, RTM, ShareBox dossier, and `TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001` ledger row were updated afterward without changing the row to VERIFIED.

## Fail-closed gate

No executable test was run outside GitHub Actions. This evidence document is not a passing test result and is not independent release acceptance.

Still open:
- exact-head GitHub Actions terminal-success evidence and artifact identity;
- shipping recipient enumeration beyond Human-direct to canonical Group/Channel/topic/saved-sublist/nested Conversation destinations;
- authoritative synchronized policy refresh across the complete shipping destination universe;
- Electron packaged E2E, temporal/reload/restart/recovery evidence;
- full upstream recursive source closure, all remaining responsibilities, and independent release ACCEPT.

Do not merge or release based on this snapshot.
