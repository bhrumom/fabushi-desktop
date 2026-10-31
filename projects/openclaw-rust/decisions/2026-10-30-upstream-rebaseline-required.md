# OCRM live upstream rebaseline — 2026-10-30

Status: accepted as a discovery decision; exact-head Actions authority pending  
Project: OCRM-001  
Decision owner: Fabushi Desktop

## Evidence

The read-only live checks for this continuation resolved:

- Fabushi `origin/main`: `cfef45568293a22deb32a6cdef17916f3ec19648`.
- Dedicated worktree subject before local commits: `074d17dcd8508ef6ca45bc5932553e7429cf62f9`, tree `3529384ecb252ef3cdda85816334c82ed20fb3f6`.
- OpenClaw `refs/heads/main`: `87facf07d6b4907846c4a8e4c7ae551f85571420`, tree `f72fbc794bbb23a04bc5afea875d4877048015c9`.
- OCRM PR heads: #169 `b5db35fa272c83833685d70a730b5f8285fa1176`; #180 `2225400ea73392523586a110301edeb87c986001`; #181 `ab4eaa0560a5b767f46d0c91e36eb6e3cfd4344a`. None is canonical `main`.
- GitHub recursive-tree API for live tree `f72fbc794bbb23a04bc5afea875d4877048015c9` returned `truncated=false`, with 63,459 tree entries: 61,221 blobs and 2,238 directories.
- Live root manifest identities are `package.json`=`95a4ad9759adf93239199b0e5f671fb552b6c967`, `pnpm-lock.yaml`=`3d04ffe75adbebb2aa2a536d80acd5ba6e374717`, `pnpm-workspace.yaml`=`c3bddd9f93fec0711312080545cc472fa76f2bc4`, `.gitattributes`=`c6118b226a4a760f7861b6b7b1e7dccd943384e4`, `LICENSE`=`ed064819ab537575d0cad902f7e5af4a3488b1d0`, and `THIRD_PARTY_NOTICES.md`=`0c9c123760188915a384ce7275df4096114e8f90`. The live root has no `.gitmodules`; `.gitattributes` contains no Git-LFS filter rules, but generated/localization paths remain in scope for disposition.

The committed OCRM inventory and generated shards still identify the prior candidate OpenClaw commit/tree `ad8d12f126d29b0db204187adf5092db80294e70` / `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. The lock now records the later live candidate below, but the prior inventory rows contain blob identities for the old tree and therefore cannot be relabeled to the live tree by editing only the lock.

The non-truncated API response is useful discovery evidence for the live tree, but it is not yet the required Actions receipt: it does not close submodules/LFS, package integrity, recursive external dependencies, generated-input provenance, license closure or the subject-target evidence envelope.

## Latest live observation (2026-10-30)

A later read-only `git ls-remote` and GitHub API check resolved OpenClaw
`refs/heads/main` to commit `c49e07385f8d17601efbed7746381213d5326ba9`, tree
`fcd6e0b9b42da1e3d39ca89c69ac779294cc3bfb`, authored at
`2026-10-30T05:56:33Z`. Its recursive-tree response remained
`truncated=false`, with 63,469 entries: 61,231 blobs and 2,238 directories.
The root manifest blob identities remained unchanged from the earlier
observation (`package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`,
`.gitattributes`, `LICENSE`, and `THIRD_PARTY_NOTICES.md`); `.gitmodules` was
still absent. This supersedes the earlier live discovery ref as the latest
candidate only; it does not promote the candidate to lock authority or make
the prior inventory reusable.

A further read-only check resolved OpenClaw `refs/heads/main` to commit
`c32556b0cea7f63b33db0ac393affd71ecb0015b`, tree
`e08d361d83ca9cae55aa93d49462d991d0a39efc`. Its recursive-tree response was
not truncated and contained 63,485 entries: 61,246 blobs and 2,239
directories. The root `package.json` and `pnpm-lock.yaml` blob identities were
`f1a92c3c04328301f0d1b55188846545534ea616` and
`14ec89238970e23718e93f5f6f160e6db217311`; the candidate remains discovery
only pending the exact-subject Actions inventory and dependency-closure run.

The latest current discovery identity is now recorded by
`upstream.lock.json` as commit `2c157ed06dff5032da1093cbb6745efeff2b6f33`,
tree `3fb582f6a1299daa74446ef001cb7cb4fd3c2270`, observed 2026-10-31. It
supersedes the c325 paragraph as a current reference but remains discovery-only;
the c325 paragraph is retained as historical evidence.

## Decision

Keep the prior inventory/shards as historical candidate evidence while the lock
records `2c157ed…` as the current discovery candidate. A fresh GitHub Actions
authority run must recursively enumerate that exact OpenClaw commit, resolve
submodules/LFS and dependency inputs, and produce a new immutable manifest. Do
not claim `baseline_identity_accepted`, `source_closure_ready`, or any
semantic/implementation flag from live discovery alone. The next P0 authority
run must rebaseline the inventory, shards, ledger/RTM provenance and root-input
digests together against one exact subject.

This decision permits read-only source review and implementation work whose source authority is explicitly the prior locked candidate, but any source-dependent acceptance or final receipt must use the new live lock after Actions rebaseline. No CI, push, PR update, or local OCRM executable verification was performed for this decision.
