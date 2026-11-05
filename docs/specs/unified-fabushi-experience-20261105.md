# Fabushi 统一产品体验重设计 — UFX-001

Status: active design / implementation pending  
Owner: Codex 产品设计与独立验收；Antigravity CLI /teamwork 实施  
Date: 2026-11-05  
Parent: FBCP-001 / TDRP-001 / OCRM-001；不替代其全部能力及验收范围

## 1. 请求与问题

用户明确要求：重新设计 Telegram 与 OpenClaw 在 Fabushi 内的完整融合；学习两份源码，保留所有功能，统一自然的 UI/UX；设计完成后由 Antigravity CLI `/teamwork` 推进，Codex 负责验收。

截图观察：两个聊天输入区、常驻连接配置、反复失败消息、内部 executor subagent 暴露为普通会话、横向空间浪费、语言混杂。

源码基线 `7f6548d42`（release/fabushi-next），不是远端主线。2026-11-05 远端 main=`5528062177e3865eef60a073a066cca144f9e3e8`。工作区独立为 `design/unified-fabushi-20261105`；不得覆盖其他未提交工作，不能假定本分支已经发布或安装。

已直接确认：`frontend/src/production/openclaw/OpenClawAskCopilotPanel.tsx` 使用本地数组与 setTimeout 生成执行回复、默认在线且硬编码“18 个连接器全量就绪”；`ProductionRenderer.tsx` 在没有真实执行 props 的情况下装配它。该面板是产品真实性缺陷，不是可验收能力。其名称、颜色与独立状态也造成第二套产品语义。

## 2. 目标与范围

UFX-R01：一个 Fabushi 身份体系、会话空间、消息记录、输入器、任务状态、搜索和配置入口。支持人与人、人与助手、群组/频道/话题、混合协作。

UFX-R02：全部锁定源范围的产品功能继续纳入 FBCP/TDRP/OCRM；不得用首批 UI 范围替代总目标。源能力映射缺失算未完成。开发工具、平台专属实现与协议等按现有来源处置合同处理，不凭目录名整体删除业务。

UFX-R03：不复制两个品牌的应用壳，不新增 TelegramProvider/第二 Conversation 或后台真相。Fabushi 原生通信与 OpenClaw 外部渠道连接是不同能力：前者用 Fabushi 身份/协议；后者保留受权的 Telegram 等渠道互操作。

不在本设计中授权：更换来源锁定版本、删除用户数据、读取/展示凭据、绕过权限、把未验收候选直接覆盖安装应用。实施可在任务分支编辑/提交、按现有策略推送并创建 PR/执行必要 Actions；正式发布须满足既有独立验收门禁。

## 3. 信息架构

一个壳：一级导航 | 当前领域列表 | 对象工作区 | 按需详情。默认详情关闭。

| 一级入口 | 默认内容 | 次级入口与完整能力 |
|---|---|---|
| 会话 | 全部、未读、自定义文件夹；新建 | 人、助手、群、频道、话题；归档/置顶/草稿/搜索；动态在会话集合入口，非永久第二排功能按钮 |
| 任务 | 需要我处理、进行中、已完成 | 定时与自动化、运行历史、失败/取消、产物、子任务树、执行设备、用量；每项可回原会话 |
| 联系人 | 人与助手分类 | 联系人、邀请、群资料、助手配置/记忆/工具；发消息回同一会话 |
| 设备 | 已授权设备与可用性 | 远程操作、文件、浏览器、节点配对、执行环境、设备权限 |
| 扩展 | 已安装与发现 | 插件市场、MCP、技能、应用/工作台；模型/账号/渠道在设置 |
| 设置（底部） | 账号、外观、通知、隐私 | 模型与提供方、外部渠道、语音、连接与诊断、存储/导出/更新 |

“四个核心场景”不表示删除插件市场或原有 Agents/Automations。扩展保持一级可达；助手在联系人有直接分类与旧路由兼容；自动化在任务有直接标签与旧深链兼容。一级菜单必须提供文字或 tooltip/a11y 名称，不只剩未知图标。

UFX-R04：新建统一入口选择联系人/助手/群组/频道；搜索统一 Cmd/Ctrl+K，各领域仅传递 scope。旧入口与深链映射到新路由，保留对象 ID 与返回栈。

## 4. 核心交互与屏幕合同

