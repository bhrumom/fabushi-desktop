# H74 completed renderer and source-authority evidence

## Verdict and immutable subject

**Renderer scoped execution: passed with original evidence. Source-authority: failed before canonical expansion and P0 validation. Full migration acceptance remains unapproved.**

- Repository: `bhrumom/fabushi-desktop`.
- Subject H74: `74f11d3d31c6245ac5886dc0181460b2d4349744`.
- Subject tree: `aca91e1cfbedeecb039d442fb3528caac89873a3`.
- Fixed upstream: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Runtime run `41580539491`, attempt `1`; renderer job `125734079487`, completed **success**, 2026-11-03 04:51:43–04:52:19 UTC.
- Source run `41580539540`, attempt `1`; source-authority job `125734077461`, completed **failure**, 04:51:42–04:52:47 UTC.

The runtime/source run API payloads bind H74 to its exact tree; the upstream Git commit API independently binds U to its tree. All three original renderer receipts agree on H74/tree/U/run/attempt. This report does not transfer those results to a descendant, a later evidence carrier, another H, or other runtime jobs.

Review worker scope was evidence collection **and provenance/verdict assessment**; parent recorded actual launch at **xhigh**, configured model unchanged. No repository build, test, linter, generator, schema validator, parser or harness was run locally or on htch-runtime. Only read-only Git/API retrieval, original-evidence extraction/readback, and byte/hash comparisons were performed. Writes are confined to this review directory.

## Source-authority terminal failure

Original log: [source-log.log](source-log.log). API metadata: [source-jobs-api.json](source-jobs-api.json), [source-run-api.json](source-run-api.json).

Step 9, **Expand canonical source records and verify P0 G01/G02 structural**, failed at the first pre-expansion integrity invocation:

```text
python3 projects/openclaw-rust/scripts/verify_audit_integrity.py --before-expansion
AssertionError: Missing reverse source/responsibility link: OCRM-RESP-MEDIA-FINAL-CUSTODY-001
Process completed with exit code 1.
```

The traceback is `verify_audit_integrity.py:100` → `source_review.py:94`. Exact-H source reading confirms this is the canonical ledger-to-inventory reverse-link guard. Its permitted pre-expansion exception requires an exact pending addition and pending verification/acceptance state; the observed subject was refused by that guard. This is direct evidence of a structural accounting failure. It does not by itself diagnose a product-runtime defect.

Retained earlier positives are bounded:

| Executed work | Original evidence |
| --- | --- |
| Exact checkout/lock agreement and fixed U acquisition/tree check | Steps 3 and 5 success; original log includes H74/U identity |
| Risk-review/fail-closed unit suite | `python3 projects/openclaw-rust/scripts/test_source_review.py`: **33 tests, OK** |
| Recursive tracked-source inventory generation, LFS candidate scan, dependency-root recording | Steps 6–8 success; their generated outputs were not uploaded |
| Inventory-materialization unit suite | `python3 projects/openclaw-rust/scripts/test_inventory_materialization.py`: **21 tests, OK** |

Because step 9 runs with `set -euo pipefail`, the following commands after the failing integrity invocation were **not reached**: `expand_inventory.py`, `shard_inventory.py --write`, `validate_p0.py ... --negative-probes`, and final `verify_audit_integrity.py`. The quoted command block is not proof those later commands ran.

The single source-artifact read returned an empty list. The terminal job independently records **Upload immutable authority evidence: skipped**; `persist-verified-inventory` was also skipped. There is no original successful P0 structural receipt to collect or promote from this run. The 33/21 unit-suite positives do not establish global G01/G02, final AC02, complete dependency closure, or acceptance.

## Renderer actual commands and results

Original log: [renderer-log.log](renderer-log.log); build section preserved separately in [renderer-build-original-excerpt.log](renderer-build-original-excerpt.log).

The job used Ubuntu/Linux, Node `v24.22.1`, branch push scoped as `OCRM_PLUGIN_SCOPE=true` / `validation_scope=ocrm-config`. In repository root it ran:

