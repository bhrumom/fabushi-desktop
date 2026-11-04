# Complete read — fixed-U dependency-watch skill

## Authority snapshot

- Desktop checkpoint/live head before spec publication: `099d79fc5c50bf61ae08298150f9a5186e8ca23c`.
- Desktop main reread this round: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, reread `truncated=false`, 7,194 entries.
- Source entry: `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`, recursive tree index 56.
- Exact source was read completely.

## Responsibility classification

The source is repository-development dependency/supply-chain advisory governance. It audits freshly fetched source dependency inputs, release/security state and patched-fork/backport candidates, then persists advisory findings. It explicitly does not bump dependencies, build Telegram, create queue records, commit/push, open PRs or apply backports.

Existing-owner-first result:

- canonical owner: existing Fabushi repository quality/test-evidence governance;
- durable policy surface: root `AGENTS.md`;
- qualification: `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`;
- evidence: `projects/fabushi-communication-platform/quality/evidence-contract.md`;
- new ledger responsibility: `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001`;
- no new product/runtime owner and no platform-core production delta.

The shipping Electron updater (`source/electron-main/update/*`) is rejected: it owns signed product update/install/rollback lifecycle, not source dependency release/advisory analysis. Dependency acquisition inventory is evidence input, not the decision owner. Telegram-specific Python monitoring runtime is not copied.

## Snapshot and coverage contract

A current audit starts from one fresh `origin/dev` fetch, resolves the commit once, and reads blobs at that exact revision without switching the working tree. Fetch failure means the current audit failed; stale refs cannot masquerade as freshly checked. Offline/no-fetch state is explicitly unverified.

Coverage is the full union of dependency-bearing prepare, Docker, Snap, Qt-selection, workflow, package/lock and recursive gitlink/submodule inputs. Candidate extraction is only navigation. Newly discovered dependencies are added every run, with shipped/system/build-tool distinctions and platform/configuration conditions retained. Missing recursive or external evidence remains partial/unavailable rather than clean.

The helper stores runs under Git common-directory storage, preserving prior reports. User-prepared libraries and live submodules are not fetched/reset/checked out for research, and downloaded build scripts are not executed/imported merely to inspect them.

## Release, security, trust and forks

Each dependency records identity, upstream, consumed version/revision, manifest paths, platform/configuration, release scheme and maintained series. Official release/tag metadata and release notes are checked separately from security advisories; absence from one database is not safety evidence.

Ordinary stable version comparison is only for supported three-component stable schemes. Commit hashes, dates, Chromium milestones, OpenSSL letter series, four-component versions and prereleases require their native policy rather than forced SemVer.

Urgency and adoption readiness are distinct. Release assessment preserves publication date, immutable tag/peeled commit/package/artifact identity, source form, trust/regression evidence, unavailable checks and next review condition. Signatures/attestations/checksums are evidence only if actually verified; they do not prove benign code. Routine fresh releases can be held for observation, while confirmed security fixes can justify immediate update/backport review.

Patched/copied dependencies retain three identities: consumed fork pin, fork head and original upstream/import provenance. For `tg_owt` and `tg_angle`, the monitor examines import history, upstream ranges and fix applicability. Security candidates are classified as confirmed applicable-and-missing, already present/backported, not applicable, or needs investigation. Missing ancestry, matching subjects or patch applicability alone do not establish a missing fix.

## Persistent findings and report

Stable finding identity is dependency + consumed series/platform + advisory or target version. First/last seen, pending status, decisions/reasons, source links, fork evidence and unresolved checks survive later failures. A pin change does not alone prove a fix.

Run-local `report.md` and `findings.json` are written before atomic replacement of `latest.md`/`latest.json`; prior run reports are retained. Partial scans name gaps and never say all dependencies are current. The report always contains “Most pressing updates pending” and “Review someday”, then compact coverage.

## Direct callers/consumers and adjacent implementation evidence

- `.agents/skills/dependency-watch/agents/openai.yaml@a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d`: discovery/default-prompt adapter invoking the skill; not state authority.
- `.agents/skills/dependency-watch/scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38`: snapshot/compare helper. It fetches only `origin/dev`, resolves exact commit, stores common-dir run state, extracts navigation candidates, compares supported stable versions and marks no-fetch snapshots unverified.
- `.agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea`: regression consumer for stable ordering/prefixes, no-working-tree mutation, fetch failure, required-manifest failure, offline-unverified state and linked-worktree common storage.
- `.agents/skills/dependency-watch/references/release-trust.md@81a68395f9ad54ed651a84464a933ed7e57c9d32`: adoption/trust decision contract.
- `.agents/skills/dependency-watch/references/forks.md@29a7aa587ae26118a0611a2e4037a69141a3ffde`: patched/copy provenance and backport-analysis contract.

Repository search found the OpenAI adapter and these skill-local helper/reference consumers; no independent shipping runtime caller or second durable dependency-advisory owner was found. Adjacent blobs remain independent fixed-U inventory entries unless separately represented later.

## Authored verification contracts

Authored in Stage A and deliberately not run:

- `CONTRACT-TDRP-DEPENDENCY-SNAPSHOT-COVERAGE-001`
- `CONTRACT-TDRP-DEPENDENCY-RELEASE-TRUST-001`
- `CONTRACT-TDRP-DEPENDENCY-SECURITY-FORK-001`
- `CONTRACT-TDRP-DEPENDENCY-FINDING-LIFECYCLE-001`
- `FAULT-TDRP-DEPENDENCY-PARTIAL-RECOVERY-001`
- `INT-TDRP-DEPENDENCY-ADVISORY-OWNER-001`

Validation status remains `not-run`; acceptance remains false.

## Stage-A attestation

No build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was run. No GitHub Actions/check/status/workflow query or trigger was performed.