### 4.1 会话

顶部显示头像、名称、对象身份和必要状态；右侧搜索、通话（支持时）、详情、更多。移除常驻 workspace 面包屑与含义不清的“布局”按钮。分享等低频动作移至对象菜单，不改变权限语义。

消息列表：紧凑时间分组；普通消息短气泡；助手长回答/代码/表格按内容阅读宽度展现；工具噪音收进任务详情，不把已有历史文本按字符串匹配删除或折叠。

一个输入器：附件、输入、语音、发送。仅助手/混合会话显示可选模型、推理强度；人类会话不展示模型。保留回复、编辑、计划发送、静默发送、媒体预览、上传状态、录音、命令/提及/贴纸等完整 typed actions；扩展附件菜单分组呈现。中文 IME Enter 不误发。

UFX-R05：内部子代理仅在已有 parent/task/type 元数据证明是内部执行对象时，默认归入父任务树；不得按显示名“executor”过滤，也不删存储。用户创建的同名助手正常显示；被明确固定/独立打开的执行对象可查看历史与返回父任务。未知来源保留可达并标明待归类。

### 4.2 任务与真实状态

UFX-R06：接受请求后在原会话显示一张稳定 taskId 卡。卡片包含任务目标、真实状态、最近有效进展、必要阻塞原因、一个主要下一动作；详情展示步骤、子代理、工具、审批、设备与产物。

状态投影：待开始 → 执行中 → 等待授权/等待连接 → 执行中 → 完成/失败/已取消。重试必须是显式 attempt，不回写旧完成结果。不因 UI timeout 判定后台失败/成功；断线显示未知/等待连接并恢复订阅。

失败示例：“暂时无法访问网络 · 尚未获取新闻” + “重试 / 查看连接”。只有后台已确认注册的重试才显示“下次重试”；不能说“我会继续”而没有任务。重复中间状态更新同一卡，不生成一串道歉聊天消息。

UFX-R07：移除所有模拟回复、硬编码已就绪、定时器成功状态；在线、能力可用、模型可用、已授权、执行中分别来自对应 owner。未知不是在线。右侧详情没有第二个聊天输入器。

### 4.3 人群与通信

UFX-R08：同一 ConversationWorkspace 支持回复/引用、转发、编辑/删除范围、撤销条件、反应、置顶、话题、已读、未读定位、搜索、分页、发送队列、定时与静默、媒体/文件/语音、通话、屏幕共享、群成员/权限/封禁、加入申请、邀请/分享链接、频道发布与评论。

动态、投票/问卷、协作清单、商业/支付/礼物/权益、Mini App、下载/导出等均由对象动作或专用能力面板进入，关闭恢复原会话草稿、阅读位置、选中项。清单仅是导航分类，完整范围以锁定来源责任清单为准；不得宣称此枚举覆盖全部源文件。

### 4.4 助手、渠道与扩展

UFX-R09：助手资料统一展示身份/模型、工具权限、技能、记忆、工作环境；模型凭据通过安全表单，不能要求用户贴到聊天。渠道连接：选择渠道 → 实际身份认证/配对 → 实际状态检查 → 展示账号与路由/接收策略；可取消/恢复，表单未保存时不得静默覆盖。消息回到同一会话/渠道来源标识中。

保留 OpenClaw gateway/HTTP/RPC/CLI、SDK/插件/skills、multi-agent、ACP、automations/cron/heartbeat、memory、media/TTS/talk、node/browser/computer、webhook、权限/沙箱、用量和恢复等全部范围，按 current owner 接线；诊断、协议和 API 不因 UI 简化消失。

## 5. 视觉与响应式

沿用 FDS semantic tokens（`--fabushi-*`）和 canonical primitives/icon registry，消除新增 `oc-*` 平行主题；兼容适配器可暂存但不能作为第二公共设计系统。

默认：rail 52px（可提供文字展开）、列表 288px 可调 240–360、header 52、详情 320 按需；正文宽度约 680–760、CJK 正文 14/22；聊天短内容不强行拉满，代码/表格可扩展。4px 间距体系、一个强调色、中性背景、8/12px 圆角；清晰选中与焦点；避免所有消息同一大灰胶囊。

