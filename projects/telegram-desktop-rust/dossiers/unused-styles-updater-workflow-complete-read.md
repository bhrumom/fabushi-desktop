# Unused styles updater workflow complete read — strict index 94

## Authority
- Upstream: `telegramdesktop/tdesktop@83f9f56347cc3421c1e3ef537c7bb6dd06ce7cd0`, tree `bb0159bfedb38dd96e579af4ffe6360b6beb7332`.
- Source: `.github/workflows/unused_styles_updater.yml@2c021d1cd09f7ac59b0e0d5ef6be2bf546167274`, 17 line slots / 330 bytes; complete read and SHA revalidated.
- Provider: `desktop-app/action_code_updater@master` -> `0a8d3ebb437e5c77cf5d066d4a54cc93d34b534a`, tree `5f51a9339cccc975911e3dcff02027395f2dee61`.
- Provider root: `action.yml@5a9c7dff415a31f18a614119809f2d396b0e47c7`, `index.mjs@322bfa9e3e874299a09c35650611c2cd753cca3f`, `check_unused_styles.py@6a061912121f49ffa386ff9692b989378b6bdd2b`; no committed package manifest, package lock, or LICENSE is present in this audited tree.
- The composite action performs `npm init -y` and unversioned runtime installs of `simple-git`, `isbinaryfile`, `@actions/core`, and `@actions/github`.
- Canonical owners: `AGENTS.md@da3e37258c74963970ff5cfc5d54671a37558b97`, `TEST_STRATEGY.md@4d2c88e118ab489d3d0339d2a61371a7ea74e72e`, `evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70`.
- Parent exact-head 14cdc784c167e8fb5a35c742145abb1a57b77dc2/tree 313ad81c1e1a3eae5960d29e5a155814eeeac4cc completed TDRP run 39891473256 with all 30 returned jobs successful (authority 12475286085 sha256:8187a40afb67a5c44476b3cd5764f1cc231fb36352b4e1078514d25403c2886a; closure 12475485949 sha256:1bb2e602f49574894e262e4f3364156f7b18348c74592464e1eea39e4fd1e197; reachability 12474953222 sha256:cb12c63e196894675c5539676a32978e6ed90e62be7a7ceddb1282895ec95a3f) and Rust desktop runtime run 39891473262 attempt 2 at 6/6 success (artifact 12475172253 sha256:0b3b352315c0191a5811c52ae92896b350dc704d69bdfdebdf4dd630c9c38e50).

## Complete-read behavior
The workflow runs monthly at 03:00 UTC on day one and also permits repository-dispatch and manual dispatch. The provider performs a shallow recursive-submodule clone with all branches visible for tdesktop. Before any tdesktop deletion proposal it requires recursive submodule discovery to return at least one path, then limits candidate definitions to Telegram while excluding submodule-owned definition paths.

The checker parses top-level `.style` variable definitions, preserves `[[maybe_unused]]`, searches `.cpp/.h/.mm` for `st::name` references and searches style files for identifier cross-references. Removal is fixed-point: remove the current unused set, rescan the rewritten tree, and repeat until no further unused definitions remain. Zero removals is a clean successful no-op.

When removals exist the provider creates `unused_styles_YYYY_MM`, commits and pushes the changes, then creates or reuses a pull request. Existing matching branch detection fences a duplicate mutation path. Pull-request API errors are caught and logged without rethrowing, so a green workflow can coexist with a pushed branch whose PR was not successfully created. Green is therefore not publication settlement.

## Existing-owner-first disposition
The transferable responsibility is source-neutral repository hygiene: exact-source observation, conservative finding semantics, protected exceptions, reviewable mutation, deterministic re-observation and fail-closed evidence. It maps to the existing Fabushi repository quality-health/task/review/evidence lifecycle and ordinary PR review. `novel_capability=false`.

Telegram `.style` syntax, the monthly branch name, provider token and GitHub PR mechanics do not create a second product style system, design-system owner, source store, task ledger, release owner, or runtime. Only a reviewed repository commit can become canonical Fabushi source.

## Fail-closed, concurrency and security conclusions
- `desktop-app/action_code_updater@master` is mutable provenance and runtime npm packages are unversioned.
- Absence of a committed LICENSE/package lock in the audited provider tree is incomplete provenance, not a legal conclusion.
- `github.token` is used for authenticated clone/push/PR administration only and cannot become product credential or evidence authority.
- Missing tdesktop submodule discovery is a hard failure that protects write ownership.
- The bounded scanner produces review proposals; it cannot prove a product responsibility obsolete.
- No concurrency group exists. Branch fencing reduces duplicate publication but is not a canonical lifecycle lock.
- Cancellation or PR failure after push may leave partial publication state; green cannot prove merge, release readiness, source closure or migration acceptance.
- Zero removals is a valid no-op, not global quality or migration-completion evidence.

## Exact traceability IDs
- Requirement: `TDRP-R9-UNUSED-STYLES-MAINTENANCE-PROVENANCE-001`
- Oracles: `ORA-TDRP-UNUSED-STYLES-EXACT-SOURCE-001`, `ORA-TDRP-UNUSED-STYLES-FIXED-POINT-001`, `ORA-TDRP-UNUSED-STYLES-NO-PUBLICATION-AUTHORITY-001`
- Invariants: `INV-TDRP-UNUSED-STYLES-NO-PRODUCT-RUNTIME-001`, `INV-TDRP-UNUSED-STYLES-SUBMODULE-WRITE-EXCLUSION-001`, `INV-TDRP-UNUSED-STYLES-GREEN-NOT-SETTLEMENT-001`
- Contracts: `CONTRACT-TDRP-UNUSED-STYLES-DETECTION-001`, `CONTRACT-TDRP-UNUSED-STYLES-REVIEWABLE-PROPOSAL-001`
- Temporal: `TEMP-TDRP-UNUSED-STYLES-MONTHLY-BRANCH-FENCE-001`, `TEMP-TDRP-UNUSED-STYLES-FIXED-POINT-001`
- Fault/recovery: `FAULT-TDRP-UNUSED-STYLES-MISSING-SUBMODULES-001`, `FAULT-TDRP-UNUSED-STYLES-PROVIDER-NPM-DRIFT-001`, `FAULT-TDRP-UNUSED-STYLES-PR-ERROR-SWALLOWED-001`
- Security: `SEC-TDRP-UNUSED-STYLES-GITHUB-TOKEN-001`, `SEC-TDRP-UNUSED-STYLES-MUTABLE-SUPPLY-CHAIN-001`
- Regression: `REG-TDRP-UNUSED-STYLES-NO-RUNTIME-001`, `REG-TDRP-UNUSED-STYLES-OWNER-REUSED-001`

Closing strict index 94 changes counters to unknown=16,280, unread=16,255, omitted=0, responsibilities=112. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 95 `.github/workflows/user_agent_updater.yml@1e3dae8182aefc4e4c76df065f547a43a1e50437`.
