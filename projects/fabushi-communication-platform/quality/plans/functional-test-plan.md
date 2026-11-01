# Functional Test Plan

Status: active / normative  
Plan ID: FQT-FUNC-001  
Updated: 2026-10-07

每个 capability 至少设计 positive、boundary、negative、persistence/restart 与 permission/recovery 中适用 cases。

Suites: FUNC-ID(account/identity/contact/privacy)；FUNC-CONV(create/list/group/channel/topic/archive/pin/mute)；FUNC-MSG(send/reply/quote/forward/edit/delete/reaction/read/draft/schedule/expiry/poll)；FUNC-AGENT(final/thinking/tool/approval/task/artifact/handoff/isolation)；FUNC-RES(media/files)；FUNC-SEARCH(context/object/universal/picker/local+remote)；FUNC-CALL；FUNC-STORY/MEDIA；FUNC-PLUGIN/Marketplace/MCP/MiniApp；FUNC-TASK/AUTO/COMPUTER；FUNC-SETTINGS/NOTIFY；FUNC-COMMERCE where applicable。

Domain rules 优先 unit/property，module boundaries contract/integration，用户关键旅程 real packaged E2E。Canned test host response 不能作为真实 Agent/service equivalence 唯一证据。