UFX-R10：≥1200 宽可三栏含详情；900–1199 默认两栏，详情以互斥抽屉显示；<900 列表/会话分屏切换且有明确返回。最小 360 宽、200% 放大、长名称/RTL/深浅主题不裁切。触控目标 44px；键盘焦点可见，正文对比 ≥4.5:1，非文本控件 ≥3:1，reduced motion。不在无人为操作时抢焦点。

## 6. 架构/状态合同

遵守 existing-owner-first 与 single composition。现有入口 `frontend/src/production/ProductionRenderer.tsx` 仅做窄接线；新内聚模块放现有 owner 子目录，每文件目标 200–600 行，遵守现有上限，不把重设计堆进巨石文件。

Conversation/Transcript/Composer/draft 为 canonical owner；Task/Automations 和 Host/Runner 拥有执行事实；Coordinator 管路由；ChannelRuntime 管外部消息；Computer 管设备。前端只投影真实事件，不另造任务数据库。

投影至少携带 accountId、conversationId、taskId/runId、parentId、attempt、event sequence、authoritative status、updatedAt、capabilities、artifacts、recoverable/error kind。先检查现有协议并适配已有字段，缺失字段必须写合同与兼容策略，不能自造仅前端可见的语义。

UFX-R11：跨账号、跨会话、取消、迟到事件必须 fencing；重连去重与排序、终态单调；任务列表与原会话是同一对象的两个视图。任务卡不吞审批；高风险动作保留现有确认/授权。

## 7. 源码来源与学习结果

OpenClaw 锁定 `1440bb1771372334bea23d7642bb52deb7db1845`，本地 `/Users/bhrum/.gemini/antigravity/scratch/openclaw` HEAD 一致。
Telegram Desktop 锁定 `f985723b552b30fe1c17d377d57f44e61a15a8c4`，本地 `/Users/bhrum/.gemini/antigravity/scratch/tdesktop` HEAD 一致。
工作树只读研究，不修改上游。来源锁自身 accepted=false 等未通过门禁保持真实，找到源码不等于完成来源验收。

本轮已读示例：
- OpenClaw `ui/src/app-navigation.ts`：导航 route 与个人 pin 分离、Skills 归插件、Worktrees 归 Sessions、配置按用户注意力分类；学习渐进披露，不照搬菜单。
- OpenClaw `ui/src/pages/channels/wizard-setup.ts`：依赖真实 gateway connected、dirty config 保护、cancel/retire、完成后 refresh 与 stale identity fencing；用于渠道 UX 合同。
- Telegram `Telegram/SourceFiles/dialogs/dialogs_widget.h`：统一 dialogs/search/forum、返回与滚动/焦点连续性；不是独立论坛应用。
- Telegram `Telegram/SourceFiles/history/history_widget.h`：draft/cloud draft、reply return、媒体发送与 send options 在同一会话；输入器不能仅换皮后丢行为。
- Fabushi `ProductionRenderer.tsx`、`OpenClawAskCopilotPanel.tsx`、`OpenClawNavigationRail.tsx`、FBCP architecture-map/IA/design-system：已有唯一 owner 规则与新增平行 UI 相冲突，先收口生产入口。

这些是设计抽样，不宣称已读完整上游。Teamwork 必须从现有 inventory/ledger 增量补齐 exact-source → responsibility → owner → UI route → production call chain → oracle/evidence，不新增平行权威 ledger。所有 source review / implementation / wiring / verification 状态分开。

## 8. 实施顺序与团队分工

M0：核对实际生产基线、现有规范与来源；形成现有 ledger 的缺口分类；阅读本设计，冻结接口与截图判定标准。

M1（首批必须实际实现）：移除假的第二聊天；统一 header/composer/详情、收口主题；保留既有对象动作；没有事实依据的状态显示未知；构建可验证回归。

M2：任务真实状态投影、子代理归属、重连/审批/失败闭环；统一任务入口与自动化。

M3：Telegram 通信完整纵向闭环：native service → owner → shell，所有尚未 wired 的职责按依赖补齐。

M4：OpenClaw agent/channel/plugin/skill/automation/device/media 完整纵向闭环，兼容性与安全保持既有要求。

M5：冻结候选，完整来源/功能/视觉/时序/性能/设备验收；独立审查；满足发布门禁后交付。M1 通过只能宣称首批通过。

