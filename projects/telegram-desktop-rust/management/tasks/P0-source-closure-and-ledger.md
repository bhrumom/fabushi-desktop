# P0 — Telegram Source Research and Existing-Owner Resolution

Status: active  
Project: TDRP-001 Revision 4  
Parent: FBCP-001  
Execution: executable checks only on GitHub Actions or htch-runtime.

## Goal

Completely understand Telegram Desktop's communication capabilities and route them into the current Fabushi architecture.

## Required outputs

### A. Recursive source closure

Inventory root source, nested submodules, downloads, patches, generators, resources, shaders, platform definitions and licenses.

### B. Capability graph

Every product-relevant source region belongs to one or more capabilities.

### C. C++ responsibility inventory

Identify product logic, protocol/state machines, UI behavior, platform policy, build/runtime tools and third-party C++.

### D. Existing-owner resolution

For every capability record:

- owner candidates from current exact-head Fabushi
- selected existing_owner
- absorption plan
- model/state changes
- UX changes
- native-network requirements
- tests
- blockers

### E. New owner exception

Only when existing_owner is none:

- rejected existing owners + reasons
- new_owner_proposal
- minimal responsibility
- ADR path

### F. Research dossiers

Prioritize:

1. message/history lifecycle
2. conversation/dialog lifecycle
3. groups/members/permissions
4. drafts/composer/scheduled send
5. attachments/media
6. sync/reconnect/multi-device behavior
7. search
8. notifications
9. calls/screen sharing
10. channel/topic
11. settings/privacy
12. long-tail product capabilities

## Prohibited

- standalone Telegram product shell
- Telegram Provider
- MTProto-as-Fabushi-network
- source file → target file completion
- broad parallel Communication Core
- same-name placeholder modules
- source reading counted as implementation
- local executable validation

## Exit

P0 passes only when:

- recursive research closure is complete;
- all product source areas are capability-classified;
- C++ production responsibilities are known;
- every researched capability has exact-head owner resolution;
- any new owner proposal is minimal and ADR-backed;
- native-network implications are captured;
- ledger is nonempty/fail-closed;
- executable validation, if used, ran only on allowed infrastructure.
