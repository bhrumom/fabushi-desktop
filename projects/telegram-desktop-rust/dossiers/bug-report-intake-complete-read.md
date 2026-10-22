# Bug-report intake complete read — strict index 72

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Source: `.github/ISSUE_TEMPLATE/BUG_REPORT.yml@847b56e543fc28586ad281640218dfe8954d8bee`, 74 lines / 2112 bytes; complete read and SHA revalidated.
- Applicable responsibilities: reproducible steps, expected-versus-actual oracle, exact OS/environment and affected build/version, distribution/install provenance, and optional crash/log evidence.
- Provenance-only policy: Telegram product/version naming, fixed install-source list, Linux distro exclusions, beta-channel Crash ID procedure, Settings path, `viewlogs` cheat code, labels/emoji and branded wiki links.
- Current Fabushi `.github` tree contains only `scripts/` and `workflows/`; no parallel `ISSUE_TEMPLATE` owner exists.
- Existing owner evidence: `quality/defect-regression-policy.md@b31ecd424cac2abf8a2322e2996bb1b53c561378` requires ID, SHA/environment, steps, expected/actual, frequency, screens/video/logs, requirement/oracle, severity, root cause, fix SHA, regression IDs, evidence and reviewer; closure requires reproduce -> fix -> targeted regression -> affected regression -> current-head evidence.
- Parent exact head/tree: `eb648bb5281fd25acae990f43e1ebb2d06718c4a` / `4fbd3c9276b61efcfa9856f38ea61e47acd9fc3c`.
- Parent TDRP run `39735542881` completed success, 61/61; authority `12406837458` / `sha256:5deb940f4e394ae9d4b613ec414a07353b6759bb10830d3fc65e757471a44239`; closure `12406714964` / `sha256:980682d1136c7e3c2888ef6ad54d9f323250133d93dccf4196f3d796eeae2e70`; reachability `12406962101` / `sha256:92608d577c8a1f1dc48ed0841d18d82e499b50fc62282c177ca1638d37cff35d`.
- Parent Rust run `39735542846` completed success, 6/6; artifact `12406738950` / `sha256:15292be0c02d8e450eb83efb741c72351b1d08d19375cf9324d3171dfeb2663d`.

## Existing-owner-first disposition
The issue form is an optional provider adapter into the existing FQT-DEF-001 defect/regression lifecycle. It may collect structured evidence, but it cannot own defect identity, severity, root cause, regression coverage, current-head closure or release verdict. Product crash/log collectors remain their own platform/runtime owners. Telegram beta/Crash ID/viewlogs, distro and installation-source policy stay provenance. No second defect store, support service, telemetry owner, diagnostic runtime or release gate is introduced; `novel_capability=false`.

## Exact traceability IDs
- Requirement: `TDRP-R9-BUG-REPORT-INTAKE-001`
- Oracles: `ORA-TDRP-BUG-REPORT-REPRODUCIBILITY-001`, `ORA-TDRP-BUG-REPORT-EXACT-ENVIRONMENT-001`, `ORA-TDRP-BUG-REPORT-DIAGNOSTIC-PROVENANCE-001`
- Invariants: `INV-TDRP-BUG-REPORT-NO-FORM-AS-DEFECT-TRUTH-001`, `INV-TDRP-BUG-REPORT-NO-TELEGRAM-DIAGNOSTIC-CONTRACT-001`, `INV-TDRP-BUG-REPORT-NO-SUBMISSION-AS-PASS-001`
- Contracts: `CONTRACT-TDRP-BUG-REPORT-REQUIRED-FIELDS-001`, `CONTRACT-TDRP-BUG-REPORT-OPTIONAL-EVIDENCE-001`, `CONTRACT-TDRP-BUG-REPORT-CANONICAL-ROUTING-001`
- Fault/recovery: `FAULT-TDRP-BUG-REPORT-MISSING-EXACT-VERSION-001`, `FAULT-TDRP-BUG-REPORT-INCOMPLETE-REPRODUCTION-001`, `FAULT-TDRP-BUG-REPORT-SENSITIVE-LOG-001`
- Security: `SEC-TDRP-BUG-REPORT-DIAGNOSTIC-REDACTION-001`
- Regression: `REG-TDRP-BUG-REPORT-NO-SECOND-DEFECT-STORE-001`, `REG-TDRP-BUG-REPORT-NO-TELEGRAM-BRAND-POLICY-001`

Closing strict index 72 changes counters to unknown=16,231, unread=16,206, omitted=0, responsibilities=90. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 73 `.github/ISSUE_TEMPLATE/FEATURE_REQUEST.yml@e2a8becdb9afcbd028b62378321458a2028fe348`.
