# H74 shared contract recovery — R1 general/security review

## Verdict

**APPROVE — source review only.** No blocking correctness or security finding was identified in the complete frozen six-path change. The patch repairs source-accounting declarations and test expectations without changing production behavior, weakening the transaction exception boundaries, or advancing verification/acceptance state.

This verdict does **not** establish successful compilation, mutation-test execution, inventory materialization, runtime behavior, or final OCRM migration acceptance. A successor exact-H GitHub Actions result remains necessary. No historical failed result is replaced by this review.

## Authority, independence, and applied settings

| Item | Reviewed identity |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| Assigned canonical branch | `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Frozen base commit | `4f0639321c4ae03cc7dbef1d8e49dea0f042e180` |
| Frozen base tree | `cbc9418ee7572f44d59e067e450a67f6c0bc22a0` |
| Candidate tree | `406f8668a83a30a24c83b442b97d3a2b4d64927f` |
| Complete patch | `r1-complete.patch`, 23,279 bytes |
| Patch SHA-256 | `17280ae7b638e814a51ea05e25970abfc42481c495ac5c08e6a4b465f7922995` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Reviewer | `shared_recovery_general_r1` |
| Applied effort | **xhigh** — parent confirmed the supported launch field `reasoning_effort: "xhigh"`, with `fork_turns: "none"` |
| Applied model | Configured **GPT-6 Astra Pro**, retained with no model override |

The effort classification follows repository policy for shared transaction/evidence contracts, security-sensitive failure ownership, and independent review. The applied setting is supported by the parent's launch record; it is not inferred from prompt wording.

I read the entire supplied patch and independently regenerated the binary/full-index, rename-aware diff between the two frozen trees. Its SHA-256 exactly matches the supplied patch. Git reports six modified files, 152 insertions and 29 deletions. The base commit object was unavailable locally without lazy fetch; the exact GitHub Git-commit API independently confirmed its base-tree identity. All candidate source reads used immutable Git objects with `GIT_NO_LAZY_FETCH=1`. Uncached adjacent documents/build declarations were read through the GitHub file API at the exact base commit, with returned blob identities matching the candidate tree's unchanged entries. Mutable worktree source was not used as authority.

No other reviewer’s report for this recovery round or runner pre-fix decision was read. The existing `fixed-u-contract.md` and historical `current-owner-audit.md` were inspected only as the declared, unchanged specification/evidence targets whose filename references this patch repairs. They were not used as an independent verdict on this candidate.

Applicable authority inspected includes root `AGENTS.md`; current OCRM source-of-truth/owner-map constraints; main specification §17.2 and §17.5; `media-settlement-unknown.md`, especially MSU-01 and MSU-06–10; and `cron-job-identity.md`, especially CJI-03 and CJI-07. The frozen tree has no nested `AGENTS.md` governing the six changed paths.

## Complete changed scope

| Path | Candidate blob | Source-review result |
| --- | --- | --- |
| `projects/openclaw-rust/inventory/index.json` | `6905d89a058a59f09a2ec3f5b3b4afc6f751919d` | Exact pending reverse mappings; no count or state promotion |
| `projects/openclaw-rust/parity-ledger.json` | `99fd980c1f8bee47327efc49e59e0d5544dcf46f` | Two evidence filenames corrected; states and history retained |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `bd1a56a6990168c384bb553c269a3fc1b695a14a` | Four references corrected; acceptance/gates unchanged |
| `source/host/src/extensions/session/agent_db.rs` | `87186bc4b246d88a5ce51ebda0623b76f4425d3d` | Private test helper and two exact expectations only |
| `source/host/tests/openclaw_chat_cron_node_contract.rs` | `2bde1d3f9ab87f53e00079140a414ad492ac2ab6` | Existing projection assertions moved to actual owners; wrapper checks added |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `35719fc77edb1fea8f7acf0b22524c6eff7b29aa` | Inherent-owner selection and logical-newline handling repaired; negative guards retained |

## 1. Inventory and evidence integrity

The new `pending_mapping_additions` is anchored to `8994988c264876be709930e2f87c6b60ae124a6077a833ccd324dc1117d26d93`, exactly the unchanged canonical shard-manifest digest. All five triples match the existing `OCRM-RESP-MEDIA-FINAL-CUSTODY-001.source_refs` and the exact stored shard rows:

| Fixed-U path | Exact blob | Existing reverse-map state |
| --- | --- | --- |
| `src/gateway/agent-turn/agent-run-final-media.ts` | `f4bf5b4308cb2a7160ad1db0142160d14fb71551` | Unclassified; no responsibility IDs |
| `src/gateway/agent-turn/agent-run-media.ts` | `27a3a65974fad078872b257ee28cafccdad4691f` | Already mapped to `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001` |
| `src/infra/sqlite-worker-error.ts` | `b13f8354e7244d630ee274b3b0debf9a3f5bb8c6` | Unclassified; no responsibility IDs |
| `src/gateway/managed-image-attachments.custody.ts` | `1975104ea6cdcfea25d3bc4362edbaea5e673adf` | Unclassified; no responsibility IDs |
| `src/media/store.read-scope.ts` | `73515a9ed6a2dc9f1a0adddc638d542ad1323ee5` | Unclassified; no responsibility IDs |

The five reverse links are distinct. They do not imply five newly mapped paths: `agent-run-media.ts` already contributes to the mapped denominator. Four additional mapped paths would follow from these five links if the existing materializer succeeds; that is a source-derived expectation, not an executed result. The patch correctly leaves the current counters at 62,130 tracked paths, 884 mapped, 61,246 unclassified, and 62,067 unread/partial. All five inspected rows still have literal `read_status: "unread"`, `review_status: "pending"`, and their original review/history fields.

The unchanged consumer chain is concrete:

- `scripts/source_review.py::pending_mapping_additions` requires the exact declaration shape, matching prior digest, nonempty entries, and unique triples.
- `validate_source_responsibility_links` permits a missing reverse link only before expansion and only for the explicitly declared triple, with mapping `mapped`, verification `not-run`, and acceptance `pending`. It requires the actual missing set to equal the declaration and retains the existing mapped-denominator check.
- `expand_inventory.py` merges ledger IDs only for exact path/blob matches and preserves prior reading/review fields.
- `shard_inventory.py::write` checks that each declared reverse link actually materialized before consuming the declaration and derives coverage from rows.
- `verify_audit_integrity.py` rejects unresolved additions in the normal committed-source mode. `.github/workflows/ocrm-p0-authority.yml` still runs the pre-expansion integrity gate before generation and the ordinary gate afterward.

These are already existing schema/gate semantics, so this patch does not introduce a new review status, bypass, or parallel source-accounting owner. Existing materialization tests retain the pending-link consumption and unresolved-link refusal controls.

The two ledger filename changes resolve to unchanged existing files: `fixed-u-contract.md` (blob `b143ea8d36f84c02793263bbadf39a2e138619bf`) and `current-owner-audit.md` (blob `01b724729a97f33ad1ef63b937360022ee5e48b9`). The four RTM replacements consistently point to the former for R23, R24, AC23, and AC24. These documents describe bounded source discovery/audit, not successful candidate execution. The responsibility remains `read: partial`, `mapping: mapped`, `implementation: implementing`, `wiring: wired`, `verification: not-run`, `acceptance: pending`; blockers, requirement identities, gate declarations, and all eight false global completion flags remain unchanged. No shard, source-read history, source lock, or evidence content is modified.

## 2. Exact durable receipt expectations

Every `agent_db.rs` hunk is inside `#[cfg(test)] mod media_settlement_tests`. Production transaction behavior, public signatures, persisted formats, and notification paths are byte-unchanged by this diff.

