# Original Actions audit — e19a0a2

## Result and exact subject

The original Linux job completed successfully. Independent evidence parsing confirms the 48 portable groups and all 91 selected App cases against exact immutable commit `e19a0a265e400a4c9b1d96520c84fb612a3843e8`, tree `5be2d7ca128dae5fefc8f7551e7088934daf19ac`. Windows media failed one existing test; its original failure remains preserved. macOS is queued. These are bounded observations for e19, not acceptance of the later DTR or media correction commits.

Runtime run: [41571103928, attempt 1](https://github.com/bhrumom/fabushi-desktop/actions/runs/41571103928). Source run: [41571103969, attempt 1](https://github.com/bhrumom/fabushi-desktop/actions/runs/41571103969). Fixed upstream remains `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.

| Original job | ID | Latest observed result |
| --- | --- | --- |
| Linux sqlite-native-path | 125703933269 | Completed, success |
| Windows media cleanup | 125703932910 | Completed, failure at step 6 |
| macOS rust-host | 125703933131 | Queued, no execution credit |
| Renderer | 125703933368 | Completed, success; API status only in this audit |
| Linux installation-key restore | 125703933453 | Completed, success; API status only in this audit |
| Pressure profiler | 125703934413 | Skipped |
| Platform worker | 125703934861 | Skipped |
| Source authority | 125703931828 | Completed, success |
| Source inventory persistence | 125704500035 | Completed, success; no carrier-to-subject evidence transfer |
| Original-host-log recovery | 125703933943 | Skipped |

The runtime run is not reported as globally successful while its Windows failure and queued macOS job remain. Original full API observations are preserved in the e19-prefixed JSON files. No workflow was dispatched, rerun or changed by this auditor.

## Portable evidence

The immutable workflow delegates to `projects/openclaw-rust/scripts/capture_portable_contracts.py`. Literal/AST source inspection derives 48 exact ordered command/name selections and 366 distinct inputs. Generated `*.ts` input membership is enumerated from the immutable Git tree, never the mutable checkout. All declared SHA-256 values match e19 bytes, including the extracted capture script, MSI additions, SGF producer/codec inputs and generated protocol files. Every captured command, required name list, raw log digest, observed name sequence, summary and exit state agrees with the original receipt and source-derived selection.

| Group | Original result | Exact raw-log SHA-256 |
| --- | --- | --- |
| DRT role-token authority | 27 passed; 0 failed/ignored/measured/filtered; each required name once | `0f6239a8aa82e65551f08898aca6458abc9f8cb991ee382e8214e7e8222c79c5` |
| DRT role-token HTTP | 10 passed; 0 failed/ignored/measured/filtered; each required name once | `b7d688e61a75c29fe7afe6dfd741f75435172a4751fd843178359437cd78ffae` |
| Retained streams | 20 passed; 0 failed/ignored/measured/filtered; original 14 plus 6 DRT cases | `785cb570f5fbad9bbe8099bc2600ac1bcc59943cdf7b665d346e5f204a301975` |
| MSI owner selection | 6 passed, 0 failed/ignored; 687 other library tests filtered | `9d5cd7f8830f1b202a893bf554e08a57bee1f294d87368cbce9f3ce7b9f86867` |
| SGF recorder selection | 30 passed, 0 failed/ignored; 663 other library tests filtered | `144d37d2390d9a5cd51d776b4e831a137aba73b2afc2868ce0538c0f41cbb671` |
| SGF three complete targets | 13 + 15 + 14 = 42 positive names, all required once | `9b878da95e97c91af128e8f49c82d2ba314e1d60b2e6dc93a41f09e7d8b1bf47` |
| Canonical admission contracts | 6 summaries: 12 + 29 + 15 + 12 + 19 + 4 = 91 unique successes, including 3 expected-panic successes | `6f89259996581baf9d9de36076e141ffe93b83e836d0773d2b73de3f63672ba9` |

The 14 canonical capture controls match source labels and expected booleans, and all original observed booleans agree. They remain synthetic oracle qualifications rather than product-test positives. Other groups retain their authored filtered and owned-child summary semantics; no cross-group summed product-test total is asserted. The same-H palette export digest also matches the archived JSON.

Original portable receipt SHA-256: `7d8a020178502e600e1159716a4350c789a89b0a6ffd0e82e66637fcc6241474`. The before-App and final Linux artifacts contain byte-identical copies of all 49 portable members. See `e19-portable-audit.json` for every exact command, positive-name occurrence, input Git blob/SHA-256 and per-log check; `e19-final-portable-equivalence.json` records the archive comparison.

## Actual App evidence

The original App receipt passes independent checks for 91 selected outer cases, 185 commands (2 compile commands, 91 exact listings, 91 exact executions and 1 startup command), and 247 source input hashes. Every stdout/stderr file matches its recorded digest and every command exits zero. Exact listings select precisely their authored case; every outer and owned-child summary is one passed, zero failed and zero ignored. Child summaries do not inflate the 91-case count.

The Cargo JSON has one matching `mahayana-app-host` binary artifact with test profile and the exact `source/host/app/src/main.rs` path. Its path matches all exact executions and belongs to the reported owned target directory. The binary SHA-256 `2ec616dbbc6dcba16feae220c6d87830040114b38ac895280959d7d47bfca234` is an Actions-captured value; binary bytes are absent from the archive, so it is not independently rehashed here.

All three new DRT App cases pass:

- `tests::device_pairing_revocation_tests::token_revocation_shipping_operator_retirement_preserves_node_and_local_projection_after_reopen`
- `tests::device_pairing_revocation_tests::token_revocation_shipping_node_retirement_keeps_pairing_history_and_other_role_after_reopen`
- `tests::device_pairing_revocation_tests::token_revocation_shipping_hook_keeps_paired_proof_native_ceiling_and_existing_owner`

The separate startup target has seven required names, each once, and one seven-passed/zero-failed/zero-ignored summary. Separate filtered link-preview App selections are not added to the 91-case total. Full machine evidence is in `e19-app-audit.json`.

## Windows media failure and owner boundary

Original `explicit-html-consumers` command:

```text
cargo test --locked --manifest-path source/host/Cargo.toml --test openclaw_web_media_contract -- --test-threads=1
```

It exits 101, capture exits 0. The raw result is **22 passed, 1 failed, 0 ignored, 0 measured, 0 filtered**. `test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references` panics at `tests\openclaw_web_media_contract.rs:253:48` with:

```text
rooted drive remains valid: LocalAccess { code: "invalid-file-url", message: "File URL cannot be converted to an absolute local path" }
```

The capture is sound and fails closed. The raw target log SHA-256 is `7bf08cb08fb238ba582f12b4c323f18aa2209444346b40a53fab3e83e9d089d0`. All other 11 Windows media groups pass. All 12 Linux media groups pass. Both receipts have the exact source-selected commands and names, 37 unchanged source input identities and 13 agreeing capture controls. Windows raw CRLF hashes are retained separately from normalized Git blob identities; all observed raw hashes match the expected LF or CRLF form of source.

**Classification: TEST_FLAW, based on locked dependency source inference.** The test treats four drive-root spellings as positives. The original panic does not include the current reference, so the raw run does not isolate which iteration failed. The locked `url` 2.5.8 archive was downloaded read-only and its SHA-256 matches Cargo.lock: `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`.

Source inspection explains the unsupported triple-slash pipe-drive positive `file:///C|/`: an initial empty segment shifts `C|` away from the parser's first-drive normalization branch; leading empty segments are collapsed afterward; the Windows converter then requires a colon or encoded colon and refuses the remaining pipe. The supported single-slash form `file:/C|/` takes the normalization path. This is a source-derived diagnosis, not a runtime-observed identification of the failing iteration and not evidence that a correction passes.

Recovery belongs to the existing Attachments/FUA media owner. Preserve the test name/count and all prior negative assertions, make the unsupported spelling an explicit refusal, keep the other rooted positives, add the supported normalization form and include each reference in diagnostics. The parent reports an owner correction at `78aa967cc9400bb5c207441b2a78dbf7021d596c`; this audit neither edits that work nor grants it runtime credit. Exact identities and bounded diagnosis are in `e19-windows-failure-diagnosis.json`.

## Source structural evidence

The source artifact and its original job log confirm tracked-tree and structural G02 success for 62,130 entries. The archived recursive inventory digest matches the receipt, summary and digest file. Its exact path/mode/type/blob tuples match the archived raw NUL-delimited Git-tree listing.

The structural receipt is generated **after** the workflow runs its inventory expansion and shard materialization. Of 26 repository input hashes, 23 match immutable e19. The index, shard manifest and parity ledger reflect deliberate generated state and differ from pre-expansion H. Those generated raw files are not in this artifact, so their reported bytes cannot be independently reconciled here. The 27th input is the archived generated upstream inventory and matches. The receipt's input-map digest is internally consistent. This limitation is preserved explicitly in `e19-source-audit.json`; the receipt is not misrepresented as 27 unchanged source files.

The source receipt leaves dependency/LFS closure open, final G02 not run, independent review pending, and all eight completion flags false. It establishes neither full semantic source closure nor unit/whole-product acceptance.

## Original archive and log identities

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| 13201218844 — Linux before App | 441280 | `e2c0103828f1ca8d8c666ec7ab0f0daaa49c8081c30cf12bc0edd5666b0b8c13` |
| 13202122194 — final Linux | 688802 | `88e0fe123cc3775600fdb496c560554d3fc1c5f57dc5ada75310003bcdbaf18f` |
| 13201234032 — Windows media | 90117 | `e6724c09ab10215588c4a9ccb9c632b477b5af66cdd4e94af8e70c58ef3c66e2` |
| 13201108946 — Linux media | 85816 | `9b0dd52d358bb7cb639ca86bb09f9d9c4c49c8b3bd2cdd2b8eb93a61be90f966` |
| 13201392577 — source authority | 4951796 | `425d043d55b6f57a5d8e2629bdf06d15dfdf2861ec86ef7918d5e13b4c285140` |

All ZIP hashes match GitHub's artifact metadata. Complete decoded original Linux job log: 5,234,930 bytes, SHA-256 `69ffbe5180a7105e5b2fccc274810c271dc62f8843575082e9c7b6686b693aa4`. Complete Windows job log: 824,240 bytes, SHA-256 `01c913c13b02fd8bf3103166ed822a053105f3736462a37a75291ea20a39f3e6`. Complete source-authority job log: 31,521 bytes, SHA-256 `d91c4dcd6992c80a73252d02e6ac01f13f78d4c21036f9f02c430fd44c9f5242`. Decoded API log text is preserved exactly as supplied; no ANSI/timestamp stripping is used for those hashes.

## Authority and continuation

This campaign used immutable Git bytes, literal/AST source extraction and parsing of original Actions files. It ran no candidate builds/tests/lint/generators/schema validators or synthetic oracle reproductions locally or on htch. No product, workflow or GitHub state was changed. No prior blocked H5593 raw archive was published or reused for e19 credit. Earlier failed captures and admission failures remain intact.

The configured model/effort is recorded from the parent assignment as gpt-6-astra/xhigh, not as an introspection claim. The aggregate machine proof is `e19-evidence-audit.json`. All eight global flags, unit verification, independent acceptance and global closure remain false.

Next evidence subject is DTR `113c32276b99ed668be095e0be6ed164dffdf46d`, with its own runtime/source runs. Neither these e19 results nor the media correction checkpoint is relabeled as DTR execution evidence.
