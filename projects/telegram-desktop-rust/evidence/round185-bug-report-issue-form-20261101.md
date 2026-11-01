# Round 185 — fixed-U repository bug-report Issue Form

- Task round: 154
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `310f9f40c7b87818751cc1f1d5a9045c01dbfff3`
- Spec-first desktop commit: `cc3a7b5b7a30fec9a07f61381bd2eb57a00cb3c6`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, complete, 7,194 entries
- Closed source: `.github/ISSUE_TEMPLATE/BUG_REPORT.yml@847b56e543fc28586ad281640218dfe8954d8bee`
- Tree index: 107 zero-based; mode 100644
- Source identity: 2,112 Git bytes / 2,100 Unicode code points / 74 LF newlines / 0 CR / trailing newline / 75 split lines
- Responsibility: `TDRP-R9-REPOSITORY-BUG-REPORT-ISSUE-FORM-001`

## Verified source semantics

The exact source was read through EOF. It defines a GitHub Issue Forms bug-report template with automatic `bug` label intent and eight ordered controls. Six are required: reproduction steps, expected behavior, actual behavior, operating system, Telegram Desktop version, and installation source. Crash ID and Logs are optional; installation source is a required single-select dropdown with seven choices.

## Validation, environment and reproduction

The source's machine-enforced schema is limited to `required: true` and the dropdown option set. Exact-OS/no-kernel-version, exact-version/not-`LATEST`, and Linux package-manager support routing are human/maintainer guidance, not regex/custom validation. Later evidence must not invent a rejection path absent from the YAML.

Reproduction quality is split into steps, expected result and actual result. Environment quality is split into OS, exact app version and installation source. Telegram-specific package-support routing remains provenance only and does not become a Fabushi support rule.

## Diagnostics, privacy and security

Crash ID is optional and its source instructions use Telegram's beta-version workflow. Logs are optional and its source instructions use Telegram's `viewlogs` cheat code. `render: shell` formats pasted text only; it does not execute it.

The source declares no redaction warning, protected security-report route, consent checkbox, secret detector, automated log collector, telemetry uploader or diagnostic retention service. Existing Fabushi repository privacy/security governance therefore remains authoritative: sensitive diagnostics are redacted/omitted/protected, and reporter-controlled text is inert.

## Direct consumers and existing-owner-first disposition

GitHub Issue Forms is the direct host consumer; bug reporters and maintainers/triagers are the human consumers. The current Fabushi checkpoint has no `.github/ISSUE_TEMPLATE` directory. Ownership remains root repository issue/reporting/development-process + privacy/security + shared quality/test-evidence governance. No Telegram issue/support subsystem, second issue tracker, crash/log product store, product UI/runtime or platform-core service is introduced.

## Failure/recovery and deferred qualification

Missing required fields remain host-blocked. Ambiguous `LATEST` or kernel-only environment text can satisfy a nonempty required field and therefore remains a triage-quality failure, not a proven schema rejection. Sensitive logs, routing/label drift or executable treatment of rendered text remain non-passing until corrected through existing owners and freshly qualified on the exact frozen candidate.

Authored only, not run: ten contract cases, one integration case, three fault-recovery cases and two security/privacy cases. Ledger state is `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`. Coverage moves exactly one independent source entry: unknown `16124 -> 16123`, unread `16097 -> 16096`, omitted stays 0, responsibilities `154 -> 155`, unmapped remains 0.

No Actions/check/status/workflow query or trigger; no Issue Form submission, issue creation/label mutation, crash/log acquisition, beta toggle, cheat-code invocation, reporter-log execution, build/compile/typecheck/lint/validator/test/E2E, packaging/deployment/installation/runtime evidence or acceptance was performed. The next fixed-U independent blob is `.github/ISSUE_TEMPLATE/FEATURE_REQUEST.yml@e2a8becdb9afcbd028b62378321458a2028fe348`.
