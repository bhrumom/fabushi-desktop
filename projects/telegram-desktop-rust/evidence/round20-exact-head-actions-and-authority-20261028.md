# Round 20 — exact-head Actions and authority reconciliation (2026-10-28)

## Exact live authority and read-only diagnostics
- Branch main HEAD from GitHub branches/main: `cfef45568293a22deb32a6cdef17916f3ec19648`.
- PR #178 remains open/draft, branch `work/tdrp-revision9-f985723-rebaseline-20261027`, examined HEAD `24f0a60c35e4126760f91dbd7681aeb82ae93d92`. Git ref read-back also equals that SHA. Its Git tree is `83ee881bc6f27ff20effd2c85cd865bf3dc8487a`.
- Exact-head `/actions/runs?head_sha=24f0a60c35e4126760f91dbd7681aeb82ae93d92` returned `total_count=0`; `/commits/24f0a60c35e4126760f91dbd7681aeb82ae93d92/check-runs` returned `total_count=0`; commit workflow-runs wrapper returned empty. Combined commit status endpoint returned `state=pending` with empty statuses, not a passing check. No exact-head CI acceptance can be claimed.
- GitHub connector fetch denied `/actions/workflows`, `/actions/permissions` and `/hooks` as disallowed endpoint families (HTTP wrapper INVALID_ARGUMENT 400), even though repository metadata reports the connected principal `admin=true`. Repo admin status is **not** evidence that the connector can access Actions administrative endpoints or webhook delivery records. The root cause of unscheduled runs is not established.
- `.github/workflows/tdrp-revision9-authority.yml` blob `4b6821dc0ac3db8323b1956748f0f1c5d0535807`: its `pull_request` path filter explicitly includes `projects/telegram-desktop-rust/**`, and its `TDRP_TARGET_SHA` binds to PR head SHA. On paper this evidence path is eligible; workflow enabled status and actual event delivery remain unknown.
- `scripts/test-tdrp-revision9-identity-contract.mjs` blob `6c3a2cc79df779a5abdef38773f11e750d60ed18` asserts historical ledger and acquisitions retain historical source commit, candidate index retains candidate identity and acceptance remains false. `scripts/validate-tdrp-revision9-authority.mjs` blob `c972812cfd5fe5fea8ab89556e0ded3968f24590` contains GitLab project tree pagination and candidate/historical separation. These are **static source checks**; no current-head Actions execution evidence exists.

## Canonical authority anchors
- Root `AGENTS.md` blob `e6a1f21ba5206cdc807b6f9738dad2cec2942100` requires spec-first implementation.
- FBCP SOURCE_OF_TRUTH blob `283858bb70bab14e240da781a20e749ed1f1359e`; TDRP SOURCE_OF_TRUTH blob `c181c90ac52043975ef508042f4c20181211d9f5`.
- TDRP Revision 9 spec blob `439c9a3871a31f007cc914227d4ce5aa5fa87a09`; FBCP architecture map blob `77a314c1c982da3aa85bb65165d80468b2663db3`; quality README blob `d24f59cb4fec3cf8bb8b0acf2eb9fa0483ac8cb0`.
- Architecture map names existing canonical shell, conversation, search, permissions, Bot/Agent, Marketplace and platform owners; those are intended owner candidates, **not** proof of full production migration. Quality README requires current-head Actions and independent acceptance; a documented plan is not execution evidence.

## External source acquisition gate
- This execution environment's connected native tool catalog exposes GitHub, but no GitLab original-repository reader. The GitHub connector accepts GitHub hosts, not GitLab project 7705067. Therefore pinned `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` tree, pinned `.gitmodules`, recursive gitlinks, LFS/external objects, patches, generated-input contracts, and all file/blob SHAs remain **unverified**. No values have been inferred or carried forward as completed.

## Decision
- The source baseline is not complete; do not change the upstream lock, inventory, ledger, dossiers, production implementations or acceptance flags on the basis of these partial observations.
- Keep `accepted=false`, `baseline_ready=false` and `status=next`. Do not use historical branch runs or a successful repository read as current-head production acceptance.
- Blockers: enabled workflow state, Actions permissions, PR synchronize delivery record and authenticated original GitLab project tree access. Work that remains independently available: inspect exact-head shipping canonical owners, validator and workflow paths, and CI-test coverage without asserting execution.
