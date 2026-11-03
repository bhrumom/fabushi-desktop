# R6 exact-H source-authority Actions evidence assessment

## Terminal evidence verdict

**The exact-H source-authority tracked-tree and structural gates passed, and collection of this run’s source-authority and inventory-persistence evidence is complete.** Run `41586608762`, attempt `1`, finished with `success` on H `8621173a53d1f46f7dc613c1b2e162c3350b94bc`. Its source-authority job reported **33 risk/source-review contract tests**, **21 inventory-materialization tests**, four rejected structural counterexamples, and successful G01 tracked-tree / G02 structural checks. The downloaded artifact ZIP, API metadata, source upload log and persistence download log agree on artifact identity and SHA-256.

The receipt’s 26 repository-file hashes all match the corresponding exact-H Git blobs and bytes. Its additional canonical-inventory output hash also matches the complete downloaded output. The receipt is produced after expansion/materialization, so I checked those output-phase hashes against the committed inputs rather than assuming they were checkout hashes. In this run they are identical, including the ledger, index and shard manifest. The persistence job reports **“No new source-derived shard content”**; it produced no generated carrier commit and its conditional dispatch step was skipped.

This assessment does **not** grant runtime/media/App acceptance, dependency or LFS closure, final AC02, or global OCRM acceptance. The MSU responsibility remains `read: partial`, `implementation: implementing`, `verification: not-run`, `acceptance: pending`; all eight global completion flags remain `false`. Runtime run `41586608776` is owned by the parent and was not queried or assessed here. The completed source-review report `/workspace/scratch/7d27ea239237/media-settlement-r6-general.md` was not modified.

## Scope, authority and exact identity

This is the Actions evidence phase for task `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, R6. It is not a new source-review round. The full evidence-assessment scope was classified `xhigh`; this worker retained its existing actual model/session configuration and made no model or effort override. I followed the previously read authority at `/workspace/scratch/7d27ea239237/media-settlement-authority/AGENTS.md` and the parent’s phase restriction to source/Git/API/JSON/ZIP/hash evidence work.

| Identity | Value |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| Existing branch | `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Exact tested H | `8621173a53d1f46f7dc613c1b2e162c3350b94bc` |
| H tree | `077065fdeeb9aa17213a4f6d2dece8cba168fbf5` |
| Prior frozen R6 source-review tree | `4b94e525d6790d6f355ea6e2113db02cb5e5a46f` |
| Fixed U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed-U tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Workflow | `.github/workflows/ocrm-p0-authority.yml` |
| Workflow blob at H | `2d22dce2d6067e7f4c16c3c01ed95c58cd3354df` |
| Workflow SHA-256 | `15e7cd4e6402dd515bbd023c2b421162fb854e13c8744ba6abec8f7fc33fc86e` |
| Workflow ID / run number | `392577148` / `350` |
| Run / attempt / event | `41586608762` / `1` / `push` |
| Run created / started | `2026-11-03T05:54:20Z` |
| Runner reported by source job | Ubuntu `26.04.1`, image `ubuntu-26.04`, image version `20261025.179.1` |
| Receipt interpreter / platform | Python `3.14.4`; `Linux-7.0.0-1017-azure-x86_64-with-glibc2.43` |

The run URL is <https://github.com/bhrumom/fabushi-desktop/actions/runs/41586608762>. The exact commit API record and the untruncated recursive H tree are retained under `api/`. The H tree response contains 11,351 entries; it binds the repository-file identities used below. The source log independently records checkout of H and acquisition/identity checks of fixed U. I did not substitute a later branch head for the tested subject.

### Bounded status observations

| Observation | UTC time | Observed result |
| --- | --- | --- |
| 1 | First jobs read `05:55:33`; initial metadata collection completed by `05:55:45` | Run and source-authority job `in_progress`; G01/G02 step running. Recovery job already `skipped`. |
| 2 | `05:56:47` | Run `completed/success`; all three job records terminal. |

The second observation was 74 seconds after the first timestamp and more than 60 seconds after completion of the first collection. Only two of the allowed three observations were used. No further polling was needed after the terminal result. No single blocking wait exceeded 60 seconds.

| Job | ID | Terminal conclusion | Completed at UTC | Treatment |
| --- | --- | --- | --- | --- |
| `source-authority` | `125753466394` | `success` | `2026-11-03T05:56:17Z` | Assessed against complete raw log and artifact. |
| `persist-verified-inventory` | `125754056567` | `success` | `2026-11-03T05:56:32Z` | Assessed against complete raw log and exact source/persistence inputs. |
| `recover-original-host-log` | `125753467693` | `skipped` | `2026-11-03T05:54:21Z` | No execution/pass claim. |

