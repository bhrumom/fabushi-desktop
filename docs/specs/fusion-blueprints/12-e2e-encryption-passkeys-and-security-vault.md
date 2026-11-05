# Fabushi 统一融合蓝图 12：端到端双棘轮加密、Passkeys/云密码与安全保险箱体系
# Volume 12: E2E Encryption, Passkeys, and Security Vault

> **规范编号**: `FBCP-BLUEPRINT-12` / `UFX-BLUEPRINT-12`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. **根除伪造哈希与伪安全逻辑（Eradicate Fake btoa/Date.now Hashes）**：彻底清除 `frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx` 中使用的 `btoa(password)` 与 `Date.now()` 伪盐，全面升级为行业标准 SRP-6a（RFC 5054）与 Argon2id 密码哈希派生及 W3C 标准 WebAuthn / Passkeys 物理硬件认证。
> 2. **端到端加密与双棘轮状态机闭环（Double Ratchet & Forward Secrecy Closure）**：将 `native/mahayana-messaging/src/secret_chat.rs` 中已实现的 X25519 密钥交换、HKDF-SHA256 密钥派生与 XChaCha20Poly1305 防重放（128-bit Replay Window）加密引擎全面接线至会话管道，形成前向保密（PFS）私密会话。
> 3. **本地安全密码锁与挂起锁屏门禁（Local Passcode & Screen Blurring Vault）**：打通 `source/electron-main/account/`、`source/electron-main/secrets/secret-store.ts` 与 OS 安全存储（Electron `safeStorage` / Keychain / DPAPI），实现超时自动锁屏、连续输错指数退避与防截屏保护。
> 4. **敏感资产与凭证保险箱（Secrets Vault & Self-Destruct Policy）**：统领 Telegram 6 个安全与隐私能力域（TCG-N24, N25, N32, N45, N46, N52）与 OpenClaw 2 个安全域（OCRM-M03, M10），闭环端到端加密身份凭证保险箱、敏感内容遮罩（Blur Shield）与服务端权威阅后即焚（Self-Destruct）倒计时销毁机制。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照矩阵

| 功能项与能力域 | OpenClaw 锁定源码锚点 | Telegram Desktop 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **端到端双棘轮加密**<br>(TCG-N52 / E2E Security) | `src/secrets/vault.ts`<br>`src/pairing/` | `Telegram/SourceFiles/tde2e/`<br>`storage/storage_encryption.cpp` | `native/mahayana-messaging/src/secret_chat.rs:1-320`<br>`source/host/src/extensions/session/native_messaging.rs` | `native/mahayana-messaging/src/secret_chat.rs`<br>`source/host/src/extensions/session/` |
| **云密码 2FA 与 SRP 握手**<br>(TCG-N25 / Cloud Password) | `src/secrets/`<br>`src/config/` | `storage/storage_cloud_password.cpp`<br>`core/core_cloud_password.cpp` | `source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs:65-90`<br>`frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx:71-110` | `source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs`<br>`frontend/src/production/security/` |
| **Passkeys / WebAuthn 硬件认证**<br>(TCG-N25 / Passkeys) | `src/secrets/`<br>`extensions/device-pair/` | `platform/platform_webauthn.cpp`<br>`window/window_lock_widgets.cpp` | `source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs:93-100`<br>`frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx:180-210` | `source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs`<br>`source/electron-main/account/` |
| **本地密码锁与锁屏防窥**<br>(TCG-N25 / Passcode Lock) | `src/security/approvals.ts` | `core/passcode_lock.cpp`<br>`window/window_lock_widgets.cpp` | `source/electron-main/secrets/secret-store.ts:25-32`<br>`source/electron-main/account/account-authorization.ts:1-21` | `source/electron-main/security/passcode-manager.ts`<br>`frontend/src/production/security/LocalPasscodeModal.tsx` |
| **用户隐私与黑名单拦截**<br>(TCG-N24 / Privacy & Block) | `src/security/`<br>`packages/net-policy/` | `api/api_user_privacy.cpp`<br>`api/api_blocked_peers.cpp`<br>`settings/settings_privacy_controllers.cpp` | `source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs:40-63`<br>`frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx:57-70` | `source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs`<br>`source/host/src/extensions/session/session_roster.rs` |
| **身份凭证保险箱 (Passport)**<br>(TCG-N32 / OCRM-M03) | `src/secrets/vault.ts`<br>`extensions/onepassword/` | `passport/passport_form_controller.cpp`<br>`passport/passport_encryption.cpp`<br>`api/api_passport.cpp` | `source/electron-main/secrets/secret-store.ts:10-23`<br>`source/host/src/host_secret_store.rs`<br>`source/shared/box-secrets.ts` | `source/electron-main/secrets/`<br>`source/host/src/extensions/secrets/secrets_service.rs` |
| **阅后即焚与定时销毁**<br>(TCG-N45 / Self-Destruct) | `src/storage/`<br>`src/sessions/` | `api/api_self_destruct.cpp`<br>`history/history_item.cpp`<br>`data/data_user.cpp` | `source/host/src/extensions/transcript/transcript_store.rs`<br>`source/host/src/extensions/session/production.rs` | `source/host/src/extensions/transcript/`<br>`source/host/src/extensions/session/` |
| **敏感内容遮罩与安全门禁**<br>(TCG-N46 / OCRM-M10) | `src/security/approvals.ts`<br>`src/audit/` | `api/api_sensitive_content.cpp`<br>`settings/settings_privacy_controllers.cpp` | `frontend/src/recovered/features/conversation/cards/transcript-card/`<br>`source/host/src/extensions/local_tool_permission/` | `frontend/src/production/security/SensitiveContentShield.tsx`<br>`source/host/src/runner/sand_computer_auto_review.rs` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **Telegram Desktop 侧**：
  - **云密码与 2FA**：`storage/storage_cloud_password.cpp` 与 `core/core_cloud_password.cpp` 严格实现 SRP-6a（RFC 5054）无密码泄露证明协议。客户端使用 `srp_id`、质数模数 $N$、生成元 $g$ 以及加盐派生的 $x = \text{Argon2id}(\text{password}, \text{salt})$ 计算 $A = g^a \pmod N$ 并验证服务端 $B$ 与证明凭证 $M_1/M_2$，服务端全程绝不接触明文密码或静态哈希。
  - **本地 Passcode 锁**：`core/passcode_lock.cpp` 与 `window/window_lock_widgets.cpp` 提供应用级本地锁屏，支持 1 分钟至 5 小时自动锁屏定时器，输错 5 次触发指数退避冷却（1s, 5s, 15s, 60s, 300s），锁屏时整个主窗口覆盖高斯模糊并阻断所有快捷键。
  - **端到端加密与阅后即焚**：`tde2e/` 与 `api/api_self_destruct.cpp` 提供设备级密钥协商与权威 TTL 销毁，附件与消息在 TTL 归零后物理擦除（Secure Zeroize）。
