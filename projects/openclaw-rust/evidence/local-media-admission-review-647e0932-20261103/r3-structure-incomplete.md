# LFA/FUA R3 focused structure, reuse and performance review

Terminal status: **INCOMPLETE — required blind approval**.

Task: `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, frozen review round 3. Repository: `bhrumom/fabushi-desktop`, branch `codex/openclaw-rust-fusion-chat-20261101-writer`. Lens: structure/reuse/performance. The assigned model and effort were explicitly applied at spawn as `gpt-6.1-sol` / `xhigh`; no fallback is known to this worker.

No source-confirmed blocking structure, reuse or performance defect was found in the focused three-file integration. This statement is source evidence, **not** the required blind approval. While inspecting the supplied, unchanged evidence JSON for source identities and selected coverage, a text search exposed embedded historical R1 finding and root-decision summaries. I immediately disclosed the exposure to the parent. The parent instructed this worker to retain the source checks, mark blind approval incomplete, and leave the required verdict to a fresh replacement with those evidence JSON files excluded. I did not open any other review report or separate root arbitration/decision file. The exposed historical conclusions were not used to establish the findings or identity comparisons below and are not reproduced here.

## Frozen identities and reconstruction

| Subject | Independently checked identity |
| --- | --- |
| R3 exact base/current H | `3438eb5bac755c399c9ec686414bbb4c0816888d` |
| Base tree, from Git and reconstructed from its recursive leaf entries | `b544c1d377ca82618cdadcf5106cb3c10d82411b` |
| R3 full candidate tree, reconstructed from that base plus all 23 frozen path contents/modes | `e9a172d00139550e8a055a61bf06e225b9d4166c` |
| Prior R2 tree, reconstructed from its own exact `0e845ce1679e0cdf95346adc54f5d58bf06464e8` base plus its frozen 22 paths | `a4f1c59eb577a45c1b05da1de0a8f13644dc1a5c` |
| Fixed U in the unchanged exact-base lock | `1440bb1771372334bea23d7642bb52deb7db1845` / tree `90c8aab39972da11cfad220361781d49e1bc6700` |
| R3 manifest SHA-256 | `ba6170be901468a209d64f1e9a837b63f522de5093be1ec32a678ce7de97b532` |
| Complete patch SHA-256 / bytes | `e7601e75db970790c37844c9ed972a56cf04174708461ffc9deb4a1c11642adc` / 184,128 |
| R2-to-R3 integration patch SHA-256 / bytes | `7789887f05eee4f5dc6f3a3801b60ebd716b07ca8df07b9d74a7a8cb0a47c539` / 197,136 |

All 23 candidate paths matched their manifest byte counts, SHA-256 values and Git blob identities. Each base snapshot path matched `git show H:path`, including absence of added paths. Exactly three supplied paths differed from R2: the workflow, portable capture script and parity ledger. The other 20 supplied paths were byte-identical to R2. Full candidate-tree reconstruction also establishes that the overlay retains the current base outside these 23 paths; it does not discard incoming inventory/spec/evidence changes outside the LFA scope.

The candidate tree was not present as a readable object in the foreign Git object store. I therefore calculated the Git tree hash in memory from the read-only base leaf listing and frozen file contents, first checking the same algorithm against the exact base tree. No Git object or index was written. R2 was reconstructed against its own base, rather than incorrectly treating the older candidate as an overlay on the newer base.

| Focused R3 file | Git blob | SHA-256 | Bytes |
| --- | --- | --- | ---: |
| `.github/workflows/rust-desktop-runtime.yml` | `9419b5467e6882a7aeff16c91aa97317a4e001eb` | `6b341356e10d3699634461f62789e3bda98daecd37ff61904788e5f7bcb81e23` | 431,076 |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `dce650d0425bc35db032234753cf5e4ec298896c` | `101b33df807b86e7d9cd1d3544f36c0ac3b05844b8033a8309067f7a9e45d3e4` | 91,609 |
| `projects/openclaw-rust/parity-ledger.json` | `fda6a051b5f15ac379bc289f00290a4c25638d98` | `fa03fe933ca7228dde76428408139d031de90a46e1438a66c9f93d0fe2a81a37` | 1,441,928 |

## Source assessment

**Extraction and existing ownership.** The original incoming portable Python body at exact `0e845ce1679e0cdf95346adc54f5d58bf06464e8` has 997 lines and SHA-256 `8f0044fd5f4d4d4fe6224d1d146ec47ee5e9f7dac3e7ba9722259b4514b1582f`. I removed only the workflow's ten-space YAML indentation for comparison. The incoming H3438 script equals that body plus its own path in the input list. The R3 script differs from the incoming H3438 script only by the two LFA facade required names at lines 558–559 and the SendMessage summary change at line 975 from 15 to 17. Independently, comparison to the complete R2 inline body yields exactly one added line: the script's own input path at line 12. Thus the extraction uses the current body, preserves the LFA additions, and does not revive a stale earlier script.

The workflow's `channel_pending_recovery` step at lines 3594–3604 retains its existing condition, timeout, `TMPDIR`, exact run/attempt palette fixture, Bash shell and `set -euo pipefail`, and invokes the existing script path once. The R2-to-R3 workflow patch has one extraction hunk and no other workflow edit. The script still runs from the repository working directory, uses the same relative source/evidence paths, launches the same ordered Cargo commands, and writes the same receipt/log/export locations. It introduces no production module, crate, runtime, state owner, external dependency or second capture invocation.

**Preserved commands, inputs and oracles.** Portable commands retain the existing locked Host manifest, case ordering, flags and target sets. The literal input paths continue to exist in the candidate source tree; the existing set/sort union and generated-proto discovery remain unchanged. At script lines 4–8 the exact-H assertion and fresh palette-export requirement remain present. At lines 925–945 the receipt still binds target commit/tree, fixed-U fields, run/attempt/job/OS/toolchain, all input hashes, planned commands and synthetic oracle results. The new self-hash input is consumed by the unchanged input-hashing comprehension at line 942.

For the affected generated group, all 44 required names are unique and equal the test-definition union of the frozen SendMessage target (17), exact-base Session conversation target (14), and exact-base generated occurrence codec target (13). The two added facade test definitions exist at `source/host/tests/send_message_tool_contract.rs:651` and `:665`; the positive checks exact native Unicode path and metadata, and the negative checks invalid authorities/encodings. Script lines 963–976 retain per-name uniqueness, command/capture exits, zero failed/ignored summaries, the exact 13/14/17 multiset and total observed-positive cardinality. No assertion or synthetic oracle control was removed. Other inherited portable oracle rules are retained verbatim; this review does not broaden their claims.

**Workflow consumers and App selection.** The workflow retains the two external LFA required names at lines 1844–1845. Its existing Linux App capture retains all six exact LFA names at lines 4160–4169, and these are the exact six test definitions in the frozen App test module. Existing listing/execution at lines 4172–4183 still requires the exact listed test, successful command execution, one passed/zero failed/zero ignored summaries, and complete case accumulation. The input block at lines 3883–3889 still binds the LFA spec, actual App tests, shared converter and Runner/AgentManagement owners.

I read the relevant actual consumers: `source/host/app/src/main.rs:639–704` consumes the shared Result before native read or Box fallback; SendMessage at lines 880–905 and AgentManagement at lines 1175–1189 propagate its error. The selected App tests exercise the real shipping sinks through existing Session/ForeverBox owners, observing custody and rollback rather than substituting a resolver. The six test bodies cover invalid admission, a previously materialized first image followed by refusal, native/Unicode materialization across reopen, repeat refusal and valid recovery, AgentManagement's invalid later image with unchanged state and valid recovery, and HTTPS passthrough. These are test-source observations, not executed results.

**Palette and artifacts.** The existing palette consumer at workflow lines 3625–3639 still reads the same portable producer receipt, requires matching target commit/tree, a passed producer, the exact fixture path/hash, and one observed Rust export test. Producer freshness and output hashing remain at script lines 6–8 and 991–1000. Node setup, dependency installation, consumer/typecheck commands and their conditions are unchanged. The existing Host cache at workflow lines 3284–3290, the pre-App evidence upload at lines 3729–3736, failed-fixture preservation at lines 4364–4371, and final native evidence upload at lines 4372–4379 are unchanged. A failed or incomplete portable execution retains non-passing exit behavior.

**Ledger and source state.** The entire R2-to-R3 ledger delta is `mapped_paths` 877→884 and `unclassified_paths` 61,253→61,246 at lines 15 and 18. The resulting complete `file_coverage` object equals the current incoming base object. Conservation holds: 884 + 61,246 = 62,130. The eight completion flags at lines 2–10 equal the incoming base and remain false. No responsibility, review, implementation, wiring, verified, accepted or evidence-history field changes in this focused ledger delta. This is preservation of incoming declarations, not independent semantic approval of all inventory classifications.

**Performance and maintainability.** R2 workflow size falls from 532,528 to 431,076 bytes, removing 101,452 bytes of embedded YAML text. Relative to current H3438, the workflow adds only the retained LFA integration (2,152 bytes) and the script adds 145 bytes for the two required names. The script invocation replaces the same Python process/body previously fed through stdin; it adds no test command or CI lane over R2. Receipt computation gains one bounded source-file hash. This supports a source-level conclusion that the extraction does not introduce material new runtime/build reach. No timing, memory, power or CI parser acceptance measurement was performed or inferred from the size reduction.

## Authority, coverage and unresolved limits

Governing sources were read from exact Git blobs, without reading the foreign working files: root `AGENTS.md` `e4682748087759b2d4bb2b3a47916b6148e8a6f2`; current authority/constraints in Source of Truth `3748d734b10c9ed5b2d948467fbaa9483f1dd306`; and applicable ownership, evidence, execution and §17 rules in main spec `69c9667af2328e20a1fba09ebf80c5aab4d5c336`. Both frozen LFA/FUA specs and the full native media capture were read. Necessary adjacent workflow, production sink, selected test and palette receipt-consumer sources were inspected. Historical source navigation material was not treated as current product acceptance.

This is a focused review of the three integration files with all 23 supplied paths checked for identity and unchanged product/test context consulted. It is not a replacement for the separate complete R2 product review, full dependency closure, full upstream package reading, or all responsibilities in the large inherited portable suite. Locked URL/dependency behavior was not executed or independently re-proven by this CI-only delta review. The fixed-U identity was checked in the unchanged lock; this worker did not acquire U's complete tree or dependency archives.

Only source text, hashes, immutable Git reads, in-memory comparisons and tree-hash reconstruction were used. No product, tests, builds, lint, capture scripts, schema validators, parser examples, generators or benchmarks were executed locally or on htch. Only this report was written.

R2/R3 have no exact published-H runtime result in this review. The parent-supplied adjacent H `e19a0a265e400a4c9b1d96520c84fb612a3843e8`, runtime run `41571103928`, Windows job `125703932910`, had one rooted-file-URL positive failure (22 passed, 1 failed); its cause is unresolved. I did not acquire or diagnose that log, reclassify it as an R3 result, weaken its assertions, or infer current platform success. Exact-H workflow creation and native/App/portable receipts remain necessary.

Required next review action: obtain the fresh blind focused structure/reuse/performance verdict requested by the parent. No source repair is proposed by this report; no blocker is invented from the independence failure.

**Terminal: INCOMPLETE — independently checked source facts retained; required blind approval withheld.**
