Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@65e23ba7137ea4129b6bc1b2616104a1f59495ef (root tree 6b616494f3465324e749a04dcd1c9d508657a998). Root non-directory=6,653; recursive non-directory=16,125; read-through=6,041; first unread=6,042 Telegram/SourceFiles/ui/controls/stars_rating.cpp@7b7dce6efc5e62cf48b0821dbfff7da510bd74f2; unread=10,084; unknown=15,846; omitted=0. Reading alone never closes unknown. Fresh descendant exact-head GitHub Actions evidence is required.

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
