# Source intake

2026-09-20 user requirement: fix the observed “你好” flow where reply latency is high, two assistant replies appear, streaming temporarily renders one Chinese character per line, the 全球法布施 Bot appears late, and the composer can disappear. Match the supplied Grok-like video behavior.

Reference implementation requested by user: `https://github.com/RongleCat/grok-app`.

Reference patterns verified in that repository: keep one current-turn assistant row, bind late tokens back to that row, reconcile live segments with final content, flush/coalesce high-frequency stream events before settle, and isolate live state updates.
