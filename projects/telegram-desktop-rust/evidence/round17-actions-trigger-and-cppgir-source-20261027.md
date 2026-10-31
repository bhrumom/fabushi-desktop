# Round 17 — exact-head Actions trigger and cppgir acquisition audit (2026-10-27)

Audit starting PR #178 head: `494921ae5d1534940fbe5f8209dd164f4fcc3dac`; main observed: `cfef45568293a22deb32a6cdef17916f3ec19648`. PR is open/draft, same-repository branch `work/tdrp-revision9-f985723-rebaseline-20261027`. Read-only authority audit; not acceptance.

## Exact-head CI

- REST `GET /actions/runs?head_sha=494921ae5d1534940fbe5f8209dd164f4fcc3dac&per_page=100`: `total_count=0`. REST `GET /commits/494921ae5d1534940fbe5f8209dd164f4fcc3dac/check-runs`: `total_count=0`; combined status statuses array empty. Therefore no passing exact-head execution is evidenced.
- REST branch workflow-runs query reports 22 historical runs. Latest displayed runs are two failures at `816050450309044dae2bd6aba79d369077696341` (2026-10-27T13:49:08Z), not the audit head. Historical runs cannot be inherited.
- Exact-head `.github/workflows/tdrp-revision9-authority.yml` (blob `4b6821dc0ac3db8323b1956748f0f1c5d0535807`) declares `pull_request.paths` for `projects/telegram-desktop-rust/**`, `scripts/*tdrp*.mjs`, and its own workflow path; the PR changed-files API includes all these categories. It declares `workflow_dispatch` and `push` only on `main`. It uses an exact PR head checkout. The **main** version is blob `b89c935927a2b8159b75c47988283e5f045252a4` and also contains these PR path filters.
- `.github/workflows/tdrp-search-responsibility.yml` is only `push: main` and `workflow_dispatch`, so its absence on an unmerged PR head is expected from its trigger definition; it is not evidence of successful Search acceptance.
- Repository REST `/actions/runs?per_page=5` returned an unrelated successful `workflow_dispatch` at 2026-10-27T15:38:02Z, establishing that at least some GitHub Actions runs exist at repository level. It does **not** establish Actions enablement or permitted pull_request triggers for this branch.
- The connector rejected `/actions/workflows?per_page=100` and `/actions/permissions`; PR issue-events list returned empty. No definitive Actions-configuration root cause or PR delivery event record was obtained. A draft PR is not, by itself, proof of suppression. Treat zero runs as an unresolved trigger/delivery/permissions gap, not as a passing gate.

## GitLab source acquisition

- Attempted public web retrieval of `https://gitlab.com/api/v4/projects/7705067/repository/tree?ref=47cf94f83b54cda59018135601e19d7fb0c77776&per_page=100&page=1`. The endpoint was disabled by the available web fetch environment. The GitHub connector only accepts supported GitHub hosts and is not a GitLab raw repository API.
- No GitLab tree page, pinned `.gitmodules`, recursive gitlink, LFS object, patch, generated input or exact source blob SHA was obtained. Project ID ↔ repository identity remains unverified from an original GitLab object. Do **not** infer zero missing entries or accept a new baseline.

## Fail-closed disposition

- No changes to `upstream.lock.json`, `inventory/index.json`, `parity-ledger.json`, source dossiers, production code, or acceptance flags. Keep `accepted=false`, `baseline_ready=false`, `status=next` and independent release verdict non-ACCEPT until actual exact-head evidence closes every required gate.
- Next executable checks: inspect the PR synchronization event delivery and repository Actions settings via an administrative-capable API, then obtain GitLab project 7705067 raw pages with authenticated original-host access; use source objects and complete hashes before any authority-ledger update.
