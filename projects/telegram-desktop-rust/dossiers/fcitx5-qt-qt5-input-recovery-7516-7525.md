# fcitx5-qt Qt5 platform-input compatibility recovery — 7,516–7,525

Authority: `fcitx/fcitx5-qt@0285a5d18367d8f3af80dab7e4819d5555982340`, mounted by `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`.

All ten entries are exact-blob read and responsibility-decomposed. Fcitx4/DBus/socket transport is replacement-only; no source-shaped runtime is retained.

Applicable semantics map to existing Fabushi platform/input/network-recovery owners:
- exact service-owner identity and second-registration fencing before binding;
- exactly-one async input-context creation with pending-call cleanup and Destroy-on-teardown;
- focus/reset/key/cursor/surrounding-text event lifecycle;
- fallback endpoint discovery with strict socket payload shape and daemon/process liveness validation;
- unique connection identity, owner/filesystem watching, stale-connection cleanup, and immediate rediscovery/reconnect after disconnect to avoid races.

Reading closes no unknown. Accounting: 7,525/16,125 read; 8,600 unread; 15,845 unknown; 0 omitted. Next: 7,526 `Telegram/ThirdParty/fcitx5-qt::qt5/platforminputcontext/fcitx5.json.in`.
