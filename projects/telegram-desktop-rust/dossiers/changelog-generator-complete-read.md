# Fixed-U `.github/scripts/generate_changelog.py` complete read

- Upstream repository: `telegramdesktop/tdesktop`
- Fixed upstream commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source path/blob: `.github/scripts/generate_changelog.py@6cdd12652d31b2a47ed9a8ef906308af59cca3d0`
- Tree index: 112 zero-based; mode `100644`; Git size 9,036 bytes
- Decoded content: 9,036 code points, 328 LF, 0 CR, trailing newline, 329 split lines
- Triage: `full-read`
- Target responsibility: `TDRP-R9-REPOSITORY-CHANGELOG-PAGES-GENERATOR-001`
- Task round: 158
- Source evidence round: 189
- Desktop checkpoint parent: `7c63ae7c8bb75007cac18aaf41ec2f0c5382d36f`
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`

## Complete acquisition and I/O semantics

The helper is a Python-stdlib repository utility. It imports `re`, `shutil`, `sys`, `html` and `pathlib.Path`; it performs no network call, subprocess execution, package install, secret lookup or external service request.

With no CLI arguments, repository root is three parents above the script location, input is root `changelog.txt`, and output is `docs/changelog/index.html`. argv[1] replaces the input path and argv[2] replaces the output path. Input is read as UTF-8; output parent directories are created; HTML is written as UTF-8. After writing, the helper looks for `Telegram/Resources/art/icon16.png` and `icon32.png`; each existing icon is copied with metadata to the output directory. Missing icons are silently tolerated. Read/decode/write/stat/copy errors are not caught, so the process fails instead of reporting successful generation.

Exact fixed-U inputs are `changelog.txt@41b7466f8a11a1866a66ff583809dfe74ad9a3c3`, `icon16.png@561369d2b2688cc3799fc2b7a7a2f918d99cc155` and `icon32.png@cff636894428e42c4fa5a063e3aa472c4bded8fc`. No `docs/changelog` output is tracked in the fixed-U tree before generation.

## Parsing, filtering and version semantics

`VERSION_RE` is anchored and accepts:
- `major.minor` or `major.minor.patch`;
- optional `alpha`, `beta`, `dev` or `stable`;
- date `DD.MM.YY` or `DD.MM.YYYY`.

Two-digit years are normalized by adding 2000. Month is used as an index into the month-name table and day/month ranges are otherwise not validated; an out-of-range month can fail generation. A `sort_key` string is computed for each recognized version but never used anywhere else.

`parse_changelog` scans linearly and preserves input order. Text before the first recognized version header is ignored. A new header closes the prior entry. Before the first body line, blanks are skipped. A body line whose stripped value is exactly `),` is skipped as a stray artifact. Other lines are retained, and trailing blank lines are removed when parsing finishes.

The generator does not sort, allocate versions, prove monotonicity or decide release eligibility. `build_html` treats the first parsed entry as `latest_version` and the last parsed entry as the beginning date; therefore newest-first changelog order is a required input invariant. Fixed-U begins with `7.3.3 (21.10.26)`.

## Formatting and HTML behavior

Each entry anchor is `v<version>`. `stable` is omitted from the visible heading, while alpha/beta/dev labels are rendered. Lines beginning with ASCII `- ` or the escaped em-dash marker `\u2014 ` become list items. Any other non-empty body line becomes a paragraph; a blank line closes a currently open list.

Version, tag, date, list text and paragraph text pass through `html.escape`. The static page includes fixed CSS, a local search box that toggles entries by `textContent`, and anchor-click code that replaces the fragment and copies the current URL when the Clipboard API exists. The footer links to Telegram's live `dev/changelog.txt` and repository GPL page; those links are presentation/navigation, not immutable source evidence.

## Direct caller and consumers

Exact direct caller is fixed-U `.github/workflows/changelog.yml@445277fe4b44dc9ccef17b7b602016572c08fe7c`.

The workflow:
- triggers on `dev` pushes limited to `changelog.txt` and on manual dispatch;
- grants `contents: read`, `pages: write`, `id-token: write`;
- uses one `pages` concurrency group with cancel-in-progress;
- checks out source, configures Python 3.12, and invokes `python .github/scripts/generate_changelog.py` with no path overrides;
- uploads the entire `docs` directory as the Pages artifact;
- deploys that artifact to the `github-pages` environment.

Action references are version tags: `actions/checkout@v7`, `actions/setup-python@v7`, `actions/upload-pages-artifact@v5`, `actions/deploy-pages@v5`. They are dependency identities as written in the exact workflow blob, not immutable commit provenance.

Direct machine consumers are the GitHub Actions Python step, Pages artifact upload and deploy chain. Human consumers are changelog maintainers and Pages readers. The script is not consumed by Telegram client runtime.

## Failure, recovery, security and quality

Failure cases include malformed/out-of-range date components, unreadable or non-UTF-8 input, no recognized entries producing an empty-summary page, unexpected source ordering producing misleading latest/since summary, unwritable output, output stat failure, icon-copy failure, stale/mismatched fixed-U identities, missing caller lineage and workflow/action drift.

Recovery is preservation-first: correct the exact repository input/invocation, reread source/input/caller identities, and regenerate only in an authorized later validation/publication context. Do not silently sort history, drop malformed entries, rewrite existing changelog history, broaden argv path authority, or treat partial output as success.

Security positives are HTML escaping and fixed no-argument workflow paths. Security negatives are exposing CLI path overrides as remote product input, treating a generated page as trusted executable content, treating mutable action version tags as immutable provenance, or allowing source Pages publication to imply Fabushi release acceptance. The helper reads no credentials and performs no network access itself.

## Existing-owner-first disposition

Use:
- root `AGENTS.md` repository release/process governance;
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`;
- `projects/fabushi-communication-platform/quality/evidence-contract.md`;
- `projects/fabushi-communication-platform/quality/release-entry-exit-criteria.md` (`FQT-REL-001`).

