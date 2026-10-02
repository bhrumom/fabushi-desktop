# Fabushi Native Communication Capability Absorption — Source of Truth

Status: active  
Project ID: FBCP-001  
Revision: 2  
Date: 2026-10-02  
Repository: `bhrumom/fabushi-desktop`

## Canonical statement

**PR #20 / canonical Fabushi architecture is the only target architecture. Telegram Desktop is a research source. Telegram features are decomposed and absorbed into existing Fabushi owners. Fabushi owns its communication network.**

## Must read

1. root `AGENTS.md`
2. `docs/specs/fabushi-bot-communication-platform.md`
3. current PR #20 / canonical Grok Bot spec
4. `docs/specs/telegram-desktop-rust-equivalence-migration.md`
5. `projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md`
6. current P0/ADR/evidence

## Hard rules

- no Telegram Provider as product/network foundation
- no MTProto dependency for Fabushi native communication
- no parallel Communication Core
- no second sidebar/conversation/message/identity truth
- existing owner first
- new owner only with approved minimal-responsibility ADR
- infrastructure services support existing owners; they do not become another product architecture
- Human and Agent share the evolved existing product shell/workspace
- all Telegram capabilities remain in research scope
- source-informed C++ responsibilities are reimplemented, not shipped as original Telegram C++
- all executable verification only GitHub Actions or htch-runtime

## Current PR #20 design input

Before this Revision was aligned, current PR #20 HEAD was:

`91cbe2f26e0a81b76a08c06c7cd4895f0c4b21d2`

Always reread the live exact HEAD before work.

## Owner resolution rule

For every Telegram capability:

`research → existing_owner → absorption_plan`

Only if no valid owner exists:

`rejected_existing_owners + new_owner_proposal + ADR`.

## Next task

`management/tasks/P0-product-domain-and-telegram-absorption.md`

P0 must inventory current owners before proposing any new communication domain.