Antigravity 团队：Orchestrator 管依赖/唯一写入权；UX worker 管壳与 primitives；communication worker 管 FBCP/TDRP slices；agent worker 管 OCRM slices；state worker 管任务/恢复合同；独立 Critic/Challenger/Auditor 只读评判或在隔离测试区执行。共享 ProductionRenderer、合同、ledger 的修改由整合者串行收口。Codex 做最终独立验收，不照抄 worker success。

复杂阶段请求 xhigh；实际模型继承 CLI 现有配置，不擅自改全局设置。记录实际应用 effort 或 fallback。工作在本独立分支/子 worktree；禁止覆盖其他项目工作。

## 9. 验收矩阵

| ID | 场景/判定 | 最低证据 |
|---|---|---|
| UFX-AC01 | R01/R04/R07：任意对象只有一个 canonical workspace/composer；无假面板或伪在线 | 生产引用图、真实 packaged UI 操作与截图；断网负例 |
| UFX-AC02 | R05：内部子代理归父任务，用户同名助手保留、固定历史可达 | 元数据正反例、搜索/深链/删除回归 |
| UFX-AC03 | R06/R11：一次新闻请求中断/重连/授权/取消/重试不刷屏；终态与产物一致 | 真实 Host 事件、重复/乱序/迟到事件 oracle、重启时序证据 |
| UFX-AC04 | R08：两个真实账号跨设备发消息→回复→转发→搜索；群角色/频道/话题/通话等全责任逐项对照 | 服务部署身份、真实账号脱敏、交互证据；mock 不算 |
| UFX-AC05 | R09：真实 provider AI 回复和历史恢复；一个渠道真实收发与断线恢复；全部渠道/SDK 能力按 OCRM gates | 生产调用链、对应 oracle 与各责任证据 |
| UFX-AC06 | R04/R10：宽窄屏、深浅色、IME、键盘/读屏、放大、切换草稿/阅读位置 | 目标窗口截图、a11y/交互结果，回归对照 |
| UFX-AC07 | R02/R03：全部上游产品职责映射，无未解释遗漏；未接线不算 implemented 产品 | canonical ledger/inventory/RTM 一致与原始完整门禁 |
| UFX-AC08 | R01/R11：原有账号、插件/MCP、设备、审批、附件、任务、自动化、更新无回退 | exact candidate 的相关回归与产品流程 |
| UFX-AC09 | 长记录滚动、流式回复、后台任务不反复全量重绘/抢焦点，性能不退化 | 固定数据量/设备/基线、frame/CPU/memory/latency 采样与差值 |

验收状态：全部 not-run。设计文档、prototype、worker 宣告、局部单测都不能代替产品通过。执行环境遵守根 AGENTS 与当前专项政策，默认证据验证仅 GitHub Actions；不在用户 Mac 上执行 build/test/benchmark/acceptance。发现专项政策冲突先记录并沿更严格现行约束，不擅自降低。

## 10. 迁移、回滚、观测

UI 路由迁移保持原 ID、草稿、固定项、未读、账号边界；先收口入口后清理已无引用旧展示代码。禁止按名称批量删除会话。字段变更遵守版本兼容和向前/向后读取。

每批独立提交可回滚；出现消息丢失/重复、权限泄露、草稿损失、终态回退即拒绝验收。回滚 UI 不应恢复虚假状态面板，可退至已验证 canonical shell。数据迁移需独立回滚方案和备份证据。

日志保留 task/run/account 的脱敏关联、事件序号、错误类别和真实执行环境；不得记录密钥/认证 token/私聊原文。记录 exact head、命令/Action run、测试数量/退出码、截图/录像、缺失环境和解除条件。

## 11. 规范协同与完成记录

本文件是现有项目的体验实现增补，不新建产品架构。涉及一级导航/字体默认等变化需在同一实现批次窄更新 FBCP IA/design-system/canonical-screen-patterns/组件合同与当前 task；旧文档中具体默认值与本轮用户授权设计冲突时由本增补说明变更，不宣称历史状态已完成。

实施前完整阅读根 AGENTS、spec-first、FBCP/TDRP/OCRM 主规范、适用 current tasks、质量/验收和当前执行修订；这里只列设计决策，不豁免治理。

最终完成条件：UFX-AC01..09 + 所有原项目要求都有 exact-candidate 真实证据，0 open P0/P1/blocker，独立 ACCEPT，发布/安装状态分别核验。当前 complete=false。
