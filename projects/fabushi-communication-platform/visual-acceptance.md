# Fabushi UI Visual & Interaction Acceptance Contract

Status: active  
Acceptance contract ID: FVA-001  
Revision: 1  
Parent: FBCP-001 Revision 6  
Execution: GitHub Actions only  
Updated: 2026-10-07

## 1. Goal

确保“功能迁进去了”不等于“UI 拼起来了”。UI acceptance 必须证明同一 design system、canonical components、screen patterns、state continuity、a11y、locale 和 responsive behavior 在正式 shipping composition 中成立。

## 2. Evidence identity

每次 visual acceptance 绑定 target SHA、workflow/run/attempt/job、platform/runtime、viewport、theme、locale、fixture、artifact ID/digest。HEAD 或 relevant token/component/screen baseline 改变后旧截图只作历史证据。

## 3. Static design-system gate

- migrated feature code 只使用允许的 Fabushi semantic tokens/canonical components；
- raw color/font/radius/shadow/motion magic values 为 0，除登记 exception；
- legacy `sand-*`/`cursor-*` direct consumption 只存在 compatibility/evidence adapter allowlist；
- canonical root/component owner 不重复；
- protected component internals 不被 feature deep-selector 覆盖；
- icons 来自 canonical registry，用户可见外部品牌 asset 无非法残留。

## 4. Fixed screenshot matrix

- Viewports: 1440x900, 1280x800, 1024x768，另取产品支持的最小窗口。
- Theme: light + dark。
- Locale: en + zh-CN；至少一个 long-text/RTL stress fixture。
- State: ready + loading/empty/error/offline/permission/data-heavy 中适用状态。
- Key surfaces: ProductShell, Messages, Human/Agent/Group/Channel conversation, Contacts/Profile, CreationFlow, Universal Search, Marketplace, Settings, Tasks/Automations, Computer, Call/Story/Media/MiniApp when implemented。

动态时间、随机 avatar、network timestamp 等须 fixture 化，不能模糊忽略整个区域。

## 5. Visual regression policy

Approved baseline 必须来自已验收 design contract，不能把失败状态重新录成 baseline。默认 perceptual/pixel diff tolerance 可从单 surface 0.5% changed pixels 起步，但 protected geometry、missing control、overlap、text clipping、unexpected brand/style change 即使低于阈值也失败。具体 threshold 在 workflow 中版本化。

故意视觉变更需要 design rationale、affected screenshots、before/after artifact、reviewer、baseline update commit。不能自动 accept all snapshots。

## 6. Structural assertions

截图之外断言：唯一 primary nav、唯一 ConversationWorkspace、唯一 Composer、唯一 Search/CommandPalette root、正确 route/slot、没有重复 hidden app tree。Visual green 不能掩盖 architecture duplication。

## 7. Layout quality gates

0 unintended horizontal page scroll；0 overlapping interactive controls；0 clipped primary action；0 text outside container；sticky header/footer 不遮 focused item；dialog/popover 在 viewport 内；scrollbar 不覆盖 content；drag regions 不吞 buttons；window controls 不与 app actions 冲突。

长 CJK、长英文无空格 token、emoji、mixed RTL、long file name、99+ unread、large participant count 有 stress fixture。

## 8. Accessibility gates

- axe/等价扫描 0 serious/critical violations；
- normal text contrast >= 4.5:1，大文本和非文本 control boundary >= 3:1；
- keyboard-only 完成核心 flow；
- focus-visible 始终可见且顺序合理；
- dialog/menu/popover focus trap/return 正确；
- screen reader name/role/state 正确；
- status/async error 使用合理 live region；
- color 不是唯一信息编码。

## 9. Keyboard scenarios

至少覆盖 rail navigation、Messages/Contacts list、open conversation、composer send/reply/attachment、message action menu、Conversation search、Cmd/Ctrl+K Universal Search、CreationFlow、dialog confirm/cancel、Settings、Escape/back、focus restoration。

## 10. Motion and reduced-motion

默认 motion 使用 FDS-001 durations/easing。`prefers-reduced-motion: reduce` 下非必要动画为 0 或最小状态变化；Agent thinking/upload/call 等持续状态仍有静态可读等价。Animation 不得阻止输入、改变阅读顺序或造成 layout shift。

## 11. State continuity

切换 Conversation/primary domain、打开关闭 detail/overlay、进入退出 Search/Call/Media/Settings 后，验证 draft、scroll、selection、unread anchor、search query、running Agent/task/call、upload state。无理由丢失即失败。

## 12. Large-data / performance visual acceptance

至少用 1000+ conversation/search rows、长历史 transcript、100+ members、多个附件和连续 Agent tool/task entries 验证 virtualization、scroll、sticky、ellipsis、skeleton、progressive loading；不得出现跳动、错位、重复 row 或空白断层。

## 13. Content and brand acceptance

扫描用户可见 Telegram/Grok/Gok/其他来源品牌文案与 asset；合法 provenance 不属于产品 UI。检查术语、按钮动词、错误/空状态、危险操作确认、中英文一致。Icon style/size、Avatar fallback/status badge 必须匹配 FDS-001。

## 14. Artifact bundle

artifact 至少包含 manifest.json（SHA/run/viewport/theme/locale/state）、screenshots、diff images、a11y report、keyboard scenario result、structural assertions、design-token/component lint result、baseline IDs/digests。Packaged acceptance 还包含安装包 provenance。

## 15. Fail-closed rules

截图缺失、只测 light 或单 locale、visual diff 未解释、a11y serious/critical、核心按钮不可见/不可键盘访问、responsive 隐藏功能且无替代路径、feature-local second visual system、未经批准的新 primitive/pattern、复用旧 HEAD screenshot，任一发生都不能验收 UI 完成。

## 16. Current status

本文件定义 gate，不代表仓库已经实现 screenshot harness、token lint 或 visual baseline。后续实现只能在 GitHub Actions 取证。
