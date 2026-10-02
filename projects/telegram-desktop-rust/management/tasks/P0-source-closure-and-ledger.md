# P0 — Telegram Source Closure, Capability Research and FBCP Mapping

Status: active  
Project: TDRP-001  
Parent: FBCP-001  
Execution: GitHub Actions or htch-runtime only for executable checks.

## Goal

Understand the complete frozen Telegram source and convert that knowledge into provider/product capability contracts for FBCP.

This task does not build a standalone Telegram app.

## Required outputs

### A. Complete source closure

Freeze and inventory:

- root source
- recursive submodules
- build downloads
- patches
- generators
- resources
- shaders
- platform build definitions
- dependency licenses

### B. C++ production logic closure

Classify all C++ areas and identify every Telegram/desktop-app production responsibility that must become Rust.

### C. Capability graph

Map source areas to capabilities, not target files.

### D. FBCP destination map

Every capability must identify:

- FBCP product domain
- Telegram Provider responsibility
- typed Telegram extension if required
- provider-specific persistent state
- InteractionGateway implications
- data/terms class

### E. Research dossiers

Prioritize:

1. accounts/auth
2. MTProto/session
3. updates/consistency
4. provider storage/recovery
5. contacts/identity
6. dialogs/conversations
7. message lifecycle
8. groups/channels/topics
9. media
10. calls
11. UI/product behavior requirements
12. platform lifecycle

### F. Capability ledger

Use the capability schema under `contracts/parity-ledger.schema.json`.

A capability cannot advance beyond research without FBCP destination/provider mapping.

## Prohibited

- source file → target file completion mapping
- same-name Rust placeholders
- standalone Telegram product shell
- Telegram sync directly invoking Agent runtime
- claiming source reading equals implementation
- running inventory/checkers locally
- treating Rust rewrite as automatic GPL escape

## Exit

P0 passes when:

- recursive closure is complete;
- every source leaf is research-classified;
- all C++ production areas are identified;
- top-level Telegram capability graph has no unknown area;
- each researched capability has an FBCP destination/provider map;
- first dossiers exist;
- InteractionGateway/data-policy impacts are recorded;
- ledger is nonempty and fail-closed;
- executable validation, if run, is on an allowed runner and bound to exact target SHA.

P0 pass does not mean FBCP product or Telegram Provider is implemented.
