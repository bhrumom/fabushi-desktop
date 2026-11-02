# Fabushi 全量迁移执行规范：产品批次、及时反馈、最终完整验收

Status: active / normative execution addendum
Spec ID: FMEP-001
Revision: 2
Updated: 2026-11-02
Applies to: FBCP-001 / TDRP-001

## 1. 权威与目标

用户于 2026-11-02 明确批准采用“又快又好”的方式：按功能闭环成批实现，每批最小必要验证，最后集中完整验收。本 Revision 替代 Revision 1 的全量完成前零 CI、禁止实现分支 push/PR、只在阶段 B 执行等限制，并优先解释 FBCP-QA-19、TDRP-EXEC-01、旧任务与提示词。原文件名保留以兼容链接，不表示继续延期所有测试。

完整源责任、已有 Bot 能力、统一 UI/架构、服务端、安全、许可证、平台和最终验收范围不变。不承诺最快绝对工期；以减少重复工作、及时发现错误和可复用证据降低总成本。

## 2. 工作单位与阶段

阶段 A：固定 U，按现有 owner/依赖组织功能闭环，实施 → 写用例 → 冻结批次 H → Actions 最小必要验证 → 修复与接线。阶段 B：全范围实现闭合后冻结最终候选，集中完整回归与真实环境验收。阶段 C：独立 ACCEPT 后集成发布。

一批是可审查的用户行为或必要公共合同，不是一份源文件、一次提交或整场迁移。每批明确入口、状态权威、服务、持久化、消费者、失败/恢复路径及出口。基础合同先稳定，可独立 owner 并行，公共文件由单一集成人合入；支持 worker 时使用，不虚构并行。

## 3. 阶段 A 的最小必要验证

所有可执行验证仍只在 GitHub Actions；不在本机、htch-runtime 或其他环境运行。本次授权允许必要 push/PR/workflow_dispatch，沿用现有迁移线和测试体系。

每批根据变更和风险选择能发现错误的最小集合：受影响编译/typecheck，关键单元/合同/状态机，跨 owner 的接线集成。修改 UI 应做受影响交互/布局检查；修改权限、身份、存储、同步或迁移时，必须尽早覆盖相应越权、重复/乱序、崩溃/重启与兼容性负例，不为省时间推迟关键安全和数据风险验证。纯文档变更人工审查即可，不机械触发产品全量测试。

实现与测试同时写；批次达到可验证状态即冻结 H 运行，不要求全项目已完成或全部 verified。缺服务/设备时可先做合同验证，但真实环境覆盖保持 blocked/pending。阻塞独立批次不阻止其他工作；依赖已失败公共合同的批次不得继续把错误假设扩散。禁止静态断言/字符串匹配冒充行为验证。

## 4. 控制重复开销

先读现有 workflow 的事件、paths、依赖和缓存；复用现有 job/harness，必要时窄化路径或增加定向输入，不建立第二测试体系。不得每文件、每小提交执行全量 CI，也不得以全局停用 workflow、跳过 required checks 或伪造成功达成节流。

同 H、同测试/环境输入的有效通过结果复用；只重跑失败、缺证或已失效项。修改后冻结 H2，记录影响面并验证受影响合同/消费者；旧 H 结果保留来源，不冒充 H2 执行。批次期间只取消明确被替代且属于本任务的 run，不影响他人任务。不要重复轮询无变化状态；等待时推进独立工作。

## 5. 保存与合入

沿既有实现分支持续提交并及时推送可恢复云端检查点；检查触发范围，使保存和批次验证协调。draft PR 不等于隔离；skip 标记只适用于已核对事件的纯文档或不需执行检查点，不能绕过必需验证。不得 force push 覆盖他人工作或提前发布未验收产品。仅政策文档可独立合并，不作为产品验收。

## 6. 来源理解与目标设计

固定 exact upstream U，完整枚举递归依赖、构建期下载、资源、工具和法律来源。按目标功能批次理解相关文件、符号和调用关系，保留全量处置；不机械一文件一模块，不因浮动 dev 更新反复重开基线。blob/合同未变则复用理解，只补读影响范围。

