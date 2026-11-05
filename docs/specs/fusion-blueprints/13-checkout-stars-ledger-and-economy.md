# Fabushi 统一融合蓝图 13：收银台第三方支付结算、虚拟星币账本与礼物经济系统
# Volume 13: Checkout, Stars Ledger, Invoices, Gifts, and Economy Infrastructure

> **规范编号**: `FBCP-BLUEPRINT-13` / `UFX-BLUEPRINT-13`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. 彻底根除全端虚假星标余额：消灭 `TdrpProfileDetailsDrawer.tsx:29` 硬编码 `1250 Stars`、`TdrpStoriesAndStarsBar.tsx:37` 孤立假状态 `1280 Stars`、以及 `TdrpConversationOverlay.tsx:77-110` 中使用 `setStarsBalance((b) => b +/- amount)` 和 `setTimeout` 制造的假成功回显。
> 2. 贯彻落地 `FBCP-ADR-002-payment-settlement.md` 架构决议：确立权威 `PaymentSettlement` 与 `StarsLedger` 基础设施所有者，坚决剥离于 Settings、Transcript 聊天历史、Session 或 Automations，建立独立的金融状态机与审计追溯。
> 3. 完整打通 Rust Coordinator 权威服务：消除前后端 RPC 契约脱节，将前端调用全面对齐至 `source/node-agent-coordinator/src/tdrp/stories_gifts_stars.rs`（`stars.*`、`gifts.*`）与 `source/node-agent-coordinator/src/tdrp/export_payments_bots.rs`（`payments.*` 发票与收银流水）。
> 4. 实现无缝收银台与发票生命周期：支持发票（Invoice）卡片在消息流中原生渲染、第三方支付提供商签约集成、带签名的支付凭证（Receipt）归档与 CAS 乐观并发重试。
> 5. 严格多账号资产隔离：以 `accountScopeGeneration` 单调递增为屏障，实现切账号即刻清空星币缓存、取消挂起支付轮询与保护敏感财务数据，杜绝账户间资产穿透。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照矩阵

| 功能项与能力域 | OpenClaw 锁定源码锚点 | Telegram Desktop 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **虚拟星币权威账本**<br>(TCG-N29 / Credits Ledger) | `src/gateway/client.ts`<br>`src/agents/cost-ledger.ts`<br>`src/security/quota-gate.ts` | `Telegram/SourceFiles/api/api_credits.cpp`<br>`data/data_credits.cpp`<br>`boxes/credits_box.cpp` | `source/node-agent-coordinator/src/tdrp/stories_gifts_stars.rs:32-38,178-200,414-447`<br>`frontend/src/production/tdrp/TdrpProfileDetailsDrawer.tsx:29,160-180` | `starsLedgerStore`<br>`source/node-agent-coordinator/src/tdrp/stories_gifts_stars.rs` |
| **礼物赠送与转让流转**<br>(TCG-N29 / Star Gifts) | `src/agents/identity.ts`<br>`ui/src/pages/identity/profile.ts` | `Telegram/SourceFiles/data/data_star_gift.cpp`<br>`boxes/star_gift_box.cpp`<br>`api/api_star_gift.cpp` | `source/node-agent-coordinator/src/tdrp/stories_gifts_stars.rs:40-51,242-295,483-539`<br>`frontend/src/production/tdrp/TdrpStarsModal.tsx:1-199`<br>`frontend/src/production/tdrp/TdrpConversationOverlay.tsx:89-110` | `StarGiftsDockModal.tsx`<br>`stories_gifts_stars.rs`（CAS 原子结算） |
| **收银结账与发票结算**<br>(TCG-N31 / ADR-002 Checkout) | `packages/gateway-protocol/`<br>`src/infra/accounting.ts` | `Telegram/SourceFiles/payments/payments_checkout_box.cpp`<br>`payments/payments_form.cpp`<br>`api/api_payments.cpp` | `docs/adr/FBCP-ADR-002-payment-settlement.md`<br>`source/node-agent-coordinator/src/tdrp/export_payments_bots.rs:33-83,168-287,358-406` | `paymentSettlementStore`<br>`InvoiceCheckoutModal.tsx`<br>`export_payments_bots.rs` |
| **消息流发票与凭证卡片**<br>(TCG-N31 / Invoice Message) | `ui/src/components/chat/card.ts` | `history/view/controls/history_view_invoice_button.cpp`<br>`data/data_invoice.cpp` | `frontend/src/recovered/features/conversation/cards/transcript-card/views/widget.tsx`<br>`source/node-agent-coordinator/src/tdrp/export_payments_bots.rs:41-48,270-287` | `InvoiceTranscriptCard.tsx`<br>`paymentSettlementStore` |
| **会员权益门禁与限额**<br>(TCG-N28 / OCRM-M10 Quota) | `src/security/approvals.ts`<br>`OCRM-M10`（配额门禁） | `Telegram/SourceFiles/api/api_premium.cpp`<br>`settings/settings_premium.cpp`<br>`data/data_user.cpp` | `source/electron-main/account/human-identity.ts`<br>`source/host/src/runner/sand_computer_auto_review.rs:20-45` | `entitlementsStore`<br>`source/host/src/extensions/session/` |
| **充值档位与支付对账**<br>(TCG-N31 / Recharge & Reconcile) | `src/gateway/gateway_request_dispatcher.rs` | `payments/payments_panel_data.h` (`LabeledPrice`)<br>`api/api_authorizations.cpp` | `frontend/src/production/tdrp/TdrpStarsModal.tsx:22-27`<br>`source/node-agent-coordinator/src/tdrp/export_payments_bots.rs:35-38,219-224` | `StarsRechargeModal.tsx`<br>`export_payments_bots.rs` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **Telegram Desktop 侧**：
  - 虚拟星币（Telegram Stars）由 `api/api_credits.cpp` 与 `data/data_credits.cpp` 统管，提供原子余额读取（`Api::CreditsTopup`）、消费扣除与对账记录（`Data::CreditsStatus`）。
  - 星礼系统由 `boxes/star_gift_box.cpp` 与 `data/data_star_gift.cpp` 驱动，支持指定接收人、留言附言、匿名赠送（`hide_my_name`）、以及是否允许接收方折现转换为等值星币（`convert_stars`）。
  - 收银发票由 `payments/payments_checkout_box.cpp`、`payments_form.cpp` 与 `api/api_payments.cpp` 实现，采用 `LabeledPrice` 细分项目列表与 `Receipt` 票据机制。客户端通过安全 Webview 或三方 SDK 完成支付，客户端绝不触碰银行卡或私密凭据。
