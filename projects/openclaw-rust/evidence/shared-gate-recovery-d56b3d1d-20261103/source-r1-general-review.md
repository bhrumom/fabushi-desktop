# General / source-integrity review — shared-source-74, round 1

## Verdict

**No actionable findings in the complete declared delta.** The five `pending_mapping_additions` are the exact five reverse links missing from the existing canonical inventory for `OCRM-RESP-MEDIA-FINAL-CUSTODY-001`. Their digest anchor, source identities, responsibility status and count semantics agree with the existing pre-expansion, expansion, sharding and strict-consumer contracts.

This is a read-only source-contract assessment. It is **not** an executed validator, schema-check, test, build, Actions-success, semantic-equivalence or acceptance result. The declaration snapshot remains intentionally unmaterialized; the ordinary strict committed-inventory gate must reject it until the existing Actions materialization removes the declaration and persists the reverse links. The resulting materialized commit requires its own exact-subject evidence.

## Frozen scope and identity

| Item | Reviewed identity |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| Read-only bare source | `/tmp/ocrm-source-84120837.git` |
| Frozen base / authoring checkpoint | `b15e58957a597997363e1273f2f32a222365c310` |
| Immutable review snapshot | `87012a9cc62b1102bb3fe95ff759747aff9beee0` |
| Snapshot tree | `e8ee7c61202c5228545f90572266bd6797d12acb` |
| Entire changed scope | `projects/openclaw-rust/inventory/index.json` |
| Reviewed blob | `60a9159101b5eb8ce42a8e29c5ddb942c2324783` |
| Reviewed file bytes / SHA-256 | `2972` / `2d7adae73c322ea5aadfe13fede7d4ddfe06e14321cb8e29b0b053686ec8e03d` |
| Complete diff bytes / SHA-256 | `1671` / `3fcc6e184f4d71cdc9de20738cd96dd0d540e1d60752a43a4bbd686df2f70f42` |
| Fixed upstream U | `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |

The complete supplied manifest and diff were read. An independent read-only Git diff with full object names and binary-safe/rename-aware options reproduced the supplied diff byte-for-byte. Git reports exactly one modified path and 30 added lines. The snapshot commit directly names the stated base as its parent and the stated tree as its tree.

The manifest describes read-isolated source copies/private-index acquisition, with private index `/tmp/shared-source-74-r1-private-index`, no checked-out developer HEAD and `published=false`. I reviewed the resulting immutable Git objects, not the private index or a mutable developer checkout. Independent Git observations confirm this repository is bare, its symbolic `HEAD` names `refs/heads/master`, and that `HEAD` does not resolve to a commit. No developer HEAD, live publication state or remote branch lease was inferred from this bare repository.

## Reviewer independence and model / effort observability

- Lens: fresh blind general/source-integrity review of the exact one-file scope, including its existing source consumers.
- Requested configuration: configured model, `xhigh`; the assignment identifies this as high-risk evidence reasoning because it affects canonical OCRM links and pre/strict acceptance.
- Session instruction identifies the configured assistant as GPT-6 Astra Pro. No model override, alternate CLI, extra worker or global setting change was attempted.
- The reviewer has no exposed getter/attestation for the effective serving model or effort. The requested/inherited configuration is therefore recorded as such; prompt wording is not treated as proof of an applied `xhigh` control.
- No diagnostic report, runner/root decision, prior review report or peer review was opened. Path listings and existing source JSON necessarily exposed some evidence filenames, but those reports were not read or used to reach this verdict.

## Read coverage and limits

The complete changed file, its complete base version and the complete delta were inspected. Relevant adjacent authority and consumer reading included:

- Root `AGENTS.md` in full, including the immutable independent-review rules, model/effort observability rules, OCRM execution restrictions and source-review amendment. Tree-path inspection found no nested `AGENTS.md` governing the changed subtree.
- `docs/specs/openclaw-rust-equivalence-migration.md` current OCRM-EXEC-02 §17 in full, plus the source-record/state contracts in §6, structural/final separation and evidence publication in §12, and global completion predicates in §15.1. Historical/live-discovery portions outside those applicable sections were not comprehensively reread.
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md` current fixed-identity/precedence sections and applicable state/evidence/materialization guidance; module-map source-accounting and single-path/multiple-responsibility guidance. Historical navigation snapshots were not treated as present authority.
- `upstream.lock.json` in full; the inventory index and shard manifest; the relevant complete ledger responsibility; the existing read-scope responsibility's source/status fields; and the relevant RTM rows' identity, responsibility, oracle, blocker and acceptance fields.
- Complete production source of `scripts/source_review.py`, `verify_audit_integrity.py`, `expand_inventory.py`, `shard_inventory.py` and `validate_p0.py`. Relevant source-inventory schema fields were inspected without executing schema validation.
- Existing pending-link contract-test source in `test_source_review.py:250–325` and materializer contract-test source in `test_inventory_materialization.py:308–347`, read only.
- `.github/workflows/ocrm-p0-authority.yml:1–285`, covering exact-subject acquisition through guarded persistence and child dispatch. The unrelated later recovery job was not reviewed.
- `.github/workflows/rust-desktop-runtime.yml` trigger/subject guards and the strict committed-inventory consumer at lines 2925–2931; the rest of its broad runtime test matrix was outside this metadata review.
- `docs/specs/final-assistant-media-custody.md` and `docs/specs/media-settlement-unknown.md` in full, as the current scope/status contract for this responsibility. Their linked diagnostic/source-review reports were not opened.

