# Round 22 — exact-head Actions and release wiring evidence (2026-10-28)

## Live authority / lease
- `main`: `cfef45568293a22deb32a6cdef17916f3ec19648` (GitHub refs/heads/main read).
- PR #178: open, draft, unmerged; branch `work/tdrp-revision9-f985723-rebaseline-20261027`; inspected exact head `0ff1f0721dd7ac4a31b731cc084ca226b4fb53b9` (GitHub PR read). PR base_sha metadata `3f8e7ae768d71897958198dc4924e10c77098a5c` must not be mistaken for current main.
- PR-head Git tree REST recursive `truncated=false`, 9,521 entries, includes 26 workflow YAML paths. This is the Fabushi tree only and proves nothing about full upstream recursion.

## Current-head CI (not accepted)
- REST `GET /actions/runs?head_sha=0ff1f0721dd7ac4a31b731cc084ca226b4fb53b9&per_page=100`: total_count=0.
- REST `GET /commits/0ff1f0721dd7ac4a31b731cc084ca226b4fb53b9/check-runs?per_page=100`: total_count=0.
- REST `GET /commits/0ff1f0721dd7ac4a31b731cc084ca226b4fb53b9/status`: state=pending, statuses=[], total_count=0. Zero statuses is not success.
- Repository-wide `/actions/runs?event=pull_request&per_page=10` shows **earlier** PR #178 runs for `816050450309044dae2bd6aba79d369077696341`: TDRP Revision 9 run `40515425001` failure and Rust desktop runtime run `40515424774` failure, created 2026-10-27T13:49:08Z. Also previous `481f0ccfffc4c1f7c70a5cc8d4f3f464b723f40a` runs. Thus the workflow operated for earlier heads; none can be attributed to the inspected current head. This does not prove delivery or enabled state for the present synchronize event.

## Static workflow / production wiring (not an E2E acceptance)
- `.github/workflows/tdrp-revision9-authority.yml` blob `4b6821dc0ac3db8323b1956748f0f1c5d0535807`: `pull_request.paths` includes `projects/telegram-desktop-rust/**` and exact PR head checkout/identity assertion; also `push` to main and `workflow_dispatch`. It is statically eligible for an evidence-file change but no current-head dispatch is observed.
- `.github/workflows/release-macos-main.yml` blob `b0a3a4d5503e6104c083f85e31475523a2b30643`: `workflow_dispatch` with exact source SHA / publication flags and `push` to main with path filters including `projects/telegram-desktop-rust/**`; **no pull_request trigger**. Its publish job pins checkout SHA and requires signing/notarization credentials; public publication requires immutable candidate promotion and acceptance. Presence of wiring is not proof of run, signed package, or ACCEPT.
- `frontend/src/main.tsx` blob `ccb3e932cb9b017742c7d6a6ecb77c2faf5814d7`: imports/mounts `ProductionRenderer` and production bootstrap. This is a canonical entrypoint source inspection, not a packaged runtime assertion.
- `AGENTS.md` blob `e6a1f21ba5206cdc807b6f9738dad2cec2942100`; FBCP SOT blob `283858bb70bab14e240da781a20e749ed1f1359e`; TDRP SOT blob `c181c90ac52043975ef508042f4c20181211d9f5` were read at inspected head. Their spec-first and zero-counter acceptance gates remain binding.

## External blockers and scope integrity
- Connector `fetch` rejected `/actions/workflows?per_page=100`, `/actions/permissions`, `/hooks?per_page=100`. The app's repository metadata reports `permissions.admin=true` for the linked user, which does **not** grant these API endpoints through this connector. Workflow enabled status, Actions permissions and PR synchronize webhook delivery remain unverified. Do not guess why new HEAD has zero runs.
- No GitLab tool is available in this tool environment. GitLab project 7705067 raw tree/.gitmodules/recursive gitlinks/LFS/patch/generated inputs/blob SHA for `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` remain unread and unverified.
- No changes to lock, inventory, parity ledger, dossiers, production, workflows or release settings. `accepted=false`, `baseline_ready=false`, `status=next`. No merge or release.

## Reproducible GitHub API anchors
- `GET /repos/bhrumom/fabushi-desktop/git/ref/heads/main`
- `GET /repos/bhrumom/fabushi-desktop/pulls/178`
- `GET /repos/bhrumom/fabushi-desktop/git/trees/0ff1f0721dd7ac4a31b731cc084ca226b4fb53b9?recursive=1`
- `GET /repos/bhrumom/fabushi-desktop/actions/runs?head_sha=0ff1f0721dd7ac4a31b731cc084ca226b4fb53b9&per_page=100`
- `GET /repos/bhrumom/fabushi-desktop/commits/0ff1f0721dd7ac4a31b731cc084ca226b4fb53b9/check-runs?per_page=100`
- `GET /repos/bhrumom/fabushi-desktop/commits/0ff1f0721dd7ac4a31b731cc084ca226b4fb53b9/status`
- `GET /repos/bhrumom/fabushi-desktop/actions/runs?event=pull_request&per_page=10`
