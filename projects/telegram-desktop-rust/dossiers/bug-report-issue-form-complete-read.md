# Fixed-U `.github/ISSUE_TEMPLATE/BUG_REPORT.yml` complete read

- Upstream repository: `telegramdesktop/tdesktop`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (recursive tree complete, 7,194 entries)
- Source path/blob: `.github/ISSUE_TEMPLATE/BUG_REPORT.yml@847b56e543fc28586ad281640218dfe8954d8bee`
- Recursive-tree index: 107 zero-based
- Mode: `100644`
- Source identity: 2,112 Git bytes / 2,100 Unicode code points / 74 LF newline characters / 0 CR / trailing newline / 75 split lines
- Target responsibility: `TDRP-R9-REPOSITORY-BUG-REPORT-ISSUE-FORM-001`
- Stage: A; executable validation deferred

## Exact schema and field order

The source was read through EOF. Top-level metadata is:
- name: `🐞 Bug report`
- description: `Report errors or unexpected behavior.`
- labels: `[bug]`

The body has exactly eight controls in order:
1. textarea — `Steps to reproduce`; required; numbered 1/2/3 placeholder
2. textarea — `Expected behaviour`; required
3. textarea — `Actual behaviour`; required
4. input — `Operating system`; required
5. input — `Version of Telegram Desktop`; required
6. dropdown — `Installation source`; required; `multiple: false`
7. input — `Crash ID`; optional
8. textarea — `Logs`; optional; `render: shell`

No title prefix, assignee, project, contact link, acknowledgement checkbox, private-report control or security-report route is declared.

## Schema validation versus prose validation

The YAML's actual host-enforced validation is narrow. Six controls carry `validations.required: true`. The installation source is constrained to the declared dropdown options. No regex, custom action, script, JavaScript validator or hidden rule exists in this source.

Several stronger statements are only field-description guidance:
- Operating system asks for an exact OS and desktop environment example and says `NO KERNEL VERSIONS (uname)`.
- Version asks for an exact version, says not to write `LATEST`, and states such issues will be closed.
- Linux repository/package-manager versions are declared unsupported by upstream and routed to the distro tracker unless the reporter is the distro maintainer.

These are maintainer/triage rules, not machine validation. A nonempty but poor string can satisfy `required: true`, so later qualification must not claim a schema rejection that the source does not implement.

## Reproduction and environment contract

The source requires a three-part behavioral report: reproduction steps, expected behavior, and actual behavior. Environment identity is split across exact operating system, exact application version, and installation source.

The seven installation-source options are:
- Downloaded from desktop.telegram.org or GitHub
- Microsoft Store
- Mac App Store
- Flatpak
- Snap
- Linux package manager (apt, dnf, pacman, etc)
- Other source

The upstream distro-package policy is support routing, not product behavior and not a Fabushi support policy by inheritance.

## Crash ID contract

Crash ID is optional. The description instructs a Telegram Desktop reporter to enable beta versions in Settings -> Advanced, reproduce the crash, and copy the Crash ID on the next launch. That sequence is upstream diagnostic workflow provenance. It does not authorize Fabushi beta toggles, telemetry collection, crash-store persistence, account changes or a new crash service.

## Logs contract

Logs are optional. The source instructs the reporter to type `viewlogs` directly into Telegram settings to open the logs folder, then paste `log.txt`. The textarea uses `render: shell`.

`render: shell` is GitHub Issue Forms presentation metadata; it renders submitted text in a shell-flavored code block. It does not execute the text and cannot be interpreted as authority to run commands supplied by a reporter. The source has no automated log collector, upload client, parser, retention policy or network endpoint.

## Privacy and security boundary

The source contains no explicit instruction to redact secrets, tokens, account identifiers, local filesystem paths, network addresses, private message content or other sensitive diagnostics. It has no consent checkbox, secret scanner, protected attachment path or private security-vulnerability route.

That omission is a source fact, not a target policy. Existing Fabushi repository privacy/security governance remains authoritative. Sensitive diagnostics must be redacted or omitted from public issues, or routed through an already-approved protected reporting path if one exists. This closure creates no new private-report service and does not weaken security policy.

