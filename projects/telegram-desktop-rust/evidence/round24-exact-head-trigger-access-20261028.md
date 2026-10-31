# Round 24 — live exact-head CI and trigger-access evidence (2026-10-28)

## Lease and verified readings
- Repository: `bhrumom/fabushi-desktop`; PR #178 remains open/draft/unmerged at observed head `560abc52313c60b098a52e3a760272ccb542998f` and branch `work/tdrp-revision9-f985723-rebaseline-20261027`.
- Live main ref: `cfef45568293a22deb32a6cdef17916f3ec19648` (read from GitHub git/ref/heads/main). PR base_sha metadata is historical, not live main.
- Exact-head `GET /actions/runs?head_sha=560abc52313c60b098a52e3a760272ccb542998f&per_page=100`: `total_count=0`.
- Exact-head `GET /commits/560abc52313c60b098a52e3a760272ccb542998f/check-runs?per_page=100`: `total_count=0`.
- Exact-head `GET /commits/560abc52313c60b098a52e3a760272ccb542998f/status`: `pending`, `total_count=0`, no statuses. Zero runs/checks does not establish success.

## Workflow configuration versus actual invocation
- `.github/workflows/tdrp-revision9-authority.yml` blob `4b6821dc0ac3db8323b1956748f0f1c5d0535807`: `pull_request` path `projects/telegram-desktop-rust/**`; exact target `github.event.pull_request.head.sha || github.sha`; explicit exact-checkout assertion; push restricted to main, and workflow_dispatch configured.
- `.github/workflows/rust-desktop-runtime.yml` blob `620719ca0bf63cc040bbdf28372fe4a8062400d5`: `pull_request` path `projects/telegram-desktop-rust/**`; push branch and path filters; exact PR branch HEAD checkout and assertion. Static eligibility is not delivery or execution evidence.
- `desktop/scripts/check-agent-workspace-boundary.mjs` blob `a2ddb992fa459fd2328c924f9e39840a70f2f3d5` statically checks `frontend/src/main.tsx` / production renderer, Electron Main / preload, utility-process Rust Coordinator carrier, and independent Coordinator / Host / Runner ownership. The workflow declares actual Rust Host, Coordinator, Runner tests and desktop renderer checks. No exact-head production E2E or packaged acceptance was obtained.

## Permissions and source exclusions
- `get_repo` reports the connected identity's repository `permissions.admin=true`, but this is **not** Actions-management authorization for the GitHub App installation. Connector `fetch` rejected `/actions/workflows`, `/actions/permissions`, and `/hooks` as disallowed endpoint families, before returning raw endpoint payloads. Therefore workflow enabled status, Actions permissions and PR synchronize webhook delivery remain **unverified**, and zero runs' root cause is unresolved.
- Available connector inventory exposes GitHub but no GitLab raw-repository access action. Consequently raw GitLab project `7705067` `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` complete tree, .gitmodules, recursively nested gitlinks, LFS, patches, generator inputs and blob SHA remain **unverified**. Never convert guessed or partial source into accepted inventory.
- `projects/telegram-desktop-rust/upstream.lock.json` blob `3a66a16258af39bdf9621c3b57d4469d9360693d` still records candidate `f985723b552b30fe1c17d377d57f44e61a15a8c4`, `upstream.accepted=false`, `acceptance.baseline_ready=false`, `acceptance.accepted=false`.

## Fail-closed disposition
- No source lock, inventory, ledger, dossier, implementation, workflow, or test was modified in this round. This is a read-only evidence record, not source completeness or successful verification.
- Keep `status=next`, source closure open, no merge/release, no independent ACCEPT. Need Actions-management-capable raw reads and GitLab raw-project read before promoting source truth; independently continue examining shipping call-chain contracts on current head.
