# TDRP Revision 9 — orders 5,849–5,859 complete read

Authority: `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`.

All eleven exact blobs were read and responsibility-decomposed. Reading does **not** close global unknown; `unknown=15,846` and `omitted=0` remain unchanged.

## 5,849–5,854 — professional evidence responsibilities

- **5,849–5,850 `tests/test_main.*`**: deterministic Qt UI-test application bootstrap, nested event-loop accounting, postponed-call ordering, UpdateRequest/lifetime fencing, integration hooks, scale/cache/touch fixtures. This is evidence infrastructure only.
- **5,851 `tests/test_text.cpp`**: executable text-layout evidence for formula-like inline objects, replacement/export semantics, selection, custom-emoji distinction, geometry/baselines/hit testing and offscreen paint. It does not create a second Text owner.
- **5,852 `tests/test_update_unpack.cpp`**: fail-closed decompression/extraction evidence, exact payload/permission checks, conflicting-path rejection, inner-version fencing and outside-sentinel preservation.
- **5,853 `tests/test_update_verify.cpp`**: root/manifest/signature/channel/target/version trust-chain evidence including AND-of-OR stable signatures, canary separation and impostor/expired/revoked key refusal.
- **5,854 `tests/test_updater_win.cpp`**: Windows parent identity and PID-reuse fencing, exact creation-time binding, graceful/forced termination sequencing, denied-termination behavior and self/absent-process safety.

These rows are `mapped-open-test-harness`: GitHub Actions must supply equivalent source-neutral executable evidence over the unique shipping owners.

## 5,855–5,859 — Tray production responsibilities

- **5,855–5,856 `tray.cpp/.h`**: unread-aware tray tooltip/icon projection, capability-driven icon lifecycle, menu refresh, single-vs-double-click suppression, active-state show/hide routing, quit, native tray message support, and reversible notification/sound/flash preference mutation with change publication.
- **5,857–5,858 `tray_accounts_menu.cpp/.h`**: lifetime-bound multi-account menu composition, canonical account ordering, live-session filtering, weak target fencing, avatar/identity projection and safe account activation.
- **5,859 `tray_accounts_menu_dummy.cpp`**: explicit unsupported-platform adapter; absence of a native account tray menu is a platform disposition, not evidence that the responsibility is globally complete.

A current-main search found no direct reusable Tray-named production owner. These rows therefore remain **mapped-open** against the source-neutral Fabushi Desktop shell/platform notification-settings/account-session activation responsibilities until exact owner path/symbol, shipping composition, platform behavior, tests and packaged temporal evidence are established. No Telegram-named parallel shell/runtime is authorized.

## Accounting after this batch

- recursive total: **16,125**
- read-through: **5,859**
- unread: **10,266**
- unknown: **15,846**
- omitted: **0**
- first unread: **5,860** `Telegram/SourceFiles/ui/boxes/about_cocoon_box.cpp@4aab8e79a2fbbb0cf7feffffcb6cacf246e0275b`

