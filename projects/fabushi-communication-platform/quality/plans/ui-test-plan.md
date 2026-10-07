# UI Test Plan

Status: active / normative  
Plan ID: FQT-UI-001  
Parent: FDS-001 / FUI-001 / FSP-001 / FVA-001 Revision 2  
Updated: 2026-10-07

UI 有两条独立 pass 轴：functional（route/action/control/focus/state owner）与 presentation（geometry/style/hierarchy/a11y/locale/responsive），两者都 pass。

Component：Button/Input/Menu/Dialog/Search/ListRow/ConversationRow/Avatar/Composer/TranscriptEntry/ProfileSection/Picker/CreationFlow 全 states + keyboard/ARIA/theme/long text。

Screen journeys：ProductShell、Messages、Contacts、Conversation、Profile、Creation、Search、Marketplace、Settings、Tasks/Automations、Computer、Call/Story/Media/MiniApp；测 primary/overflow/back/close/state continuity/permission/error/offline。

Visual 按 FVA matrix；Temporal UI 对 streaming/tool/final、upload/call/sync/reconnect 做 timeline screenshot+DOM+video，特别检查 terminal 在 +2s/+8s、switch-back、reconnect/reload/restart 后仍一致。

A11y：0 serious/critical；keyboard-only；focus-visible/trap/restore；name/role/state；contrast；screen-reader status；reduced motion；IME/RTL/long text。Large data 验证 virtualization/scroll/sticky/selection/ellipsis/lazy loading。
