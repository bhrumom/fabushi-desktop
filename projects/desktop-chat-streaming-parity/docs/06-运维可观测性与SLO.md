# 运维可观测性与SLO

User-facing responsiveness targets for this project:
- optimistic user row: within one renderer turn after submit;
- in-progress assistant surface: painted locally before Host acceptance;
- no intentional polling delay before account Bot hydration.

Existing runtime logs/events remain authoritative. New persistent telemetry is N/A for this repair; revisit if CI cannot isolate latency regressions. Owner: Fabushi Desktop.
