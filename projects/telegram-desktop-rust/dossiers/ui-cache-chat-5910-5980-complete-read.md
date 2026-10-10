# TDRP Revision 9 — UI cache/chat 5,910–5,980 complete read

Authority: `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / tree `6b616494f3465324e749a04dcd1c9d508657a998`.

All 71 exact blobs were read and responsibility-decomposed. Reading does **not** close global unknown.

The batch covers: rounded-corner cache; attachment preparation/preview/album/downloads; Bot WebView security and Linux shell fencing; chat style/theme/readability; filter tags; send-as identity; per-conversation theme selection; forward privacy options; group-call status/userpics; message bar/bubble projection; more-chats, pinned and pending-request bars; sponsored/recommended disclosure; torn-edge visual primitive; unsupported-content notice.

The Bot WebView slice is security-sensitive: exact origin/source validation, bounded native messages, external-shell token + command namespace separation, permissions/storage/downloads/clipboard/external-link/payment/fullscreen/close lifecycle and Linux shell origin restrictions were read explicitly. Current Fabushi MCP App/Mahayana Mini App and attachment owners were inspected, but exact equivalence remains open until path/symbol + focused same-head evidence is bound.

Accounting after this batch: **5,980/16,125 read; 10,145 unread; 15,846 unknown; 0 omitted**. First unread: **5,981** `Telegram/SourceFiles/ui/color_contrast.cpp@f981e717c24b6f42ada8b4655b8aef50ad617e5d`.

The parent commit `9c969d1` fixed the Revision 9 contract-required inventory narrative tokens after predecessor exact-head validation exposed only that narrative drift. This batch preserves that repair and does not weaken the validator.
