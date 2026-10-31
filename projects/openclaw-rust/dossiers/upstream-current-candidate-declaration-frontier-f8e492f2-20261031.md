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
