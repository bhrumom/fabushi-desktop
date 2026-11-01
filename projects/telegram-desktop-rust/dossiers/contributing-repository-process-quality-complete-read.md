# Fixed-U `.github/CONTRIBUTING.md` complete read

- Upstream repository: `telegramdesktop/tdesktop`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (recursive tree complete, 7,194 entries)
- Source path/blob: `.github/CONTRIBUTING.md@4265e974d0b87622af223d5b8096b67edcbd4aab`
- Recursive-tree index: 105 zero-based
- Mode: `100644`
- Source identity: 6,074 Git bytes / 6,074 ASCII characters / 118 LF newline characters / 0 CR / trailing newline / 119 split lines
- Target responsibility: `TDRP-R9-REPOSITORY-CONTRIBUTION-PROCESS-QUALITY-001`
- Stage: A; executable validation deferred

## Exact document structure

The source was read through EOF. Its behavioral headings are:

1. `What contributions are accepted`
2. `Build instructions`
3. `Pull upstream changes into your fork regularly`
4. `How to get your pull request accepted`
5. `Keep your pull requests limited to a single issue`
6. `Squash your commits to a single commit`
7. `Don't mix code changes with whitespace cleanup`
8. `Keep your code simple!`
9. `Test your changes!`
10. `Write a good commit message`

## Contribution acceptance contract

Telegram Desktop's upstream contribution channel explicitly welcomes bug fixes, source-code optimizations and documentation fixes through pull requests. Maintainers review contributions and may request changes. The source explicitly states that new feature implementations, translations to new languages and new UI elements are not merged through this upstream PR channel; translation work is routed to Telegram's translations platform.

That acceptance scope is repository-maintainer policy, not product behavior. It cannot be used to reduce Fabushi migration scope, skip a fixed-U responsibility, or create a Telegram-branded contribution-policy owner. The private-roadmap explanation remains provenance only.

## Build/test contract and direct dependency

The document delegates build details to its `build_instructions` link, whose source URL points at `telegramdesktop/tdesktop/blob/master/README.md#build-instructions`. Because that URL floats on `master`, this closure pins the direct fixed-U dependency `README.md@15a6194f73ae6084a2ab25d14005dc10262cdfb3` instead. At fixed U, README's build section dispatches to:

- Windows -> `docs/building-win.md`
- macOS -> `docs/building-mac.md`
- GNU/Linux using Docker -> `docs/building-linux.md`

The contribution guide separately requires contributors to test changes before submitting a pull request and to verify that Telegram Desktop still works without newly introduced issues or crashes. In Fabushi Stage A this becomes an authored Stage-B qualification requirement only; nothing is executed now.

## Source-lineage / fork synchronization contract

The exact source-side lineage recipe is:

- add `https://github.com/telegramdesktop/tdesktop.git` as remote `upstream`;
- `git fetch upstream master`;
- inspect with `git log upstream/master` before integrating;
- `git rebase upstream/master`;
- then `git push --force` the contributor's rewritten fork branch.

The stated intent is to prevent pull requests from diverging too far from fast-moving upstream. This is not Fabushi publication authority. Fabushi's migration checkpoint remains governed by existing intent-aware rebase/process rules and non-force `expected_sha` CAS. The upstream force-push recipe must not be copied onto protected Fabushi refs.

## Pull-request quality and reporting contract

The guide requires small/atomic pull requests limited to one issue. Documentation cleanup must not drag unrelated files into a change, and code changes must not be mixed with broad whitespace cleanup. Upstream contributors are asked to squash a PR to one commit, keep code straightforward and names descriptive, test before submission, and write a commit message explaining why the change exists.

When a commit fixes an issue, the guide directs contributors to supported issue-closing keywords plus the issue number (example given in source: `Fix #545`). Maintainers may reject broad or unreadable changes and ask that the specific issue be isolated.

## Link/reporting surface