- **OpenClaw 侧**：
  - `src/secrets/` 与 `src/pairing/` 采用安全密钥存储抽象，将敏感 API Token 与凭证存储在 OS 系统级 Keyring / Keychain 中，支持按 Provider 角色与 Account Scope 隔离。
  - `src/security/approvals.ts` 实施严格的工具调用高危审批门禁与权限生命周期管理。

#### Q2（现有链路定位）
- **端到端加密链路**：
  `native/mahayana-messaging/src/secret_chat.rs` 实现了完整的 `SecretChatSession`，采用 X25519 协商 32 字节共享密钥，经 HKDF-SHA256 派生会话密钥，XChaCha20Poly1305（24 字节 Nonce）执行加解密，并使用 128 位滑动位图（`ReplayWindow`）彻底防御重放攻击。
- **云密码与 Passkeys 链路**：
  前端 `frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx` -> JSON-RPC -> `source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs`（`SettingsCloudPasskeysState`）-> 持久化与验证。
- **本地凭证与秘钥安全存储链路**：
  `source/electron-main/secrets/secret-store.ts` -> Electron `safeStorage` API -> macOS Keychain / Windows DPAPI / Linux SecretService。

#### Q3（断点精确诊断）
1. **伪造客户端哈希与伪盐致命断点（P0 欺骗性安全漏洞）**：
   在 `frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx:132-133` 与 `152, 167` 中赫然存在：
   ```tsx
   // 致命缺陷：使用 btoa 伪造哈希，使用 Date.now() 伪造盐！
   const hash = `hash_${btoa(passwordInput.trim())}`;
   const salt = `salt_${Date.now()}`;
   ```
   同时在 `185-187` 行 Passkeys 注册中：
   ```tsx
   const credentialId = `passkey_${Date.now().toString(36)}`;
   const publicKeyBase64 = btoa(`pubkey_${credentialId}`);
   ```
   **诊断**：这是严重的假逻辑作弊代码！密码仅经过简单的 Base64 编码即作为“哈希”发送给 Coordinator，完全违背了加密安全规范；Passkeys 更是使用 `btoa` 生成伪造公钥，根本没有调用浏览器的 `navigator.credentials.create()` 真实 WebAuthn 硬件接口。
2. **端到端加密与前端会话舞台割裂（`implemented` 但 `not-wired`）**：
   `native/mahayana-messaging/src/secret_chat.rs` 虽然单元测试完备，但前端会话舞台（`02-unified-conversation-stage-and-header.md`）与 Host 会话分派器（`native_messaging.rs`）并未接入 `SecretChatSession` 的生命周期，密聊通道无法从 UI 显式建立。
3. **本地密码锁（Passcode）仅有后端类型，缺少 Electron 主进程锁屏控制器与 UI 遮罩**：
   `source/electron-main/secrets/secret-store.ts` 具备安全加解密能力，但没有维护独立的本地锁屏状态机（`Locked` / `Unlocked` / `GracePeriod`），亦未在超时后向渲染进程发送全局锁屏遮罩通知。

#### Q4（状态机与安全防护）
- **本地密码锁 4 态状态机**：
  $$\text{PasscodeState} \in \{\text{Disabled}, \text{Unlocked}, \text{GracePeriod}, \text{Locked}\}$$
  - $\text{Unlocked} \xrightarrow{\text{user\_idle} \ge \text{timeout}} \text{Locked}$
  - $\text{Locked} \xrightarrow{\text{verify\_passcode\_ok}} \text{Unlocked}$
  - $\text{Locked} \xrightarrow{\text{verify\_fail}[N]} \text{Locked}(\text{backoff}=2^{N-5} \text{s})$
