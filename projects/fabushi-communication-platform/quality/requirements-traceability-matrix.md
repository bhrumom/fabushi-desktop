# Requirements Traceability Matrix

Status: active / normative schema  
Matrix ID: FQT-RTM-001  
Parent: FBCP-001 Revision 7  
Updated: 2026-10-07

## Rule

每个 applicable requirement/AC 必须追到 oracle/invariant、test case IDs、execution evidence 与 verdict；test 也必须反查 requirement。`TBD` 仅可 planning，不能 verified/release。

## Required columns

`requirement_id | risk | oracle_ids | invariant_ids | unit/property | contract/integration | e2e/temporal | ui/visual/a11y | perf/security | regression_ids | evidence_ids | reviewer | verdict`

## AC coverage plan

| AC | Primary suites/gates |
| --- | --- |
| AC-01 | ARCH-CANON, UI-ROOT |
| AC-02 | FUNC-NATIVE-SVC, SEC-NET |
| AC-03 | MAP-OWNER, ARCH-WIRING |
| AC-04 | ARCH-NO-PARALLEL |
| AC-05 | FUNC-CAPABILITY, RTM-COVERAGE |
| AC-06 | UI-CONVERSATION, FUNC-MIXED |
| AC-07 | BOT-REGRESSION |
| AC-08 | ARCH-ADR |
| AC-09 | PROP-PROTOCOL, TEMP-RECOVERY |
| AC-10 | ARCH-LANGUAGE |
| AC-11 | PROP-CANONICAL-TRUTH, INT-PROJECTION |
| AC-12 | BOT-REGRESSION |
| AC-13 | EVIDENCE-IDENTITY |
| AC-14 | PKG-ACCEPTANCE |
| AC-15 | INVENTORY-TRACE |
| AC-16 | UI-IA, UI-A11Y |
| AC-17 | BRAND-SCAN |
| AC-18 | FUNC-SERVICE-E2E |
| AC-19 | SEC-PRIVACY |
| AC-20 | PERF-SOAK |
| AC-21 | FUNC-MIGRATION-ROLLBACK |
| AC-22 | STUB-BLOCKER-SCAN |
| AC-23 | BASELINE-DIFF |
| AC-24 | RELEASE-INDEPENDENT |
| AC-25 | ARCH-COMPOSITION |
| AC-26 | ARCH-NOVEL-CAPABILITY |
| AC-27 | UI-ENTRY-ROUTE |
| AC-28 | UI-CREATION-CONVERSATION |
| AC-29 | FUNC-SEARCH, ARCH-SEARCH |
| AC-30 | FUNC-SEARCH-NEG, PROP-SEARCH |
| AC-31 | UI-INTERACTION, UI-STATE-CONTINUITY |
| AC-32 | UI-TOKEN-LINT |
| AC-33 | UI-COMPONENT-OWNER |
| AC-34 | UI-SCREEN-STATE |
| AC-35 | VISUAL-SYSTEM |
| AC-36 | VISUAL-MATRIX, UI-A11Y |
| AC-37 | VISUAL-REGRESSION |
| AC-38 | UI-LARGE-DATA, PERF-UI |
| AC-39 | BRAND-ASSET-COPY |
| AC-40 | UI-DESIGN-EXCEPTION |
| AC-41 | RTM-COVERAGE |
| AC-42 | UNIT-PROP-CONTRACT-INT-PKG |
| AC-43 | ORA-TURN-001, TEMP-TURN |
| AC-44 | ORA-ORDER-001, ORA-RECON-001, PROP-RECON |
| AC-45 | UI-FUNCTIONAL, VISUAL-TIMELINE, EXP-REVIEW |
| AC-46 | DEFECT-REGRESSION |
| AC-47 | FAULT-RECOVERY, FLAKE-GATE |
| AC-48 | PERF-SOAK |
| AC-49 | SEC-PRIVACY, UI-A11Y |
| AC-50 | RELEASE-ENTRY-EXIT, RELEASE-REPORT |

实际执行时每行展开 concrete case IDs + run/artifact；本表目前定义 coverage plan，不声称任何 AC 已通过。


## Revision 9 concrete responsibility rows

These rows are additive to the AC coverage plan. A row at `IMPLEMENTED` is not a release verdict.

| requirement_id | risk | oracle_ids | invariant_ids | unit/property | contract/integration | e2e/temporal | ui/visual/a11y | perf/security | regression_ids | evidence_ids | reviewer | verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TDRP-R9-SEARCH-ROW-REPLACEMENT-001 | high | ORA-TDRP-SEARCH-ROW-REPLACEMENT-001 | INV-TDRP-SEARCH-CANONICAL-ID-001; INV-TDRP-SEARCH-LATEST-ROW-001 | PROP-TDRP-SEARCH-ROW-REPLACEMENT-001 | CONTRACT-TDRP-CANONICAL-SEARCH-001 | E2E-TDRP-CANONICAL-SEARCH-001 | UI-TDRP-CANONICAL-SEARCH-RESULT-001 | SEC-TDRP-SEARCH-AUTHORIZED-INPUT-001 | REG-TDRP-SEARCH-DUPLICATE-PARTICIPANT-001 | commit:674d60bbc1e9059e6160cbfd321fbfff47612fff; workflow:37597016700@d99de586173377e4618fc1956bb2f30be8f1db11 | pending-independent-review | IMPLEMENTED |

### TDRP-R9-SEARCH-ROW-REPLACEMENT-001 oracle

- `ORA-TDRP-SEARCH-ROW-REPLACEMENT-001`: when an authoritative participant/dialog row is replaced or a collection scope is rebuilt, the current Search projection contains at most one result for the stable participant identity and projects the replacement row.
- `INV-TDRP-SEARCH-CANONICAL-ID-001`: one stable participant id may produce at most one canonical Command Palette participant result.
- `INV-TDRP-SEARCH-LATEST-ROW-001`: if an old and replacement row coexist at projection input, replacement metadata/visibility wins.
- `PROP-TDRP-SEARCH-ROW-REPLACEMENT-001`: replacement is idempotent under repeated duplicate rows and preserves the first canonical list position while updating the row value.
- `CONTRACT-TDRP-CANONICAL-SEARCH-001`: the behavior is implemented in the existing Command Palette Search owner; no Telegram-specific or second Search root is permitted.
- `E2E-TDRP-CANONICAL-SEARCH-001`: `desktop/e2e/tdrp-canonical-search-contract.spec.ts` plus shipping renderer typecheck in `.github/workflows/tdrp-search-responsibility.yml`.
- `SEC-TDRP-SEARCH-AUTHORIZED-INPUT-001`: dedupe may only project identities already supplied by the authorized canonical roster/provider; it cannot manufacture or broaden result visibility.
- Dossier: `projects/telegram-desktop-rust/dossiers/search-share-box-row-replacement.md`.

The row is intentionally not `VERIFIED`: final current-head evidence, artifact-bound execution evidence, and independent reviewer acceptance are still open.
