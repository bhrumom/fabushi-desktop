# Fixed-U `.github/ISSUE_TEMPLATE/FEATURE_REQUEST.yml` complete read

- Upstream repository: `telegramdesktop/tdesktop`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (recursive tree complete, 7,194 entries)
- Source path/blob: `.github/ISSUE_TEMPLATE/FEATURE_REQUEST.yml@e2a8becdb9afcbd028b62378321458a2028fe348`
- Recursive-tree index: 108 zero-based
- Mode: `100644`
- Source identity: 925 Git bytes / 922 Unicode code points / 29 LF newline characters / 0 CR / trailing newline / 30 split lines
- Target responsibility: `TDRP-R9-REPOSITORY-FEATURE-REQUEST-ISSUE-FORM-001`
- Stage: A; executable validation deferred

## Exact schema and field order

The source was read through EOF. Top-level metadata is:
- name: `💡 Feature request`
- description: `Suggest an idea.`
- labels: `[enhancement]`

The body has exactly four textarea controls in order:
1. `Is your feature request related to a problem?` — required; asks for a clear and concise problem description.
2. `Describe the solution you'd like` — required; asks for the desired outcome.
3. `Describe alternatives you've considered` — required; asks for alternative solutions/features already considered.
4. `Additional context` — optional; placeholder invites other context or screenshots.

No title prefix, assignee, project, milestone, severity, component/product-area dropdown, checkbox, contact link, acknowledgement, security-report route or private-report control is declared.

## Input-validation boundary

The only host-enforced validation declared by this YAML is `validations.required: true` on the first three textareas. There is no regex, custom action/script, character/length bound, duplicate detector, feasibility validator, scope classifier, prioritization score, security scanner or screenshot/attachment validator.

Accordingly, nonempty but vague, duplicate or out-of-scope text can satisfy schema presence. Such cases are repository triage-quality/process issues, not machine rejection supplied by this source.

## Routing and scope contract

Top-level `labels: [enhancement]` expresses the source's sole automatic routing intent. The form separates:
- problem/motivation;
- desired solution/outcome;
- alternatives considered;
- optional supporting context/screenshots.

Submission does not itself mean accepted scope, priority, roadmap placement, implementation commitment, release target or SLA. Telegram-specific feature policy is provenance only; Fabushi's current specs/product/process owners remain authoritative.

## Privacy and security boundary

The optional Additional context placeholder expressly mentions screenshots, but the source does not tell reporters to redact secrets, account identifiers, private messages, tokens, customer data, filesystem paths or other sensitive content. It has no consent checkbox, private security route, secret detector or protected attachment path.

This omission cannot weaken Fabushi repository privacy/security policy. Sensitive supporting material must be redacted/omitted from public issues or routed through an already-approved protected process. This closure creates no screenshot collector, upload service, telemetry store, credential store, private-report service or platform-core authorization path.

## Direct consumers and non-consumers

Direct host consumer: GitHub Issue Forms repository template discovery/rendering/submission. Direct human consumers: people proposing features and maintainers/triagers/product planners evaluating requests.

The current Fabushi checkpoint has no `.github/ISSUE_TEMPLATE` directory. No Fabushi shipping runtime, Conversation/Resource/Settings owner, Agent/Coordinator/Host/Runner runtime, platform-core service, package/update runtime or CI runner consumes this YAML as product behavior.

## Existing-owner-first mapping

Portable responsibility maps to existing Fabushi owners:
- root repository issue/reporting/development-process authority for feature-request schema/routing/triage semantics;
- existing repository privacy/security governance for supporting-material disclosure boundaries;
- shared TEST_STRATEGY and evidence governance for qualification and non-passing cases;
- FMEP-001 for Stage-A authored-only timing and later Stage-B evidence.

Rejected owners: Telegram-branded support/backlog runtime; product Tasks/Automations; product Settings/Permissions; Agent/Coordinator/Host/Runner; platform-core authorization/service; second issue tracker/backlog; second quality/evidence truth.

Because the target repository currently lacks an issue-template tree, this Stage-A closure records exact behavior and owner contracts without inventing repository UX. Any later Fabushi feature-request form must be source-neutral, current-owner approved and independently qualified.

## Failure / recovery contract

Non-passing conditions include missing required fields; vague/duplicate/out-of-scope request text; incorrect/missing enhancement routing; sensitive screenshots/context in public text; treating form submission as approval/priority/roadmap authority; or copying Telegram-specific product policy as Fabushi scope.

Recovery is explicit through existing owners: complete/correct the three required dimensions, refine scope/rationale, correct routing metadata, redact/omit/protect sensitive context, and requalify the exact frozen candidate. No hidden retry, automatic attachment collection, backlog escalation or second planning system is introduced.

## Authored Stage-B qualification

Authored only, not run:
- `CONTRACT-TDRP-FEATURE-REQUEST-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-FEATURE-REQUEST-SCHEMA-001`
- `CONTRACT-TDRP-FEATURE-REQUEST-REQUIRED-INPUTS-001`
- `CONTRACT-TDRP-FEATURE-REQUEST-PROBLEM-SCOPE-001`
- `CONTRACT-TDRP-FEATURE-REQUEST-SOLUTION-001`
- `CONTRACT-TDRP-FEATURE-REQUEST-ALTERNATIVES-001`
- `CONTRACT-TDRP-FEATURE-REQUEST-ADDITIONAL-CONTEXT-001`
- `CONTRACT-TDRP-FEATURE-REQUEST-ENHANCEMENT-ROUTING-001`
- `INT-TDRP-FEATURE-REQUEST-PROCESS-COMPOSITION-001`
- `FAULT-TDRP-FEATURE-REQUEST-MISSING-REQUIRED-001`
- `FAULT-TDRP-FEATURE-REQUEST-LOW-QUALITY-SCOPE-001`
- `FAULT-TDRP-FEATURE-REQUEST-SENSITIVE-CONTEXT-001`
- `SEC-TDRP-FEATURE-REQUEST-PRIVACY-BOUNDARY-001`
- `SEC-TDRP-FEATURE-REQUEST-NO-ACCEPTANCE-ESCALATION-001`

Stage A performs no Issue Form submission, issue creation/label mutation, screenshot acquisition/upload, build/typecheck/lint/validator/test/E2E, CI, packaging/deployment/installation/runtime-acceptance operation.
