# Issue closer routing workflow complete read — strict index 85

## Authority
- Upstream: `telegramdesktop/tdesktop@42e2af80cdd20a99edecb4255470f0aa6bae6bcb`, tree `19740740209ea84bf3768f3f63ca92ea8d861087`.
- Source: `.github/workflows/issue_closer.yml@355405165590712b1826b2f92a8f867c8d568dd9`, 15 line slots / 251 bytes.
- Provider at publication: `desktop-app/action_issue_closer@master -> b2fc72db6cc262c8e84e10740c44296ec4be2e02`, tree `a949a72cc7db2446287cc7c88e651966c1243f77`; `action.yml@91083e19bf270c6b8fe109fd264522032a5f10a1` (27 line slots / 697 bytes), `index.js@ea3b9b265e070522b56e75da86debaa80140278f` (138 line slots / 3439 bytes).
- Parent exact head/tree: `6687739f469d66b426656d1d01d3c3ea870c4093` / `b3ac507d5530648e74eaac04b3536a6f8aa6531a`.
- Parent exact-head 6687739f469d66b426656d1d01d3c3ea870c4093 completed TDRP run 39776914082 at 61/61 success (authority 12424866760 sha256:b917b4f022dbd19220e361655234df411771b4f12a1156410b7cb3992fd1e08c; closure 12424758517 sha256:0956608ba4c75dfc2a71edd705aea9cb7a31c988ecf08ae889ac25dae53ad2be; reachability 12425102856 sha256:b9320c5f9d55eb734347adf0bfeb099f2b1878f5f2d0e401f130f150a70ceb59) and Rust desktop runtime run 39776913954 at 6/6 success (artifact 12424429338 sha256:c35cfcbe332ee17673ea2a53bd467fcfaf24be813e393e1a27990b369f477dea).

## Complete-read behavior
The workflow runs only for newly opened GitHub issues and delegates the issue-maintenance decision to mutable `desktop-app/action_issue_closer@master` with `secrets.GITHUB_TOKEN`.

The composite provider first runs `wget https://osx.telegram.org/updates/versions.xml || exit 0`. It then creates an npm package at runtime and installs `compare-versions`, `fast-xml-parser`, `@actions/core` and `@actions/github` without a reviewed lock or explicit versions before invoking `index.js`.

The JavaScript reads the opened issue body between the headings `Version of Telegram Desktop` and `Installation source`, extracts a version by regex, reads `repos.listTags(...).data[0].name`, parses the first macOS item from `versions.xml`, and compares those values. Missing headings, an unparsable version, missing XML, or inconclusive comparisons are no-op paths. On the route-to-macOS branch it starts three provider writes: a comment, addition of `TG macOS Swift`, and `state: closed`.

The three issue-write calls are not awaited. Therefore successful JavaScript completion does not prove that comment, label and close all durably completed.

## Existing-owner-first disposition
Fabushi already has two exact owners that bound the applicable source-neutral responsibility:
- `AGENTS.md`: Routing never proves a defect invalid, a proposal rejected, a support outcome complete, or an applicable source responsibility unnecessary.
- `projects/fabushi-communication-platform/quality/defect-regression-policy.md` / FQT-DEF-001: reproduce->fix->targeted regression->affected regression->current-head evidence 才 close。

The provider route can help redirect an intake record, but it cannot determine defect validity, reproduce a bug, establish root cause/fix coverage, or create current-head evidence. No new issue closer service, second defect ledger, support product, product telemetry owner, release gate or acceptance authority is introduced.

## Fail-closed / security conclusions
- `action_issue_closer@master` is mutable; the resolved commit above is audit-time provenance only.
- The macOS XML endpoint is mutable and download failure is intentionally converted to a successful download step; later missing-file handling simply no-ops.
- Runtime npm dependencies are not version/digest pinned by this action execution path.
- Issue body parsing depends on literal headings and a permissive version regex.
- `repos.listTags().data[0]` is assumed to supply the comparison tag; the code does not establish semantic latest-version ordering.
- The workflow passes a repository token into a mutable third-party action that may comment, label and close issues.
- The three provider writes are not awaited, so partial side effects can occur without being represented by control-flow completion.
- A provider issue marked closed is never equivalent to FQT-DEF-001 closure, source disposition, or release ACCEPT.

## Exact traceability
- Requirement: `TDRP-R9-ISSUE-INTAKE-ROUTING-MAINTENANCE-001`
- Oracles: `ORA-TDRP-ISSUE-CLOSER-EXACT-SOURCE-001`, `ORA-TDRP-ISSUE-CLOSER-ROUTING-NOT-DEFECT-CLOSURE-001`, `ORA-TDRP-ISSUE-CLOSER-PROVIDER-WRITE-OBSERVABILITY-001`
- Invariants: `INV-TDRP-ISSUE-CLOSER-NO-SECOND-DEFECT-OWNER-001`, `INV-TDRP-ISSUE-CLOSER-NO-RELEASE-AUTHORITY-001`, `INV-TDRP-ISSUE-CLOSER-MISSING-SIGNAL-NO-PASS-001`
- Temporal/fault: `TEMP-TDRP-ISSUE-CLOSER-ACTION-MASTER-DRIFT-001`, `TEMP-TDRP-ISSUE-CLOSER-UNAWAITED-WRITES-001`, `FAULT-TDRP-ISSUE-CLOSER-XML-DOWNLOAD-001`, `FAULT-TDRP-ISSUE-CLOSER-NPM-DRIFT-001`, `FAULT-TDRP-ISSUE-CLOSER-PARTIAL-WRITE-001`
- Security: `SEC-TDRP-ISSUE-CLOSER-GITHUB-TOKEN-001`, `SEC-TDRP-ISSUE-CLOSER-MUTABLE-SUPPLY-CHAIN-001`

Closing strict index 85 changes counters to unknown=16,218, unread=16,193, omitted=0, responsibilities=103. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 86 `.github/workflows/linux.yml@dbedd23593c5ea66ff9b8b880bda6a8bc5a4cc55`.
