# FBCP Architecture Map

Status: active  
Spec: FBCP-001  
Purpose: define product responsibilities, not physical file mirroring.

## 1. Product root

```text
Fabushi Product
├── Product Shell
│   ├── Inbox
│   ├── Conversation
│   ├── Composer
│   ├── Search
│   ├── Context
│   └── Settings
├── Communication Core
│   ├── Identity
│   ├── Participants
│   ├── Conversations
│   ├── Messaging
│   ├── Media
│   ├── Calls
│   ├── Contacts
│   ├── Notifications
│   └── Provider Bindings
├── Agent Runtime
│   ├── Coordinator
│   ├── Host
│   ├── Runner
│   ├── Tasks/Subagents
│   ├── Plugins/MCP
│   ├── Computer
│   └── Automations
├── Interaction Gateway
│   ├── Explicit Context Routing
│   ├── Permissions
│   ├── Consent / Policy
│   ├── Redaction
│   └── Publish Control
└── Providers
    └── Telegram
        ├── Auth / Accounts
        ├── MTProto / TL
        ├── Sync / Updates
        ├── Telegram Storage
        ├── Media / Calls
        └── Telegram Extensions
```

## 2. Ownership rules

| Responsibility | Canonical owner |
| --- | --- |
| Model/tool execution | Agent Runtime |
| Human/Agent product identity | Product Identity |
| Telegram network identity/session | Telegram Provider |
| Product Conversation projection | Communication Core |
| Telegram peer/message/update truth | Telegram Provider |
| UI navigation/inbox/composer | Product Shell |
| Communication → Agent context | Interaction Gateway |
| Agent output publication | Interaction Gateway + provider command |
| Tasks/Computer/Automations | Agent Runtime |
| Telegram calls/media protocol | Telegram Provider |
| Generic product media presentation | Communication Core/Product Shell |

## 3. No duplicate truths

Forbidden examples:

- TelegramChatStore and FabushiConversationStore both independently editing the same product message truth
- Agent transcript and Telegram transcript silently merged by string ID
- renderer owning a second Telegram account/session state
- provider update handler directly mutating Agent memory
- Agent Runner directly calling MTProto send without product permission/publish contract

## 4. Identity mapping

Mapping is explicit:

```text
Fabushi ParticipantId
  ├── FabushiIdentity?
  ├── AgentIdentity?
  ├── TelegramUserIdentity?
  └── TelegramBotIdentity?
```

Provider identity is not replaced by product ID; both are retained.

## 5. Conversation binding

```text
Conversation
  ├── Product metadata
  ├── Participants
  ├── Product capabilities
  └── ProviderBinding
       └── Telegram
            ├── account
            ├── peer
            ├── topic/thread
            └── provider extension state
```

## 6. Agent interaction

```text
Communication Event
      │
      ├── no explicit Agent route ──> UI only
      │
      └── explicit Agent route
              ↓
      InteractionGateway
              ↓
      Permission / Policy / Redaction
              ↓
      Coordinator → Host → Runner
              ↓
      Task / Artifact / Agent output
              ↓
      Publish decision
              ↓
      Communication command / Telegram Provider
```

## 7. UI rule

Do not create top-level Telegram navigation.

Provider-specific controls appear inside the product surface when applicable.

## 8. Capability preservation

A common product abstraction may not erase Telegram-only features. Use typed extensions when needed.

The architecture is successful only when full Telegram capability and existing Agent capability coexist without duplicate ownership.
