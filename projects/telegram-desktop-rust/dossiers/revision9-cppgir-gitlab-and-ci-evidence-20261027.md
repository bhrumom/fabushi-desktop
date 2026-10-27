# Revision 9 — cppgir source authority and exact-head CI evidence (2026-10-27)

Status: **OPEN / unaccepted**. Evidence below is intentionally incomplete; it does not update an upstream lock or imply source/production acceptance.

## Source identity

- The currently recorded nested gitlink is `mnauw/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776`, discovered through the cmake_helpers .gitmodules lineage. Its pinned Git tree, blob set, nested gitlinks, .gitmodules contents, patch/LFS/generator inputs and build acquisition set have **not** been recovered in this round.
- The stated GitLab origin is `https://gitlab.com/mnauw/cppgir.git`; its project identity and pinned object contents were not independently verified this round. No inference about its project numeric ID or any nested submodule is accepted.
- The direct GitLab API request `GET https://gitlab.com/api/v4/projects/mnauw%2Fcppgir/repository/tree?ref=47cf94f83b54cda59018135601e19d7fb0c77776&per_page=100&page=1` could not be completed from the available execution environment; DNS resolution failed in the isolated container, and the web-facing API endpoint was disabled. No page counts, tree entries or object hashes are asserted.
- Required recovery: enumerate every GitLab tree page for this exact pin, fetch pinned .gitmodules and .gitattributes and inspect all gitlinks; resolve nested children and exact blob/content hashes; inspect relevant LFS pointers, build-time downloads, patches and generator inputs. Preserve missing/unread/unknown coverage until each check has positive evidence.

## CI evidence scope — do not substitute previous heads

- PR #178 HEAD observed at audit: `0adaf19bae76573094ff805dd60dcfc97a21880a`. Repository main HEAD at audit: `cfef45568293a22deb32a6cdef17916f3ec19648`.
- GitHub Actions branch query showed 22 earlier runs, but the commit-specific query for `0adaf19bae76573094ff805dd60dcfc97a21880a` returned **zero** workflow runs. Hence this current HEAD has **no passing exact-head CI evidence**.
- Historical run `40515425001` at `816050450309044dae2bd6aba79d369077696341`: Source authority job `122228028376` failed with `Error: build-time acquisition inventory upstream commit drift`; the accepted-mode rejection test job `122228028382` also failed because its expected diagnostic was absent after the earlier failure. These logs predate the current validator SHA `0895ff61500d0125d4173f227b0299505e03488d` and **do not establish** that the present fix passed or failed.
- Historical run `40515424774` at the same prior HEAD: renderer job `122228056471` failed `Check Grok architecture inventory`: `frontend/src/production/command-palette-model.ts expected 8f3ac50a132c2b2e050a51ccc911b81dca897b21, found 0022ccac258f8f42f70c559dc933959dbb01c746`. Earlier fixes cannot be accepted without new current-HEAD CI evidence.
- Historical successful root authority shards, child shards, Rust host and other jobs are **not** transferable to this current HEAD without a risk/oracle-based exact-head evidence review.

## Remaining validator concern

The validator at the current SHA uses `historicalCommit` to check legacy acquisition and Qt build inputs. Its `candidateIsAccepted` branch simultaneously requires the same acquisition inventory commit to match the new candidate. This is a future acceptance-transition contradiction: when candidate differs from historical, both equality checks cannot succeed. Keep this **open** until candidate-specific acquisitions, candidate source blob anchors and build reachability have been independently refreshed and the validator cleanly separates historical verification from candidate acceptance. Do **not** weaken historical checks, fabricate candidate data, or turn baseline flags on merely to make CI green.

## Gate

`accepted=false`; `baseline_ready=false`; `status=next`. `unknown=0`, `unread=0`, `omitted=0`, full production implementation, current-HEAD required Actions and independent release ACCEPT remain unproven.
