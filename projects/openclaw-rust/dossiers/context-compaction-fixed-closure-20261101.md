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
`scripts/qualify_fixed_lockfile.py`). The larger fragment contains `200`
importers, `1749` package keys, and `1142` snapshots; of `81` candidate package
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

The corrected lockfile instrument now emits these classifications directly:
in the workspace fragment, all `15` unmatched named roots are fixed-tree
workspace manifests and the only `2` non-workspace unmatched roots are the
bounded recognizer's `+`/`:` candidates; the pnpm package-manager fragment
also carries the same workspace roots plus the whitespace candidate. This
separates valid workspace links from registry/archive resolution work without
turning either category into approval.

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