- **OpenClaw 侧**：
  - 由 `src/gateway/`、`src/agents/cost-ledger.ts` 与 `src/security/quota-gate.ts`（OCRM-M10）驱动，对 Token 消耗、工具调用与算力租赁做配额扣减与用量审计；配额不足时直接抛出 `QUOTA_EXHAUSTED`，阻止任务继续执行。

#### Q2（现有链路定位）
- **现有前端至后端链路断裂现状**：
  - 前端抽屉展示：`frontend/src/production/tdrp/TdrpProfileDetailsDrawer.tsx:29` 声明 `starsBalance = 1250` 默认参数，直接在界面展示死数据。
  - 前端孤岛栏：`frontend/src/production/tdrp/TdrpStoriesAndStarsBar.tsx:37` 独立定义 `starsBalance = 1280`，虽尝试调用 `stars.status`，但该组件根本未在 `ProductionRenderer.tsx` 中被引入挂载，沦为孤岛。
  - 前端模态框操作：`TdrpConversationOverlay.tsx:81` 在充值时调用 `client.call("stars.recharge", { amount })`，然而后端的真实 RPC 路由名称是 `stars.purchase`（`stories_gifts_stars.rs:425`）。
  - 礼物赠送参数严重不一致：`TdrpConversationOverlay.tsx:98-103` 传递 `{ peerId, giftName, hideMyName, convertStars }`，但后端 `stories_gifts_stars.rs:485-502` 强制校验 `{ senderId, recipientId, title, starsPrice }`，缺少必要参数直接抛出 `MISSING_ARGUMENT` 失败。
  - 本地伪造状态：`TdrpConversationOverlay.tsx:83,106` 使用 `setTimeout` 在 2.5 秒后清除 `transferStatus` 字符串，并在未等待 RPC 成功返回前强行执行 `setStarsBalance((b) => b - cost)`，脱离服务端真相。

#### Q3（断点精确诊断）
1. **硬编码假余额断点**（`TdrpProfileDetailsDrawer.tsx:29,177`）：
   - `starsBalance = 1250` 硬编码打桩，在用户资料抽屉的星标卡片中渲染固定数值，与用户真实资产完全无关。
2. **RPC 协议断头断点**（`TdrpConversationOverlay.tsx:81,98-103` vs `stories_gifts_stars.rs:425,485-502`）：
   - 方法名冲突：前端请求 `stars.recharge`，后端仅监听 `stars.purchase`，导致充值请求 100% 报错 `UNKNOWN_STARS_METHOD`；
   - 参数键名错配：前端传 `peerId`，后端要求 `recipientId` 与 `senderId`；前端传 `giftName`，后端要求 `title`；前端未传 `starsPrice`，后端直接断言错误。
3. **缺少 ADR-002 规范的收银台集成断点**（`docs/adr/FBCP-ADR-002-payment-settlement.md`）：
   - 后端 `export_payments_bots.rs:51-83,168-287` 已完整实现 `PaymentInvoice`、`LabeledPrice`、`PaymentReceipt` 与 `settle_payment`，但前端没有任何收银台组件（`CheckoutBox`）或发票卡片组件连接该接口。
4. **缺少跨账号资产围栏断点**（`TdrpConversationOverlay.tsx:40`）：
   - `starsBalance` 仅存储在组件局部 React State 中，切换用户账号（`accountScopeGeneration` 改变）后未执行重置与拉取，前一账号的余额残留显示在后一账号界面。

#### Q4（状态机与安全防护）
- **经济结算确定性有限状态机**：
  $$\mathcal{M}_{\text{economy}} = \langle S, \Sigma, \delta, s_0 \rangle$$
  - 账本状态 $S \in \{\text{Unloaded}, \text{Synchronizing}, \text{Ready}(\text{balance}), \text{Transacting}(\text{opId}, \text{balance}), \text{Settled}, \text{Failed}(\text{code})\}$
  - 发票状态 $S_{\text{inv}} \in \{\text{Draft}, \text{Created}, \text{PendingPayment}, \text{Processing}, \text{Paid}(\text{receiptId}), \text{Expired}, \text{Cancelled}\}$
  - 必须采用 CAS（Compare-And-Swap）乐观锁校验：每次扣减必须携带客户端已知的 `balance_version`；若服务端版本已递增，必须退避并重新拉取最新余额，杜绝并发双花。
- **幂等性与防护围栏（Fencing）**：
  - 每一笔星币转移、礼物赠送与外部充值必须生成全局唯一 `idempotency_key = UUIDv4`。
  - 严格绑定 `accountScopeGeneration`：当 generation 变化时，立即丢弃所有在途查询响应，清空前端账本缓存，终止正在轮询的发票状态任务。

#### Q5（UI/UX 极致融合落点）
- **一级直达与渐进披露**：
  - 高频展示：在右侧上下文抽屉 `ProfileTab` 顶部渲染精美的星标余额金色徽章（Stars Badge），点击直达充值弹窗；
  - 动作内嵌：在主输入台（Composer）动作菜单与群成员操作栏中集成“送礼（Send Star Gift）”与“赞赏（Tip）”按钮；
  - 发票卡片：聊天流中渲染原生 `InvoiceTranscriptCard`，展示项目明细、含税总价与一键“安全支付”按钮；点击唤起无缝嵌入式 `InvoiceCheckoutModal`，支持选择支付渠道并展示实时凭证。
  - 设计语言：100% 消费 `--fabushi-*` 官方语义 Token，无缝支持高对比度 WCAG AA 标准。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-13-01** | TdrpProfileDetailsDrawer 硬编码假星标余额 1250 | `frontend/src/production/tdrp/TdrpProfileDetailsDrawer.tsx:29,176-179` | 原型打桩遗留。默认参数 `starsBalance = 1250` 直接渲染，脱离真实账户资产。 | **P0 严重虚假资产** |
