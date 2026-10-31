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

## Subsequent live recheck — 2026-10-31T05:14:48Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`b924498923dff81a02fb8a89d57828717c6e160d`, root tree
`af9b32fcb093badf0e05912411f5acfa65698e79`. The recursive tree response was
not truncated and contained 63,902 entries: 61,661 blobs and 2,241 trees.
Root inputs remain identity-stable and `.gitmodules` remains absent; the ACP
commands path remains absent while the auto-reply command registry list remains
present.

This supersedes the `622fa3fab...` discovery candidate in the lock. It remains
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

## Subsequent live recheck — 2026-10-31T05:09:05Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`622fa3fab8e154819a329d3c288d13491facea7b`, root tree
`2590235af3db54166cf88801806e2b63b3f4688b`. The recursive tree response was
not truncated and contained 63,901 entries: 61,660 blobs and 2,241 trees.
Root inputs remain identity-stable and `.gitmodules` remains absent.

This supersedes the `44069f9c...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.

## Subsequent live recheck — 2026-10-31T05:23:17Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`b513040e660678693cee8d6131f35b206f0df833`, root tree
`c48aeaf4c192e112f1ed38b4d693e13bd1f1d222`. The recursive tree response was
not truncated and contained 63,906 entries: 61,665 blobs and 2,241 trees; no
gitlinks were present. Root inputs remain identity-stable and `.gitmodules`
remains absent.

This supersedes the `b9244989...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.

The read-only identity audit also confirms that `inventory/index.json` and all
58,750 inventory shard rows remain bound to the implementation baseline
`ad8d12f126d29b0db204187adf5092db80294e70` / tree
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`; the parity ledger and RTM carry the
same baseline identity. They are not relabeled to this discovery subject and
cannot serve as current source-closure evidence until a fresh exact-subject
Actions inventory, dependency closure and receipt exist.

The same audit found six ledger responsibilities absent from the old inventory
`responsibility_ids` projection: four declared Crabbox P1 task-local rows and
the two mapped non-P1 rows `OCRM-RESP-NODE-HOST-LOCAL-RUNTIME-PROJECTION-001`
and `OCRM-RESP-SKILLS-CONFIG-MUTATION-001`. The referenced paths and all
ledger/RTM IDs are otherwise present; this is projection staleness, not a
dangling reference. Because the shards carry canonical compressed and
uncompressed digests, the old inventory is not edited by hand. The new
exact-subject inventory generation must reconcile these rows and prove the
result independently.

## Subsequent live recheck — 2026-10-31T05:28:35Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`42c05e2b352ab9efbd3f06267cbd396589e02c52`, root tree
`574e2fa28cb932349a192f36309557d6c17f4c7e`. The recursive tree response was
not truncated and contained 63,911 entries: 61,670 blobs and 2,241 trees; no
gitlinks were present. Root inputs remain identity-stable and `.gitmodules`
remains absent.

This supersedes the `b513040e...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.

## Subsequent live recheck — 2026-10-31T05:37:42Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`002f25a97fccb7eff05b768a3fd7bc7e13488ed6`, root tree
`553796fe9bd2b9298f6d843c63cf9a4d588f5b71`. The recursive tree response was
not truncated and contained 63,922 entries: 61,681 blobs and 2,241 trees; no
gitlinks were present. `.gitmodules` remains absent; `package.json` and
`pnpm-lock.yaml` changed blob identity from the preceding candidate while the
other declared root inputs remained stable.

This supersedes the `42c05e2...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.

## Subsequent live recheck — 2026-10-31T05:45:33Z

Read-only GitHub ref and commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`f92366d3c7fb390dc9bba3b89e06b794e0cb5119`, root tree
`260ace762cbdaf2e25a8335319601b199d6848e0`. The recursive tree response was
not truncated and contained 63,946 entries: 61,705 blobs and 2,241 trees; no
gitlinks were present. Root dependency/license input identities remained
stable and `.gitmodules` remained absent.

This superseded the `002f25a...` discovery candidate in the lock. It remained
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance was relabeled or reused, and all authority/completion flags
remained unchanged. No CI, push, PR update or local OCRM executable
verification was performed.

## Subsequent live recheck — 2026-10-31T05:51:22Z

Read-only GitHub ref and commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`bcf23b2ea0eee92c8c497c6b1c4d486588e4d5ab`, root tree
`5d9b96f3208eb2daf4c566fc60865a1850e4c77f`. The recursive tree response was
not truncated and contained 63,946 entries: 61,705 blobs and 2,241 trees; no
gitlinks were present. Root dependency/license input identities remained
stable and `.gitmodules` remained absent.

This superseded the `f92366d...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result or
Actions provenance is relabeled or reused, and all authority/completion flags
remain unchanged. No CI, push, PR update or local OCRM executable verification
was performed.

