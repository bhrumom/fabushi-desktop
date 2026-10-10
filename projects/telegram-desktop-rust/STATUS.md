Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@28ac576967a1026ecc89a927360fa0e138d7c88c (root tree 9d8fa87ce68c832d2f4c99f372a5bb672463c3e1). Root non-directory=6,653; recursive non-directory=16,125; read-through=6,367; first unread=6,368 Telegram/SourceFiles/webauthn/cable.h@6909eb913dbb41765ebbaeecb2072f130f868655; unread=9,758; unknown=15,846; omitted=0. Reading alone never closes unknown. Fresh descendant exact-head GitHub Actions evidence is required.
<!-- TDRP_CURRENT_SUMMARY read-through=6367 unread=9758 unknown=15846 omitted=0 first-unread=6368 path=Telegram/SourceFiles/webauthn/cable.h -->

# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-10-07  
Spec: TDRP-001 Revision 9  
Parent: FBCP-001 Revision 7  
Complete: false

| Area | Status | Note |
| --- | --- | --- |
| Product role | active | tdesktop is complete source authority, not a runtime/provider |
| Discovery upstream | observed | `f23c37857220eb84f8559f0901ea26fb304b564b`; not yet accepted baseline |
| Historical baseline | stale | `33261535a0e747f125e0ed25486f01e556330677`; evidence cannot be inherited |
| Recursive source closure | rebaseline required | current lock/inventory must be reconciled to one exact upstream baseline |
| File-level inventory | not accepted | must reach unknown=0, unread=0, omitted=0 |
| Responsibility decomposition | not accepted | every source symbol/responsibility requires traceable destination |
| Exact-head owner inventory | in progress | existing-owner-first remains mandatory |
| UI composition / IA | in progress | one shell, one Conversation/Search/Profile/Marketplace truth |
| Design-system migration | in progress | new migrated UI must use Fabushi semantic tokens/components |
| Native service completeness | in progress / blocked where absent | blocked service is not functional acceptance |
| Production implementation | partial | existing slices do not imply full migration |
| Existing Bot regression | required | Agent/Host/Coordinator/Runner/Plugins/MCP/Computer/Automations/Tasks etc. must remain green |
| Quality/RTM gates | not fully implemented | Revision 9 schema/validator fields and traceability still require executable gates |
| Packaged acceptance | fail-closed required | acceptance failure must block release |
| Independent release acceptance | not accepted | final verdict must be ACCEPT |

## Current live reconciliation

On 2026-10-07, `main` was observed at `e0a7a4221970088beeaf145d0d9be1763e58229f`. That commit weakened the real signed packaged acceptance step with `continue-on-error: true`, which conflicted with FBCP Revision 7 / TDRP Revision 9. The regression was removed on `main` by commit `2115a91eb888aa65702ef8f90f01848042bb65c7`.

This status file records only observed repository state. It is not completion evidence. Any newer `main` supersedes these SHAs and must be re-read before implementation or acceptance.

## Metrics

Report separately:
- accepted upstream baseline completeness;
- source file read coverage;
- responsibility understanding coverage;
- existing-owner resolution coverage;
- production implementation coverage;
- exact-head test/evidence coverage;
- packaged temporal acceptance;
- independent release acceptance.

Do not report Telegram-provider, Telegram-network, survey, document count, or vertical-slice progress as migration completion.


