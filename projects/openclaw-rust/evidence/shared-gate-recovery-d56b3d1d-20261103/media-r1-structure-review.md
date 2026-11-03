# SG74-MEDIA-v1 round 1 — blind structure, reuse and performance review

## Verdict and limits

**APPROVE for this source-review lens, with one non-blocking P3 byte-policy edge noted below. No blocking structure, reuse or performance finding was established in the frozen three-file delta.** This is a source review of high-risk transaction test/oracle recovery. It is not a production behavior rewrite review, a new-H test result, or full MSU/OCRM acceptance.

The reviewed repair preserves the independently literal durable receipt oracle, adds the actual presentation-page check, narrows inherent-impl selection without deleting trait bodies from the inspected image, and binds the media mutation families to an accepted actual-source baseline. No source, developer index, private index or Git ref was changed by this reviewer. No local or htch executable project verification, Rust parser execution, test, generator, schema check, build, lint, benchmark or project import was run. Commands were read-only Git/source inspection and Python standard-library reading/hash inspection of original bytes and original ZIP evidence. Only this report was written.

Requested lens setting: `gpt-6.1-sol`, effort `xhigh`, because the assignment covers high-risk transaction recovery, source exceptions, normalization and oracle sensitivity. The assignment names this setting, and AGENTS.md pins this lens to that model when supported. The worker has no reliable runtime model/effort introspection surface; it cannot independently attest that the requested setting was actually applied. Treat the setting as requested/inherited, with actual host application unobservable to this worker. No extra worker or CLI process was started to tune it.

## Frozen authority

- Repository: `bhrumom/fabushi-desktop`; read-only object source `/tmp/ocrm-source-84120837.git`.
- Exact base: `b15e58957a597997363e1273f2f32a222365c310`.
- Snapshot: `1cf5d76383b363535ba33864273d06a2bebe7b65`.
- Snapshot tree independently read from Git: `86f7d2de95296a494ac89f4b06f11b85fe16bbcf`.
- Acquisition context from manifest: explicit source copies plus a private index over the exact base; dedicated bare HEAD is unborn; no developer checkout/index is the review subject. This report does not substitute a guessed HEAD for the explicit snapshot.
- Complete delta: `/tmp/shared-media-74-r1.diff`, 32,962 bytes, SHA-256 `4f3ca38e7183442d94dd86c1f6c8a992487bc4b89af9a9b97a0bb74c922c19f6`.
- The Git-produced binary/full-index diff between the exact base and snapshot independently has the same length and SHA-256. Git reports precisely three modified paths, 220 insertions and 22 deletions.

| Path | Snapshot blob | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `06b9c5c721aec53449f68684b352ebde89f130a6` | 17,241 | `d4481e0f03074d4c8dae2d0be1051247c58cd25e8db5d4cf7baf04ded042231b` |
| `source/host/src/extensions/session/agent_db.rs` | `8ffec064fad5e220fa1b2dadbb3fe9bc975f0d67` | 427,027 | `5cf63b89b1d2c1d9b4c283b60257376cddc087ac4051aa5936ff4cadf0bd2ea3` |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `85e3552d92e96037ea44a3923dac89a342166179` | 170,216 | `6415e02b5915074acba5e9b3ed5e7d894fb8f02f91d84bb0b4a4e46bebcf0802` |

## Read coverage and blind-review discipline

I read the entire supplied delta and the complete 253-line MSU-v1/SG74-MEDIA-v1 specification. I read root `AGENTS.md` in bounded overlapping ranges covering the complete file, including the independent-review, immutable-snapshot, effort/model and Actions-only rules. Repository tree enumeration found no nested AGENTS.md on the three scoped paths; the other nested instructions are under the unrelated Mahayana TUI. I read applicable OCRM §17, including the historical section and the current OCRM-EXEC-02 amendment (§17.1–17.5). The source-of-truth introduction/authority/constraints (lines 1–94) and module-map introduction plus relevant Session/media boundary rows were read as supporting context; I do not claim complete reading of their historical appendices or the complete OCRM specification.

