# Menu foundations and send responsibilities 5184-5210 complete read

Status: exact-source read complete; all responsibilities remain mapped-open except explicitly identified existing partial Fabushi slices.
Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`
Accepted root tree: `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`

## Scope

Orders 5184-5210 were fetched directly from the accepted Telegram tree and read in deterministic order. The range covers checked/thumbnail menu primitives, anti-spam moderation, macOS Dock actions, emoji status, multi-file download, transcription rating, mark-as-read, notification mute/sound and the broad SendMenu family.

Generic Qt menu widgets are not migration targets by name. Their presentation responsibilities map to Fabushi's canonical Menu/ContextMenu/ListRow/Checkbox/Switch/Dialog/AlertDialog/Toast/Button primitives. Domain state and mutations remain owned by existing source-neutral session/server/settings/resource/composer owners.

## Domain findings

- 5188-5189 anti-spam is more than a toggle: eligibility can be locked by server-configured member thresholds; mutation is server-authoritative; false-positive reporting maps fake event ids back to real messages.
- 5194-5195 Dock behavior is macOS-specific platform composition for account windows, quit and active media previous/pause-resume/next. Electron/macOS APIs are the correct platform adapter.
- 5196-5197 emoji status includes premium eligibility, set/remove settlement and undo restoring the previous status/expiry.
- 5198-5199 download action includes configured/default/temp destinations, folder prompting, safe filenames, timestamps, completion tracking and show-in-folder feedback.
- 5200-5203 transcription rating includes eligibility, one-shot positive/negative feedback, keyboard access, mutation and settlement feedback.
- 5204-5205 mark-as-read recursively covers histories/topics/sublists, muted/mention policy and confirmation above 1,000 unread messages.
- 5206-5207 mute includes thread/default notification state, forever/custom periods, durable custom duration choices, sound selection/on-off and volume.
- 5208-5210 SendMenu spans ordinary/silent/scheduled/send-when-online plus media quality, spoiler, caption placement, cover, price and effects. Fabushi already has a real source-neutral Human silent/scheduled slice in the canonical composer/submission/Coordinator/Host path; that slice is reused as partial evidence rather than recreated in a Telegram-specific menu runtime. Remaining applicable actions stay open.

Broad repository searches did not by themselves prove absence/non-applicability of the other domain owners, so none are marked omitted or unknown-closed.

## Accounting

After this direct read: recursive 16,120; read-through 5,210; unread 10,910; unknown 15,841; omitted 0. Reading alone grants no unknown, implementation, verification, baseline-ready or release credit.