| **BUG-13-02** | TdrpStoriesAndStarsBar 孤岛组件假状态 1280 与未挂载 | `frontend/src/production/tdrp/TdrpStoriesAndStarsBar.tsx:37,68-75` | 孤立原型代码。写死初始状态 1280，且未被 `ProductionRenderer.tsx` 引入装配，处于失联状态。 | **P1 孤岛代码** |
| **BUG-13-03** | TdrpConversationOverlay 本地乐观篡改与假定时器回显 | `frontend/src/production/tdrp/TdrpConversationOverlay.tsx:77-110` | 假装成功反模式。前端在无服务端确认下执行 `setStarsBalance` 并使用 `setTimeout` 伪造 2.5 秒提示消失，发生网络错误时导致账实不符。 | **P0 数据完整性破坏** |
| **BUG-13-04** | 前后端 RPC 方法名与入参严重错配（断头调用） | `frontend/src/production/tdrp/TdrpConversationOverlay.tsx:81,98-103`<br>`source/node-agent-coordinator/src/tdrp/stories_gifts_stars.rs:425,485-502` | 接口契约从未对齐。前端请求 `stars.recharge` vs 后端 `stars.purchase`；前端传 `peerId/giftName` vs 后端要求 `senderId/recipientId/title/starsPrice`。 | **P0 核心链路中断** |
| **BUG-13-05** | 缺乏 ADR-002 权威结算所有者，商业逻辑无处安放 | `docs/adr/FBCP-ADR-002-payment-settlement.md`<br>`source/node-agent-coordinator/` | 缺少独立的 `PaymentSettlement` 领域模型与 Store，支付与星币逻辑被随意散落于 UI 抽屉中，违反单一职责原则。 | **P1 架构治理缺陷** |
| **BUG-13-06** | export_payments_bots 完整发票引擎未接入前端消息流 | `source/node-agent-coordinator/src/tdrp/export_payments_bots.rs:51-83,168-287`<br>`frontend/src/recovered/features/conversation/cards/` | 后端实现了高标准的 Invoice 与 Receipt 状态机，但前端完全缺少发票卡片与收银台交互组件，能力被废弃。 | **P1 能力未闭环** |
| **BUG-13-07** | 跨账号切换（accountScopeGeneration）未重置财务缓存 | `frontend/src/production/tdrp/TdrpConversationOverlay.tsx:40`<br>`ProductionRenderer.tsx:4337-4347` | 资产状态未绑定账号作用域世代，用户退出登录或切换账号后仍显示前一用户的星币余额与送礼记录，存在敏感财务泄露风险。 | **P0 安全隔离漏洞** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 ADR-002 统一经济与支付结算拓扑架构

根据 `docs/adr/FBCP-ADR-002-payment-settlement.md` 的核心决议，支付结算必须作为独立的基础设施所有者（PaymentSettlement Infrastructure Owner），与会话历史、配置面板、自动化调度解耦：

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   ProductionRenderer (Root Application Shell)                         │
├────────────────────────────────┬──────────────────────────────────────┬────────────────────────────────┤
│ 1. 统一上下文抽屉 (ProfileTab) │ 2. 聊天主舞台 (Message Transcript)   │ 3. 权威输入台 (Composer)       │
│  - StarsBalanceCard 金色徽章   │  - InvoiceTranscriptCard 发票卡片    │  - 动作抽屉: 赞赏/送礼 Action  │
└────────────────────────────────┴──────────────────────────────────────┴────────────────────────────────┘
                                                 │
                                                 ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                         Canonical Economy Stores (frontend/src/recovered/features/economy/)            │
├───────────────────────────────────────────────┬────────────────────────────────────────────────────────┤
│ A. starsLedgerStore (星币账本状态机)          │ B. paymentSettlementStore (收银结算与发票流转)         │
│  - balance / balanceVersion (CAS 校验)        │  - activeInvoices / checkoutSession                    │
│  - transactionsLedger[] (不可变流水日志)      │  - createInvoice / initiateCheckout / pollReceipt      │
│  - purchaseStars / sendGift / transferStars   │  - idempotencyKeyManager / accountScopeGeneration Fence│
└───────────────────────────────────────────────┴────────────────────────────────────────────────────────┘
                                                 │
                                                 │ Typed RPC via ProductionCoordinatorClient
                                                 ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│               Rust Node Agent Coordinator (source/node-agent-coordinator/src/tdrp/)                    │
├───────────────────────────────────────────────┬────────────────────────────────────────────────────────┤
│ stories_gifts_stars.rs                        │ export_payments_bots.rs                                │
│  - stars.balance / stars.purchase             │  - payments.createInvoice                              │
│  - stars.transfer (CAS 原子转账)              │  - payments.settlePayment (三方收银核销)               │
│  - gifts.send / gifts.listReceived            │  - payments.getReceipt (不可变票据生成)                │
└───────────────────────────────────────────────┴────────────────────────────────────────────────────────┘
                                                 │
                                                 ▼
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│             Durable Storage & Audit Ledger (source/host/src/storage/store_db.rs / SQLite WAL)          │
│  - star_ledger 表 (entry_id, user_id, amount, reason, timestamp_ms)                                     │
│  - payment_invoices 表 (invoice_id, amount_cents, currency, is_paid, provider_charge_id, receipt_json) │
│  - received_gifts 表 (gift_id, sender_id, recipient_id, title, stars_price, message, is_anonymous)     │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 模块拆分与 200–600 行预算明细表（Same Owner != Same File）

严格遵循单文件 200–600 行内聚预算，在 `frontend/src/recovered/features/economy/` 目录下建立高内聚模块拓扑：

