# Fabushi Bot Native Communication Capability Absorption Spec

Status: active  
Spec ID: FBCP-001  
Revision: 2  
Last updated: 2026-10-02  
Owner: Fabushi Desktop  
Canonical project: `projects/fabushi-communication-platform`  
Implementation status: **not implemented / not accepted by this spec**

> **Fabushi / PR #20 的现有 Bot 架构是唯一目标架构。Telegram Desktop 只是完整通信功能、成熟行为和源码实现经验的研究来源。**
>
> 本项目不是做 Telegram，不依赖 Telegram 网络，不创建 Telegram Provider，也不新建一个与 Agent Runtime 平行的 Communication Core。我们研究 Telegram Desktop 的全部功能，然后把每个 capability 拆开，优先吸收到 Fabushi 现有 owner；只有当前架构确实没有合理 owner 时，才允许新增最小职责模块或基础设施服务。

## 1. 产品定义

Fabushi Desktop 是一个原生的 Human + Agent 通信与工作产品。

它自己的产品能力包括：

- Human communication
- Agent conversation / execution
- private chat
- groups / rooms
- channels / broadcast
- topics / threads
- messages / reactions / replies / forwards
- contacts / identity / presence
- files / media
- voice / video / calls / screen sharing
- search / notifications
- tasks / artifacts
- Computer
- Plugins / MCP
- Automations
- settings / privacy / permissions
- social/content capabilities such as Stories where accepted
- commerce/security/data-lifecycle capabilities where accepted

这些能力运行在 **Fabushi 自己的身份、存储、同步、通信网络和服务端协议** 上。

Telegram Desktop 的作用只有：

1. 完整功能清单来源；
2. 成熟产品行为与 edge-case 参考；
3. 源码研究对象；
4. 架构优缺点研究对象；
5. performance / UX / lifecycle 参考 oracle；
6. 帮助发现我们自己从零设计时容易遗漏的能力。

### 1.1 明确禁止的误解

最终架构不得出现：

- Telegram Provider 作为核心网络层；
- Telegram account/session 作为 Fabushi account 的基础；
- TelegramUserIdentity / TelegramPeerId 作为 Fabushi identity truth；
- MTProto 作为 Fabushi 自有通信协议；
- 独立 Telegram workspace / Telegram sidebar / Telegram settings app；
- 与 PR #20 平行的一整套 Communication Core；
- TelegramChat / TelegramMessage / TelegramGroup 等第二套产品模型；
- “先完整做一个 Telegram clone，再与 Bot 融合”的实施路径。

未来如果需要 **兼容 Telegram 网络**，必须作为独立 interoperability/bridge 项目另写 Spec；它不是 FBCP-001 的基础，也不能改变 Fabushi native network 的 ownership。

## 2. 权威架构：PR #20 / canonical Fabushi architecture

本 Revision 设计输入时，PR #20 `refactor/grok-018-architecture-rebuild` 的 exact HEAD 为：

`91cbe2f26e0a81b76a08c06c7cd4895f0c4b21d2`

每轮实现必须重新读取 live exact HEAD；上面的 SHA 只是 Revision 2 设计输入。

PR #20 / canonical Fabushi 已经拥有或正在建立的 product owners，包括但不限于：

- frontend / product shell
- sidebar / navigation
- conversation workspace
- transcript / transcript cards
- composer
- Agent model
- Shared Room / Human + Agent member model
- Coordinator
- Host
- Runner
- permissions / approval
- attachments
- Plugins / MCP
- Computer
- Automations
- async tasks / subagents
- settings
- desktop platform / lifecycle
- durable state / recovery

**这些 owner 是默认吸收目标。**

FBCP 不另起炉灶创建一套 parallel architecture。

## 3. Capability Absorption Law

Telegram 每一个 capability 都必须走同一个决策流程：

```text
Telegram capability
      ↓
research complete behavior
      ↓
find existing Fabushi owner
      │
      ├── owner exists
      │      ↓
      │   extend that owner
      │      ↓
      │   production wiring + tests
      │
      └── no valid owner
             ↓
      new_owner_proposal
             ↓
      prove why existing owners are wrong
             ↓
      add the smallest possible owner/service
             ↓
      integrate into existing architecture
```

### ABSORB-01 — Existing owner first

每项 capability 必须首先填写 `existing_owner`。

禁止直接因为 Telegram 有一个模块就创建新模块。

### ABSORB-02 — Extend, do not duplicate

如果已有 owner 能合理承担职责，就扩展它。

例如：

