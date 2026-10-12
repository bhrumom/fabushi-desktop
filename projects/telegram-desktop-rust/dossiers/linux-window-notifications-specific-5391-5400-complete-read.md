# Linux window, notifications and desktop integration 5391-5400 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`).

Orders 5391-5400 were read completely from exact blobs. Portable responsibilities map to existing source-neutral Fabushi owners; Telegram/Stripe/Linux runtimes or UI roots are not copied.

- **5391 — `Telegram/SourceFiles/platform/linux/main_window_linux.cpp` @ `5c09c6b57acbec40159b84d1bd52380c002a6db6`**: tray/taskbar, unread badge, global menu focus/action enablement and screen identity; canonical owner: application-menu + canonical window/notification.
- **5392 — `Telegram/SourceFiles/platform/linux/main_window_linux.h` @ `4a5ba8f7fa2246f6bac4a9e21cb3d66081ee0bc9`**: typed platform main-window integration contract; canonical owner: Electron window.
- **5393 — `Telegram/SourceFiles/platform/linux/notifications_manager_linux.cpp` @ `a22951015adf6da9aea3e2b3613fefa142552138`**: notification capabilities/actions/reply/activation-token/inhibition/exact-scope cleanup; canonical owner: os-notification-manager + canonical notification/message.
- **5394 — `Telegram/SourceFiles/platform/linux/notifications_manager_linux.h` @ `d2e740750f463a2cfcaacd7d43aa8f5e89f96bb5`**: typed native notification manager boundary; canonical owner: Electron notification.
- **5395 — `Telegram/SourceFiles/platform/linux/org.freedesktop.Notifications.xml` @ `c969a61f9bb819a711f81d2005f5a445aa6fb7bf`**: notification method/signal/capability wire schema; canonical owner: Linux native notification adapter.
- **5396 — `Telegram/SourceFiles/platform/linux/org.freedesktop.login1.Manager.xml` @ `0059cb783fac801becff7cdc4c453756068b6e27`**: system sleep signal wire contract; canonical owner: app lifecycle/platform.
- **5397 — `Telegram/SourceFiles/platform/linux/org.freedesktop.portal.Flatpak.xml` @ `4f2139355af455e91ffffcbfaa1a2b0f15530cfe`**: sandbox spawn/update-monitor contract; canonical owner: packaging/updater/platform sandbox.
- **5398 — `Telegram/SourceFiles/platform/linux/overlay_widget_linux.h` @ `6430f4b200ce892e84b72baa4701c65885c09f49`**: default overlay-window helper selection; canonical owner: canonical window/overlay.
- **5399 — `Telegram/SourceFiles/platform/linux/specific_linux.cpp` @ `8faa6d9051c0b76a09179b67af70d1c0098f2ca9`**: autostart/desktop-service/single-instance/scheme launch/permissions/settings/file move; canonical owner: app lifecycle/deep-link/settings/packaging.
- **5400 — `Telegram/SourceFiles/platform/linux/specific_linux.h` @ `8e1ac22aed6c40a4636d43cc411ce04b84a439b8`**: platform capability/no-op/default boundary; canonical owner: Electron platform adapter.

Orders 5371-5374 exposed a bounded domain gap: invoice-requested customer data lacked fail-closed requiredness/bounds validation before charging. Commit `39da06252e33ef7f8d53061514f8234108ee609e` implements that source-neutral slice plus a focused Rust contract. Exact-head Actions and the wider checkout/reaction/platform responsibilities remain open.

Read-through does not close unknown. `unknown=15841`, `omitted=0`.