先复用已有 Identity/Conversation/Transcript/Composer/Resource/Permission/Search 等 owner；新 owner 必须有最小 ADR。C++ 产品职责默认 Rust，React/TypeScript 承担 UI，平台最薄适配。保留所有现有 Agent/Host/Coordinator/Runner/Tools/Plugins/MCP/Computer/Automations 能力。具体收口要求见 §12。

## 7. 进度与证据

沿用单一 inventory、ledger 和 RTM，分别记录 understood、mapped、implemented、shipping-wired、tests-authored、execution 与 independent acceptance；不另造状态真相。局部通过只证明该 H 和所测范围，未执行项保持 not-run，不能将整个 responsibility 或产品提前 verified。

每批报告用户行为、实际 owner/入口、剩余缺口、H/U、所选检查及选择理由、run/job/artifact/结果、保存位置和下一批。工具治理与产品进度分开；分母不完整不报总百分比。失败保留记录，区分实现、用例/oracle、环境原因并修根因，不重跑挑绿。

## 8. 阶段 B 入口

全范围来源已理解/处置，适用职责已进入生产链，必要服务/数据/UI/平台/打包输入及测试已备齐，批次关键失败已解决后，冻结 Hfinal/U/依赖/工具链/环境。仍有来源或实现缺口时继续阶段 A；候选可验证不是 ACCEPT。

## 9. 集中完整验收

最终候选在 Actions 完成全部适用 required gates：来源/RTM、功能/状态机/合同/集成、真实服务/设备、跨平台包体、视觉/可访问性/语言/响应式、时序、权限安全、升级/回滚/恢复、性能/soak。相同 H 和输入的有效批次证据可复用并明确映射，不能省略未覆盖检查。每个平台/配置尽量构建一次、多 job 使用同一不可变构件。

复杂 Agent/tool 与消息流程覆盖 terminal、+2s、+8s、切换、重连、重载、重启的内容/ID/顺序/存储/DOM 一致性。真实审阅截图和视频；不以一个气泡或 marker 代替完整结果。保留取消竞态、乱序与旧快照回滚负例。

## 10. 修复与发布

修复后冻结新 H，先回归受影响项；最终 shipping SHA 必须取得全部适用门的确切证据，旧 PR 绿色不能替代合并 SHA。缺服务、签名、设备、来源证据或 required not-run/skipped/flaky 均不能算通过。独立评审 ACCEPT、0 open P0/P1/blocker 后才按既有保护与发布规则交付。

## 11. 执行入口与本次变更验收

主提示词沿用 `projects/fabushi-communication-platform/management/prompts/full-migration-deferred-ci.md`，质量时机见 `quality/execution-timing.md`。本次仅修订政策，人工审查 root AGENTS、两主 spec/SOT、P0 与主提示词的时机一致性；不改变产品 ledger 验收值，不运行产品构建测试。政策回滚通过普通 revert，不能抹除历史证据。

## 12. 产品责任优先与生产归属收口（2026-11-02）

本节落实用户要求：修正迁移方式并合并为 AI 执行依据。它细化阶段 A 的工作选择与实现判定，优先于旧提示词中“最早未闭合文件优先”的机械顺序；完整来源范围与最终验收标准不变。

### FMEP-OWNER-01 — 当前生产图决定目标

PR #20 是架构沿革，不是永久冻结的目标源码。每批绑定实际 base/working head，从 shipping entrypoint 跟踪 UI → command → owner → state/storage → service → event/projection；在既有 current-owner-inventory 和 owner-resolution-matrix 中增量记录路径、符号、权威状态、持久化、权限与调用者。旧 inventory/STATUS 只作有日期的历史证据；不得因旧表写“无 owner”直接新建。只补查当前批次和公共边界，不把全仓盘点反复作为开工前置。

### FMEP-OWNER-02 — 同一语义只有一个权威写入者

