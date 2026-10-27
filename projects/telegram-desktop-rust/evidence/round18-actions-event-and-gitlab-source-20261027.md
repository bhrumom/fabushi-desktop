# Round 18 — GitHub PR event visibility, Actions access, and GitLab source gate (2026-10-27)

## Live anchors and observed facts
- GitHub main ref read as `cfef45568293a22deb32a6cdef17916f3ec19648`; PR #178 open/draft, same-repository source branch `work/tdrp-revision9-f985723-rebaseline-20261027`, HEAD `6b61fa742bdefdf24e8911e8b45a82bbc62a85d6` at investigation time.
- GitHub REST `/actions/runs?head_sha=6b61fa742bdefdf24e8911e8b45a82bbc62a85d6&per_page=10`: `total_count=0`. REST `/commits/6b61fa742bdefdf24e8911e8b45a82bbc62a85d6/check-runs`: `total_count=0`. These establish **missing exact-head evidence**, not why workflow scheduling failed.
- GitHub REST `/issues/178/events?per_page=100` returned `[]`. An empty issues-events response is **not** proof no synchronize webhook was delivered: webhook deliveries and GitHub Actions settings were not available via this API surface.
- REST branch workflow query reports 22 runs. Two most recent runs (IDs 40515425001, 40515424774) are `pull_request` at earlier `816050450309044dae2bd6aba79d369077696341`, both failure, created 2026-10-27T13:49:08Z. They cannot validate this HEAD.
- Connector `get_repo` for authenticated identity reports repository permissions `admin=true, maintain=true, pull=true, push=true`; **repository role does not prove** that its GitHub App installation has Actions administration API permission.
- GitHub connector rejected REST `/actions/permissions` and `/actions/workflows?per_page=100` with `INVALID_ARGUMENT` / endpoint not allowed. Consequently neither repository Actions permissions nor individual workflow enabled/disabled status has been verified.
- Main `.github/workflows/tdrp-revision9-authority.yml` blob `b89c935927a2b8159b75c47988283e5f045252a4`, PR HEAD blob `4b6821dc0ac3db8323b1956748f0f1c5d0535807`: both define `pull_request` paths matching `projects/telegram-desktop-rust/**` and their workflow file; head also explicitly lists the identity contract script. PR changed-files API confirms matching paths. This rules out *obvious* path mismatch, not event delivery or settings suppression.
- `.github/workflows/tdrp-search-responsibility.yml` at both refs is blob `9b82dab2f13f7982257a7a47869297b83ae99594`; configured only for `push: main` and `workflow_dispatch`. Its absence on an unmerged PR is expected.

## Original GitLab source constraint
- Attempted direct GitLab project 7705067 original tree endpoint for `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776`; web access returned `DisabledError`. GitHub connector fetch is restricted to approved GitHub hosts. No authenticated GitLab original repository reader was available for this check.
- **Not obtained**: complete paginated tree, pinned `.gitmodules`, nested gitlinks, LFS/external objects, patches, generator inputs, complete source blobs and SHA evidence. Do not claim `unknown=0`, `unread=0`, or `omitted=0`.

## Gate decision
- Root cause of zero exact-head Actions remains **unresolved**. Do not equate no runs with passing CI or definitively attribute it to draft PR, settings, webhook, permissions, or paths.
- No changes authorized to `upstream.lock.json`, `inventory/index.json`, `parity-ledger.json`, source dossiers, production code, or acceptance flags. Keep `accepted=false`, `baseline_ready=false`, `status=next` and independent release non-ACCEPT until primary-source and exact-head production evidence close.
- Next inspect Actions workflow enablement and PR synchronize webhook delivery with an appropriately privileged GitHub API/browser session; independently obtain GitLab project 7705067 raw paginated repository objects with authenticated original-host access, then reconcile source hashes and rerun exact-head validation.