`Fixture::expected_committed_entries` extracts the same independently written JSON expectation previously embedded in `assert_committed_storage`. It still requires the ordered send entry followed by exactly one receipt with the exact ID, kind, session key, work ID, lifecycle revision, original path, and transcript-entry IDs. It does not derive expected data by calling the production receipt builder, filter actual rows, or weaken equality.

The two changed reads are owner-level `SandAgentDb::get_transcript_entries` calls. That method delegates to `read_transcript_entries_from_db`, which reads `LIST_TRANSCRIPT_ENTRIES_SQL` in sequence order and uses `agent_db_serde::parse_owner_transcript_entry`. The owner decoder deliberately retains `outgoing-media-settlement` control rows. Presentation filtering is a different boundary. Therefore, the old one-send-only expectations were inconsistent with the actual durable owner API; requiring the complete send-plus-receipt vector is the correct exact oracle.

The lost-acknowledgement test continues to require the real fault to be claimed and applied, typed unknown, exact connection retirement, zero publication/write-generation advance, retained bytes, a released durable fence, refusal by the retired owner, and two fresh-owner observations. The fault-isolation test still proves that a separate live owner cannot consume another owner's fault and remains usable after the invoking owner retires. The independent SQL/storage assertions remain in both scenarios.

## 3. Media oracle ownership, CRLF, and negative guards

