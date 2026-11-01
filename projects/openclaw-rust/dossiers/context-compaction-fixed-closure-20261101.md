# Context-compaction fixed-source closure probe — 2026-11-01

Status: discovery evidence only; not source authority, dependency approval, or
OCRM acceptance.

## Exact subject

- Repository: `openclaw/openclaw`
- Commit: `1440bb1771372334bea23d7642bb52deb7db1845`
- Tree: `90c8aab39972da11cfad220361781d49e1bc6700`
- Fixed-tree tracked entries observed: `62130`
- Fabushi branch at probe time: `codex/openclaw-rust-full-fusion-20261101`

## Instrument and command

The existing read-only instrument
`projects/openclaw-rust/scripts/trace_fixed_relative_imports.py` was run from
a temporary immutable checkout with:

```text
python3 projects/openclaw-rust/scripts/trace_fixed_relative_imports.py \
  --repo <temporary-exact-checkout> \
  --commit 1440bb1771372334bea23d7642bb52deb7db1845 \
  --include-external-sources \
  --include-workspace-package-frontier
```

The temporary JSON result digest was
`2c23f16c418c5579bf62257c3150d27bec0f92d6328513129e55416035c59910`.

## Observed closure facts

- All seven configured context-compaction roots were present.
- Relative traversal reached `12251` fixed-tree files.
- Two relative edges remain unresolved, both generated Kysely artifacts:
  - `src/state/openclaw-agent-db.generated.ts` →
    `../../.artifacts/kysely/openclaw-agent-db.generated.js`
  - `src/state/openclaw-state-db.generated.ts` →
    `../../.artifacts/kysely/openclaw-state-db.generated.js`
- The traversal found `252` external module specifier candidates.
- The workspace/package frontier contained `103` manifest-found entries and
  `109` entries without a fixed-tree package manifest. Package manifests,
  exports, aliases, generated artifacts, registry-resolved packages, licenses,
  and archive identities therefore remain unqualified.

The companion bounded lockfile probe read the exact `pnpm-lock.yaml` blob
`83b4975eb96942ec007a2eb9847ccfe98d380bda` (`19227` lines; content SHA-256
`a4623b232742ae72f37072a40779d709b32436b430b88dc006902d1fa1287a4c`). It
observed two YAML document fragments separated by valid `---` markers (the
earlier probe's `duplicate_top_level_lockfile_headers` label was a parser
classification defect, now corrected in
`scripts/qualify_fixed_lockfile.py`). With inline snapshot keys recognized, the
larger fragment contains `206` importers, `1749` package keys, and `1750`
snapshots; of `81` candidate package
roots derived from the closure frontier, `64` matched and `17` did not. This
parser intentionally reports raw lockfile observations and does not resolve
aliases, peer suffixes, package archives, integrity/license provenance, or the
unmatched roots.

The `17` unmatched roots split into `15` workspace package names (`@openclaw/*`
plus `openclaw`) that must be reconciled through importer/workspace manifest
edges, and three malformed lexical candidates (`+`, whitespace, and `:`) from
the intentionally bounded import recognizer. These are classified frontiers,
not silently treated as missing registry packages.

The workspace names were independently located in the same exact tree at these
manifest paths: `packages/acp-core`, `packages/agent-core`, `packages/ai`,
`packages/gateway-client`, `packages/gateway-protocol`, `packages/llm-core`,
`packages/media-core`, `packages/model-catalog-core`, `packages/net-policy`,
`packages/normalization-core`, `packages/retry`, `packages/session-url-contract`,
`packages/workboard-contract`, `packages/worker-runtime`, and the repository
root (`openclaw`). This proves fixed-tree manifest presence only; importer
dependency edges and package archive/integrity provenance remain to be joined
before closure can be accepted.

The remaining `109` no-fixed-tree-package-manifest frontier entries were
classified without inventing package ownership: `62` are package-root
candidates, `45` are package-subpath/export candidates, and `2` are the
bounded lexical `+`/`:` candidates emitted by the import recognizer. The
package-root and subpath classes are external/package-export frontiers rather
than evidence that a fixed-tree manifest should exist; their published
manifest, exports, archive, license, and recursive dependency provenance remain
unresolved.

The corrected lockfile instrument now emits these classifications directly:
in the workspace fragment, all `15` unmatched named roots are fixed-tree
workspace manifests and the only `2` non-workspace unmatched roots are the
bounded recognizer's `+`/`:` candidates; the pnpm package-manager fragment
also carries the same workspace roots plus the whitespace candidate. This
separates valid workspace links from registry/archive resolution work without
turning either category into approval.

The optional exact-manifest dependency frontier was then emitted from the
same fixed tree with:

```text
python3 projects/openclaw-rust/scripts/qualify_fixed_lockfile.py \
  --repo <temporary-exact-checkout> \
  --commit 1440bb1771372334bea23d7642bb52deb7db1845 \
  --frontier-json <closure-json> \
  --include-manifest-dependencies
```

The machine-readable result digest is
`0a6a49a6976f3617f1fe95749acc628e417f2261e8d142bba9ac1778596302c8`.
After de-duplicating repeated frontier rows by exact manifest path/package,
the probe scanned `15` exact manifest identities; `13` of them declare at
least one dependency row. The output contains `176` declared dependency edges:
`17` workspace-specifier edges and `159` registry-or-path edges. A bounded
package-name candidate join against the same raw lockfile classified `155`
edges as exact-version candidates, `4` as name candidates requiring range/peer
selection, and all `17` workspace edges as matching the expected importer,
specifier, and `link:` target shape. This is a bounded importer/package join,
not dependency approval: the `155` registry importer versions match candidate
package keys after removing pnpm peer suffixes, while four optional peer
declarations remain explicitly `peer-selection-required` with multiple or
range candidates. It does not establish recursive package resolution, peer
selection, archive bytes, generated artifacts, or license/SBOM provenance.
The resolved registry importer keys also have `155/155` observed snapshot
keys in the lockfile; the four peer declarations remain unselected rather
than being treated as resolved snapshots.
The bounded recursive walk from the `136` deduplicated resolved snapshot roots
reached `1134` snapshots with `0` missing dependency edges after recognizing
the exact scoped `npm:` alias form. Its `closed` status applies only to this
selected manifest-root graph; it does not claim that all `1750` lockfile
snapshots are reachable from these roots or that peer candidates are selected.
The same summary counts `616` snapshots outside this selected root graph;
examples include alternate ACP/Anthropic/AWS platform and peer variants. They
remain in the full lockfile inventory and are not silently deleted from scope.
The four peer declarations yield six base candidate package keys, six exact
snapshot-root variants, and `20` reachable candidate snapshots with zero
missing edges. This is candidate-graph evidence only; it does not select a
peer resolution for the production install.
As a separate structural check, walking all `1750` snapshot keys as roots
reached all `1750` with zero missing dependency/optionalDependency edges. This
closes the bounded lockfile graph shape, not the external archive, package
license, generated-artifact, install, or Actions evidence lanes.
The selected `155` registry package keys and all `4` peer candidate sets have
an integrity observation in the lockfile; this remains lock metadata, not
independently verified archive bytes or license provenance.
At package-key level, the corrected parser observes integrity values for all
`1764` package keys across the two lockfile fragments (`0` missing). This is
still lock metadata coverage and does not prove that the referenced archives
were downloaded, match their bytes, or satisfy license/SBOM requirements.
Eight manifest edges also match a lockfile `patchedDependencies` hash entry;
the remaining registry edges are explicitly `not-listed`, not inferred
unpatched.
The same output records source-manifest provenance for all `15` manifests:
four declare `MIT`, while eleven are private package manifests with no
manifest license field. These are source declarations only; published package
license files, third-party notices, archive contents, and SBOM attribution
remain unverified.
The output still reports `2` lockfile fragments,
`document_separator_count=2`, and
`duplicate_top_level_lockfile_headers=false`.

The fixed tree's source-level LFS check found no Git LFS pointer blobs. The
exact root provenance inputs are `.gitattributes`
(`c6118b226a4a760f7861b6b7b1e7dccd943384e4`), `LICENSE`
(`ed064819ab537575d0cad902f7e5af4a3488b1d0`), and
`THIRD_PARTY_NOTICES.md`
(`0c9c123760188915a384ce7275df4096114e8f90`). The absence of LFS pointers
does not establish recursive package archive, license attribution, or SBOM
closure; those remain separate evidence lanes.

Current-subject compaction source identities were also re-read directly:

| Path | Blob |
| --- | --- |
| `src/agents/sessions/compaction/compaction.ts` | `8cc6dc9c87269c5d64de1bf4352365a79bfd57dd` |
| `src/agents/sessions/compaction/runtime.ts` | `adfc2f1cca3d58d0ebfd0391dde16a8cd7d6fed1` |
| `packages/agent-core/src/harness/compaction/compaction.ts` | `9ee5706cfd94a75f36e43752e16d1f75d9975147` |
| `packages/agent-core/src/harness/compaction/compaction-cut-point.ts` | `af55af34ac16e508aa739e12aed3bebe9878b5ea` |
| `packages/agent-core/src/harness/compaction/compaction-details.ts` | `26f973ce17756ebe0ae6754798b31eabd2773ba8` |
| `packages/agent-core/src/harness/compaction/summarization-completion.ts` | `7e4b86c73a22a0cd942f36dd73d857f4cb3c5669` |
| `packages/agent-core/src/harness/compaction/summary-checkpoint-prompt.ts` | `e04fa9be9e91ae99d40390cc122690905577732e` |
| `packages/agent-core/src/harness/compaction/utils.ts` | `eb9190049906e875fcbe268ce977a5e6400425b5` |