- **SRP-6a 两步验证握手协议**：
  客户端向 Coordinator 发起挑战请求 $\to$ Coordinator 返回加盐参数 $(s, B)$ $\to$ 客户端生成私有随机数 $a$，计算公共公钥 $A = g^a \pmod N$ 及证据 $M_1 = H(H(N) \oplus H(g) \parallel H(I) \parallel s \parallel A \parallel B \parallel K)$ $\to$ Coordinator 校验 $M_1$，返回确认证据 $M_2 = H(A \parallel M_1 \parallel K)$ $\to$ 完成双向身份证明，绝不传递明文密码。
- **跨账号单调递增防护（`accountScopeGeneration`）**：
  密码锁、保险箱与端到端密钥完全按 `accountScopeGeneration` 物理隔离，切换账号立即销毁内存中的会话私钥（Zeroize），阻断旧在途密文解码。

#### Q5（UI/UX 极致融合落点）
- **标准设置落点**：
  在“设置 -> 隐私与安全（Privacy & Security）”一级面板中整合：2FA 云密码管理、Passkeys 硬件凭证管理、本地应用密码锁（含自动锁屏时长滑块与 Touch ID / Windows Hello 开关）、隐私规则大表（10 项）。
- **锁屏交互与安全遮罩**：
  当处于 `Locked` 状态时，主渲染窗口立即加载全屏高斯模糊安全遮罩（`backdrop-filter: blur(20px)`），禁止 Alt+Tab 预览缩略图，提供密码输入框与生物识别解锁按钮。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-12-01** | 云密码与通行密钥使用 `btoa` 与 `Date.now()` 伪造哈希/公钥，虚假安全 | `frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx:132-133`<br>`frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx:185-187` | 前端未接入真实 WebAuthn（`navigator.credentials`）与 Argon2id 派生库，直接使用明文 Base64 冒充加密哈希。 | **P0 伪安全作弊** |
| **BUG-12-02** | 双棘轮密聊仅沉淀在 Rust native 库中，未与统一会话流打通 | `native/mahayana-messaging/src/secret_chat.rs:181-208`<br>`source/host/src/extensions/session/gateway.rs:40-60` | 前端无发起私密会话的 typed action，Host Session 未持有 `SecretChatSession` 实例及 Epoch 轮换机制。 | **P1 链路断头** |
| **BUG-12-03** | 缺少本地密码锁（Passcode）与闲置自动锁屏守护，防窥能力为零 | `source/electron-main/secrets/secret-store.ts:20-30`<br>`source/electron-main/entry.ts:80-110` | 只有 OAuth Token 的安全存储，缺失应用层独立的 Passcode 验证器与系统闲置时间（`powerMonitor.getSystemIdleTime()`）监听。 | **P1 功能缺失** |
| **BUG-12-04** | 云端密码验证缺乏 SRP-6a 协议，服务端单方面保存密码哈希，易受彩虹表攻击 | `source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs:187-203` | Coordinator 直接进行简单的字符串等值比较（`config.password_hash_hex == candidate_hash_hex`），缺少抗重放与零知识证明。 | **P1 协议降级** |
| **BUG-12-05** | 敏感证件与机密凭证（Passport / Vault）未与 Secure Enclave 硬件加密绑定 | `source/host/src/host_secret_store.rs:45-80`<br>`source/shared/box-secrets.ts:20-45` | 未使用主密码对敏感附件派生独立的数据保护密钥（DEK），导致机密凭据存储在常规数据库目录中。 | **P2 架构隐患** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 统一安全分层架构拓扑

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              Frontend Presentation Layer                               │
│  ┌───────────────────────────────────────────┐  ┌───────────────────────────────────┐  │
│  │ Settings > Privacy & Security Panel       │  │ Global Lock Overlay / Passcode UI │  │
│  │ - 10 项细粒度隐私策略 (PrivacyRules)      │  │ - 全屏高斯模糊安全遮罩 (Blur)      │  │
│  │ - 云端双因子密码 (Argon2id + SRP-6a)      │  │ - Touch ID / Windows Hello 生物识别│  │
│  │ - WebAuthn Passkeys 物理硬件密钥列表      │  │ - 指数退避重试输入框 (PIN/Password)│  │
│  └─────────────────────┬─────────────────────┘  └─────────────────▲─────────────────┘  │
└────────────────────────┼──────────────────────────────────────────┼────────────────────┘
                         │ IPC (ContextBridge / Typed RPC)          │ Native Lock Signal
┌────────────────────────▼──────────────────────────────────────────┴────────────────────┐
│                        Electron Main Process Security Layer                            │
│  ┌───────────────────────────────────────────┐  ┌───────────────────────────────────┐  │
│  │ Passcode & Auto-Lock Manager              │  │ OS SafeStorage & Credential Vault │  │
│  │ - powerMonitor 闲置时间探测 (Idle Timer)  │  │ - Electron safeStorage (DPAPI/Key)│  │
│  │ - 加盐 Argon2id 本地密码哈希校验          │  │ - Scoped Token 物理隔离缓存       │  │
│  │ - 密码输错指数退避计数器 (Backoff Gate)   │  │ - 硬件安全芯片 (Secure Enclave)   │  │
│  └─────────────────────┬─────────────────────┘  └─────────────────┬─────────────────┘  │
└────────────────────────┼──────────────────────────────────────────┼────────────────────┘
                         │ JSON-RPC Channel                         │ Zero-Knowledge SRP
