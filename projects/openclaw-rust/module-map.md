# OpenClaw 全源码责任域与 Fabushi owner 映射

Status: active specification; all migration domains pending  
Spec: [OCRM-001 Revision 1](../../docs/specs/openclaw-rust-equivalence-migration.md)  
Last updated: 2026-10-22（Asia/Shanghai）  
Project: [SOURCE_OF_TRUTH.md](SOURCE_OF_TRUTH.md)

## 1. 本表证明什么

本文件把完整 OpenClaw 源范围分为 OCRM-M01 至 OCRM-M32 共 32 个实施责任域，并列出首先检查的现有 Fabushi owner、关键故障边界及验收方向。它是逐文件、逐责任工作的入口，不能替代 recursive inventory、完整阅读 dossier、责任台账或生产验收。一个源文件可能承载多个责任域；必须拆责任并保留单一路径记录，不能遗漏或重复统计。

本次目录与源码观察固定于：

| 对象 | Commit | Tree |
| --- | --- | --- |
| OpenClaw | `e8cde7518c558f1b4e44cb8139a149289b9719ad` | `25c8ee611b72762ae1a0170124fa1fee9e519704` |
| Fabushi canonical main | `ba9ecce728ef021fee609796c8704d8af6fe4b5a` | `29e6837e6ea9098d64c5de3e0952b3960abbfc86` |

状态严格区分：

- **路径存在**：固定 tree 中发现文件/模块；不能证明被 shipping entrypoint 使用。
- **接线已观察**：实际入口调用该 owner；仅证明现有功能的某段接缝存在，不能证明 OpenClaw 等价覆盖或 packaged acceptance。
- **迁移验收**：按主 Spec 的 `responsibility_kind` 合同逐项验收。业务运行时责任必须有真实 Rust 实现、生产 producer/owner/consumer、对应 Actions 证据和产品验收；表现/平台桥接、schema/codegen、构建发布、测试/oracle、文档/治理、资源/许可证等责任提供各自类型要求的目标产物、接入/使用证据和验收。非运行时责任不强求伪造 Rust 函数或 packaged runtime 路径；某维度不适用须有依据，责任仍保留在分母。mixed 文件中的业务责任须单独拆出并满足 Rust 要求。

以下 32 个域全部为 **pending**。没有把任何域标为 read-complete、implemented、verified 或 accepted；递归文件总量与完整迁移进度尚未建立。

## 2. 当前 main 的装配事实与易混淆边界

`frontend/src/main.tsx` mount `frontend/src/production/ProductionRenderer.tsx`；`source/electron-main/entry.ts` 装配生产 adapters 和 Coordinator。`source/node-agent-coordinator/src/main.rs` 启动独立 Host；`source/host/app/src/main.rs` 调用 `start_production_host_extensions`，并启动 Session、Transcript、MCP、Automations 等 owner。来源：[F1]、[F2]、[F3]、[F4]。

Host `host_runner_composition.rs` 将 Session/Memory 的状态接给 Runner；自身不成为第二份会话/记忆真相。`desktop/package.json` 指向 `source/host/app`、`source/node-agent-coordinator`、`source/box-exec-daemon` 的构建入口。它们是源码接线观察，不能视为本次编译、打包或运行通过。来源：[F5]、[F6]。

### 2.1 已有 Human 消息与通话 owner

当前 Renderer 的 Human 分支调用 `sendHumanMessage`，Agent 分支保留 `sendPrompt`，二者进入同一 Composer/Transcript。Host `extensions/session/gateway.rs` 分派 Human 消息和 CallSession 方法；`production.rs` 管理 native conversation 与 CallSession；`native_messaging.rs` 处理 Fabushi 原生 identity/message/resource/call 服务交互。旧 FBCP owner inventory 中“尚不存在 Human/call owner”的历史判断不能直接沿用。上述事实也不证明群组通话、OpenClaw voice/Talk 或所有外部渠道已完成。来源：[F1]、[F7]。

### 2.2 外部 channel 的 default 尚未注册

`source/host/src/connectors/channel_delivery.rs` 的 `HostChannelDelivery::default()` 使用 `UnregisteredChannelDeliveryPort`；本次读取的 Host app 仍注入该 default。因此 `connectors` 目录存在只是复用接缝，不是可投递渠道的证明。每个 OpenClaw channel 都必须补足真实 ingress/egress registration、鉴权、状态机、UI 配置与服务证据。外部渠道不成为 Fabushi 原生身份或通信网络。来源：[F4]、[F8]。

### 2.3 未合并 PR 仅作候选依赖