These current blobs show the facade/runtime split, cut-point and tool-result
pairing, bounded compaction details, and summarization completion path. They
are source mapping evidence only; the complete runtime/package dependency
closure and a production Runner producer/typed replay consumer remain open.

The two unresolved relative edges are generated Kysely declarations and are
absent from the exact tree:
`.artifacts/kysely/openclaw-agent-db.generated.js` and
`.artifacts/kysely/openclaw-state-db.generated.js`. The fixed tree does retain
the generator `scripts/generate-kysely-types.mts`
(`08060fa786ea57601647279c8302cba8b78be544`) and schema input
`src/state/openclaw-agent-schema.sql`
(`e3a3cfdb772cbaf28ee6d2f0af46a33428938489`); `CONTRIBUTING.md` states that
the generated declarations live in ignored `.artifacts/kysely/` and are
refreshed by install/build tooling. Therefore the generated edge is
`source-present/generated-artifact-absent`, not a missing source file and not
closure approval. Generator execution and generated-byte verification remain
reserved for the exact-subject Actions lane.

For the compaction behavior roots, the first workspace dependency chain is
explicit in exact manifest blobs:

- `packages/agent-core/package.json`
  (`d6a0a193eaa5ca83cf26eb1b3cc85a425c23d42a`) links workspace `ai`,
  `llm-core`, and `normalization-core`, plus registry `typebox@1.4.3`.
- `packages/ai/package.json`
  (`2ee99c5de54591d5dcad3b7b29cde9fc07549718`) declares the registry AI,
  telemetry, `openai@7.43.0`, `typebox`, and Zod family, with optional tracing
  peers.
- `packages/llm-core/package.json`
  (`d7b6312ebf2eaf900b6d3216048afe439d58247d`) links
  `normalization-core` and `typebox`.
- `packages/normalization-core/package.json`
  (`13606635cecef520cc2a8ebc4e10d7769559f59f`) declares
  `libphonenumber-js@1.13.15` and `typebox@1.4.3`.

The root manifest and exact lockfile blobs are respectively
`6989c0618742d3c2b93cc336cbe1d9c3c4f0a657` and
`83b4975eb96942ec007a2eb9847ccfe98d380bda`. This establishes a bounded
workspace→registry frontier for the compaction closure; it does not yet prove
archive bytes, peer resolution, generated artifacts, or license/SBOM closure.

The bounded lock observation has a matching registry key/integrity for
`typebox@1.4.3`, `@anthropic-ai/sdk@0.137.0`, `openai@7.43.0`, and
`zod@4.6.7`; it also records alternate peer-compatible `openai` and `zod`
versions in the same lock. Manifest-only dependencies such as
`libphonenumber-js`, `partial-json`, and `proxy-from-env` still require the
recursive importer/package join before they can be counted as closed.

## Disposition

This probe is stronger exact-subject discovery evidence for `CC-01`, but it
does not close recursive package/dependency closure. The unresolved generated
edges and no-manifest frontier remain explicit follow-up work. It must not
modify `upstream.lock.json`, inventory identity, parity-ledger status, or any
global completion flag. The local OCRM generator/validator/build/test/lint and
Actions authority lanes remain intentionally untouched during the development
freeze.

## Typed replay boundary audit (2026-11-01)

The current Fabushi production boundary cannot yet carry the fixed OpenClaw
compaction projection without losing transcript semantics. The inference
owner's `ProviderMessage` at
`source/host/src/extensions/inference/provider_session.rs:89-92` contains only
`role` and text `content`. `source/host/app/src/main.rs:6989-7010` decodes
request rows into that two-field shape, and
`source/host/src/runner/routed_provider_runtime.rs:404-414` forwards the same
flattened representation to the provider attempt executor.

The canonical transcript owner is richer than that boundary: transcript
validation accepts `send-message`, `message`, `user-attachment`, `tool-call`,
`notice`, and `event` rows in
`source/host/src/extensions/session/agent_db_serde.rs:108-132`; production
message rows can also carry `attachments` (for example
`source/host/src/extensions/session/production.rs:3560-3578`), while the
fixed OpenClaw compaction source explicitly preserves compaction summaries,
tool calls, tool results, and non-text content. The existing replay API
`ProductionSessionWorkers::read_agent_compacted_transcript_entries` therefore
returns JSON rows but has no production consumer that can validate and project
those typed cases into inference input.

