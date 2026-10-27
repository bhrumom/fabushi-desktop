# TDRP Revision 9 — round 13 exact-head evidence gap (2026-10-27)

This is a read-only investigation record. It does not certify baseline acceptance or any production capability.

## Current GitHub authority

- Repository main HEAD read from GitHub Git ref API: `cfef45568293a22deb32a6cdef17916f3ec19648`.
- PR #178 head before this evidence commit: `e095c5d028c224ad069217200eef0069b674af46`; branch `work/tdrp-revision9-f985723-rebaseline-20261027`; PR open and draft.
- GitHub Actions REST `actions/runs?head_sha=e095c5d028c224ad069217200eef0069b674af46&per_page=100`: `total_count=0`, `workflow_runs=[]`.
- GitHub Checks REST `commits/e095c5d028c224ad069217200eef0069b674af46/check-runs?per_page=100`: `total_count=0`.
- GitHub combined statuses for `e095c5d028c224ad069217200eef0069b674af46`: empty. Consequently the new historical/candidate identity test and the corrected null-coverage gate have **no exact-head execution evidence**. Do not infer a pass from the workflow YAML or from previous commits.

## Previous-head diagnostic (not transferable)

- Workflow run `40515425001` is on head `816050450309044dae2bd6aba79d369077696341`, **not** the current PR head.
- Candidate-authority job `122228028376`: failed at `Validate Revision 9 source authority` with `build-time acquisition inventory upstream commit drift`. Evidence upload also failed because no artifact files existed.
- Accepted-gate-contract job `122228028382`: failed at `Verify full source-closure mode rejects current open baseline` (grep did not match the validator's error). Neither result establishes failure of the later fixes.
- The exact-head workflow includes `node scripts/test-tdrp-revision9-identity-contract.mjs` and a distinct null-coverage branch for unaccepted candidates; configuration presence is not run evidence.

## External source / GitLab availability

- `mnauw/cppgir` public project identified as GitLab project `7705067`. Its README describes a GObject-Introspection C++ binding generator, a submodule update requirement, and generated outputs.
- Attempted the GitLab exact-commit API tree for `47cf94f83b54cda59018135601e19d7fb0c77776`. The web API read was disabled; the container network attempt failed DNS resolution. **No exact tree entries or blob SHA were obtained.**
- Therefore the project's exact `.gitmodules`, nested gitlinks, LFS objects, patches, generator inputs and recursive completeness remain unverified. Public README information is not immutable evidence for the pinned commit. No lock, inventory, ledger or dossier source entry was accepted from it.

## Release disposition

`accepted=false`, `baseline_ready=false`, `status=next`. Unknown, unread, omitted, source completeness, production implementation and independent release acceptance remain open. This record is diagnostic evidence only.
