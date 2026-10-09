Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@cf478d37c8f57df831cdedb5621e1cf2ef069a0f (root tree fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,793; first unread=5,794 Telegram/SourceFiles/test/test_log.cpp@17c72a8ae1bc672e7c380b855887b859b78fabec; unread=10,332; unknown=15,846; omitted=0. Changed accepted-prefix orders 11/30/31/92/4599/5759/5764/5765 were explicitly re-read; lib_ui is now order 6,589; runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 are historical-only for 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e + b4d9f7f8cf580eefd553c5ce105f1d3e682de87f.

# P0 — Revision 9 source authority, inventory and traceability closure

Status: active / fail-closed  
Project: TDRP-001 Revision 9  
Parent: FBCP-001 Revision 7  
Execution: executable validation only in GitHub Actions.

## Goal

Establish one accepted exact `telegramdesktop/tdesktop` baseline and prove complete source accounting before any global migration-complete claim.

The accepted scope is recursive and includes tracked files, direct and nested gitlinks/submodules, build-time acquisitions, patches, LFS/external objects, generated-input contracts, functional resources/locales/assets, build/packaging/updater, tests, tooling, docs and licenses.

## Hard counters

P0 cannot exit until the accepted baseline proves:

- `unknown = 0`
- `unread = 0`
- `omitted = 0`

Tree enumeration is not semantic reading. Directory citation is not responsibility understanding. A generated inventory is not permission to bulk-mark rows `understood`, `mapped`, `implemented` or `verified`.

## Required machine authorities

1. `projects/telegram-desktop-rust/upstream.lock.json` — exact candidate/accepted source authority and recursive pins.
2. `projects/telegram-desktop-rust/contracts/parity-ledger.schema.json` — Revision 9 row contract.
3. a ledger instance covering every non-directory source entry and every independently owned responsibility.
4. module dossiers containing symbols, state machines, lifecycle, failure semantics, ownership and behavior oracles.
5. `projects/fabushi-communication-platform/quality/requirements-traceability-matrix.md` — requirement/oracle/invariant/test/evidence/reviewer chain.
6. GitHub Actions evidence from the same exact Fabushi HEAD.

## Bidirectional gate

Every applicable responsibility must resolve both directions:

`source file -> source symbol -> responsibility -> capability -> existing Fabushi owner -> target path/symbol -> shipping entrypoint -> tests/evidence`

and

`shipping entrypoint/target symbol -> owner -> capability/responsibility -> source symbol/file -> requirement/oracle/invariant -> current-head evidence`.

The validator must fail on orphan source entries, orphan target symbols, duplicate canonical roots, verified rows without exact-head evidence, or any row that bypasses existing-owner-first.

## Canonical composition gates

Revision 9 additionally requires:

- one ProductShell/navigation composition;
- one canonical Conversation/Message/Participant model family;
- one canonical Search owner for contextual/object/universal scopes and pickers;
- one typed ConversationCreation flow;
- existing Plugins/MCP/Marketplace/Computer/Automations/Task owners;
- Fabushi semantic design tokens and canonical component/screen contracts;
- no source-named second visual or product architecture.

## Current candidate

On 2026-10-07, live upstream `dev` was observed at `f23c37857220eb84f8559f0901ea26fb304b564b` with tree `782e61795af00688671ce3a03d3b646504e3cd5f`.

That observation is **candidate discovery only**. It does not become accepted because a human or connector saw it. GitHub Actions must independently re-read and validate the exact commit/tree, direct and nested gitlinks, changed build-time pins, root inventory and Revision 9 contracts before promotion.

Historical `33261535a0e747f125e0ed25486f01e556330677` evidence is historical only and cannot satisfy Revision 9.

## No-stall rule

A service/account/platform blocker blocks only its dependent responsibility. Continue all independent source reading, owner resolution, implementation, tests and service contracts. Do not hide blockers with mocks, fake local responses, disabled assertions or skipped release gates.

## Exit

P0 passes only when the accepted exact baseline, full recursive inventory, full semantic-read/responsibility coverage, owner resolution, schema/ledger validators and Revision 9 traceability/composition gates all pass on one exact Fabushi HEAD in GitHub Actions.

P0 exit is not product completion; production responsibilities, existing Bot regression, real services, signed packaged temporal acceptance and independent release `ACCEPT` remain separate gates.


### cf478d37 live-authority rebaseline

Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@cf478d37c8f57df831cdedb5621e1cf2ef069a0f (root tree fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,793; first unread=5,794 Telegram/SourceFiles/test/test_log.cpp@17c72a8ae1bc672e7c380b855887b859b78fabec; unread=10,332; unknown=15,846; omitted=0. Changed accepted-prefix orders 11/30/31/92/4599/5759/5764/5765 were explicitly re-read; lib_ui is now order 6,589; runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 are historical-only for 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e + b4d9f7f8cf580eefd553c5ce105f1d3e682de87f.

Direct delta: channel_earn.style replaces static negative placeholder margins with floating placeholderShiftLeft; lib_ui 91ff446 implements horizontal interpolation in InputField and MaskedInputField. Canonical TextField remains the sole product owner. Orders 5,774-5,793 were identity/order reconciled; reading alone closes no unknown.
