# Test Evidence Contract

Status: active / normative  
Evidence ID: FQT-EVID-001  
Updated: 2026-10-07

每份 passing evidence 至少包含 target SHA、upstream/baseline SHA（若相关）、workflow/run/attempt/job/event、actual checkout SHA、suite/case IDs、fixture/environment IDs、start/end、result、artifact name/ID/digest。

Unit/property/contract 保存 machine-readable result 与 failing seed/counterexample；E2E/temporal 保存 runtime trace、semantic/DOM manifest、screenshots，关键 journey 加 video；visual 保存 baseline/actual/diff；a11y 保存 scanner + keyboard/focus；performance 保存 raw samples/time series；fault 保存 injected profile/time/recovery trace。

相关 source/spec/oracle/baseline 改变后旧证据降 historical。截图/视频存在但未关联 case/verdict 不算 pass。独立验收记录 reviewer、reviewed evidence IDs、异常与 verdict。