- Telegram dialog list → existing sidebar / conversation list
- Telegram history/message → existing transcript/message model
- Telegram composer → existing composer
- Telegram reactions → existing transcript reaction model
- Telegram group member → existing Shared Room / member model
- Telegram scheduled send → existing Automations / send action
- Telegram attachments → existing attachment/artifact/resource model
- Telegram Bot interaction → existing Agent interaction concepts where semantically appropriate
- Telegram settings → existing settings system
- Telegram screen sharing → existing Computer/call/realtime capabilities where ownership合理

### ABSORB-03 — New owner is an exception

只有下面都成立，才允许 `new_owner_proposal`：

1. 当前所有 existing owners 都已被分析；
2. 把职责塞入现有 owner 会破坏 cohesion / lifecycle / security / performance；
3. 新 owner 的职责无法进一步缩小；
4. dependency direction 已定义；
5. state ownership 唯一；
6. ADR 已批准。

禁止用一个宽泛的 `CommunicationCore`、`TelegramSubsystem`、`MessengerRuntime` 来一次性承接大量职责。

### ABSORB-04 — Infrastructure is not a second product architecture

Fabushi 自己的通信网络确实需要新增底层基础设施，例如：

- Messaging Service
- Presence Service
- Sync Service
- Media Service
- Call Signaling
- Push / Notification Service

这些是 **支持现有产品 owner 的基础设施**，不是第二套产品模型。

它们不能再拥有一份独立的 Conversation/Message/Identity 真相。

## 4. Telegram 功能如何进入现有 Fabushi

下面是规范性吸收方向。P0 必须以实时 PR #20 源码重新确认 exact owner。

| Telegram 能力 | 优先吸收进 Fabushi 现有 owner | 可能需要的最小新增 |
| --- | --- | --- |
| Dialog list / archive / folder | sidebar / conversation list | folder/filter domain extension |
| Private chat | conversation workspace | human conversation mode |
| History / message | transcript / transcript model | human-message variants |
| Composer | existing composer | richer send intents |
| Reply / quote / forward | transcript cards / message relations | provenance relation types |
| Reaction | existing reaction model | additional reaction semantics |
| Draft | existing draft/composer state | durable cross-device sync support |
| Scheduled message | Automations + composer | send-message automation action |
| Group | Shared Room / group model | richer room roles/state |
| Member | SharedRoomMember / participant | Human identity/profile fields |
| Admin / permissions | existing permissions / room controller | room-role policy |
| Topic / forum | conversation/thread model | topic entity if missing |
| Channel / broadcast | room/conversation model | ChannelMode / broadcast policy |
| Search | existing search/command surface | message/file index service |
| Files / media | attachments + artifacts/resources | transfer/cache service |
| Upload / download | attachment/resource lifecycle | transfer service |
| Voice / video | resource/media UI | media pipeline |
| Calls | existing product surface + platform runtime | call session/signaling owner if no current owner |
| Screen sharing | Computer + call/realtime experience | realtime sharing bridge |
| Stickers/GIF/emoji | composer / rich transcript | asset/catalog support |
| Stories | existing product shell | minimal Story owner if no fit |
| Bot commands/interactions | Agent/composer interaction | generic interaction primitives |
| Mini Apps | Plugins/MCP/Web capability | secure embedded app surface only if needed |
| Notifications | desktop lifecycle / notifications | push service |
| Settings/privacy | existing settings / permissions | communication settings sections |
| Saved Messages | personal conversation/workspace | none unless required |
| Export | artifacts/data-lifecycle | export pipeline |
| Local cache/recovery | durable state/storage owners | message/media stores |
| Multi-device sync | durable state + native network | sync service |
| Presence/typing | Shared Room / conversation projection | presence service |
| Stories/Premium/Stars/Business/Payments | map by actual responsibility | smallest domain owner after ADR |

这张表不是 target file map。它是 **ownership hypothesis**，必须根据实时架构验证。

## 5. 核心产品模型如何成长

FBCP 不要求先创建新的“通用通信模型”替换现有模型。

正确方式是扩展现有 product types。

### 5.1 Conversation

现有 Agent conversation / group concepts 应逐步能表达：

- Human private conversation
- Agent conversation
- group / room
- Human + Agent hybrid room
- channel / broadcast conversation
- topic/thread

如果当前 `Conversation` 语义不足，扩展它或拆出明确的 domain types；不要创建第二个 TelegramConversation。

### 5.2 Participant / Member

从现有 Human/Agent Shared Room member 继续演化：

```text
Participant / Member
├── Human
└── Agent
```

Fabushi 自己拥有 Human identity。

不需要 TelegramUserIdentity 作为基础模型。