| 模块文件路径 | 预估行数 | 核心内聚职责与边界 | 替代的原有割裂组件 |
|---|---|---|---|
| `frontend/src/recovered/features/economy/model.ts` | 180–240 行 | 经济域核心类型定义：星币流水、礼物实体、发票详情、收银会话契约与 RPC 请求响应定义 | 分散在各组件内的临时类型 |
| `frontend/src/recovered/features/economy/stars-ledger-store.ts` | 260–340 行 | 虚拟星币全局状态管理：余额订阅、CAS 乐观并发校验、流水拉取、转账与礼物流转状态机 | `TdrpStoriesAndStarsBar.tsx`（星币部分）<br>`TdrpConversationOverlay.tsx:77-110` |
| `frontend/src/recovered/features/economy/payment-settlement-store.ts` | 280–360 行 | ADR-002 收银台与发票状态机：发票开具、收银通道选择、支付轮询、凭证确认与撤销 | 缺失的 ADR-002 前端存储层 |
| `frontend/src/recovered/features/economy/components/StarsBalanceCard.tsx` | 200–260 行 | 统一星标余额卡片组件：支持黄金渐变质感、当前余额动态格式化、一键唤起充值与送礼明细 | `TdrpProfileDetailsDrawer.tsx:160-180` |
| `frontend/src/recovered/features/economy/components/StarsRechargeModal.tsx` | 240–320 行 | 星币快速充值模态框：阶梯档位选择、法币折算、支付渠道拉起、防重复点击与加载状态 | `TdrpStarsModal.tsx:112-135` |
| `frontend/src/recovered/features/economy/components/StarGiftsDockModal.tsx` | 280–380 行 | 虚拟礼物挑选与打赏面板：礼物网格、折现选项、匿名赠送开关、个性化留言与余额即时校验 | `TdrpStarsModal.tsx:136-196` |
| `frontend/src/recovered/features/economy/components/InvoiceCheckoutModal.tsx` | 320–440 行 | 权威发票收银台弹窗：展示 LabeledPrice 明细清单、小费选择、支付方式切换、安全加密提示 | 缺失的前端收银台界面 |
| `frontend/src/recovered/features/conversation/cards/InvoiceTranscriptCard.tsx` | 240–320 行 | 消息流中内联渲染的发票卡片：状态标签（待付/已付/已退款）、总价高亮、一键支付响应 | 缺失的发票卡片渲染器 |


---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 渐进披露（Progressive Disclosure）与金融交互信息架构

为消除原系统随意散落的硬编码假余额与弹窗断层，本蓝图确立严格的四级金融交互渐进披露体系：

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ Layer 1: 顶部状态与上下文资产徽章 (Hero Balance & Quick Top-up)                                  │
│  - 在 ContextInspector 资料卡片及个人设置中展示金色星标胶囊 (Stars Badge)                       │
│  - 显示权威格式化数字 (如 2,450 Stars)；提供轻量一键“充值 (+)”按钮直达 Layer 2                  │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Layer 2: 场景化微交易抽屉与快捷操作 (Contextual Micro-Action Docks)                             │
│  - Composer 输入台右侧动作区集成【赞赏 / 送礼】Pill 按钮                                         │
│  - 呼出轻量级 StarGiftsDockModal：网格化呈现 3D 拟物礼物（咖啡/赞赏/限量火箭等）                 │
│  - 附带匿名赠送开关 (hideMyName) 与折现支持开关 (convertStars)                                   │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Layer 3: 原生发票卡片与全功能收银台 (Native Invoices & Multi-Provider Checkout)                  │
│  - 消息流中渲染结构化 InvoiceTranscriptCard：标题、详细价格清单 (LabeledPrice)、税费、最终总额 │
│  - 点击【安全支付】唤起居中拟态收银台 (InvoiceCheckoutModal)：                                  │
│    支持选择银行卡、Stripe、第三方移动支付或 Stars 快捷抵扣；集成小费 (Tip) 动态计算器           │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Layer 4: 不可变票据凭据与审计对账中心 (Immutable Receipts & Audit History)                        │
│  - 支付完成展示带数字签名的不可变凭据卡片 (PaymentReceipt)：包含流水号、商户信息与结算时间戳     │
│  - 提供全量星币交易流水明细视图 (StarsLedgerHistoryModal)，支持按收支、送礼、充值多维筛选与导出 │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.2 响应式三阶布局断点与视口自适应

严格对齐 `FSP-001 §17` 与全平台响应式规范，确保在任何屏幕尺寸下均具备无损支付体验：

1. **宽屏桌面端（视口宽度 $\ge 1200\text{px}$）**：
   - `StarsBalanceCard` 常驻于右侧 `ContextInspector` 顶部，点击充值在屏幕正中央展开 480px 宽度的 `StarsRechargeModal`；
   - `InvoiceCheckoutModal` 宽度采用 540px 居中模态窗口，左侧展示商品与价格清单，右侧展示安全支付选项与二维码/表单；
   - 消息流中的发票卡片（`InvoiceTranscriptCard`）最大宽度限制为 420px，与富媒体消息气泡宽度保持严格一致。
2. **中屏平板/紧凑桌面（$900\text{px} \le \text{视口宽度} < 1200\text{px}$）**：
   - 抽屉折叠时，顶栏状态区显示紧凑型星标图标与余额徽标；
   - 收银台与充值弹窗自动收敛为单一垂直流（Single Column），优先展示总金额与默认推荐支付通道，次要支付渠道采用手风琴式折叠；
   - 点击遮罩或按 `Escape` 键平滑退出，焦点自动返还至点击前触发按钮。
3. **窄屏移动端（视口宽度 $< 900\text{px}$）**：
   - 收银台与送礼面板升级为**沉浸式底部抽屉（Bottom Sheet）**，具备顶部防滑手柄（Drag Handle），支持向下滑动手势快速关闭；
   - 底部常驻高度为视口的 85%，支持虚拟键盘弹起时表单输入框自适应向上推挤（Scroll-into-view），避免遮挡安全确认按钮；
   - 页面底部保留至少 24px 安全边距（Safe Area Inset Bottom）。

---

### 4.3 语义设计系统变量（Design Tokens）与深浅色模式（WCAG AA）

100% 绑定官方 `--fabushi-*` 语义 Token，严禁任何硬编码十六进制色值或平行样式库：

