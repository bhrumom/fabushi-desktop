# OpenClaw current candidate declaration frontier — f8e492f2 (2026-10-31)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Exact discovery subject: `f8e492f2c9d1cc3768c222b909a82bd3e4f97455`
- Root tree: `89257a1e739896c29fbc427e43e2c59d95ab02a1`
- Observed: `2026-10-31T12:42:39Z`
- Discovery source: read-only GitHub tree and root-tree APIs

## Exact declaration inventory

The untruncated recursive tree contains 64,090 entries: 61,849 blobs, 2,241
directories, and 0 gitlinks. Applying the existing declaration-input families
to this exact tree found 245 declaration files:

| Declaration family | Count |
| --- | ---: |
| `package.json` | 214 |
| `package-lock.json` | 3 |
| `pnpm-lock.yaml` | 1 |
| `pnpm-workspace.yaml` | 1 |
| `Cargo.toml` | 5 |
| `Cargo.lock` | 3 |
| `Package.swift` | 6 |
| `Package.resolved` | 2 |
| `Gemfile` | 2 |
| `Gemfile.lock` | 2 |
| `build.gradle.kts` | 5 |
| `gradle.properties` | 1 |
| **Total** | **245** |

The exact root dependency inputs retain these blob identities:

| Path | Blob |
| --- | --- |
| `package.json` | `e52ea501bd0e2d749e7f4ab2efce5074b81a2a78` |
| `pnpm-lock.yaml` | `83b4975eb96942ec007a2eb9847ccfe98d380bda` |
| `pnpm-workspace.yaml` | `32ce4d83de4aac6a1e3665baba09a3aa9582960c` |

The root declaration identities are equal to the prior candidate's identities;
this is a positive same-blob fact, not a claim that the full dependency graph
or external artifacts are closed for this subject.

## Patch and native/platform input frontier

The exact tree contains 17 patch files under `patches/`, totaling 604,457
bytes. Every patch path enumerated by the exact tree is present:

| Patch path | Git blob | Bytes |
| --- | --- | ---: |
| `patches/@clawdbot__lobster@2026.10.1.patch` | `1426f66106f66ad1059bd380b582ae50c04bd2f6` | 44,060 |
| `patches/@homebridge__ciao@1.3.12.patch` | `b1bdfe22f24c0b7b0a4ec4044cb8dffe93e34138` | 295,140 |
| `patches/@novnc__novnc@1.7.0.patch` | `5be7150a8f19a39a9a47575b9dbd1785adfb67db` | 844 |
| `patches/@openclaw__crabline@0.1.31.patch` | `dba776eb88d4a9fb62c69037af0515a671990e69` | 31,643 |
| `patches/@shinyoshiaki__binary-data@0.6.1.patch` | `c7b57f77561e95b8bc6d43dc8b935ed3cbd5f5ba` | 10,319 |
| `patches/@solidjs__signals@2.0.0-rc.18.patch` | `6af0093af08ecbaf65f01bf19ada60ee48e056e4` | 8,860 |
| `patches/@solidjs__web@2.0.0-rc.18.patch` | `53f8271ace8c671939b9ab0f855ce9eac6a26199` | 4,218 |
| `patches/apache-arrow@21.2.0.patch` | `b57c4ef1e9bf981999e0b414038afd69709f707b` | 14,921 |
| `patches/baileys@7.0.0-rc14.patch` | `a64bf48b0d33c6eab333b870ba2e28c2f6c75fd3` | 3,461 |
| `patches/chrome-devtools-mcp@1.12.0.patch` | `6991f42bc49c7e8ccee8e808cee665941cee640b` | 26,954 |
| `patches/ghostty-web@0.4.0.patch` | `a051635259716ce174bc0fc508c13877dd36a3e1` | 4,294 |
| `patches/matrix-js-sdk@43.0.0.patch` | `a309246e52c34f76482d3dc7d8e405eaf90a565f` | 8,840 |
| `patches/minizlib@3.1.0.patch` | `9528e55542d3471ebfd8869dc7ce090e4109f4a5` | 100,124 |
| `patches/playwright-core@1.64.1.patch` | `f4517bdde34276b9f5e2e4353b319e82c87e58a3` | 2,282 |
| `patches/proper-lockfile@4.1.2.patch` | `62060ba6e5cae8abf961935f0d57a0404187fa4d` | 10,162 |
| `patches/tar@7.5.22.patch` | `c974014f227a831850225d36310ef19601ce0eaa` | 1,254 |
| `patches/vitest@5.0.5.patch` | `b77fe0e3e8657e7d5a905769a7cf67b03f71c691` | 37,081 |

The exact tree also contains 26 native/platform declaration inputs across
`Cargo.toml`, `Cargo.lock`, `Package.swift`, `Package.resolved`, `Gemfile`,
`Gemfile.lock`, `build.gradle.kts`, and `gradle.properties`. These counts and
blob identities prove path presence for this subject only; they do not prove
patch application, registry archive bytes, configured build graphs, native
artifacts, optional-platform resolution, or license/SBOM closure.

## Closure boundary

This exact-tree count does not establish recursive relative-import or
workspace-package closure, resolved registry/native/optional artifacts,
patch application, license/NOTICE/SBOM completeness, LFS objects, or an
exact-subject Actions inventory/dependency-closure receipt. It therefore does
not change `dependency_closure_enumerated`, `source_closure_ready`, or any
authority/completion flag. The fixed implementation subject and its generated
inventory remain bound to `ad8d12f126d29b0db204187adf5092db80294e70` /
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16` and were not relabeled.

## Reference

- `projects/openclaw-rust/dossiers/upstream-current-candidate-probe-f8e492f2-20261031.md`
- `projects/openclaw-rust/dossiers/upstream-current-candidate-dependency-frontier-8c69e8b7-20261031.md`
