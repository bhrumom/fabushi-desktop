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

The lockfile parses as two YAML documents, both lockfile version `9.0`, with
`201` importer rows in total (`200` unique importer paths), `1,859` package
records and `1,861` snapshot records. The tree has `209` unique
`package.json` paths. Nine manifests are not represented by a lockfile
importer: the three `.github`/release helper manifests,
`docs/snippets/plugin-publish/minimal-package.json`,
`qa/convex-credential-broker/package.json`, `scripts/cloudflare/package.json`,
and four fixture/example manifests under `src/claws/fixtures` and `test/fixtures`.
That difference is an explicit dependency-closure review set, not an omission
or proof of irrelevance. It must be classified against the exact source and
build/release consumers before dependency closure can be accepted.

The exact manifest read gives the following initial, evidence-backed
classification:

| manifest group | exact observed shape | required disposition review |
| --- | --- | --- |
| `.github/actions/setup-security-review/package.json` | private action runtime; `minimatch`, `yaml` | **applicable; standalone action closure**. The local action copies this manifest and its lockfile to a temp runtime and runs `npm ci --ignore-scripts`; the security workflow invokes the action and bundles both files. It is not a pnpm workspace importer. |
| `.github/release/clawhub-cli/package.json` | private release helper; `clawhub` | **applicable; standalone release-tool closure**. The materializer pins `clawhub@0.23.3` and its lock digest; the ClawHub publication workflow materializes this directory in multiple jobs; its workflow contract test reads both manifest and lockfile. It is not a pnpm workspace importer. |
| `.github/release/vercel-cli/package.json` | private release helper; `sandbox`, `vercel` | **applicable; standalone release-tool closure**. The materializer pins `vercel@63.1.2` and uses `sandbox`; the Vercel registry workflow materializes this directory before authentication. It is not a pnpm workspace importer. |
| `docs/snippets/plugin-publish/minimal-package.json` | public-looking sample plugin package; `typebox` | **applicable as documentation/example input, excluded from workspace closure by scope**. The plugin-building guide embeds the same `typebox@1.3.37` example and points readers to this snippet; it is not a product package or workspace importer. |
| `qa/convex-credential-broker/package.json` | private QA service; `convex`, `nostr-tools`, `@types/node`; deploy/dev/dashboard scripts | **applicable; standalone QA-service closure**. A QA test imports its validation code; the live-transports workflow supplies broker secrets and invokes lanes with `--credential-source convex`; the README documents install/dev/deploy. It is not silently excluded as a fixture and is not a pnpm workspace importer. |
| `scripts/cloudflare/package.json` | private deployment template; Cloudflare container, TypeScript and Wrangler dependencies; check/deploy scripts | **applicable as an operator-run experimental deployment template, outside the workspace importer set**. Its README directs `npm install`, `npm run check`, and `npm run deploy`, and `wrangler.jsonc` identifies the deployment template. No exact-tree workflow consumer was found in the targeted release/CI references, so it remains a separately reviewed operator path rather than “not applicable.” |
| `src/claws/fixtures/delegating-agent/package.json` | dependency-free fixture package | **fixture-only; exact test consumer found**. `src/cli/claws-cli.project.test.ts` invokes `runClawsDevCommand` against this fixture path. |
| `test/fixtures/claws/project-v1/package.json` | dependency-free golden fixture package | **fixture-only; exact test consumer found**. `src/claws/project.test.ts` resolves this directory as `goldenProject` and reads its manifest. |
| `test/fixtures/web-output-provider/package.json` | private dependency-free QA fixture package | **fixture-only; exact test consumer found**. `src/cli/capability-web-output.process.test.ts` copies this fixture into an isolated plugin directory. |

### Direct-consumer follow-up

The targeted exact-source read now resolves all nine rows above to explicit
consumers, documented operator entry points, or exact fixture tests. The three
dependency-free manifests are fixture-only rather than workspace packages;
their test/oracle responsibility remains in the source inventory. A GitHub
Code Search attempt could not be used as evidence because the endpoint
requires authentication; the exact commit's local Git object was available and
`git grep` supplied the fixture references below. Recursive package,
license/provenance, generated-input and semantic source closure remain open
beyond this manifest review.

The direct references used for the resolved rows are:

| manifest | exact consumer/documentation evidence |
| --- | --- |
| `.github/actions/setup-security-review/package.json` | `.github/actions/setup-security-review/action.yml` lines 16–21; `.github/workflows/security-review.yml` lines 154–203 |
| `.github/release/clawhub-cli/package.json` | `scripts/materialize-clawhub-cli.sh` lines 5–14; `.github/workflows/plugin-clawhub-new.yml` lines 444–445, 543–544, 693–694, 850–851; `test/scripts/plugin-clawhub-new-workflow.test.ts` lines 41–49 |
| `.github/release/vercel-cli/package.json` | `scripts/materialize-vercel-cli.sh` lines 5–15; `.github/workflows/vercel-container-registry-publish.yml` lines 107–117 |
| `docs/snippets/plugin-publish/minimal-package.json` | `docs/plugins/building-plugins.md` lines 72, 139, 241 |
| `qa/convex-credential-broker/package.json` | `test/qa-convex-credential-payload-validation.test.ts` line 2; `.github/workflows/qa-live-transports-convex.yml` lines 85–89, 639–691, 737–796, 835–882, 924–961, 1000–1038; `qa/convex-credential-broker/README.md` lines 20–35 |
| `scripts/cloudflare/package.json` | `scripts/cloudflare/README.md` sections 1–3 and 5; `scripts/cloudflare/wrangler.jsonc` line 43 |
| `src/claws/fixtures/delegating-agent/package.json` | `src/cli/claws-cli.project.test.ts` line 39 |
| `test/fixtures/claws/project-v1/package.json` | `src/claws/project.test.ts` line 16 (manifest read at line 38) |
| `test/fixtures/web-output-provider/package.json` | `src/cli/capability-web-output.process.test.ts` lines 13 and 30 |

This is a classification frontier, not a completed dependency closure: the
manifest consumers are now evidenced, but recursive dependency, generated
input, license/provenance and semantic source closure still require separate
exact-subject evidence.

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