| PR | 本次读取状态与 head | 本表如何使用 |
| --- | --- | --- |
| [#165](https://github.com/bhrumom/fabushi-desktop/pull/165) Browser Workspace | open / unmerged；`1eac85389c3d5b3fc6dfc71a1741be2427040800`；base 为上述 main | M12 的候选工作；不能冒领为 main 已有原生 Browser Workspace |
| [#166](https://github.com/bhrumom/fabushi-desktop/pull/166) Mini App authoring | open / unmerged；`26ec2ebb7159b21b0382a8c6d46d3534a5cdfab4`；base 为上述 main | M17/M27 的候选工具；main 已有安装/runtime 不等于已有完整 authoring |
| [#167](https://github.com/bhrumom/fabushi-desktop/pull/167) Hermes hardening | open / unmerged；`af2824fa5b4f2231c4d9689d580f6f50e06a957f`；依赖 #166 | M05/M08/M10/M11 的候选修复；不得引用其测试数证明 main |
| [#168](https://github.com/bhrumom/fabushi-desktop/pull/168) Mini App publication | open / unmerged；`112e06407ec990472a7d6c2f0d84429a23e4952d`；依赖 #167 | M27/M31 的候选发布能力；不能冒领真实 Release 或 signed installation acceptance |

开始实现时重新读取这些 PR 的 live 状态及 exact head，检查是否已合并、是否与当前 main 冲突；协调已有 owner 工作，不覆盖并行分支或重做第二套实现。TDRP PR #154 的范围、台账和证据独立保留。

## 3. OCRM-M01—M32 全量责任域

源路径均相对于上述 pinned OpenClaw；目标路径相对于上述 pinned Fabushi。每行列的是优先审计入口，而非所有文件或已通过的 owner verdict。域内全部实际文件、平台分支、测试 oracle、生成合同和外部依赖须在 P0 中展开。

为避免表格反复写长前缀，目标列的 Host 文件/`extensions/...`/`storage` 指 `source/host/src/` 下对应路径，Runner 文件指 `source/host/src/runner/`，Coordinator 文件指 `source/node-agent-coordinator/src/`，Electron 文件指 `source/electron-main/`；显式完整路径优先。领域名称只是 owner 候选，实施 dossier 必须记录完整 target path/symbol 和调用链。

| ID / 责任域 | 上游范围与必须迁入的行为 | 现有 owner 候选与融合方向 | 关键失败边界及验收要点 | 状态 |
| --- | --- | --- | --- | --- |
| OCRM-M01 启动、daemon 与运行生命周期 | 根入口、`src/entry.ts`、`src/runtime.ts`、`src/bootstrap`、`src/daemon`、`src/infra`；启动/探测/配置加载、服务启停、后台与无头运行、进程生命周期 | `source/electron-main/entry.ts`、`startup`、Coordinator `main.rs` / gateway supervisor、Host app `main.rs`；兼容入口只分派到现有 Rust owner | 重复启动、半启动、端口冲突、父进程丢失、睡眠/恢复、崩溃循环、graceful shutdown；真实 package/service 证明 readiness 与实际 owner identity，一次进程启动不等于可服务 | pending |
| OCRM-M02 配置、profile 与设置语义 | `src/config`、配置 schema、env/profile precedence、JSON/JSON5、默认值、验证、迁移、热重载与回滚 | `source/host/src/extensions/settings/settings_service.rs`、`gateway_config.rs`、`source/electron-main/prefs`、现有 Settings UI；业务配置规范化/应用由 Rust 持有 | 缺失/null/空值、非法字段、同时更新、账号切换、写入失败、旧 generation 覆盖新值；逐配置项验证接受后真实生效，配置与运行状态原子一致 | pending |
| OCRM-M03 身份、凭据、pairing 与账户范围 | `src/secrets`、`src/pairing`、`extensions/device-pair`、`onepassword`、`vault` 以及 provider/channel auth；SecretRef、刷新/撤销、配对和授权 | Host `extensions/auth` / `extensions/secrets`、Electron account/auth/secure storage、`network_enrollment_owner.rs`；分别选择账户、设备和 Box secret owner，不把凭据统一塞入 Settings | 错账户、过期 token、refresh 竞争、撤销后在途请求、失效 pairing、secret 泄漏；以受控真实账号验证重新授权、撤销、隔离和脱敏，不以 capability discovery 授权 | pending |
| OCRM-M04 Gateway、公开协议与兼容边界 | `src/gateway`、`src/compat`、`packages/gateway-protocol`、`gateway-client`、`sdk`、`crates/openclaw-gateway-client`、`extensions/admin-http-rpc`；v4 与规定 v3 node/probe 窗口、WS/HTTP、roles/scopes、method/event/error | `source/host/src/gateway_protocol.rs`、`gateway_server.rs`、`host_gateway_api.rs`；Coordinator gateway、现有 typed host protocol；只增 Rust compatibility adapter | 遍历 schema/dispatch/plugin/role-specific registry，而非仅 `hello-ok.features.methods`；验证 challenge、schema/limits、断线/gap、`OWNER_UNAVAILABLE`/`OWNER_REFUSED`/`OUTCOME_UNKNOWN`，已派发 mutation 不自动 replay | pending |
| OCRM-M05 Agent、harness 与 turn 执行 | `src/agents`、`src/acp`、`src/system-agent`、`packages/agent-core`、`acp-core`、`extensions/acpx`、`codex`、`agentsapi`、`advisor`、`agent-guidance`、`autoreview`；persona/workspace、全部 harness/ACP/A2A、流式执行与最终结果 | `source/host/src/runner/production_turn_agent_owner.rs`、`turn_agent_composition.rs`、`sand_agent_runner.rs`、`extensions/turn_execution`；Coordinator 路由与 Host/Runner 执行边界保留 | accepted/running/approval/terminal/settled、stop/steer/cancel、静默输出、provider failure、checkpoint/restart；真实 Agent→tool→Transcript 路径及已有 Bot 回归，不保留原 OpenClaw loop 为 owner | pending |
| OCRM-M06 模型目录、provider、账户与 usage | `src/provider-runtime`、`src/model-catalog`、`src/model-picker`、`src/llm`、`packages/ai`、`llm-core`、`model-catalog-core` 及全部 provider extensions；本地/远端模型、custom endpoint、能力选择、fallback、费用/限额 | Coordinator `inference_router.rs`；Host `extensions/inference`、Runner `routed_provider_runtime.rs`；模型/账户管理进入现有 Settings | 不同模型的 stream/tool/usage/停止语义、auth/rate-limit、fallback 重复副作用、失败请求计费、历史 usage 重算；每 provider 独立协议及真实服务矩阵，保留现有 provider 能力 | pending |
| OCRM-M07 会话、历史、分支与 Transcript | `src/sessions`、`src/agents/sessions`、`src/config/sessions`、Gateway session API、`packages/session-url-contract`；session identity、branch/fork/reset、历史/搜索/导出、异步事件和终态；仅消费 M25/M26 的媒体转写结果 | Host `extensions/session`、`extensions/transcript`、`transcript_mirror`；现有 Conversation/Composer/Transcript；保留 Human 与 Agent typed entries，不取得 meeting/audio capture/source 生命周期 | sessionKey/sessionId 不混用，分页/乱序/重复、nonce reconcile、旧 snapshot、崩溃与账户切换；证明所有消费者看到唯一稳定历史与 terminal，无第二 session DB；转写结果引用保留 source/utterance provenance | pending |
| OCRM-M08 上下文、prompt、compaction 与工具参数修复 | `src/context-engine`、`src/agents` 内 prompt/context 部分、`packages/tool-call-repair`、`normalization-core`；窗口预算、instructions、截断、compaction、pending input、重复输出防护 | Runner `system_prompt_assembly.rs`、`prompt_collector_glue.rs`、`stream_attempt.rs`、`large_output_spill.rs`；Session/Transcript 为输入真相 | malformed JSON、跨语言 Unicode/offset、超大输出、compaction 与正在执行工具竞争、丢失用户输入、恢复重计数；用确定性 oracle 检验内容保留、预算和副作用，不要求 LLM 文本逐字一致 | pending |
| OCRM-M09 多 Agent、路由、group 与子任务 | `src/routing`、`src/agents` 多 Agent 部分、A2A、group/swarm/subagent；lineage、会话绑定、并发、递归与预算、完成回传 | `source/host/src/runner/subagent_runtime.rs`、`groups`、Transcript group glue/orchestrator/fanout、Coordinator inference queue | parent/child 隔离、并发 race、steer/abort、递归配额、parent 消失、重复完成/唤醒；证明现有 canonical Agent/Group 模型及 caller→child→result 可达，无第二调度/agent-store | pending |
| OCRM-M10 工具、权限、文件/网络与 sandbox | `src/security`、`src/audit` 的授权部分、工具 registry、`packages/net-policy`、`extensions/policy`、`openshell`、文件/exec/PTY/apply-patch；审批与能力约束 | Host `extensions/local_tool_permission`、`auto_review`、`action_audit`、`local_exec`、`box`；Runner review gate；`source/box-exec-daemon` | 参数/target/grant/generation 绑定，prompt injection、SSRF/redirect、symlink/path/archive escape、权限降级、cancel 后副作用；跨平台真实 sandbox 正反例，未知执行结果先 reconcile | pending |
| OCRM-M11 后台进程、worker、Code Mode 与云资源 | `src/process`、`src/worker`、`packages/worker-runtime`、`extensions/code-mode-quickjs`、cloud/sandbox provider、后台 jobs；QuickJS/WASM guest、资源配额与回收 | Runner `background_work.rs`、`shell_terminal_watch.rs`、Host `extensions/cloud_agents`、`runner/turn_run_shell.rs`、box lifecycle；现有 JS/worker 边界先查 reachability | worker 分配失败、launcher 丢失、stdout 溢出、超时/abort、后台完成丢失、提前归还 lease；执行终态与 cleanup 分开，真实 pending-wake/checkpoint/restart 取证 | pending |
| OCRM-M12 Browser、Web Search/Fetch 与链接理解 | `extensions/browser`、`src/web-search`、`src/web-fetch`、`src/link-understanding`、browser cleanup、搜索/抓取 provider；CDP/profile/tabs/login、导航/交互/截图/上传下载 | Runner `tools/sand_browser_tools.rs`、`sand_browser_driver_source.rs`、`sand_web_tools.rs`、`host_web_dependencies.rs`；Computer、Electron 平台边界；#165 仅候选依赖 | tab/profile/account ownership、viewport/generation、导航与用户 takeover、上传路径授权、下载 provenance、SSRF/网页不可信；真实浏览器 E2E，不能以 mock DOM/截图替代可操作性 | pending |
| OCRM-M13 Node、Computer、平台设备与远程操作 | `src/node-host`、`crates/openclaw-node-host`、`extensions/linux-node`、`cua-computer`、`geolocation`、`bonjour`、`apps`；pairing、命令 manifest、camera/screen/location/通知、移动 wake | `remote_computer_target.rs`、`network_enrollment_owner.rs`、`network_worker_process.rs`、`source/fabushi-network-worker`、Runner Computer、现有 native/mobile host protocol | 宣告能力与授权/可执行命令一致；设备撤销、process death、网络切换、stale lease/坐标、未知结果、背景恢复；每平台/设备 cell 用 Actions 管理的真实 lane，不因缺设备判 N/A | pending |
| OCRM-M14 全部外部渠道、连接与 ingress | `src/channels`、`src/web` 中 WebChat/channel 职责、全部 channel extensions、pinned external channel catalog；登录/QR/OAuth、DM pairing、allowlist、群/线程/mentions、成员与入站 cursor | Host `connectors`、`extensions/automations/listener_integrations.rs`、`source/shared/channels.ts`、既有 Agent-info channels/Plugins UI；Rust adapter 投影至 canonical Conversation/Resource | default channel 尚未注册；每渠道证明实际连接、真实注册、inbound delivery、bot-loop/ambient/join-introduction 规则、内容 trust、checkpoint/reconnect/logout；逐平台能力不可凭同名接口推定 | pending |
| OCRM-M15 自动回复、路由与 outbound 投递 | `src/auto-reply`、`src/chat` 的投递部分、`src/routing`、channel plugins；分块/stream、回复/线程、编辑删除/反应/附件/富交互、回执/速率限制 | `connectors/channel_delivery.rs`、`channel_attachment.rs`；Host Transcript/Agent messaging 与已有 delivery-failure wake；Composer/Resource 投影 | accepted execution 与 accepted delivery 分开；幂等/TTL 去重、重试/限流、reply 丢失、unknown outcome、recipient 变化、撤销/重启；每渠道支持动作真实服务验证，无未注册假成功 | pending |
| OCRM-M16 插件、SDK、安装与市场生命周期 | `src/plugins`、`src/plugin-sdk`、`src/plugin-state`、`packages/plugin-sdk`、`plugin-package-contract`、全部 175 plugin entries；发现/安装/启停/更新/卸载、hooks/routes/services/tools、exclusive slots | Host `extensions/mcp`、Electron MCP、Coordinator MCP relay、既有 Plugins/marketplace；不足的 SDK hook 经最小 Rust bridge 扩展，不用原 OpenClaw host | manifest 读取不执行任意代码；integrity/schema/version、generation 原子切换、hook 顺序、撤销/dispose、owned timers/tasks、guest quota；每插件 manifest→registration→用户行为 conformance，MCP 兼容不冒领 SDK 兼容 | pending |
| OCRM-M17 全部 Skills、custodian skills 与工作流内容 | `src/skills`、`skills` 50 目录、`custodian-skills` 4 目录；SKILL 内容、触发、依赖安装、OS/CLI/script 集成、维护/创建/发布 | `extensions/mcp/plugin_skills.rs`、`plugin_skills_cache.rs`、`skill_publish.rs`、`workflows/workflow_library.rs` / `workflow_store.rs`、现有技能/插件入口 | stale cache、auth-blocked、撤销后旧 skill、安装副作用、路径/参数注入、缺 CLI/账号；每技能独立验证实际工具路径；文案可作内容保留，业务脚本责任不能靠复制 SKILL.md 闭合 | pending |
| OCRM-M18 MCP client/server/relay 与事件 | `src/mcp`、`extensions/mcp-relay`、`mcp-events`、相关 SDK；stdio/SSE/HTTP、tools/resources/prompts、分页/订阅、认证/callback、取消与重连 | Electron MCP / OAuth、Coordinator `mcp_lifecycle_relay.rs`、Host `extensions/mcp/coordinator_relay.rs`、MCP service 与 Runner tool bridge | schema 校验、工具身份、授权范围、分页漏项、callback 重放、stream close、session 失效、account switch；与真实受控 MCP peer 双向验证，发现列表与执行权限分开 | pending |
| OCRM-M19 Automation、cron、hooks 与 Lobster | `src/cron`、`src/hooks`、`extensions/lobster`、`imap`、`mcp-events`；at/every/cron/stream/on-exit/event/webhook、stream batch/match、standing orders、activeHours/idleOnly、scratch/context/grant、quiet/no-reply、heartbeat migration | `source/host/src/automations`、`extensions/automations`、现有 Automations UI、pending-wake/trigger hub；heartbeat 合并为普通 job，不新建旧 Tasks ledger/TaskFlow；stream source identity 是独立合同 | IANA timezone、非一小时 DST gap/fold、clock rollback、sleep/catch-up、job edit revision、重放 webhook、审批恢复、execution/delivery 分离；同一 schedule 的 child restart 保留持久 `streamSourceIdentity`，disable/remove/replace 原子轮换并拒绝旧来源事件；child/process lifetime 或执行 generation 单独拒绝旧实例迟到 batch/match，不以轮换持久身份代替实例隔离；现有 `CRON_TZ`/`TZ` 不能代替 stream oracle | pending |
| OCRM-M20 记忆、搜索、wiki、vault 与隐私 | `src/memory`、`src/memory-host-sdk`、`packages/memory-host-sdk`、`extensions/memory-core`、`memory-lancedb`、`active-memory`、`memory-wiki`、相关 vault/Obsidian；embedding/retrieval/flush/Remember/Forget/修订 | `extensions/memory`、`extensions/content_search`、Runner `turn_memory.rs` / `sand_memory.rs`、现有 Search/UI；知识存储不足时先做 owner analysis | 群/私密/account/Agent 隔离、来源/有效期、异步写入冲突、索引 stale、删除传播、合成覆盖、损坏恢复；检索质量与确定性隔离/保留合同分开，现有 memory file store 不等价于全部上游 memory | pending |
| OCRM-M21 项目、boards、claws 与团队工作 | `src/projects`、`src/boards`、`src/claws`、`src/decisions`、`extensions/workboard`、`team-reports`、`logbook`、`packages/workboard-contract`；对象/依赖/权限/日志/工作组织 | 先检查现有 Task/async tasks、Automations、Agent/Group、Transcript/artifacts、workspace；`workflows` 只是候选，不能把 skill workflow 或 setup flow 当成已有 board domain | 并发编辑、跨项目引用、权限变化、删除/恢复、依赖循环、版本跳跃、后台执行；所有源操作与 UI 闭合，缺领域 owner 时建最小 source-neutral ADR，不能删功能或复活已退休 TaskFlow | pending |
| OCRM-M22 Snapshot、trajectory、session cards 与共享 | `src/snapshot`、`src/trajectory`、`src/session-cards`、`extensions/session-share`、`beam`、`visitor-access`、会话分享/public transcript/GitHub preview | Session/Transcript、Resource、`extensions/cross_user_sharing`、既有权限/共享和导航接缝；只读派生视图不拥有第二 history truth | frozen snapshot 一致性、过期/撤销、访客范围、私密信息泄漏、链接重放、只读 Beam mutation 拒绝；source-only visitor-access 仍须完整权限验收，share UI/服务都需真证据 | pending |
| OCRM-M23 附件、媒体理解、文件传输与文档提取 | `src/media`、`src/media-understanding`、`packages/media-core`、`media-understanding-common`、`extensions/document-extract`、`file-transfer`；格式/MIME/大小、上传下载、provenance、缓存、富资源；与 `src/transcripts` 的 capture/export 资源衔接归属见 M25/M26 | Host `extensions/attachments`、`attachment_paths.rs`、Electron attachment/media manager、Composer/Transcript Resource；原生消息资源接口；转写原始媒体/导出产物复用资源生命周期，不取得 capture provider 状态 | 超大/损坏/伪 MIME、路径逃逸、断点/取消/重复、账户权限、资源清理与引用竞争；实际不同格式与大资源生命周期、错误呈现、原文件/utterance/summary/export provenance 和撤销清理 | pending |
| OCRM-M24 图像、视频、音乐与生成资源 | `src/image-generation`、`src/video-generation`、`src/music-generation`、`src/media-generation`、`packages/media-generation-core`、`extensions/image-generation-core` 及全部对应 provider | Host attachments generation service、Runner `host_generate_image_dependencies.rs`、既有 inference/tool/provider、Resource/Transcript；缺 modality 的 owner 必须明确补全 | 长运行/轮询、provider quota、取消后仍计费/生成、重复提交、unknown outcome、格式/附件 metadata；逐 provider/modality 验证实际生成和消费，不以文本 stub 替代 media | pending |
| OCRM-M25 音频、STT/TTS、Talk 与 realtime | `src/tts`、`src/talk`、`src/realtime-transcription`、`src/transcripts` 的 audio capture/provider/source access/utterance 职责、`extensions/talk-voice`、speech/TTS plugins、`apps/macos-mlx-tts`、`apps/swabble`；wake、流式转写、打断、设备切换、离线能力 | 现有 Electron media/平台、Host/Runner/tool 与 Resource、native/mobile host 接缝；音频 capture/provider 状态与权限分别选最小 owner，M23 处理资源生命周期，M07 仅消费结果 | 麦克风/source access、capture start/stop/cleanup、background/wake、barge-in、echo、网络/设备切换、跨会话音频、迟到 utterance、模型资源；真实音频/设备 lane 与 source/speaker/time provenance，离线/本地和云模式各自验证 | pending |
| OCRM-M26 通话、会议 capture 与会议 Agent | `src/meeting-bot`、`src/transcripts` 的 meeting capture/provider/source access、participants/utterances/store/summary/export/cleanup、`extensions/voice-call`、`google-meet`、`teams-meetings`、`zoom-meetings`、`slack-huddles`、`facetime`；加入退出、录制与 Agent 参与 | Host Session/CallSession、`native_messaging.rs`、Renderer `human-call-media.tsx`、Electron `call-media.ts`、Computer/platform；媒体转写 source/capture 与 M25 选唯一 owner，资源/导出接 M23，聊天消费接 M07 | caller/participant/source identity、capture/provider 授权、录制同意、utterance 顺序/迟到、summary/export 一致性、join 失败/重复、撤销/断网/后台恢复与 cleanup；真实会议及负例，现有 Human call/聊天历史不冒领会议 capture parity | pending |
| OCRM-M27 Canvas、Control UI 与统一产品交互 | `src/canvas`、`src/interactive`、`ui`、`extensions/canvas`、`diffs`、`diffs-language-pack`、`packages/markdown-core`、`mermaid-renderer`；Canvas/A2UI、control/chat、rich artifacts、diff/diagram | `ProductionRenderer.tsx`、既有 conversation/cards/Plugins/Settings、Mini App host/window、`extensions/mcp/installable_package.rs`；业务逻辑移 Rust，视图复用 Fabushi shell | hostile content/origin、bridge grant、过期动作、nav/close/dispose、运行中任务跨窗口/切换连续；统一 route、loading/empty/error/offline、keyboard/a11y/IME/RTL/light-dark/visual，禁止第二 Dashboard | pending |
| OCRM-M28 CLI、TUI、wizard 与操作入口 | `src/cli`、`src/commands`、`src/tui`、`src/wizard`、`src/chat` 的交互命令、`packages/terminal-core`；onboarding、slash command、快捷操作、doctor/管理命令 | 现有 Mahayana CLI/TUI/protocol 先核实真实 shipping reachability；命令分派到与 GUI 相同 Rust domain owner；Electron onboarding/settings/command palette | 非交互/headless、exit codes、stdout/stderr、pipe/terminal 编码、prompt 取消、配置缺失；逐 command/subcommand/help/flag 与 GUI/API 对等，不以 crate 存在当作已交付 CLI | pending |
| OCRM-M29 存储、导入、数据迁移与恢复 | `src/state`、`src/storage`、`src/compat` 的迁移职责、`extensions/migrate-claude`、`migrate-hermes`；SQLite/PostgreSQL/路径、配置/历史/记忆/自动化/插件导入、backup/export/delete | Host `storage`、Session `agent_db*`、`session_recovery.rs`、`agent_isolation`、Transcript mirror、现有 startup/migration；唯一 canonical writer | 磁盘满/损坏、迁移中断、旧 schema、重复导入、ID 冲突、凭据重授权、删除保留、rollback；preview/backup/transaction/idempotency 与无静默丢失，不能长期双写两套 session truth | pending |
| OCRM-M30 日志、诊断、审计与观测 | `src/logging`、`src/status`、`src/proxy-capture`、`src/audit` 的观测职责、`extensions/diagnostics-otel`、`diagnostics-prometheus`；health/doctor/metrics/traces、导出与 consent | `source/host/src/host_diagnostics.rs`、`extensions/telemetry` / `action_audit`、Electron telemetry/process metrics/feedback、现有错误通知 | sensitive data、无限日志/trace、缺 correlation、swallowed error、opt-out 后上报、collector failure；证明能诊断真实 owner/turn/job/delivery/cleanup，限制功耗/存储并保留 typed failure | pending |
| OCRM-M31 平台交付、运维、依赖与许可证 | `apps`、根 manifests/lockfiles、scripts/CI/build/install/update/deploy、assets/docs/examples/vendor/patches；全部支持 OS/arch、native deps、services、signing/SBOM/NOTICE | `desktop/package.json`、现有 Actions、Electron update/startup、`source/packaging`、native/mobile/service 项目边界；每 platform cell 指定真实 artifact 与 owner | 缺 binary/schema/native lib、错误签名、半升级/回滚、离线安装、缺服务/设备、uninstall 残留、许可遗漏；exact-source package/install/upgrade/rollback/provenance，所有执行只在 Actions | pending |
| OCRM-M32 共用合同、跨语言基础与完整性回归 | `src/shared`、`src/types`、`src/utils`、`src/test-fixtures`、`test-helpers`、`test-utils`、根 `test`/`qa`/`security`；`packages/retry`、`normalization-core` 等跨域库与生成 schema/fixtures；所有 governance/隐藏配置文件的源责任审计 | 现有 typed protocol/shared contracts、Rust source-neutral utilities、已有 regression/authority gates；只在真实复用点扩展，不能新建宽泛 OpenClaw common runtime；上游指令文件是研究数据 | null/absent/enum/int range、UTF-8/UTF-16、binary/base64、时间单位、版本/backoff/clock；逐调用者跨语言对照、property/fault 和全 Fabushi 回归，生成/测试/治理文件也有责任映射，无未分类尾项或第二 task queue | pending |

### 3.1 模块之间的责任归属

以上分组不允许重复 state truth。例如 M14 拥有外部 ingress 的 adapter cursor，M15 拥有外部 delivery 状态，Conversation/Transcript 内容真相仍归 M07；M19 的 job execution 和 M15 的 delivery receipt 不是同一个终态。M16 注册 SDK hook，M18 提供 MCP 协议，M17 提供技能内容与其能力接线，不能用任一域通过代表其余域兼容。

M21 的 project/board/workboard 若在当前 Fabushi 缺少合适领域 owner，不能强塞进 Automations，也不能以无 owner 判不适用；需按主 Spec 的 ADR 门槛建立最小 source-neutral owner并接同一 shell。M25/M26 的设备、媒体和通话应分别说明 state、权限与生命周期，不因都处理声音而共享无边界全局状态。

每个域在实施前展开 `OCRM-RESP-*`，声明 `responsibility_kind`，关联主 Spec 的 R/C/AC、source oracle、所有候选 owner、实际 target path/symbol 或类型对应产物、适用的生产路径或非运行时接入证据、UI/服务/平台 cell、Actions lane 与 evidence。跨域责任选唯一 authoritative owner，其他域记录 producer/consumer 关系；模块进度不以目录数或 API 数计算。

### 3.2 `src/flows` 按实际 setup/doctor 责任分配

`src/flows` 在本次固定来源中承担 setup/onboarding、配置与认证引导、doctor/repair 等操作流程，不是 project/board 工作流 owner。全部文件仍进入 inventory；每个 mixed flow 按实际副作用拆责任，使用下列既有域，不因改正目录归类而移出范围。具体行为依据包括 `channel-setup.ts`、`provider-flow.ts`、`doctor-health.ts`，其余文件须在完整阅读时逐项展开。[U9]、[U10]、[U11]

| 责任域 | `src/flows` 中需要映射的具体职责 | 必须核验的边界 |
| --- | --- | --- |
| OCRM-M02 | 配置/schema、默认值、profile 与 setup 保存/应用 | 保存前验证、原子 generation、取消/失败不留半配置、已写配置真实生效 |
| OCRM-M03 | 账户/credential/pairing 的认证引导与授权交接 | 正确 principal、凭据不泄漏、刷新/撤销、账号切换 fence；引导 UI 不拥有 secret 真相 |
| OCRM-M06 | provider/model/account 选择与配置流程 | provider 能力与身份、模型目录/凭据一致、失败回滚；不建立第二模型设置 store |
| OCRM-M14 | channel setup、账号连接与入站配置 | 真实 channel registration/connect、权限/allowlist、未注册/未连接明确错误、不伪造完成 |
| OCRM-M20 | memory/search provider 与检索能力 setup | 选中的真实 backend、权限/范围、索引准备/失败、现有数据保留；配置成功不等于 memory 等价 |
| OCRM-M28 | onboarding/wizard/交互式 setup 及其 CLI/GUI 操作入口 | 非交互模式、取消/退出码、可发现性、CLI/GUI 同一 owner 与状态 |
| OCRM-M29 | repair 中涉及的旧配置/数据迁移、备份与恢复 | 幂等、事务、损坏/中断、无静默删除；只有源中真实存在的修复步骤才登记为该流程行为 |
| OCRM-M30 | doctor/health 检测、诊断结果及 repair 调度 | 诊断与修改动作分开、证据真实、执行授权、失败可见、日志脱敏、不得报告假健康 |

`src/transcripts` 则依据其 [provider 合同][U12] 与 [store 合同][U13] 分解到 M25/M26，并由 M23 承接资源关系、M07 消费结果；不能按目录名把它当作 Agent 会话历史存储。stream schedule 的 source identity/batch/match/fencing 必须依据 [schedule][U14] 与 [types][U15] 落到 M19。

## 4. Pinned 上游直接子目录观察

以下列表来自对固定 commit 的 GitHub Contents API 只读返回，列出全部直接子目录名称。它们**不是 recursive inventory，不是全文件阅读，不是已接受源闭包**。目录内文件、同层普通文件、隐藏配置、gitlink/LFS/生成物及依赖必须另行完整枚举。`extensions/test-support` 是测试支持目录；`apps/.i18n` 是国际化目录，不能据此额外计算产品插件或平台。

### 4.1 `src`：77 个直接子目录

来源：[pinned src tree][U1]。

```text
acp, agents, audit, auto-reply, boards, bootstrap, canvas, channels, chat,
claws, cli, commands, compat, config, context-engine, cron, daemon,
decisions, docs, flows, gateway, hooks, image-generation, infra,
interactive, link-understanding, llm, logging, mcp, media-generation,
media-understanding, media, meeting-bot, memory-host-sdk, memory,
model-catalog, model-picker, music-generation, node-host, pairing,
plugin-sdk, plugin-state, plugins, process, projects, provider-runtime,
proxy-capture, realtime-transcription, routing, scripts, secrets,
security, session-cards, sessions, shared, skills, snapshot, state,
status, storage, system-agent, talk, test-fixtures, test-helpers,
test-utils, trajectory, transcripts, tts, tui, types, utils,
video-generation, web-fetch, web-search, web, wizard, worker
```

`src` 同层普通文件也在范围内，包括 entry/runtime/index/library/globals/logger、browser lifecycle cleanup、Docker health/build inputs、poll 参数、版本与相关测试；不能只登记上列目录。

### 4.2 `extensions`：176 个直接子目录

来源：[pinned extensions tree][U2]、[plugin inventory][U8]。plugin inventory 的 73 core、99 official external、3 source-only 共 175 plugin entries 与 176 个目录不是同一个计数维度；外置发布的 official 实现依然需要逐项来源/版本/合同闭合。

```text
a2a, acpx, active-memory, admin-http-rpc, advisor, agent-guidance,
agentsapi, alibaba, amazon-bedrock-mantle, amazon-bedrock,
anthropic-vertex, anthropic, apple-fm, arcee, autoreview, azure-speech,
baseten, beam, bifrost, bonjour, brave, browser, buzz, byteplus, canvas,
cerebras, chutes, clawrouter, clickclack, cloudflare-ai-gateway,
cloudflare, code-mode-quickjs, codex, cohere, comfy, copilot-proxy,
copilot, crabbox, cua-computer, deepgram, deepinfra, deepseek, device-pair,
diagnostics-otel, diagnostics-prometheus, diffs-language-pack, diffs,
discord, document-extract, duckduckgo, elevenlabs, exa, facetime, fal,
featherless, feishu, file-transfer, firecrawl, fireworks,
fish-audio-speech, geolocation, github-copilot, github, gmi, google-meet,
google, googlechat, gradium, groq, huggingface, image-generation-core,
imap, imessage, inworld, irc, kie, kilocode, kimi-coding, levanto, line,
linux-node, litellm, llama-cpp, llm-task, lmstudio, lobster, logbook,
longcat, matrix, mattermost, mcp-events, mcp-relay, memory-core,
memory-lancedb, memory-wiki, meta, microsoft-foundry, microsoft,
migrate-claude, migrate-hermes, minimax, mistral, moonshot, msteams, muse,
mxc, nextcloud-talk, nostr, novita, nvidia, oc-path, ollama, omlx,
onepassword, onnx, openai, opencode-go, opencode, openrouter, openshell,
parallel, perplexity, pixverse, policy, qa-channel, qa-lab, qianfan, qwen,
radius, raft, reef, runway, searxng, senseaudio, session-share, sglang,
signal, slack-huddles, slack, sms, stepfun, subq, synology-chat, synthetic,
talk-voice, tavily, team-reports, teams-meetings, telegram, tencent,
test-support, tlon, together, tokenjuice, tts-local-cli, twitch, typesafe,
vault, venice, vercel-ai-gateway, visitor-access, vllm, voice-call,
volcengine, voyage, vydra, web-readability, whatsapp, workboard, x, xai,
xiaomi, zai, zalo, zalouser, zoom-meetings
```

`extensions/AGENTS.md`、icon provenance、activity icon sources、tsconfig 与同层 provider live tests 等都属于源范围。不得仅迁移看起来像主要产品的插件；QA plugin 的测试/诊断 oracle 与 visitor-access 的权限责任必须单列。

### 4.3 `skills`：50 个直接子目录

来源：[pinned skills tree][U3]。

```text
1password, apple-notes, apple-reminders, bear-notes, blogwatcher, blucli,
camsnap, clawhub, coding-agent, control-ui, create-claw, diagram-maker,
eightctl, gemini, gh-issues, gifgrep, github, gog, goplaces, healthcheck,
himalaya, mcporter, meme-maker, model-usage, nano-pdf, node-connect,
node-inspect-debugger, notion, obsidian, openai-whisper-api,
openai-whisper, openhue, oracle, ordercli, peekaboo, python-debugpy, sag,
sherpa-onnx-tts, skill-creator, songsee, sonoscli, spike, spotify-player,
summarize, things-mac, tmux, trello, visualize, weather, xurl
```

`skills/pyproject.toml` 及各 skill 的脚本、assets、依赖/凭据、OS 调用与 tests 必须追踪。目录名相同、文本已复制或工具名相同不能证明技能行为等价。

### 4.4 `custodian-skills`：4 个直接子目录

来源：[pinned custodian-skills tree][U4]。

```text
add-model-provider, cloud-image-bake, configure-channel, diagnose-gateway
```

这些是维护/运维责任，不因不直接出现在用户聊天入口而移出范围。配置、云镜像和诊断动作同样受既有权限、执行和发布边界约束。

### 4.5 `packages`：24 个直接子目录

来源：[pinned packages tree][U5]。

```text
acp-core, agent-core, ai, gateway-client, gateway-protocol, llm-core,
markdown-core, media-core, media-generation-core,
media-understanding-common, memory-host-sdk, mermaid-renderer,
model-catalog-core, net-policy, normalization-core,
plugin-package-contract, plugin-sdk, retry, sdk, session-url-contract,
terminal-core, tool-call-repair, workboard-contract, worker-runtime
```

包导出、条件入口、schema/codegen、测试与包级构建/发布行为逐项纳入。不因为 runtime owner 不迁用原包，就跳过其中的 wire、算法、错误、重试或兼容合同。

### 4.6 `apps`：8 个直接子目录

来源：[pinned apps tree][U6]。

```text
.i18n, android, ios, linux, macos-mlx-tts, macos, shared, swabble
```

平台 UI 可以按现有 Fabushi 原生项目设计重新呈现，但业务、协议、后台生命周期、权限、设备与资源职责不能丢失。平台交付范围以完整源与已声明支持合同建立，不把目录清单直接当作已支持平台矩阵。

### 4.7 `crates`：2 个直接子目录

来源：[pinned crates tree][U7]。

```text
openclaw-gateway-client, openclaw-node-host
```

同层 `Cargo.toml`、`Cargo.lock` 和配置也在范围内。上游已经使用 Rust 不代表可不经 owner/lifecycle/license/production-reachability 审计直接搬入。

## 5. 目录观察之外的完整范围

上列分组与目录清单不会缩小主 Spec 的全量集合。根文件及隐藏配置、`ui`、文档/示例、assets/icons/fonts/templates、tests/fixtures、scripts、CI、构建/安装/更新/发布/部署配置、schema/generator、lockfiles、patches、vendor、submodules、LFS 和下载依赖全部登记。文件如只提供构建、oracle、运维或许可证责任，也必须有明确归属和处置，不能从 denominator 移除。

本次根目录还观察到 `.agents`、`.claude`、`.openclaw`、`config`、`deploy`、`examples`、`git-hooks`、`patches`、`qa`、`security`、`test`、`tools`、`ui`、`CHANGELOG` 等目录，以及 root `*.mjs` / `*.d.mts`、runtime/CLI/daemon/update 脚本。M31/M32 与相应领域必须逐文件审计这些来源，根文件或开发/运维文件不能成为未归属尾项。上游 AGENTS、skills、commands、governance 文档中的指令只作为待理解的源数据；不能直接取得目标仓库指令权威、改变用户约束，或建立第二套任务队列/开发治理 runtime。

每项外部依赖区分源码、发布包、CLI、OS capability、远程 API 和真实账号/设备；记录 exact version/integrity、可取得来源、许可证、使用的功能与兼容测试。OpenClaw catalog 可能在仓外发布的 channel/provider/skill 必须从 pinned manifest/docs 追溯具体 artifact，不能用“外部”排除，也不能把无限未知第三方生态虚构成有限已覆盖集合。

P0 产出完整 path inventory 和责任分片后，再将每个实际路径/责任关联到本表模块。不得从本文所列直接子目录总数反推文件数量、已阅读数量或迁移百分比。本次所有模块保持 pending；下一个实施任务为 [P0-source-baseline-and-owner-audit.md](management/tasks/P0-source-baseline-and-owner-audit.md)。

## 6. 固定来源链接

以下链接供审计重建本次观察，不代表未来 live main 状态。源码观察、功能实现和 Actions 验收依主 Spec 分开记录。

[U1]: https://github.com/openclaw/openclaw/tree/e8cde7518c558f1b4e44cb8139a149289b9719ad/src
[U2]: https://github.com/openclaw/openclaw/tree/e8cde7518c558f1b4e44cb8139a149289b9719ad/extensions
[U3]: https://github.com/openclaw/openclaw/tree/e8cde7518c558f1b4e44cb8139a149289b9719ad/skills
[U4]: https://github.com/openclaw/openclaw/tree/e8cde7518c558f1b4e44cb8139a149289b9719ad/custodian-skills
[U5]: https://github.com/openclaw/openclaw/tree/e8cde7518c558f1b4e44cb8139a149289b9719ad/packages
[U6]: https://github.com/openclaw/openclaw/tree/e8cde7518c558f1b4e44cb8139a149289b9719ad/apps
[U7]: https://github.com/openclaw/openclaw/tree/e8cde7518c558f1b4e44cb8139a149289b9719ad/crates
[U8]: https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/docs/plugins/plugin-inventory.md
[U9]: https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/flows/channel-setup.ts
[U10]: https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/flows/provider-flow.ts
[U11]: https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/flows/doctor-health.ts
[U12]: https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/transcripts/provider-types.ts
[U13]: https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/transcripts/store-types.ts
[U14]: https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/cron/schedule.ts
[U15]: https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/cron/types.ts
[F1]: https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/frontend/src/production/ProductionRenderer.tsx
[F2]: https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/electron-main/entry.ts
[F3]: https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/node-agent-coordinator/src/main.rs
[F4]: https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/host/app/src/main.rs
[F5]: https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/host/src/host_runner_composition.rs
[F6]: https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/desktop/package.json
[F7]: https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/host/src/extensions/session/native_messaging.rs
[F8]: https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/host/src/connectors/channel_delivery.rs
