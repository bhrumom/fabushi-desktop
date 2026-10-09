# FBCP Existing-Architecture Absorption Map

Status: active  
Spec: FBCP-001 Revision 6  
Purpose: route researched capabilities into the current Fabushi architecture.

## 1. There is one product architecture

Do **not** model the target as:

```text
Fabushi
├── Communication Core
└── Agent Runtime
```

The target is:

```text
Current Fabushi / PR #20 architecture
├── frontend / product shell
├── sidebar / navigation
├── conversation workspace
├── transcript / cards
├── composer
├── Shared Room / members
├── permissions
├── attachments / artifacts
├── settings
├── Coordinator
├── Host
├── Runner
├── Plugins / MCP
├── Computer
├── Automations
├── platform / lifecycle
└── minimal new infrastructure only where required
```

Telegram-derived capabilities enter these owners.

## 1.1 Single-composition law

统一不是“所有逻辑放进一个模块”，而是每个产品概念只有一个 canonical root composition。Conversation、Transcript、Composer、Participant/Profile、Resource viewer/editor、Search、Settings、Marketplace 等不得按 Human/Agent/Group/Channel/Telegram 来源复制完整 root。差异以 typed capability/section/renderer/action/panel/overlay 注入。

如果 current Fabushi 完全没有某项职责，先证明 existing owner 不存在；再通过 ADR 新增最小 source-neutral capability owner。该 owner 只拥有不可再缩小的领域状态，必须通过 typed contract 接入统一 product shell，并复用 canonical identity/resource/permission/navigation。Call、Story、Payment、Presence 等可以有专门 domain owner 或 capability surface，但不能形成第二应用或第二套 conversation/profile/state truth。

## 1.2 UI information architecture and entry placement

每个迁入能力先判断用户任务作用域，而不是照搬上游页面：

```text
stable cross-object product domain  -> left primary navigation
collection browse/filter/action     -> center-list surface
create domain object                -> unified creation action
act on current object               -> header/toolbar/context/detail
immersive specialized interaction   -> capability surface/overlay
cross-domain command                -> global command (Universal Search)
background/transient state          -> system surface
```

Direct/Agent/Group/Channel/Hybrid/Topic 都属于同一 Messages domain。列表可按 kind/filter 切换；新建动作进入统一 ConversationCreation surface。Contacts 维护 Participant/contact relation；“发消息”路由到 canonical Conversation。

Search 是一个 domain owner：contextual collection search、current-object search、Universal Search（`Cmd/Ctrl+K`）以及 Participant/member/admin picker 都共享 typed query/result/provider/permission contract。模块可以贡献 provider/filter/result renderer，但不能拥有第二个 Search root/index truth。

## 1.3 Design-system ownership

UI production flow is:

```text
capability responsibility
  -> ui entry class / canonical route
  -> canonical screen pattern + slot
  -> canonical component
  -> Fabushi semantic token
  -> light/dark/responsive/a11y/motion states
  -> visual + interaction acceptance
```

A feature may not skip directly from capability to bespoke JSX/CSS if a canonical slot/component exists. New shared primitive/pattern is an architecture change: record rejected existing components, the new owner, API/state/accessibility contract, reuse target and migration plan.

Current recovered Sand/Cursor token/primitives may sit below a Fabushi compatibility adapter while legacy parity is preserved. New Telegram-derived feature code targets the Fabushi semantic layer, not source-named legacy tokens.

## 2. Owner-first examples

| Capability | First owner to inspect/extend |
| --- | --- |
| dialogs / inbox | existing sidebar/conversation list |
| private human chat | existing conversation workspace |
| messages/history | existing transcript model/cards |
| composer/drafts | existing composer/draft owner |
| reply/quote/forward | transcript/message relation model |
| reactions | existing reaction owner |
| groups | Shared Room/group owner |
| members | SharedRoomMember / group-members owner |
| room permissions | existing permissions + group controller |
| scheduled send | Automations + composer |
| files/media | attachments/artifacts/resource lifecycle |
| search | existing search/command surfaces |
| settings/privacy | existing settings/permission surfaces |
| bot-style interactions | Agent/composer interaction |
| screen share | Computer/realtime/calls owners |
| notifications | desktop/platform lifecycle |
| calls | existing calls surface or minimal new call-session owner |
| channels/topics | extend room/conversation model first |
| stories | product surface; new owner only if no fit |

## 3. New owner gate

A new owner proposal must contain:

- responsibility
- why every plausible current owner is wrong
- state owned
- commands/events
- persistence
- lifecycle
- dependency direction
- process/thread placement
- language choice
- tests
- migration/cutover

Reject proposals named only by source provenance, such as:

- TelegramCore
- TelegramRuntime
- TelegramProvider
- TelegramMessaging
- CommunicationCore

unless a future separate interoperability spec explicitly authorizes one.

## 4. Native network infrastructure

Possible new infrastructure:

```text
Fabushi native network
├── identity/auth service
├── messaging service
├── sync service
├── presence service
├── media/blob service
├── call signaling service
└── push service
```

These services are below existing product owners.

They expose typed service contracts but do not own renderer product models.

## 5. Human + Agent flow

```text
Human
  ↓
existing composer
  ↓
existing conversation/transcript
  ↓
native messaging infrastructure

Human message
  ↓
existing transcript
  ↓ explicit Agent action / mention
existing permission + coordinator boundary
  ↓
Coordinator → Host → Runner
  ↓
Agent result / Artifact
  ↓
existing transcript / resource flow
```

## 6. Definition of successful absorption

A capability is absorbed only when:

- its current owner is explicit;
- production code lives behind that owner;
- no duplicate truth exists;
- existing UX is extended rather than bypassed;
- native network support is wired where needed;
- behavior is tested;
- restart/reconnect semantics are owned;
- packaged application proves the flow.