The source declares links for the Telegram project, GitHub fork help, commit-message guidance, the upstream compare/PR page, build instructions, GitHub issue-closing keywords and the translations platform. It also declares a `help_change_commit_message` link that is not otherwise used by the document body. These external destinations are source-declared routing metadata; their live content is not treated as fixed-U source evidence.

## Direct consumers and non-consumers

GitHub's conventional `.github/CONTRIBUTING.md` repository surface is the direct host consumer. Prospective contributors consume it before proposing changes; maintainers/reviewers consume the same rules when reviewing or requesting corrections. `README.md` is an outbound build dependency, not the caller/owner of this file.

No Fabushi shipping runtime, product conversation/resource/settings owner, platform-core service, deployment service, CI runner or product UI consumes this guide at runtime. The guide's Git commands are repository-maintenance instructions, not executable product code.

## Existing-owner-first mapping

The portable responsibilities map to existing Fabushi owners:

- root repository development-process/source-provenance authority for contributor scope, PR atomicity, commit rationale and issue linkage;
- `TDRP-R9-REPOSITORY-INTENT-AWARE-REBASE-AUTHORITY-001` for lineage/conflict/rebase recovery rather than a second rebase subsystem;
- shared `TEST_STRATEGY` and evidence governance for build/test qualification;
- FMEP-001 for Stage-A zero-execution timing and later Stage-B proof.

Rejected owners: Telegram-specific contribution-policy runtime; Fabushi product Settings/Permissions/Conversation/Resource owners; Agent/Coordinator/Host/Runner product runtime; platform-core authorization/service; packaging/updater runtime; second CI/test system; second Git/rebase engine; second PR/issue tracker; and any force-publication authority derived from the source-side fork recipe.

## Failure / recovery contract

Non-passing conditions include a stale/diverged fork, conflicted or interrupted lineage rewrite, broad/mixed issue scope, functional changes mixed with whitespace cleanup, untested changes, missing commit rationale, incorrect issue linkage, treating a floating `master` URL as fixed-U identity, treating Telegram-specific rejection policy as Fabushi product scope, or applying upstream force-push instructions to Fabushi migration refs.

Recovery is explicit through existing owners: refresh/rebase or abort and retry lineage safely; split scope; isolate cleanup; run required Stage-B validation; repair commit/reporting metadata; and requalify the exact frozen candidate. No hidden retry, automatic acceptance or force override is introduced.

## Authored Stage-B qualification

Authored only, not run:
- `CONTRACT-TDRP-CONTRIBUTING-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-CONTRIBUTING-ACCEPTED-SCOPE-001`
- `CONTRACT-TDRP-CONTRIBUTING-BUILD-LINK-001`
- `CONTRACT-TDRP-CONTRIBUTING-UPSTREAM-LINEAGE-001`
- `CONTRACT-TDRP-CONTRIBUTING-ATOMIC-PR-001`
- `CONTRACT-TDRP-CONTRIBUTING-SQUASH-COMMIT-001`
- `CONTRACT-TDRP-CONTRIBUTING-WHITESPACE-ISOLATION-001`
- `CONTRACT-TDRP-CONTRIBUTING-CODE-CLARITY-001`
- `CONTRACT-TDRP-CONTRIBUTING-TEST-GATE-001`
- `CONTRACT-TDRP-CONTRIBUTING-COMMIT-MESSAGE-001`
- `INT-TDRP-CONTRIBUTING-PROCESS-COMPOSITION-001`
- `FAULT-TDRP-CONTRIBUTING-STALE-FORK-001`
- `FAULT-TDRP-CONTRIBUTING-MIXED-SCOPE-001`
- `FAULT-TDRP-CONTRIBUTING-UNTESTED-001`
- `SEC-TDRP-CONTRIBUTING-NO-FORCE-PUBLISH-001`

Stage A performs no remote/fork mutation, fetch/log/rebase/reset/squash/force-push, build/typecheck/lint/validator/test/E2E, CI, PR mutation, package/deploy/install/runtime-acceptance operation.
