# P0 — Existing Owner Inventory and Telegram Capability Absorption Plan

Status: active  
Project: FBCP-001 Revision 2  
Execution: executable checks only on GitHub Actions or htch-runtime.

## Goal

Before adding communication architecture, prove where each Telegram-derived capability belongs in the **existing** Fabushi / PR #20 architecture.

## A. Freeze current architecture input

Read current exact HEAD for:

- main
- PR #20 / canonical branch

Inventory real owners and production entrypoints for:

- sidebar/navigation
- conversation workspace
- transcript/cards
- composer/drafts
- Shared Room/group members
- reactions
- attachments/artifacts
- permissions
- search
- settings
- notifications/platform
- calls if present
- Coordinator/Host/Runner
- Computer
- Plugins/MCP
- Automations
- persistence/recovery

Do not infer owners from old chat memory.

## B. Telegram capability graph

Use TDRP-001 research to enumerate every fixed-baseline Telegram capability.

## C. Owner resolution matrix

Every capability row must contain:

- capability_id
- source research
- current Fabushi owner candidates
- selected `existing_owner`
- why selected
- owner changes required
- state added/changed
- UI surface changes
- native-network requirements
- persistence/recovery impact
- tests
- blockers

## D. New owner exception

If `existing_owner = none`, the row must additionally contain:

- all rejected owner candidates
- reason each is unsuitable
- `new_owner_proposal`
- minimal state responsibility
- ADR path

P0 must reject a broad parallel Communication Core.

## E. Native network infrastructure

Derive minimum infrastructure contracts needed for Fabushi-owned communication:

- identity/auth
- messaging
- sync
- presence
- media
- call signaling
- push

Do not decide that all of them need separate processes/services before ADR.

## F. Model evolution

Specify incremental changes to existing:

- conversation
- transcript/message
- participant/member
- resource/attachment/artifact
- composer
- permissions
- automation actions

Do not create Telegram-prefixed product models.

## G. First vertical slice contract

Existing shell → Human identity → existing sidebar → existing conversation workspace → existing composer → native durable message send/receive → existing transcript → explicit Agent action → existing Coordinator/Host/Runner → Agent result/artifact → same workspace → restart recovery.

## Exit criteria

P0 passes only when:

- current owner inventory is exact-head backed;
- top-level Telegram capability graph has no unknown domain;
- every researched capability has owner resolution;
- every new owner proposal is minimal and ADR-backed;
- no parallel product architecture appears in the plan;
- native network requirements are explicit;
- first vertical slice has concrete interfaces/tests;
- executable validators, if run, use allowed runners.

P0 does not mean communication features are implemented.
