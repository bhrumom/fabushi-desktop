# Round 16: exact-head Actions trigger and source access (2026-10-27)

## Live authority
- main ref: `cfef45568293a22deb32a6cdef17916f3ec19648` (GitHub Git ref/branch read).
- PR #178: open, draft, head branch `work/tdrp-revision9-f985723-rebaseline-20261027`, inspected exact head `f7e2333daa33c77b7871d63729c72e050b46b733` (GitHub PR and Git ref).
- Root `AGENTS.md`, FBCP Revision 7 SOT, TDRP Revision 9 SOT and `.github/workflows/tdrp-revision9-authority.yml` were read at inspected head.
- GitHub repository metadata reports authenticated connection `admin=true`, `push=true`; collaborator permission endpoint for `bhrum` reports `admin`. These are repository-level permissions, **not** evidence of the app token having Actions administration permission.

## GitHub exact-head executions
- `GET /repos/bhrumom/fabushi-desktop/actions/runs?head_sha=f7e2333daa33c77b7871d63729c72e050b46b733&per_page=100`: `total_count=0`, `workflow_runs=[]`.
- `GET /repos/bhrumom/fabushi-desktop/commits/f7e2333daa33c77b7871d63729c72e050b46b733/check-runs`: `total_count=0`, `check_runs=[]`.
- `GET /repos/bhrumom/fabushi-desktop/commits/f7e2333daa33c77b7871d63729c72e050b46b733/status`: `state=pending`, `total_count=0`, `statuses=[]`. This is *absence of reported status*, not evidence of success or a running job.
- Commit workflow-runs connector also reports `workflow_runs=[]`.
- No exact-head run ID exists, so no exact-head jobs, logs, artifacts, or passing test outcome can be read.
- TDRP workflow is present at inspected head and declares `pull_request.paths` matching `projects/telegram-desktop-rust/**`, plus `workflow_dispatch` and push on main; `candidate-authority` invokes the identity-contract test and Revision 9 authority validator. Definition alone is not proof of trigger/execution.
- Direct Actions `/actions/permissions` and `/actions/workflows?per_page=100` REST collection calls were rejected by connector endpoint allowlist (not authoritative HTTP responses). Repository Actions settings, enabled workflows, event delivery and installation's Actions permissions are **unverified**. Do not assert a root cause from absent runs.

## Upstream source
- Required source: GitLab project `7705067`, `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776`.
- GitLab REST tree page 1 attempted via accessible web fetch; received `DisabledError`, no GitLab tree payload.
- No trustworthy full tree, `.gitmodules`, recursive gitlinks, LFS, patches, generated inputs or exact blob SHA has been acquired. Neither unknown/unread/omitted counts nor completeness can be inferred as zero.

## Decision
No production code, workflow, lock, inventory, ledger, dossier, or acceptance changes are justified. Retain `accepted=false`, `baseline_ready=false` and independent release verdict not ACCEPT. No local builds/tests performed. This is a negative-evidence receipt for the **pre-write** inspected head, and does not assert that a new evidence-only commit has its own Actions execution.
