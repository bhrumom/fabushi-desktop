# Dependency Watch snapshot/compare helper — complete fixed-U read

Task round: 122  
Source evidence round: 153  
Stage: A / zero executable validation  
Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/dependency-watch/scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38`  
Recursive-tree index: 63  
Blob size: 6,655 bytes / 191 logical lines

## Direct callers and consumers

The direct workflow caller is `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`. It invokes `python3 .../watch.py snapshot` for the fresh source snapshot and `watch.py compare CURRENT CANDIDATE` for ordinary three-component stable versions. The direct code consumer is `.agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea`, which imports `watch` and calls `git`, `snapshot`, and `compare`. Current repository code search finds the helper CLI path in the skill and the direct `watch.snapshot`/`watch.compare` calls in that test; no second dependency-advisory state owner is introduced. The OpenAI adapter invokes the skill rather than owning helper snapshot state.

## Snapshot and fetch contract

`git(repo, *args)` runs `git -C <repo>` with captured output, nonzero-exit enforcement and a 180-second timeout. `snapshot(repo, fetch=True)` first canonicalizes the supplied path through `rev-parse --show-toplevel`. Online mode fetches only `origin` branch `dev` into `refs/remotes/origin/dev` using `--no-tags`, `--no-recurse-submodules` and a forced branch refspec; it does not switch HEAD or recurse through live submodules. The UTC check time is taken after the successful fetch. The helper then resolves `refs/remotes/origin/dev^{commit}` once and binds the remainder of the snapshot to that exact commit.

`git ls-tree -rz <commit>` recursively enumerates the exact tree. Every gitlink is retained as path+object for later recursive review. Blob selection is the union of the five required manifests, every path under `Telegram/build/`, `snap/`, or `.github/`, and every file whose basename is one of the configured manifest/lock names. Required manifests are:

- `.gitmodules`
- `Telegram/build/prepare/prepare.py`
- `Telegram/build/qt_version.py`
- `Telegram/build/docker/centos_env/Dockerfile`
- `snap/snapcraft.yaml`

All required paths must be blob entries with mode `100644` or `100755`. Exact bytes are read with `git cat-file blob <oid>`, so local working-tree edits cannot alter the snapshot.

## Common Git directory and storage contract

The helper resolves `rev-parse --git-common-dir`; relative output is made relative to the canonical repository top level. Its storage root is exactly `<resolved common git dir>/dependency-watch`. It creates `runs/` and a timestamp-prefixed temporary run directory, then stores path-preserving exact source copies beneath `run/sources/`. Linked worktrees therefore share the primary repository's common dependency-watch storage. The helper does not create a second per-worktree findings owner.

Only regular `100644`/`100755` blobs become source copies. Non-regular blobs, non-UTF-8 blobs and NUL-bearing content are recorded in `skipped`. After extraction, every required manifest must still be in the readable file set; otherwise the helper fails. Successful `snapshot.json` records schema version 1, canonical repository path, `origin/dev`, exact commit, UTC check time, `fresh_fetch`, `audit_status=snapshot-only`, storage/run identities, files, gitlinks, skipped inputs and the explicit note that dependency/security review is still pending and gitlinks require recursive inspection.

## Candidate navigation contract

For each readable selected source line, the helper emits `path`, one-based `line` and stripped `text` when the line matches one of its navigation heuristics: URL; git clone/checkout/fetch/submodule; source tag/branch/commit/checksum key; Snap package/snap key; `FROM`, `QT`, or `rustToolchain`; pip install; dnf/apt/apt-get/pacman; or workflow `uses:`. The resulting `candidates.json` is not a parsed or complete dependency inventory and cannot establish clean coverage.

## Version-compare contract

`compare(current, candidate)` accepts only full-string stable versions with optional `v`, `n`, or `openssl-` prefix followed by exactly three numeric components. Numeric tuples classify equal as `same`, lower candidate as `older`, then higher first differing component as `major`, `minor`, or `patch`. Anything outside that grammar returns `unknown`; fixed-U tests explicitly cover prerelease, commit hash, Chromium milestone, four-component and legacy-letter OpenSSL forms. The skill separately requires upstream-native compatibility review for unsupported schemes and 0.x series.

## Freshness and failure contract

`--no-fetch` is explicitly offline test mode. It reuses the local remote-tracking ref but sets `fresh_fetch=false` and leaves `audit_status=snapshot-only`; it cannot support a fresh current result. The CLI catches Git nonzero exit, Git timeout, `OSError`, and helper `ValueError`, prints `Snapshot failed; no fresh audit is available: ...` to stderr and returns 1. A fetch failure occurs before dependency-watch storage is created, so stale `origin/dev` is never silently reused as fresh. A missing required manifest also fails before the run root. A later unreadable required manifest can leave only a partial run-local source directory; because failure happens before successful snapshot/candidate JSON publication, it remains non-success evidence.

Successful `snapshot` CLI output includes exact commit, freshness flag, snapshot-only status, storage root, run directory, and source/gitlink/skipped counts. Successful `compare` output contains current, candidate and classification.

## Existing-owner-first disposition

This helper is support implementation for the already implemented `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` responsibility. Fabushi keeps the behavior in existing repository quality/test-evidence governance rather than copying a Telegram-specific Python monitor. Existing owners are root repository dependency-advisory governance, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, and `projects/fabushi-communication-platform/quality/evidence-contract.md`. The shipping Electron updater, platform-core runtime, Product Tasks/Automations, Agent/Coordinator/Host/Runner, and build-time acquisition inventory as decision owner remain rejected.

No dependency pin, product updater state, runtime code, platform-core production code, build, test execution, backport, PR, release, deployment or acceptance state changes in this source closure. No ADR is required because the single owner already exists.

## Authored contracts, not run

- `CONTRACT-TDRP-DEPENDENCY-WATCH-SNAPSHOT-001`
- `CONTRACT-TDRP-DEPENDENCY-WATCH-STORAGE-001`
- `CONTRACT-TDRP-DEPENDENCY-WATCH-CANDIDATES-001`
- `CONTRACT-TDRP-DEPENDENCY-WATCH-VERSION-COMPARE-001`
- `FAULT-TDRP-DEPENDENCY-WATCH-FRESHNESS-001`

Fixed-U upstream test cases were read but not executed: `VersionTests.test_stable_ordering_and_prefixes`; `SnapshotTests.test_fetches_remote_without_touching_working_tree`; `SnapshotTests.test_fetch_failure_never_uses_stale_ref`; `SnapshotTests.test_missing_required_manifest_fails`; `SnapshotTests.test_offline_snapshot_is_marked_unverified`; and `SnapshotTests.test_linked_worktree_uses_common_report_storage`.

Stage A validation remains `not-run`; acceptance remains false.