```css
/* 官方星币资产卡片与金色徽章规范 */
.fabushi-stars-badge {
  background: linear-gradient(135deg, var(--fabushi-gold-subtle, rgba(234, 179, 8, 0.12)), rgba(249, 115, 22, 0.08));
  border: 1px solid var(--fabushi-badge-border-gold, rgba(234, 179, 8, 0.32));
  color: var(--fabushi-text-primary, #1c1c1e);
  border-radius: var(--fabushi-radius-full, 9999px);
  padding: 4px 10px;
}

.fabushi-stars-badge__icon {
  color: var(--fabushi-gold-primary, #eab308);
  fill: var(--fabushi-gold-primary, #eab308);
}

/* 原生发票卡片规范 */
.fabushi-invoice-card {
  background-color: var(--fabushi-bg-surface-elevated, #ffffff);
  border: 1px solid var(--fabushi-border-weak, rgba(128, 128, 128, 0.16));
  border-radius: var(--fabushi-radius-card, 14px);
  box-shadow: 0 4px 20px var(--fabushi-shadow-card, rgba(0, 0, 0, 0.06));
}

.fabushi-invoice-card__price-total {
  font-size: 20px;
  font-weight: 700;
  color: var(--fabushi-text-emphasis, #000000);
}

/* 支付状态标签规范 */
.fabushi-invoice-status--paid {
  background-color: var(--fabushi-status-success-bg, rgba(39, 174, 96, 0.12));
  color: var(--fabushi-status-success-fg, #27ae60);
  border: 1px solid var(--fabushi-status-success-border, rgba(39, 174, 96, 0.3));
}
```

- **WCAG AA 达标要求**：
  - 金色文本在深浅两套主题下均通过算法加深加暗，保持文本对比度 $\ge 4.5:1$；
  - 待支付（黄色）、已支付（绿色）、已取消/失败（红色）状态必须由**文本标签 + 图标 + ARIA 属性**三重表征，坚决禁止仅凭色彩传递结算状态。

---

### 4.4 支付安全防误触、全键盘无障碍与输入法（IME）保护

1. **金融级防误触与防重放保护**：
   - 所有结算与送礼按钮内置**单飞防双击保护（Single-Flight Lock）**：点击后立即进入不可逆的 Loading 态，并自动禁用（`disabled={isPending}`）；
   - 快速连续点击防护：内置 800ms 防抖节流，杜绝用户因网络卡顿重复点击导致发送多次扣款请求；
   - 针对大于等于 1,000 Stars 或等值 100 元法币的大额支付，必须弹出二次滑动或弹窗确认（Two-Step Confirm）。
2. **全键盘无障碍导航（A11y）**：
   - 收银台与充值弹窗遵循 WAI-ARIA Dialog 规范，容器附带 `role="dialog"`, `aria-modal="true"`, `aria-labelledby="checkout-title"`；
   - 激活焦点陷阱（Focus Trap），`Tab` 键在弹窗内部各输入框与支付按钮间循环；
   - 按 `Escape` 键立即呼出确认放弃提示，防止误关导致表单状态丢失；若确认退出，焦点精准返还至唤起按钮。
3. **中文输入法（IME）保护**：
   - 送礼附言输入框与自定义打赏金额输入框，监听 `onCompositionStart` 与 `onCompositionEnd`；
   - 在 `isComposing === true` 期间，按 `Enter` 键选词严格被事件捕获拦截，绝不误触支付表单提交或赠送动作。

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 前端核心 TypeScript 类型契约（Zero Any）

```typescript
/** 官方星币资产快照 */
export interface StarBalanceSnapshot {
  readonly userId: string;
  readonly balance: number;
  readonly balanceVersion: number;
  readonly lastSyncedAtMs: number;
}

/** 官方星币不可变账本明细条目 */
export interface StarLedgerEntryItem {
  readonly entryId: string;
  readonly userId: string;
  readonly amount: number; // 正数为增加，负数为扣减
  readonly reason: string;
  readonly timestampMs: number;
}

/** 虚拟礼物定义与实体 */
export interface StarGiftItem {
  readonly giftId: string;
  readonly senderId: string;
  readonly recipientId: string;
  readonly title: string;
  readonly starsPrice: number;
  readonly message?: string;
  readonly isAnonymous: boolean;
  readonly sentAtMs: number;
}

/** 充值预设档位 */
export interface StarPurchaseTier {
  readonly amount: number;
  readonly priceDisplay: string;
  readonly priceCents: number;
  readonly currency: string;
  readonly badgeLabel?: string;
}

/** 发票细分项目清单 (对标 upstream LabeledPrice) */
export interface LabeledPriceItem {
  readonly label: string;
  readonly amountCents: number;
}

/** 支付不可变凭据 (对标 upstream PaymentReceipt) */
export interface PaymentReceiptSnapshot {
  readonly receiptId: string;
  readonly paidAtMs: number;
  readonly totalAmountCents: number;
  readonly currency: string;
  readonly providerName: string;
}

/** 发票实体快照 (对标 upstream PaymentInvoice) */
export interface PaymentInvoiceSnapshot {
  readonly invoiceId: string;
  readonly title: string;
  readonly description: string;
  readonly currency: string;
  readonly totalAmountCents: number;
  readonly isPaid: boolean;
  readonly providerPaymentChargeId?: string;
  readonly prices: readonly LabeledPriceItem[];
  readonly suggestedTipAmounts: readonly number[];
  readonly maxTipAmount: number;
  readonly isRecurring: boolean;
  readonly needName: boolean;
  readonly needPhone: boolean;
  readonly needEmail: boolean;
  readonly needShippingAddress: boolean;
  readonly isTest: boolean;
  readonly receipt?: PaymentReceiptSnapshot;
  readonly createdAtMs: number;
}

/** 收银结算会话状态 */
export type CheckoutSessionStatus =
  | "idle"
  | "preparing"
  | "awaiting_payment"
  | "verifying"
  | "settled"
  | "failed"
  | "cancelled";

/** 礼物赠送 RPC 请求入参 */
export interface SendGiftPayload {
  readonly senderId: string;
  readonly recipientId: string;
  readonly title: string;
  readonly starsPrice: number;
  readonly message?: string;
  readonly isAnonymous: boolean;
  readonly idempotencyKey: string;
  readonly accountScopeGeneration: number;
}

/** 发票创建 RPC 请求入参 */
export interface CreateInvoicePayload {
  readonly title: string;
  readonly description: string;
  readonly currency: string;
  readonly totalAmountCents: number;
  readonly prices: readonly LabeledPriceItem[];
  readonly suggestedTipAmounts?: readonly number[];
  readonly maxTipAmount?: number;
  readonly isRecurring?: boolean;
  readonly isTest?: boolean;
  readonly idempotencyKey: string;
  readonly accountScopeGeneration: number;
}
```