The first owner-local repair is now present in
`source/host/src/extensions/session/typed_replay.rs`, exposed through
`ProductionSessionWorkers::read_agent_typed_compacted_transcript_entries`.
It preserves summary metadata, user/assistant/tool roles, tool-call/result
identity, and attachment values, and rejects legacy unpaired `tool-call` rows
instead of stringifying them. This is an owner-boundary contract and focused
test surface only; no Runner or provider path calls it yet.

This is a fail-closed `CC-07`/`AC-CC-05` boundary, not permission to stringify
unsupported rows or widen `ProviderMessage` opportunistically. Until the
owning inference/provider contract defines and verifies a typed message
representation plus provider-specific serialization for summary context,
assistant/user roles, tool-call/tool-result pairing, and attachments, the
production replay consumer remains unresolved and compaction wiring must not
be enabled.

## Provider serialization capability matrix (2026-11-01)

The inference owner has no single serialization path that is rich enough for
the typed replay contract:

| Provider path | Current source boundary | Typed-history coverage | Blocking gap |
| --- | --- | --- | --- |
| Cursor | `cursor_inference_transport.rs` has `CursorCheckpointMessage` and protobuf tool-call/tool-result fields | tool-call/result pairing is representable | attachment-bearing content is not represented in the protobuf; the public `ProviderMessage` conversion still discards checkpoint structure |
| Codex / Fabushi | `provider_session.rs` maps input through the Responses request owner | typed summary/tool-call/result replay is now native through `CodexDirectOptions.input` | attachment resolution still requires provider-usable URLs; live exact-head/provider acceptance is open |
| OpenRouter | `provider_session.rs` builds Chat Completions `messages` through the existing OpenRouter owner | typed summary/tool-call/result replay now uses native assistant/tool messages | image attachments require provider-usable URLs; live exact-head/provider acceptance is open |
| Claude Code | `provider_session.rs` renders one CLI prompt through the existing Claude owner | summary and plain user/assistant replay can be represented; unsupported tool/attachment rows reject | typed tool pairing and attachments remain unsupported by the current CLI invocation |

This matrix makes the next ownership boundary explicit: the inference owner
must accept a typed replay input and select a provider-specific serializer;
the attachment owner must resolve durable attachment identities into a
provider-supported representation or return a fail-closed unsupported-media
result. The current `ProviderMessage` path remains valid for ordinary turns
and must not be silently reused for compacted typed replay.

An inference-owner pure serializer contract is now present in
`source/host/src/extensions/inference/typed_replay_serialization.rs`:
Responses serialization preserves the summary envelope and balanced
function-call/output items; Chat Completions serialization preserves native
assistant tool calls and tool results. Both reject unresolved local
attachments, and Chat Completions rejects non-image media because no provider
file-part contract has been established. Cursor serialization now preserves
its native assistant-tool/tool-result checkpoint pairing, while rejecting all
attachments because the current Cursor protobuf has no attachment part. The
Claude serialization now explicitly accepts only summary and plain
user/assistant text, rejecting typed tool rows and attachments because the
current CLI has no typed replay surface. The provider-specific serializers are
now connected to the live provider owners: Cursor through its typed checkpoint
conversation, Codex/Fabushi through Responses input, OpenRouter through native
Chat Completions messages, and Claude through its supported text-only subset.
Unsupported media/tool shapes and exact-head/provider acceptance remain open.

The optional typed replay channel is now threaded through the existing
production turn path: `ProductionTurnInputProjection` →
`GeneratedAgentTurnContext` → `TurnAgentComposition` → `RoutedProviderRun` →
`ProductionRoutedProviderAttemptExecutor` → `RoutedProviderOptions`. Ordinary
turns pass `None`. A non-empty replay is rejected at the provider runtime and
provider-session dispatch boundaries until the canonical replay producer,
system/current-turn merge, attachment resolution, and provider-specific
serialization are all wired. Cursor now consumes the channel through its
existing typed checkpoint conversation; the canonical app entrypoint supplies
the channel only when an active compaction exists. Cursor attachments remain
fail-closed because its protobuf has no attachment part. Claude now accepts
only the serializer's summary/plain-text subset and rejects typed tool or
attachment rows. This is a partial production wiring
slice, not full replay acceptance.

The existing attachment resolution evidence is app-owned rather than an
inference contract: `source/host/app/src/main.rs:605-670`
`resolve_production_attachment_source` can persist host bytes or download box
bytes into the agent media store and return a `file://` URL, while the current
provider-facing attachment resolver at `main.rs:833-875` is scoped to an
outgoing tool/send-message path and its admitted media settlement. No
inference-owner API currently resolves a historical transcript attachment for
replay with a provider capability policy. Reusing the outgoing send path for
compaction would cross ownership and settlement boundaries; a narrow
read-only attachment projection contract is required before typed replay can
be enabled.