Reject:
- shipping updater/update-feed runtime;
- `source/electron-main/update/release-metadata.ts` as changelog publication owner;
- Product Tasks/Automations;
- Agent/Coordinator/Host/Runner product runtime;
- platform-core service;
- Telegram-specific changelog service/store;
- second release/version/acceptance truth.

No ADR, shipping production delta, target Pages workflow, or platform-core delta is required for this pure repository/process provenance responsibility.

## Authored validation — not run

- `CONTRACT-TDRP-CHANGELOG-GENERATOR-ACQUISITION-001`
- `CONTRACT-TDRP-CHANGELOG-GENERATOR-VERSION-PARSE-001`
- `CONTRACT-TDRP-CHANGELOG-GENERATOR-FILTERING-001`
- `CONTRACT-TDRP-CHANGELOG-GENERATOR-ORDERING-001`
- `CONTRACT-TDRP-CHANGELOG-GENERATOR-HTML-FORMAT-001`
- `CONTRACT-TDRP-CHANGELOG-GENERATOR-INPUT-OUTPUT-001`
- `CONTRACT-TDRP-CHANGELOG-GENERATOR-SOURCE-LINEAGE-001`
- `CONTRACT-TDRP-CHANGELOG-GENERATOR-PAGES-CALLER-001`
- `INT-TDRP-CHANGELOG-GENERATOR-RELEASE-PROCESS-001`
- `FAULT-TDRP-CHANGELOG-GENERATOR-MALFORMED-DATE-001`
- `FAULT-TDRP-CHANGELOG-GENERATOR-IO-RECOVERY-001`
- `SEC-TDRP-CHANGELOG-GENERATOR-HTML-ESCAPE-001`
- `SEC-TDRP-CHANGELOG-GENERATOR-PATH-AUTHORITY-001`

All remain Stage-A authored-only: no Python execution, workflow run, Pages upload/deploy, validator, build, lint, test/E2E, packaging, signing, release, deployment, installation or acceptance was executed.

## Queue continuity

Exactly one independent fixed-U source entry is represented by this dossier. The next independent blob after tree node index 113 is `.github/telegram-bot-api/Dockerfile@a361602441040028a3332c3732c4902d9c880122` at recursive-tree index 114 zero-based.