The terminal jobs response is the attempt-specific API endpoint with `per_page=100`; its `total_count` is three and all three records were retained. The artifacts endpoint reports exactly one artifact, which was collected. This phase did not dispatch, rerun, cancel or push anything.

## Raw evidence collection and byte identities

All evidence paths below are relative to `/workspace/scratch/7d27ea239237/media-r6-source-ci/`. API response content was saved as returned UTF-8 JSON text. Job logs were returned by the GitHub connector as decoded text and saved in UTF-8 with the initial BOM preserved; the reported log hashes identify those complete saved log bytes. The ZIP was downloaded through the connector’s supported reusable file reference and retained as the original ZIP bytes.

| Artifact field | Value |
| --- | --- |
| Artifact ID | `13207619380` |
| Name | `ocrm-p0-authority-8621173a53d1f46f7dc613c1b2e162c3350b94bc` |
| Workflow run / head SHA | `41586608762` / H |
| ZIP bytes | `4,951,968` |
| ZIP SHA-256 | `75178fb1e14b9f4219b0417e2f5fa917fd2a16347a3862e1c3ebc76eea507e23` |
| Created / updated | `2026-11-03T05:56:14Z` |
| Expires / expired at collection | `2026-12-03T05:56:12Z` / `false` |
| ZIP path | `artifacts/source-authority-13207619380.zip` |
| Uncompressed member directory | `artifacts/source-authority-13207619380/` |

Source-log lines 353–364 bind the eight uploaded files, artifact ID, ZIP byte count and digest. Persistence-log lines 125–144 bind the same artifact download and digest. The locally downloaded ZIP matches both logs and the complete artifact API metadata. Its eight members were safely read and retained; member byte counts, CRCs, compressed sizes and SHA-256 values are recorded in `artifacts/source-authority-13207619380-manifest.json`.

| Complete log | Lines | Bytes | SHA-256 |
| --- | ---: | ---: | --- |
| `logs/source-authority-job-125753466394.log` | 380 | 31,514 | `838e6ea2ae35f184417032906e0050d92f9ca833f8dd6a8d568d96837103ca51` |
| `logs/persist-inventory-job-125754056567.log` | 216 | 18,126 | `ac55a0f511d3e9c0967b2d9dfe4372f87390d5c551c19b253166d92fe606da4a` |

| ZIP member | Bytes | SHA-256 |
| --- | ---: | --- |
| `authority-summary.json` | 607 | `525cf85ad14c5ccffdcb3e27a4eabdf845d7bba39c8f43add759a9e929ceb497` |
| `dependency-root-inputs.tsv` | 171 | `dd26a8316202da93dbca27ab63cd046c365ac2db499de18af844e51f654806f4` |
| `lfs-pointer-candidates.count` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| `lfs-pointer-candidates.txt` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `ls-tree.z` | 6,271,094 | `ad8da8a294e1f3bbb98f4f711163e84b1a416b8bfa1745db76b027cee2bc52f0` |
| `p0-structural-receipt.json` | 5,080 | `5b15f0acf53720dba3bbc9b99c78df0a5296cd121808ee14223513e33dfacca7` |
| `upstream-recursive-inventory.json` | 55,595,275 | `d0049087bdbcc8eecdc8d0a69bd48bfc7d70688f8917122198a6e15c80863d8a` |
| `upstream-recursive-inventory.sha256` | 65 | `4acc8ff9da82505ebd5006c8038c46324893975b9a87ef517ee727b94bfc5823` |

`collection-manifest.json` records paths, sizes and SHA-256 values for the collected evidence files and this report, with explicit exclusion of the manifest’s own hash. `evidence-assessment.json` preserves the machine-readable identity, job/step outcomes, read boundaries and terminal assessment. Temporary signed download URLs are not needed for identity and were not retained as durable authority.

## What actually executed in Actions

The following findings come from the complete source log and terminal step metadata, not local execution. The log records the exact commands before their output. `test_source_review.py` is a source-accounting/risk-contract suite, and `test_inventory_materialization.py` exercises synthetic inventory accounting through the actual expansion/sharding pipeline; these are not Rust media or App tests.