---

### 5.2 后端 Rust 核心状态结构与 RPC 路由对齐

完全对齐 `source/node-agent-coordinator/src/tdrp/stories_gifts_stars.rs` 与 `export_payments_bots.rs` 的权威数据模型：

```rust
// 摘自 source/node-agent-coordinator/src/tdrp/stories_gifts_stars.rs:32-51
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StarLedgerEntry {
    pub entry_id: String,
    pub user_id: String,
    pub amount: i64,
    pub reason: String,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GiftItem {
    pub gift_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub title: String,
    pub stars_price: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub is_anonymous: bool,
    pub sent_at_ms: i64,
}

// 摘自 source/node-agent-coordinator/src/tdrp/export_payments_bots.rs:35-82
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LabeledPrice {
    pub label: String,
    pub amount_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentReceipt {
    pub receipt_id: String,
    pub paid_at_ms: i64,
    pub total_amount_cents: u64,
    pub currency: String,
    pub provider_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaymentInvoice {
    pub invoice_id: String,
    pub title: String,
    pub description: String,
    pub currency: String,
    pub total_amount_cents: u64,
    pub is_paid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_payment_charge_id: Option<String>,
    #[serde(default)]
    pub prices: Vec<LabeledPrice>,
    #[serde(default)]
    pub suggested_tip_amounts: Vec<u64>,
    #[serde(default)]
    pub max_tip_amount: u64,
    #[serde(default)]
    pub is_recurring: bool,
    #[serde(default)]
    pub need_name: bool,
    #[serde(default)]
    pub need_phone: bool,
    #[serde(default)]
    pub need_email: bool,
    #[serde(default)]
    pub need_shipping_address: bool,
    #[serde(default)]
    pub is_test: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt: Option<PaymentReceipt>,
    pub created_at_ms: i64,
}
```

#### 前后端 RPC 路由映射与修复标准表

| 业务场景 | 客户端发起方法 | 服务端真实处理方法 | 修复前缺陷诊断 | 修复后权威入参 Payload | 权威返回 Result |
|---|---|---|---|---|---|
| **获取余额** | `stars.status` / `stars.balance` | `stories_gifts_stars.rs:416` (`stars.status`) | 无版本号返回，仅返回数字 | `{ "userId": string }` | `{ "balance": u64, "balanceVersion": u64 }` |
| **充值星币** | `stars.purchase` | `stories_gifts_stars.rs:425` (`stars.purchase`) | 前端错调 `stars.recharge` | `{ "userId": string, "amount": u64, "reason": string }` | `{ "balance": u64, "balanceVersion": u64 }` |
| **转账打赏** | `stars.transfer` | `stories_gifts_stars.rs:448` (`stars.transfer`) | 缺失幂等键与版本号 | `{ "fromId": string, "toId": string, "amount": u64, "reason": string }` | `{ "fromBalance": u64, "toBalance": u64 }` |
| **赠送礼物** | `gifts.send` | `stories_gifts_stars.rs:485` (`gifts.send`) | 错传 `peerId` 与 `giftName` | `{ "senderId": string, "recipientId": string, "title": string, "starsPrice": u64, "message"?: string, "isAnonymous": bool }` | `{ "gift": GiftItem, "remainingBalance": u64 }` |
| **收到礼物** | `gifts.listReceived` | `stories_gifts_stars.rs:527` (`gifts.listReceived`) | 前端未接线该接口 | `{ "recipientId": string }` | `{ "gifts": Vec<GiftItem> }` |
| **创建发票** | `payments.createInvoice`| `export_payments_bots.rs:360` (`payments.createInvoice`) | 前端缺少发票卡片唤起 | `{ "title": string, "description": string, "currency": string, "totalAmountCents": u64, "prices": Vec<LabeledPrice> }` | `{ "invoice": PaymentInvoice }` |
| **收银核销** | `payments.settlePayment`| `export_payments_bots.rs:394` (`payments.settlePayment`) | 前端收银台未对账 | `{ "invoiceId": string, "chargeId": string }` | `{ "invoice": PaymentInvoice }` |
| **获取凭据** | `payments.getReceipt` | `export_payments_bots.rs:400` (`payments.getReceipt`) | 前端无凭据展示 | `{ "invoiceId": string }` | `{ "receipt": PaymentReceipt }` |

---

### 5.3 严格数学状态转移方程与 CAS 并发控制

#### 1. 资产变动乐观并发控制（CAS）方程
任何扣减操作必须遵循以下原子转移方程：
$$\text{NextBalance} = \begin{cases} 
\text{Balance} - \text{cost}, & \text{if } \text{Balance} \ge \text{cost} \land \text{Version}_{\text{client}} = \text{Version}_{\text{server}} \\
\text{REJECT}(\text{INSUFFICIENT\_FUNDS}), & \text{if } \text{Balance} < \text{cost} \\
\text{RETRY\_WITH\_BACKOFF}, & \text{if } \text{Version}_{\text{client}} \neq \text{Version}_{\text{server}}
\end{cases}$$

#### 2. 发票支付状态转移方程
发票有限状态机由以下映射驱动：
$$\delta_{\text{invoice}}(S_{\text{inv}}, E) = \begin{cases}
\text{PendingPayment}, & (S_{\text{inv}}=\text{Draft}) \land (E=\text{CREATE}) \\
\text{Processing}, & (S_{\text{inv}}=\text{PendingPayment}) \land (E=\text{SUBMIT\_PAYMENT}) \\
\text{Paid}(\text{receiptId}), & (S_{\text{inv}}=\text{Processing}) \land (E=\text{CHARGE\_SUCCESS}) \\
\text{PendingPayment}, & (S_{\text{inv}}=\text{Processing}) \land (E=\text{CHARGE\_DECLINED}) \\
\text{Cancelled}, & (S_{\text{inv}}=\text{PendingPayment}) \land (E=\text{USER\_CANCEL}) \\
\text{Expired}, & (S_{\text{inv}}=\text{PendingPayment}) \land (E=\text{TIMEOUT})
\end{cases}$$

