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

The committed OCRM lock and generated inventory still identify the prior candidate OpenClaw commit/tree `ad8d12f126d29b0db204187adf5092db80294e70` / `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. The prior inventory rows contain blob identities for that tree and therefore cannot be relabeled to the live tree by editing only the lock.

## Decision

Keep the prior lock/inventory as historical candidate evidence until a fresh GitHub Actions authority run recursively enumerates the live OpenClaw commit, resolves submodules/LFS and dependency inputs, and produces a new immutable manifest. Do not claim `baseline_identity_accepted`, `source_closure_ready`, or any semantic/implementation flag from the live discovery alone. The next P0 authority run must rebaseline the lock, inventory, shards, ledger/RTM provenance and root-input digests together against one exact subject.

This decision permits read-only source review and implementation work whose source authority is explicitly the prior locked candidate, but any source-dependent acceptance or final receipt must use the new live lock after Actions rebaseline. No CI, push, PR update, or local OCRM executable verification was performed for this decision.