┌────────────────────────▼──────────────────────────────────────────▼────────────────────┐
│                  Node Agent Coordinator & Native Rust Core                             │
│  ┌───────────────────────────────────────────┐  ┌───────────────────────────────────┐  │
│  │ Coordinator Security & Passkeys Registry  │  │ Mahayana Native Secret Chat Core  │  │
│  │ - SRP-6a 双向认证协议状态机 (RFC 5054)    │  │ - X25519 临时密钥对动态协商      │  │
│  │ - W3C WebAuthn 挑战值与反重放计数器仲裁   │  │ - HKDF-SHA256 棘轮推进派生密钥    │  │
│  │ - 隐私规则与黑名单原子落盘持久化          │  │ - XChaCha20Poly1305 128-bit 防重放 │  │
│  └───────────────────────────────────────────┘  └───────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 模块职责与代码文件行数预算（200–600 行内聚预算）

遵循 `Same Owner != Same File` 与单文件 200–600 行内聚预算原则，严禁新增巨石文件：

1. `frontend/src/production/security/UnifiedSecurityPanel.tsx`（**380–480 行**）：
   取代旧有的 `TdrpCloudSecurityPanel.tsx`，负责隐私规则、两步验证云密码配置与通行密钥列表的统一展示。
2. `frontend/src/production/security/LocalPasscodeModal.tsx`（**240–320 行**）：
   负责全屏防窥锁屏模态框、PIN 码输入、生物识别快捷触发与输错退避倒计时动画。
3. `source/electron-main/security/passcode-manager.ts`（**300–420 行**）：
   主进程常驻单例，负责系统闲置时间监听、本地密码哈希存储、自动锁屏定时器调度与 IPC 通信。
4. `source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs`（重构收敛至 **450–550 行**）：
   实现纯正的 SRP-6a 握手状态机、WebAuthn 挑战值缓存池与账户隐私规则仲裁。
5. `native/mahayana-messaging/src/secret_chat.rs`（已存在 **666 行**，保持稳定）：
   端到端双棘轮加密底层引擎，提供密聊建立、密钥旋转（Epoch Rotation）与加解密服务。

---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 渐进披露布局与设计层次（Progressive Disclosure）

针对安全与隐私配置的高敏感度与低频/高频使用差异，界面采用三级渐进披露架构：

1. **第一级：安全健康度总览看板（Security Health Dashboard）**
   - 顶部提供动态安全健康状态卡片：计算综合安全评分（`0–100`），实时呈现 4 大核心维度：
     - **云密码两步验证**：未开启（红 / Warning）、已开启（绿 / Protected）、重置中（黄 / Pending Reset 7-day countdown）。
     - **通行密钥（Passkeys）**：已绑定硬件密钥数量及最近认证时间。
     - **本地应用锁屏**：当前是否已启用、自动锁屏时间（如 5 分钟）、生物识别状态。
     - **活跃端到端会话**：当前持有的 PFS 密聊会话数与临时公钥指纹。
2. **第二级：分类功能卡片抽屉（Security Category Cards）**
   - **两步验证云密码配置区**：
     - 未设置状态：展示设置向导，输入主密码 $\to$ 实时密码强度计响应 $\to$ 确认密码 $\to$ 设置密码提示（Hint） $\to$ 设置恢复邮箱（Recovery Email）。
     - 已设置状态：展示“修改密码”、“禁用密码”、“重置密码”操作组；若处于重置等待期（7天），顶部出现醒目黄色冻结横幅与“取消重置”立即求助通道。
   - **通行密钥（WebAuthn Passkeys）管理区**：
     - 密钥列表卡片：展示每个凭证的名称（如 “MacBook Pro Touch ID” / “YubiKey 5C NFC”）、Credential ID 前缀、注册时间与使用计数器（Counter）。
     - 添加新密钥按钮：点击唤起系统级 WebAuthn 对话框，完成生物或硬件指纹绑定。
   - **本地锁屏密码（Passcode Lock）配置区**：
     - 提供“开启应用锁”开关、4 位或 6 位数字 PIN / 字母数字密码切换。
     - “闲置自动锁屏”下拉框：支持“立即”、“1分钟”、“5分钟”、“15分钟”、“1小时”、“5小时”。
     - “允许使用 Touch ID / Windows Hello 解锁”切换开关。
   - **细粒度隐私策略矩阵（Privacy Rules Table）**：
     - 包含 10 项核心隐私项目（手机号、最后上线、头像、转发来源保留、呼叫、P2P 直连、语音留言权限、个人简介、生日、礼物自动展示）。
     - 每项提供三态选择器：`所有人（AllowAll）`、`仅联系人（ContactsOnly）`、`任何人都不（DisallowAll）`，并支持点击展开例外白名单（Always Allow）与黑名单（Never Allow）。
3. **第三级：高危与敏感二次确认弹窗（High-Impact Confirmation Dialogs）**
   - 禁用云密码、删除 Passkey 或清空本地密码锁时，强制要求先验证当前密码或触发硬件生物识别，防止设备被他人暂借时遭恶意篡改。

---

### 4.2 本地锁屏遮罩、防窥与生物识别交互规范

当应用进入 `Locked` 状态时，渲染进程与主进程协同启动防窥门禁：

