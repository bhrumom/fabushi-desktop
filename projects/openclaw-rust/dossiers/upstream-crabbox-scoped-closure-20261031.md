# OpenClaw Crabbox scoped closure frontier — 2026-10-31

This is a read-only, exact-subject closure note for the current live candidate
`openclaw/openclaw@129583af012bf4177b37ba6d88e8b2e5fa0d532f`, root tree
`84860d166bdddef60cbc3ede3909d1a28696b4d9`. It is discovery evidence only. It
does not promote the candidate to `upstream.lock.json` authority, does not
replace the required GitHub Actions inventory/closure receipt, and does not
authorize new product implementation or acceptance flags.

## Exact scoped inventory

The GitHub recursive tree API returned `truncated=false` for the candidate's
full tree (`63,874` entries: `61,633` blobs and `2,241` directories). The
`extensions/crabbox/` prefix contains `118` entries: `114` blobs and `4`
directories. The scoped identities below are from that same exact tree:

| Path | Blob identity |
| --- | --- |
| `extensions/crabbox/package.json` | `4a9de08781371a6803f08c11b26b4285f76aa747` |
| `extensions/crabbox/openclaw.plugin.json` | `4c09ec4e64721e7e9f9de5b4833015a8975745e0` |
| `extensions/crabbox/index.ts` | `4beae4b29809778dc25644a4e13b64dd437560a5` |
| `extensions/crabbox/src/crabbox-worker-provider.ts` | `b33b8bfeb488be5ea9bb0f9d56b0a953b36e39be` |
| `extensions/crabbox/src/crabbox-worker-provider-binary.ts` | `965879edcbf65a1545d0954ffb3110e70564d4a4` |
| `extensions/crabbox/src/crabbox-worker-node-enrollment.ts` | `96df2ae9e60ed32d83700bd95057d8b7c285bf3e` |
| `extensions/crabbox/src/crabbox-worker-node-process.ts` | `414d84ab591091511ee6abd66bd12d8b064e4763` |
| `extensions/crabbox/src/crabbox-process.ts` | `0ccfe03b9db815de9ea466891396eacc76597203` |
| `extensions/crabbox/src/crabbox-gateway-methods.ts` | `5c9cffc5b1ad9150ca3c28ab3de8d79f16e9451e` |

The prefix includes the plugin assets, manifest, skill, CLI/runtime surface,
gateway methods, provider/process/bootstrap/desktop/sandbox/warm-image source,
and their tests/support files. Tests and support files remain in the scope;
they cannot be dropped merely because the target is a production Rust path.

## Confirmed first-hop dependency edges

`extensions/crabbox/index.ts` registers the plugin entry, tool, CLI, gateway
methods, worker provider and sandbox backend. Its first-hop dependencies are
Node's `node:url`, the OpenClaw plugin-entry and sandbox SDK surfaces, and the
Crabbox modules `crabbox-sandbox-backend`, `crabbox-sandbox-config`,
`crabbox-sandbox-lease`, `crabbox-tool`, `crabbox-worker-profile`,
`crabbox-worker-provider`, and `crabbox-worker-warm-image-policy`.

`crabbox-worker-provider.ts` directly depends on the plugin-entry/error runtime
and the Crabbox command, heartbeat, inspect, lifetime, machine-options,
node-enrollment diagnostics, profile, project, provider-binary,
provision-command, setup-egress, snapshot-actions, timeout, wallpaper,
warm-image-policy/store and warm-image modules. The provider is not a passive
catalog: it resolves the configured/managed binary, provisions and stops
leases, manages heartbeat and warm-image state, runs setup egress, performs
node enrollment and exposes provider operations.

`crabbox-worker-provider-binary.ts` directly crosses the plugin error/time
runtime, binary resolver and managed-binary modules. The node-enrollment path
directly crosses desktop setup, node-process, profile, worker-script and
timeout modules. The command path directly crosses plugin logging,
process-runtime and text-utility surfaces plus the Crabbox process and timeout
modules. The gateway path directly crosses plugin error, entry and string
coercion helpers plus snapshot/actions, warm-image state and profile modules.

The workspace boundary is also identity-bearing: the Crabbox package depends on
the workspace `@openclaw/plugin-sdk`; the workspace includes the root package,
`ui`, `packages/*`, `extensions/*`, `examples/*` and `tools/solid-lint`; and
the root `pnpm-lock.yaml` plus package-manager policy, patches, build scripts,
generated runtime manifests and native/optional build inputs remain closure
edges. `packages/plugin-sdk/package.json` is the package export authority for
the imported runtime surfaces. A source-only read of the Crabbox prefix is
therefore not a complete dependency closure.

## Closure frontier and disposition

This scoped read establishes a stronger frontier than the older historical
Crabbox notes, but it does not close the following required edges:

- recursive relative imports for all `114` scoped blobs and their test/support
  graph;
- every imported `openclaw/plugin-sdk/*` implementation and its transitive
  imports;
