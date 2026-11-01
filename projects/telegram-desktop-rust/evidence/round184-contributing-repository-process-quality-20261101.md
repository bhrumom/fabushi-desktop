# Round 184 — fixed-U repository contribution / PR process and quality policy

- Task round: 153
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `30e82da2015644d4f12312a238a0dfce9f0b49bc`
- Spec-first desktop commit: `e0d78aef130704c9ba1da2b9c576a57bc6ae9ffe`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, complete, 7,194 entries
- Closed source: `.github/CONTRIBUTING.md@4265e974d0b87622af223d5b8096b67edcbd4aab`
- Tree index: 105 zero-based; mode 100644
- Source identity: 6,074 Git bytes / 6,074 ASCII characters / 118 LF newlines / 0 CR / trailing newline / 119 split lines
- Responsibility: `TDRP-R9-REPOSITORY-CONTRIBUTION-PROCESS-QUALITY-001`

## Verified source semantics

The exact source was read through EOF. It defines Telegram Desktop's contribution acceptance/routing, delegates build details to README, documents fork refresh through upstream/master fetch/log/rebase plus contributor-fork force-push, requires focused one-issue PRs and one-commit history, separates whitespace cleanup, asks for clear code/naming, requires testing before submission, and requires commit rationale with issue-closing linkage when applicable.

Telegram-specific rejection of new features, new-language translations and new UI elements is an upstream maintainer-process rule, not authorization to omit fixed-U responsibilities from Fabushi. Translation routing and other external links are recorded as source-declared destinations only.

## Direct consumers and fixed-U dependency

GitHub's conventional `.github/CONTRIBUTING.md` repository surface is the direct host consumer; prospective contributors and maintainers/reviewers are the direct human consumers. The source's build hyperlink floats through `master`, so fixed-U `README.md@15a6194f73ae6084a2ab25d14005dc10262cdfb3` was read instead and pins the build section to Windows, macOS and GNU/Linux build documents.

## Existing-owner-first disposition

Target ownership remains root repository development-process/source-provenance + `TDRP-R9-REPOSITORY-INTENT-AWARE-REBASE-AUTHORITY-001` + shared quality/test-evidence governance. No Telegram contribution subsystem, second PR/issue tracker, second rebase owner, second CI/test truth, product runtime or platform-core service is introduced. The source-side force-push recipe is fenced to the contributor fork; Fabushi target publication remains non-force expected-SHA CAS.

## Failure/recovery and deferred qualification

Stale/diverged lineage, conflicted/interrupted rebase, mixed issue scope, unrelated whitespace cleanup, untested changes, unclear commit rationale, incorrect issue linkage, floating-link identity drift or force-publish misuse remain non-passing until corrected through existing owners and freshly qualified on the exact frozen candidate.

Authored only, not run: ten contract cases, one integration case, three fault-recovery cases and one security case. Ledger state is `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`. Coverage moves exactly one independent source entry: unknown `16125 -> 16124`, unread `16098 -> 16097`, omitted stays 0, responsibilities `153 -> 154`, unmapped remains 0.

No Actions/check/status/workflow query or trigger; no fork mutation, fetch/log/rebase/reset/squash/force-push, build/compile/typecheck/lint/validator/test/E2E, PR mutation, packaging/deployment/installation/runtime evidence or acceptance was performed. The next fixed-U independent blob is `.github/ISSUE_TEMPLATE/BUG_REPORT.yml@847b56e543fc28586ad281640218dfe8954d8bee`.