```
┌────────────────────────────────────────────────────────────────────────┐
│                        FABUSHI SECURITY SHIELD                         │
│                                                                        │
│                                 [ 🔒 ]                                 │
│                           应用已处于安全锁定状态                          │
│                                                                        │
│                       ┌─────────────────────┐                          │
│                       │  ● ● ● ● ● ●        │                          │
│                       └─────────────────────┘                          │
│                                                                        │
│                      [ 👆 使用 Touch ID 解锁 ]                          │
│                                                                        │
│        [连续输错提示: 密码错误，请等待 15 秒后重试 (退避中...)]        │
└────────────────────────────────────────────────────────────────────────┘
```

1. **瞬时高斯模糊与系统层防窥（Blur & Window Concealment）**：
   - 锁屏触发瞬间（0ms），全局根节点注入 `backdrop-filter: blur(24px) saturate(180%)`，并遮盖 `--fabushi-color-surface-veil`（不透明度 85%）。
   - 主进程调用 `win.setContentProtection(true)`，在 macOS 调度中心、Windows 任务切换器（Alt+Tab）及系统截屏工具中将窗口内容全黑化，杜绝敏感聊天泄露。
2. **键盘流与焦点陷阱（Focus Trap）**：
   - 输入焦点强制锁定于 PIN 密码框，禁用 `Tab` 键跳出，拦截所有应用快捷键（`Ctrl+K`、`Esc` 等）。
   - 按下 `Enter` 触发解锁校验；支持点击“使用 Touch ID 解锁”重新激活系统生物识别弹窗。
3. **输错指数退避机制（Exponential Backoff Motion）**：
   - 第 1–4 次输错：输入框产生水平左右晃动抖动动画（`@keyframes shake 0.3s ease`），错误计数加 1。
   - 第 5 次及以上输错：密码输入框强制置灰禁用，展示倒计时进度条，锁定时间依 $T = 2^{N-5} \times 1\text{s}$ 指数递增（第 5 次 1s，第 6 次 2s，第 7 次 4s，第 8 次 8s，第 9 次 16s，第 10 次 32s，上限 300s）。

---

### 4.3 密码强度计与无障碍合规（zxcvbn & WCAG AA）

1. **动态熵值密码强度指示条**：
   - 在设置/修改云密码与本地密码时，实时计算密码熵值（基于长度、字符集多样性、常见字典词频与键盘模式）：
     - 弱（Weak，<30 bits）：红色指示条（`--fabushi-color-status-error`），提示“密码过短或易被猜测”。
     - 中（Fair，30–59 bits）：黄色指示条（`--fabushi-color-status-warning`），提示“建议加入大小写字母、数字及特殊符号”。
     - 强（Strong，60–79 bits）：浅绿指示条（`--fabushi-color-status-info`），提示“密码强度良好”。
     - 极强（Excellent，≥80 bits）：翠绿指示条（`--fabushi-color-status-success`），提示“高安全防御等级”。
2. **无障碍标准（A11y ARIA & Screen Reader）**：
   - 所有密码框标有显式 `aria-label` 与 `aria-describedby` 提示。
   - 错误状态使用 `role="alert"` 与 `aria-live="assertive"` 声明，屏幕阅读器在输错时即刻读出失败原因与剩余尝试次数。
   - 支持高对比度模式，文本与输入框边框对比度均满足 WCAG AA 级要求（$\ge 4.5:1$）。

---

### 4.4 响应式断点与语义 Token 规范

严禁残留任何历史 `--sand-*`、`--cursor-*` 样式变量，全面收口至 `--fabushi-*` 语义设计系统：

| 断点区间 | 布局与展示策略 | 交互适配 |
|---|---|---|
| **桌面宽屏 (≥1200px)** | 隐私安全控制面板采用双列卡片流：左列展示云密码与 Passkeys 硬件凭证，右列展示 10 项细粒度隐私规则与黑名单。 | 支持大屏无遮挡对比查看，操作按键 hover 态浮雕光效。 |
| **中等视口 (900–1199px)** | 采用单列响应式流，各安全分区采用可折叠卡片（Accordion）呈现，高频项默认展开。 | 减少纵向滚动距离，右侧操作按钮保持 36px 舒适点击区。 |
| **紧凑视口 (<900px)** | 单列全宽紧凑卡片，隐私规则表格自适应为卡片式列表，弹窗全屏底部抽屉化（Bottom Sheet）。 | 触摸手势友好，移动端虚拟键盘弹起时不遮挡 PIN 码输入区。 |

**设计语义 Token 绑定表**：
- 安全卡片背景：`var(--fabushi-surface-card, #ffffff)` / 深色 `var(--fabushi-surface-card-dark, #1c1d22)`
- 全局防窥遮罩：`var(--fabushi-surface-veil, rgba(15, 17, 23, 0.85))`
- 成功/已保护指示：`var(--fabushi-status-success, #10b981)`
- 警告/待重置状态：`var(--fabushi-status-warning, #f59e0b)`
- 危险/错误阻断：`var(--fabushi-status-error, #ef4444)`
- 边框与分割线：`var(--fabushi-border-subtle, rgba(226, 232, 240, 0.12))`

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 前端 TypeScript 接口契约（Zero `any`）

