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

## Disposition

This probe is stronger exact-subject discovery evidence for `CC-01`, but it
does not close recursive package/dependency closure. The unresolved generated
edges and no-manifest frontier remain explicit follow-up work. It must not
modify `upstream.lock.json`, inventory identity, parity-ledger status, or any
global completion flag. The local OCRM generator/validator/build/test/lint and
Actions authority lanes remain intentionally untouched during the development
freeze.
