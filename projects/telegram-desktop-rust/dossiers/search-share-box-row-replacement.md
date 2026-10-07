# Dossier — canonical Search participant replacement and dedupe

Status: implemented / verification-open  
Project: TDRP-001 Revision 9  
Responsibility: `TDRP-R9-SEARCH-ROW-REPLACEMENT-001`

## Source authority

- Repository: `telegramdesktop/tdesktop`
- Commit: `f23c37857220eb84f8559f0901ea26fb304b564b`
- Path: `Telegram/SourceFiles/boxes/share_box.cpp`
- Blob: `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be`
- Relevant symbols:
  - `ShareBox::prepare`
  - `ShareBox::searchByUsername`
  - `ShareBox::Inner::Inner` / `dialogsRowReplacements`
  - `ShareBox::Inner::applyChatFilter`

This dossier covers only the stale-row / replacement / filter-rebuild responsibility. It does **not** mark the rest of `share_box.cpp` read or migrated.

## Source behavior

The source keeps local filtered dialog rows and global username-search rows coherent while the backing dialog collection changes.

Observed invariants:

1. changing chat-filter scope immediately reapplies the local list and restarts/reuses username search for the active query;
2. a dialog-row replacement replaces or removes the old row from the current filtered projection instead of leaving a stale object alive;
3. if a filter changes while a query is active, the local filtered list is rebuilt from the new list;
4. global results are then cleared/refilled because their dedupe relation depends on the current local list;
5. selection/highlight state is reset before refresh;
6. asynchronous username results are tied to request/query identity so an old response cannot become the current search truth.

Failure modes prevented by that behavior include duplicate participants, an old row surviving a replacement, hidden/visibility state from the replaced row leaking into current results, and local/global search sets being deduped against different collection generations.

## Existing-owner-first resolution

Existing owner candidates inspected at the current Fabushi architecture:

- `frontend/src/production/command-palette-model.ts`
- `frontend/src/production/command-palette-provider.ts`
- `frontend/src/production/command-palette-message-provider.ts`
- `frontend/src/production/group-members-root.ts`
- `source/host/src/extensions/transcript/roster_search.rs`

Chosen owner:

- Universal Search participant projection: `command-palette-model.ts`
- Query/request freshness remains in the existing Command Palette provider lifecycle; no second Search owner was introduced.

Rejected as the primary owner:

- group-member search: object-scoped picker/member surface, not Universal Search truth;
- host roster search: backend/provider responsibility, not renderer projection identity;
- any Telegram-specific Search owner: prohibited parallel architecture.

## Fabushi production implementation

Implementation commit: `674d60bbc1e9059e6160cbfd321fbfff47612fff`

Target:

- `frontend/src/production/command-palette-model.ts`
- `dedupeCommandPaletteAgents`
- `commandPaletteEntries`

Contract:

- stable `CommandPaletteAgent.id` is the canonical participant identity at this projection boundary;
- when multiple roster objects for that identity are present during replacement, preserve the original list slot but replace its value with the newest row;
- only the canonical row is allowed into visible/hidden result projection;
- tab and fuzzy-query filtering happen after canonical identity dedupe;
- no new persistence, search root, participant root, route root, or visual root is created.

Shipping path:

`ProductShell -> ProductionRenderer -> CommandPalette -> commandPaletteEntries -> Participant result -> canonical open-agent/conversation action`

## State, lifecycle, concurrency, recovery

- State owner: existing renderer roster + Command Palette Search projection.
- Lifecycle owner: existing Command Palette/ProductionRenderer lifecycle.
- Persistence: none added; authoritative roster/session state remains upstream of the projection.
- Concurrency: the projection is synchronous; provider-side async searches continue to use existing request-id/abort stale fencing.
- Restart/reload: dedupe is recomputed from the authoritative roster on every projection, so no replacement cache survives reload.
- Cancellation: no new cancellable task is introduced.
- Security: this responsibility cannot expose an identity that was not already supplied by the authorized canonical roster/provider; it only removes stale duplicates.

## UI/IA/design disposition

- Entry class: Universal Search.
- Composition root: existing Fabushi ProductShell.
- Surface: existing Command Palette; no Telegram share dialog is ported.
- Result kind: canonical Participant/Agent projection.
- Search owner: existing canonical Search owner.
- Components/tokens: existing CommandPalette/result-row composition and existing Fabushi design-system surface; no source-specific component/token is added.
- Keyboard/a11y/navigation contracts remain those of the existing Command Palette.

## Requirement / oracle / invariant

- Requirement: `TDRP-R9-SEARCH-ROW-REPLACEMENT-001`
- Oracle: `ORA-TDRP-SEARCH-ROW-REPLACEMENT-001`
  - after a row replacement or collection-scope rebuild, a stable participant identity appears at most once and reflects the current row;
- Invariant: `INV-TDRP-SEARCH-CANONICAL-ID-001`
  - one stable participant identity maps to at most one Command Palette participant result;
- Invariant: `INV-TDRP-SEARCH-LATEST-ROW-001`
  - if stale and replacement rows coexist at projection input, the latest row controls rendered metadata/visibility.

## Tests and evidence

Focused contract:

- `desktop/e2e/tdrp-canonical-search-contract.spec.ts`
- stale + replacement row with the same id -> one result using replacement metadata;
- hidden stale row + visible replacement -> one visible result.

GitHub Actions evidence already obtained on exact heads:

- `37596344185` at `154014fb1e61c212c0cdcbfef2d3c8cbcabda8ff`: exact checkout, dependency install, shipping renderer typecheck, focused contract all success.
- `37596506426` at `cde8979d4939b2c65b6e593f4534bf6016e15799`: same Search gate success.
- `37596744958` at `cc318d81532c44f8710681039cc1dba3dafe72cc`: same Search gate success.
- `37597016700` at `d99de586173377e4618fc1956bb2f30be8f1db11`: same Search gate success.

The ledger row remains `implemented`, not `verified`, until its Revision 9 RTM row and the final current-HEAD evidence/reviewer requirements are closed. This dossier does not reduce the global `unknown`/`unread` counters for the source file.
