# OCRM live upstream rebaseline — 2026-10-31

Status: accepted as a discovery decision; exact-head Actions authority pending  
Project: OCRM-001  
Decision owner: Fabushi Desktop

## Evidence

Read-only GitHub checks resolved `openclaw/openclaw` `refs/heads/main` to
commit `298546d88934bf05bc7e208d13bf6ce2c538e3f2`, whose root tree is
`120f973ab7da856b4bc1c545c437dd94d35ef9eb`. The recursive tree response was
not truncated and contained 63,847 entries: 61,606 blobs and 2,241 trees.

The exact commit exposes these root input identities:

- `package.json`: `5e468c9625b7670af23096df4588fd5e066c58a8`
- `pnpm-lock.yaml`: `df897f3361f5968912f27b31971a60c08a1132b4`
- `pnpm-workspace.yaml`: `32ce4d83de4aac6a1e3665baba09a3aa9582960c`
- `.gitattributes`: `c6118b226a4a760f7861b6b7b1e7dccd943384e4`
- `LICENSE`: `ed064819ab537575d0cad902f7e5af4a3488b1d0`
- `THIRD_PARTY_NOTICES.md`: `0c9c123760188915a384ce7275df4096114e8f90`
- `.gitmodules`: absent (GitHub contents lookup returned 404)

The package manifest reports OpenClaw version `2026.10.9`; package version and
wire protocol version remain separate contracts and are not acceptance proof.

## Decision

Record this commit/tree as the current discovery candidate in
`upstream.lock.json`. Keep `baseline_identity_accepted=false`,
`source_closure_ready=false`, and all downstream implementation, wiring,
ecosystem, UI and release flags unchanged. The prior inventory/shards and
dependency declarations remain bound to earlier subjects and must not be
relabeled or reused.

A fresh Actions authority run must enumerate this exact subject, resolve
dependencies/LFS/generated inputs/licenses and publish the immutable evidence
envelope before any source-closure or final acceptance claim.

No CI, push, PR update, or local OCRM executable verification was performed for
this decision.
