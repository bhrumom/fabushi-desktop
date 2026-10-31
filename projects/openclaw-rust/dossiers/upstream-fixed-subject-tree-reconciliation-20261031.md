# OpenClaw Fixed-Subject Tree Reconciliation — 2026-10-31

**Subject**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`

**Expected tree**: `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`

**Evidence source**: GitHub Git Trees API, recursive request for the exact
commit, retrieved 2026-10-31 (Asia/Shanghai). This is direct source evidence,
not an OCRM GitHub Actions receipt.

## Exact tree result

The API returned the expected commit SHA and `truncated=false`:

- recursive entries: `60,926`;
- blobs: `58,750`;
- directories: `2,176`;
- gitlinks: `0`.

The committed canonical inventory contains `58,750` file rows: `58,748` blob
rows and `2` symlink rows, with `omitted_known=0`. A direct sorted comparison
of path, mode, blob identity and entry type matched every path. The only two
reported differences are representation labels for the symlinks: GitHub reports
them as `type=blob` with mode `120000`, while the canonical inventory records
`entry_kind=symlink`. Their paths, modes and blob SHAs are identical:

| path | mode | blob SHA |
| --- | --- | --- |
| `.claude/skills` | `120000` | `2b7a412b8fa0fb7e985b0793321bd4e698f2b6cd` |
| `apps/macos/Tests/OpenClawIPCTests/GatewayTLSStoreFixture.swift` | `120000` | `33019408a314ba5d7e416651727a7ec7ea4cbbb3` |

Root dependency/license inputs are present in the same exact tree. Their blob
SHAs from the tree response are:

| path | blob SHA |
| --- | --- |
| `package.json` | `af543c4c033b5c5e0c466061ada470feeda777f3` |
| `pnpm-lock.yaml` | `80fb4d9fa985297a24ac88931ef534e85a40ba47` |
| `pnpm-workspace.yaml` | `d4ebbf1c13db7e189fe4053122a3e3fa77bb0e26` |
| `.gitattributes` | `c6118b226a4a760f7861b6b7b1e7dccd943384e4` |
| `LICENSE` | `ed064819ab537575d0cad902f7e5af4a3488b1d0` |
| `THIRD_PARTY_NOTICES.md` | `0c9c123760188915a384ce7275df4096114e8f90` |

`.gitmodules` is absent, consistent with zero gitlinks.

## Root dependency-input probe

The same exact commit's raw root inputs were fetched and verified against the
tree blob identities with Git's blob hash algorithm. `package.json` resolves to
the tree SHA above and declares `pnpm@12.15.0` with its integrity-qualified
package-manager string, `75` runtime dependencies and `58` dev dependencies.
The recursive tree contains `209` `package.json` paths. `pnpm-lock.yaml` also
hashes to its exact tree SHA and begins with lockfile version `9.0`, the pnpm
12.15.0 package-manager dependency and importer/package/snapshot sections. The
lock input is therefore present and identity-bound; a full workspace importer,
peer/optional dependency, patch, native-build and external-package closure has
not yet been independently materialized.

The exact root-input identities are:

| input | Git blob SHA |
| --- | --- |
| `package.json` | `af543c4c033b5c5e0c466061ada470feeda777f3` |
| `pnpm-lock.yaml` | `80fb4d9fa985297a24ac88931ef534e85a40ba47` |

## Evidence boundary

This proves the fixed subject's recursive path/blob/mode inventory is complete
against the direct API response and that the committed inventory's source
identity is consistent. It does not prove recursive package dependency closure,
LFS content closure, semantic reads, applicability/disposition closure, or any
Actions-backed implementation or acceptance claim. The existing P0 authority
flags therefore remain false; this dossier does not promote
`baseline_identity_accepted` or `source_closure_ready` by itself.

## Reproduction

```text
GET https://api.github.com/repos/openclaw/openclaw/git/trees/ad8d12f126d29b0db204187adf5092db80294e70?recursive=1
```

Local comparison inputs were the response body and all committed
`projects/openclaw-rust/inventory/shards/part-*.jsonl.gz` rows. No product build,
test, OCRM workflow, push, or PR-head update was performed.