---

### 5.4 跨账号、并发请求与网络分区 Fencing 围栏

1. **`accountScopeGeneration` 单调递增安全隔离**：
   - 客户端内部维护由用户认证系统单调递增的 `accountScopeGeneration: number`；
   - 当检测到该世代标识变更时，`starsLedgerStore` 与 `paymentSettlementStore` 立即执行**物理重置**：
     清空本地内存中的星币余额、清空收银会话、终止所有针对未决发票的轮询请求；
   - 所有在途的 RPC 请求携带发起时的 `generation`。当响应到达时，若 `response.generation !== currentGeneration`，必须立即静默丢弃（Fail-closed Drop），杜绝将旧账号的数据投射给新登录账号。
2. **离线重连与网络中断对账（Reconciliation on Reconnect）**：
   - 当发生网络闪断或客户端重启时，处于 `Processing` 状态的订单不得盲目发起二次扣款；
   - 恢复连接后，客户端自动以 `invoiceId` 和 `idempotencyKey` 向服务端发起 `payments.getReceipt` 幂等查询；
   - 服务端若已扣款成功，则同步返回票据并推进状态至 `Paid`；若服务端未收到扣款，则恢复为 `PendingPayment` 允许用户重新发起。


---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 实施切片拆解与派工清单（Work Slices 13-A 至 13-E）

为确保后续子智能体（Worker Subagents）在落地修改代码时高效并发、职责边界清晰且无合并冲突，本蓝图将全域任务解耦为 5 个原子实施切片：

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ 切片 13-A: 核心数据模型与 Store 状态机 (Model & Economy Stores)                                 │
│  - 目标路径: frontend/src/recovered/features/economy/model.ts                                   │
│              frontend/src/recovered/features/economy/stars-ledger-store.ts                       │
│              frontend/src/recovered/features/economy/payment-settlement-store.ts                 │
│  - 核心职责: 建立 TS 契约接口，实现 CAS 乐观并发控制、RPC 真实接线与 accountScopeGeneration 围栏 │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 切片 13-B: 真实星标资产卡片与充值模态框 (Stars Balance Card & Recharge Modal)                     │
│  - 目标路径: frontend/src/recovered/features/economy/components/StarsBalanceCard.tsx             │
│              frontend/src/recovered/features/economy/components/StarsRechargeModal.tsx           │
│              frontend/src/production/tdrp/TdrpProfileDetailsDrawer.tsx (消除 1250 硬编码)         │
│  - 核心职责: 彻底移除 1250 硬编码；接入真实 starsLedgerStore 订阅；实现充值档位选择与防重放提交   │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 切片 13-C: 礼物赠送中心与参数对齐 (Star Gifts Dock Modal & Coordinator RPC Alignment)             │
│  - 目标路径: frontend/src/recovered/features/economy/components/StarGiftsDockModal.tsx           │
│  - 核心职责: 完全对齐 stories_gifts_stars.rs:485-502 的入参键名；实现礼物网格、匿名与折现配置   │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 切片 13-D: 原生发票卡片与多渠道收银台 (Invoice Transcript Card & Checkout Modal)                 │
│  - 目标路径: frontend/src/recovered/features/conversation/cards/InvoiceTranscriptCard.tsx        │
│              frontend/src/recovered/features/economy/components/InvoiceCheckoutModal.tsx         │
│  - 核心职责: 在聊天消息流中内联渲染发票状态；打通 export_payments_bots.rs 的 settlePayment 核销 │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 切片 13-E: 生产壳窄接线与孤岛废弃 (ProductionRenderer Narrow Wiring & Orphan Elimination)        │
│  - 目标路径: frontend/src/production/tdrp/TdrpStoriesAndStarsBar.tsx (标记废弃/清空假状态)       │
│              frontend/src/production/tdrp/TdrpConversationOverlay.tsx (清空本地假余额与定时器)   │
│              frontend/src/production/ProductionRenderer.tsx (执行窄接线组装)                    │
│  - 核心职责: 根除 setTimeout 假回显，挂载权威全局收银台与充值弹窗容器                            │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 6.2 文件修改白名单与只读参考范围

严格限定子智能体的写操作权限，防止跨界破坏其他稳定模块：

#### 允许修改的文件白名单（Writable Whitelist）
1. `frontend/src/recovered/features/economy/**`（新建单一权威经济与支付目录下的全部实现文件）
2. `frontend/src/recovered/features/conversation/cards/InvoiceTranscriptCard.tsx`（新建发票消息卡片渲染器）
3. `frontend/src/production/tdrp/TdrpProfileDetailsDrawer.tsx`（彻底删除 `starsBalance = 1250` 硬编码与静态假展示，替换为 `<StarsBalanceCard ... />` 或真实属性订阅）
4. `frontend/src/production/tdrp/TdrpStoriesAndStarsBar.tsx`（清空 `starsBalance = 1280` 假状态，重构为消费权威 `starsLedgerStore`，或将通用星币逻辑下沉并标记废弃）
5. `frontend/src/production/tdrp/TdrpConversationOverlay.tsx`（删除 `setStarsBalance((b) => b +/- amount)` 本地客户端乐观篡改与 `setTimeout` 假状态；将送礼与充值代理至权威 Store）
6. `frontend/src/production/ProductionRenderer.tsx`（仅限执行窄接线：挂载全局收银台与充值弹窗容器，向底层提供 `client` 与 `accountScopeGenerationRef.current`）

#### 只读参考范围（Strictly Read-Only References）
- `source/node-agent-coordinator/src/tdrp/stories_gifts_stars.rs`（权威 Rust 礼物与星币服务，只读对照）
- `source/node-agent-coordinator/src/tdrp/export_payments_bots.rs`（权威 Rust 发票与支付核销服务，只读对照）
- `docs/adr/FBCP-ADR-002-payment-settlement.md`（支付基础设施架构规范）
- `source/host/src/storage/store_db.rs`（持久化数据库结构参考）
- `contracts/automation/cross-platform-journeys.json`（跨平台自动化测试契约）

---

### 6.3 窄接线架构纪律（Narrow Wiring Disciplines）

