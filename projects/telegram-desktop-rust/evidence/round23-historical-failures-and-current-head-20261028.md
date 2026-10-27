# Round 23 — exact-head evidence and historical failed-job classification (2026-10-28)

## Re-read immutable and live authority
- GitHub refs/heads/main: `cfef45568293a22deb32a6cdef17916f3ec19648`.
- PR #178 open, draft, unmerged; live head at read: `fc3509dae8a51f854dbd173a6f50aecc4a5119cc`; branch `work/tdrp-revision9-f985723-rebaseline-20261027`. PR metadata base_sha is not current main.
- Root AGENTS.md blob e6a1f21ba5206cdc807b6f9738dad2cec2942100, FBCP SOT blob 283858bb70bab14e240da781a20e749ed1f1359e, TDRP SOT blob c181c90ac52043975ef508042f4c20181211d9f5 reviewed. TDRP is Revision 9 and is not accepted.

## Exact inspected head before this evidence commit
- `GET /actions/runs?head_sha=fc3509dae8a51f854dbd173a6f50aecc4a5119cc&per_page=100`: total_count=0.
- `GET /commits/fc3509dae8a51f854dbd173a6f50aecc4a5119cc/check-runs?per_page=100`: total_count=0.
- `GET /commits/fc3509dae8a51f854dbd173a6f50aecc4a5119cc/status`: pending, statuses=[], total_count=0. No current-head CI success is evidenced.

## Historical Actions — do NOT transfer to current head
- Both runs tested `816050450309044dae2bd6aba79d369077696341`, not `fc3509dae8a51f854dbd173a6f50aecc4a5119cc`.
- TDRP Revision 9 run 40515425001: 61 jobs, failures:
  - job 122228028376 "Source authority / recursive inventory": validator throws `Error: build-time acquisition inventory upstream commit drift` at `scripts/validate-tdrp-revision9-authority.mjs:96`; its evidence upload also fails because no files exist in artifacts/tdrp-authority.
  - job 122228028382 "Accepted gate is executable and fail-closed": shell expected a rejected accepted mode to contain `accepted mode requires|G-INVENTORY`, but the grep failed (exit 1); this is distinct from evidence of a correctly working accepted gate.
- Rust desktop runtime run 40515424774: six jobs, one failed:
  - job 122228056471 "renderer": `scripts/check-grok-architecture.mjs` reports `frontend/src/production/command-palette-model.ts` semantic adaptation hash drift, expected `8f3ac50a132c2b2e050a51ccc911b81dca897b21`, observed `0022ccac258f8f42f70c559dc933959dbb01c746`. Renderer earlier recorded 57 passing tests; the architecture check failed later.
- These are historical diagnostics, not assertions that the current validator/renderer still fails.

## Workflow and shipping boundary
- `.github/workflows/tdrp-revision9-authority.yml` blob 4b6821dc0ac3db8323b1956748f0f1c5d0535807 includes `pull_request.paths: projects/telegram-desktop-rust/**` and exact checkout assertions.
- `.github/workflows/rust-desktop-runtime.yml` blob 620719ca0bf63cc040bbdf28372fe4a8062400d5 includes `pull_request.paths: projects/telegram-desktop-rust/**`, exact `github.event.pull_request.head.sha` binding, and frontend/source/desktop paths.
- `desktop/scripts/check-agent-workspace-boundary.mjs` blob a2ddb992fa459fd2328c924f9e39840a70f2f3d5 asserts renderer -> Electron -> independent Coordinator -> Host -> Runner boundaries. This is static contract inspection, NOT packaged production E2E evidence.
- `scripts/validate-tdrp-revision9-authority.mjs` blob c972812cfd5fe5fea8ab89556e0ded3968f24590 separates historical acquisition identity from unaccepted candidate identity. Current execution on exact head has not been observed.

## Permissions and unresolved source closure
- GitHub connector fetch explicitly denied `/actions/workflows`, `/actions/permissions`, `/hooks`; workflow-enabled, repository Actions permissions and PR synchronize webhook delivery are UNVERIFIED. Static path eligibility alone does not prove event delivery.
- The available GitHub connector cannot access raw GitLab project 7705067; `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` complete tree, .gitmodules, recursive gitlinks, LFS, patches, generator inputs, blobs are UNVERIFIED. No candidate source lock/inventory/ledger/dossier values were advanced.
- Remain fail-closed: `accepted=false`, `baseline_ready=false`, `status=next`. No merge or release. All eight final gates, particularly exact-head CI and independent release ACCEPT, remain unproven.
