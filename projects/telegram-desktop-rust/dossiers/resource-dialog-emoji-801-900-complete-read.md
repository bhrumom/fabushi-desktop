# Telegram dialog and emoji resources 801–900 — complete source read

Upstream: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

Evidence: Fabushi Desktop `1bfb51f03daf9fa5dd0309b6fe1152a1c06f497e`; GitHub Actions run `37854075471`; source-authority job `113574164576`; artifact `11583202909`; digest `sha256:dd803339010524179896b959b8aad11bb248198968584be1a80c9ddea5d0b968`.

## Evidence contract
`source-binary-evidence-801-900.txt` records exact blob SHA, byte size and file type for every order 801–900 entry. `source-consumer-reachability-801-900.txt` and `source-consumer-trace-801-900.txt` bind accepted-tree consumers. The machine rows preserve exact evidence and keep unresolved consumer reachability fail-closed.

## Responsibility groups
- 801–802 complete the premium-marker scale family begun at 800.
- 803–805 `dialogs_reaction` have no accepted-tree named consumer and remain reachability-open.
- 806–836 cover received/sending/sent state, search-from filtering, topic navigation, unread media, verified identity, inaccessible users and locked/search navigation details.
- 837–878 cover emoji categories, back/delete, search, settings and skin-tone controls using the canonical Composer/Picker/Popover ownership boundary.
- 879–887 cover premium emoji/sticker entitlement and emoji empty-state presentation.
- 888–899 cover quick comment/share/go-to-original/go-to-message actions bound to TranscriptEntry/ResultRow and existing canonical navigation/share/composer owners.
- 900 begins the `folder_existing_chats` scale family; 901–902 remain part of the same semantic unit.

## Accounting
Read-through advances 800 -> 900. Unread falls 15,320 -> 15,220. Unknown stays 16,033 and omitted stays 0. Source-read credit does not fabricate production verification.