```typescript
// frontend/src/production/security/contracts.ts

export type PrivacyOptionKey = "AllowAll" | "ContactsOnly" | "DisallowAll";

export interface PrivacyRuleCustomConfig {
  readonly option: PrivacyOptionKey;
  readonly always: readonly string[]; // Allowed user IDs
  readonly never: readonly string[];  // Disallowed user IDs
}

export interface UnifiedPrivacySettings {
  readonly phoneNumber: PrivacyRuleCustomConfig;
  readonly lastSeen: PrivacyRuleCustomConfig;
  readonly profilePhotos: PrivacyRuleCustomConfig;
  readonly forwards: PrivacyRuleCustomConfig;
  readonly calls: PrivacyRuleCustomConfig;
  readonly callsP2p: PrivacyRuleCustomConfig;
  readonly voiceMessages: PrivacyRuleCustomConfig;
  readonly bio: PrivacyRuleCustomConfig;
  readonly birthday: PrivacyRuleCustomConfig;
  readonly giftsAutoSave: PrivacyRuleCustomConfig;
}

export interface CloudPasswordStatus {
  readonly hasPassword: boolean;
  readonly hint: string | null;
  readonly hasRecoveryEmail: boolean;
  readonly maskedEmail: string | null;
  readonly pendingResetDateMs: number | null;
  readonly attemptsRemaining: number;
}

export interface SrpClientProofPayload {
  readonly userId: string;
  readonly ephemeralA: string;      // A = g^a mod N in hex
  readonly clientProofM1: string;   // M1 = H(H(N) xor H(g) | H(U) | s | A | B | K)
}

export interface SrpServerChallengeResponse {
  readonly saltHex: string;
  readonly ephemeralB: string;      // B = (k*v + g^b) mod N in hex
  readonly srpId: string;
}

export interface PasskeyDeviceItem {
  readonly credentialId: string;
  readonly displayName: string;
  readonly createdAtMs: number;
  readonly counter: number;
  readonly transport: readonly ("usb" | "nfc" | "ble" | "internal")[];
}

export interface LocalPasscodeSettings {
  readonly isEnabled: boolean;
  readonly autoLockTimeoutSeconds: number; // 0 = disabled, 60, 300, 900, 3600, 18000
  readonly biometricUnlockEnabled: boolean;
  readonly lockOnMinimizeOrSleep: boolean;
}

export interface UnifiedSecurityPanelProps {
  readonly accountScopeGeneration: number;
  readonly currentUserId: string;
  readonly onNotice: (notice: { kind: "success" | "error" | "info"; message: string }) => void;
}

export interface LocalPasscodeModalProps {
  readonly isLocked: boolean;
  readonly remainingBackoffSeconds: number;
  readonly biometricAvailable: boolean;
  readonly onUnlockSubmit: (passcode: string) => Promise<boolean>;
  readonly onBiometricTrigger: () => Promise<boolean>;
}
```

---

### 5.2 后端 Coordinator 与 Rust 契约

```rust
// source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SrpChallengeRequest {
    pub account_scope_generation: u64,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SrpChallengeResponse {
    pub salt_hex: String,
    pub ephemeral_b_hex: String,
    pub srp_id: String,
    pub srp_group: String, // "RFC5054_4096"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SrpVerificationRequest {
    pub account_scope_generation: u64,
    pub user_id: String,
    pub srp_id: String,
    pub ephemeral_a_hex: String,
    pub client_proof_m1_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SrpVerificationResponse {
    pub verified: bool,
    pub server_proof_m2_hex: Option<String>,
    pub attempts_remaining: u32,
    pub backoff_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnRegisterChallenge {
    pub challenge_base64: String,
    pub relying_party_id: String,
    pub user_id: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnRegisterVerification {
    pub account_scope_generation: u64,
    pub user_id: String,
    pub credential_id: String,
    pub public_key_base64: String,
    pub attestation_object_base64: String,
    pub client_data_json_base64: String,
}
```

---

### 5.3 严格数学状态转移方程

#### 1. 双棘轮会话推进转移方程（Double Ratchet State Transitions）
设会话根密钥为 $RK_i$，临时私钥为 $x_i$，对端公钥为 $Y_i = y_i \cdot G$。
1. **Diffie-Hellman 棘轮步进（DH Ratchet Step）**：
   $$DH_i = \text{X25519}(x_{\text{local}}, Y_{\text{peer}})$$
   $$(RK_{i+1}, CK_{i+1}^{\text{recv}}) = \text{HKDF-Extract-and-Expand}(RK_i, DH_i, \text{"Fabushi-Ratchet-DH"})$$
2. **对称密钥链步进（Symmetric Chain Step）**：
   对于发送链中的第 $j$ 条消息：
   $$(CK_{j+1}^{\text{send}}, MK_j) = \text{HKDF-Extract-and-Expand}(CK_j^{\text{send}}, \text{"Fabushi-MsgKey-Step"})$$
   消息密文通过 XChaCha20Poly1305 加密：
   $$C_j = \text{XChaCha20Poly1305}_{MK_j}(N_{24}, P_j, \text{AAD}_j)$$
   其中 $\text{AAD}_j = \text{epoch} \parallel \text{sender\_id} \parallel \text{recipient\_id} \parallel j$。
3. **防重放滑动窗口更新（128-bit Replay Window Rule）**：
   设当前收到的最大消息序号为 $H$（Highest），位图为 $W \in \{0, 1\}^{128}$。
   - 若 $c > H$：偏移 $\delta = c - H$；若 $\delta \ge 128$ 则 $W \gets 1$，否则 $W \gets (W \ll \delta) \mid 1$；$H \gets c$。
   - 若 $c \le H$ 且 $H - c < 128$：检查 $W$ 的第 $(H - c)$ 位；若为 1 则**拒绝重放（ReplayDetected）**，否则置位为 1。
   - 若 $H - c \ge 128$：直接拒绝为**过期丢弃（StaleMessage）**。

