# Fixed-U `.github/ISSUE_TEMPLATE/config.yml` complete read

- Upstream repository: `telegramdesktop/tdesktop`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (recursive tree complete, 7,194 entries)
- Source path/blob: `.github/ISSUE_TEMPLATE/config.yml@1b864704be6a3e3aafdb1e02519bec84886942ce`
- Recursive-tree index: 109 zero-based
- Mode: `100644`
- Source identity: 753 Git bytes / 727 Unicode code points / 14 LF newline characters / 0 CR / trailing newline / 15 split lines
- Target responsibility: `TDRP-R9-REPOSITORY-ISSUE-TEMPLATE-CONFIG-001`
- Stage: A; executable validation deferred

## Exact configuration

The source was read through EOF. It sets:

- `blank_issues_enabled: false`
- exactly four ordered `contact_links`

The links are:
1. `📱🎨🌍 Mobile apps, UI/UX Design, or Missing Features` -> `https://bugs.telegram.org`; about text says matters not exclusively a Telegram Desktop app bug / UI complaints should use that route.
2. `📝 I found a typo or translation error` -> `https://translations.telegram.org`; about text says not to open an issue and to suggest fixes in the crowdsourced translations service.
3. `🚫 My account is banned / I can't log in` -> `https://telegram.org/support`; about text says repository maintainers cannot unban the reporter and points to web support.
4. `❓ I just want to ask a question` -> `https://t.me/TelegramDesktopTalk`; about text says not to open an issue and instead use the group chat.

No title prefix, labels, assignees, project, milestone, body schema, form input, acknowledgement, private security-report route, redaction warning, attachment policy, retry rule or custom validator is declared in this file.

## Blank-issue and routing contract

`blank_issues_enabled: false` is a GitHub repository issue-template chooser policy: the upstream chooser does not offer an unrestricted blank issue path. It is not a product runtime feature, Fabushi support policy, or proof that target repository settings currently match.

The four contact links are source routing metadata. They partition upstream non-Desktop-only/mobile/UI-feature matters, translations, account/login support and general questions into Telegram-owned external destinations. The source does not say those destinations are appropriate for Fabushi and does not give them authority over Fabushi product scope, acceptance, priority, roadmap, SLA, account/session state, localization state or security policy.

## Security and privacy boundary

The source declares no security-vulnerability contact link and no private-report path. It also provides no warning to redact credentials, tokens, account identifiers, private messages, customer data, local paths or other sensitive information.

The account/login and group-chat routes therefore cannot be generalized into approved Fabushi destinations for sensitive material. The static URLs carry no source-declared reporter form payload or issue body, but following a link leaves GitHub and crosses into an external trust domain. Any future Fabushi contact-link configuration must use current source-neutral repository/privacy/security authority, not Telegram provenance.

## Direct consumers and non-consumers

Direct host consumer: GitHub Issues issue-template chooser/configuration metadata. Direct human consumers: would-be issue reporters and repository maintainers/triagers. The external Telegram services are navigation destinations named by the source, not Fabushi owners.

The current Fabushi checkpoint has no `.github/ISSUE_TEMPLATE` tree. No Fabushi shipping ProductShell, Conversation, Resource, Identity/account, localization, Agent/Coordinator/Host/Runner, platform-core service, package/update runtime or CI runner consumes this file as product behavior.

## Existing-owner-first mapping

Portable responsibility maps to:
- root repository issue/reporting/development-process authority for chooser/routing semantics;
- existing repository privacy/security governance for external-link and sensitive-report boundaries;
- shared TEST_STRATEGY and evidence governance for qualification and non-passing cases;
- FMEP-001 for Stage-A authored-only timing and later Stage-B evidence.

Rejected owners: Telegram-branded support/help/translation/community runtime; Fabushi product account/session owner; product localization owner; Product Tasks/Automations/Settings/Permissions; Agent/Coordinator/Host/Runner; platform-core authorization/service; second issue tracker/support desk; second privacy/security or quality truth.

## Authored Stage-A contracts

Authored only, not executed:
- `CONTRACT-TDRP-ISSUE-CONFIG-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-ISSUE-CONFIG-BLANK-ISSUES-DISABLED-001`
- `CONTRACT-TDRP-ISSUE-CONFIG-CONTACT-LINKS-ORDER-001`
- `CONTRACT-TDRP-ISSUE-CONFIG-DESKTOP-SCOPE-ROUTING-001`
- `CONTRACT-TDRP-ISSUE-CONFIG-TRANSLATION-ROUTING-001`
- `CONTRACT-TDRP-ISSUE-CONFIG-ACCOUNT-SUPPORT-ROUTING-001`
- `CONTRACT-TDRP-ISSUE-CONFIG-QUESTION-ROUTING-001`
- `CONTRACT-TDRP-ISSUE-CONFIG-EXTERNAL-ROUTE-PROVENANCE-001`
- `INT-TDRP-ISSUE-CONFIG-PROCESS-COMPOSITION-001`
- `FAULT-TDRP-ISSUE-CONFIG-BLANK-ISSUE-BYPASS-001`
- `FAULT-TDRP-ISSUE-CONFIG-WRONG-ROUTE-001`
- `FAULT-TDRP-ISSUE-CONFIG-SENSITIVE-EXTERNAL-DISCLOSURE-001`
- `SEC-TDRP-ISSUE-CONFIG-EXTERNAL-LINK-TRUST-BOUNDARY-001`
- `SEC-TDRP-ISSUE-CONFIG-NO-TELEGRAM-AUTHORITY-ESCALATION-001`

Oracles:
- `ORA-TDRP-ISSUE-CONFIG-EXACT-CONFIG-001`
- `ORA-TDRP-ISSUE-CONFIG-BLANK-ISSUE-POLICY-001`
- `ORA-TDRP-ISSUE-CONFIG-ROUTING-SCOPE-001`
- `ORA-TDRP-ISSUE-CONFIG-PRIVACY-BOUNDARY-001`

Invariants:
- `INV-TDRP-ISSUE-CONFIG-ONE-ISSUE-PROCESS-OWNER-001`
- `INV-TDRP-ISSUE-CONFIG-NO-TELEGRAM-AUTHORITY-ESCALATION-001`
- `INV-TDRP-ISSUE-CONFIG-NO-SENSITIVE-EXTERNAL-DISCLOSURE-001`
- `INV-TDRP-ISSUE-CONFIG-NO-STAGE-A-EXECUTION-001`

## Stage-A execution boundary

No issue chooser interaction, blank issue creation, contact-link navigation, issue/support mutation, external account/support action, validator, build, compile, typecheck, lint, test, CI, packaging, deployment, installation, runtime evidence or acceptance was executed. The responsibility remains `validation_status=not-run`, `accepted=false` until the frozen candidate enters Stage B.
