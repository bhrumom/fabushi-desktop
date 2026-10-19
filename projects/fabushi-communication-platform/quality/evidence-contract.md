# Test Evidence Contract

Status: active / normative  
Evidence ID: FQT-EVID-001  
Updated: 2026-10-07

每份 passing evidence 至少包含 target SHA、upstream/baseline SHA（若相关）、workflow/run/attempt/job/event、actual checkout SHA、suite/case IDs、fixture/environment IDs、start/end、result、artifact name/ID/digest。

Unit/property/contract 保存 machine-readable result 与 failing seed/counterexample；E2E/temporal 保存 runtime trace、semantic/DOM manifest、screenshots，关键 journey 加 video；visual 保存 baseline/actual/diff；a11y 保存 scanner + keyboard/focus；performance 保存 raw samples/time series；fault 保存 injected profile/time/recovery trace。

相关 source/spec/oracle/baseline 改变后旧证据降 historical。截图/视频存在但未关联 case/verdict 不算 pass。独立验收记录 reviewer、reviewed evidence IDs、异常与 verdict。

## Independent review evidence

独立 review evidence 还必须记录 declared change scope、reviewer role/lens、实际 checked paths/surfaces、每个 finding 的 location/severity/concrete failure/fix，以及 reviewer verdict。NOT_APPLICABLE 必须保存与 changed paths/hunks 绑定的 proof；synthesis 必须记录 merged/dropped findings 及理由。Focused re-review 必须逐项记录 previous finding 的 CONFIRMED FIXED / NOT FIXED、检查过的 adjacent callers/consumers、new findings 与最终 verdict。

Cross-review session evidence 还必须持久化 frozen scope identity（base revision、scope mode、explicit in-scope uncommitted state）、每轮 immutable snapshot identity、fresh/blind reviewer set、每份原始 reviewer report 与 synthesis。Runner arbitration ledger 对每个 synthesized finding 记录 `FIX|REJECT|DEFER|COMPROMISE`、rationale、change/no-change、round/synthesis provenance、以及 supersedes 链；`NON_BLOCKING` 也记录但不能单独触发 fix round。

若 finding 构成 reversal，evidence 必须引用先前 decision，记录新增的 concrete failure scenario（若有），并保存新旧 rationale。一次 supersession 后再次试图翻转同一 decision，必须记录 human-decision-needed stop，而不是继续自动循环。每个 round 还要记录 finding/blocking 数量、fix/reject/defer/compromise/reversal-dismissed 统计、focused re-review confirmation，以及最终 stop reason：converged / round-cap / non-converging / human-decision-needed。Round cap 默认值与任何人工 override 也必须入证据。
