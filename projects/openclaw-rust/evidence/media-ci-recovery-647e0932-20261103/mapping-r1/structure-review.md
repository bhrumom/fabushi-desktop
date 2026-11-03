# Media mapping recovery R1 — independent structure, reuse and performance review

**Verdict: APPROVE for the frozen source change. No blocking or non-blocking findings.** This verdict assesses the declared metadata repair and its existing consumers. It is not an Actions result, source-review completion, product verification, or final acceptance.

## Reviewer and limits

- Reviewer task: `/root/media_mapping_structure_r1`.
- Assigned and applied worker settings: `gpt-6.1-sol`, `reasoning_effort=xhigh`, `fork_turns=none`. The parent confirmed these explicit spawn API settings during this review. No model/effort fallback or global/session-setting change was used.
- Scope reason for xhigh: independent source verdict across the exact source ledger, compressed canonical inventory, pre-expansion exception, deterministic materializer, and ordinary/final gate boundaries; a small data diff still interacts with identity and completion authority.
- Fresh blind review: no peer reports or current runner decisions were inspected. Reviewed source was read from the immutable candidate tree. Repository source and staging state were not modified. The only authored artifact is this report.
- Read-only instruments: Git object/diff inspection, text reads/searches, JSON/gzip evidence parsing, and SHA-256 calculation. No repository code was imported or executed. No build, test, lint, validator, generator, harness, benchmark, schema check, `htch-runtime` validation, or local product execution was run.
- This is the structure/reuse/performance lens for this one metadata change. It does not reopen media product equivalence, other media repairs, or DFA work.

## Frozen identities

