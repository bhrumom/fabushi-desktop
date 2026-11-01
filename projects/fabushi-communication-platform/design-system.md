# Fabushi Desktop Design System

Status: active  
Design system ID: FDS-001  
Revision: 1  
Parent: FBCP-001 Revision 6  
Updated: 2026-10-07

## 1. Purpose

本文件定义 Fabushi Desktop 的统一视觉语言。AI 和开发者不根据单个 Telegram 页面临时设计；功能先进入 canonical product architecture，再使用本 design system 的 semantic tokens、components、patterns 与 states。

目标：成熟桌面通信/Agent 产品的紧凑密度、清晰层级、低视觉噪声、稳定 light/dark、Human/Agent 一致身份语言、可访问、可扩展，并保留现有 Bot 的产品连续性。

## 2. Current exact-head design foundation observed

2026-10-07 UI foundation audit snapshot `7f4af401a6cbd707d076095a1c303a63a05b8a34` 显示：
- runtime theme generator 已有 light/dark palette、4px spacing rhythm、2/4/6/8/12/14/16/18/full radii、20/24/28/32 control heights、11/12/13/14 base type sizes、50/100/150/200/300ms motion durations；
- shared Sand button/icon-button/input/checkbox/switch/tab/menu/overlay primitives 已存在；
- OverlayDialog 已负责 focus restoration、Escape/backdrop dismissal、nested layer ownership 和 scroll lock；
- CommandPalette 已有 keyboard navigation、virtualized results、return-focus 和 messages/files/links/routines/actions 等 provider behavior；
- conversation workspace/list/header/transcript/composer 已有实际 shipping owners。

这些是 migration input，不是来源品牌永久 API。必须在不破坏现有行为的前提下建立 Fabushi-owned semantic layer。

## 3. Visual principles

1. Content first：消息、联系人、任务和产物优先，装饰退后。
2. One accent：主要交互只有一个 accent family；status color 只表达语义。
3. Neutral surfaces：大面积背景以中性层级区分，不依赖重阴影或渐变。
4. Compact desktop density：信息密度高但可扫描，列表与工具栏遵守统一节奏。
5. Progressive disclosure：低频/危险操作隐藏到 context/detail，不把所有功能常驻。
6. Typed difference, visual continuity：Human/Agent/Group/Channel 可以有 badge/section/entry 差异，但仍像同一个 Fabushi。
7. Motion explains change：动画只解释进入、退出、状态变化和进行中反馈。

## 4. Canonical semantic token namespace

新 UI 的公共 token 名称使用 `--fabushi-*`。建议层级：color/text/surface/border/space/radius/control/type/shadow/motion/layer/layout。

legacy `--sand-*` / `--cursor-*` 可在一个明确 compatibility adapter 内映射到上述 semantic tokens；新 Telegram-derived feature CSS/JSX 不直接消费来源命名 token。不得为单个页面新增局部“临时主题”。

## 5. Color baseline

| Role | Light | Dark |
| --- | --- | --- |
| canvas | `#fcfcfc` | `#070707` |
| chrome/subtle | `#f7f7f7` | `#111111` |
| elevated | `#fcfcfc` | `#181818` |
| text primary | `#141414` | `#fcfcfc` |
| text secondary | `#14141499` | `#fcfcfc99` |
| text tertiary | `#14141466` | `#fcfcfc66` |
| border weak | `#1414141a` | `#fcfcfc1a` |
| border default | `#14141426` | `#fcfcfc26` |
| accent | `#1084fe` | `#1084fe` |
| success | `#00c972` | `#00c972` |
| warning | `#ff9800` | `#ff9800` |
| danger action | `#ff263c` | `#ff263c` |
| danger text | `#c21d2e` | `#ff5667` |

该表是 semantic baseline，不授权 feature 使用 raw hex；实现必须引用 Fabushi tokens。品牌 logo/illustration 色彩另由 brand asset authority 管理，不能用历史 token 推断 Fabushi 品牌主色。

## 6. Typography

默认字体：system sans stack，代码/技术值使用 system mono；禁止为了迁移新增未经许可的 bundled font。

| Token | Size / line-height | Weight | Usage |
| --- | --- | --- | --- |
| caption | 11 / 14 | 400-500 | timestamp, tertiary metadata |
| small | 12 / 16 | 400-500 | secondary label, menu hint |
| body | 13 / 18 | 400 | message/list body |
| label | 13 / 18 | 500-600 | controls, row title |
| subtitle | 14 / 22 | 500-600 | header/list section |
| title | 18 / 24 | 600 | screen/dialog title |
| display | 24 / 32 | 600-700 | onboarding/rare hero |

