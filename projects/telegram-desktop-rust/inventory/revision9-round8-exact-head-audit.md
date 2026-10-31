# TDRP Revision 9 — round 8 exact-head authority check (2026-10-27)

This record is evidence of a **partial** read, not accepted baseline authority or production acceptance.

- PR #178 live head before this write: `63c0a9e66b791af06dd36cccfbe101494dc85f8c`, open and draft; main: `cfef45568293a22deb32a6cdef17916f3ec19648`.
- GitHub Actions REST `actions/runs?head_sha=63c0a9e66b791af06dd36cccfbe101494dc85f8c&per_page=100` returned `total_count=0`, and `commits/63c0a9e66b791af06dd36cccfbe101494dc85f8c/check-runs?per_page=100` returned `total_count=0`. This is **missing exact-head validation**, not success.
- Earlier run `40515425001` (TDRP authority) and `40515424774` (Rust desktop runtime) are bound to `816050450309044dae2bd6aba79d369077696341`, both `in_progress` at observation, and **do not validate** the current head.
- `scripts/validate-tdrp-revision9-authority.mjs` still requires `acquisitionInventory.upstream_commit===lock.upstream.commit` even while `lock.upstream.accepted=false`. The historical acquisition record is at `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`; candidate lock at `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Other validator sections also hard-code historical acquisitions, blobs and reachability checks. A complete repair must version/partition candidate and historical evidence rather than bypass checks.
- Candidate `upstream.lock.json` remains `upstream.accepted=false`, `acceptance.baseline_ready=false`. Its recursive source total / unknown minimum / unread minimum remain `null`, not zeros.
- GitLab `mnauw/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` API tree could not be fetched from the current web access (DisabledError); its tree, nested gitlinks, LFS and generated inputs were **not** inspected and must not be inferred from a different repository mirror.
- Grok semantic adaptation hash was updated in the predecessor commit, but **no exact-head GitHub Actions verification** was available here; do not mark it verified.

**Disposition:** source completeness and production acceptance are still open; unknown=0/unread=0/omitted=0 cannot be asserted. Preserve all accepted/baseline-ready flags as false. Continue partitioning candidate acquisition authority and exact GitLab recursive inventory on the next independently verifiable pass.