## Routing and quality contract

Top-level `labels: [bug]` expresses source routing intent. The issue form otherwise declares no assignee/project/title routing. Quality comes from required reproduction/environment fields plus maintainer prose around exact versions and supportable installation sources.

Later Stage-B evidence must prove current target host/routing behavior if a source-neutral form is adopted; it cannot assume that a named label or route exists merely because the upstream YAML declares it.

## Direct consumers and non-consumers

Direct host consumer: GitHub Issue Forms repository template discovery/rendering/submission. Direct human consumers: people filing bugs and maintainers/triagers reading/routing those reports.

The current Fabushi checkpoint has no `.github/ISSUE_TEMPLATE` directory. No Fabushi shipping runtime, Conversation/Resource/Settings owner, crash telemetry service, platform-core service, Agent/Coordinator/Host/Runner runtime, package/update runtime or CI runner consumes this YAML as product behavior.

## Existing-owner-first mapping

Portable responsibility maps to existing Fabushi owners:
- root repository issue/reporting/development-process authority for report schema/routing semantics;
- existing repository privacy/security governance for diagnostic disclosure boundaries;
- shared TEST_STRATEGY and evidence governance for qualification and non-passing cases;
- FMEP-001 for Stage-A authored-only timing and later Stage-B evidence.

Rejected owners: Telegram-branded support/runtime; product crash uploader or telemetry store; product Settings/Permissions; Agent/Coordinator/Host/Runner runtime; platform-core authorization/service; second issue tracker; second quality/evidence truth; copied Telegram beta/cheat-code/distro-support policy.

Because the target repository currently lacks an issue-template tree, this Stage-A closure records the exact behavior and target-owner contract without introducing Telegram-specific repository UX. Any later Fabushi issue form must be source-neutral, current-owner approved and independently qualified.

## Failure / recovery contract

Non-passing conditions include missing required fields; ambiguous `LATEST` or kernel-only environment content; unsupported/misreported installation source under the source policy; routing/label drift; sensitive diagnostics pasted into public text; executing or treating `render: shell` text as executable; or copying Telegram-specific beta/cheat-code instructions into Fabushi as if they were product requirements.

Recovery is explicit through existing owners: complete/correct required report dimensions; provide exact environment data; correct route/label metadata; redact/omit/protect sensitive diagnostics; keep reporter text inert; and requalify the exact frozen candidate. No hidden retry, automatic log collection or second support system is introduced.

## Authored Stage-B qualification

Authored only, not run:
- `CONTRACT-TDRP-BUG-REPORT-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-BUG-REPORT-SCHEMA-001`
- `CONTRACT-TDRP-BUG-REPORT-REQUIRED-INPUTS-001`
- `CONTRACT-TDRP-BUG-REPORT-REPRODUCTION-001`
- `CONTRACT-TDRP-BUG-REPORT-ENVIRONMENT-001`
- `CONTRACT-TDRP-BUG-REPORT-VERSION-001`
- `CONTRACT-TDRP-BUG-REPORT-INSTALL-SOURCE-001`
- `CONTRACT-TDRP-BUG-REPORT-CRASH-ID-001`
- `CONTRACT-TDRP-BUG-REPORT-LOG-RENDERING-001`
- `CONTRACT-TDRP-BUG-REPORT-BUG-LABEL-ROUTING-001`
- `INT-TDRP-BUG-REPORT-PROCESS-COMPOSITION-001`
- `FAULT-TDRP-BUG-REPORT-MISSING-REQUIRED-001`
- `FAULT-TDRP-BUG-REPORT-AMBIGUOUS-ENVIRONMENT-001`
- `FAULT-TDRP-BUG-REPORT-SENSITIVE-LOGS-001`
- `SEC-TDRP-BUG-REPORT-NO-LOG-EXECUTION-001`
- `SEC-TDRP-BUG-REPORT-PRIVACY-BOUNDARY-001`

Stage A performs no Issue Form submission, issue creation/label mutation, crash/log acquisition, beta toggle, cheat-code invocation, log execution, build/typecheck/lint/validator/test/E2E, CI, packaging/deployment/installation/runtime-acceptance operation.