首先审计桌面 Session/Transcript/native_messaging、native/mahayana-messaging、FeatureHost MessagingService 和旧 Telegram Provider/FFI 的实际关系。目录名、Cargo 成员、存在测试、注释“desktop uses”均不能证明 shipping reachability。记录每个支持平台/构建配置的入口、注册条件和依赖路径；区分可调用、实际选用、仅研究/历史。

为 Identity、Conversation、Message、Resource、Permission、Search 明确唯一逻辑权威与写入入口；允许服务端权威、本地 durable outbox、离线状态、缓存和 UI 投影分层，但必须定义 ID/版本、幂等、排序、权限、冲突、重放和重启的合同，不要求所有状态挤进一个进程或文件。禁止仅通过重命名隐藏第二 owner，或让同一行为分别在桌面路径和孤立领域库重复实现。领域库只有经真实调用链复用后才计入产品接线。

### FMEP-OWNER-03 — 旧路径有明确退出条件

Telegram core/runtime/provider/network/storage 不作为本迁移的 shipping owner。先识别所有调用者、平台依赖和存量数据，再将适用行为迁入选定 Fabushi owner，补齐版本化数据迁移、失败恢复与回滚方案，最后移除旧注册、默认依赖和产品入口。不能因名字含 Telegram 就直接删库或用户数据，也不能因“遗留代码”无限保留第二运行路径。法律声明和研究 provenance 保留。若另有明确授权的互操作规范，单独标明适用范围，不把互操作算作原生融合完成。

### FMEP-FAST-08 — 完整核算，产品批次驱动

所有源文件继续完整理解并有处置。产品行为、构建/平台/安全/许可证必要支撑、上游开发辅助工具分别记账；沿用一个 inventory/ledger/RTM，用兼容字段或既有 notes 标注类别，不建第二套真相。

上游 .agents、审查提示词、代码健康工具不是 Fabushi 产品需求的自动来源。理解后应说明复用现有机制、必要适配或经证据说明不适用；不能仅因源仓有该文件就改 root AGENTS、复制治理体系或制造 Rust 对应物。已有明确必迁责任不得偷偷改成不适用；冲突必须在当前规范/ADR 中解决。工具工作仅在解除具体产品、构建、安全或最终验收阻塞时优先，并指出被解锁的能力。

按依赖、用户价值和风险选择下一批，不按文件名字典序：先权威模型与生产链收口，再消息/同步/恢复与统一 UI，再群频道/权限/搜索、资源媒体/通话等可并行域；Stories、商业、平台等全部适用职责仍必须完成。每批交付实际 owner 修改、调用接线、数据/错误恢复合同和已编写用例。纯阅读批次如确有必要，记录它解锁的下一生产批次，不冒充功能实现。

### FMEP-STATE-01 — 进度不得混算

分别报告来源理解、产品职责实现、shipping 接线、测试编写、执行与独立接受。治理文档条数不得进入产品完成率；分母尚未完整分解时不得给全产品百分比。单纯政策说明不能把可执行工具职责标 implemented；独立 helper 或领域模型完成但 UI/服务调用未接入时，保留局部实现说明和 wiring gap，不标整个产品责任 implemented。历史行逐项纠正且保留原记录，不批量升级状态。

每批更新既有 STATUS 和 ledger 的相关项、exact head 与未决问题；计数、表头 revision、行状态不一致时明确修正来源。未执行仍 not-run；本规范变更不修改任何产品验收值。

### 首个生产批次与退出标准

下一实现批次从 OWNER-01/02 的三条消息路径及旧 Telegram 注册开始，复用现有迁移线。交付一份写入既有 owner matrix 的路径/权威/调用者差异，以及相应生产收口实现和用例；禁止再次只扩充治理报告。发现跨仓服务依赖时同步追踪服务 owner 和接口版本，缺少源码或权限明确阻塞，继续其他独立能力。

阶段 A 人工审查必须能回答：实际用户入口在哪里；哪个 owner 决策；什么状态被持久化；谁发送/同步；谁消费事件；失败/重启怎么恢复；旧路径何时退出。缺一项则保留该项缺口。阶段 B 再在 Actions 证明支持配置无旧 Telegram 原生路径、同一消息跨 UI/服务/持久化的 ID 与状态一致、重试重启及数据迁移可恢复、现有 Agent 能力不回退。阶段 A 按批次做针对性验证，阶段 B 完成全范围验收；不能把政策合并当产品完成。


