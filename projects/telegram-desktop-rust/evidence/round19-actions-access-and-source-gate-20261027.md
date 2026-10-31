# Round 19 — exact-head Actions access and cppgir source gate (2026-10-27)

## Live authority
- Main HEAD: `cfef45568293a22deb32a6cdef17916f3ec19648` (GitHub REST branches/main).
- PR #178: open, draft, same-repository branch `work/tdrp-revision9-f985723-rebaseline-20261027`, initial exact HEAD `c698a424d7bcd0ccd452544593ad84490e51573b`; branch ref re-read immediately before this evidence commit.
- Root AGENTS.md blob `e6a1f21ba5206cdc807b6f9738dad2cec2942100`; TDRP SOURCE_OF_TRUTH blob `c181c90ac52043975ef508042f4c20181211d9f5`; TDRP Revision 9 spec blob `439c9a3871a31f007cc914227d4ce5aa5fa87a09`; FBCP SOURCE_OF_TRUTH blob `283858bb70bab14e240da781a20e749ed1f1359e`.

## Actions / webhook diagnostic
- GitHub REST `/actions/runs?head_sha=c698a424d7bcd0ccd452544593ad84490e51573b&per_page=100`: `total_count=0`. Commit Actions wrapper returned `workflow_runs=[]`. REST `/commits/c698a424d7bcd0ccd452544593ad84490e51573b/check-runs`: `total_count=0`.
- Branch runs query still reports 22 runs: recent 40515425001 and 40515424774 were failures, attached to older SHA `816050450309044dae2bd6aba79d369077696341`, not the exact HEAD.
- `/issues/178/events?per_page=100` returned `[]`; issue events are not the webhook-delivery log and do not prove failure to deliver synchronize.
- Connector explicitly rejected `/actions/workflows` and `/actions/permissions` as disallowed endpoint families; workflow enabled state and repository Actions permissions remain **unverified**. No webhook delivery-management endpoint is provided by this GitHub connector. Do not infer workflow disabled from absent runs.
- PR workflow `.github/workflows/tdrp-revision9-authority.yml` blob `4b6821dc0ac3db8323b1956748f0f1c5d0535807`: `pull_request` path `projects/telegram-desktop-rust/**` applies to this evidence file. It uses exact PR head SHA in `TDRP_TARGET_SHA`. YAML eligibility alone does not establish scheduling or execution.

## GitLab original authority
- Available connector catalog in this execution environment includes GitHub but no GitLab native repository reader. GitHub fetch is limited to GitHub hosts; the prior round recorded original GitLab project 7705067 tree endpoint as `DisabledError`. No authenticated original-host GitLab tree, pinned `.gitmodules`, recursively resolved gitlinks, LFS, patches, generated inputs, or full content/blob hashes were obtained this round. None may be marked complete.

## Immutable gates
- Exact-head CI acceptance is **missing**; Actions scheduling cause is **unresolved**; source completeness, production implementation, UI/UX, service, Bot regression, packaged E2E and independent release acceptance are **not established**.
- Preserve `accepted=false`, `baseline_ready=false`, `status=next`. No change to upstream lock, inventory, ledger, dossiers, code, or acceptance flags.
- Next action: use a separately authorized GitHub Actions administration and webhook-delivery reader to inspect workflow state, repository Actions permissions and PR synchronize delivery; obtain original GitLab project 7705067 tree via authenticated paginated API and recursively validate object SHAs before changing source records.
