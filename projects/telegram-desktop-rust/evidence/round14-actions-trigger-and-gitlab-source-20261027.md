# Round 14: exact-head Actions trigger and GitLab source-access evidence (2026-10-27)

## Identity and GitHub observations

- Repository: bhrumom/fabushi-desktop; PR #178 remains open/draft at pre-write head `f8c27b3a273cf0ac69337d2cafc6df73271e664e`; base main ref was `cfef45568293a22deb32a6cdef17916f3ec19648` at initial read.
- GitHub REST `GET /repos/bhrumom/fabushi-desktop/actions/runs?head_sha=f8c27b3a273cf0ac69337d2cafc6df73271e664e&per_page=100` returned `total_count: 0, workflow_runs: []`.
- GitHub REST `GET /repos/bhrumom/fabushi-desktop/commits/f8c27b3a273cf0ac69337d2cafc6df73271e664e/check-runs?per_page=100` returned `total_count: 0, check_runs: []`.
- GitHub REST `GET /repos/bhrumom/fabushi-desktop/commits/f8c27b3a273cf0ac69337d2cafc6df73271e664e/status` returned `state: pending, total_count: 0, statuses: []`. Empty statuses are **not** a test pass.
- Branch run listing returned prior run `40515425001` at `816050450309044dae2bd6aba79d369077696341`, failed; it is *not* exact-head evidence for `f8c27b3a273cf0ac69337d2cafc6df73271e664e`.
- Pre-write commit metadata confirms `f8c27b3a273cf0ac69337d2cafc6df73271e664e` modifies `projects/telegram-desktop-rust/evidence/round13-exact-head-actions-and-gitlab-access-20261027.md`. This matches the workflow's `pull_request.paths` entry `projects/telegram-desktop-rust/**`.
- `.github/workflows/tdrp-revision9-authority.yml` at that head specifies `pull_request` matching TDRP paths, `push` only for `main`, and `workflow_dispatch`. The `candidate-authority` job invokes `node scripts/test-tdrp-revision9-identity-contract.mjs`, then `node scripts/validate-tdrp-revision9-authority.mjs`; the accepted-gate job is separately declared. Workflow definitions prove only intended wiring, not actual execution.
- GitHub connector fetch of `/actions/permissions` and `/actions/workflows` returned an endpoint allowlist `INVALID_ARGUMENT`. Repo-wide Actions enabled/disabled or actor permissions remain unverified. No exact-head run means no exact-head jobs/logs or successful coverage-gate evidence can be asserted.

## GitLab original repository access

- Intended GitLab project ID: `7705067`; requested cppgir commit: `47cf94f83b54cda59018135601e19d7fb0c77776`.
- Attempted raw GitLab REST repository-tree API page 1 via web retrieval: `DisabledError`. A read-only container curl to the same GitLab API reported `Could not resolve host: gitlab.com` (HTTP `000`). The GitHub connector only permits approved GitHub hosts and cannot substitute for GitLab raw repository evidence.
- Consequently no GitLab exact-commit tree page, complete tree, `.gitmodules` blob, nested gitlink, patch, LFS object, or generated-input blob was verified in this round. No upstream lock/inventory/ledger/dossier source claims are updated.

## Fail-closed outcome

The current candidate has no exact-head executed identity-contract or coverage-gate result and the GitLab recursive source evidence remains incomplete. Do **not** infer `unknown=0`, `unread=0`, `omitted=0`, production implementation acceptance, or independent release ACCEPT. Keep `accepted=false`, `baseline_ready=false`, and task `status=next`. This document is an access/negative-evidence record, not authority acceptance.