## 13. FMEP-SOURCE-01 — 全量核算、风险分层阅读（2026-11-02）

本条替代旧规范对全部文件同等深读与全局 unread=0 的要求；完整产品能力范围不变。每个固定 U 来源项必须有 repository/commit/path/blob 的明确身份及处置，但工作单位是产品责任/功能批次。机器枚举是来源覆盖证据，不是语义理解证明。

- product / mixed-ui：深读适用产品逻辑、调用关系、状态、异常、权限、并发和恢复；逐责任映射到实际 owner。高风险、安全关键或不确定项默认 deep，不能抽样代替。
- generated：读生成器、输入/schema 和输出合同，核对生成关系；输出按精确成员清单成组处置，不逐行重复读。手工补丁/未知差异升级 deep。
- dependency：核对锁定版本、实际使用/API、构建可达性、配置/补丁、安全边界与许可证；关键或修改部分 deep，其余 contract。不要求重写通用成熟依赖。
- resource / documentation / tooling：按内容用途采用 disposition；工具不自动成为产品需求。不得仅凭目录或扩展名判断无产品责任；存在疑问升级 deep。

在原 ledger 增加 source_reviews，记录精确 source identity、category、risk、review_depth、status、rationale、evidence、responsibility_ids。允许同一审查证据被多个精确成员引用；禁止用目录通配符假装覆盖。complete 必须有真实证据；产品项必须 deep 且关联产品责任，high 风险必须 deep。不读取而作合同/处置审查不标成 read-complete。历史 unread/unknown 保留其原统计含义，不自动清零。

新完成门为来源无遗漏、分类无未知、必需深读无缺口、产品责任全映射/实现/接入且验收通过。accepted 检查必须以实际 inventory identity 集合对照 source_reviews，不能信任自报零计数。外部下载/LFS/构建依赖仍按原闭合门另行验证；recursive inventory 通过不代表外部来源自动闭合。缺 source_reviews 的旧账本可继续作为未接受候选，最终接受必须迁移到新合同。

责任账本使用核心字段加适用域合同；UI/search/design 字段仅对相应责任适用，不适用要有类别与理由，不能拿空值隐藏产品义务。现有必填字段在 schema/validator 完成对应条件化之前继续兼容，不批量伪造字段或完成状态。每批只做本批必要文档，复用共享合同。

本次验收：在 Actions 以正例证明 contract/disposition 可闭合非产品来源，以负例拒绝缺项、重复、blob 漂移、低深度产品/高风险、无证据和未映射责任。执行结果只证明检查器合同，不证明现有 16k 来源已完成。

accepted 模式不能沿用历史审计的“必须 open”断言。构建来源的必要状态门使用明确正值 `complete`：lock 的 acquisition closure/disposition/recursive-authority、实际 acquisition 的 discovery disposition/recursive-authority/evidence、current-source 与 recursive-build-reachability 均须为 `complete`；pending-classification 和 pending-external-scan 计数须为整数零。acquisition 的 repository/commit、reachability 的 upstream commit 必须对应固定 U；任何保留的 rebaseline_candidate（含 null/false）、候选 JSON pointer 或明确为 false 的 generated/LFS/legal 完整性声明都拒绝。未知/缺失状态不能按“非 open”推断闭合。这个状态 guard 仅提供 accepted 的必要条件，不替代精确来源审查、外部对象/生成关系/许可证证据、现有网络核验或独立接受；当前候选及其 null unknown/unread 合同保持不变。

accepted 状态门的行为证据必须包含现有 authority entrypoint 的子进程负例：在移除 ledger 候选标记并构造符合其他前置条件的 fixture 后，仍因实际 acquisition 未闭合而失败，不能仅凭直接调用 helper 的用例宣称生产检查入口已接入。此用例只在 GitHub Actions 执行。
