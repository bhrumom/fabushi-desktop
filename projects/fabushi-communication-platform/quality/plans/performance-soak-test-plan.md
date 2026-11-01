# Performance & Soak Test Plan

Status: active / normative  
Plan ID: FQT-PERF-001  
Updated: 2026-10-07

Workloads：1000+ conversations/search rows、large transcript、100+ members、large attachments、多 Agent/tool/task activity、持续 reconnect/reopen/search。

Metrics：submit-to-local-paint、first-text、terminal、scroll/frame responsiveness、CPU、memory、DOM/node count、disk/db growth、idle resource、reconnect/reload latency。

Soak 检测 unbounded growth、duplicate listeners/subscriptions、stale timers/tasks、transcript duplication、visual drift。预算由 workflow/spec version 化；未测量不能声称满足。Artifacts 保留 raw samples、percentiles、resource time series。