| Command / operation | Remote evidence | Bounded result |
| --- | --- | --- |
| `python3 projects/openclaw-rust/scripts/test_source_review.py` | Source log 163–177 | `Ran 33 tests in 0.003s`, `OK`; terminal step success. |
| Acquire fixed U and compare commit/tree | Source log 178–210 | Exact U fetched; identity checks completed. |
| Recursive `git ls-tree -r -z --full-tree` enumeration | Source log 211–261; raw `ls-tree.z` and summary | 62,130 tracked paths: 62,128 blobs, two symlinks, zero gitlinks. |
| LFS candidate scan and root dependency identity recording | Source log 262–291; corresponding artifact members | Zero observed pointer candidates; three root dependency inputs recorded. Closure remains open. |
| Install `jsonschema==4.25.1` | Source log 292–321 | Installation completed in Actions. No local package install or schema execution. |
| `python3 projects/openclaw-rust/scripts/test_inventory_materialization.py` | Source log 295, 322–326 | `Ran 21 tests in 4.801s`, `OK`; encompassing fail-fast step success. |
| `verify_audit_integrity.py --before-expansion` | Source log 297, 327 | Integrity PASS: 62,130 paths, 888 mapped, 62,067 unread/partial, zero pending forward links. |
| `expand_inventory.py ocrm-evidence/upstream-recursive-inventory.json` | Source log 298, 328 | Canonical candidate `d0049087…`; source closure explicitly open. |
| `shard_inventory.py --write ocrm-evidence/upstream-recursive-inventory.json` | Source log 301, 329–330 | Committed-shard identity/reverse reconstruction PASS; 61 shard candidates; dependencies unresolved. |
| `validate_p0.py ocrm-evidence --negative-probes` | Source log 302, 331–333 | G01 tracked-tree and G02 structural PASS, four counterexamples rejected. |
| `verify_audit_integrity.py` | Source log 303, 334 | Committed integrity PASS with the same accounting totals and zero pending forward links. |

The source suite runs under `bash -e`; the materialization/gate sequence runs under `set -euo pipefail`. Successful terminal step metadata, `OK` summaries and continuation through the full sequence establish successful command completion. The log does not supply a separate numeric exit-code file; any `0` exit interpretation is an inference from that success evidence, not an invented raw numeric record. The unittest output is aggregate, so this report does not manufacture per-test-name result lines or additional subtest counts.

I read the complete exact-H bodies of both test files and `validate_p0.py` to interpret those results. The 33-test suite retains identity/membership/evidence rejection, risk-proportionate depth and generated/dependency-role controls, plus pending-forward-mapping constraints. The 21-test suite retains committed-history preservation, source/shard/canonical identity rejection, literal-read versus review separation, idempotent history controls, and exact pending-link materialization. The complete source files are retained under `inputs/exact-h/`; their Git blobs and hashes match H.

The four additional G02 probes are explicitly reported at source-log line 332:

- Duplicate source path.
- Unread denominator tampering.
- Unbacked accepted verdict.
- Orphan responsibility.

`validate_p0.py:277–313` runs the valid structural baseline first, mutates each case separately, refuses a false-negative sentinel, restores the mutated state and only then emits the rejection summary. These checks establish rejection of the supplied counterexamples; the log does not record the individual internal rejection message, and I make no claim about which internal guard was first to reject each case. G02 final mode was not run.

## Committed inputs, generated outputs and carrier distinction

The structural receipt identifies H/tree, U/tree, run, attempt and `source-authority`. It contains **27 hashed paths: 26 repository paths plus one generated canonical-inventory output**. I retained all 26 repository files, recomputed their Git blob identities against the untruncated H tree and their SHA-256 values against the receipt. Every comparison matches. Existing immutable bytes were reused only after their computed blob matched H; the exact H workflow and shard manifest were also obtained through the repository file API. The per-file acquisition provenance, full path, H blob, size and both hashes are in `inputs/source-receipt-input-index.json`.

The receipt digest is `e0af690266d3b7509fdb25d409f3e64d4633c9df7b5fb9be8ef42ba1531527ec`. Recomputing the documented sorted-JSON digest of its hash map produces that exact value. The canonical inventory is fully hashed and matches its `.sha256` file, authority summary, structural receipt and committed shard manifest.

The hash collection in `validate_p0.py:322–363` happens **after** the workflow’s expansion/shard writes. This distinction matters when interpreting `subject_inputs`. The following current-run equality was checked directly:

| Path under `projects/openclaw-rust/` | H blob | SHA-256 of H bytes and receipt-phase bytes |
| --- | --- | --- |
| `parity-ledger.json` | `285f9ca672947d9ad054832557c0b4abf38a0d68` | `d8b197d9e9bf18b5a7f1515116ef524bcf20d518382618cab1f9446c3ae70c8b` |
| `inventory/index.json` | `aa5ef87202903d9ba344a4e1e7fbe6ffedb04772` | `b1605cfab7367390177f63c25d6264b575ace04813bc4a3999f876ec1e5f994a` |
| `inventory/shards/manifest.json` | `911019b1c4c3b629a4425610bf941caf88a97e18` | `9b4dc21b09d80a94259d0688b552c920f41fb93363589235ba4d14a55470b6d3` |
| `requirements-traceability-matrix.json` | `1bdbd1ca17b2ff334f365183c51bcd15c7ca0871` | `e724440db1cb0391d4512c03f8fab5f7e3a08f5762e35c42c82b41f3fa271961` |
| `dependencies/source-manifest-inputs.json` | `ca14f1db252329d98780f318729ef18dd76e9ae7` | `b16d26dbbaa63ede053231e615463d63c82351e68c8ad9761094d16d8bc8b46e` |

The original ZIP contains the canonical inventory and receipts, not separate ledger/index/manifest copies. The output-phase equality above therefore rests on the exact receipt digests plus independently acquired H bytes. I did not locally rerun the expansion or shard generator to manufacture output copies. The persistence job downloads the same exact-H artifact, checks its subject and canonical digest, then materializes the shards in Actions. Persistence-log line 200 is the actual no-change result; its preceding script text includes a potential commit/push path, but that path did not execute. Terminal step metadata marks the subsequent dispatch step `skipped`.

Thus this run has no new generated carrier SHA to substitute for H. A generated commit in another run would be a different evidence subject and would require its own exact-head receipt. Historical Hfc metadata and artifacts were used only as read-only acquisition precedents or as sources of bytes independently matched to H; their earlier verdicts and identities were not rewritten or reused as this run’s execution result.

## Preserved incomplete states

The source receipt reports:

| Gate | Recorded verdict |
| --- | --- |
| `OCRM-G01-tracked-tree` | `success` |
| `OCRM-G01-dependency-LFS-closure` | `open` |
| `OCRM-G02-structural` | `success` |
| `OCRM-G02-final` | `not-run` |
| Receipt `independent_review` | `pending` |

The index and ledger retain 62,130 tracked paths, 888 mapped paths, 61,242 unclassified paths, and 62,067 unread/partial paths. The index records zero applicability-assessed and zero dispositioned paths, with `semantic_read_complete`, `responsibility_decomposition_complete`, `applicability_disposition_closure`, dependency closure and source closure all false. These are source-accounting states; successful enumeration or materialization does not turn them into semantic review or product acceptance.

`OCRM-RESP-MEDIA-FINAL-CUSTODY-001` retains:

| Dimension | Status |
| --- | --- |
| Read | `partial` |
| Mapping | `mapped` |
| Implementation | `implementing` |
| Wiring | `wired` |
| Verification | `not-run` |
| Acceptance | `pending` |

The receipt, committed ledger and RTM all keep these eight global flags false: `baseline_identity_accepted`, `ecosystem_compatibility_accepted`, `production_wiring_complete`, `release_accepted`, `responsibility_mapping_complete`, `rust_implementation_complete`, `source_closure_ready`, and `ui_accepted`. The fixed-U lock also leaves implementation-round/baseline identity acceptance false. No ledger, index, workflow or source file was changed by this collector.

## Read depth and practical limits

I read both complete saved job logs, all run/job/artifact metadata used for the terminal result, the complete structural receipt and authority summary, the lock and index, the manifest’s accounting fields, the ledger/RTM completion states and MSU responsibility, both test files, the structural validator, and the workflow sections executed by the source and persistence jobs. All 26 receipt repository inputs were completely acquired and byte-hashed, which is distinct from a new semantic review of every file. The large ledger, RTM and referenced evidence documents were inspected at the fields relevant to this evidence binding; they were not reread in full as an independent source review.

The 55.6 MB canonical inventory and 6.3 MB raw tree were completely retained and byte-hashed. Their 62,130 source records were not each semantically reviewed here. The 61 compressed repository shards were not independently reconstructed locally; their reconstruction/identity PASS is the exact-H Actions result, bound by the manifest and artifact. The receipt’s declared input set is bound, but this report does not claim a separately audited full transitive Python/package dependency closure. The historical recovery job was skipped and its unrelated recovery implementation was not reviewed as executed code.

All local processing was limited to evidence acquisition, source/log reading, JSON serialization/accounting, ZIP extraction and cryptographic hashing. No cargo, Rust compiler, test runner, test-list operation, source parser, schema validator, project generator, capture script, lint, benchmark or acceptance program was executed locally. The only writes were the assigned evidence directory and this report. No API mutation was invoked.

**Final disposition:** retain the exact-H source-authority structural pass and its complete bound evidence. Keep runtime acceptance, source/dependency closure, final AC02 and all global completion claims separate and unpromoted.