### cf478d37 live-authority rebaseline

Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@cf478d37c8f57df831cdedb5621e1cf2ef069a0f (root tree fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,793; first unread=5,794 Telegram/SourceFiles/test/test_log.cpp@17c72a8ae1bc672e7c380b855887b859b78fabec; unread=10,332; unknown=15,846; omitted=0. Changed accepted-prefix orders 11/30/31/92/4599/5759/5764/5765 were explicitly re-read; lib_ui is now order 6,589; runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 are historical-only for 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e + b4d9f7f8cf580eefd553c5ce105f1d3e682de87f.

Direct delta: channel_earn.style replaces static negative placeholder margins with floating placeholderShiftLeft; lib_ui 91ff446 implements horizontal interpolation in InputField and MaskedInputField. Canonical TextField remains the sole product owner. Orders 5,774-5,793 were identity/order reconciled; reading alone closes no unknown.


### Revision 9 live read-through note: 5794-5801

Orders 5,794-5,797 preserve professional evidence-log one-line/completion-forgery integrity plus an independent raw-byte oracle. Orders 5,798-5,799 preserve complete mapped-target/viewport capture readiness. Orders 5,800-5,801 preserve the bounded reversible not-marking-read evidence lever. These are test/evidence responsibilities only, create no second product owner, and close no unknown. The new 811b83a1 readability delta is order 5,951 and remains unread/unknown.


### Revision 9 live read-through note: 5802-5805

Orders 5,802-5,803 close the exact-blob read/decomposition of popup/context-menu professional evidence semantics: same-turn fresh-menu identity, explicit refusal taxonomy, isolated QAction queued-callback delivery, prepared-frame capture and lock/teardown behavior. Orders 5,804-5,805 close the SentMessageWatcher client→server id reconciliation contract with stable-history/candidate fencing and a five-second diagnostic probe throttle. These are test/evidence responsibilities only and close no unknown. First unread is now 5,806 `test_notify_override.cpp`.

### Revision 9 live rebaseline/read-through note: b0d1fe5e / 5166 / 5806-5813

The 811b83a1→b0d1fe5e upstream delta changes only order 5,166 `media_view_overlay_widget.cpp`: the old `forbidsSaving()` control-refresh branch hid controls and could call `DocumentSaveClickHandler::Save(...ToCacheOrFile)` when media was not loaded; b0d1fe5e removes that viewer-driven automatic persistence path. The new blob `e35f67119323850a3cea450548566186c8a2231c` was re-read and the mapped-open MediaViewer lifecycle responsibility now explicitly requires protected/forbids-saving media not to be auto-persisted as a rendering/control-refresh side effect. Path order and total counts are unchanged. Separately, orders 5,806–5,813 are read-complete/responsibility-decomposed professional test harnesses for reversible notify overrides/no-server-write locality, document-open handoff inspection without OS launch, SeparatePanel pointer/liveness evidence, and post-paint temporal sampling. They add no product owner, fake fallback, unknown closure or release credit. First unread is 5,814 `test_probe.cpp`; read-through is 5,813/16,125, unread 10,312, unknown 15,846, omitted 0.


### Revision 9 live read-through note: 5814-5821

Orders 5,814–5,821 are exact-blob read-complete/responsibility-decomposed professional evidence harnesses. Probe preserves mark-bounded, keyed issue→answer correlation with explicit ambiguity/outstanding states and positive controls; controlled RPC fixtures reject malformed/truncated/trailing/wrong-constructor buffers before live parser delivery; retry evidence distinguishes 500 retry registration from 400 fail/unregister and unknown-request controls while logging only code/type/constructor; Runner preserves bounded stage/watchdog execution, explicit N/A, QPointer/paint readiness, exactly-once onFinish teardown, disposable-copy markers, completion drain and post-quit fuse behavior. These rows add no product owner or fake fallback and close no global unknown. First unread is 5,822 `Telegram/SourceFiles/test/test_scenario.cpp`; read-through is 5,821/16,125, unread 10,304, unknown 15,846, omitted 0.

### Revision 9 live read-through note: 5822-5824

Orders 5,822–5,824 are exact-blob read-complete/responsibility-decomposed professional evidence code. The checked-in scenario slot is deliberately no-op/test-only. The secrecy oracle is fail-closed over accepted launch identities, controls, bidirectional MTP headers and canaries; distinguishes client-written plain/Send from report-only Recv, withholds unsafe sites, narrowly handles declared computed fields, emits only public evidence and rescans its own rows. Missing/foreign/unreadable/control/canary/input failures are undecided rather than clean. These rows close no global unknown and create no product owner. First unread is 5,825 `Telegram/SourceFiles/test/test_style.cpp`; read-through is 5,824/16,125, unread 10,301, unknown 15,846, omitted 0.

### Revision 9 live read-through note: 5825-5830

Orders 5,825–5,830 are exact-blob read-complete/responsibility-decomposed UI evidence harnesses: stable-window style/baseline verification, real InputField text-drop delivery/refusal/ancestor shielding with no clipboard-content access, and narrowly-scoped U+00A0/U+202F text-read normalization with deliberate negative controls. They create no production owner and close no global unknown. First unread is 5,831 `Telegram/SourceFiles/test/test_toast_capture.cpp`; read-through 5,830/16,125, unread 10,295, unknown 15,846, omitted 0.


### Source closure through 5,848

Under live `65e23ba7` authority, orders **5,837–5,848** were completely read and exact-blob dispositioned. Accounting is now **5,848/16,125 read, 10,277 unread, 15,846 unknown, 0 omitted**; these test/evidence-only rows do not reduce global unknown. Fresh exact-head Actions are required.


### Source closure through 5,859

Under live `65e23ba7` authority, orders **5,849–5,859** are now exact-blob read-complete/responsibility-decomposed. Tests 5,849–5,854 are professional evidence-only responsibilities; Tray 5,855–5,859 remains mapped-open production work pending canonical source-neutral owner/composition and exact-head evidence. Accounting: **5,859/16,125 read, 10,266 unread, 15,846 unknown, 0 omitted**. First unread is 5,860 `ui/boxes/about_cocoon_box.cpp@4aab8e79…`. No baseline-ready or release credit is granted.


### Source closure through 5,871

Orders **5,860–5,871** (`ui/boxes`) are exact-blob read-complete/responsibility-decomposed. They remain mapped-open product/UI responsibilities; reading does not reduce unknown. Current accounting: **5,871/16,125 read; 10,254 unread; 15,846 unknown; 0 omitted**. First unread 5,872 `choose_font_box.cpp@56a7386c…`. No baseline/release credit is granted.


### Source closure through 5,909

The deterministic live prefix is exact-blob read-complete/responsibility-decomposed through **5,909**, completing `ui/boxes/**`. Accounting is **5,909/16,125 read; 10,216 unread; 15,846 unknown; 0 omitted**. First unread is 5,910 `ui/cached_round_corners.cpp@7da69a22…`. The predecessor d52b8c5 authority failure identified a stale `upstream.lock.json.source_disposition_evidence.read_through`; this descendant fixes that data contract without modifying or weakening the validator. No baseline-ready, accepted, implementation, or release credit is granted.


### Source closure through 5,980

Orders **5,910–5,980** are exact-blob read-complete/responsibility-decomposed. Accounting is **5,980/16,125 read; 10,145 unread; 15,846 unknown; 0 omitted**. First unread is 5,981 `ui/color_contrast.cpp@f981e717…`. Parent `9c969d1` repaired the validator-required narrative tokens; this batch preserves that exact contract. No baseline-ready, accepted, implementation or release credit is granted.


### Source closure through 5,991

Orders **5,981–5,991** are exact-blob read-complete/responsibility-decomposed for shared color semantics and canonical button busy/context-menu/two-label behavior. Busy presentation is not credited as duplicate-operation refusal; the command owner still needs an independent in-flight fence. Accounting: **5,991/16,125 read; 10,134 unread; 15,846 unknown; 0 omitted**. First unread 5,992 `call_button.cpp@af74333f…`. No baseline/release credit is granted.


### Source closure through 6,001

Orders **5,992-6,001** are exact-blob read-complete/responsibility-decomposed for call action/mute state projection, service Checkbox presentation, compose-AI/large-paste classification and custom-emoji Toast projection. They map to source-neutral canonical owners; no Telegram-named component is introduced. The shipping Human call owner now reuses canonical `SandButton` and gives mute a command-level in-flight fence with failure restoration and teardown, but broader call/composer/toast responsibilities remain mapped-open pending same-head evidence. Accounting: **6,001/16,125 read; 10,124 unread; 15,846 unknown; 0 omitted**. First unread 6,002 `delete_message_context_action.cpp@9cecf76e…`. Reading/partial implementation grants no baseline or release credit.


### Source closure through 6,011

Orders **6,002-6,011** are exact-blob read/decomposed for TTL-aware delete ContextMenu actions, aggregate download-status presentation, keyboard/pointer dynamic image selection, and emoji IconButton/input-field picker composition. They map to canonical ContextMenu + Message expiry/delete, Downloads/Resource + status surface, Avatar/Picker, and IconButton + Composer/TextField + Popover/emoji-provider owners. No source-named component is introduced. Accounting: **6,011/16,125 read; 10,114 unread; 15,846 unknown; 0 omitted**. First unread 6,012 `feature_list.cpp@8c3bf36c…`. Reading grants no implementation, verification, baseline or release credit.

### Source closure through 6,032

Orders **6,012-6,032** are exact-blob read/decomposed for feature/detail rows, filter/share header state, invite-link actions/ContextMenu, jump-down unread projection, labeled emoji Tabs, location-picker service/UI boundaries, participant LoadingState skeletons and Popover lifecycle. All map to source-neutral canonical owners; reading does not reduce unknown. The descendant Human-call shipping surface also serializes camera/screen-share mutations through one `video-media` command fence with canonical pending projection, rollback and capture teardown. ForceMuted/RaisedHand/scheduled/audio-reactive call presentation remains mapped-open. Accounting: **6,032/16,125 read; 10,093 unread; 15,846 unknown; 0 omitted**. First unread 6,033 `ui/controls/round_video_recorder.cpp@b52b73bd…`. No baseline/release credit is granted.

### Source closure through 6,041

Orders **6,033-6,041** are exact-blob read/decomposed for round-video recording/encoding, sender identity, the primary Composer send-state control and silent-send accessibility. Round-video remains mapped-open as a real device/media/codec lifecycle, not a cosmetic widget. The descendant Composer now also removes its raw HTML send button and reuses canonical `SandIconButton`. Accounting: **6,041/16,125 read; 10,084 unread; 15,846 unknown; 0 omitted**. First unread is 6,042 `ui/controls/stars_rating.cpp@7b7dce6e…`.

### Source closure through 6,052

Orders **6,042-6,052** are exact-blob read/decomposed for Stars rating, source-neutral tabs/subsection reorder and swipe gesture/scroll ownership. The existing canonical `SandTabs` owner is extended with optional overflow visibility, context-menu requests, locked reorder boundaries, pointer cancellation/edge-scroll and Alt+Arrow keyboard reorder; no Telegram-derived Tabs owner was added. Stars account/reputation truth, complete Avatar/Badge subsection composition and the navigation/conversation swipe owner remain mapped-open. Accounting: **6,052/16,125 read; 10,073 unread; 15,846 unknown; 0 omitted**. First unread is 6,053 `ui/controls/tabbed_search.cpp@ca207d0b…`. Reading alone does not reduce unknown.

### Source closure through 6,068

Orders **6,053-6,068** are exact-blob read/decomposed for grouped search/category picking, detail rows and transient tooltip lifetime, title/status chrome, localized fixed-precision amount input/IME, ephemeral-media countdown presentation, and Avatar/profile-media acquisition/upload/privacy/streaming lifecycle. The existing canonical `SandTooltip` gains optional auto-dismiss, outside-press/Escape dismissal and focus-return policy with a focused contract test; no Telegram-derived Tooltip owner was added. Search/Picker shipping composition, full profile-media ownership, monetary-domain applicability and authoritative ephemeral-message lifecycle remain mapped-open. Accounting: **6,068/16,125 read; 10,057 unread; 15,846 unknown; 0 omitted**. First unread is 6,069 `ui/controls/who_reacted_context_action.cpp@6851dee0…`. Reading alone does not reduce unknown.

### Source closure through 6,084

Orders **6,069-6,084** are exact-blob read/decomposed across who-read/reaction context actions, platform-obsolescence and screen-reader-mode Banners, country selection, dynamic media thumbnails, animated text, and credits/commerce graphics/style. Canonical owners remain source-neutral and all rows remain mapped-open unless independently closed; reading does not reduce unknown. Accounting: **6,084/16,125 read; 10,041 unread; 15,846 unknown; 0 omitted**. First unread is 6,085 `ui/effects/drifting_particles.cpp@ed119cf9…`.


### Source closure through 6,104

Orders **6,085-6,104** are exact-blob read/decomposed across decorative particles, reaction/custom-emoji fly overlays, fireworks/glare/LoadingState skeletons, Composer-to-Transcript message-send transitions, Stars particles/Story outlines, Premium commerce styling and an optional GPU 3D commerce cover. The source responsibilities require pause/teardown correctness, reduced-motion/power-policy handling, target-identity fencing, theme/DPR/RTL/responsive behavior, resource completion recovery and GPU capability fallback. They map only to source-neutral canonical animation/LoadingState/Transcript/Resource/Avatar/Status/commerce/design-system owners; no Telegram-derived parallel visual runtime is accepted. Accounting: **6,104/16,125 read; 10,021 unread; 15,846 unknown; 0 omitted**. First unread is 6,105 `ui/effects/premium_3d_mesh.cpp@b8541b31…`. Reading alone does not reduce unknown or grant implementation/release credit.


### Source closure through 6,125

Orders **6,105-6,125** are exact-blob read/decomposed across validated 3D asset loading, GPU support gating, reactive credits/limit/subscription presentation, coin/diamond RHI renderers, promo particle strategies and the interactive Premium Star lifecycle. Domain truth stays in canonical commerce/credits/gifts/subscription owners; graphics reuse canonical controls and the thinnest visual/GPU adapters. Malformed assets, unsupported RHI, shader/buffer/pipeline failures, power-saving/reduced-motion, pause/resume and partial-init teardown remain explicit failure/lifecycle obligations. Accounting: **6,125/16,125 read; 10,000 unread; 15,846 unknown; 0 omitted**. First unread is 6,126 `ui/effects/premium_star_model.cpp@499d751a…`. Reading alone grants no implementation or release credit.


### Platform-obsolescence Banner implementation slice

The accepted orders **6,071-6,072** now have a source-neutral implementation slice: shared status primitives expose one canonical `SandBanner`, and `frontend/src/production/platform-obsolescence-policy.ts` preserves the exact civil-date **7/30/90-day** dismissal rule with fail-open handling for corrupt/future persisted dates and an adapter for canonical client persistence. This deliberately does **not** invent an OS-support cutoff: the current Desktop bridge has no authoritative `WhenSystemBecomesOutdated` equivalent, so platform cutoff/reason sourcing and root shipping composition remain mapped-open. The focused contract is wired into the existing renderer GitHub Actions gate; no local build/test was used. Unknown remains unchanged.


### Source closure through 6,145

Orders **6,126-6,145** are exact-blob read/decomposed across Premium Star assets/particles/GPU renderer, colored collectible decoration, responsive commerce top-bar capability fallbacks, reaction fly presentation, Checkbox/Avatar selection rendering, scroll-edge shadows, and send-action status animations. Send-action types include record/upload/round/speaking/choose-sticker with typing fallback; same-family restart and speaking finish/restart transitions remain visual projections over authoritative conversation state, and animation-disabled mode resolves to static frames. All responsibilities map to source-neutral canonical owners; reading does not reduce unknown. Accounting: **6,145/16,125 read; 9,980 unread; 15,846 unknown; 0 omitted**. First unread is 6,146 `ui/effects/shake_animation.cpp@38fbd6f0…`.


### Source closure through 6,165

Orders **6,146-6,165** are exact-blob read/decomposed across shake/LoadingState skeleton feedback, snowflake/star-burst decoration, the message-removal dissolve capture/collapse/GPU compute lifecycle, disclosure-arrow affordance and TTL timer-icon projection. The dissolve path is explicitly presentation after authoritative deletion: it capability-gates power/RHI/compute, pre-captures exact Transcript items, reconciles collapse gaps and scroll baselines, bounds GPU particles/frame delta, and tears down pending/per-item resources; it never owns deletion truth. Accounting: **6,165/16,125 read; 9,960 unread; 15,846 unknown; 0 omitted**. First unread is 6,166 `ui/effects/unique_gift_message_bubble.cpp@22802623…`.


### Read-through 6166-6171

Accepted Telegram authority remains `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`. Orders **6,166-6,171** are exact-blob read and responsibility-decomposed: unique-gift message bubble geometry maps to canonical TranscriptEntry/Avatar plus Gift/Commerce presentation; upload progress lifecycle maps to canonical Resource/attachment upload + Status/Progress/IconButton; voice-once particles map to canonical voice-message playback presentation with reduced-motion/teardown policy. Reading alone does not reduce unknown. Accounting after orders 6,166-6,171 was **6,171 / 16,125 read**, **9,954 unread**, **15,846 unknown**, **0 omitted**. The next unread order was **6,172** `Telegram/SourceFiles/ui/empty_userpic.cpp`.

### Read-through 6172-6182

Accepted Telegram authority remains `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`. Orders **6,172-6,182** are exact-blob read and responsibility-decomposed: fallback identity presentation maps to canonical Avatar/semantic Icon/Resource owners; filter-icon selection maps to canonical Picker/Popover/Menu/IconButton with the existing conversation-filter domain owner; grouped-media geometry maps to the canonical TranscriptEntry/Resource layout utility and must remain deterministic and single-owned. Reading alone does not reduce unknown. Accounting after orders 6,172-6,182 was **6,182 / 16,125 read**, **9,943 unread**, **15,846 unknown**, **0 omitted**. The next unread order was **6,183** `Telegram/SourceFiles/ui/image/image.cpp`.

### Read-through 6183-6191

Orders **6,183-6,191** are exact-blob read and responsibility-decomposed across the image/resource base layer: prepared image cache keys and DPR transforms; typed download/image locations with versioned serialization, cache-key derivation and file-reference refresh; protocol-to-resource factories including progressive/cached/in-memory/web/video forms; bounded local-image decoding; and sanitized SVG preview rendering with explicit byte/dimension limits. These map to the existing canonical `Resource`/attachment/media owner rather than a second download/cache owner. Reading does not reduce `unknown`. Current accounting is **6,191 / 16,125 read**, **9,934 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,192** `Telegram/SourceFiles/ui/item_text_options.cpp`.

### Read-through 6192-6202

Orders **6,192-6,202** are exact-blob read and responsibility-decomposed across conversation-aware text option projection, single-window layer-stack lifecycle, canonical context-menu icon semantics, new/attention Badge projection, and local passcode-strength/transliteration helpers. These responsibilities map to existing source-neutral TranscriptEntry, Dialog/Popover, ContextMenu/Menu/Icon, Badge/Status and security/passcode owners. Sensitive passcode candidates must remain ephemeral and unlogged. Reading alone does not reduce unknown. Current accounting is **6,202 / 16,125 read**, **9,923 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,203** `Telegram/SourceFiles/ui/peer/color_sample.cpp@d11ac016c6e318786c3a1206efa415c91928ee21`.

### Read-through 6203-6214

Orders **6,203-6,214** are exact-blob read/decomposed across color/profile selection, video Avatar Resource playback, power-saving/reduced-motion policy, resize lifecycle, bounded row-scroll render caching and SearchField query/a11y composition. All map to existing source-neutral canonical owners and remain mapped-open. Reading alone does not reduce unknown. Accounting: **6,214 / 16,125 read**, **9,911 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,215** `Telegram/SourceFiles/ui/text/format_song_document_name.cpp@d5a77025b392b8385bff3268be1f7465d91d48a2`.

### Read-through 6215-6224

Orders **6,215-6,224** are exact-blob read/decomposed across media naming, locale/value/currency/credits formatting, custom-emoji animation lifecycle and canonical rich-text parser presets. All remain mapped-open; reading alone does not reduce unknown. Accounting: **6,224 / 16,125 read**, **9,901 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,225** `Telegram/SourceFiles/ui/top_background_gradient.cpp@7325d188f856d76f08c67a2bdb43ce1f50eb94b7`.


### Read-through 6225-6239

Orders **6,225-6,239** are exact-blob read/decomposed across commerce/profile top-gradient pattern rendering, unread/peer Badge precedence and animation/power-saving behavior, unread counter formatting, Avatar/Resource cache invalidation and fallback shapes, canonical vertical-list composition, and WebView theme/zoom/style serialization plus attribute/script escaping. All remain mapped-open and source-neutral; build-only `ui_pch.h` is recorded without claiming product closure. Reading alone does not reduce unknown. Accounting: **6,239 / 16,125 read**, **9,886 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,240** `Telegram/SourceFiles/ui/widgets/chat_filters_tabs_mode.h@f5d37194063110eb2ba8698f5b714e8a7ebe37ea`.


### Read-through 6240-6244

Orders **6,240-6,244** are exact-blob read/decomposed across persisted chat-filter Tabs presentation mode, canonical Tabs/Badge/ContextMenu composition, locked-range handling, custom-emoji pause, and drag-reorder lifecycle with pinned intervals, threshold start, edge auto-scroll, cancel/apply convergence and stable index remap. These remain mapped-open and must reuse existing source-neutral Tabs/ContextMenu/Badge/Status plus the single canonical ordering state owner. Reading alone does not reduce unknown. Accounting: **6,244 / 16,125 read**, **9,881 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,245** `Telegram/SourceFiles/ui/widgets/chat_filters_tabs_strip.cpp@750feab6cbf3b30a3b1e2bc946d85eaed4a4382c`.


### Read-through 6245-6251

Orders **6,245-6,251** are exact-blob read/decomposed across the shipping chat-filter Tabs composition, premium lock/menu/edit/remove/mark-read and saved ordering lifecycle, plus canonical Color Picker/validated field synchronization and Continuous/Media Slider pointer-wheel-keyboard-a11y progress/finished semantics. These remain mapped-open and must reuse source-neutral Tabs/ContextMenu/Badge/Picker/TextField/Slider and existing filter/settings/media domain owners. Reading alone does not reduce unknown. Accounting: **6,251 / 16,125 read**, **9,874 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,252** `Telegram/SourceFiles/ui/widgets/cross_fade_label.cpp@fb094439ceebc0ad1b45db68ebf08d31450464e3`.


### Read-through 6252-6261

Orders **6,252-6,261** are exact-blob read/decomposed across cross-fade text motion, Discrete/Settings Slider selection timing and ripple state, expandable participant Checkbox/Avatar collection semantics, phone/country/username masked-input normalization including Windows IME re-entry fencing, and count-aware time-part placeholders. These remain mapped-open and must reuse source-neutral Status/text motion, Tabs/Slider, ParticipantRow/Checkbox/Avatar, and TextField owners. Reading alone does not reduce unknown. Accounting: **6,261 / 16,125 read**, **9,864 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,262** `Telegram/SourceFiles/ui/widgets/glare_tooltip.cpp@b38e44a0769199bf65b5d584181e3b4f24d716aa`.


### Read-through 6262-6269

Orders **6,262-6,269** are exact-blob read/decomposed across glare Tooltip tracking/timer/teardown and motion, gradient Button glare/ripple/cache behavior, equal-width horizontal Button layout, and LevelMeter value projection. These are presentation responsibilities over existing source-neutral Tooltip/Button/layout/Status owners and must honor reduced-motion and lifecycle teardown. Reading alone does not reduce unknown. Accounting: **6,269 / 16,125 read**, **9,856 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,270** `Telegram/SourceFiles/ui/widgets/marquee_label.cpp@ddb4fac4dc2eb2f4d990b283aad018416ce7d1d2`.


### Read-through 6270

Order **6,270** `Telegram/SourceFiles/ui/widgets/marquee_label.cpp@ddb4fac4dc2eb2f4d990b283aad018416ce7d1d2` is exact-blob read-complete/responsibility-decomposed. It owns overflowing text presentation with delayed wraparound marquee motion; animation is gated by visibility, active window, overflow, reduced-motion policy and selection/menu pause state. It also carries DPR/palette/text/geometry cache invalidation, selectable text with word/paragraph selection and edge-scroll, keyboard/mouse clipboard projection, outside-click/menu teardown, and StaticText accessibility. The responsibility maps only to source-neutral canonical text/Status, ContextMenu/Menu, clipboard, accessibility and reduced-motion owners and remains mapped-open pending shipping composition plus focused lifecycle/a11y evidence. Reading does not reduce unknown. Accounting: **6,270 / 16,125 read**, **9,855 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,271** `Telegram/SourceFiles/ui/widgets/marquee_label.h@b2623a32af81775016bed793ed4c31d66da81a24`.


### Read-through 6271-6292

Orders **6,271-6,292** are exact-blob read/decomposed across the remaining marquee contract, middle-click and selecting autoscroll, responsive MultiSelect, participant expander checks, ephemeral passcode-strength projection, peer identity bubbles, sensitive verification-code normalization/resend-call countdown, Slider width adaptation, SlidingTabs lifecycle/preload fencing and VerticalDrumPicker input/snap semantics. All map to existing source-neutral canonical text/status, ContextMenu, ScrollArea, Picker/SearchField/TextField/ListRow/ParticipantRow/Avatar, Checkbox/Button, Security/Passcode, Authentication/Verification, Slider/Tabs and shared motion/a11y/platform owners. Sensitive candidates remain ephemeral/unlogged; hidden tabs must not preload; timers/animations/cursors/event filters require teardown and stale-callback fencing. Reading alone does not reduce unknown. Accounting: **6,292 / 16,125 read**, **9,833 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,293** `Telegram/SourceFiles/ui/window_palette.cpp@003a1871a2242d25a48f0826938f6d8a1c1d8b29`.


### Read-through 6293-6303

Orders **6,293-6,303** are exact-blob read/decomposed across per-window palette lifetime, Wallet presentation tokens, TON address/transfer-link parsing, animated amount-entry/glyph-diff/IME/selection/reduced-motion semantics, amount painting, and the session-scoped TON-center request adapter. The Wallet API owns validated shifted-DC routing, request IDs, cancellation, 30s timeout settlement, late/duplicate answer fencing, pending accounting and 10s idle-session teardown; it maps only into the canonical Wallet/Payments service owner. The amount field carries the concrete glyph matching/roll/fade/scale and deterministic animation-disabled settlement semantics relevant to the animated-text responsibility, but reading alone does not constitute Fabushi shipping closure. Accounting: **6,303 / 16,125 read**, **9,822 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,304** `Telegram/SourceFiles/wallet/wallet_card_angle.cpp@94800a1532b5ca0baaa878c4d06785f084120ec5`.
Orders **6,304-6,315** are exact-blob read/decomposed across Wallet card pointer-follow motion and DPR-aware gradient caches, a capability-restricted Wallet-to-chat/session Show adapter, collectible metadata/artwork priority lanes with a 10-chain budget, 60-second deadline, generation fencing, Gift-to-web fallback, cancellable Resource loading and preload-window projection, plus the encrypted transfer-comment reveal lifecycle with account/session/app-lock/sleep fencing, bounded identity resolution, key authorization, revision/scope cancellation and plaintext wipe on reset. These remain mapped-open to the existing source-neutral Wallet/Payments/Commerce, Resource/file-transfer/cache, Session/Navigation, Security/Key Protection, Theme, motion and canonical design-system owners; no Telegram-derived parallel Wallet UI/runtime is introduced. Reading alone does not reduce unknown. Accounting: **6,315 / 16,125 read**, **9,810 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,316** `Telegram/SourceFiles/wallet/wallet_content.cpp@6aa42680fee3870ae9969104fdb5872c309e8ef2`.
Orders **6,316-6,319** are exact-blob read/decomposed across the full Wallet content/composition monolith and its public seam plus the durable custody store. The source proves exact-dependency/revision fee quotes, per-press frozen recipient/amount/comment/link expiry, key-ladder and vault-epoch fencing, bounded signing readiness, QuoteExpired retries only before anything leaves the device, SubmissionUnknown operation handoff rather than blind resend, bounded recipient/name/address lookups, secure phrase/restore/import/conflict/backup/rotation lifecycles, polling/teardown and list/scroll-anchor behavior. Custody persistence is v4, separates immutable anchor and current signing keys, retains pending rotation/new-key/awaiting-server reconciliation, validates old/broken data fail-closed and centralizes secret-reference removal. All remain mapped-open to existing source-neutral Wallet/Payments, Security/Key Protection/Custody, storage, Session/Navigation, Resource, Motion and canonical design-system owners; no Telegram-derived parallel owner is introduced. Reading alone does not reduce unknown. Accounting: **6,319 / 16,125 read**, **9,806 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,320** `Telegram/SourceFiles/wallet/wallet_diamond_flight.cpp@9444c0eac18a5c625aad9a2671bbd319974129fb`.
Orders **6,320-6,323** are exact-blob read/decomposed across the send-diamond animation and the Wallet engine/host bridge. Motion semantics include a dynamic target, target-loss cancellation, exact single landing/finish, reduced-motion and power-saving suppression, loop continuity and bounded dirty-region repaint. Engine semantics include strict provider-origin/path enforcement, Accepted/Rejected/Uncertain routed submission, a 40-second ambiguous-broadcast wait that preserves late MTProto settlement, durable send-journal compare-exchange, account/clear-epoch protected-secret fencing, exact failed-call secret cleanup, stale private-result rejection, serial worker ownership and teardown that wakes blocked calls before shutdown while relying on durable journal recovery on restart. These remain mapped-open to existing source-neutral Wallet/Payments, Network, Vault/Key Protection, durable Storage/Journal and Motion owners. Reading alone does not reduce unknown. Accounting: **6,323 / 16,125 read**, **9,802 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,324** `Telegram/SourceFiles/wallet/wallet_fiat.cpp@6e16d23676a8d24ff840c71f88310cb21d540ce4`.
Orders **6,324-6,325** are exact-blob read/decomposed across Wallet fiat/rates formatting: unavailable-rate ellipsis, currency-specific exponent/decimal/grouping, approximate U+2248 output, tiny positive-value significant digit projection, localized currency names, stable top-currency ordering from the active currency plus cloud/system/input-method/phone-country locale signals, and minor-unit nanos. They remain mapped-open to source-neutral canonical Money/Currency formatting and Wallet rates owners.

Orders **6,326-6,339** are exact-blob read/decomposed across Wallet key protection, funding/on-ramp, palette replacement, singleton panel lifecycle, recovery phrase shares, live fiat-rate refresh, and sending effects. Existing-owner-first production work at parent exact HEAD `882728682058ea6235cf625c7e8a33f851fad1a3` extends `native/mahayana-messaging/src/wallet.rs`, `engine.rs`, and `service.rs`: provider-neutral authenticated key wrapping with epoch fencing and zeroized secret bytes; fixed-field recovery seed/XOR shares plus authenticated holder encryption; 5-minute rate refresh / 30-second retry state; funding-request generation fencing and optional base-currency fallback; singleton panel state; deterministic clock/glare math; native/server-authoritative runtime projection. Exact-head evidence: Desktop Chat Parity run `38016864742` completed success; Rust desktop runtime run `38016864748` bound the same SHA, canonical messaging tests succeeded and artifact `11656926928` has digest `sha256:cb27c66d4893776f5105741254f86df84d25a707ff7cd9f256e708c76676b0fa`. These rows intentionally remain mapped-open: no concrete platform key-protection adapter, real funding/rates service adapter, wallet shipping renderer/panel consumer, wallet-specific palette consumer, recovery-share product entrypoint, or sending-effect paint consumer has exact-head production evidence yet. Reading therefore does not reduce global unknown. Accounting: **6,339 / 16,125 read**, **9,786 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,340** `Telegram/SourceFiles/wallet/wallet_session.cpp@7000a36395819deea9ca8ba0cce227ff0e535f32`.


### Read-through 6340-6343

Orders **6,340-6,343** are exact-blob read/responsibility-decomposed across the Wallet Session monolith/public state contract and the real-time Wallet stream. Existing-owner-first production work extends only canonical `native/mahayana-messaging` Wallet/ConnectedApp owners: parked-wallet discovery, panel-scoped legacy-balance lookup, durable connected-app recovery fallback separation, and a source-neutral live-stream epoch/backoff/keepalive/renew/coalesce/history-recheck state machine. The stream still lacks a real Fabushi Host/network service adapter and 6,340-6,341 retain substantial mapped-open custody/history/collectibles/preview/send-recovery/rotation/TonConnect/UI responsibilities, so none of these rows is marked verified and global `unknown` is unchanged. Accounting: **6,343 / 16,125 read**, **9,782 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,344** `Telegram/SourceFiles/wallet/wallet_ton_connect.cpp@60bf68a9…`. Fresh exact-head GitHub Actions are required for the accounting descendant.

### Read-through 6344-6353

Orders **6,344-6,353** are exact-blob read/responsibility-decomposed across the connected-app session/key/connect/disconnect lifecycle, canonical connect/settings UI projection contract, durable claim journal, transaction emulation, and deep-link/start-param boundary. Existing-owner-first production descendants already implement transport message identity, staged claim recovery, source-neutral connect/key/disconnect state, emulation parsing and start-param security/encoding inside canonical native/mahayana-messaging owners. The connect/settings UI projection, real service/navigation composition and complete request-family closure remain mapped-open; no row is promoted to verified and reading does not reduce global unknown. Accounting: **6,353 / 16,125 read**, **9,772 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,354** Telegram/SourceFiles/wallet/wallet_ton_connect_request.cpp@496881cf….

### Read-through 6354-6357

Orders **6,354-6,357** are exact-blob read/responsibility-decomposed across the full connected-app request scheduler/Flow contract and its request-confirmation projection. Existing-owner-first production descendants now cover transport message identity, active/silent/waiting arbitration, durable claim recovery rebuild, recovery polling/submission settlement, flow-completion ownership fencing and Recovery::Offer fallback through canonical native/mahayana-messaging ConnectedApp owners. Remote pending/service composition, closed-session late context, wallet resolve/polling, cryptographic decrypt/sign/encrypt, fee/emulation send integration, Recovery::Answer re-encryption and shipping request UI/a11y/visual evidence remain mapped-open. No row is promoted to verified and reading does not reduce global unknown. Accounting: **6,357 / 16,125 read**, **9,768 unread**, **15,846 unknown**, **0 omitted**. First unread is **6,358** Telegram/SourceFiles/wallet/wallet_transfer_messages.cpp@69cfdacd….



### Revision 9 source closure through 6,361

Orders **6,358-6,361** are exact-blob read/responsibility-decomposed for optimistic outbound transfer-message reconciliation and the durable submitted-transfer store. The canonical Wallet journal already owns handoff/lookup/terminal recovery; this descendant additionally fail-closes noncanonical source, destination and collectible addresses through the existing wallet-address validator. The Message owner still lacks the complete per-draft server-id floor, transport-identity and identical-twin adoption/refusal contract, so 6,358-6,359 remain explicitly mapped-open rather than receiving false implementation credit. Accounting is **6,361/16,125 read; 9,764 unread; 15,846 unknown; 0 omitted**. First unread is 6,362 `Telegram/SourceFiles/wallet/wallet_unlock.cpp@5fbaea9d950772164094e7477e5f786f9a78279f`.


### Revision 9 read-through 6362-6367

Wallet unlock, user-address and vault files are exact-blob read/responsibility-decomposed. Descendant `cb6d56174fb3e3f34c99e7bed3156c3b1e8d523d` closes one security gap: unlocked Wallet projection is runtime-only across serde/restart. Full grant/retention/platform-factor/address-service/multi-account vault lifecycle remains mapped-open. Accounting: 6,367/16,125 read; 9,758 unread; 15,846 unknown; 0 omitted. First unread: 6,368 `Telegram/SourceFiles/webauthn/cable.h@6909eb913dbb41765ebbaeecb2072f130f868655`.
