# Stale issue maintenance workflow complete read — strict index 93

## Authority
- Upstream: `telegramdesktop/tdesktop@83f9f56347cc3421c1e3ef537c7bb6dd06ce7cd0`, tree `bb0159bfedb38dd96e579af4ffe6360b6beb7332`.
- Source: `.github/workflows/stale.yml@0d57eb7dbd9a9f758e1e33d45afbd1aefd212614`, 25 line slots / 813 bytes; complete read and SHA revalidated.
- Provider tag: `actions/stale@v11`, audit-time immutable resolution `4391f3da665fdf50b6810c1a66712fb9ba21aa93`, tree `fd7ac6d5fc9b8173e8218f51d9647b2055c6f0c9`.
- Provider production/source identities: `action.yml@b3354e9d52f319ff836a35b299bc1f266ed0f480`; `dist/index.js@3171f8fb5b6a9388ff5524197c02723fcf801875`; `src/main.ts@228dbc9169c7e94eb38717a912af828395e57ec8`; `src/classes/issues-processor.ts@62189bf16f29e6c6dd55800bc9258cb8a9a9fbf9`; `src/services/state.service.ts@710891cbc5ff0872d5e3b003314b0a6a175195de`; `src/classes/state/state.ts@362f245d30d0765583361828ff7f5f636323a415`; `src/classes/state/state-cache-storage.ts@ee3e9b11b7be0a209ed33f520706d5966253dff6`.
- Provider package provenance: `package.json@90c287625590b9b083f05a4cbf9dcb51c9c88de6` version 11.0.0; `package-lock.json@22d1317b390f40adb3212ed7baf6c442ad227185`; `LICENSE@5dad4d23768e80045d5067599d233a3f7cfe138d` MIT.
- Canonical defect owner: `FQT-DEF-001@b31ecd424cac2abf8a2322e2996bb1b53c561378`.
- Parent exact head/tree: `b762e52394aa93b2cb593af0738b504aaf6624fc` / `3712ea3dbe69d818ddb4bbad9a4e09eaf39b24e0`.
- Parent exact-head b762e52394aa93b2cb593af0738b504aaf6624fc completed TDRP run 39889852905 at 61/61 success (authority 12474003752 sha256:a77578b59b1767023a6a8bcb5ae843f2f5eaef255cc9dac7984ba69d3a6be684; closure 12474795600 sha256:26475ac3006a066da3c2f84a82e93dac2348c6641abf22457175d386cee0465e; reachability 12474577509 sha256:12cd41f4d0bac17a4e27026c4defe4d52e3c5efb7294ba2d03be6066460c428a) and Rust desktop runtime run 39889852847 at 6/6 success (artifact 12473519648 sha256:521d53d832e1b2741187f23bb6d88991fc4a930b699563b30a63b8da2f931ad2).

## Complete-read behavior
The workflow runs daily at 01:30 UTC. It delegates to `actions/stale@v11` with a Telegram-specific stale message, label `stale`, exemption label `enhancement`, issue stale age 180 days, close delay 30 days, pull-request stale disabled with `days-before-pr-stale: -1`, and an operation budget of 1000.

The provider defaults `remove-stale-when-updated=true` and `close-issue-reason=not_planned`. It pages open issues in batches of 100, consumes an explicit API-operation budget, skips closed/locked/exempt items, and uses cache-backed state to remember already processed issue IDs across invocations when one run does not finish the repository. Cache restore/save/delete therefore belongs to provider maintenance state, not canonical Fabushi defect truth.

Provider mutation functions intentionally catch and log some comment, label and close API errors instead of failing the whole action. Consequently a green workflow run cannot prove that every intended stale/close mutation settled. Likewise the operation budget and cache continuation mean one successful invocation is not evidence that all eligible issues were processed.

The workflow declares no explicit `permissions` block and passes no `repo-token`, so the provider uses its default `github.token` and actual provider authority is bounded by repository workflow-token defaults. The workflow also declares no concurrency group; overlapping schedule/manual reruns and cache/provider mutations must be treated as provider state races, not canonical defect transitions.

## Existing-owner-first disposition
This responsibility maps to the existing source-neutral defect/regression lifecycle, with GitHub stale state as non-authoritative repository maintenance only. `novel_capability=false`. The upstream 180/30-day policy, exemption label, provider cache, comments and close reason do not create a second Fabushi defect ledger, support owner, lifecycle root or release gate.

A provider issue may be marked stale, unstaled, skipped or closed while the canonical Fabushi defect remains open. FQT-DEF-001 still requires the full defect record, root cause/fix/regression evidence and current-head validation before canonical closure. Provider automation cannot erase or substitute those obligations.

## Fail-closed, concurrency and security conclusions
- `actions/stale@v11` is a mutable major tag in source; audit-time resolution is recorded, but future provider resolution drift remains supply-chain provenance.
- `github.token` is an administrative provider credential only; workflow-token permission defaults must not be promoted to product/runtime authority.
- Enhancement-labeled issues are exempt from this provider maintenance policy; Automatic PR stale marking is disabled; an already-stale PR can still enter provider stale/close processing.
- Operation-budget exhaustion and cache continuation mean a green run can represent partial repository coverage.
- Comment/label/close API errors can be logged and swallowed; green execution is not per-item mutation-settlement evidence.
- No concurrency group exists, so overlapping provider runs/cache transitions are not canonical lifecycle ordering.
- Provider issue closure reason defaults to `not_planned`; that semantic must never overwrite canonical FQT cause/fix/closure truth.

## Exact traceability IDs
- Requirement: `TDRP-R9-ISSUE-STALE-MAINTENANCE-PROVENANCE-001`
- Oracles: `ORA-TDRP-STALE-180-30-POLICY-001`, `ORA-TDRP-STALE-EXEMPTION-001`, `ORA-TDRP-STALE-NOT-DEFECT-CLOSURE-001`
- Invariants: `INV-TDRP-FQT-DEF-SOLE-CLOSURE-OWNER-003`, `INV-TDRP-STALE-GREEN-NOT-SETTLEMENT-001`, `INV-TDRP-STALE-CACHE-NOT-CANONICAL-001`
- Contracts: `CONTRACT-TDRP-STALE-PROVIDER-ADAPTER-001`, `CONTRACT-TDRP-STALE-OPERATION-BUDGET-001`
- Temporal: `TEMP-TDRP-STALE-CACHE-CONTINUATION-001`, `TEMP-TDRP-STALE-CONCURRENT-RUN-001`
- Fault/recovery: `FAULT-TDRP-STALE-MUTATION-ERROR-SWALLOWED-001`, `FAULT-TDRP-STALE-CACHE-SAVE-001`, `FAULT-TDRP-STALE-BUDGET-EXHAUSTION-001`
- Security: `SEC-TDRP-STALE-GITHUB-TOKEN-BOUNDARY-001`, `SEC-TDRP-STALE-MUTABLE-TAG-001`
- Regression: `REG-TDRP-FQT-DEF-REMAINS-SOLE-CLOSURE-OWNER-003`, `REG-TDRP-STALE-NO-PRODUCT-OWNER-001`

Closing strict index 93 changes counters to unknown=16,281, unread=16,256, omitted=0, responsibilities=111. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 94 `.github/workflows/unused_styles_updater.yml@2c021d1cd09f7ac59b0e0d5ef6be2bf546167274`.
