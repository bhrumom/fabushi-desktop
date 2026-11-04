# Round 154 — Dependency Watch upstream regression fixture

Task round: 123  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea`

Live desktop checkpoint was reread at `e4259b9b7b1fabab51a9216632e067bf315cef78`; spec-first CAS published `d6562479b20149b06fc9980baad5ea2cbefc2375`. Desktop main was reread as `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core exact authority remains `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed-U recursive tree is complete (`truncated=false`, 7,194 entries); target fixture is tree index 64 and exact blob size 4,347 bytes / 99 logical lines.

Direct code dependency is sibling `watch`; executable consumer boundary is Python `unittest` discovery / the file's `unittest.main()` entrypoint. Production dependency-watch `SKILL.md` calls `watch.py snapshot` and `watch.py compare`, not this regression fixture. No second durable dependency-advisory or test state owner exists here.

`VersionTests.test_stable_ordering_and_prefixes` contains ten exact subtests: five supported stable cases for patch/minor/major/older/same and five unsupported prerelease/hash/milestone/four-component/legacy-letter cases that must be `unknown`.

`SnapshotTests` creates a disposable `dev` remote with every required manifest and a cloned checkout. Fresh-fetch regression commits changed remote source plus a synthetic mode-160000 gitlink, leaves an unfinished local edit, then requires saved source bytes and commit identity to come from the remote while checkout status and HEAD stay unchanged, exact gitlink identity is preserved, and no `latest.md` exists. Additional regressions require fetch failure to raise before `.git/dependency-watch` storage exists, missing `snap/snapcraft.yaml` to raise `ValueError`, offline mode to remain `fresh_fetch=false/audit_status=snapshot-only`, and a detached linked worktree to use the primary checkout's resolved common `.git/dependency-watch` storage.

Existing-owner-first remains `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` / Fabushi repository quality/test-evidence governance. The unique ledger's prior nonexistent `DependencyWatchTests` aliases were corrected to the exact `VersionTests` / `SnapshotTests` class names. No new owner, ADR, product/runtime/updater state, dependency mutation, or platform-core production delta.

Authored, not run: `CONTRACT-TDRP-DEPENDENCY-WATCH-UPSTREAM-VERSION-REGRESSION-001` and `CONTRACT-TDRP-DEPENDENCY-WATCH-UPSTREAM-SNAPSHOT-REGRESSION-001`.

After closure: `source_entries_total=16202 / unknown=16154 / unread=16127 / omitted=0 / responsibilities_total=134 / unmapped_responsibilities=0`. Next fixed-U independent blob: `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`.

Zero-CI attestation: no GitHub Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance.