正文不得小于 12px；重要状态不得仅靠字号/颜色区分。CJK/Arabic/Hebrew/emoji fallback 不得造成裁切。

## 7. Spacing, sizing and density

基础 rhythm 为 4px。允许 semantic steps：2, 4, 6, 8, 12, 16, 20, 24, 32, 40, 48。新 feature 不使用无意义 7px/11px/13px 等 magic spacing，除 legacy geometry adapter 或 design exception。

Control heights：24(xs), 28(sm), 32(md default), 36(lg)。IconButton 使用同一方形高度。Conversation/participant rows 默认 56-60px；紧凑 rail row 40-44px；header band 约 52px。偏离来自 canonical screen pattern。

## 8. Shape and elevation

Radius：2(xs), 4(sm), 6(md), 8(lg), 12(xl), 16(2xl), full。普通 controls 6px；list selection/card 8px；popover/dialog 12-16px。

Elevation 只用于 popover/menu/dialog/floating control，不用 shadow 代替 layout hierarchy。基础页面分区优先 surface/border。

## 9. Desktop layout tokens

- primary navigation rail：52px canonical width；仅图标时保持 tooltip/a11y label。
- collection/list pane：288px default，可调整 240-360px。
- optional detail pane：320px default，可调整 280-400px。
- chat/header band：约 52px；composer collapsed minimum 48px。
- transcript prose/message content：常规阅读宽度优先，bubble/card 不超过约 640-720px 或 container 合理比例。

Responsive collapse 由 FSP-001 规定；页面不得私自改变整体 grid。

## 10. Iconography

通用 UI 只使用一个 canonical icon registry/family。标准尺寸 14/16/18/20/24px；同一层级线宽一致。一级导航默认 20px。Emoji 不是通用 action icon。外部产品品牌图标不得成为 Fabushi 通用 UI asset。

Icon-only control 必须有 accessible name + tooltip；危险动作不能只靠红色图标表达。

## 11. Avatar and identity

一个 Avatar primitive 服务 Human、Agent、Group、Channel。推荐尺寸 24/28/32/40/48/64。图片缺失使用 deterministic Fabushi fallback；类型差异通过 typed badge/status/stack 表达，不创建多套 avatar system。

Presence、verified/admin/Agent-running 等 badge 有独立语义和可访问文本；不能用头像颜色作为唯一状态。

## 12. Motion and feedback

Canonical durations：instant 50ms、fast 100ms、normal 150ms、slow 200ms、slower 300ms。一般 hover/selection 100ms；panel/menu 150-200ms；大型 route/overlay 200-300ms。超过 300ms 需要理由。

Agent thinking、upload/download、call connecting、sync 可有持续反馈，但必须低干扰、有文字/状态等价，并在 `prefers-reduced-motion` 下静态化或最小化。

## 13. Content design

用短、直接、面向用户动作的文案。Primary action 用动词；危险操作明确对象和结果；失败说明“发生什么 + 用户可做什么”；空状态给下一动作。中英术语在 copy registry 中唯一化。

时间、文件大小、成员数、状态使用统一 formatter；模块不得各自拼相对时间/单位。

## 14. State palette

每个 interactive component 至少定义 default/hover/pressed/focus/selected/disabled/loading/error；异步 surface 还定义 empty/offline/permission-denied/stale/reconnecting。Focus ring 永远可见。

## 15. Theme and accessibility

Light/dark 是同一 semantic token graph，不是两份 CSS。正常文本对背景满足 WCAG AA 4.5:1；大文本和非文本 UI control 边界至少 3:1。Color 不是唯一状态编码。

Theme owner 唯一；切换 theme 不丢 route/draft/scroll。

## 16. Implementation policy

Feature code 只组合 semantic tokens + canonical components/pattern slots。禁止 inline style magic values 形成新的视觉规则；确有动态 geometry 时由 component API/semantic CSS variable 提供。Data visualization、media intrinsic content、third-party web surface、legacy evidence geometry 是受审查例外。

新增 token/component/pattern 必须说明：现有为什么不够、语义、所有 states、a11y、light/dark、responsive、reuse consumers、migration plan、tests、visual baseline。

## 17. Completion

本 Design System accepted 需要：Fabushi semantic layer 已进入 shipping product；新 migrated UI 只使用该公共合同；核心 legacy surfaces 通过 adapter 收口；visual acceptance 全绿。本文当前是规范，不宣称实现完成。