```text
npm --prefix desktop ci --prefer-offline --no-audit --no-fund
npm --prefix desktop run build:electron -- --plugin-skills-contracts
```

The build log shows actual esbuild output for shipping Electron main/preload, the Coordinator carrier and selected contract bundles. This is shipping esbuild compilation, not a full TypeScript strict typecheck, full renderer build, native Rust execution or packaged acceptance.

The Plugin/MCP command was:

```text
node --test --test-reporter=tap \
  desktop/.tmp-grok-electron/ocrm-plugin/host-plugin-skills-loader.contract.test.cjs \
  desktop/.tmp-grok-electron/ocrm-plugin/loader.contract.test.cjs \
  desktop/.tmp-grok-electron/ocrm-plugin/coordinator-control-server.contract.test.cjs \
  desktop/.tmp-grok-electron/ocrm-plugin/mcp-runtime.contract.test.cjs \
  desktop/.tmp-grok-electron/ocrm-plugin/main-edge-model.contract.test.cjs \
  2>&1 | tee .evidence/ocrm-plugin-edge.tap
```

The carrier environment pointed to the same built `desktop/dist/node-agent-coordinator/main.cjs`. The step completed successfully under `set -euo pipefail`; unlike MAB, the Plugin artifact does not contain separate numeric command/capture exit files.

| Original group | Completed counts | Required positives read back |
| --- | --- | --- |
| Plugin/MCP edge | **148 tests / 148 pass**, zero fail/cancelled/skipped/todo | 144 top-level positives plus 4 nested positives; 11 materialization-custody and 11 router positives each exactly once |
| MAB-v1 attachment binding | **85 / 85**, zero fail/cancelled/skipped/todo | All 85 required names exactly once |
| ONP-v1 notice contracts | **21 / 21**, zero fail/cancelled/skipped/todo | All 21 required names exactly once; notice-only command also exited 0 |

The Plugin count was read from H74 original TAP, **not inherited from H23's 133**. H74 includes the MO-05/MO-06/MO-07 official credential/callback/mutation cases. The 4 nested Plugin positives are `peek-before-logout-login`, `login-before-peek`, `departed`, and `unreadable`.

MAB ran `node --test --test-reporter=tap` on `coordinator-account-runtime.contract.test.cjs`, `coordinator-main-port-client.contract.test.cjs`, `coordinator-runtime.test.cjs`, and `attachment-gateway.contract.test.cjs`, all under `desktop/.tmp-grok-electron/ocrm-plugin/`.

Original MAB exit files establish production Node/capture **0/0**, inherited errexit **1**. Its two shared-wrapper controls preserve **7/0/1** (Node/capture/wrapper) and **0/1/1**, both with inherited errexit 1; their original wrapper-log hashes match the receipt.

Notice commands were `node desktop/scripts/check-electron-runtime-bundle.mjs --notice-only` and `node --test --test-reporter=tap desktop/scripts/openclaw-notice.contract.test.mjs`. Both command/capture exits were **0/0**.

Renderer job steps **15–26 were skipped**, including architecture/boundary checks, TypeScript contract compilation, full renderer typecheck/build, co-staged runtime/bundle checks and final architecture gate. Renderer job success does not make those checks passing.

## Original artifact custody and input identity

| Artifact | ID | Original ZIP SHA-256, matched to GitHub |
| --- | --- | --- |
| Plugin edge | 13206320377 | `6bad6ca91105832529a1dde5d05132731acbff5883da97aa811a6e1d143e766e` |
| MAB | 13205204882 | `71e068a817540f58d9f6ca44f0a58a7766457856a822bcac5d9ce0f5dfa564a9` |
| Notice | 13205785701 | `6f59330ac0fb72e69124dafd6f285046632b3c61ce0665d73b52c509366acd68` |

The untouched original receipts are:

- [Plugin receipt](plugin-edge-artifact/ocrm-plugin-edge.json), with [raw TAP](plugin-edge-artifact/ocrm-plugin-edge.tap).
- [MAB receipt](mab-artifact/receipt.json), [raw TAP](mab-artifact/tests.tap), and original control/exit files in `mab-artifact/`.
- [Notice receipt](notice-artifact/receipt.json), [notice-contract log](notice-artifact/notice-contract.log), and [source-notice log](notice-artifact/source-notice.log).

All three TAP byte sequences match both their original receipt hashes and the corresponding original job-log sections after removal of GitHub timestamp framing only.

| Raw evidence | SHA-256 |
| --- | --- |
| Plugin TAP | `c19add4713a4cc5169bfd6872edd34122713b82dec4ec6921b768b0a97784dd2` |
| MAB TAP | `13d42d7713fb2a74e3c209291c33b24a630b338acc7cf4317f70603930b78e58` |
| Notice contract log | `f2acf084bbba9b4476806936e02709f92fb73f6ece44fd1414f9760eeb0b9198` |
| Notice-only log | `4da2c122733058bc013660d36ec4ae41a29f651da970884f382ed63f4e7a731f` |
| Renderer job log, 202,722 bytes | `676554df0a6df0000d9c1e1bc40959209d034979894b03b835856804b4b55840` |
| Source job log, 28,920 bytes | `31f25cb8af36bfb77ebfbd4179ea5728774379f2cf70f7dddebf2e1f2fc9dd1c` |

Job-log hashes refer to the unchanged UTF-8 bytes of the connector's decoded log content, including the leading BOM. Original ZIP hashes refer to downloaded binary archive bytes.

The receipts declare **20 Plugin, 25 MAB and 9 notice inputs: 48 unique paths**. Every recorded SHA-256 matches bytes retrieved from H74's immutable tree; every computed Git blob identity matches that tree. Three uncached objects were retrieved by exact blob API and checked identically. [source-input-comparison.json](source-input-comparison.json) records every path, Git blob, byte count, SHA-256 and per-receipt comparison.

MAB also records shipping bundle SHA-256 values for main/preload/Coordinator. Those values are original runner-attested receipt fields; bundle bytes are not included in these artifacts, so this assessment does not pretend to have recomputed them independently. Declared-input equality likewise does not establish a complete transitive dependency closure.

## Acceptance boundaries and remaining facts

The original receipts limit these positives to actual existing owners exercised under controlled transport/process/account carriers and real temporary-file I/O where declared. They retain pending independent acceptance and explicitly leave real marketplaces/accounts/providers, process-loss/revocation, real Electron/native binaries, complete Rust permission migration, supported packaged platforms/signing and full G01–G12/AC01–AC24 acceptance open.

Exact-H ledger and RTM both retain all eight global flags as **false**: `baseline_identity_accepted`, `source_closure_ready`, `responsibility_mapping_complete`, `rust_implementation_complete`, `production_wiring_complete`, `ecosystem_compatibility_accepted`, `ui_accepted`, `release_accepted`. Their source Git/blob hashes are retained in [evidence-integrity.json](evidence-integrity.json).

The applicable OCRM §12.2/§12.3/§15.1 boundaries, root instructions, materialization-custody and MAB scoped contracts were read. Supporting source snapshots are retained for provenance; no blanket full-source deep-read claim is made.

**Concrete blocker from this collection:** H74's source structural guard rejects the reverse link for `OCRM-RESP-MEDIA-FINAL-CUSTODY-001`. Preserve renderer successes at H74 while reconciling that governed source accounting failure. Other runtime jobs are owned by the parent assessment and are outside this report's verdict.

## Inventory

[FILE-INVENTORY.tsv](FILE-INVENTORY.tsv) lists every retained review file with byte size and SHA-256, excluding the inventory itself. [raw-evidence-readback.json](raw-evidence-readback.json) preserves literal counts/named-positive occurrence readback; [evidence-integrity.json](evidence-integrity.json) preserves artifact, receipt, log, capture-control and flag identity comparisons. No authenticated download URL is retained in this report or its files.

