# R1 structure, reuse and performance review

**Verdict: APPROVE — source-only.** No concrete, reproducible structure, reuse or performance defect was found in the complete declared six-path change. There are no blocking or non-blocking findings requiring a fix from this review.

## Immutable subject and execution setting

- Repository: `bhrumom/fabushi-desktop` at `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Canonical branch: `codex/openclaw-rust-fusion-chat-20261101-writer`.
- Exact base commit: `4f0639321c4ae03cc7dbef1d8e49dea0f042e180`.
- Exact base tree: `cbc9418ee7572f44d59e067e450a67f6c0bc22a0`.
- Frozen candidate tree: `406f8668a83a30a24c83b442b97d3a2b4d64927f`.
- Frozen patch: `r1-complete.patch`, SHA-256 `17280ae7b638e814a51ea05e25970abfc42481c495ac5c08e6a4b465f7922995` (independently read back with `sha256sum`).
- Actual worker launch: `fork_turns: "none"`, `model: "gpt-6.1-sol"`, `reasoning_effort: "xhigh"`; the parent confirmed the exposed launch fields were supported and applied, with no fallback. The complete assignment is high-risk shared transaction/evidence-oracle correction, which justifies xhigh rather than a bookkeeping classification.

The review was fresh and blind. I read no other review report or prior runner arbitration. The full frozen patch and its manifest were read, and Git's exact-tree changed-path list independently matched the six declared paths. Source reads used `GIT_NO_LAZY_FETCH=1 git show 406f8668a83a30a24c83b442b97d3a2b4d64927f:<path>`, not the mutable checkout. Adjacent missing specification blobs were read through GitHub's file API at exact base `4f0639321c4ae03cc7dbef1d8e49dea0f042e180`; the returned identities matched the candidate tree entries: MSU-v1 `e42ea96e12c4647c23d98154fabd4a87749ea5c7`, CJI-v1 `988f36d855e3efffc5b5563006dddf926393fcbe`, and MAC-v1 `bab905e889a472027a906b551a874081131cc85e`.

## Governing contracts and adjacent consumers inspected

The review applied root `AGENTS.md`, the relevant OCRM Source of Truth/module-owner rules, OCRM-001 §17.2/§17.5, MSU-v1 (especially MSU-06/MSU-09), CJI-v1 (especially CJI-07 and its bounded wiring assertion), and the MSU correction in MAC-v1. Adjacent source inspection included:

- The existing inventory owner chain in `source_review.py`, `verify_audit_integrity.py`, `expand_inventory.py`, `shard_inventory.py`, `validate_p0.py`, existing pending-mapping negative controls, and `.github/workflows/ocrm-p0-authority.yml`.
- Exact candidate inventory manifest and the five affected canonical rows in shards 0036, 0043 and 0044; compressed records were decoded only to inspect their source bytes.
- The complete media proof and counterfactuals, original channel proof/token scanner and their composed consumer in `openclaw_sessions_and_config_contract.rs`.
- `SandAgentDb::append_transcript_entries_and_release_media`, `MediaSettlementCommands::{commit,rollback}`, `Drop for SandAgentDb`, receipt construction, raw/owner transcript reading, and the affected private media fixtures/tests.
- The shipping `list_openclaw_cron_jobs`/`trigger_openclaw_cron_job` helpers, `UnifiedGatewayApi` wrappers, the changed shared cron source assertion, and the adjacent App test seam that imports and executes those production helpers with existing Session/AutomationRuntime owners.

## Assessment

### Inventory declarations and evidence references

`inventory/index.json` reuses the existing `pending_mapping_additions` interface rather than altering generated shards or creating another source-accounting owner. Its digest equals the existing canonical manifest digest `8994988c264876be709930e2f87c6b60ae124a6077a833ccd324dc1117d26d93`. Each declared responsibility/path/blob triple equals an existing `OCRM-RESP-MEDIA-FINAL-CUSTODY-001` ledger source reference and an exact-U canonical row. All five rows currently lack that responsibility; `agent-run-media.ts` also retains its separate existing read-scope responsibility. The delta therefore describes five missing edges, not five newly read or accepted sources.

The existing pre-expansion check admits only an exact declared missing edge with mapped/not-run/pending responsibility state and checks that every pending declaration is actually used. The existing expansion derives responsibility IDs from the canonical ledger; shard writing then demands that each declared exact edge was materialized and removes the pending input. Normal validation continues to require actual reverse links. The Actions workflow already orders these operations. No schema/validator change or new state is introduced. Historical rows, read ranges, counts, source identities, and global flags are not hand-promoted by this patch.

The two ledger and four RTM edits repair references to exact-tree files `fixed-u-contract.md` and `current-owner-audit.md`. The target files exist in the candidate tree; the replaced long names do not. No requirement IDs, acceptance/mapping states, evidence outcomes or flags change.

### Private committed-state fixture

`Fixture::expected_committed_entries` extracts the previously inline, explicit send-plus-receipt expectation and reuses it for the two reader assertions. It does not call the production receipt constructor. The expected vector still pins row order and all receipt fields: receipt ID/kind, session, media work, lifecycle revision, owned path, and transcript entry IDs. The separate file-backed reader assertion remains in `assert_committed_storage`; fence release and retained byte assertions remain independent.

The expectation matches the existing owner contract: the transaction inserts the send and then its settlement receipt, while the owner transcript reader uses `parse_owner_transcript_entry` and retains both. The production projection and schema are unchanged. Lost-ACK reopen observations and the second live owner's read now use the same exact durable-row expectation. Existing real-fault-applied, connection-retirement, no-publication, released-admission and file-preservation assertions remain intact. The extraction removes duplicated expectation construction without adding product abstractions.

### Media proof, traits and source transport

Filtering traits out of `media_commit_inherent_impl` and the parallel owner-body parser correctly limits ownership proof to the unique actual inherent implementation. The real `Drop for SandAgentDb` can coexist with that owner; it is not another inherent transaction owner. The change does not exempt a whole implementation or remove a trait body from the global input.

The existing method-sized proof remains intact: exact helper method text must be unique; reparsing after removal must remove precisely that inherent method; the real typed media owner must preserve its signature, admission-bearing synchronous write closure and complete returned settlement tail. Only the separately proven commit method and the exact already-checked media call are normalized out before composition with the unchanged channel oracle. Rollback and all other source remain in scope, and the original channel scanner still requires its sole argument-taking commit call.

New controls make the distinction observable: the actual Drop body and harmless traits on both owner types survive the residual source; aliased and raw-identifier commit calls inside traits must still be rejected; raw COMMITs inserted into traits must still reach the global prohibition. Existing duplicate/conditional/foreign owner, command, alias/macro, changed settlement and admission negatives remain present.

CRLF conversion is limited to the oracle's source image and negative-fixture construction. It does not rewrite production source or persisted data. The tests compare complete LF/CRLF residual images, including surviving traits, and retain CRLF alias/raw-COMMIT/settlement-tail negative checks. This preserves the original channel proof while allowing the media literal proof to operate on logical source newlines.

### Cron helper and entry wiring

The shared source assertion now slices the existing private production helpers at their actual adjacent boundaries rather than inspecting thin wrappers for logic that has moved. The projection requirements are preserved: actual event schedule representation, newest-first history, observed status, previous-run comparison, accepted/finished classification, and no invented run identity or swallowed enumeration error. The changed indentation anchor matches the actual extracted helper.

The new wrapper slices independently require the real wrappers to pass their existing store/arguments to those helpers and require trigger dispatch through `self.call("runAgentAutomationNow", run_args)`. This is the bounded wiring proof required by CJI-07; actual behavioral tests continue to exercise the production helpers through file-backed existing owners. No copied production implementation, parallel resolver/store, or new execution seam is introduced.

### Performance and scope

The Rust runtime implementation is unchanged by these six paths. The only AgentDb edit is inside its private `cfg(test)` module. Additional allocation, parsing and LF/CRLF images belong to test oracles; the fixture helper constructs only the same small explicit expected vector. The pending inventory declaration contains five entries and uses existing Actions materialization. Source inspection reveals no material production memory, latency, build-dependency or hot-path consequence, and no unnecessary abstraction that would justify a finding.

## Verification boundary

No parser, test, build, lint, generator, validator, benchmark, local harness or htch executable validation was run. Git/API/source, source decoding, and byte/hash inspection alone support this verdict. The report approves the structure/reuse/performance of the frozen source delta; it does not claim these tests passed, resolve prior failed Actions evidence, mark the batch verified, or accept runtime behavior or the whole migration. Successor exact-revision Actions and the unchanged final OCRM §17.5 gates remain required.
