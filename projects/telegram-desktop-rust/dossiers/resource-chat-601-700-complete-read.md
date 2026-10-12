# Telegram chat resources 601–700 — complete source read

Upstream: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

Evidence: Fabushi Desktop `a37b4b1f8856c162c88d8de07544df1c1385c2f2`; GitHub Actions run `37851562740`; source-authority job `113565442156`; artifact `11582785137`; digest `sha256:f97372546b0e50d81168b3c00174be929d85e0d303090278f6b61f195cb9420c`.

## Evidence contract
`source-binary-evidence-601-700.txt` re-hashes every order 601–700 file from the accepted upstream checkout and records exact blob SHA, byte size and file type. `source-consumer-trace-601-700.txt` records exact shipping/style consumers for semantically sensitive resources. `source-consumer-reachability-601-700.txt` records fail-closed probes for unresolved names. No Telegram artwork is copied into Fabushi by source-read credit.

## Responsibility groups
- 601–602: `input_video` scale variants continue order 600; no accepted-tree named consumer, so no video-recording behavior is inferred.
- 603–616: away/locked/direct-message/quick-reply/welcome/tag empty/about states -> existing ConversationWorkspace, EmptyState, participant/profile and entitlement owners.
- 617–625: media enlarge, live-location presentation, markup WebView/Mini App -> canonical media viewer, message/resource model and existing Mini App/WebMCP owner.
- 626–628: chat-tab placement variants -> canonical responsive Tabs/navigation.
- 629–659: compact copy/external-link/gift/privacy/lock/TTL/quote/payment/subscriber/currency markers and Bot thread/topic creation -> canonical TranscriptEntry, payment/entitlement, Bot and typed ConversationCreationSurface owners.
- 660–662: paid-message approve/decline/edit -> canonical message action + payment authorization boundary.
- 663–683: reaction picker strip/bubble/expand/round assets -> canonical reactions Popover/Picker. Orders 672–677 (`reactions_premium_bg/star`) have no named consumer and remain reachability-open.
- 684: compose-AI refresh -> existing Agent/Composer AI assistance owner.
- 685–693: reply attribution channel/group/user -> canonical TranscriptEntry reply metadata.
- 694–695: history summary; 696 text-to-file Composer attachment conversion; 697 unsupported-content notice; 698–700 voice transcription/premium availability -> existing summary, Composer/resource, ErrorState/Banner, transcription/entitlement owners.
- 636 `mini_hidden` has no accepted-tree named consumer and remains reachability-open.

## Accounting
Read-through advances 600 -> 700. Unread falls 15,520 -> 15,420. Unknown stays 16,033 and omitted stays 0; source-read evidence does not fabricate production verification. All applicable production/visual/a11y/release gates remain open until their exact-head evidence closes them.
