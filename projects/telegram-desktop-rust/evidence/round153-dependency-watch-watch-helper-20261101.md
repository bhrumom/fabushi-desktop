# Round 153 — Dependency Watch snapshot/compare helper

Task round: 122  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/dependency-watch/scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38`

Live desktop checkpoint was reread at `4431051c55093cf31669fb49aaa11f3fed223c5c`; spec-first CAS published `98ca5286b5fe0c2b63e8da0c4e9bb39ad48b176e`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed-U root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was reread complete (`truncated=false`, 7,194 entries); target helper is tree index 63 and exact blob size 6,655 bytes / 191 logical lines.

Direct workflow caller is dependency-watch `SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`; direct code/test consumer is `watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea`. Code search found the helper CLI path in the skill and direct `watch.snapshot`/`watch.compare` calls in the helper test. No separate findings/state owner is introduced.

Snapshot semantics: canonical repository top level; online fetch only `origin/dev` into `refs/remotes/origin/dev` with no tags and no recursive submodules; successful-fetch timestamp; one exact resolved commit; exact tree/blob reads; required regular manifests; selected build/Snap/workflow/manifest inputs; gitlinks recorded separately. Storage semantics: resolve Git common dir; shared `dependency-watch/runs` root; timestamped run; exact source copies; skipped reasons; successful snapshot/candidate JSON only after required readability.

Candidate output is navigation-only. Version compare recognizes only three numeric components with optional `v`, `n`, or `openssl-` prefix and returns same/older/major/minor/patch; unsupported schemes return unknown. Freshness fails closed: offline mode is test-only and unverified; Git/timeout/OS/missing/unreadable-required failures cannot yield a fresh audit. Fetch failure occurs before report-root creation.

Existing-owner-first remains `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001`; no updater/runtime/platform-core/second-monitor owner and no ADR. Authored, not run: `CONTRACT-TDRP-DEPENDENCY-WATCH-SNAPSHOT-001`, `CONTRACT-TDRP-DEPENDENCY-WATCH-STORAGE-001`, `CONTRACT-TDRP-DEPENDENCY-WATCH-CANDIDATES-001`, `CONTRACT-TDRP-DEPENDENCY-WATCH-VERSION-COMPARE-001`, `FAULT-TDRP-DEPENDENCY-WATCH-FRESHNESS-001`.

After closure: `source_entries_total=16202 / unknown=16155 / unread=16128 / omitted=0 / responsibilities_total=134 / unmapped_responsibilities=0`. Next fixed-U independent blob: `.agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea`.

Zero-CI attestation: no GitHub Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance.
