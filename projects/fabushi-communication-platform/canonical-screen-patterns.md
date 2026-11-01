# Fabushi Canonical Screen Patterns

Status: active  
Screen pattern ID: FSP-001  
Revision: 1  
Parent: FBCP-001 Revision 6  
Updated: 2026-10-07

## 1. Screen grammar

所有 screen 使用同一 grammar：primary navigation -> collection/search/new -> canonical workspace -> optional detail/capability panel。Overlay/fullscreen 只用于沉浸交互，关闭后回到原上下文。

每个 screen 定义 route owner、header、collection/workspace/detail slots、primary/secondary actions、search scope、empty/loading/error/offline/permission states、responsive collapse、keyboard focus start/return、visual baseline。

## 2. Product shell

Primary rail 只放稳定 product domains；至少可达 Messages、Contacts、Agents、Tasks、Computer、Automations、Marketplace、Settings/account。Unread/status badge 不改变 rail geometry。

窗口缩小时 collapse 顺序：optional detail -> collection pane -> compact rail labels；不得先隐藏 current object back path、composer 或 active call/task status。

## 3. Messages

Collection header：title + canonical SearchField + New/+。下面是 saved/filter views（全部/未读/私聊/群组/频道/Agent 等）和一个 ConversationList。

点击 Direct/Agent/Group/Channel/Hybrid 都进入同一 ConversationWorkspace。New/+ 打开统一 ConversationCreationFlow。

## 4. ConversationWorkspace

Header：Avatar + title + secondary status + capability-aware actions（search/call/info/more）。Transcript：一个 canonical virtualized feed + typed entry registry。Composer：一个 canonical shell + reply/attachment/voice/Agent actions。Optional detail：Profile/Members/Media/Permissions/Tasks。

Human/Agent/Group/Channel 只改变 typed sections/actions/policy；header/transcript/composer/scroll/pagination/draft/read state 不分叉。

Message action hierarchy：高频 reply/reaction 可直接出现；forward/edit/delete/copy/pin 等按 permission 放 hover/focus toolbar 或 overflow；危险动作进入 confirm。

## 5. Contacts

Collection header：Contacts + Participant Search + Add/Invite。列表用 ParticipantRow。Workspace/Profile 用统一 Participant/Profile pattern。Message action find-or-create Direct Conversation；Call/Add-to-group/Block 是 object actions。

## 6. Profiles

统一 header（avatar/title/status/actions）+ section stack。Human：identity/presence/privacy/shared groups/media。Agent：runtime/model/tools/permissions/tasks。Group/Channel：members/roles/media/moderation/publish policy。Section 不同，根布局和 back behavior 一致。

## 7. Conversation creation

一个 CreationFlow 承接 Direct/Group/Channel/Agent/Hybrid。Step 1 kind（可由入口预选）；Step 2 participant selection；Step 3 kind-specific metadata/policy；Step 4 review/create when required。

Channel public/private/publish/admin、Group member/role 作为 typed sections，不创建独立 app。Cancel/Back 对 dirty state 有明确处理。

## 8. Search

Contextual search 与当前 collection 同位；object-scoped search 在 workspace header 下展开；Universal Search 使用 CommandPalette pattern（`Cmd/Ctrl+K`）。点击结果统一导航到 canonical owner 并定位对象/entry。

No results、search unavailable、remote pending、partial local results、permission-filtered state 都有标准反馈。

## 9. Marketplace

一个 Marketplace route：search/category/list -> plugin detail -> install/update/enable/auth。复用 Plugins/MCP owner。Install/auth progress、permission review、failure/retry 为标准 states。

## 10. Settings

一个 Settings root。Account/security/privacy/notifications/theme/language/update/Computer/Plugins 进入 sections，不创建来源型设置 app。Deep-link 打开 section 后 Back/Close 一致。

## 11. Tasks and Automations

Tasks：collection/list + detail/activity/output。Automations：routine collection + schedule/editor + run history。Conversation 中的 Task/Artifact 跳转到同一 canonical owner。

## 12. Computer

Computer 可有专用 interactive surface，但由统一 shell 路由；连接/重连/permission/update/handoff 有明确 banner/status。Fullscreen 退出恢复原 workspace route。

## 13. Call

Call 是 Conversation/Participant capability。启动点在 object action/header；active call dedicated overlay/surface 显示 identity、connection、mute/camera/device/share/leave。后台/最小化时主 shell 有 return-to-call indicator。

## 14. Story / Media / Rich content

Story viewer、media viewer/editor 可沉浸式；资源仍属 canonical Resource owner。Viewer next/prev/close/save/share；editor 有 dirty/cancel/discard/save。关闭返回 origin conversation/profile/resource。

## 15. Mini Apps

Mini App 由 Plugins/MCP/security Web surface 打开。Header 显示来源/权限/close；loading/error/offline/reload/permission 标准化。Mini App 不拥有第二 account/navigation/settings。

## 16. Standard state matrix

所有核心 screens 至少定义 loading、empty、ready、partial/stale、error、offline/reconnecting、permission denied、disabled/read-only、destructive confirmation、large-data。Async execution 再定义 queued/running/canceling/completed/failed。

## 17. Responsive behavior

wide（>=1200）可同时 collection + workspace + optional detail；standard（900-1199）默认 collection + workspace，detail overlay/panel；compact（<900，若产品允许）优先 rail + current workspace，collection/detail 用 drawer/overlay。具体 minimum window 由 shipping platform contract 决定。

## 18. Data-heavy rules

长标题 ellipsis + tooltip；正文 wrap；超长 code/token 只在 code/resource container 横向滚动；1000+ rows virtualization；sticky section 不遮 focus；selection anchor 不因 virtualization 丢失；attachment grid lazy-load。

## 19. Visual style boundary

Screen pattern 规定结构/slot/priority，不授权复制微信/Telegram/Grok 视觉。颜色、尺寸、字体、icon、motion 来自 FDS-001。新增 screen pattern 必须证明现有 grammar + capability surface 无法表达。

## 20. Acceptance

每个核心 screen current-head packaged evidence 覆盖 state matrix、light/dark、zh-CN/en、responsive、keyboard/focus 和至少一个 large-data case。