Deep adjacent source reads, all at the frozen snapshot:

- `agent_db.rs`: 4710–5132 (fault lease/commands and the complete media transaction owner through the next-owner boundary), 5835–5905 (owner entries/page/window methods), 6230–6255 (Drop and command-helper boundary), 6655–6720 (receipt shape), 7760–7915 (owner decoding, retained replay and receipt-ID reads), 9080–9648 (complete media-settlement private fixture/tests and adjacent boundary).
- `openclaw_sessions_and_config_contract.rs`: 710–980 (all admission owner registration and guard traversal), 1510–2875 (all changed transaction-oracle functions, media constants, inherent owner proof, bounded method removal, global caller traversal, retained mutations and next-test boundaries).
- `agent_db_serde.rs`: 1–225, especially presentation versus owner decoders and retained control-record kinds.
- `agent_db_transcript_pages.rs`: 1–180, including the actual SQL page path and presentation decoder.
- `production.rs`: 1–65, 1280–1340, 1720–1835 (receipt hiding, public transcript consumers and durable media recovery receipt reader).
- Complete `projects/openclaw-rust/scripts/capture_media_cleanup.py` (through its main guard), plus targeted current workflow ranges for exact-H dispatch/checkout, native Linux/macOS/Windows capture, eight App test names and actual Host compiler-artifact feature observations. Host/App Cargo feature and dependency declarations were read.
- A narrowly scoped search for Rust CRLF normalization precedents found direct test-local replacements in `box_store_sync_production_wiring_contract.rs` and the new helper. No existing shared normalization API requiring reuse was established by that search; I did not claim full reading of those unrelated tests.

Original evidence inspection was limited to the two expressly allowed ZIPs: member inventories, original receipt identities/input hashes, and the post-`running` sections of `media-settlement-owner.log` and `transaction-commit-owners.log`. Compiler warning preambles were not treated as fully read. I did not open prior, peer, root decision, worker or diagnostic reports. The first broad filename discovery listed some unrelated paths but did not read their report contents.

## Findings on the repair

### SG74-M01: receipt authority and presentation remain separate

The only `agent_db.rs` edits are inside `#[cfg(test)] mod media_settlement_tests`. `Fixture::rows` reads physical rows from a separate file-backed connection in sequence order (9168–9175). `assert_committed_storage` independently spells the exact two-row vector and every receipt field, then checks the fully cleared admission record and original media bytes (9189–9206). It does not call the production receipt builder to derive the expected receipt.

The lost-ACK test still checks claimed/applied fault state, invoking-owner retirement and handle count, no recovery callback, no publication, unchanged write generation, refusal of a repeated call, and empty reads from the retired owner (9340–9358). Both fresh reopens remain. Each now compares the owner projection with the physical image and separately requires the real page API to return exactly the send (9360–9373). The independent literal storage check is retained before the loop and after every fresh close. Consequently `fixture.rows()` has not replaced the literal storage oracle with an unchecked self-comparison.

The exact-owner fault-isolation test retains the three mismatch controls, independent same-file owner commit, unclaimed original lease before replay, true claimed/applied state on the invoking owner, `assert_retired(1)`, and usability of the other owner (9551–9593). Its new owner-image and presentation checks are followed by the same literal storage/bytes assertion and zero-publication check.

These expectations match actual adjacent APIs: `get_transcript_entries` uses `parse_owner_transcript_entry`, which deliberately keeps settlement control records; page reads use `parse_transcript_entry`, which does not expose those records. Session public transcript methods independently hide work receipts, and first-open media recovery validates the durable receipt by ID. Changing production decoding would have damaged receipt authority. This delta correctly repairs the two expectations inside the existing tests.

### SG74-M02: precise exception boundaries remain intact

