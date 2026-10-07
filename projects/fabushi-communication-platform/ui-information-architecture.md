# Fabushi Unified UI Information Architecture & Interaction Contract

Status: active  
Parent: FBCP-001 Revision 5  
Migration contract: TDRP-001 Revision 7  
Updated: 2026-10-07

## 1. Purpose

本合同规定完整 Telegram Desktop 能力迁入 Fabushi 后的 UI/UX 组织原则。上游 UI 是行为与功能来源，不是目标菜单树。最终只有一个 Fabushi product shell；能力按用户任务、作用域和对象关系放置，不能按 Telegram 模块名复制页面或一级入口。

UI 目标同时满足：完整功能、低认知负担、可发现、统一状态、可返回、键盘/可访问、响应式、现有 Bot 无回退。

## 2. Entry classification

每个用户可见 capability 必须恰当地归入一个主要入口类别：

| Entry class | 何时使用 | 例子 |
| --- | --- | --- |
| primary-domain | 跨对象、长期独立浏览的产品领域 | 消息、联系人、Agents、任务、Computer、Automations、插件市场、设置 |
| collection-action | 当前领域列表/筛选/批量操作 | 未读、群组、频道、归档、置顶筛选 |
| creation-action | 创建当前领域对象 | 新聊天、新群、新频道、添加联系人 |
| object-action | 依赖当前对象 | 回复、转发、编辑、删除、通话、查看资料 |
| detail-section | 当前对象详情中的扩展 | Human privacy、Agent tools、Group members |
| capability-surface | 沉浸式专用交互 | Call、Story viewer、media editor、payment flow |
| global-command | 跨领域即时命令 | Universal Search、command palette |
| system-surface | 系统/后台/瞬态 | Notification、tray、permission prompt |

“上游有独立窗口/菜单”不是 primary-domain 的充分条件。若可作为当前对象动作、typed section 或 filter 完成，就不得提升为一级 App。

## 3. Product shell

默认桌面信息架构：

```text
primary nav | collection/search/new | canonical workspace | optional detail/capability panel
```

消息、联系人、插件市场必须一级可达；既有 Agents、任务、Computer、Automations、设置等能力继续可达。Group、Channel、Topic、Call、Story、Poll、Forward、Media 等不能只因为 Telegram 有专门入口而自动成为一级项。

## 4. Messages / Conversation information architecture

一个 Messages domain 包含 Direct Human、Agent、Group、Channel、Hybrid、Topic/Thread。中间列表使用一个 canonical ConversationList，可提供全部、未读、私聊、群组、频道、Agent 以及后续确有价值的 saved/filter view。filter 只改变 query/view，不拥有第二份列表状态或 workspace。

ConversationWorkspace 共享 header、transcript、scroll/pagination、draft/composer、reply/quote、attachment/resource、read/unread、selection、loading/error/empty、a11y/i18n。不同 kind 只能贡献 typed header actions、profile sections、entry renderers、composer actions、panels/overlays 和 policy。

## 5. Unified Conversation creation

Messages 列表顶部提供一个统一 New/+ action。它打开一个 ConversationCreationSurface（或等价 typed flow）：Human direct、Group、Channel、Agent conversation、Hybrid Human + Agent room。底层统一为 typed `CreateConversationIntent`，包含 kind、participants、capabilities、policy、initial metadata。

Group/Channel 的名称、头像、公开性、成员、管理员、发布权限等作为 typed sections 出现，不建立 GroupCreateApp/ChannelCreateApp。成员、管理员、邀请对象选择复用 Participant Search + eligibility/permission policy。Contacts 的“发消息”必须 find-or-create 同一 Direct Conversation 后打开 canonical ConversationWorkspace。

## 6. Contacts / Profile

Contacts 管 Participant identity/contact relation；Messages 管 Conversation creation/list。统一 Participant/Profile framework：Human 提供 presence/username/privacy/shared groups/block 等 sections；Agent 提供 runtime/model/tools/permissions/tasks；Group/Channel 提供 members/roles/media/moderation。Profile 上的发消息、通话、加群、权限等都是 typed object actions，路由到 canonical owner。

## 7. Search architecture

只有一个 canonical Search domain/owner。UI 有三层作用域：