## Subsequent live recheck — 2026-10-31T08:07:28Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`db59290c33e1768cce8a461e2697cff7108c5a69`, root tree
`c1e4818f8f877576c02da1573e7dc809d1fdd800`. The recursive tree response was
not truncated and contained 63,993 entries: 61,752 blobs and 2,241 trees; no
gitlinks were present. Root dependency/license input identities remained
stable and `.gitmodules` remained absent.

The exact compare from `2910c0de8d7d290f9a82ca388f2893d067c06cb1` to this
subject reported two commits ahead and eight changed files, limited to
UI/Cron-windowing, gateway-network, and their test/support paths. This is a
discovery refresh only: no prior inventory, dependency closure, LFS/integrity
result or Actions provenance was relabeled or reused, and all authority and
completion flags remain unchanged. No CI, push, PR update or local OCRM
executable verification was performed.

## Subsequent live recheck — 2026-10-31T08:09:58Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`1b870b2c78e81fded72913ecba732b84b611ca12`, root tree
`199fe97259d0226ce0e4d3dcec603252376c9205`. The recursive tree response was
not truncated and contained 63,994 entries: 61,753 blobs and 2,241 trees; no
gitlinks were present. Root dependency/license input identities remained
stable and `.gitmodules` remained absent.

The exact compare from `db59290c33e1768cce8a461e2697cff7108c5a69` to this
subject reported two commits ahead and seven changed files. It includes mapped
session/cron lifecycle paths (`src/config/sessions/lifecycle.ts`,
`src/cron/isolated-agent/session-cleanup.ts`, and subagent cleanup) plus PDF
provider/test support. Existing responsibility evidence for those paths must be
re-read against this subject. No prior inventory, dependency closure,
LFS/integrity result or Actions provenance was relabeled or reused; authority
and completion flags remain unchanged. No CI, push, PR update or local OCRM
executable verification was performed.

The three changed session/cron runtime paths are recorded in
`dossiers/upstream-current-session-lifecycle-delta-20261031.md`. That dossier is
discovery evidence only; it does not alter the old-subject ledger or establish
Rust production wiring.

## Subsequent live recheck — 2026-10-31T08:26:10Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`1d2e685b76b2bb7da964a2d6445cec2e07bf5883`, root tree
`a7f33a87af80c203422677be70f83aebb0857666`. The commit is an upstream Matrix
durable-final-reply recovery change; this is an upstream drift fact only, not a
session lifecycle acceptance fact.

The recursive tree response was not truncated and contained 63,996 entries:
61,755 blobs and 2,241 trees, with no gitlinks. Root package/dependency/license
inputs remain identity-stable. This supersedes the `1b870b2c...` discovery
candidate in the lock. It remains discovery-only: no prior inventory,
dependency closure, LFS result or Actions provenance was relabeled or reused;
all authority and completion flags remain unchanged. No CI, push, PR update or
  local OCRM executable verification was performed.

## Subsequent live recheck — 2026-10-31T09:47:18Z

Read-only GitHub ref and commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`2464bd85e0bc2927701c3147bf3dfc6993daaae3`, root tree
`08b1e6627c310e8640a340be49838e2936b44346`, with commit message
`fix(test): bind SDK hosts in direct Gateway fixtures (#189244)`.
The recursive tree was not truncated and contained 64,009 entries: 61,768
blobs, 2,241 directories, and no gitlinks. Root dependency/license inputs are
unchanged from the preceding candidate.

This supersedes `d27a8c9d...` as the live discovery candidate in
`upstream.lock.json`. It remains discovery-only: no inventory, dependency
closure, LFS/integrity result, Actions provenance, or source-semantic
conclusion was relabeled or reused. All authority and completion flags remain
unchanged. The fixed `ad8d12f...` implementation subject remains unchanged.

