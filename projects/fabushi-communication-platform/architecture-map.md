# FBCP Existing-Architecture Absorption Map

Status: active  
Spec: FBCP-001 Revision 2  
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
