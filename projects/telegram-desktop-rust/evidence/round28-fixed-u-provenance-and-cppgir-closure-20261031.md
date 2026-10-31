# Round 28 — fixed-U provenance and cppgir closure reconciliation (2026-10-31)

Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 3
Checkpoint parent: `5926a13630342bb83c450c082fbb640d3b34ec75`
Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Stage A execution boundary

- Manual Git/API/source evidence only.
- No build, typecheck, lint, schema validator, test, benchmark, package, deploy, workflow dispatch, or Actions execution/query was performed.
- Acceptance remains fail-closed.

## Fixed-U root identity

- Git commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Git tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`.
- Recursive root tree: `truncated=false`.
- Root entries: 7,194 total; 6,859 blobs; 299 trees; 36 direct gitlinks.
- Root `.gitmodules` blob: `2a71e861fab57dfd74d66e787867f2ede203a099`.

## parity-ledger provenance reconciliation

The ledger had 54 source-responsibility rows rooted in the historical commit `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`.

- 39 rows have the exact same `source_path -> blob SHA` at fixed U and were re-anchored to `source_commit=f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- 15 rows differ at fixed U and retain their historical source commit/blob provenance pending source-semantic reread; no changed blob was silently upgraded.
- Changed rows:
  - `TDRP-R9-SEARCH-ROW-REPLACEMENT-001`: `Telegram/SourceFiles/boxes/share_box.cpp` historical `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be` -> fixed-U `f4787dbb34cdb5fb2af78a69ae7537e91e06395a`
  - `TDRP-R9-SHARE-SEND-OPTIONS-001`: `Telegram/SourceFiles/boxes/share_box.cpp` historical `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be` -> fixed-U `f4787dbb34cdb5fb2af78a69ae7537e91e06395a`
  - `TDRP-R9-SHARE-DESTINATION-POLICY-001`: `Telegram/SourceFiles/boxes/share_box.cpp` historical `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be` -> fixed-U `f4787dbb34cdb5fb2af78a69ae7537e91e06395a`
  - `TDRP-R9-SHARE-LINK-ENTITY-OFFSET-001`: `Telegram/SourceFiles/boxes/share_box.cpp` historical `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be` -> fixed-U `f4787dbb34cdb5fb2af78a69ae7537e91e06395a`
  - `TDRP-R9-SHARE-THREAD-DESTINATION-001`: `Telegram/SourceFiles/boxes/share_box.cpp` historical `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be` -> fixed-U `f4787dbb34cdb5fb2af78a69ae7537e91e06395a`
  - `TDRP-R9-SHARE-TOPIC-ELIGIBILITY-001`: `Telegram/SourceFiles/boxes/share_box.cpp` historical `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be` -> fixed-U `f4787dbb34cdb5fb2af78a69ae7537e91e06395a`
  - `TDRP-R9-SHARE-FORWARD-IDEMPOTENCY-001`: `Telegram/SourceFiles/boxes/share_box.cpp` historical `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be` -> fixed-U `f4787dbb34cdb5fb2af78a69ae7537e91e06395a`
  - `TDRP-R9-SHARE-FORWARD-PRIVACY-001`: `Telegram/SourceFiles/boxes/share_box.cpp` historical `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be` -> fixed-U `f4787dbb34cdb5fb2af78a69ae7537e91e06395a`
  - `TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001`: `Telegram/SourceFiles/boxes/share_box.cpp` historical `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be` -> fixed-U `f4787dbb34cdb5fb2af78a69ae7537e91e06395a`
  - `TDRP-R9-CODEX-LEAF-REVIEW-001`: `.agents/shared/codex-cli-leaves.md` historical `659da44a8083c433c4e9b59ca28f64ba525bfbf8` -> fixed-U `11c59b68df43e442e596c3052296fc51a044d46a`
  - `TDRP-R9-AI-PHASE-EFFORT-GOVERNANCE-001`: `.agents/shared/phase-effort.md` historical `dd07a4c611922a29a6892332db98afe76cd58952` -> fixed-U `8d7b385dbb526e87fc090774ee35191e1619bb08`
  - `TDRP-R9-ADAPTIVE-TEST-LOOP-GOVERNANCE-001`: `.agents/shared/test-loop.md` historical `83c4732bf6139f7c6557f18dd87bb35c3d68808d` -> fixed-U `bb51685e417759939cadc1e2e74b1c0199fcb8f0`
  - `TDRP-R9-CODE-HEALTH-GOVERNANCE-001`: `.agents/skills/code-health/SKILL.md` historical `6885938b3d186b55f5d69cb8b9b9a96bbca0e097` -> fixed-U `101e69bc18d8f37c73d3b077f04d266bd939360f`
  - `TDRP-R9-CODE-HEALTH-COMMAND-ORCHESTRATION-001`: `.agents/skills/code-health/scripts/health.py` historical `f08675ef7a03c016d85f91c60b8f29e99454abe9` -> fixed-U `4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d`
  - `TDRP-R9-CODE-HEALTH-INTEGRATED-REGRESSION-001`: `.agents/skills/code-health/scripts/health_test.py` historical `2176aea672fd9ed16c01342939fbffb41490ae73` -> fixed-U `6058fa64a5121c966021d8589b2010480aec8794`

The root ledger `upstream_commit` is now the fixed U. Existing recursive coverage counts are retained as historical progress data and are not promoted to current-U source-completeness evidence; `baseline_ready` / acceptance remain false elsewhere.

## cppgir recursive provenance

Current fixed-U parent evidence is complete only through the parent gitlink declaration:

- parent: `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095`
- parent `.gitmodules` blob: `c92fbea3467f72e188cba4cbfe51c8c68a7e2e77`
- declared URL: `https://gitlab.com/mnauw/cppgir.git`
- parent tree gitlink: `external/glib/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776`

The current environment still cannot obtain authoritative original-host GitLab recursive tree/blob pages for `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776`. Therefore cppgir recursive source closure remains open.

A separate GitHub read confirms `nonstd-lite/expected-lite@95b9cb015fa17baa749c2b396b335906e1596a9e` has `truncated=false`, 55 entries, and no nested gitlinks. This is not used to assert cppgir reachability because the pinned cppgir tree/.gitmodules was not independently obtained from GitLab in this round.

## Result

Only source identities supported by exact commit/tree/blob evidence were updated. No changed upstream blob was treated as semantically equivalent without reread. Fixed-U recursive source completeness, cppgir original-host closure, build-time acquisitions, LFS/generated inputs, resources/assets/locales, packaging/update inputs, and behavior-level unknown/unread closure remain open.

## Post-publication provenance correction

A remote readback exposed two stale lock fields that were corrected with a second non-forced lease update: fixed U commit time is `2026-10-26T18:17:45Z`, and GitHub compare from historical ledger root `351204f3695b4a39bc3f4d2f230c061b9a01fe2a` to fixed U reports `ahead_by=245`, `behind_by=0`, `total_commits=245`. The compare returned 300 file records, so the lock records that list as incomplete rather than claiming an exact changed-path count.
