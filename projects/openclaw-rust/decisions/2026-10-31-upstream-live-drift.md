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

## Subsequent live recheck — 2026-10-31T03:34:16Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`ece4ca85af0e0ffa7cc619522b9e8b7e40b08da9`, root tree
`25f9aa8f30eef5b41f8873ddc0a7c4a8a0350772`, with committer time
`2026-10-31T03:34:16Z`. The recursive tree response was not truncated and
contained 63,865 entries: 61,624 blobs and 2,241 trees.

Root inputs were re-read for this exact subject:

- `package.json`: `bb155612041150b48dc93246039cb4eac65306ec`
- `pnpm-lock.yaml`: `df897f3361f5968912f27b31971a60c08a1132b4`
- `pnpm-workspace.yaml`: `32ce4d83de4aac6a1e3665baba09a3aa9582960c`
- `.gitattributes`: `c6118b226a4a760f7861b6b7b1e7dccd943384e4`
- `LICENSE`: `ed064819ab537575d0cad902f7e5af4a3488b1d0`
- `THIRD_PARTY_NOTICES.md`: `0c9c123760188915a384ce7275df4096114e8f90`
- `.gitmodules`: absent

This supersedes the `298546d...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.

## Subsequent live recheck — 2026-10-31T03:46:32Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`5ce7a597cbe3f8bd42456cac3b5dbe0e74d9e321`, root tree
`63813e46d6f93598cce22bff02f9def2c098f5ca`, with committer time
`2026-10-31T03:46:32Z`. The recursive tree response was not truncated and
contained 63,872 entries: 61,631 blobs and 2,241 trees.

The exact subject retained these root input identities:

- `package.json`: `bb155612041150b48dc93246039cb4eac65306ec`
- `pnpm-lock.yaml`: `df897f3361f5968912f27b31971a60c08a1132b4`
- `pnpm-workspace.yaml`: `32ce4d83de4aac6a1e3665baba09a3aa9582960c`
- `.gitattributes`: `c6118b226a4a760f7861b6b7b1e7dccd943384e4`
- `LICENSE`: `ed064819ab537575d0cad902f7e5af4a3488b1d0`
- `THIRD_PARTY_NOTICES.md`: `0c9c123760188915a384ce7275df4096114e8f90`
- `.gitmodules`: absent

This supersedes the `ece4ca8...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.

## Subsequent live recheck — 2026-10-31T03:54:32Z

Read-only GitHub commit/tree API checks resolved `openclaw/openclaw`
`refs/heads/main` to commit
`129583af012bf4177b37ba6d88e8b2e5fa0d532f`, root tree
`84860d166bdddef60cbc3ede3909d1a28696b4d9`, with committer time
`2026-10-31T03:54:32Z`. The recursive tree response was not truncated and
contained 63,874 entries: 61,633 blobs and 2,241 trees.

The exact subject retained the same root dependency/license input identities
as the immediately preceding candidate, and `.gitmodules` remains absent.
This supersedes the `5ce7a597...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.

## Subsequent live recheck — 2026-10-31T04:08:46Z

Read-only GitHub commit/tree API checks resolved `openclaw/openclaw`
`refs/heads/main` to commit
`510e1687e965c1a5e2c372f6622704c88448fb5b`, root tree
`8260de46bfdd6ac1a78369ebf2343cb255de20e5`. The recursive tree response was
not truncated and contained 63,877 entries: 61,636 blobs and 2,241 trees.

The exact subject retained the same root dependency/license input identities
as the immediately preceding candidate, and `.gitmodules` remains absent.
This supersedes the `129583af...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.

## Subsequent live recheck — 2026-10-31T04:20:22Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`8e61893b6e692b14d4741ccdf681e1f932289f2f`, root tree
`4e9cca115794d90ad9f056871103790101022764`, with committer time
`2026-10-31T04:20:22Z`. The recursive tree response was not truncated and
contained 63,880 entries: 61,639 blobs and 2,241 trees.

The exact subject retained the same root dependency/license input identities
as the immediately preceding candidate, and `.gitmodules` remains absent.
This supersedes the `510e1687...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.

## Subsequent live recheck — 2026-10-31T04:25:12Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`10d93efc9c8730b7a2fad831c20c649adec7176b`, root tree
`d33ed7c344544baf60148a41afab9e9a9547e9f4`, with committer time
`2026-10-31T04:25:12Z`. The recursive tree response was not truncated and
contained 63,881 entries: 61,640 blobs and 2,241 trees.

The exact subject retained the same root dependency/license input identities
as the immediately preceding candidate, and `.gitmodules` remains absent.
This supersedes the `8e61893b...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.

## Subsequent live recheck — 2026-10-31T04:40:37Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`44069f9c1ab6d1713c2e558dd631fccf8b5adda0`, root tree
`44b33ff34591447276bb9a42916ce23ed029dead`, with committer time
`2026-10-31T04:40:37Z`. The recursive tree response was not truncated and
contained 63,888 entries: 61,647 blobs and 2,241 trees. Root inputs remain
identity-stable; `.gitmodules` remains absent. `src/acp/commands.ts` remains
absent while `src/auto-reply/commands-registry-list.ts` remains present.

This supersedes the `10d93efc...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.
