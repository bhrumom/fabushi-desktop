# Fixed-U complete read — Claude release command adapter

## Authority snapshot

- Task round: 146.
- Source evidence round: 177.
- Date: 2026-11-01.
- Desktop checkpoint parent: `c14ca1ffff726a38d4138d261e08e329f0de5b4f`.
- Desktop main reread: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core checkpoint: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs.
- Source entry: `.claude/commands/release.md@baaa7c261db67dced2ce052c8cde016147031a7d`, recursive-tree index 96 zero-based.
- Exact source read completely: 5,840 Git bytes / 5,822 decoded characters / 119 newline characters / trailing newline / 120 split lines.
- Root changelog read: `changelog.txt@41b7466f8a11a1866a66ff583809dfe74ad9a3c3`; latest fixed-U entry is `7.3.3 (21.10.26)`.
- Version wrapper read: `Telegram/build/set_version.bat@7e7c3f69e8706905032e58cc05dcc25802dccebe`.
- Version implementation read completely: `Telegram/build/set_version.py@305a2a5f37aa13c8bef29d13ea37cab7a776cde6`, 5,203 decoded characters / 153 newline characters / trailing newline.
- Upstream host instruction pointer: `CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155` -> `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32`.
- Next fixed-U tree blob: `.claude/grab_clipboard.ps1@bece778c6038820ca4d16bbf4224548de1e385a8`.

## Complete source semantics

The source is a Claude Code slash-command wrapper whose front matter allows Read, Bash, Edit, Grep and AskUserQuestion. Its full flow is deliberately linear: require a clean Git worktree; read the first changelog version; parse optional explicit version and beta intent; derive a non-reused shared beta/stable version number; obtain explicit user confirmation; refresh tag visibility and locate the release baseline; read commits since that baseline; draft a short user-facing changelog; prepend the new block without modifying old history; stop for explicit human file review; run the canonical version helper only after approval; create a release commit; then read that commit back.

Version identity is shared across beta and stable tracks. An explicit major.minor is normalized to patch zero, an explicit major.minor.patch is preserved, and omitted input advances patch from the latest changelog version. A beta number is consumed and cannot later be reused as stable. The wrapper warns on an explicitly supplied already-used latest number rather than silently treating beta/stable as separate namespaces.

Changelog lineage is not raw commit dumping. The source reads the repository changelog and the release commit range, then selects only user-visible behavior. It excludes CI/build-only/submodule/style/refactor/intermediate WIP noise, collapses related commits, orders features before improvements before fixes, and aims for 4-12 imperative bullets with periods. The exact new block is prepended and prior entries remain immutable. Stable releases that span beta entries must describe noteworthy changes since the last stable release, not erase or rewrite beta history.

The source has two blocking human gates. The chosen version is confirmed before release mutation continues. After the changelog edit, the file itself is the review surface and the adapter must not run versioning until the user explicitly approves. These gates are authoritative; a generated draft is not a release decision.

The fixed-U Windows versioning chain is `set_version.bat` -> `set_version.py`. The Python helper validates a version argument, requires exactly one matching changelog entry, and fail-closes on missing replacement patterns. It updates canonical version inputs in `Telegram/build/version`, `Telegram/SourceFiles/core/version.h`, `Telegram/Resources/winrc/Telegram.rc`, `Telegram/Resources/winrc/Updater.rc`, and `Telegram/Resources/uwp/AppX/AppxManifest.xml`. A failure is a stop boundary, not permission to commit partial version state.

Commit formatting is part of the contract: stable uses `Version ...`, beta uses `Beta version ...`, then one blank line and the approved changelog bullets, wrapping only lines over 72 characters with two-space continuation indentation. The final `git log -1` is local commit readback.

## Consumers and publication boundary

Direct consumer:
- Claude Code command discovery/user `/release`.

Repository host composition:
- `CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155` points Claude Code at canonical upstream `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32`.

The command performs repository-local release preparation and a release commit only. It does not create a tag, push a ref, force-push, publish a GitHub release, sign/package/deploy artifacts, or decide final release acceptance. The upstream `git fetch origin --tags` is source-lineage acquisition for release-note baselining, not publication. Under FMEP-001 Stage A none of these executable Git or versioning steps is run for migration evidence.

## Existing-owner-first mapping

New source responsibility: `TDRP-R9-RELEASE-CLAUDE-COMMAND-ADAPTER-001`.

Existing owners:
- root `AGENTS.md` repository development/release process governance;
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`;
- `projects/fabushi-communication-platform/quality/evidence-contract.md`;
- `projects/fabushi-communication-platform/quality/release-entry-exit-criteria.md` (`FQT-REL-001`) for final candidate release eligibility.

Rejected owners:
- shipping updater or packaging runtime;
- Product Tasks / Automations;
- Agent / Coordinator / Host / Runner product runtime;
- platform-core runtime/service;
- Claude AI task/phase publication pipeline;
- Telegram-specific release queue/store;
- autonomous tag/push/GitHub-release publisher;
- a second release-acceptance truth.

No ADR, new shipping owner, platform-core production delta or new publication service is required.

## Authored contracts — not run

- `CONTRACT-TDRP-RELEASE-CLAUDE-HOST-001`
- `CONTRACT-TDRP-RELEASE-INVOCATION-VERSION-001`
- `CONTRACT-TDRP-RELEASE-CLEAN-TREE-001`
- `CONTRACT-TDRP-RELEASE-CHANGELOG-LINEAGE-001`
- `CONTRACT-TDRP-RELEASE-VERSION-SEQUENCE-001`
- `CONTRACT-TDRP-RELEASE-CHANGELOG-CONTENT-001`
- `CONTRACT-TDRP-RELEASE-APPROVAL-GATE-001`
- `CONTRACT-TDRP-RELEASE-SET-VERSION-001`
- `CONTRACT-TDRP-RELEASE-COMMIT-001`
- `CONTRACT-TDRP-RELEASE-PUBLICATION-BOUNDARY-001`
- `INT-TDRP-RELEASE-CLAUDE-COMPOSITION-001`
- `FAULT-TDRP-RELEASE-DIRTY-TREE-001`
- `FAULT-TDRP-RELEASE-SET-VERSION-001`
- `SEC-TDRP-RELEASE-NO-PUBLISH-001`

All remain `validation_status=not-run`, `accepted=false`. No git status/fetch/tag/log/set_version/commit command, build, typecheck, lint, validator, reviewer, test/evidence campaign, CI, packaging, signing, deployment, publication simulation or acceptance was executed.

## Source-closure disposition

Exactly one independent fixed-U source entry is represented by this dossier. The Claude release wrapper maps onto existing repository release/process/quality governance and final release criteria without creating a second runtime or publication owner. The next independent fixed-U blob in tree order is `.claude/grab_clipboard.ps1@bece778c6038820ca4d16bbf4224548de1e385a8`.
