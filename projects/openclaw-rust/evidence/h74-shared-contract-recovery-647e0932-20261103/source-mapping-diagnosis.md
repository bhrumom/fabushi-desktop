# H74 source-accounting and evidence-reference diagnosis

## Frozen scope, method, and conclusion

Repository: `bhrumom/fabushi-desktop`.

- Target H: `74f11d3d31c6245ac5886dc0181460b2d4349744`.
- Target tree: `aca91e1cfbedeecb039d442fb3528caac89873a3`.
- Fixed upstream U: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Worker: `current_evidence_reference_read`; inherited actual `xhigh`, configured model retained. This is source-authority diagnosis, not routine bookkeeping or an implementation/acceptance review.
- The exact GitHub commit API confirmed the H/tree association. H was absent from the local object database, so source authority came from exact-ref GitHub API reads. The mutable local checkout was used only to locate paths and available candidate object IDs; it was never treated as target authority.

**Two concrete defects are confirmed.** First, the new `OCRM-RESP-MEDIA-FINAL-CUSTODY-001` ledger responsibility has five exact source references whose committed inventory rows lack the reverse link to that responsibility, and H74 supplies no `pending_mapping_additions` declaration. This directly explains the supplied pre-expansion reverse-link diagnostic under the existing validator. Second, the ledger and RTM contain six references to two nonexistent evidence paths. The correct existing files contain the intended bounded discovery inputs, with historical scopes that must be retained.

The minimal correction uses the existing owners: five pending mapping tuples in `inventory/index.json`, plus six literal reference corrections in `parity-ledger.json` and `requirements-traceability-matrix.json`. No validator, workflow, schema, source-reading claim, status, blocker, or completion-flag change is needed for these defects. No correction was applied by this worker.

Root supplied the original Actions diagnosis for run `41580539540`, job `125734077461`: step 9, `Expand canonical source records and verify P0 G01/G02 structural`, failed at `verify_audit_integrity.py` → `source_review.py:94` with `Missing reverse source/responsibility link: OCRM-RESP-MEDIA-FINAL-CUSTODY-001`. This worker did not fetch, rerun, dispatch, or independently adjudicate that run; the source diagnosis below is independently supported by the frozen files.

