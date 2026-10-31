# OpenClaw current candidate dependency frontier — 2026-10-31

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Exact discovery subject: `8c69e8b71d3be0a8543e3480161e8e01f4fd9e5c`
- Root tree: `f4e34cf339e1645079baa9c8a6bdf1b9a169fa4f`
- Observed: `2026-10-31T12:18:40Z`
- Discovery source: read-only GitHub tree/raw APIs

## Exact declaration inventory

The untruncated recursive tree contains 64,075 entries: 61,834 blobs, 2,241
directories, and 0 gitlinks. Applying the repository's declaration-input
families to that exact tree found 246 declaration files:

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
| `gradle.properties` | 2 |
| **Total** | **246** |

The exact root manifest is OpenClaw `2026.10.9`, with integrity-qualified
`pnpm@12.15.0`, 77 runtime dependencies, 57 dev dependencies, 1 optional
dependency, and 0 peer dependencies. The exact workspace declaration contains
`.`, `ui`, `packages/*`, `extensions/*`, `examples/*`, and `tools/solid-lint`.

The exact `pnpm-lock.yaml` blob is
`83b4975eb96942ec007a2eb9847ccfe98d380bda`. A YAML-stream read preserves two
documents in that one blob:

| Document | Importers | Packages | Snapshots | Interpretation |
| --- | ---: | ---: | ---: | --- |
| 1 | 1 | 15 | 15 | pnpm bootstrap/package-manager self-lock |
| 2 | 206 | 1,749 | 1,750 | workspace dependency lock |

The second document's 206 importers cover the root, `ui`, `packages/*`,
`extensions/*`, `examples/*`, and `tools/solid-lint` workspace members observed
in the exact tree. Counting only the first YAML document would therefore
under-report the workspace graph. This is a structural lock observation, not a
claim that every registry archive, patch, native artifact, optional platform
package, license or LFS object has been externally verified.

## Manifest-to-importer crosswalk

The exact tree contains 215 `package.json` paths. Normalizing the root manifest
to importer `.` and other manifests to their containing directory yields:

| Crosswalk result | Count |
| --- | ---: |
| exact-tree `package.json` paths | 215 |
| second-document workspace importers | 206 |
| manifests with a matching importer | 206 |
| importers without a matching manifest | 0 |
| manifests without a matching importer | 9 |

The nine manifest paths without a workspace importer are:

- `.github/actions/setup-security-review/package.json`
- `.github/release/clawhub-cli/package.json`
- `.github/release/vercel-cli/package.json`
- `docs/snippets/plugin-publish/minimal-package.json`
- `qa/convex-credential-broker/package.json`
- `scripts/cloudflare/package.json`
- `src/templates/fixtures/delegating-agent/package.json`
- `test/fixtures/templates/project-v1/package.json`
- `test/fixtures/web-output-provider/package.json`

This is a path-level workspace crosswalk, not a declaration that those nine
paths are irrelevant: their scripts, release tooling, fixtures and QA/build
inputs still require explicit applicability and disposition review. Conversely,
the zero unmatched-importer result does not prove that resolved package edges,
patches, platform artifacts or dynamic imports are closed.

## Patch, native and LFS-input frontier

The second lock document declares 44 overrides and 17 patched dependencies.
All 17 corresponding `.patch` files are present under the exact tree's
`patches/` prefix, with 604,457 total bytes; the lock-to-tree path existence
check therefore has no missing patch path. This does not yet prove patch
content semantics, patch application, registry archive integrity or the
license/provenance of the patched artifacts.

At `2026-10-31T12:26:00Z`, each of those 17 patch files was fetched from the
exact commit and hashed as a Git blob. All 17 fetched blob identities matched
the recursive-tree SHA (`patch_blob_checks=17`, `mismatches=0`). This closes
the patch path/content-identity check for the exact subject only; it does not
close application semantics or the external package bytes being patched.

The exact tree contains 26 native/platform declaration inputs across
`Cargo.toml`, `Cargo.lock`, `Package.swift`, `Package.resolved`, `Gemfile`,
`Gemfile.lock`, `build.gradle.kts`, and `gradle.properties`, totaling 323,239
bytes. These inputs remain separate closure edges; a path inventory does not
prove a configured build graph or downloaded native artifact set.

The exact `.gitattributes` blob is
`c6118b226a4a760f7861b6b7b1e7dccd943384e4` and contains text/generated-file
rules but no `filter=lfs` rule. That is useful negative evidence about the
declared root policy, not proof that no LFS pointer or externally fetched asset
exists; LFS/content closure therefore remains an independent unresolved gate.

## Drift against the fixed-baseline declaration input

The committed `dependencies/source-manifest-inputs.json` remains deliberately
bound to the fixed implementation subject `ad8d12f126d29b0db204187adf5092db80294e70`
and tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`; it must not be relabeled
to this discovery candidate. Its 244 declaration entries compare with the
current candidate as follows:

| Comparison | Count |
| --- | ---: |
| baseline entries | 244 |
| current exact-tree declaration entries | 246 |
| baseline paths absent from current tree | 6 |
| new current-tree paths | 8 |
| shared paths with changed blob identity | 208 |
| shared paths with unchanged blob identity | 30 |

Absent baseline paths are:

- `apps/android/settings.gradle.kts`
- `scripts/docs-i18n/go.mod`
- `scripts/docs-i18n/go.sum`
- `skills/pyproject.toml`
- `src/claws/fixtures/delegating-agent/package.json`
- `test/fixtures/claws/project-v1/package.json`

New current-tree paths are:

- `examples/inline-incident/package.json`
- `extensions/abliteration/package.json`
- `extensions/knowledge/package.json`
- `extensions/liquid-ai/package.json`
- `extensions/particle/package.json`
- `extensions/requesty/package.json`
- `src/templates/fixtures/delegating-agent/package.json`
- `test/fixtures/templates/project-v1/package.json`

This is exact-subject declaration-input drift evidence. It is not permission
to rewrite the fixed-baseline inventory, ledger source references, or existing
responsibility mapping.

## Closure boundary and disposition

This frontier enumerates declaration inputs and records their exact-tree
identity comparison. It does **not** establish:

- recursive relative-import or workspace-package closure;
- resolved npm/pnpm, Cargo, Swift, Ruby, Gradle, native or optional artifacts;
- patch bytes, registry tarball integrity/SRI, generated inputs or build graph;
- license/NOTICE/SBOM or LFS object completeness;
- exact-subject Actions inventory or dependency-closure receipt.

Therefore `dependency_closure_enumerated`, `external_recursive_dependencies_resolved`,
`license_and_lfs_closure_accepted`, `source_closure_ready`,
`baseline_identity_accepted`, and all final global flags remain unchanged and
false. The prior fixed-baseline inventory and historical dependency probes are
not reused for this candidate.

## References

- `projects/openclaw-rust/upstream.lock.json`
- `projects/openclaw-rust/dependencies/source-manifest-inputs.json`
- `projects/openclaw-rust/dossiers/upstream-current-candidate-probe-8c69e8b7-20261031.md`
- `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md`
- `docs/specs/openclaw-rust-equivalence-migration.md`