The early `trait_.is_some()` filter in `media_commit_inherent_impl` (1704) and matching filter in the second source parser (1801) change candidate selection only. Neither removes a trait implementation from `source`. The existing `Drop for SandAgentDb` is now irrelevant to inherent-owner uniqueness, while the complete trait source remains available to later traversal and the final raw-COMMIT prohibition.

The helper still requires one exact unqualified inherent owner, rejects conditional/unsafe/default/generic/where-clause variants, requires one method, and proves that removing the uniquely matching whole literal actually removes that AST-owned method. The media owner still pins its public typed signature, exact prefix, directly invoked synchronous write closure with three guards, complete returned tail and unique known commit call. Rollback is checked and retained. The only actual helper body removed from scope is the exact checked commit method; only its checked owner call is replaced by the oracle-local sentinel. The channel proof retains its original `Some(1)` caller requirement and exact helper/tail proof.

The new benign trait controls cover both relevant types and retain the complete trait text and Drop in the returned inspection image. Trait-only owner/helper mutations are refused. Trait-contained raw COMMIT controls prove that the raw statement remains in the returned image; trait-contained alias/raw-identifier/associated/function-reference controls must revoke the exception. They complement the retained free-function, helper-sibling, macro, conditional, malformed, discarded-tail, rollback and retirement mutations. No original mutation was removed by this delta.

### SG74-M03: the actual LF/CRLF repair is narrow

`transaction_commit_source_image` uses one explicit paired-CRLF replacement and otherwise borrows the original string (1693–1701). The new local Cow variables outlive all borrows inside their functions; returned images are owned Strings produced by removal/replacement, so no borrowed temporary escapes. `into_owned()` at the actual fixture boundary materializes a stable normalized source before any `find` or `replacen`. Windows no longer searches CRLF source for LF-only signature literals.

For the exact snapshot, all three scoped blobs contain zero CR bytes. LF baselines and derived CRLF baselines therefore have an unambiguous common image. The actual fixture first requires the composed exception to accept the unchanged source and its CRLF counterpart and requires each mutation boundary to occur exactly once. Every media negative family invokes that fixture; the CRLF assertion helper refuses unchanged fixtures, proves reversible line-ending conversion of the entire changed image and requires rejection. Raw-COMMIT controls correctly distinguish adapter proof success from retained forbidden statement detection. They do not mistakenly expect the bounded adapter proof itself to reject unrelated raw SQL.

The channel composition entry, channel AST helper and media entry all use the same helper policy. Channel changed-tail/helper families gain LF/CRLF baseline controls plus changed-fixture assertions and CRLF rejection. Existing unrelated tests are not globally normalized or filtered. No repository Git EOL setting, production parser, schema or capture acceptance rule changes.

### Non-blocking finding STR-P3-01: repeated normalization can consume a retained CR in an overlapping sequence

**Severity/disposition:** P3, NON_BLOCKING for this frozen exact source. This is a concrete edge in the stated byte-preservation policy, not an observed new-H failure or evidence of weakened transaction behavior.

**Path/symbol:** `source/host/tests/openclaw_sessions_and_config_contract.rs`, `transaction_commit_source_image` (1695–1699), composed through `source_without_owned_media_commit_adapter` (1873), `source_without_owned_channel_commit_adapter` (2491) and `agent_db_transaction_owner_source` (2403).

**Trigger and falsifiable consequence:** consider source text outside the pinned helper/owner literals containing bytes `CR CR LF`, for example a comment ending that way. One stated paired-CRLF pass maps `a\r\r\nb` to `a\r\nb`, preserving the first CR. A second invocation on that intermediate image maps it to `a\nb`. Because the composed oracle normalizes at multiple layers, the complete returned scope can lose the first retained CR. The positive control at 1942 covers an isolated CR between ordinary characters, not this overlap. This conclusion follows from the exact replacement and call graph; no normalizer or parser test was executed locally.

**Why non-blocking here:** original-byte inspection proves zero CR in the current source, and every CRLF control derived from it is formed from clean LF source. This edge does not establish a false pass, hidden caller or failing mutation in the declared repair subject. Elevating it to a blocking regression for the current subject would exceed the evidence.