Governing authority read: root [AGENTS.md](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/AGENTS.md), current [main spec §17](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/docs/specs/openclaw-rust-equivalence-migration.md#L671-L720), applicable Source of Truth/module-map passages, and the narrow media specifications and evidence. §17.2 preserves one inventory/ledger/RTM and literal reading history; legal metadata alone does not establish semantic or actual-reference evidence. §17.5 preserves exact-subject acceptance and all global flags.

Only Git/API reads, immutable source-byte acquisition/decompression, and text inspection were used. No validator, test, schema check, parser, generator, build, capture, or acceptance command was executed. The sole authored file is this report.

## 1. The reverse-link failure and its existing producer

### 1.1 Why the pre-expansion gate fails

The ledger's final responsibility is `OCRM-RESP-MEDIA-FINAL-CUSTODY-001`, unit `OCRM-UNIT-MEDIA-008`; it describes **final outgoing-media** MAC-v1/MSU-v1 custody. Its five source references appear at ledger lines 36869–36915. Its state at lines 36916–36923 is:

```json
{
  "acceptance": "pending",
  "implementation": "implementing",
  "mapping": "mapped",
  "read": "partial",
  "verification": "not-run",
  "wiring": "wired"
}
```

The row's reviewer remains `null`. Its two blockers are `BL-MEDIA-FINAL-CUSTODY-FULL-EQUIVALENCE-001` and `BL-MEDIA-SETTLEMENT-EXACT-H-EVIDENCE-001`. These facts are not authorization to upgrade the row.

The existing pre-expansion gate [verify_audit_integrity.py, lines 88–108](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/projects/openclaw-rust/scripts/verify_audit_integrity.py#L88-L108) reads pending additions from the **existing index**, then calls `validate_source_responsibility_links` against committed rows. [source_review.py, lines 73–103](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/projects/openclaw-rust/scripts/source_review.py#L73-L103) requires each ledger path/blob to exist and each reverse responsibility ID to exist. Before expansion it permits a missing reverse edge only when all of the following hold:

1. `before_expansion` is true.
2. The responsibility remains `mapping=mapped`, `verification=not-run`, `acceptance=pending`.
3. That exact `(responsibility_id, path, blob_sha)` tuple is declared in `pending_mapping_additions`.
4. The complete observed pending tuple set equals the declared tuple set.

H74's [inventory/index.json](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/projects/openclaw-rust/inventory/index.json) has no `pending_mapping_additions` field. Thus the newly introduced ledger edge cannot reach the materializer through the pre-expansion gate.

This is distinct from the dangling evidence references. All five affected source rows have `review_status=pending`, so they leave the shared review validator before the completed-review evidence existence checks.

### 1.2 Exact current rows, including the link that must survive

The exact-H74 GitHub file API returned these compressed shard blob IDs:

| Shard | Exact Git blob |
| --- | --- |
| `part-0036.jsonl.gz` | `a41302e3cb662b4f0b17c84fca8a59205ebb4054` |
| `part-0043.jsonl.gz` | `6642a2b5abd7e4fdd9f477192e655b9c6229fbea` |
| `part-0044.jsonl.gz` | `a764a21974401c0b0353c4ba3904c51da0d9af32` |

The same immutable blobs were locally available. Their raw rows were read with `git cat-file blob <exact-API-sha> | gzip -dc | rg -n -F ...`; no JSON parser or repository checker was run. All five rows bind the fixed U repository/commit/tree above.

| Exact source path | U blob | Shard / JSONL line | H74 responsibility state |
| --- | --- | --- | --- |
| `src/gateway/agent-turn/agent-run-final-media.ts` | `f4bf5b4308cb2a7160ad1db0142160d14fb71551` | 0036 / 96 | `unclassified`, IDs `[]` |
| `src/gateway/agent-turn/agent-run-media.ts` | `27a3a65974fad078872b257ee28cafccdad4691f` | 0036 / 98 | `mapped`, IDs `["OCRM-RESP-MEDIA-STORE-READ-SCOPE-001"]` |
| `src/infra/sqlite-worker-error.ts` | `b13f8354e7244d630ee274b3b0debf9a3f5bb8c6` | 0043 / 463 | `unclassified`, IDs `[]` |
| `src/gateway/managed-image-attachments.custody.ts` | `1975104ea6cdcfea25d3bc4362edbaea5e673adf` | 0036 / 956 | `unclassified`, IDs `[]` |
| `src/media/store.read-scope.ts` | `73515a9ed6a2dc9f1a0adddc638d542ad1323ee5` | 0044 / 914 | `unclassified`, IDs `[]` |

Every row has `read_status=unread`, empty `read_ranges`, empty `evidence_refs`, `reviewer=null`, `category=unknown`, `risk=unknown`, `required_review_depth=deep`, `review_status=pending`, `applicability_status=unknown`, `disposition_status=pending`, and empty review-input/dependency arrays. The fixed-U report describes additional historical reading, but this bounded repair does not materialize it as new canonical reading or review authority.

### 1.3 Exact bounded declaration to add

The hand-editable admission field belongs in the existing `projects/openclaw-rust/inventory/index.json`. The producer of actual `responsibility_ids` is already the sole ledger: [expand_inventory.py, lines 106–160](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/projects/openclaw-rust/scripts/expand_inventory.py#L106-L160) collects each ledger `(path, blob_sha)` and unions its IDs with the historical row IDs. There is no missing second source registry to invent, and no need to hand-edit the generated shards or add these entries to `read-observations.json`.

The exact H74 manifest's `canonical_inventory_sha256` is `8994988c264876be709930e2f87c6b60ae124a6077a833ccd324dc1117d26d93`. The already implemented [pending_mapping_additions contract, lines 51–70](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/projects/openclaw-rust/scripts/source_review.py#L51-L70) requires the following shape, with no additional keys inside the declaration or tuples:

```json
"pending_mapping_additions": {
  "from_canonical_inventory_sha256": "8994988c264876be709930e2f87c6b60ae124a6077a833ccd324dc1117d26d93",
  "entries": [
    {
      "responsibility_id": "OCRM-RESP-MEDIA-FINAL-CUSTODY-001",
      "path": "src/gateway/agent-turn/agent-run-final-media.ts",
      "blob_sha": "f4bf5b4308cb2a7160ad1db0142160d14fb71551"
    },
    {
      "responsibility_id": "OCRM-RESP-MEDIA-FINAL-CUSTODY-001",
      "path": "src/gateway/agent-turn/agent-run-media.ts",
      "blob_sha": "27a3a65974fad078872b257ee28cafccdad4691f"
    },
    {
      "responsibility_id": "OCRM-RESP-MEDIA-FINAL-CUSTODY-001",
      "path": "src/infra/sqlite-worker-error.ts",
      "blob_sha": "b13f8354e7244d630ee274b3b0debf9a3f5bb8c6"
    },
    {
      "responsibility_id": "OCRM-RESP-MEDIA-FINAL-CUSTODY-001",
      "path": "src/gateway/managed-image-attachments.custody.ts",
      "blob_sha": "1975104ea6cdcfea25d3bc4362edbaea5e673adf"
    },
    {
      "responsibility_id": "OCRM-RESP-MEDIA-FINAL-CUSTODY-001",
      "path": "src/media/store.read-scope.ts",
      "blob_sha": "73515a9ed6a2dc9f1a0adddc638d542ad1323ee5"
    }
  ]
}
```

This is an addition to the existing index object, not a replacement document. Its digest must continue to identify the unchanged prior committed inventory. If the integrator's actual base has changed, reread that base's manifest and missing tuples rather than copying the H74 anchor onto a different inventory.

### 1.4 Preservation and expected materialized effect

Leave the committed inventory, its manifest, counters, dependency declarations, read observations and all statuses unchanged in the source repair. Existing Actions performs the derived materialization: [workflow lines 153–166](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/.github/workflows/ocrm-p0-authority.yml#L153-L166) calls pre-expansion integrity, expansion, sharding, structural validation, and ordinary integrity in that order.

Expansion copies historical fields, applies conservative review defaults only where absent, unions the responsibility IDs, and changes only previously `unclassified` mapping state to `mapped`. The existing `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001` ID on `agent-run-media.ts` must remain. [shard_inventory.py, lines 90–163](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/projects/openclaw-rust/scripts/shard_inventory.py#L90-L163) checks that every declared pending edge materialized, writes the derived inventory, removes the pending declaration, and projects actual counts.

For these five edges alone, four paths become newly mapped; one was already mapped. The expected derived mapping count is therefore **884 → 888**, and unclassified is **61,246 → 61,242**. Tracked paths remain **62,130**; unread/partial remains **62,067** because none of these five rows becomes read-complete. Applicability/disposition/review states and their current zero assessed/dispositioned counts remain unchanged. These are source-derived expectations, not an executed result or numbers to hand-edit. Preserve all existing evidence and historical reading; do not relabel the discovery report's full reads as completed review.

## 2. Six literal references name absent paths

Let `E` denote `projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/`.

Exact-ref GitHub reads for both `E/media-settlement-fixed-u-contract.md` and `E/media-settlement-unknown-current-audit.md` returned **404 / NOT_FOUND** at H74. The exact directory listing instead contains:

| Existing path | Exact H74 Git blob | Size |
| --- | --- | --- |
| `E/fixed-u-contract.md` | `b143ea8d36f84c02793263bbadf39a2e138619bf` | 34,655 bytes |
| `E/current-owner-audit.md` | `01b724729a97f33ad1ef63b937360022ee5e48b9` | 30,927 bytes |

The immutable directory can be re-read at [this exact contents URL](https://api.github.com/repos/bhrumom/fabushi-desktop/contents/projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103?ref=74f11d3d31c6245ac5886dc0181460b2d4349744). Absence is established by the exact directory contents plus individual exact-ref 404s, not by a mutable checkout search.

| Canonical document / stable row | H74 line | Current invalid suffix | Bounded replacement suffix |
| --- | --- | --- | --- |
| Ledger `OCRM-RESP-MEDIA-FINAL-CUSTODY-001.evidence_refs` | 36791 | `media-settlement-fixed-u-contract.md` | `fixed-u-contract.md` |
| Same ledger row | 36792 | `media-settlement-unknown-current-audit.md` | `current-owner-audit.md` |
| RTM `OCRM-R23.evidence_refs` | 1399 | `media-settlement-fixed-u-contract.md` | `fixed-u-contract.md` |
| RTM `OCRM-R24.evidence_refs` | 1556 | `media-settlement-fixed-u-contract.md` | `fixed-u-contract.md` |
| RTM `OCRM-AC23.evidence_refs` | 2922 | `media-settlement-fixed-u-contract.md` | `fixed-u-contract.md` |
| RTM `OCRM-AC24.evidence_refs` | 3102 | `media-settlement-fixed-u-contract.md` | `fixed-u-contract.md` |

All four RTM rows are `acceptance_status=in-progress` and retain the new final-custody responsibility. The old `media-settlement-unknown-current-audit.md` string occurs only in the ledger, not in RTM. The old fixed-U string occurs once in the ledger and four times in RTM. These counts came from full exact-file text acquisition and literal inspection.

Correct these six strings in their current rows. Do not create duplicate alias files, rename the existing evidence, delete the references, rewrite the historical report bodies, or broaden this correction into a new validator/governance project.

## 3. Why the existing files are appropriate, and what they do not prove

Both existing reports were read completely. Their content establishes the intended provenance connection; the recommendation does not rely on similar filenames.

### 3.1 `fixed-u-contract.md`: fixed-U MSU design input

The [report](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/fixed-u-contract.md) is titled “Fixed-U final assistant media settlement — bounded source contract,” identifies the same task/round, and explicitly states that it is read-only discovery for the next MSU specification, not a candidate review, runtime result, migration acceptance, or LFA status change (lines 1–13).

It binds U `1440bb1771372334bea23d7642bb52deb7db1845` and task-pinned tree `90c8aab39972da11cfad220361781d49e1bc6700`. It distinguishes acquisition/hashing from semantic reads and discloses that the tree identity was task-pinned rather than freshly audited there. The ledger's five U blobs and ranges match the report's five corresponding full-read entries exactly:

| Source | Report entry | Report's semantic read scope |
| --- | --- | --- |
| `agent-run-final-media.ts` | S01 | 1–243, full |
| `agent-run-media.ts` | S02 | 1–127, full |
| `sqlite-worker-error.ts` | S11 | 1–135, full |
| `managed-image-attachments.custody.ts` | S15 | 1–167, full |
| `store.read-scope.ts` | S20 | 1–269, full; verified parent cache |

The report's broader table (lines 126–178) names 20 TypeScript sources plus LICENSE and explicitly preserves partial reading: dispatch; chat-reply media; transcript persistence; session-entry patch; transcript-write context; SQLite worker contract imports; and managed-image attachments have bounded ranges. This report must not become a claim that those whole files or their full dependency graphs were reviewed.

Substantively it documents successful retention after transcript rewrite, canonical unknown-outcome retention, native commit receipts versus ordinary replies, delegated descriptor custody, exact cleanup claims, commentary differences, and the limitation of transient expiry. It expressly warns that immediate upstream retention is not proof of indefinite unresolved retention. Its derived AgentDb → Session → TranscriptRuntime → App → OutgoingMediaCustody requirements correspond directly to the ledger's MAC-v1/MSU-v1 behavior and invariants.

Its dependency scope is limited to the pinned `@openclaw/fs-safe` archive identity, package declaration and license; archive/native implementation, complete dependency/license closure, all storage/publication/recovery branches, current Fabushi behavior, and exact-H Windows/Linux execution remain open (lines 180–197). This worker read the report and matching canonical references; it did not independently re-perform all of that historical upstream source review.

### 3.2 `current-owner-audit.md`: historical target defect and owner-chain input

The [report](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/current-owner-audit.md) is titled “Media settlement unknown-outcome audit,” names the same task/round, and binds **target H `78aa967cc9400bb5c207441b2a78dbf7021d596c`, tree `f62fc666ade7ad6f8dbf40e889e5ec100a74e974`** (lines 1–16). It explicitly claims no new U source-reading credit and no reproduced failure or executable validation. Its inspected App blob was `ca038b1aa9f3a78a1951551e0a80d543142c08dd`; it excludes the optional App overlay and LFA review.

It identifies the historical DB rollback-result loss and unknown-outcome erasure through Session/Runtime into App cleanup; traces real producer, transaction owner, receipt, publication and first-open recovery; and proposes typed outcome propagation and exact connection retirement using existing mechanisms. The cited ranges include DB 4660–4919, Session 4062–4080, Runtime 1797–1833, App 965–1091, and attachment custody 21–63, all at H78aa. The final identity table (lines 184–212) preserves the concrete historical blobs for those paths and adjacent consumers/test sources. Proposed MSU-T01–T08 are expressly future Actions oracles, not executed outcomes.

This is the intended historical current-owner diagnosis behind the MSU correction. It is **not an audit of H74**. The current `media-settlement-unknown.md` spec itself identifies a later implementation base `fdb0f6acb06165b6f2796961186e06a2de98c229` / tree `abeb6ffdf09b676279462fea3b9e1f6d72abc1b6` and treats the adjacent source/design reports as specification inputs, not executable evidence (lines 15–24). Neither the historical target nor its diagnosis should be silently relabeled to that base or to H74. The body supplies historical provenance and design rationale; fresh H74 behavior and any remaining defects require their own evidence.

The audit retains the broader gaps: no-fence compatibility semantics, complete descriptor identity and placement, all capture/concurrency cases, full detached recovery, and provider/platform acceptance. Both it and the fixed-U report therefore support restoring the references while preserving `partial`, `mapped`, `implementing`, `not-run`, and `pending` states.

## 4. What the current validators actually enforce

This is a source-code interpretation, not a local execution result.

| Record/state | Actual H74 enforcement relevant to references |
| --- | --- |
| Ledger/RTM schema `evidence_refs` | Arrays of unique strings; no filesystem existence or semantic-content validation in the schema. Ledger reviewer may be string or null. See ledger schema 520–532; RTM schema 97–126. |
| Literal source `read_status=read-complete` | `source_review.py` 228–232 requires range/content identity, reviewer, nonempty evidence, language/module classification. That branch alone does not call `evidence_exists`. |
| Legacy source row with no new review fields | Allowed only in nonfinal mode; it gains no completion authority. Lines 248–255. |
| Source review `pending`, `in-progress`, `blocked` | Category/risk/depth/status metadata is checked; nonfinal mode then continues before evidence existence. Final mode fails unfinished review. Lines 256–267. |
| Source review `deep-complete`, `contract-complete`, `disposition-complete` | Requires resolved classification, correct depth/status, rationale/reviewer/evidence, acquired row evidence, reviewed responsibility mapping, reviewed responsible ledger row with reviewer/evidence, and acquired responsibility evidence. It also checks exact input/dependency identities and type-specific depth requirements. Lines 268–342. |
| Ledger `mapping=reviewed` | Outer `validate_p0.py` 219–229 requires owner/oracle/reviewer, or non-product reviewer/evidence. Final mode requires reviewed mapping. This is not an independent general path-existence scan. |
| Ledger `implementation=implemented`, `wiring=wired`, `verification=verified` or `acceptance=accepted` | Outer validator checks the specified nonempty target/integration/evidence/reviewer fields; business implementation must be Rust. It does not resolve every ledger evidence path in this loop. Lines 230–237. H74's media row only reaches the wired condition among these positive-state checks. |
| RTM `in-progress` | The RTM loop checks responsibility IDs; no reference-existence check is conditioned on in-progress. Verified/accepted require nonempty evidence/reviewer; N/A requires reason/reviewer/evidence. Lines 263–269. |
| Any ledger source reference, including this pending media mapping | Reverse source/responsibility linkage is independently enforced even when review remains pending. The only pre-expansion allowance is the exact anchored pending tuple declaration. |

Both production gates pass a filesystem resolver to the shared source-review code: [validate_p0.py 239–246](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/projects/openclaw-rust/scripts/validate_p0.py#L239-L246) and [verify_audit_integrity.py 93–97](https://github.com/bhrumom/fabushi-desktop/blob/74f11d3d31c6245ac5886dc0181460b2d4349744/projects/openclaw-rust/scripts/verify_audit_integrity.py#L93-L97). These strip a `#` fragment, require an existing file confined to the repository, and do not inspect the report's target/source scope, content hash, semantic sufficiency, or anchor validity. The P0 resolver additionally rejects absolute paths. Existence is therefore a necessary, narrower check when that completed-review branch runs; it does not establish exact-H acceptance.

There is also an unconditional existence check for top-level evidence supplied by `apply_literal_read_observations` (source_review.py 118–125), used by expansion for `inventory/read-observations.json`. The two bad strings do not appear in that H74 observations document. This does not change the conclusion about the six ledger/RTM references.

The six broken references are a real provenance defect even though current pending/in-progress states do not make them the observed pre-expansion gate failure. The existing reverse-link gate is functioning as written. Do not delete or relax it, promote statuses to route around it, or describe the eventual structural pass as full source closure.

## 5. Bounded integration handoff

The authorized integrator can repair these defects with changes confined to three existing canonical metadata files:

1. Add the five exact, anchored pending mapping entries to `inventory/index.json` while retaining all current fields/counters and the frozen prior inventory.
2. Replace the two incorrect evidence literals in the final-custody ledger row.
3. Replace the fixed-U literal in RTM R23, R24, AC23, AC24.

Preserve the row's requirements, source refs/blobs/ranges, unit/owner, existing source-reader evidence bodies, existing read-scope responsibility link, statuses, reviewer values, both media blockers, and every global flag. Preserve the shipping source/test/workflow changes and all six LFA cases and incoming-media custody work owned elsewhere. Do not change current/past H identities, rebaseline U, rewrite historical reading, replace the sole registry, manually fabricate generated counters, or borrow TDRP evidence.

The original H74 failure remains historical failed evidence. Any corrected successor is a new subject, and the existing Actions flow owns its materialization and validation. Root owns publication, original log collection, and any necessary successor validation; this worker has neither dispatched nor pushed. Full source/dependency closure, semantic review, implementation verification, ecosystem/platform acceptance, and all eight global flags remain open.
