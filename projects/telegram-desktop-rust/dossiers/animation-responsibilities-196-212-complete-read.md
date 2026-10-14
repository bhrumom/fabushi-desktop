# Deterministic animation responsibilities 196–212 — complete source read

Accepted upstream: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

All 17 gzip/Lottie payloads were fully decompressed and structurally inspected, then traced to QRC packaging and active source consumers. No responsibility was inferred from filenames alone.

Orders 196–205 are action-result Toast assets: mute/unmute and pin/unpin reflect canonical action state after mutation; save-to-gallery and save-to-music report successful resource persistence; saved-messages/tagged report self-forward/tag state with navigation affordances; `star_premium_2` marks premium-locked rich formatting; `voip_invite` is in practice a generic success/info icon reused broadly for copied links, invites and completed actions. Fabushi must route these through canonical `Toast` plus each originating action owner; the animation may never become a second state owner.

Order 206 `transcribe_loading` is a shared async-loading glyph used by voice transcription, streamed draft loading emoji and moderation title resolution. Order 207 `ttl` is the looping global auto-delete/retention settings illustration. Orders 208–209 are ephemeral voice/media TTL visuals: `voice_ttl_idle` is active and loops in history media; `voice_ttl_start` is packaged but its intended one-shot start path is currently commented, so it is recorded as dormant rather than credited as an active shipping behavior.

Orders 210–211 are wallet phrase/backup assets: `wallet/paper` accompanies seed phrase introduction/import/restore flows and `wallet/test` accompanies randomized backup-word verification. Their applicability remains subject to Fabushi payment/wallet product policy; no TON-specific UI is imported by default. Order 212 `writing` is the Business Quick Replies settings explanation animation.

Every row is source-read/decomposed, but none receives implementation or verification credit from the asset itself. Required Fabushi owners are canonical Toast/action state, loading/transcription, Settings/retention, TranscriptEntry ephemeral-media state, payment policy, and Business Quick Replies. Focused production, visual, reduced-motion, theme and accessibility evidence remains open.
