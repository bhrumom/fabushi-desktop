# Nimf server/client/engine lifecycle — deterministic read 10,605–10,668

## Authority

- Accepted upstream: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`, root tree `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6`.
- Nested source: `hamonikr/nimf@498ec7ffab3ac140c2469638a14451788f03e798`.
- Exact source evidence: Fabushi `188e67c679f86628c00bbd7ec0a116e4d5a63fab`, run `38130075791`, source-authority job `114442445077`, artifact `11698588655`, digest `sha256:13f309ef6dda0b2fe85de73e4a4da8a3c3c2058efdb4782c10a021eb67b2499e`.
- Exact-head shipping regression evidence: Desktop Chat Parity run `38130075774`, Renderer job `114442391039` success, Focused Electron chat E2E job `114442391194` success.
- Range: orders 10,605–10,668 inclusive. Every row is exact blob-identified and remains `mapped-open`.

## Responsibility decomposition

`nimf-message.c` supplies bounded typed request/reply framing, exact input-context/message correlation and stale-dispatch clearing before synchronous settlement. libnimf module/server/service sources add dynamic load/unload, startup failure removal, focus/last-focused ownership, reset/filter/cursor/use-preedit state, surrounding-text retrieval/deletion, engine changes and cleanup. GTK and Qt adapters demonstrate commit/preedit/focus/cursor/surrounding replacement and Unicode-index behavior on real editable surfaces. Anthy and Libhangul add language conversion and candidate lifecycle. The m17n build inputs and language SVGs establish catalogue/resource responsibilities, not a separate product runtime.

## Existing-owner evidence

- `frontend/src/recovered/features/conversation/workspace/rich-text-editor.tsx`: shipping composition start/update/end, `event.isComposing` submit fencing, focus and reconciliation behavior.
- `frontend/src/recovered/features/conversation/workspace/input-composition-state.ts`: canonical composition scope and stale-tail settlement fencing.
- `source/electron-main/platform-input.ts`: bounded native input-source observation with unsupported fields kept typed `unknown`.
- Current exact-head Focused Electron E2E `114442391194` is successful and covers live composition status, non-submit during active IME and cross-conversation stale composition-tail isolation.

Chromium and the operating system remain the native language-engine, candidate/preedit and surrounding-text integration owners. This range therefore does **not** justify a Nimf daemon, local socket protocol, GTK/Qt client, Anthy/Libhangul engine, second input-context store, second Settings truth or Fabushi-owned candidate UI. Build/settings/resource inputs remain mapped to existing Build/Quality, Packaging, Settings, Localization and Design System owners.

## Closure status

All 64 rows stay `mapped-open`, `unknown_closed=false`, `omitted=false`. Deterministic reading does not claim production/release closure. After this tranche: **10,668/16,125 read; 5,457 unread; 15,845 unknown; 0 omitted**.

Next exact entry: order 10,669 `Telegram/ThirdParty/nimf::modules/engines/nimf-m17n/icons/scalable/nimf-m17n-hi.svg@ea2270ac3575895b456c1c808639d83bead23295`.
