# R9 source recovery carrier — independent fidelity review

## Terminal result

**PASS — artifact fidelity verified.** 55 of 55 artifact consistency checks passed. No fidelity or provenance discrepancy was found. This is a terminal decision about the carrier bytes and the authority boundaries recorded in them. It grants no new source-semantic approval, execution result, unit verification, or OCRM/global acceptance.

Reviewer: `/root/dvr_security_r9`, configured/inherited GPT-6 Astra Pro, `xhigh`, as supplied by the parent assignment. No runtime model introspection is claimed. Observation: `2026-11-03T01:43:05.358437+00:00`. The earlier security report authored by this reviewer was compared for fidelity only.

## Archive and reconstruction

The two binary parts, in declared offset order, reconstruct the archive byte for byte:

- Archive: **621,978 bytes**, SHA-256 `5ce001a1454b4e12452e71ebeea1e021202f6c18a787923de219f4b3df918515`, Git blob `4e80bdc47445a986e020979981af1ba96c1b8a84`.
- Part 1: **384,000 bytes**, offset `0`, SHA-256 `82514d0d39cac56f0bad33cbd529d5593d53533ef59fc9e90909544607db7165`, Git blob `a6bd736e9b2a9e96ea81a95700b2df5de4a9b7e3`.
- Part 2: **237,978 bytes**, offset `384000`, SHA-256 `5c67fc1e3d7b63c538670fe95375725b686500def2067d216d06f5a64b208cce`, Git blob `7b38e003e1f2c3bab844105e703440ce2400dacf`.
- Carrier index SHA-256: `691a89828510ef11984b61b2919d5251c87ce4a8cd21a3ce860360a14e017c3d`.

Every member's independently computed byte count, SHA-256 and Git blob identity agrees with the index. All 19 members also equal their original scratch files at this observation; there is no live-original drift to qualify. The tar contains only regular files with unique relative names, no traversal, and UTF-8 lexicographic ordering. Headers use mtime/uid/gid zero, mode `0644`, empty owner/group names; the gzip header has mtime zero and no original filename. These checks concern the preserved archive, not a newly generated archive.

| Member relative to scratch | Bytes | Index and original comparison |
| --- | ---: | --- |
| `ci-recovery/dvr-exact/r9-input-identities.json` | 68,665 | Exact |
| `ci-recovery/dvr-exact/r9-manifest-check.json` | 5,917 | Exact |
| `ci-recovery/dvr-exact/r9-rebinding.json` | 3,192 | Exact |
| `ci-recovery/dvr-exact/r9-source-expectations.json` | 139,461 | Exact |
| `reviews/dvr-ci-011-decision.json` | 2,240 | Exact |
| `reviews/dvr-ci-012-decision.json` | 2,150 | Exact |
| `reviews/dvr-decisions.json` | 90,412 | Exact |
| `reviews/dvr-r9-security-review.md` | 17,647 | Exact |
| `reviews/dvr-r9-structure-review.md` | 16,812 | Exact |
| `reviews/dvr-r9/api-readback.json` | 2,394,153 | Exact |
| `reviews/dvr-r9/complete.patch` | 471,322 | Exact |
| `reviews/dvr-r9/fix-delta.patch` | 12,303 | Exact |
| `reviews/dvr-r9/manifest.json` | 6,319 | Exact |
| `reviews/dvr-r9/paths.txt` | 809 | Exact |
| `reviews/dvr-r9/publication.json` | 5,636 | Exact |
| `reviews/dvr-r9/publication.patch` | 12,303 | Exact |
| `reviews/freeze_dvr_r9.py` | 2,374 | Exact |
| `reviews/import_dvr_r9_publication.py` | 3,640 | Exact |
| `state/requirement-checklist.json` | 42,908 | Exact |

## Source and publication provenance

The preserved API response contains all **15 full UTF-8 file contents, totaling 2,304,656 bytes**. Independently re-encoding each content string reproduces its declared Git blob identity and the full byte/SHA-256/blob identities in the frozen manifest, publication record and manifest-check record. The paths text agrees with the manifest in order. All 277 selected execution-input identities match the union of source-selection input paths; the 14 paths overlapping the review manifest match as well. The RTM is outside that execution-input selection and remains covered by its full API content and the other three identity records.

The recorded API commit and branch observation consistently bind recovery H `5593e6d96e849ce5bf93de299aedeffca70b9568` to tree `17ddcd3b19126ece3cadb722ea7b9bbdc5dcb561`, parent `2ff201200b01e3d4b1dc74c816c2e610fea2e2d0`, and fixed upstream U `1440bb1771372334bea23d7642bb52deb7db1845`. Publication equals the copy preserved in the root decision. Manifest equals the root's R9 snapshot. Both review report hashes equal the R9 root adjudication entries, and both separate CI decision JSON files equal their root-decision copies. The two focused patches are identical at 12,303 bytes, and the complete 471,322-byte patch matches its frozen identity. The four rebinding paths and their resulting identities match the focused delta and manifest.

This independently verifies the full contents retained in the connector readback. It does not make a fresh GitHub request or claim the branch still points to that commit. Root report-reading assertions and supplied launch metadata remain recorded attestations. The publication's `verified` flag describes content/identity comparison; the API commit's signature record remains `verified: false`, reason `unsigned`.

## Historical snapshots and acceptance boundaries

The source-expectation, source-input and rebinding records intentionally preserve their pre-publication null commit fields and source-only/pending-execution status. Their tree equals the subsequently published H5593 tree. Recorded chronology is coherent: freeze, root source decision, commit, publication comparison, checklist observation, carrier preparation.

The checklist is its `2026-11-03T01:37:44.748525+00:00` task-local snapshot. Its pending carrier-publication statement is historical, and its stated authority remains the canonical OCRM ledger/inventory/RTM. All eight completion flags are false. Root convergence and carrier flags retain false unit verification, executable acceptance and global completion. The two source reviews remain focused source verdicts with exact-subject Actions evidence still required; their unchanged-scope limitations survive serialization.

Original failed H `8bdcf1012c15c08a0320c85c9d4d42e1cc100949` remains a separate subject. Both CI decisions retain original runtime run **41559093519**, Linux job **125665070447**, and their original artifact IDs/digests. These references match the sealed original evidence index (SHA-256 `23cbad8efb3a128f2bebc117bf0fb4f16fac10f0ec31a63d892f50c58ed7709e`). At its `2026-11-03 01:31:25 UTC` observation, Linux was completed/failure and macOS job **125665070217** was queued. Original source-run success was structural only. No original failure or retained positive is relabeled as H5593 execution, and this review grants neither subject acceptance.

## Read and action boundary

The entire carrier index, archive, two parts, all 19 decoded member byte streams and all 19 original file byte streams were read for integrity comparison. All 11 JSON members were parsed. Human provenance reading covered both reports' verdict/launch/scope/limits sections, R9 root decisions, CI decisions, publication, and task-local status. The sealed original evidence index was parsed for identity, observation, jobs and artifact cross-references. Full source readback was a byte-identity operation; it was not a new semantic review.

No product code, archived script, test, build, formatter, validator or generator was executed. No source, archive, ledger, GitHub or repository state was changed. Only this report and its companion JSON were written. The JSON contains every member identity, all full-content identities, every check, recorded chronology and limitations.
