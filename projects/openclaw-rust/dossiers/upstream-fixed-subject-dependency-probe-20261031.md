# OpenClaw fixed implementation subject — dependency probe

**Observed**: 2026-10-31 (local Git object read only)  
**Subject**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`  
**Tree**: `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`  
**Authority**: implementation subject only; accepted/source-closure flags remain false

## Exact root inputs

| Path | Blob |
| --- | --- |
| `AGENTS.md` | `c3deeccdb6b1a4c7137d56297e933bbf19f4d3bf` |
| `package.json` | `af543c4c033b5c5e0c466061ada470feeda777f3` |
| `pnpm-lock.yaml` | `80fb4d9fa985297a24ac88931ef534e85a40ba47` |
| `pnpm-workspace.yaml` | `d4ebbf1c13db7e189fe4053122a3e3fa77bb0e26` |
| `.gitattributes` | `c6118b226a4a760f7861b6b7b1e7dccd943384e4` |
| `LICENSE` | `ed064819ab537575d0cad902f7e5af4a3488b1d0` |
| `THIRD_PARTY_NOTICES.md` | `0c9c123760188915a384ce7275df4096114e8f90` |
| `.gitmodules` | absent |

The tree has 58,750 paths and no gitlinks. These are source/tree facts only;
they do not establish package or source closure.

## Manifest and workspace facts

The exact root manifest declares 75 runtime dependencies, 58 development
dependencies, 1 optional dependency, and 0 peer dependencies. Its package
manager is pnpm `12.15.0` with a pinned package-manager digest. The workspace
patterns are `.`, `ui`, `packages/*`, `extensions/*`, `examples/*`, and
`tools/solid-lint`; the exact tree contains 208 `package.json` paths matching
the root/workspace candidate shape.

## Lock-structure and parser boundary

The exact `pnpm-lock.yaml` is 19,772 lines and contains these top-level section
occurrences:

| Section | Occurrences | Lines |
| --- | ---: | --- |
| `importers` | 2 | 4, 221 |
| `packages` | 2 | 13, 3,193 |
| `snapshots` | 2 | 97, 11,165 |
| `patchedDependencies` | 1 | 213 |

The lock contains two YAML documents separated at line 158. The first document
has one root importer with only `pnpm@12.15.0` and 15 pnpm platform/package
entries; the second document starts with its own lockfile version and contains
the application/workspace dependency graph. A single-document parser such as
Ruby Psych reads only the first document, so it is not a valid closure oracle
for this lock. The later `YAML.load_stream` recheck in this dossier is the
authoritative static interpretation of the two-document structure for this
probe.

The static recheck is stronger evidence but not accepted closure evidence:
root manifest dependencies, 208 workspace candidates, patched dependencies,
optional/peer relations, external package bytes, and install provenance still
require independent exact-subject verification. `dependency_closure_enumerated`
and `source_closure_ready` therefore remain false.

## Required next instrument

The exact-head Actions authority must independently establish which lock
section is authoritative (or regenerate a canonical lock from the same exact
subject), enumerate every applicable workspace importer and resolved package,
bind patches/native/optional/peer edges and integrity/license inputs, and emit
the immutable inventory/closure artifact. Until then the fixed subject remains
the implementation comparison subject, not an accepted source authority.

## Evidence boundary

No package was installed, downloaded, rehashed, built, or executed by this
probe. No OCRM workflow, local OCRM check, Rust build/test/formatter/lint,
push, PR update, or completion flag was changed.

## Parser correction and static graph recheck (2026-10-31)

The earlier wording above treated the repeated section names as an ambiguous
single YAML document because the first read used a single-document parser. The
exact lock actually contains two YAML documents: the first document is the
pnpm `12.15.0` package-manager lock, and the second document is the
OpenClaw application/workspace lock. The separator is the `---` at line 158;
the second document starts with its own `lockfileVersion` and contains the
application `settings`, `overrides`, `patchedDependencies`, `importers`,
`packages`, and `snapshots` sections.

A read-only `YAML.load_stream` recheck against the exact Git object produced:

| Document | Importers | Packages | Snapshots | Patched dependencies |
| --- | ---: | ---: | ---: | ---: |
| pnpm package-manager lock | 1 | 15 | 15 | 0 |
| OpenClaw application lock | 200 | 1,844 | 1,846 | 6 |

The application root importer has 75 dependencies, 58 devDependencies, and 1
optional dependency, matching all 134 declared root manifest names. Every root
importer resolution matched a package/snapshot key, and all 3,396 dependency
edges in the application snapshot graph resolved after honoring pnpm alias
values (including platform-optional Codex packages and the domexception
alias). This is stronger static lock consistency evidence and corrects the
prior single-document parser interpretation.

The recheck still does not establish accepted dependency closure: workspace
manifest-to-importer identity, patch/native/optional/peer semantics, external
archive bytes, license/notice provenance, source closure, and exact-head
Actions evidence remain unverified. `dependency_closure_enumerated` and
`source_closure_ready` therefore remain `false`.

The same exact-object recheck compared workspace importer keys with every
`package.json` path. There are 208 package manifests in the tree, 200 importer
keys, and all 200 importer keys have a matching manifest. The eight manifests
without importers are outside the declared workspace package roots and are
classified as non-workspace candidates: three `.github`/release entries,
`qa/convex-credential-broker`, `scripts/cloudflare`,
`src/claws/fixtures/delegating-agent`, and two `test/fixtures` entries. No
importer lacks a manifest. This closes the prior workspace-count ambiguity at
the static mapping layer, while external package provenance and Actions
authority remain open.

## Workspace and package-edge recheck (2026-10-31)

The exact application lock was checked against the exact Git tree without an
install. All 200 importer paths matched a manifest with no name/path mismatch;
all 263 `link:` edges resolved to an existing workspace manifest after
resolving each link relative to its importer directory. All 1,844 package
resolution entries carry a declared integrity string. The lock also records
285 platform-qualified package entries, 318 optional snapshots, and 131
package entries with peer-dependency declarations. All six patched dependency
keys have corresponding patch files under `patches/`.

These are static consistency and declared-provenance facts only. They do not
verify downloaded archive bytes, patch application, native build outputs,
peer-environment correctness, license/NOTICE contents, or exact-head Actions
reproduction; closure and authority flags remain false.