All 61 committed gzip shards were read directly from snapshot Git objects, decompressed and parsed using standard-library gzip/JSON. Their compressed/uncompressed hashes, row cardinalities and recorded bounds were compared as read-only data inspection. All 62,130 rows participated in identity/link/count arithmetic and canonical-digest reconstruction; this is not a claim to have semantically reviewed 62,130 upstream files.

Only Git/source reads, standard-library read-only gzip/JSON/hash/arithmetic inspection, and writing this review report were performed. No project helper was imported or executed. No validator, generator, test, build, local/htch validation or workflow dispatch ran. No repository file, index or ref was edited. A `git status` read attempt against the bare repository returned “must be run in a work tree”; subsequent identity inspection used explicit immutable objects and did not rely on that failed command.

The bare review repository does **not** contain the fixed upstream commit U as a Git object. Accordingly, this review establishes exact agreement with the hash-checked committed inventory, current ledger and current lock; it does not independently reacquire the upstream repository or prove that GitHub Actions acquired U. Fresh upstream acquisition belongs to the existing source-authority job.

## Direct source and data observations

### Digest and exact membership

The new `from_canonical_inventory_sha256` at `inventory/index.json:66` is:

`8994988c264876be709930e2f87c6b60ae124a6077a833ccd324dc1117d26d93`

It equals both the existing shard manifest's canonical digest and the independently reconstructed canonical JSON digest of the committed shard rows, using the canonical serialization specified in `verify_audit_integrity.py:75–84`. All 61 shard compressed/uncompressed digests and recorded row bounds/cardinalities matched their manifest records. There are 62,130 distinct paths: 62,128 ordinary blobs, two symlinks and zero gitlinks.

The pending entries are unique and exactly equal the complete set of missing ledger-to-source reverse links across the current ledger. No extra declaration, omitted missing edge or ledger/source blob mismatch was observed. Existing source-to-ledger links resolve to exact matching path/blob references, and the currently mapped path set equals the set of paths with retained ledger links.

All five additions name `OCRM-RESP-MEDIA-FINAL-CUSTODY-001`:

| Path | Exact blob | Committed shard / decompressed JSONL row | Existing responsibility IDs | New mapped-path contribution |
| --- | --- | --- | --- | ---: |
| `src/gateway/agent-turn/agent-run-final-media.ts` | `f4bf5b4308cb2a7160ad1db0142160d14fb71551` | `part-0036.jsonl.gz`, row 96 | none | 1 |
| `src/gateway/agent-turn/agent-run-media.ts` | `27a3a65974fad078872b257ee28cafccdad4691f` | `part-0036.jsonl.gz`, row 98 | `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001` | 0 |
| `src/gateway/managed-image-attachments.custody.ts` | `1975104ea6cdcfea25d3bc4362edbaea5e673adf` | `part-0036.jsonl.gz`, row 956 | none | 1 |
| `src/infra/sqlite-worker-error.ts` | `b13f8354e7244d630ee274b3b0debf9a3f5bb8c6` | `part-0043.jsonl.gz`, row 463 | none | 1 |
| `src/media/store.read-scope.ts` | `73515a9ed6a2dc9f1a0adddc638d542ad1323ee5` | `part-0044.jsonl.gz`, row 914 | none | 1 |

Every listed row retains the exact fixed U repository/commit/tree, ordinary blob kind, matching `sha`/`blob_sha`, and `100644` mode. These are existing source rows; the patch does not invent source membership.

### Current responsibility and acceptance state

The ledger row identified at `parity-ledger.json:36810` currently declares:

| Dimension | Actual value |
| --- | --- |
| read | `partial` |
| mapping | `mapped` |
| implementation | `implementing` |
| wiring | `wired` |
| verification | `not-run` |
| acceptance | `pending` |
| reviewer | `null` |

In particular, `mapped` / `not-run` / `pending` is exactly the unfinished-status combination required by `source_review.py:88–95` for a missing link to be permitted before expansion. The allowance cannot be used by a `reviewed`, `verified` or `accepted` responsibility.

The existing responsibility links to `OCRM-R23`, `OCRM-R24`, `OCRM-AC23` and `OCRM-AC24`; all four current RTM rows reference it, remain `in-progress`, and retain both `BL-MEDIA-FINAL-CUSTODY-FULL-EQUIVALENCE-001` and `BL-MEDIA-SETTLEMENT-EXACT-H-EVIDENCE-001`. Their MSU-01 through MSU-10 oracle references remain present. These are existing declarations, not promoted evidence.

The old `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001` ledger row at line 15135 still references the exact `agent-run-media.ts` blob and remains separately incomplete (`implementation=not-started`, `wiring=not-wired`, `verification=not-run`, `acceptance=pending`). Adding the final-custody link does not discharge or replace it.

### Counts, literal history and flags

Removing only the newly added `pending_mapping_additions` key from the snapshot index produces exactly the base index JSON. Thus this delta changes no existing upstream identity, historical artifact/run reference, count, reading claim, closure flag or current-evidence field.

The committed inventory currently has 884 mapped paths and 61,246 unclassified paths. Five new links affect only four previously unclassified paths, because `agent-run-media.ts` is already mapped. The mapping-only materialization effect is therefore **888 mapped / 61,242 unclassified**, with the tracked denominator unchanged at 62,130. A count of 889 would incorrectly count links as paths.

All five affected rows remain literally `unread`, with no read ranges, no row evidence references, `review_status=pending`, unknown category/risk/applicability, and pending disposition. The existing read-observation declaration contains no entry for any of the five paths. Its 91 existing entries introduce no new full-read status relative to the present rows. The link materialization therefore supplies no basis to reduce the current 62,067 unread-or-partial denominator, promote review completion, or manufacture full reading.

All eight ledger and RTM completion flags remain false. The index keeps `latest_exact_head_evidence=null`, source/dependency/LFS/applicability-disposition closure false, baseline identity acceptance false, and semantic-reading/responsibility-decomposition completion false. Historical candidate evidence is retained byte-for-byte in the changed file.

## Existing consumer chain and integration assessment

### 1. Declaration admission is exact and bounded

`source_review.py:51–70` accepts only the existing declaration shape, matching prior canonical digest, a nonempty entry list, exact entry keys and unique nonempty string tuples. The new data satisfies this shape without requiring a new schema state or alternate authority mechanism.

`verify_audit_integrity.py:42–102` first binds index/ledger/RTM/manifest to the lock, reconstructs the hash-checked inventory, validates existing source links through the shared review contract, and then passes the declaration into the pre-expansion link checker. `source_review.py:73–103` admits only genuinely missing exact path/blob links on unfinished responsibilities and requires the declared set to equal the actual pending set. Existing mappings remain checked, including the retained read-scope link. The current delta meets those specific source-level preconditions.