## Subsequent live recheck — 2026-10-31T08:42:15Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`54a2fe137a70a5d2c0c0187ba27294c5ba74d3a0`, root tree
`4e8c368864db959f8a188bc23333a6d24dd221a0`. The recursive tree response was
not truncated and contained 63,998 entries: 61,757 blobs and 2,241 trees, with
no gitlinks. Compared with the preceding `1d2e685...` candidate, the ref is
ahead by seven commits and has 30 changed files. Root package/dependency/license
inputs remain identity-stable.

This supersedes the `1d2e685...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS result or Actions
provenance was relabeled or reused; all authority and completion flags remain
unchanged. No CI, push, PR update or local OCRM executable verification was
performed.

The four lifecycle paths were also re-read by exact blob identity at this
subject: `src/cron/isolated-agent/run.ts`
(`617257faccf4b8cb7d696e0a8890d77916e83fa5`),
`src/cron/isolated-agent/session-cleanup.ts`
(`5cdeb385daf097dd9d7b87febdc7178dd4a91050`),
`src/agents/subagents/registry/subagent-session-cleanup.ts`
(`1669914996a02fcecd45012811d0a42f18bd32d7`), and
`src/config/sessions/lifecycle.ts`
(`7e3c7a6999cb44417ec634b0e77d1299c672f8c1`). Their behavior evidence may be
carried forward as same-blob evidence only; no broader inventory or dependency
closure is implied.

## Subsequent live recheck — 2026-10-31T16:58:00Z

Read-only `git ls-remote` and GitHub commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`9f3da25c107e88380e84f8a70f8f703422ba5995`, root tree
`5957adf997a86b4937f6b7c162ebccb116e0aafa`, committed at
`2026-10-31T08:57:14Z`. The recursive tree was not truncated and contained
63,998 entries: 61,757 blobs, 2,241 directories, and no gitlinks. Root
dependency/license identities remain unchanged; `.gitmodules` remains absent.

This supersedes the `54a2fe137...` discovery candidate in the lock. It remains
discovery-only: no earlier inventory, dependency closure, LFS/integrity result,
Actions provenance, or source-semantic conclusion is relabeled or reused. All
authority and completion flags remain unchanged. No CI, push, PR update, or
local OCRM executable verification was performed.

## Subsequent live recheck — 2026-10-31T17:02:22Z

Read-only `git ls-remote` and GitHub tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`e8dade039bdeb729754d2b5956e6f4cb0a84bf84`, root tree
`2e35087eee94a7c9380ea4ad9fa9c0a6b41e9232`, committed at
`2026-10-31T08:58:55Z`. The recursive tree was not truncated and contained
63,999 entries: 61,758 blobs, 2,241 directories, and no gitlinks. The root
dependency/license input blobs remain unchanged.

This supersedes the `9f3da25c...` discovery candidate in the lock. It remains
discovery-only: no earlier inventory, dependency closure, LFS/integrity result,
Actions provenance, or source-semantic conclusion is relabeled or reused. All
authority and completion flags remain unchanged. No CI, push, PR update, or
local OCRM executable verification was performed.

## Subsequent live recheck — 2026-10-31T09:14:47Z

Read-only GitHub commit/tree API checks resolved `openclaw/openclaw`
`refs/heads/main` to commit
`e0f1e0a6af388d18fcfcda210f39f5106c7f7c10`, root tree
`63d39e4b31fbb462aab29ff0f8b728597bb3ca20`, committed at
`2026-10-31T09:13:56Z`. The recursive tree was not truncated and contained
64,003 entries: 61,762 blobs, 2,241 directories, and no gitlinks. Root
dependency/license input blobs remain unchanged; the compare from `e8dade03...`
is 15 commits ahead with 62 returned changed files.

This supersedes the `e8dade03...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result,
Actions provenance, or source-semantic conclusion is relabeled or reused. All
authority and completion flags remain unchanged. No CI, push, PR update, or
local OCRM executable verification was performed.

## Subsequent live recheck — 2026-10-31T12:33:09Z

Read-only GitHub branches/commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`be8eca7f92b2d7c9acb4043debef785906af6188`, root tree
`6b24eed3a7d79dcb099ad4c17aee7f85c0ea3842`. The recursive tree was not
truncated and contained 64,088 entries: 61,847 blobs, 2,241 directories, and
no gitlinks. The commit message is `fix(ci): remove leftover internal exports
(#189345)`; root dependency/license input blobs remain unchanged.

