# Temporal & Reliability Test Plan

Status: active / normative  
Plan ID: FQT-TEMP-001  
Updated: 2026-10-07

目标风险：final disappearing、intermediate-as-final、duplicate/reorder、late baseline overwrite、optimistic/authoritative race、cross-conversation leak、completed rollback、reconnect/restart drift。

Timeline: T0 submit -> T1 optimistic -> T2 accepted -> T3 first intermediate -> T4 tool/substate -> T5 terminal -> T6 +2s -> T7 +8s -> T8 switch away/back -> T9 reconnect/reload -> T10 restart。每点记录 IDs、role/kind/status、order key、semantic payload/hash、DOM position、canonical store state。

Scenarios：no-tool；preamble+tool+final；multi-tool；tool failure recovery；long streaming；cancel then next；rapid consecutive turns；switch agent during stream；late baseline；duplicate/out-of-order；disconnect/reconnect；restart after final；attachment/reply+Agent。

terminal 后 +2s/+8s 无 removal/reorder/text rollback/role flip/status rollback。Fault profiles delay/drop/duplicate/reorder/late snapshot/reconnect/timeout/cancel，seed 可重复。Pass 依据 ORA-TURN/ORDER/RECON。
