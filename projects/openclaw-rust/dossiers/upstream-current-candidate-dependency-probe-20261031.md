# OpenClaw Prior Discovery Candidate — Dependency Input Probe

**Observed at**: 2026-10-31T07:55:14Z  
**Repository**: `openclaw/openclaw`  
**Exact discovery subject**: `77b08d1345a8e5734ada19dc95e77f4c5135bd0a`  
**Tree**: `a52b727858379d14b1b1ed4bde71837326c127f7`  
**Status**: discovery-only; not accepted source authority

**Superseded**: OpenClaw `main` later advanced to
`2910c0de8d7d290f9a82ca388f2893d067c06cb1` / tree
`42dc443420e1b5b84e481f170cd8fccfcc00d7be`. This probe remains valid only for the
exact subject above; its manifest counts and blob identities must not be reused as
closure evidence for the later candidate. A separate compare observation found no
root-input or mapped product-owner path changes, but that narrow fact does not
promote or retarget this probe.

## Observed root dependency inputs

The exact tree's root files were read through the GitHub contents/raw interface and
matched against the recursive tree inventory:

| Input | Blob | Observed fact |
| --- | --- | --- |
| `package.json` | `3b8d23201d08859353b3ada3915f9e6fe5266dcf` | `packageManager` is the pinned `pnpm@12.15.0` integrity-qualified release; 77 runtime dependencies and 57 dev dependencies |
| `pnpm-lock.yaml` | `83b4975eb96942ec007a2eb9847ccfe98d380bda` | exact lockfile input exists in the same tree; full graph closure is not established by this probe |
| `pnpm-workspace.yaml` | `32ce4d83de4aac6a1e3665baba09a3aa9582960c` | workspace patterns are `.`, `ui`, `packages/*`, `extensions/*`, `examples/*`, and `tools/solid-lint` |
| `.gitattributes` | `c6118b226a4a760f7861b6b7b1e7dccd943384e4` | source/LFS policy input present |
| `.gitmodules` | absent | no gitlink input in the recursive tree |

## Evidence boundary

This is a root-input and manifest-shape probe only. It does not enumerate every
workspace manifest, resolved package edge, peer/optional dependency, override,
patch, native build input, artifact digest, or LFS object. It therefore cannot set
`dependency_closure_enumerated`, `lfs_scan_complete`, or `source_closure_ready`,
and it cannot promote the candidate to `baseline_identity_accepted`.

The required closure remains a fresh exact-subject GitHub Actions inventory and
dependency-closure artifact, with immutable provenance and validation against the
same commit/tree. Historical inventory shards and dependency manifests remain
bound to the old implementation baseline and are not reused here.

## References

- `projects/openclaw-rust/upstream.lock.json`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md`
- `projects/openclaw-rust/decisions/2026-10-31-upstream-live-drift.md`