### 5.3 Message / Transcript

现有 transcript 是优先 owner。

扩展以支持：

- human message
- agent message
- rich text
- reply / quote
- forward provenance
- edit/delete lifecycle
- reactions
- media/resource
- read state
- scheduled send state
- system events

Agent-specific tool/thinking/task entries继续保持 typed entries；不要为了“像 Telegram”把它们降级成纯文本消息。

### 5.4 Resource

把现有 attachment / Agent artifact / Computer artifact 与通信媒体能力逐步统一到共享 resource lifecycle：

```text
Resource
├── user attachment
├── message attachment
├── agent artifact
├── computer artifact
├── image
├── video
├── audio
└── file
```

必须保留 provenance，而不是复制成无来源 blob。

## 6. Fabushi Native Communication Network

Fabushi 通信网络是自己的基础设施，不依赖 Telegram。

### 6.1 Native identity

至少定义：

- FabushiUserId
- FabushiAgentId
- ConversationId
- MessageId
- RoomId
- ChannelId / mode where required
- DeviceId
- ResourceId
- CallSessionId

### 6.2 Native event/protocol model

协议应原生支持 Human + Agent 产品，而不是模仿 MTProto wire format。

候选 typed events 包括：

- HumanMessage
- AgentMessage
- MessageEdited
- MessageDeleted
- ReactionChanged
- MemberJoined/Left
- RoleChanged
- Typing/Presence
- TaskCreated/Completed
- ArtifactPublished
- ApprovalRequested/Resolved
- AutomationTriggered
- CallStateChanged
- ComputerHandoff

具体 wire protocol / storage encoding / transport 由 ADR 决定。

### 6.3 Native services

只有在现有 Host/Coordinator/shared/platform owner 无法承担基础设施职责时，新增最小 service：

- identity/auth service
- messaging service
- sync service
- presence service
- media/blob service
- call signaling service
- push service

这些 service 提供 infrastructure contract，不拥有 renderer product state。

## 7. Agent 与通信不是两套系统

Human communication 和 Agent execution 在产品层共享现有 conversation/workspace，而运行职责仍保持边界。

示例：

```text
Human message
    ↓
existing Conversation / Transcript
    ↓
explicit @Agent / Ask Agent
    ↓
existing permission / interaction boundary
    ↓
Coordinator
    ↓
Host
    ↓
Runner
    ↓
Task / Artifact / Agent output
    ↓
existing Conversation / Transcript
```

不需要一个独立 “InteractionGateway service” 作为新一级架构。若现有 permission/composer/coordinator-client 边界不足，可以增加一个**最小 interaction policy/adapter**，但它必须嵌入现有 flow。

## 8. UX 吸收规则

最终用户只使用 Fabushi 原有产品 shell 的演化版本。

### Sidebar

现有 sidebar 扩展为同时显示：

- Human
- Agent
- Group
- Channel
- Hybrid room

不是增加 Telegram sidebar。

### Conversation

现有 conversation workspace 扩展支持 human messaging / group / channel / topic / agent runtime。

### Composer

现有 composer 扩展支持：

- message
- attachment
- @Human
- @Agent
- reply / quote
- schedule
- task / automation
- plugin/tool affordance

### Context / settings

Telegram 中成熟的成员、媒体、权限、通知、隐私等能力，进入现有 info/settings surfaces；只有缺失时才新增最小 surface。

## 9. TDRP-001 的角色

TDRP-001 不再实现 Telegram Provider 或 Telegram 网络兼容。

它负责：

- 固定 Telegram Desktop 源码基线；
- 完整 source/dependency/resource inventory；
- capability discovery；
- behavior/state-machine/edge-case research；
- C++ production responsibility inventory；
- source-informed provenance；
- feature coverage oracle；
- 为每项 capability 输出 `existing_owner` 候选与 absorption requirements；
- 识别值得保留和应该改善的设计。

TDRP-001 的“C++ → Rust”含义是：

> 对从 Telegram C++ 产品逻辑中研究得到、最终需要进入 Fabushi 产品的职责，用更好的 Rust 或 best-fit Fabushi 实现重新实现，不把原 C++ 作为 production dependency。

它不要求 MTProto interoperability。

## 10. P0 — Absorption Architecture

P0 必须先做：

1. 读取 PR #20 / current canonical exact HEAD；
2. 建立 existing-owner inventory；
3. 完整 Telegram capability graph；
4. 对每项 capability 进行 owner resolution；
5. 输出 absorption plan；
6. 只有 owner 不存在时才创建 new-owner ADR proposal；
7. 设计 Fabushi native communication network 所需的最小 infrastructure contracts；
8. 定义 conversation/message/member/resource 的增量演化；
9. 定义 Human + Agent 共同使用现有 workspace 的 UX；
10. 定义 persistence / sync / recovery ownership；
11. 定义测试与迁移策略。

