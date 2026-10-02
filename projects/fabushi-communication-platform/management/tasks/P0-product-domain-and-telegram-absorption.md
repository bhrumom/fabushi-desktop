# P0 — Product Domain + Telegram Capability Absorption Architecture

Status: active  
Project: FBCP-001  
Execution rule: executable checks only on GitHub Actions or htch-runtime.

## Goal

Turn the product direction into implementable contracts before building Telegram UI or protocol code.

## Workstream A — Bot foundation inventory

Read current PR #20 / canonical Bot architecture and record:

- Conversation/Agent model
- Shared Room/Human-Agent membership
- Coordinator/Host/Runner
- transcript/composer
- Computer
- Plugins/MCP
- Automations
- permissions
- persistence
- lifecycle/recovery

Do not assume a prior PR HEAD is current.

## Workstream B — Telegram capability graph

Consume TDRP-001 research and enumerate every Telegram capability required by the fixed baseline.

Each capability must identify:

- product destination
- provider responsibility
- generic vs Telegram-specific semantics
- C++ Rust-replacement requirement
- data/AI policy implications
- test/real-account/platform requirements

## Workstream C — Unified domain contracts

Define ADR/contracts for:

- Identity
- Participant
- Conversation
- ProviderBinding
- Message
- Thread/Topic
- Attachment
- Reaction
- Call
- Task linkage
- Artifact linkage
- Agent membership
- permissions

## Workstream D — InteractionGateway

Define:

- explicit invoke actions
- @Agent routing
- selected context/attachment handoff
- policy/consent
- provenance
- redaction
- model/provider routing
- publish-back semantics
- automation restrictions

## Workstream E — Data ownership

Specify exactly:

- Telegram provider state owner
- product projection owner
- Agent state owner
- storage transactions
- restart recovery
- ID mapping
- stale session/epoch behavior
- deletion/retention propagation

## Workstream F — Unified UX

Design one product:

- Inbox
- Conversation
- Composer
- Search
- Context panel
- Settings

No separate Telegram workspace.

Define state examples for:

- human private chat
- Agent chat
- group
- channel
- topic
- hybrid Human+Agent room
- bot/miniapp
- active Computer/task

## Workstream G — Terms / privacy policy boundary

Architecture must support current Telegram Client API/Bot/Business policies without hardcoding assumptions that may change.

Default fail-closed:

- no implicit Telegram → model transfer
- no AI scraping/global memory
- no silent send as user
- no third-party model disclosure without allowed policy + user authorization

## Required ADR backlog

- ADR-FBCP-DOMAIN-MODEL
- ADR-FBCP-IDENTITY
- ADR-FBCP-CONVERSATION
- ADR-FBCP-PROVIDER-INTERFACE
- ADR-FBCP-TELEGRAM-BINDING
- ADR-FBCP-INTERACTION-GATEWAY
- ADR-FBCP-PERMISSIONS
- ADR-FBCP-DATA-OWNERSHIP
- ADR-FBCP-UNIFIED-UX
- ADR-FBCP-STORAGE
- ADR-FBCP-SEARCH
- ADR-FBCP-MEDIA-CALLS
- ADR-FBCP-TERMS-PRIVACY

## P0 exit

P0 passes only when:

- PR #20 current architecture is understood;
- Telegram capability graph has no unknown top-level product area;
- unified domain contracts exist;
- provider-specific extension strategy exists;
- InteractionGateway is specified;
- data/permission ownership is explicit;
- unified UX is specified;
- no duplicate state owner remains in the design;
- first vertical slice has exact interfaces and acceptance scenarios.

P0 does not require a Telegram demo app and must not create one.

## First implementation slice after P0

Fabushi product shell → Telegram sign-in → Telegram Provider → unified Inbox → human private chat send/receive → explicit Ask Agent → existing Agent Runtime → Artifact → user-controlled publish back → restart recovery.