#### 2. 本地锁屏定时器状态转移方程
设当前系统时间戳为 $t$，最后用户交互时间戳为 $t_{\text{last\_active}}$，配置的闲置超时为 $T_{\text{idle}}$：
$$\text{LockStatus}(t) = \begin{cases}
\text{Locked}, & \text{if } \text{PasscodeEnabled} \land (t - t_{\text{last\_active}} \ge T_{\text{idle}}) \\
\text{Locked}, & \text{if } \text{UserExplicitTriggerLock} \\
\text{Unlocked}, & \text{otherwise}
\end{cases}$$

#### 3. 阅后即焚（Self-Destruct）单调销毁方程
设消息权威入库时间戳为 $t_{\text{created}}$，首度被对端阅读确认（Read Receipt）时间戳为 $t_{\text{first\_read}}$，TTL 寿命为 $\Delta t$：
$$t_{\text{expire}} = t_{\text{first\_read}} + \Delta t$$
$$\text{MessageVisibility}(t) = \begin{cases}
\text{Visible}, & t < t_{\text{expire}} \\
\text{Purged}(\text{ZeroizeDiskMemory}), & t \ge t_{\text{expire}}
\end{cases}$$
一旦 $t \ge t_{\text{expire}}$，触发底座原子抹除：覆盖消息内容为全 0 字节，并物理注销关联的磁盘附件缓存。

---

### 5.4 跨账号物理隔离与内存密钥安全擦除（Zeroize）

1. **`accountScopeGeneration` 单调递增隔离**：
   每个主账号在 Electron 与 Coordinator 内部具备单调递增的世代代数。切换账号时，主进程立即广播 `ACCOUNT_REVOCATION(new_gen)`，Coordinator 与 Native Messaging 会话池清空内存中的当前会话私钥。
2. **Rust `zeroize::Zeroize` 析构物理擦除**：
   `SecretPrivateKey`、`SecretChatSession` 中的 `root_key` 及本地 PIN 派生密钥均实现 `Drop` trait，内存释放前必须调用 `self.0.zeroize()` 彻底覆写内存页，防止通过内存转储（Core Dump）泄露会话根密钥。

---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 任务切片划分

为保障多智能体并行实施的绝对隔离与零冲突，本蓝图将端到端安全与凭证保险箱工程细化为 5 个高内聚原子实施切片：

| 切片编号 | 切片目标 | 允许修改文件白名单 | 只读参考文件 | 验收准则 |
|---|---|---|---|---|
| **SLICE-12-A** | 根除伪造 `btoa` / `Date.now()` 哈希，重构两步验证云密码与 Passkeys 前端 | `frontend/src/production/security/UnifiedSecurityPanel.tsx`<br>`frontend/src/production/security/contracts.ts`<br>`frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx` (废除/重定向) | `frontend/src/production/coordinator-client.ts`<br>`source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs` | 彻底移除 `hash_btoa` 与伪盐；接入真实 WebAuthn API（`navigator.credentials`）；实现密码强度计与 10 项隐私规则设置。 |
| **SLICE-12-B** | Coordinator 端纯正 SRP-6a 协议与 WebAuthn 握手状态机实现 | `source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs`<br>`source/node-agent-coordinator/src/protocol.rs` | `source/node-agent-coordinator/src/main.rs`<br>`projects/fabushi-communication-platform/current-owner-inventory.md` | Coordinator 严格实现 SRP-6a 双向认证，服务端保存 verifier 与 salt，不再进行明文/弱哈希对比；支持 7 天重置锁。 |
| **SLICE-12-C** | 本地密码锁（Passcode）与系统闲置自动锁屏守护接线 | `source/electron-main/security/passcode-manager.ts`<br>`source/electron-main/security/passcode-ipc.ts`<br>`frontend/src/production/security/LocalPasscodeModal.tsx`<br>`frontend/src/production/ProductionRenderer.tsx` (窄接线) | `source/electron-main/secrets/secret-store.ts`<br>`source/electron-main/entry.ts` | 实现 `powerMonitor` 闲置监听；输错 5 次触发指数退避；锁屏时全屏高斯模糊与系统防窥（`setContentProtection(true)`）。 |
| **SLICE-12-D** | 双棘轮端到端加密（E2E Double Ratchet）会话管道接线 | `source/host/src/extensions/session/secret_chat_bridge.rs`<br>`source/host/src/extensions/session/native_messaging.rs`<br>`frontend/src/production/conversation/SecretChatBadge.tsx` | `native/mahayana-messaging/src/secret_chat.rs`<br>`docs/specs/fusion-blueprints/02-unified-conversation-stage-and-header.md` | 打通 `secret_chat.rs` 与会话流；支持 X25519 临时密钥协商、128 位防重放校验与密钥定期轮换（Epoch Rotation）。 |
| **SLICE-12-E** | 敏感证件保险箱（Passport Vault）与权威阅后即焚（Self-Destruct）底座 | `source/host/src/extensions/secrets/vault_manager.rs`<br>`source/host/src/extensions/transcript/transcript_store.rs`<br>`frontend/src/production/security/SensitiveContentShield.tsx` | `source/electron-main/secrets/secret-store.ts`<br>`source/host/src/host_secret_store.rs` | 敏感附件使用主密码派生 DEK 加密落盘；阅后即焚倒计时结束执行物理覆盖（Zeroize）擦除；敏感图片具备模糊保护遮罩。 |

