# node.invoke progress heartbeat lifecycle — 2026-10-30

`NodeInvokeProgressWriter` now gates both `heartbeat_due` and
`queue_heartbeat` on the explicit `start_heartbeats()` lifecycle state.
Elapsed idle time before heartbeat scheduling is enabled cannot emit a
heartbeat; `stop()` remains terminal and disables subsequent scheduling.

The change stays inside the existing progress owner and does not add a
transport, timer, or invocation registry. Evidence is limited to source
inspection, a focused negative contract, and `git diff --check`; no local
OCRM test/build, Actions, or live node transport acceptance was performed.
