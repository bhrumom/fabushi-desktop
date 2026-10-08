# Composer stash exchange core

Accepted upstream: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db`.

Requirement `TDRP-R9-COMPOSER-STASH-EXCHANGE-001`, oracle `ORA-TDRP-R9-COMPOSER-STASH-EXCHANGE-001`, invariants `INV-TDRP-R9-COMPOSER-STASH-ATOMIC-001` and `INV-TDRP-R9-COMPOSER-STASH-SCOPE-001`.

The live upstream `Data::ComposeStash`/`StashManager` was read through the keyed History store and active compose-control consumers. The core behavior is a per-`DraftKey` transient slot: exchange current and stashed composer payload atomically, preserve text/rich state and prepared files, refuse an inapplicable stash before displacing current state, remove exact stash, and clear child-scope stash when that child disappears. Upstream also layers forward draft, send-files-box reclaim, send/remove menu, silent/schedule and suggestion/hint behavior on this core; those are not collapsed into this row.

Fabushi reuses its one `ComposerDraftStateStore`. `exchangeDraft` moves the full existing `ComposerDraft` object (prompt, serialized rich text, reply metadata and staged attachments) between current and transient keyed stash in one synchronous state transition. The caller supplies validation; production composition rechecks the attachment limit before restore. `clearScope` removes exact current+stash state and persists the ordinary draft mutation. Stashes are explicitly cleared on account restore/reset/dispose and never enter the persisted schema.

`ConversationComposer` remains the unique UI owner. It receives one canonical `SandIconButton` using the existing `arrow-swap` icon, labels itself Store/Swap according to stash presence, and exposes `Ctrl/Cmd+Shift+Y`; disabled and voice-busy state fence the shortcut. `ProductionRenderer.exchangeComposerStash` binds that affordance back to the store and clears the transient reply controller after a successful exchange so a stale visible reply selection cannot override the restored draft.

Focused contract IDs: `CONTRACT-TDRP-COMPOSER-STASH-EXCHANGE-001`, `TEMP-TDRP-COMPOSER-STASH-RESTORE-001`, `FAULT-TDRP-COMPOSER-STASH-INVALID-RESTORE-001`, `UI-TDRP-COMPOSER-STASH-SWAP-001`, `A11Y-TDRP-COMPOSER-STASH-SHORTCUT-001`, `SEC-TDRP-COMPOSER-STASH-SCOPE-001`, `REG-TDRP-COMPOSER-STASH-DRAFT-LOSS-001`. Tests are authored in `recovered-conversation-infra.contract.test.ts` and must execute only in GitHub Actions.

Open remainder: forward-draft ownership/integration, stashed ordinary-send/remove menu, silent-send transport, durable scheduling/reminder semantics, and full Mahayana topic/child composition. Those remain blockers and receive no implementation or verification credit from this row.