- root runtime/plugin registration, Gateway worker-provider contracts,
  sandbox backend ownership, state store/migration owners and generated build
  inputs;
- `pnpm-lock.yaml` package resolution, patch bytes, native/optional package
  artifacts, integrity/SRI, license/notice/SBOM and LFS/asset integrity;
- dynamic imports, command registration, config-schema consumers and packaged
  CLI/UI/Electron/NodeHost consumers outside the prefix;
- test oracle closure and exact target-side Rust owner/consumer evidence.

Accordingly `dependency_closure_enumerated`, `source_closure_ready`,
`baseline_identity_accepted`, responsibility mapping and all final acceptance
flags remain false. No `upstream.lock.json` authority field was changed from
this note. The evidence is based on GitHub API tree/blob reads only; no local
OCRM build/test/lint, Actions run, push or PR update was performed.

## References

- `projects/openclaw-rust/upstream.lock.json`
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `docs/specs/openclaw-rust-equivalence-migration.md`
- `projects/openclaw-rust/decisions/2026-10-29-crabbox-worker-owner-boundary.md`
- `projects/openclaw-rust/dossiers/cloud-workers-status-consumer-20261030.md`

## Subject recheck — 2026-10-31T04:08:46Z

The same scoped tree was re-read against the newer live candidate
`openclaw/openclaw@510e1687e965c1a5e2c372f6622704c88448fb5b`, root tree
`8260de46bfdd6ac1a78369ebf2343cb255de20e5`. The full recursive tree was not
truncated; `extensions/crabbox/` still contains `118` entries (`114` blobs,
`4` directories), and all nine key blob identities listed above are unchanged.

This is an identity-preserving scoped observation, not a closure receipt: the
candidate's full tree has changed from the earlier `129583af...` subject, so
the earlier scoped note cannot be relabeled as current full-tree authority.
The dependency frontier, Actions inventory/closure, LFS/integrity, generated
inputs, licenses and packaged acceptance remain open for the new subject.

The current subject's first-hop runtime surfaces were also identity-checked:

| Imported surface | Exact blob |
| --- | --- |
| `packages/plugin-sdk/package.json` | `2ae3706819610eea18d127130a2eb64cbfb89c25` |
| `packages/plugin-sdk/src/plugin-entry.ts` | `82796abdb21c4a24bafd18be164efc70f0d3766f` |
| `packages/plugin-sdk/src/error-runtime.ts` | `e0151c9bc5ced6f6d7d6fb98a876df8ede62a89c` |
| `packages/plugin-sdk/src/time-runtime.ts` | `6749e3e370038475d6b76f1827dd356fb0118d40` |
| `packages/plugin-sdk/src/text-utility-runtime.ts` | `93efdcde9465d20d655ae2488ba94bda4c9c2f2b` |
| `src/plugin-sdk/logging-core.ts` | `d53d7eaa9e4ebdd3b6e4f215b2d03bb8a86c9cf9` |
| `src/plugin-sdk/process-runtime.ts` | `b84627cbccc5febf67df529e3456f254200efeee` |
| `src/plugin-sdk/sandbox.ts` | `fbbe7446fde322d86ba59f3f54a89e818ac950a2` |
| `src/plugin-sdk/string-coerce-runtime.ts` | `ecb5392d5b93242bd2899b11246197a9fbd4ea26` |

These are direct identity edges only. Their own imports, generated exports,
build-time aliases and transitive package/runtime dependencies remain outside
this scoped note and must be closed by the exact-subject authority run.

## Subject recheck — 2026-10-31T04:25:12Z

The same scoped tree was re-read against the newer live candidate
`openclaw/openclaw@10d93efc9c8730b7a2fad831c20c649adec7176b`, root tree
`d33ed7c344544baf60148a41afab9e9a9547e9f4`. The recursive tree remained
untruncated with 63,881 entries (61,640 blobs and 2,241 directories).
`extensions/crabbox/` remains 118 entries: 114 blobs and 4 directories, and the
nine previously recorded implementation identities remain unchanged. The
scoped result is still discovery-only; recursive imports, generated exports,
package/patch/native closure, LFS/integrity, license/NOTICE and Actions evidence
remain open.

## Subject recheck — 2026-10-31T04:20:22Z

The same scoped tree was re-read against the newer live candidate
`openclaw/openclaw@8e61893b6e692b14d4741ccdf681e1f932289f2f`, root tree
`4e9cca115794d90ad9f056871103790101022764`. The recursive tree remained
untruncated with 63,880 entries (61,639 blobs and 2,241 directories).
`extensions/crabbox/` remains 118 entries: 114 blobs and 4 directories.

The nine previously recorded Crabbox implementation identities remain unchanged
on this exact subject, including the plugin entry/manifest, provider and binary,
node enrollment/process, process lifecycle and gateway-methods sources. The
scoped result is still discovery-only: recursive imports, generated exports,
build aliases, package/patch/native closure, LFS/integrity, license/NOTICE and
Actions evidence remain open and no authority or completion flag is promoted.
