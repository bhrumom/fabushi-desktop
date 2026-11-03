# OpenClaw Current Discovery Candidate — Dependency Input Probe

**Observed at**: 2026-10-31T17:02:22Z  
**Repository**: `openclaw/openclaw`  
**Exact discovery subject**: `e8dade039bdeb729754d2b5956e6f4cb0a84bf84`  
**Tree**: `2e35087eee94a7c9380ea4ad9fa9c0a6b41e9232`  
**Status**: discovery-only; not accepted source authority

## Observed root dependency inputs

The exact commit's root tree and raw manifest were read through the GitHub
contents/tree interfaces and matched against the same recursive-tree subject:

| Input | Blob | Observed fact |
| --- | --- | --- |
| `package.json` | `3b8d23201d08859353b3ada3915f9e6fe5266dcf` | `packageManager` is integrity-qualified `pnpm@12.15.0`; 77 runtime dependencies and 57 dev dependencies |
| `pnpm-lock.yaml` | `83b4975eb96942ec007a2eb9847ccfe98d380bda` | exact lockfile input exists in the same tree; full graph closure is not established by this probe |
| `pnpm-workspace.yaml` | `32ce4d83de4aac6a1e3665baba09a3aa9582960c` | six workspace patterns: `.`, `ui`, `packages/*`, `extensions/*`, `examples/*`, and `tools/solid-lint` |
| `.gitattributes` | `c6118b226a4a760f7861b6b7b1e7dccd943384e4` | source/LFS policy input present |
| `LICENSE` | `ed064819ab537575d0cad902f7e5af4a3488b1d0` | license input present |
| `THIRD_PARTY_NOTICES.md` | `0c9c123760188915a384ce7275df4096114e8f90` | third-party notice input present |
| `.gitmodules` | absent | no gitlink input in the recursive tree |

## Evidence boundary

This is a root-input and manifest-shape probe only. It does not enumerate every
workspace manifest, resolved package edge, peer/optional dependency, override,
patch, native build input, artifact digest, or LFS object. It therefore cannot
set `dependency_closure_enumerated`, `lfs_scan_complete`, or
`source_closure_ready`, and it cannot promote this candidate to
`baseline_identity_accepted`.

The required closure remains a fresh exact-subject GitHub Actions inventory and
dependency-closure artifact with immutable provenance and validation against the
same commit/tree. Earlier inventory shards and probes remain bound to their
original exact subjects and are not reused here.

## References

- `projects/openclaw-rust/upstream.lock.json`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md`
- `projects/openclaw-rust/decisions/2026-10-31-upstream-live-drift.md`