This supersedes the `8c69e8b7...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result,
Actions provenance, or source-semantic conclusion is relabeled or reused. All
authority and completion flags remain unchanged. No CI, push, PR update, local
OCRM executable verification, build, package, release, or runtime/device
acceptance was performed.

## Subsequent live recheck — 2026-10-31T12:53:51Z

Read-only GitHub branches/commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`eb0183deb6891f07686a1f3cefeaf063c119aaa7`, root tree
`6c66d83fbeba3e74a7142073a2980bd2433722a8`. The recursive tree was not
truncated and contained 64,097 entries: 61,856 blobs, 2,241 directories, and
no gitlinks. The commit message is `chore: speed up state lease and preservation
tests (#189366)`; its GitHub commit verification is valid and root
dependency/license input blobs remain unchanged.

This supersedes the `f8e492f2...` discovery candidate in the lock. Its exact
diff is limited to two state test files, so it is not production Rust
implementation or wiring evidence. It remains discovery-only: no prior
inventory, dependency closure, SBOM, LFS/integrity result, notice audit,
Actions provenance, or source-semantic conclusion is relabeled or reused. All
authority and completion flags remain unchanged. No CI, push, PR update, local
OCRM executable verification, build, package, release, or runtime/device
acceptance was performed.

## Subsequent live recheck — 2026-10-31T12:42:39Z

Read-only GitHub branches/commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`f8e492f2c9d1cc3768c222b909a82bd3e4f97455`, root tree
`89257a1e739896c29fbc427e43e2c59d95ab02a1`. The recursive tree was not
truncated and contained 64,090 entries: 61,849 blobs, 2,241 directories, and
no gitlinks. The commit message is `test(ui): expose reconnect service worker
policy (#189374)`; its GitHub commit verification is valid and root
dependency/license input blobs remain unchanged.

This supersedes the `ed19b276...` discovery candidate in the lock. Its exact
diff is limited to `scripts/README.md` and
`scripts/bench-control-ui-reconnect.ts`, so it is not production Rust
implementation or wiring evidence. It remains discovery-only: no prior
inventory, dependency closure, LFS/integrity result, Actions provenance, or
source-semantic conclusion is relabeled or reused. All authority and
completion flags remain unchanged. No CI, push, PR update, local OCRM
executable verification, build, package, release, or runtime/device
acceptance was performed.

## Subsequent live recheck — 2026-10-31T12:40:59Z

Read-only GitHub branches/commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`ed19b276e1630a14c66ddf8cc4594d2915ba8c56`, root tree
`0f29f49072123eae942faa152907fbbebaab103a`. The recursive tree was not
truncated and contained 64,090 entries: 61,849 blobs, 2,241 directories, and
no gitlinks. The commit message is `perf(test): build stock Bun image fixtures
without ZIP subprocesses (#189359)`; its GitHub commit verification is valid
and root dependency/license input blobs remain unchanged.

This supersedes the `dfd49043...` discovery candidate in the lock. Its exact
diff is limited to `test/scripts/setup-node-env-bun.test.ts`, so it is not
production Rust implementation or wiring evidence. It remains discovery-only:
no prior inventory, dependency closure, LFS/integrity result, Actions
provenance, or source-semantic conclusion is relabeled or reused. All
authority and completion flags remain unchanged. No CI, push, PR update, local
OCRM executable verification, build, package, release, or runtime/device
acceptance was performed.

## Subsequent live recheck — 2026-10-31T12:36:43Z

Read-only GitHub branches/commit/tree API checks resolved
`openclaw/openclaw` `refs/heads/main` to commit
`dfd4904312e87674d468d89232180973859e26e0`, root tree
`9cf481d8e97f4428d9d9e2f80f941c9cbb7a410d`. The recursive tree was not
truncated and contained 64,090 entries: 61,849 blobs, 2,241 directories, and
no gitlinks. The commit message is `fix: restore spawn boundary SDK resource
lifecycle (#189329)`; its GitHub commit verification is valid and root
dependency/license input blobs remain unchanged.

This supersedes the `be8eca7f...` discovery candidate in the lock. It remains
discovery-only: no prior inventory, dependency closure, LFS/integrity result,
Actions provenance, or source-semantic conclusion is relabeled or reused. All
authority and completion flags remain unchanged. No CI, push, PR update, local
OCRM executable verification, build, package, release, or runtime/device
acceptance was performed.