This is a static conclusion about the affected contract. No claim is made that the complete pre-expansion executable gate has run or passed.

### 2. Expansion uses the existing ledger and preserves the old link

`expand_inventory.py:41–88` checks committed shard content against the freshly acquired fixed-U inventory before using prior rows. Lines 106–160 build an exact `(path, blob_sha)` ledger map, retain prior source fields and literal history, and union ledger IDs with existing IDs. Only `unclassified` mapping status is advanced to `mapped`.

Consequently, `agent-run-media.ts` retains `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001` and gains `OCRM-RESP-MEDIA-FINAL-CUSTODY-001`; the other four affected rows gain the final-custody ID. There is no replacement operation or blanket completion upgrade. Existing literal-read observations and conservative review fields retain their separate ownership.

### 3. Sharding consumes the declaration only after the links exist

`shard_inventory.py:90–100` checks each pending tuple against the expanded row's exact blob and responsibility list before writing shards. Lines 133–163 update the canonical manifest and row-derived aggregate projections, remove `pending_mapping_additions`, derive mapped-path counts from row status, and derive unread counts from literal `read_status`.

The existing source tests describe positive materialization, unresolved-link rejection and preservation of `unread`/`pending`; they were read, not run. The patch relies on this already-existing mechanism rather than editing generated shards locally.

### 4. Strict consumers remain strict

`verify_audit_integrity.py:98–102` explicitly rejects any nonempty pending declaration unless `--before-expansion` is present. `validate_p0.py:204–210` separately requires every ledger source reference to have its reverse inventory link after materialization. `validate_p0.py:253–274` checks the derived coverage/read counts and unpromoted global flags.

The source-authority workflow runs the pre-expansion check, expansion, shard write, structural validator and strict committed-source check in that order at `.github/workflows/ocrm-p0-authority.yml:153–166`, under `set -euo pipefail`. The runtime workflow invokes `verify_audit_integrity.py` without the pre-expansion allowance at `.github/workflows/rust-desktop-runtime.yml:2925–2931`.

A runtime strict check on the original declaration commit is therefore expected to reject the pending state. Treating that same declaration commit as already materialized, weakening the strict gate, or treating its source-job output as current runtime acceptance would violate the existing contract. None of those changes is present in this delta.

### 5. Persisted child H is a new evidence subject

The source-authority job checks the exact checkout and optional frozen dispatch SHA before acquisition. The persistence job depends on source-authority success, checks the original subject against the downloaded summary/receipt and canonical digest, and materializes from that artifact (`ocrm-p0-authority.yml:177–220`). It stages the existing generated inventory/ledger owners, checks the branch still names the original subject, and pushes without force (`222–252`).

For the existing migration branch, a materialized commit triggers explicit source-authority and `ocrm-config` runtime dispatches with `expected_subject=RESULT_HEAD`, after another branch-head comparison (`254–285`). Source and runtime entry guards reject a moved dispatch ref. The manual child source dispatch does not enter the PR/push persistence condition, consistent with checking the materialized child rather than creating an evidence-carrier commit loop.

This source path supports the required sequence: published declaration H → source materialization and strict checks → guarded generated commit Hm → fresh source/runtime checks on Hm. The review snapshot is not itself a published H. No live branch lease, generated child, dispatch outcome, Actions result or acceptance receipt was observed or claimed.

## Findings and remaining evidence boundary

**Findings: none.** No concrete failure caused by the new one-file declaration was found after reading the complete delta and its actual adjacent consumers. No preference-only edits, speculative cleanup or out-of-scope runtime defect is proposed.

The source-contract conclusion is limited to the immutable snapshot above. Fresh Actions acquisition of U, execution of the existing pre/materialization/strict pipeline, guarded persistence to the intended branch, and the materialized child's exact-subject results remain evidence obligations. OCRM final review, source/dependency closure, complete media equivalence and final acceptance remain unproved exactly as the current declarations state.
