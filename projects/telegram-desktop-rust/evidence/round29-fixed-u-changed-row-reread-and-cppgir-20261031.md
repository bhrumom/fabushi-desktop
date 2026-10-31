# Round 29 — fixed-U changed-row semantic reread and cppgir retry

Date: 2026-10-31  
Stage: A (manual Git/API/source evidence only; no CI/build/test/schema/lint/package/deploy execution)  
Checkpoint before write: `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1@5c4d02ce42794b7ebc0878e4c7c4654765d706f0`  
Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Scope

This round reread the 15 parity-ledger rows whose source blobs changed between historical `351204f3695b4a39bc3f4d2f230c061b9a01fe2a` and fixed U. Re-anchoring was allowed only where the row's cited source symbols and behavior contract remain supported by the fixed-U blob.

## Result

All 15 rows satisfy that narrow re-anchoring condition. Nine rows share `Telegram/SourceFiles/boxes/share_box.cpp`; the compare from historical source to fixed U contains 85 additions and 0 deletions, all limited to new includes and a new `FastShareImage` implementation after the cited regions. The remaining six rows have additive/refactoring deltas that preserve their mapped contract:

| Responsibility | Source | historical blob -> fixed-U blob | Fixed-U reread verdict |
| --- | --- | --- | --- |
| TDRP-R9-SEARCH-ROW-REPLACEMENT-001 | `Telegram/SourceFiles/boxes/share_box.cpp` | `b7bf9f0d…` -> `f4787dbb…` | cited constructor replacement handling, username search and chat-filter rebuild remain unchanged |
| TDRP-R9-SHARE-SEND-OPTIONS-001 | same | `b7bf9f0d…` -> `f4787dbb…` | send-menu/schedule/silent submit path remains unchanged |
| TDRP-R9-SHARE-DESTINATION-POLICY-001 | same | `b7bf9f0d…` -> `f4787dbb…` | FastShare filters and submit authorization source remain unchanged |
| TDRP-R9-SHARE-LINK-ENTITY-OFFSET-001 | same | `b7bf9f0d…` -> `f4787dbb…` | FastShareLink formatted-comment/entity-shift source remains unchanged |
| TDRP-R9-SHARE-THREAD-DESTINATION-001 | same | `b7bf9f0d…` -> `f4787dbb…` | Chat/topic/sublist/community child selection and canonical thread projection remain unchanged |
| TDRP-R9-SHARE-TOPIC-ELIGIBILITY-001 | same | `b7bf9f0d…` -> `f4787dbb…` | forum-topic selection/eligibility source remains unchanged |
| TDRP-R9-SHARE-FORWARD-IDEMPOTENCY-001 | same | `b7bf9f0d…` -> `f4787dbb…` | submit/default-forward/send-options source remains unchanged |
| TDRP-R9-SHARE-FORWARD-PRIVACY-001 | same | `b7bf9f0d…` -> `f4787dbb…` | forward options/fill/submit source remains unchanged |
| TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001 | same | `b7bf9f0d…` -> `f4787dbb…` | FastShare filter, peopleReceived and applyChatFilter source remains unchanged |
| TDRP-R9-CODEX-LEAF-REVIEW-001 | `.agents/shared/codex-cli-leaves.md` | `659da44a…` -> `11c59b68…` | 3-line model-scope delta; availability/sandbox/start/continue/recovery/fallback contract remains |
| TDRP-R9-AI-PHASE-EFFORT-GOVERNANCE-001 | `.agents/shared/phase-effort.md` | `dd07a4c6…` -> `8d7b385d…` | +6 harness model-scope lines; effort/lens/fallback/evidence contract remains |
| TDRP-R9-ADAPTIVE-TEST-LOOP-GOVERNANCE-001 | `.agents/shared/test-loop.md` | `83c4732b…` -> `bb51685e…` | helper docs refactor + host-load gate; core evidence state machine/recovery contract remains |
| TDRP-R9-CODE-HEALTH-GOVERNANCE-001 | `.agents/skills/code-health/SKILL.md` | `6885938b…` -> `101e69bc…` | harness metrics added; exact-source ledger/atlas/survey/check/proposal/audit/report governance remains |
| TDRP-R9-CODE-HEALTH-COMMAND-ORCHESTRATION-001 | `.agents/skills/code-health/scripts/health.py` | `f08675ef…` -> `4fd8d801…` | harness scan artifacts added; cited state/source/worktree/command orchestration functions remain |
| TDRP-R9-CODE-HEALTH-INTEGRATED-REGRESSION-001 | `.agents/skills/code-health/scripts/health_test.py` | `2176aea6…` -> `6058fa64…` | additional regression cases/interpreter portability; all cited integrated test classes remain |

The ledger source commits/blobs and provenance statements are therefore re-anchored to fixed U. This does not change `implementation_status`, `validation_status`, coverage counters, `baseline_ready`, or acceptance.

## cppgir recursive closure

Verified parent evidence remains:

- `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095`;
- parent `.gitmodules` blob `c92fbea3467f72e188cba4cbfe51c8c68a7e2e77` declares `https://gitlab.com/mnauw/cppgir.git`;
- parent tree pins `external/glib/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776`.

This round retried the original GitLab API v4 recursive tree endpoint and the pinned-commit tree page. Both requests were blocked by the execution network layer before repository content was returned. No original-host recursive tree, blob list, or nested `.gitmodules` evidence was obtained. Therefore cppgir remains `git_tree_verified=false`, nested closure remains blocked/fail-closed, and the standalone GitHub read of `nonstd-lite/expected-lite@95b9cb015fa17baa749c2b396b335906e1596a9e` is not used as reachability proof.

## Stage A execution discipline

No GitHub Actions runs/statuses were queried or dispatched. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E, packaging, deployment or installation verification was run.
