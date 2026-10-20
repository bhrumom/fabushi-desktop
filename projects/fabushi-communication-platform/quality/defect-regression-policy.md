# Defect & Regression Policy

Status: active / normative  
Policy ID: FQT-DEF-001  
Updated: 2026-10-07

## Severity

P0: data/security/privacy/cross-account/corruption/release unusable。P1: core messaging/Agent/final/order/recovery/major UI path broken。P2 bounded degradation。P3 cosmetic/minor。

## Record

ID、SHA/environment、steps、expected/actual、frequency、screens/video/logs、affected requirement/oracle、severity、root cause、fix SHA、regression IDs、evidence、reviewer。

## Escaped defect

人工/生产/后阶段发现但 earlier automation 未发现时，必须做 gap analysis：缺哪个 oracle/时序/fixture/fault/visual review？除最小 regression 外，在能更早抓住问题的最低合理层补 test。例如 final appeared then disappeared 至少补 turn invariant + reconciliation/property + packaged temporal regression。

## Closure/flaky

reproduce->fix->targeted regression->affected regression->current-head evidence 才 close。Flaky 是 defect；critical gate flaky 即 block，禁止 rerun-until-green。
