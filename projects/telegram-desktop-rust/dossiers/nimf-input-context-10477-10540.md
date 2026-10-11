# Nimf build/settings/input-context API — deterministic source read 10,477–10,540

Authority: `hamonikr/nimf@498ec7ffab3ac140c2469638a14451788f03e798`, mounted by accepted `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`.

Exact evidence: Fabushi `84250c295a108a9b5f108b6c474cbeff71dc1097`, run `38128873892`, source-authority job `114438921053`, artifact `11698811298`, digest `sha256:278638d19fd8a347cf20681953a9589ec06413033ff72e7cf9e3b4c6b5e1c209`.

The range defines Nimf's package/build/license boundaries, GSettings-backed settings surface, daemon/signal/socket lifecycle, AppArmor-scoped per-user IPC, system IM activation and its public engine/service/input-context state model. Product semantics include focus, key-event consumption, reset, cursor/surrounding text, preedit lifecycle, commit/delete/retrieve events, candidate pagination/selection and engine/service lifecycle.

Existing Fabushi shipping owner audit:
- `frontend/src/recovered/features/conversation/workspace/rich-text-editor.tsx` owns Chromium compositionstart/update/end, preedit/focus lifecycle, submit suppression from both `event.isComposing` and internal composition state, and old-scope final-input fencing/reconciliation.
- `frontend/src/recovered/features/conversation/workspace/input-composition-state.ts` centralizes submit/escape decisions and fail-safe composition status.
- `source/electron-main/platform-input.ts` probes installed native providers with absolute commands, 750ms timeout, bounded 512-byte values and fail-unknown semantics rather than guessing unsupported language/width/script/candidate state.
- The browser/system IM remains the sole input-method runtime. Nimf daemon/socket/environment/module activation is replacement-only.

No production gap is proven by this tranche, so no new runtime or owner is introduced. All 64 rows remain `mapped-open`, `unknown_closed=false`, `omitted=false`.

Accounting: **10,540 read; 5,585 unread; 15,845 unknown; 0 omitted**. First unread: 10,541 `Telegram/ThirdParty/nimf::docs/html/executables.html@c627d62e1c8e6bf766845a9e7b5116330cb04d12`.