## 11. 第一条 implementation vertical slice

第一条不能是 Telegram demo，也不能先造一套 communication app。

必须是：

1. existing Fabushi app shell；
2. existing sidebar；
3. 新增一个 Fabushi Human identity；
4. sidebar 同时出现 Human conversation 与 Agent；
5. 点击 Human conversation 仍进入 existing conversation workspace；
6. existing composer 发送 HumanMessage；
7. Fabushi native messaging backend/service 完成 durable send/receive；
8. message 显示在 existing transcript；
9. 从该 Human message 显式触发现有 Agent flow；
10. Agent 结果进入同一 existing transcript / artifact flow；
11. restart/reconnect 恢复；
12. 不存在第二套 sidebar/conversation/message owner。

这条闭环通过后，再扩展 group/channel/media/calls 等能力。

## 12. Verification

所有 executable verification 只能运行在：

- GitHub Actions
- `htch-runtime`

禁止本地 build/test/lint/generator/schema/benchmark/fuzz/package/acceptance。

必须验证：

- exact-head existing owner mapping；
- no duplicate product owner；
- new owner proposals have ADRs；
- native network does not depend on Telegram；
- existing Agent workflows regressions；
- Human + Agent same-shell behavior；
- durable send/recovery；
- packaged app；
- platform/security/performance/accessibility；
- Telegram feature coverage research completeness；
- C++ source-informed responsibilities are implemented without production C++ fallback where applicable。

## 13. Acceptance Criteria

**AC-01 — Existing architecture is the only product architecture**  
PR #20 / canonical Fabushi architecture remains the single product skeleton.

**AC-02 — No Telegram network dependency**  
Fabushi identity, messaging, sync, media, calls and push operate on Fabushi-owned infrastructure/protocols. Telegram is not a runtime provider.

**AC-03 — Existing-owner-first absorption**  
Every researched Telegram capability records an existing owner first; new owners exist only with approved justification.

**AC-04 — No parallel Communication Core**  
There is no second top-level product model/runtime containing duplicate Identity/Conversation/Message truth.

**AC-05 — Complete Telegram feature coverage**  
All applicable Telegram Desktop product capabilities are researched and either absorbed or explicitly blocked with evidence; no capability disappears because it is difficult.

**AC-06 — Human + Agent unified product**  
Human, Agent, group, channel and hybrid rooms use the evolved Fabushi shell/workspace instead of separate applications.

**AC-07 — Existing capabilities are extended, not replaced casually**  
Transcript, composer, Shared Room, attachments, Automations, Computer, Plugins/MCP, settings and other owners are reused when appropriate.

**AC-08 — New owners are minimal**  
Every added product/runtime/service owner has a focused responsibility and approved ADR proving existing owners were unsuitable.

**AC-09 — Native network semantics**  
Fabushi native protocol supports communication plus Agent-native events without forcing them into Telegram wire semantics.

**AC-10 — Source-informed C++ replacement**  
Telegram/desktop-app C++ source may be studied, but the final Fabushi production responsibility uses Rust/best-fit implementation rather than the original C++ production code.

**AC-11 — No duplicate state truth**  
No parallel message, identity, room, conversation, resource or permission owner exists.

**AC-12 — Bot architecture preserved**  
Coordinator/Host/Runner/Computer/Plugins/Automations continue to satisfy their canonical requirements.

**AC-13 — Exact-head evidence**  
All acceptance evidence binds the exact target SHA and allowed runner.

**AC-14 — Packaged product acceptance**  
A packaged Fabushi build proves Human communication + Agent work are one coherent product.

## 14. Licensing / provenance

本路线仍然是 source-informed，不是 clean-room。

研究 Telegram Desktop GPL 源码后重新实现，不得声称“换成 Rust 就自动消除 GPL 风险”。所有 source-derived research、复制/改编、第三方依赖、资源和发布许可证义务必须记录并在 release 前审查。

Fabushi 自有通信网络不改变这一 provenance 事实。

## 15. Current status

本 Revision 只修正架构方向：

- existing-owner-first rule：specified
- Telegram as research source only：specified
- native Fabushi network：specified
- existing owner inventory：not complete
- Telegram capability research：not complete
- absorption mapping：not complete
- native messaging infrastructure：not implemented
- Human messaging in existing workspace：not implemented
- full feature absorption：blocked
- release：blocked
