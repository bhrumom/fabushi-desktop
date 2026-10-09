# Conversation Turn Oracle

Status: active / normative  
Oracle ID: ORA-TURN-001  
Updated: 2026-10-07

A logical turn: one addressed user submission -> accepted/running -> optional intermediate/thinking/tool/approval/task/artifact -> one terminal answer/failure/cancel -> settlement。

- TURN-I01: exactly one terminal outcome。
- TURN-I02: completed turn exactly one terminal semantic answer；intermediate/preamble/tool segment cannot become terminal after settlement。
- TURN-I03: terminal answer cannot disappear/regress under baseline/live/reconnect/reload/restart。
- TURN-I04: completed entry cannot return to streaming/running。
- TURN-I05: addressed conversation/agent owner does not change with visible selection。
- TURN-I06: same-turn relative order remains valid。
- TURN-I07: cancelled turn emits no later terminal content；next turn independent。
- TURN-I08: terminal semantic payload at T5 equals T6/T7/T8/T9/T10 after normalization of non-semantic UI metadata。

Single marker appearing once is insufficient evidence for TURN-I02/I03/I08。
