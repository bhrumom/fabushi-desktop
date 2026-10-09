# WBS 原子任务

| Task | Action | Dependency | Acceptance | Status | Next |
|---|---|---|---|---|---|
| FDS-P0001-T01 | coalesce assistant deltas and final reconciliation | S0 | one body part, no duplicate | in-progress | implement |
| FDS-P0001-T02 | replace dual thinking surfaces with provisional single turn | T01 | one in-progress assistant surface | pending | implement |
| FDS-P0001-T03 | stabilize BotConversationView/composer sizing | none | composer visible | pending | implement |
| FDS-P0001-T04 | parallelize account Bot first hydration | none | Bot membership does not wait on self-hosted sync | pending | implement |
| FDS-P0001-T05 | regression E2E / CI | T01-T04 | required checks pass | pending | run CI |
| FDS-P0001-T06 | protected merge + canonical verification | T05 | main contains verified fixes | pending | merge |
