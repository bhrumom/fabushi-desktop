# Dependency Watch upstream regression fixture — complete fixed-U read

Task round: 123  
Source evidence round: 154  
Stage: A / zero executable validation  
Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea`  
Recursive-tree index: 64  
Blob size: 4,347 bytes / 99 logical lines

## Callers, consumers, and owner

The file imports the sibling `watch` helper and directly calls `watch.git`, `watch.snapshot`, and `watch.compare`. It is a standard-library `unittest` fixture with a self-entrypoint `if __name__ == '__main__': unittest.main()`; Python test discovery is its executable consumer boundary. Fixed-U Dependency Watch `SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8` invokes `watch.py snapshot` and `watch.py compare` but does not invoke `watch_test.py` as workflow logic. No production state, findings database, dependency pin, updater state, or runtime lifecycle is owned here.

Existing-owner-first therefore keeps this source under `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` and the existing Fabushi repository quality/test-evidence governance. No new owner, ADR, product runtime, Telegram-specific test harness, or platform-core production delta is needed.

## VersionTests regression contract

`VersionTests.test_stable_ordering_and_prefixes` has ten subtest cases.

Stable positive controls:
- `v1.23.5 -> v1.23.10` must be `patch`.
- `n6.1.6 -> n6.2.0` must be `minor`.
- `openssl-3.2.1 -> openssl-4.0.0` must be `major`.
- `1.2.9 -> 1.2.8` must be `older`.
- `v1.2.3 -> 1.2.3` must be `same`.

Fail-closed negative controls:
- prerelease `v1.2.4-rc1`,
- hash `e2d0e88d -> 48bc60bdb1`,
- milestone `M123 -> M140`,
- four-component `1.2.3.4`,
- legacy-letter OpenSSL `1.1.1w`

must all classify `unknown`. The regression therefore guards both numeric ordering and refusal to force unsupported schemes through the stable-version comparator.

## SnapshotTests fixture

`setUp` creates a temporary source root, initializes a remote Git repository on branch `dev`, sets fixture identity, writes every path in `watch.REQUIRED`, commits, and clones a checkout. `commit` stages all fixture changes and makes the fixture commit.

### Fresh fetch without working-tree mutation

`test_fetches_remote_without_touching_working_tree` changes the remote prepare manifest to a pinned dependency line, captures the remote HEAD object, stages a synthetic gitlink with mode `160000`, commits the remote, and then writes an unrelated unfinished local edit into the checkout's same manifest. Before snapshot it captures checkout `status --porcelain` and HEAD.

After `watch.snapshot(self.repo)`, the regression requires:
- the saved run source copy to contain the remote pinned bytes, not the local edit;
- `fresh_fetch=true`;
- snapshot `commit` to equal current remote HEAD;
- `gitlinks` to equal exactly `[{'path': 'third-party', 'object': oid}]`;
- checkout status to equal the pre-snapshot bytes;
- checkout HEAD to equal the pre-snapshot bytes;
- no `latest.md` at the helper storage root.

This is the concrete upstream proof surface for immutable exact-tree/blob reads and working-tree/HEAD non-mutation.

### Fresh-fetch failure

`test_fetch_failure_never_uses_stale_ref` changes origin to a nonexistent path, requires `watch.snapshot` to raise `subprocess.CalledProcessError`, and requires checkout `.git/dependency-watch` not to exist. The regression prevents stale `refs/remotes/origin/dev` reuse from being mistaken for a fresh audit and verifies the failure occurs before helper storage creation.

### Required-manifest failure

`test_missing_required_manifest_fails` removes `snap/snapcraft.yaml` in the remote, commits, and requires `watch.snapshot` to raise `ValueError`. Required source authority therefore cannot silently shrink.

### Offline freshness

`test_offline_snapshot_is_marked_unverified` calls `watch.snapshot(self.repo, fetch=False)` and requires `fresh_fetch=false` and `audit_status='snapshot-only'`. Offline mode is explicitly unverified evidence.

### Linked-worktree common storage

`test_linked_worktree_uses_common_report_storage` creates a detached linked worktree, snapshots it offline, and requires reported `storage_root` to equal the primary checkout's resolved `.git/dependency-watch`. Linked worktrees therefore share the common Git-dir evidence root instead of creating a second report owner.

## Fabushi regression mapping

The prior ledger named these upstream cases under a nonexistent `DependencyWatchTests` class. The exact fixed-U class names are `VersionTests` and `SnapshotTests`; this round corrects the case IDs instead of preserving the inaccurate alias.

Authored, not run:
- `CONTRACT-TDRP-DEPENDENCY-WATCH-UPSTREAM-VERSION-REGRESSION-001`
- `CONTRACT-TDRP-DEPENDENCY-WATCH-UPSTREAM-SNAPSHOT-REGRESSION-001`

No upstream test, Fabushi test, build, lint, validator, generator, package, deployment, installation, or runtime acceptance is executed in Stage A.
