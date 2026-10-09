# Fabushi Canonical UI Component Contract

Status: active  
Component contract ID: FUI-001  
Revision: 1  
Parent: FBCP-001 Revision 6 / FDS-001 Revision 1  
Updated: 2026-10-07

## 1. Rule

AI/开发者先查 canonical component registry，再写 UI。能由现有 primitive/pattern + props/slots 表达的能力，不得复制 feature-local 版本。共享 store 不足以证明复用；DOM behavior、states、focus、keyboard、a11y 和 visual tokens 也必须复用。

现有 SandButton/SandIconButton/InputGroup/OverlayDialog 等可作为迁移实现基础，但目标公共 API 是 Fabushi-owned semantic component contract；compatibility wrapper 可以保留旧实现内部细节。

## 2. Component layers

1. Primitive：Button/Input/IconButton/Tooltip/Menu/Dialog/Popover/Tabs/Switch。
2. Pattern：SearchField/ListRow/Picker/EmptyState/Toast/Toolbar/DetailPanel。
3. Product pattern：ConversationRow/TranscriptEntry/Composer/ProfileSection/CreationFlow/SearchResults。
4. Capability extension：PollEntry/CallControls/StoryCard/AgentThinkingEntry/PaymentReceipt 等 typed leaf。

Capability extension 不得重新拥有父级 layout/navigation/state。

## 3. Canonical registry

| Component / pattern | Required contract |
| --- | --- |
| Button | primary/secondary/ghost/danger, sm/md/lg, loading, disabled, icon slots, focus |
| IconButton | icon size, tooltip, accessible name, selected/pressed, danger |
| TextField/TextArea | label/help/error, leading/trailing slot, keyboard, IME, disabled/read-only |
| SearchField | query, clear, scope/filter trigger, loading, result count |
| Checkbox/Radio/Switch | checked/indeterminate, label hit target, keyboard, disabled |
| Tabs/Segmented | selected id, arrow-key navigation, overflow, persistence owner |
| Menu/ContextMenu | menu semantics, nested layer, destructive grouping, keyboard/typeahead |
| Tooltip | pointer + keyboard/focus discoverability；never sole critical content |
| Popover | anchor, collision, dismissal, focus return, nested layer |
| Dialog/AlertDialog | title/description, initial focus, trap/restore, Escape policy, confirm |
| Avatar | typed participant, fallback, image error, badge/status slots |
| Badge/Status | semantic tone, icon/text, not color-only |
| ListRow | leading/body/trailing, selected/hover/focus, compact/default density |
| ConversationRow | avatar/title/preview/time/unread/mute/pin/status；all kinds |
| ParticipantRow | avatar/name/secondary/presence/role/selection |
| ResultRow | typed search result + provenance + navigation target |
| Toolbar | primary vs overflow actions, responsive collapse, keyboard order |
| EmptyState | reason + one clear next action |
| LoadingState | skeleton/spinner based on known geometry；avoid layout jump |
| ErrorState | user-safe error + retry/recovery |
| Toast/Banner | transient/persistent severity, dedupe, action, live announcement |
| Composer | text/attachments/reply/voice/Agent actions/send state；one shell |
| TranscriptEntry | typed renderer registry；shared actions/provenance/status shell |
| ProfileSection | title/actions/content；typed Human/Agent/Group/Channel sections |
| DetailPanel | header/back/close/scroll, width, focus/return |
| Picker | search + eligibility + selection + confirm |
| CreationFlow | steps, validation, back/cancel, dirty protection, submit lifecycle |
| CommandPalette/UniversalSearch | query/providers/results/keyboard/virtualization/focus return |

## 4. Styling ownership

Canonical component owns geometry, states and token usage。Feature 控制 content、typed slots 与 domain policy。Feature CSS 不可用 deep selector 重写 protected internals，除非 component 暴露 documented semantic slot/variant。

禁止 `TelegramButton`、`ChannelDialogButton`、`AgentSearchInput`、`GroupMenuItem` 等可由 canonical primitive 表达的局部复制。

## 5. Interaction states

每个 interactive component 实现 default、hover、pressed、keyboard focus-visible、selected/checked（如适用）、disabled、busy/loading（如适用）。Async action 对成功/失败/重试有明确反馈。

Hover 才显示的功能必须同时能由 focus 或 accessible menu trigger 到达。

## 6. Accessibility

Native semantic element first。Icon-only button 有 accessible name。Dialog restore focus。Menu/tab/listbox 使用正确 keyboard model。Error/async status 使用合理 live region，不重复轰炸。

## 7. Forms and destructive actions

Label 可见或 programmatically associated；placeholder 不是唯一 label。Validation 靠近 field。危险操作使用 danger variant 并明确对象；不可逆操作按风险确认。

## 8. Lists and virtualization

大 Conversation/Search/Member/Task list 支持 virtualization 且不破坏 aria position、keyboard selection、sticky section、scroll restoration、unread anchor 或 result navigation。Virtualization 不是第二 list owner。

## 9. Extension gate

新增 component/pattern 仅当现有 component/slot 无法表达语义；需要 API/states/a11y/theme/responsive contract、reuse target、migration plan 和 visual acceptance。

## 10. Acceptance

Component gate 扫 production composition：duplicate primitive family、direct feature styling of protected internals、missing states/a11y、unregistered variants 都 fail。migrated capability 在落到 canonical contract 前不得 verified。