---

### 6.2 与 `ProductionRenderer.tsx` 的窄接线规范

在主装配器 `frontend/src/production/ProductionRenderer.tsx` 中，遵循最小侵入式窄接线原则，禁止在装配器内书写复杂密码学或锁屏状态机：

```tsx
// frontend/src/production/ProductionRenderer.tsx 窄接线示例

// 1. 全局防窥锁屏模态框挂载（置于顶层 DOM 门户）
{isApplicationLocked && (
  <LocalPasscodeModal
    isLocked={isApplicationLocked}
    remainingBackoffSeconds={lockBackoffSeconds}
    biometricAvailable={isBiometricSupported}
    onUnlockSubmit={handleUnlockWithPasscode}
    onBiometricTrigger={handleUnlockWithBiometric}
  />
)}

// 2. 设置抽屉中的隐私安全面板接入（Slot 注入）
{activeSettingsTab === "privacy_security" && (
  <UnifiedSecurityPanel
    accountScopeGeneration={accountScopeGeneration}
    currentUserId={currentAccountId}
    onNotice={handleSecurityNotice}
  />
)}
```

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

### 7.1 正反例检查表（Anti-Cheating Checklist）

本蓝图设立极为严苛的独立审计标准，由独立验收角色（Auditor / Challenger）对照以下检查表逐项核验。任何一条违反均视为作弊（Cheating），一票否决并拒绝合并：

| 检查项 | 正确实现标准（Acceptance Passing） | 严禁作弊/违规行为（Forbidden Cheating） | 审计判定方式 |
|---|---|---|---|
| **密码哈希与盐值真实性** | 必须采用标准 Argon2id（至少 3 轮，64MB 内存）或 SRP-6a 计算 $x = H(s \parallel p)$，盐值长度 $\ge 16$ 字节强随机数 | 严禁使用 `btoa(password)`、`md5`、`sha1` 或拼接 `Date.now()` 充当伪哈希和伪盐 | 全文搜索检查文件：`grep -E "(btoa\(|Date\.now\(\))" frontend/src/production/security/*`；审计调用堆栈确认使用了密码学安全 RNG（`crypto.getRandomValues` / `getrandom`）。 |
| **Passkeys 硬件凭据规范** | 必须调用浏览器/Electron 提供的标准 `navigator.credentials.create()` 与 `get()` API，生成标准二进制 attestation | 使用 `btoa("pubkey_" + Date.now())` 生成假 Base64 字符串伪造 Passkey 注册 | 检查代码中是否存在真实的 WebAuthn `PublicKeyCredential` 流程，抓包检查 ClientDataJSON 是否具备挑战值签名。 |
| **SRP 零知识双向认证** | 服务端仅保存 $(s, v = g^x \pmod N)$，双方通过交换 $(A, B)$ 验证证明 $M_1/M_2$，客户端明文密码不出端 | 服务端存储明文密码或静态明文哈希，并在 RPC 请求中直接比对候选字符串 | 检查 `settings_cloud_passkeys.rs` 的验证函数，确认其包含模幂运算与 $M_1$ 证据校验，无直接相等性判定（`==`）。 |
| **本地锁屏防窥真实性** | 锁定后立即调用 `win.setContentProtection(true)`，并在根 DOM 渲染高斯模糊遮罩；快捷键与菜单全局阻断 | 仅隐藏部分文本或通过 CSS `display:none` 隐藏聊天列表，仍可通过审查元素或快捷键读取内存数据 | 触发锁屏后尝试截屏验证窗口是否被系统黑化；检查控制台是否阻断所有未授权的本地存储查询。 |
| **输错指数退避机制** | 输错 5 次后，状态机必须严格进入时间退避锁定期，期间任何提交均被底层就地拒绝，倒计时不可刷新绕过 | 输错次数存储在前端内存变量中，用户通过刷新页面（`F5` / 重启）即可清空计数器继续暴力尝试 | 在退避期间重启应用或重新挂载组件，验证主进程 `passcode-manager.ts` 是否持久保留锁定剩余秒数。 |
| **端到端加密防重放** | 严格基于 `secret_chat.rs` 中的 XChaCha20Poly1305 与 128 位滑动位图（`ReplayWindow`），重放消息必须报错拒绝 | 回退到普通明文传输，或对重复序号的消息静默接受并解密 | 编写测试用例重放同一序列号的数据包，断言返回 `SecretChatError::ReplayDetected`。 |
| **阅后即焚物理擦除** | TTL 倒计时归零后，调用底座将本地 SQLite 消息文本覆盖为 `0x00` 字节并删除磁盘缓存，执行 `zeroize` | 仅在 UI 层将消息隐藏或标记为 `deleted=true`，磁盘文件仍完整保留原始明文 | 检查 TTL 超时后的数据库文件与附件目录，确认物理磁盘扇区已无该附件与消息文本痕迹。 |
| **代码内聚与零 Any** | 所有安全相关新增代码文件严格保持在 200–600 行以内；TypeScript 与 Rust 定义无任何 `any` 或 TODO 占位 | 产生超过 800 行的巨石安全文件，或在敏感接口中使用 `any` 逃避类型检查 | 统计行数 `wc -l frontend/src/production/security/*`；静态分析全局搜索 `any` 关键字。 |