在更新 `ProductionRenderer.tsx` 时，子智能体必须严格遵循以下三项铁律：
1. **单一状态收敛**：
   在 `ProductionRenderer.tsx` 中仅新增单一受控模态框状态：
   ```typescript
   const [economyModalState, setEconomyModalState] = useState<{
     readonly mode: "none" | "recharge" | "gifts" | "checkout";
     readonly invoiceId?: string;
     readonly recipientId?: string;
   }>({ mode: "none" });
   ```
2. **零内联业务逻辑**：
   `ProductionRenderer.tsx` 只充当组装插座，传递已有上下文（`client`, `accountScopeGenerationRef.current`, `activeAgent`），严禁在 `ProductionRenderer.tsx` 中编写超过 15 行的充值计算、礼物扣除或支付回调逻辑。
3. **保持单向数据流与世代防护**：
   所有扣款与赠送必须经过 `starsLedgerStore` 与 `paymentSettlementStore`，最终流向 Coordinator RPC。当 `accountScopeGeneration` 变更时，全局经济弹窗自动收起，清除未决表单。

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

为保障融合质量，独立审计子智能体（Auditor/Reviewer）在审查实施代码时，必须执行以下正反例对照核验：

### 7.1 绝对禁止的反模式检查清单（Negative Anti-Patterns — Fail-Closed）

- [ ] **严禁硬编码假星标余额**：源码中绝对不允许出现 `starsBalance = 1250`、`starsBalance = 1280` 或任何形式的静态打桩数值。
- [ ] **严禁假定时器逻辑**：绝对禁止在支付、送礼、充值回调中使用 `setTimeout` 或 `setInterval` 伪造“充值成功”、“正在核销”或延时清空状态（如 `setTimeout(() => setTransferStatus(null), 2500)`）。
- [ ] **严禁客户端本地乐观篡改资产**：绝对禁止在未收到服务端权威 RPC 成功回执前，通过 `setStarsBalance((b) => b - cost)` 自行扣减或增加前端状态。
- [ ] **严禁断头 RPC 调用**：
  - 严禁调用不存在的 RPC 方法（如 `stars.recharge`）；
  - 严禁传递与后端不匹配的入参（如在 `gifts.send` 中传 `peerId` 代替 `recipientId`，或遗漏 `starsPrice`）。
- [ ] **严禁跨账号资产与账本穿透**：当 `accountScopeGeneration` 递增时，前端经济状态必须物理清空，严禁在新账号中残留旧账号的余额、历史凭据或未决发票。
- [ ] **严禁绕过 ADR-002 污染会话历史**：严禁将财务结算真相委托给聊天会话记录（Transcript）或设置面板（Settings），必须由独立的 `PaymentSettlement` 所有者托管。
- [ ] **严禁引入平行样式系统**：严禁裸写十六进制色彩或引入 `--oc-*` 全局样式，必须 100% 消费 `--fabushi-*` 官方设计变量。

---

### 7.2 必须达成的正例验收标准（Positive Acceptance Gates）

- [ ] **权威账本与 CAS 乐观并发合规**：
  - `starsLedgerStore` 正确拉取 `stories_gifts_stars.rs` 的真实余额与流水；
  - 资产扣减携带 `balanceVersion`，并发冲突时安全回滚并自动重试。
- [ ] **礼物赠送全链路闭环**：
  - `StarGiftsDockModal` 完整支持礼物挑选、匿名赠送开关（`hideMyName`）、折现开关（`convertStars`）与附言输入；
  - 调用的 `gifts.send` 参数完全对齐后端 `{ senderId, recipientId, title, starsPrice, message, isAnonymous }`。
- [ ] **消息流原生发票与收银台闭环**：
  - 消息流中能正确反序列化并渲染 `InvoiceTranscriptCard`；
  - 点击“安全支付”唤起 `InvoiceCheckoutModal`，成功调用 `payments.settlePayment` 并呈现权威不可变 `PaymentReceipt`。
- [ ] **响应式自适应与无障碍达标**：
  - 宽屏（$\ge 1200\text{px}$）居中模态；中屏（$900\text{--}1199\text{px}$）自适应单列；窄屏（$< 900\text{px}$）转为优雅的底部抽屉（Bottom Sheet）；
  - 文本对比度严格满足 WCAG AA（$\ge 4.5:1$）；包含清晰 ARIA 标签与无障碍焦点循环；
  - 输入框受中文输入法（IME）`isComposing` 保护，确认选词不误触表单提交。
- [ ] **代码内聚预算严格合规**：
  - `frontend/src/recovered/features/economy/` 下各个新建文件行数严格控制在 **200–600 行** 之间，严格遵守 `Same Owner != Same File` 原则。

---

### 7.3 独立审计核验流程（Auditor Verification Protocol）

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ 第一步：静态引用链与死代码审查 (Static Import Graph Audit)                                       │
│   - 核查 ProductionRenderer.tsx 确认仅挂载单一经济模态框宿主                                      │
│   - 确认 TdrpProfileDetailsDrawer.tsx 中 1250 硬编码已彻底清除                                   │
│   - 确认 TdrpConversationOverlay.tsx 中 setTimeout 假状态已彻底移除                              │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 第二步：代码反模式扫描 (Anti-Pattern Regex Scan)                                                 │
│   - 扫描 frontend/ 目录，确认无 starsBalance = 1250 与 starsBalance = 1280 残留                  │
│   - 扫描确认无 stars.recharge 错误调用，全部对齐为 stars.purchase                                 │
│   - 扫描确认 gifts.send 包含 recipientId 与 starsPrice 参数                                      │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 第三步：类型契约与状态机流转审查 (TypeScript & Rust Contract Check)                               │
│   - 核验 StarBalanceSnapshot、PaymentInvoiceSnapshot 与 Rust 结构体字段 1:1 镜像                 │
│   - 验证 CAS 乐观锁版本号校验与 accountScopeGeneration 世代重置逻辑                              │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 第四步：设计变量与无障碍核验 (Design Tokens & A11y Audit)                                        │
│   - 校验所有新建样式严格消费 --fabushi-* 语义变量                                               │
│   - 验证模态框 role="dialog"、aria-modal="true" 与 IME isComposing 事件拦截代码                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```
