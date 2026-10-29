# OCRM technical dossier — locked OpenGrep security artifact audit

**Status**: source read-complete; security evidence only; no product runtime imported
**Project**: OCRM-001
**Audit date**: 2026-10-29 (Asia/Shanghai)
**Upstream authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)

## Exact source reads

All files below were fetched and read against the locked upstream commit. The
blob and line counts are retained so the source-closure ledger can distinguish
these artifacts from Fabushi runtime source.

| Path | Blob | Lines | Reading result |
| --- | --- | ---: | --- |
| `security/README.md` | `2747ce131c423f22ac45869c2a2c832ca846826a` | 137 | Security rulepack purpose, lifecycle, local/CI scan surfaces and provenance contract |
| `security/opengrep/README.md` | `b4de5119d1e626f3ba6ff9c5c81471e6ccb5da0d` | 136 | Compiled rulepack format, metadata, compiler modes, fixture and validation contract |
| `security/opengrep/check-rule-metadata.mjs` | `f74dd450b7f2432e85e142ee73faa4cf16439444` | 139 | Fail-closed advisory/source ID, URL, detector bucket and source-rule metadata checker |
| `security/opengrep/compile-rules.mjs` | `7872efa4d32365a8b0c35a432b0542be6307e1d4` | 677 | YAML ingestion, rule ID/provenance rewrite, append/update/replace behavior, OpenGrep schema pruning |
| `security/opengrep/precise.yml` | `7579a6a1b2e50fb63af4fbfd6d5d2826081d15d4` | 4981 | 148-rule compiled precise pack; each rule carries advisory/source provenance |
| `security/opengrep/rules/ghsa-82g8-464f-2mv7/skill-env.js` | `12fd677d8a1937626d229e851b76eb0e4ba6df07` | 32 | Static positive/negative fixtures for skill-derived host environment injection |
| `security/opengrep/rules/ghsa-82g8-464f-2mv7/skill-env.ts` | `ce937bc068f5b3471fd20ef32c27fb562c17b28e` | 146 | TypeScript taint fixtures covering aliases, tuple propagation, sanitizer result mutation and trusted inputs |
| `security/opengrep/rules/ghsa-82g8-464f-2mv7/skill-env.yml` | `24a98f003f81059ac75bdc84844613f8ae6cedca` | 116 | Taint sources, propagators, exact sanitizer and process environment sinks |
| `security/opengrep/rules/ghsa-fv94-qvg8-xqpw/ssh-sandbox-upload.js` | `dc02f425cb174383537ed32d87acdc3d7d6e328f` | 102 | Static fixtures for symlink-boundary validation ordering, await and directory identity |
| `security/opengrep/rules/ghsa-fv94-qvg8-xqpw/ssh-sandbox-upload.ts` | `5de0496ce3ed54b36bac7864080430cb1c50c22b` | 108 | Typed equivalent of SSH/tar upload validation and abort-signal fixtures |
| `security/opengrep/rules/ghsa-fv94-qvg8-xqpw/ssh-sandbox-upload.yml` | `ddd7e41b1603414cba4a918464b4f739a3dbdf38` | 33 | Rule requiring awaited safe-upload validation before tar/SSH spawn |
| `security/opengrep/rules/ghsa-qxgf-hmcj-3xw3/image-download.js` | `cf0fdfa92907b2dbddb133c28c14d7a553c5cf5d` | 159 | Static fixtures for raw image download, redirect, host/path mutation and shared SSRF guard exceptions |
| `security/opengrep/rules/ghsa-qxgf-hmcj-3xw3/image-download.ts` | `cf0fdfa92907b2dbddb133c28c14d7a553c5cf5d` | 159 | Same image-download fixture oracle in TypeScript surface |
| `security/opengrep/rules/ghsa-qxgf-hmcj-3xw3/image-download.yml` | `4d54874b4538665401986e25f60b155478d1a11f` | 35 | Rule requiring guarded image download and allowing only narrow pinned GitHub release shape |
| `security/opengrep/rules/openclaw-policy/no-raw-http2-connect.yml` | `531f876f2ac9474b95775f273785998cd117ca43` | 33 | Policy rule requiring the managed APNs HTTP/2 connector instead of raw `http2.connect` |

## Boundary and disposition

These files are security-evidence and maintainer tooling. The JavaScript and
TypeScript files under the GHSA directories are explicitly static scanner
fixtures and are not executable product functions. The YAML files describe
OpenGrep taint/pattern rules; `precise.yml` is a generated rulepack, not a
runtime policy engine. The two `.mjs` files validate or compile rule metadata
and invoke the external OpenGrep tool; they do not create an agent loop,
Gateway client, session store, scheduler, host runtime, or UI owner.

Accordingly, the files are recorded under `OCRM-M10` with disposition
`security-evidence`, without inventing a Fabushi Rust responsibility mapping.
They may inform future security-owner acceptance and scanner CI, but they do
not prove that Fabushi's existing sandbox, SSRF, secret, transport, or APNs
owners implement the protected behavior. No `wired`, `verified`, or product
acceptance status is changed by this audit.

## Security behavior captured as source oracle

- Skill environment data must reach `process.env` only through the consumed
  `sanitizeSkillEnvOverrides(...).allowed` projection; unrelated sanitizers,
  raw results, overwritten results and trusted service/saved environments are
  intentionally distinguished.
- SSH/tar uploads must await the symlink-boundary check for the exact uploaded
  directory before spawning tar/SSH; later, conditional, swallowed or
  non-awaited checks do not authorize the earlier upload.
- Image downloads must use the shared SSRF-guarded path. Fixed-origin GitHub
  release assets are the narrow exception only when both tag and asset are
  encoded; redirects, dynamic hosts/paths, userinfo and arbitrary URLs remain
  findings.
- APNs HTTP/2 callers must use the managed `connectApnsHttp2Session()` owner;
  the raw-connect rule excludes that owner and tests only the policy boundary.

These are source-level scanner contracts. They are not evidence of a passing
OpenGrep run, runtime behavior, deployed security control, or OCRM acceptance.
Development-stage verification remains frozen under
`projects/openclaw-rust/evidence/development-verification-freeze-20261029.md`.

## References

- `docs/specs/openclaw-rust-equivalence-migration.md`
- `projects/openclaw-rust/parity-ledger.json`
- `projects/openclaw-rust/scripts/mark_source_reads.py`
- Upstream `security/README.md` and `security/opengrep/README.md` at the locked commit