**Specific follow-up:** if arbitrary mixed/overlapping CR inputs are to share the stated preservation contract, normalize once at an externally supplied image boundary and pass the normalized image through internal functions without normalizing it again. Preserve standalone test entry coverage through explicit wrappers. Add a complete-composition `CR CR LF` retention control; do not make repeated replacement a whitespace-folding policy. A future test should compare the composed retained image with exactly one normalization pass, rather than asserting idempotence of this non-idempotent policy.

## Reuse, resource cost and actual evidence

The normalizer and fixture remain private to the existing contract test binary. They reuse the existing actual source, media AST proof, original channel caller walker and capture workflow. There is no new runtime owner, registry, global cache, dependency or general parser framework. Retaining full-source inspection is justified by hidden raw/aliased/macro callers; replacing it with cropped fragments would invalidate the safety claim.

Resource cost increases in tests. The actual fixture owns roughly 427 KB, derives a CRLF image and invokes the complete proof twice before each applicable family. Mutations also derive CRLF images and re-run the existing full-source parser chain. Existing array fixtures already retain multiple source copies at once; the additional trait controls add further bounded copies. LF normalization itself is a linear scan with no allocation; a CRLF input has a scan plus one linear allocation/replacement. Normalized downstream inputs ordinarily borrow, so they do not repeatedly allocate simply because several entrypoints use the helper. The primary cost is repeated parsing and token-string comparison, not the small Cow helper. There is no new production-path CPU, I/O, power or memory cost.

No timing, peak-memory or throughput result for the repaired snapshot exists in this review. The old 27-test group reports 26.48 seconds on Linux and 21.42 seconds on Windows, but it rejected the actual baseline and skipped load-bearing positive work. Those timings cannot establish either the repaired suite's budget or mutation sensitivity. The current serial native group is the appropriate place to observe the actual cost; there is no source-based evidence here that it breaches the 45-minute Windows job or other current job budget. A speculative cache/refactor or less complete inspection is not a justified blocker.

Original archive hashes inspected:

- Linux ZIP: `284a316258ebf9b60dd29d781c6aceb44ef3d1ca351a3337e0945efd7b2699b5`.
- Windows ZIP: `184296dcafd2b230021ed8523d4e159cc34473d4436a5a9f5198c28c1bcac96f`.

Both original receipts identify H74 `74f11d3d31c6245ac5886dc0181460b2d4349744`, run `41580539491`, attempt 1, with failed-or-incomplete result. Original owner logs show 11/2 on both platforms; commit logs show Linux 21/6 and Windows 20/7. The Windows additional failure is the literal signature lookup. Reading the base blob and converting its original LF bytes to CRLF reproduces the recorded Windows byte counts and raw hashes exactly: owner 436,762 bytes / `276b6c58a5260c7c0c3fd1561d65249eba307827767c894e149ba0e4bcb9fdb9`; oracle 165,691 bytes / `119cf6ef914b4342719f63cfc7a032c754152b0c6c04a0f9d031d665ef83d946`. This is checkout-text provenance, not new execution.

The unchanged capture retains 13 exact owner names and 27 exact transaction names and runs both groups serially. Its assessment requires unique complete fixed-name positives, exact successful counts, zero failures/ignored tests, command/capture success and unchanged input hashes. The workflow also retains native platform capture, admission checks, eight exact App/Session/Runner cases and actual feature-off/on Host library compiler-artifact observations. The repair adds controls inside existing fixed names, so no capture-name update or broad acceptance exception is needed.

**Required next evidence remains unmet in this report:** Actions execution against the newly integrated/materialized H for the complete affected fixed-name native groups, existing admission/App/shipping feature checks and applicable platform gates. Original rejection-test `ok` results cannot be carried forward as mutation-sensitivity proof. Source mapping is not strict committed inventory acceptance; no ledger, global flag, packaged acceptance or final MSU/OCRM claim is advanced by this verdict.
