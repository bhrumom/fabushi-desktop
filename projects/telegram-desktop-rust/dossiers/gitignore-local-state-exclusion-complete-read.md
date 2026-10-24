# .gitignore local/generated-state exclusion complete read

## Exact source and parent evidence
- Upstream authority: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Strict99 source: `.gitignore@320c160d093a0a36ceb3f6437500d01a63d5ed00`, 76 lines / 1,013 bytes, fully read.
- Parent Fabushi exact HEAD: `4c5586e55c2f04e8de976786f1e1edf36abea517`, tree `c1be8ddabd4a165ad1fda4cf2c7815bcd1026394`.
- Parent TDRP run `39901550267`: 61/61 success; authority `12479400427` `sha256:b38f52e425ec45dc338bc7cc20abdde98ebdd81e783a926247733d46c57ef770`; closure `12478689844` `sha256:3831ca464a236aac5e45a57e50b956fe67139ce7bb32c0fea7692e45fa79db13`; reachability `12478739861` `sha256:84ef4a942845f6b50ee447fc7e299311e9a49700e563ac23dbadf972051f1194`.
- Parent Rust desktop runtime run `39901550319`: 6/6 success; artifact `12479047737` `sha256:fa3ce1ef9b4d08a48ee763e5a28983d1d67b44d8923883366cf5fbb18deb58e5`.

## Responsibility
The upstream file is repository worktree hygiene for local/generated/build/IDE/cache/session/local-configuration material. Ordered ignore semantics matter, including `.cursor/*` with `!.cursor/rules/`. Ignore matching affects ordinary untracked discovery/addition only: it does not delete bytes, untrack already tracked source, authorize TDRP inventory omission, establish product persistence/recovery, prove secret safety, or replace exact-head GitHub Actions acceptance.

## Existing-owner-first
Current Fabushi exact HEAD has no root `.gitignore`; Telegram path patterns are not copied mechanically. The responsibility maps to existing root `AGENTS.md@da3e37258c74963970ff5cfc5d54671a37558b97` development-environment/source-integrity governance and existing exact-head source-authority validators. Preflight found zero prior source-path, source-blob, or `TDRP-R9-REPOSITORY-LOCAL-STATE-EXCLUSION-PROVENANCE-001` rows; each selected AGENTS target statement occurs exactly once and has zero prior target-path+symbol ownership rows. No second local-state owner, source inventory, persistence store, credential owner, build authority, or acceptance root is introduced.

## Fail-closed boundaries
Over-broad ignore rules can hide new untracked material from ordinary status, so recursive source inventory remains independent. Tracked content remains tracked even if a pattern matches. Ignored `.env` or local configuration reduces accidental addition but is not access control or secret storage; tracked/force-added/logged/copied/uploaded bytes can still leak. Different worktrees may retain different ignored state, but it cannot serialize or advance canonical source/product/release state.

## Traceability
- Requirement: `TDRP-R9-REPOSITORY-LOCAL-STATE-EXCLUSION-PROVENANCE-001`
- Oracles: `ORA-TDRP-GITIGNORE-TRACKED-SOURCE-PRESERVATION-001`, `ORA-TDRP-GITIGNORE-NEGATION-ORDER-001`, `ORA-TDRP-GITIGNORE-NO-ACCEPTANCE-AUTHORITY-001`
- Invariants: `INV-TDRP-GITIGNORE-NO-INVENTORY-OMISSION-001`, `INV-TDRP-GITIGNORE-NO-PRODUCT-PERSISTENCE-OWNER-001`, `INV-TDRP-GITIGNORE-NO-SECRET-GUARANTEE-001`
- Contracts: `CONTRACT-TDRP-GITIGNORE-UNTRACKED-HYGIENE-001`, `CONTRACT-TDRP-GITIGNORE-CURSOR-RULES-EXCEPTION-001`, `CONTRACT-TDRP-GITIGNORE-EXACT-HEAD-BOUNDARY-001`
- Faults: `FAULT-TDRP-GITIGNORE-OVERBROAD-PATTERN-001`, `FAULT-TDRP-GITIGNORE-TRACKED-PATH-MISCLASSIFICATION-001`, `FAULT-TDRP-GITIGNORE-SECRET-FORCE-ADD-001`
- Security: `SEC-TDRP-GITIGNORE-LOCAL-CONFIG-NOT-SECRET-BOUNDARY-001`
- Regression: `REG-TDRP-GITIGNORE-NO-SOURCE-OMISSION-001`, `REG-TDRP-GITIGNORE-NO-LOCAL-ACCEPTANCE-001`

Closing strict99 yields unknown=16,277, unread=16,252, omitted=0, responsibilities=117. Next strict root is index 100 `.gitmodules@2a71e861fab57dfd74d66e787867f2ede203a099`. Fresh child exact-head TDRP/Rust evidence is required.
