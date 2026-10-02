# Telegram Desktop Capability Research — Source of Truth

Status: active  
Project ID: TDRP-001  
Revision: 4  
Date: 2026-10-02  
Parent product: FBCP-001

## Role

This project researches Telegram Desktop as a mature communication-product reference.

It does **not** define:

- a Telegram clone
- a Telegram Provider
- a Telegram network integration
- a second communication product architecture

Its output is:

- complete capability research
- source/dependency/resource inventory
- behavior/state-machine/edge-case knowledge
- C++ production responsibility inventory
- source provenance/licensing
- candidate existing Fabushi owners
- absorption requirements
- minimal new-owner proposals only when necessary

## Canonical rule

`Telegram capability → current Fabushi owner → absorption plan`

Only when no owner fits:

`existing_owner = none → rejected owner analysis → minimal new owner ADR`

## Fixed upstream

`telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`

## Must read

- root `AGENTS.md`
- `docs/specs/fabushi-bot-communication-platform.md`
- `docs/specs/telegram-desktop-rust-equivalence-migration.md`
- current PR #20/canonical architecture
- `module-map.md`
- capability ledger schema
- current task

## C++ rule

Study the C++ source deeply.

If a responsibility is required in Fabushi, implement that responsibility in the appropriate Fabushi owner, preferring Rust for native/runtime/state-machine logic.

Do not ship original Telegram/desktop-app C++ as the production owner.

## Network rule

Fabushi uses its own communication network/protocol.

MTProto and Telegram sync logic are research material for mature messaging behavior, not required runtime compatibility.

## Execution

All executable verification only on GitHub Actions or `htch-runtime`.
