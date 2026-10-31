# Round 15: exact-head Actions and cppgir source-access verification (2026-10-27)

## Live authority and status

- Repository main exact ref: `cfef45568293a22deb32a6cdef17916f3ec19648`. PR #178 is open/draft, with inspected pre-write branch HEAD `a29e8fb6838f8fc95965e78f8f99b8d442978435`.
- The TDRP Revision 9 workflow at the inspected HEAD has `pull_request.paths` including `projects/telegram-desktop-rust/**`; its candidate-authority job explicitly invokes `scripts/test-tdrp-revision9-identity-contract.mjs` and `scripts/validate-tdrp-revision9-authority.mjs`. Definition does not establish execution.
- GitHub REST `actions/runs?head_sha=a29e8fb6838f8fc95965e78f8f99b8d442978435&per_page=100`: `total_count=0`, `workflow_runs=[]`.
- GitHub REST `commits/a29e8fb6838f8fc95965e78f8f99b8d442978435/check-runs?per_page=100`: `total_count=0`, `check_runs=[]`.
- GitHub REST `commits/a29e8fb6838f8fc95965e78f8f99b8d442978435/status`: `state=pending`, `total_count=0`, `statuses=[]`.
- Branch-filtered Actions runs page (100 per page) reports `total_count=22`; none of its 22 returned runs matches `a29e8fb6838f8fc95965e78f8f99b8d442978435`. Most recent named TDRP run `40515425001` belongs to `816050450309044dae2bd6aba79d369077696341`, not this head. Its failure cannot establish any current-head gate result.
- The connector refuses GitHub `actions/permissions` and `actions/workflows` REST collection endpoints with `INVALID_ARGUMENT` URL allowlist error. Repository Actions enablement and permissions remain unverified. No current-head run ID exists from which current-head jobs, logs or artifacts could be inspected.
- The source-of-truth at current branch states TDRP Revision 9 and FBCP Revision 7, all executable verification only in GitHub Actions. No local test/build was run.

## GitLab original source acquisition

- Requested exact repository: GitLab project `7705067`, `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776`.
- Attempted GitLab REST `/api/v4/projects/7705067/repository/tree?ref=47cf94f83b54cda59018135601e19d7fb0c77776&per_page=100&page=1` through available web fetch; returned `DisabledError`, not a repository tree. This cannot validate even page 1. Prior round documents container DNS failure; that prior result was not assumed to be a new successful retrieval.
- GitHub connector fetch only supports GitHub hosts, not GitLab original source; no source-complete GitLab tree page, `.gitmodules` blob, recursive gitlinks, LFS/patch/generated input or exact file blobs were obtained in this round.

## Fail-closed disposition

No upstream lock, recursive inventory, parity ledger, dossier, production owner, acceptance or workflow was changed. A missing check is neither pass nor proof of Actions being disabled. `accepted=false`, `baseline_ready=false`, `status=next`; source completeness and independent release ACCEPT remain unverified. This is only a negative-evidence receipt bound to the inspected pre-write head, not production acceptance.