`media_commit_inherent_impl` now excludes trait implementations from its inherent-owner candidate set. This allows the already existing `impl Drop for SandAgentDb` without weakening checks on the actual inherent implementation: uniqueness, exact unqualified type, no unsupported attributes/generics/defaultness/unsafety, and the correct inherent method still have to hold. `media_commit_owner_is_handled` makes the corresponding trait exclusion before parsing an inherent `impl Type { ... }` boundary.

Neither change removes a trait body from the source image. The method-sized exception still requires the exact COMMIT method at the actual AST owner, reparses after removal, checks that exactly one method disappeared, proves the rollback method, and proves the media consumer's typed signature, entry prefix, synchronous write closure with all three admission guards, and returned settlement tail. The original channel proof still requires `channel_commit_call_count(...) == Some(1)` after the separately proven media call is neutralized. It retains the complete channel helper, handling tail, macro/alias exclusions, and global raw-COMMIT prohibition.

CRLF conversion is limited to the oracle's input image. Normalizing before both structural proof and literal mutation construction lets the same exact literals bind LF and CRLF checkouts. It does not rewrite product source or suppress any non-newline token. The added positive controls compare the complete residual source between LF and CRLF, including the actual Drop implementation and harmless trait bodies.

The new trait counterfactuals retain aliased `.commit(db)` and `.r#commit(db)` calls in the global scan for both candidate owner types and both line endings. The added raw-COMMIT trait cases require the forbidden statement to remain present after the bounded exceptions. Existing extra callers, method references, macro forwarding, duplicate/conditional/foreign owners, malformed syntax, changed COMMIT/ROLLBACK, altered unknown classification, missing retirement/autocommit checks, discarded handling, and guard controls remain present. Settlement-tail negatives now also run through CRLF source images.

I traced the literal mutation sites to the frozen production method/signature/prefix/tail. The added normalization does not make those mutations silently ineffective. The exact positive baseline and rejection/preservation assertions remain causal source-level controls; their actual execution is still an Actions obligation.

The existing Host `Cargo.toml` supplies pinned `syn = 2.0.119` with `full` and `parsing`; this patch adds no dependency or parser runtime. `capture_media_cleanup.py` continues to select the existing named media-owner and transaction-oracle tests, including every edited test. The Windows media lane invokes that capture. No changed test name is omitted from the existing required-positive list.

## 4. Cron projection and production delegation

The source anchor now selects the real same-file `list_openclaw_cron_jobs` and `trigger_openclaw_cron_job` functions in `source/host/app/src/main.rs`, matching CJI-07. Their inspected boundaries are the adjacent trigger helper and `impl UnifiedGatewayApi`, respectively. The indentation change in the `.first().filter(...)` assertion matches the extracted production helper.

All pre-existing behavioral source assertions survive: event schedules are not replaced with invented cron expressions; history stays newest-first; terminal status is not fabricated; enumeration errors are not swallowed; the trigger compares against `previous_run_id`, returns observed/accepted status, and does not synthesize a run UUID. Three added assertions pin list delegation, trigger delegation, and the native `self.call("runAgentAutomationNow", run_args)` execution port in their actual wrapper slices.

The production dispatcher still routes `cron.list`/`cron.trigger` to those wrappers. The adjacent App tests import and execute these exact helpers with the existing file-backed Session/AutomationRuntime owners; they also retain registered-dispatch and session-store wiring assertions. The runtime workflow still selects the Host cron contract and the five existing App cron identity controls. This repair changes no production identity resolver, authorization check, scheduler, run-history owner, or execution port.

## Limits and required follow-through

No parser, test, build, lint, generator, validator, fault injection, or capture was executed locally or on `htch-runtime`; none was dispatched by this reviewer. Local operations were read-only source/Git/API inspection, decompression for viewing committed shard rows, and byte/hash inspection. The only authored file is this report.

The source evidence supports approval of the six-path repair, with **zero blocking findings**. It does not assert that all repository contracts pass or that unrelated existing defects are absent. The integrator must preserve original failures, freeze the successor subject, and obtain the applicable exact-H Actions results for source materialization, private DB tests, transaction/oracle controls including Windows, and affected cron contracts. Source review cannot promote scoped verification or satisfy main-spec §17.5 final G01–G12/AC01–AC24, real ecosystem/platform, packaged-release, or independent acceptance requirements.