| Item | Exact identity |
| --- | --- |
| Repository root | `/workspace/scratch/29e7135d8bfe/fabushi-desktop` |
| Manifest read | `/workspace/scratch/29e7135d8bfe/reviews/media-mapping-recovery-r1/manifest.json` |
| Base and starting HEAD | `4f0639321c4ae03cc7dbef1d8e49dea0f042e180` |
| Base tree, resolved read-only | `cbc9418ee7572f44d59e067e450a67f6c0bc22a0` |
| Candidate tree | `1e1348964e4356a76de0f61fd6557a7fbe6d0d6b` |
| Sole changed path | `projects/openclaw-rust/inventory/index.json` |
| Base index blob | `a4f62bdd60e624b630b0e735bb42e020e8f82a35` |
| Candidate index blob | `6905d89a058a59f09a2ec3f5b3b4afc6f751919d` |
| Candidate index bytes / SHA-256 | `2972` / `88bca2d7eab937e28a9197f8576098cc6cffbaae43d9f7f1b050c2060df514e4` |
| Binary-safe full-index diff bytes / SHA-256 | `1671` / `135d1702e7ae28c459ba9b9088b34298c86aa5a38a5bddb76f496bf00b21aa92` |
| Fixed upstream | `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Prior canonical inventory SHA-256 | `8994988c264876be709930e2f87c6b60ae124a6077a833ccd324dc1117d26d93` |
| Reported failing source workflow | `41580539540` |
| Failed workflow subject | `74f11d3d31c6245ac5886dc0181460b2d4349744` |

The read-only complete base-to-candidate name/status diff contains exactly the single modified index path, with 30 insertions and no deletions. The index and patch byte/hash results match the supplied immutable manifest. The candidate is a Git tree, not a newly verified subject commit.

## Applicable authority and evidence read

Read the repository `AGENTS.md` first, including the independent-review, model/effort and OCRM Actions-only rules; the latest OCRM Spec §17, especially §17.1–§17.2 and §17.4–§17.5; the relevant source-of-truth, module-map and P0 task context. Spec §17.2 requires the existing inventory/ledger/RTM to remain the sole accounting truth, literal read facts to remain distinct, and machine declarations to grant only their documented structural authority.

The original source log was read at `/workspace/scratch/29e7135d8bfe/ci-recovery/mcp-official-exact/74-source-authority-original.log`. It binds checkout and environment to subject `74f11d3…` and the fixed U/tree, records 33 source-review contract tests and 21 materialization contract tests as passing on that historical subject, and then fails `verify_audit_integrity.py --before-expansion` with `Missing reverse source/responsibility link: OCRM-RESP-MEDIA-FINAL-CUSTODY-001` (log lines 324–338). `set -euo pipefail` stops that step before expansion. Those historical passes are not execution evidence for the candidate.

Read the candidate's canonical shard manifest and the five complete rows in shards `part-0036.jsonl.gz`, `part-0043.jsonl.gz`, and `part-0044.jsonl.gz`; the complete `OCRM-RESP-MEDIA-FINAL-CUSTODY-001` responsibility; the pre-existing media read-scope link; and the adjacent contracts in `source_review.py`, `verify_audit_integrity.py`, `expand_inventory.py`, `shard_inventory.py`, `validate_p0.py`, both existing contract test files, and `.github/workflows/ocrm-p0-authority.yml`.

For identity evidence only, the unchanged tracked shard records were decompressed and their canonical JSON representation was hashed. The reconstructed 62,130-row document (62,128 blobs, two symlinks, zero gitlinks) has SHA-256 `8994988c264876be709930e2f87c6b60ae124a6077a833ccd324dc1117d26d93`, matching both the manifest and pending declaration. This was evidence parsing, not invocation of the project's integrity validator or materializer.

## Source and responsibility correspondence

Every added triple uses `OCRM-RESP-MEDIA-FINAL-CUSTODY-001`. The following is exactly that responsibility's existing five-member `source_refs` set; no new source reference, source range, source-read observation, or responsibility is introduced.

| Fixed-U source path | Exact blob SHA | Existing canonical row state |
| --- | --- | --- |
| `src/gateway/agent-turn/agent-run-final-media.ts` | `f4bf5b4308cb2a7160ad1db0142160d14fb71551` | unclassified; no responsibility IDs |
| `src/gateway/agent-turn/agent-run-media.ts` | `27a3a65974fad078872b257ee28cafccdad4691f` | mapped to `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001` |
| `src/infra/sqlite-worker-error.ts` | `b13f8354e7244d630ee274b3b0debf9a3f5bb8c6` | unclassified; no responsibility IDs |
| `src/gateway/managed-image-attachments.custody.ts` | `1975104ea6cdcfea25d3bc4362edbaea5e673adf` | unclassified; no responsibility IDs |
| `src/media/store.read-scope.ts` | `73515a9ed6a2dc9f1a0adddc638d542ad1323ee5` | unclassified; no responsibility IDs |

All five rows identify the same repository/U/tree, have `sha == blob_sha`, are ordinary `100644` blobs, and retain `read_status=unread`, empty read ranges, null reviewer, empty evidence references, unknown category/risk, deep required review, and pending review/disposition. The final-custody responsibility remains `mapping=mapped`, `read=partial`, `implementation=implementing`, `wiring=wired`, `verification=not-run`, and `acceptance=pending`, with its existing broader-equivalence and exact-H evidence blockers. The index addition does not change any of those claims.

The selected shard bytes match their manifest hashes:

| Shard | Git blob | Compressed SHA-256 | Uncompressed SHA-256 |
| --- | --- | --- | --- |
| `part-0036.jsonl.gz` | `a41302e3cb662b4f0b17c84fca8a59205ebb4054` | `836261857bf189bcb4584d552f0cb4f67fffb9b5f36ff9aabea067bc1240c480` | `8dd58c1716949ef1a254bef832256a2e200dc6dec800934d63524758e4d175c1` |
| `part-0043.jsonl.gz` | `6642a2b5abd7e4fdd9f477192e655b9c6229fbea` | `2b142113fa92ec877cd1f1f01c2e3879f24244adf39faa2a93d54bd401b7e85b` | `40a0bcfebcae84bf67e1f8c5811714878f6aecffacfe114e35b369e83122efa6` |
| `part-0044.jsonl.gz` | `a764a21974401c0b0353c4ba3904c51da0d9af32` | `c679e7f417903b4f84268789014f396a26fb798946221955bfb97c30ee3b3edf` | `80f291a85dd3387589d8b323b9b9e373f2c16d5a1e6299c6cc67690fefebbe55` |

## Structure, reuse and completion boundaries

**Existing mechanism and owner are reused correctly.** Index lines 38–67 add only the already-supported `pending_mapping_additions` declaration. `source_review.py:51` requires the precise declaration and entry key sets, a nonempty list, the unchanged canonical digest, nonempty strings and unique triples. `source_review.py:73` requires source path/blob correspondence and limits pre-expansion missing links to `mapping=mapped`, `verification=not-run`, `acceptance=pending`. It then requires the pending set to equal actual missing links, rather than accepting surplus declarations. This repair supplies the five missing final-custody triples without replacing the canonical ledger or inventing another inventory.

**The normal gate remains strict.** `verify_audit_integrity.py:98` reads the anchored declaration, refuses unmaterialized additions outside `--before-expansion`, and calls the existing reverse-link contract. The base-to-candidate diff changes no gate, test, workflow or schema. `validate_p0.py:208` still requires real reverse edges after expansion, and its final mode still requires reviewed responsibility/source facts. No status or flag bypass is added.

**Materialization preserves existing mappings and read history.** `expand_inventory.py:106` derives responsibility membership from exact ledger path/blob keys. Lines 150–160 preserve literal read and existing responsibility state, retain conservative review fields, and union ledger IDs with existing IDs. Consequently, the already-mapped `agent-run-media.ts` retains `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001` while gaining the final-custody ID; it is not counted as a newly mapped path. The other four rows can change from unclassified to mapped when Actions materializes this delta. None becomes reviewed or read-complete through this declaration.

**The pending declaration is consumed only after materialization.** `shard_inventory.py:90` checks that each requested path/blob/ID is present in the actual canonical output before replacing shard content; line 136 removes the consumed declaration. Lines 149–163 derive mapping and unread projections from materialized rows. Keeping the present index at 62,130 tracked, 61,246 unclassified and 62,067 unread/partial is honest before materialization. The patch does not hand-edit the mapped denominator, read counts, latest exact-head evidence, upstream identity, or any closure/acceptance flag. All eight ledger completion flags remain false.

**Actions owns the writes and exact-subject evidence.** The unchanged workflow checks out and verifies the exact subject, locks U/tree, executes the pre-expansion check before expansion, performs the ordinary checks after shard writing, and persists only the exact-subject verified artifact through its existing guarded branch update. The immutable metadata candidate alone supplies no resulting materialized H or successful receipt.

**Existing negative and materialization oracles cover this mechanism.** The inspected `ForwardMappingContract` tests cover pre-expansion-only permission, no promotion of review/verification/acceptance, path/blob drift and unknown sources, preservation of historical links, digest drift, duplicate and unused declarations. The two pending-mapping tests in `test_inventory_materialization.py` exercise the real expansion/shard pipeline: consumption only after the exact reverse link exists, preservation of unread/pending facts, and refusal to consume an unresolved addition without replacing committed shards. These are existing tests, read as source; they were not run for this candidate in this review.

**Performance is proportionate to this changed surface.** The patch adds five declarative triples to an existing Actions-only metadata pass, with no product runtime, dependency, asynchronous work, I/O owner, or algorithm change. Existing set membership and exact-key map lookups consume the declaration; the full inventory traversal and deterministic shard compression already belong to the same unchanged materializer. There is no changed runtime-performance behavior to benchmark and no introduced structure/reuse/performance defect demonstrated by the source.

## Remaining evidence boundary

The repair is source-correct for the declared failure and is ready for the existing exact-subject Actions path. Executable materialization and structural checks remain **not run for this candidate** in this review. A future Actions success must be attributed to its actual frozen subject, and any generated commit requires its own exact-head evidence under the existing workflow. The report makes no claim that other later gates, broader media equivalence, risk-depth review, source closure, production acceptance, or final OCRM acceptance are complete.
