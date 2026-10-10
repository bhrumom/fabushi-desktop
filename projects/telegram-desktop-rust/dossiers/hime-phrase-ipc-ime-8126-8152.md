# TDRP Revision 9 — Hime phrase / IPC / IME 8,126–8,152

Accepted upstream: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`
Pinned component: `hime-ime/hime@9b3e6f9ab59d1fe4d9de73d3bf0fed7789f921c5`

All 27 files were read in full from the pinned component and their returned blob identities matched the accepted recursive inventory. All rows remain `mapped-open`; reading alone closes no unknown.

## Responsibility groups

- **Settings and phrase lifecycle (8,126–8,133):** phonetic/Tsin settings, explicit import/export/edit/learn commands, clipboard script conversion, bounded phrase DB editing, deterministic text↔binary compile/export, phrase-key mapping and deduplicated learning map into canonical Settings, Resource/persistence and clipboard-action owners. Source GTK/system-shell launchers are replacement-only.
- **Central input lifecycle (8,134–8,135):** trigger/protocol/focus/current-context/reload/shutdown semantics map to the source-neutral platform-input adapter and canonical Composer/editor lifecycle; the Hime XIM daemon does not ship.
- **IPC client/server (8,136–8,145):** per-user endpoint ownership, typed preedit attributes, reconnect/focus restoration, bounded request/reply framing, reset/cursor/flags, per-context dispatch and deterministic disconnect cleanup map to canonical authenticated local IPC. Hime TCP 9999–19999, XOR/password transport and implicit daemon autostart are not accepted substitutes for Fabushi authn/authz/service contracts.
- **Data/locale/build/provider state (8,146–8,152):** keyboard map compilation, Unicode validation, locale-as-default-only, checked command binding, provider build selection and Anthy's Japanese conversion/candidate/preedit state machine map into existing Resource, localization, Menu/Command, Build and native platform-input owners. Locale is explicitly not current input-source truth; Anthy/Hime runtime is not retained.

The renderer's Chromium composition/focus/preedit generation evidence is useful but still does not report OS input method, language, width mode, script variant or candidate state. Those remain mapped-open pending a truthful source-neutral native adapter and exact-head focused tests.

Accounting after this range: **8,152/16,125 read; 7,973 unread; 15,845 unknown; 0 omitted**. First unread: **8,153 `Telegram/ThirdParty/hime::src/modules/chewing-conf.c@93cc1fd31b57e7b45c1fe272ebd945ed7f6632c5`**.
