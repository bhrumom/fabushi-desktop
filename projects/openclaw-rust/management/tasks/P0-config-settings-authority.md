# OCRM-P0-CONFIG-001 — Configuration, Schema Normalization & Single-Writer Settings Authority

Status: **Historical scoped mapping retained (6 responsibilities, 8 source reads at the historical source); current fixed-U authority, exact-head verification and acceptance pending**
Unit: `OCRM-UNIT-CONFIG-001`  
Candidate requirements: `OCRM-R04`, `OCRM-R09`, `OCRM-AC05`, `OCRM-AC17`  
Dossier: `projects/openclaw-rust/dossiers/config-settings-scoped.md`  
Historical scoped source oracle (not current authority): `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`
Fixed implementation source: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` / tree `90c8aab39972da11cfad220361781d49e1bc6700`; accepted identity and scoped evidence remain pending. Later discovery candidates do not reopen this fixed subject.
Investigated target: `bhrumom/fabushi-desktop` HEAD

## Atomic closed behavior

Configuration reads and mutations enter through canonical Fabushi settings API → check immutability / deployment mode guards → acquire serial writer mutex → traverse or update properties along safe dot notation paths with prototype pollution defense → persist atomically via temporary staging files and sync → notify subscribers of effective field changes.

**Historical readiness:** Unit recorded in `parity-ledger.json` (`OCRM-RESP-CONFIG-PATH-MUTATION-001` .. `OCRM-RESP-CONFIG-AGENT-WORKSPACE-DIRS-001`) with the earlier eight source reads. That historical claim does not describe the current fixed-U shards or confer current authority. `source/host/tests/settings_service_contract.rs` contains source-level contract assertions; this is not a current exact-head Actions receipt and does not by itself prove production wiring. Full project closure and release acceptance remain false until every path is dispositioned and every applicable product responsibility is reviewed, implemented, wired and accepted.

## Current production batch — OCRM-CONFIG-CAS-v1 (2026-11-02)

Implement [the atomic Settings contract](../../docs/specs/config-settings-atomic-commit.md)
through the existing SettingsService and authenticated Gateway. The frozen
parent is `3b797be6b65095fd16b0a978e5fcd19a8ab504b3`, with main
`5528062177e3865eef60a073a066cca144f9e3e8` integrated. The current dossier records
new exact-U reads separately from historical counts above. Gateway hash CAS,
commit receipt and notification normalization share the writer transaction;
strict mutation reads must preserve invalid/unreadable prior bytes.

Subject `38cbc11c9fa522e69ff02627b8d0de714b2615ee`, tree
`8618ec4e2bed66385be7f825ab7356c3d09f39cf`, is pushed and read back through
GitHub. Independent source review found no new blocking defect after fixing
notification reentrancy ordering and stale roster receipt application. This
review covered actual Experiments, Wallpaper, diagnostics and roster consumers;
it did not execute tests or establish scoped verification. Actions run
`41381141155` passed 27 review and 10 materialization tests, then rejected an
existing malformed ledger reference before runtime dispatch; the recovery is
recorded in the P0 source task. Runtime verification remains not-run.

Use the existing `ocrm-p0-authority.yml` for the shared
risk contract, conservative materialization and exact-U structural gates; use
`rust-desktop-runtime.yml` with `validation_scope=ocrm-config` for the shipping
Host check and affected Settings/Gateway/inference contracts after freezing H.
An Actions-generated inventory commit creates a new H and requires its own
checks. No local or htch-runtime validation is authorized.

Remaining config equivalence includes cross-process/source-path authority,
HMAC/redaction and key lifecycle, JSON5/replacement/includes, complete model and
plugin validation, and post-publication recovery. Full source closure and all
other product/platform responsibilities remain in scope. GitHub delivery and
exact-H results are required before any verified or accepted status.

Current blockers are `OCRM-CONFIG-CAS-EXACT-H-EVIDENCE` (clear only with the
frozen subject's affected build/contracts and reviewed consumer wiring) and
`OCRM-CONFIG-FULL-SOURCE-AND-PROTOCOL-ACCEPTANCE` (clear only with scoped source
closure/risk review and the remaining configuration responsibilities above).
The first may close independently; it cannot close the second or project flags.