1. Contextual collection search：当前一级领域顶部搜索；Messages 可返回 Conversation/Participant/Message/Resource，Contacts 返回 Participant，Marketplace 返回 Plugin，Tasks 返回 Task。
2. Object-scoped search：在当前 Conversation/Resource/Task 等对象内搜索；Conversation 至少支持 text、sender、date、topic/thread、media/file/link/type 等过滤，点击结果定位 canonical entry。
3. Universal Search：`Cmd/Ctrl+K` 跨域返回 Participant、Conversation、Message、Resource、Agent、Task、Artifact、Plugin、Setting、Command 等 typed results。

Universal Search 是 Search owner 的全局 scope，不是另一个索引产品。

## 8. Search contract

SearchQuery 至少包含 account_scope、domain_scope、optional object_scope、text、filters、sort、cursor、permission_context。SearchResult 至少包含 type、stable_id、title/snippet、provenance、navigation_target。

模块只可注册 SearchProvider/filter/result renderer。所有 consumer，包括联系人搜索、群成员 picker、频道管理员 picker、新建会话成员搜索，都复用 canonical Search/Participant provider。

Search 必须在执行前应用 account、membership、privacy、block、retention、resource/task/artifact visibility。local index 与 remote historical search 均位于同一 owner 后，并统一 ranking、dedupe、cursor/pagination、debounce、cancellation、stale-query fencing 和 provenance。

## 9. Object actions and progressive disclosure

动作按最靠近对象的位置出现：Message 的 reply/quote/forward/edit/delete/reaction/pin；Conversation 的 search/call/info/members/settings；Participant 的 message/call/block/add-to-group；Resource 的 open/save/share/edit；Channel/Group 的 member/role/moderation/publish policy。

低频或危险操作进入 context menu/detail section；高频对象动作可进入 header/toolbar。不能为了“完整”把所有功能常驻主界面造成拥挤。

## 10. Capability surfaces

Call、Story、media editor、payment、Mini App 等可拥有专用 surface，但必须由统一 router/shell 打开和关闭，使用 canonical account/identity/resource/permission，不拥有第二 Conversation list/settings/profile truth，关闭后恢复原 selection、scroll、draft、search query、running task；后台继续运行时在主 shell 有可发现状态/返回入口。

## 11. Navigation, keyboard, accessibility, responsive

- 一级导航有稳定 keyboard/focus order、tooltip/accessibility name、selected/unread/status state。
- `Cmd/Ctrl+K` 统一给 Universal Search；其他快捷键不得与之冲突。
- Modal/overlay/surface 有 focus trap/restore 和 Escape/back 行为；破坏性动作有明确确认。
- 小窗口可以折叠 collection/detail pane，但不能让 primary nav、返回路径或活动通话/任务不可达。
- 保留 draft、scroll、selection、unread、search query、running Agent/task/call 状态。
- 支持屏幕阅读器、键盘、RTL、多语言、中文 IME、深浅色及合理最小尺寸。

## 12. Migration mapping requirements

每个用户可见 upstream responsibility 在 ledger 中至少记录：ui_entry_class、primary_navigation_target、collection_surface、creation_flow、object_action_scope、detail_section、capability_surface、route_contract、composition_root、composition_slot、search_scope/provider/result/filter/permission、responsive_disposition、accessibility_contract、keyboard_contract、state_continuity_contract、ui_acceptance_scenarios。

若多个 upstream 页面在 Fabushi 中合理合并为一个 canonical surface，必须保留行为追溯，不要求保留页面数量。

## 13. Acceptance scenarios

至少覆盖：Messages -> New -> Human/Group/Channel/Agent/Hybrid 统一 creation；Contacts -> Participant -> Message 打开同一 Direct ConversationWorkspace；kind filter 切换不丢 selection/draft/unread truth；Conversation 内搜索定位 canonical message；Contacts/member/admin picker 复用 Participant Search；Universal Search 跨域 typed results 正确导航；无权限/blocked/私密/多账号内容不从搜索泄露；local/remote 搜索无重复且分页/取消/stale query 正确；Call/Story/media editor 返回后上下文完整；keyboard/screen-reader/responsive 与 packaged application 一致。

这些场景必须由 current-head GitHub Actions / packaged acceptance 取证；规范文字本身不算完成。
