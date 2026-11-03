# Original DTR Actions evidence — 113c322

## Exact subject and result

Subject `113c32276b99ed668be095e0be6ed164dffdf46d`, tree `7644b264bf72ef57f7333a7f4f8d7accd2dd7af8`; runtime [41572993677 attempt 1](https://github.com/bhrumom/fabushi-desktop/actions/runs/41572993677), Linux job `125709987601`. Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.

The original portable capture executes all 50 source-selected groups. Forty-nine pass. The new DTR HTTP target has **12 passed and 1 failed**. The new authority target passes all 44 required cases; the retained-stream target passes all 27. This is an incomplete DTR campaign, not a passing DTR unit. Actual App step 19 is explicitly **skipped** after portable step 12 fails. The source workflow succeeds; Windows media retains its independent pre-correction fixture failure; macOS is queued. No result here is relabeled from e19 or assigned to the later media, PRID or R3 checkpoints.

## DTR HTTP failure: TEST_FLAW

Original command:

```text
cargo test --locked --manifest-path source/host/Cargo.toml --test device_token_rotation_wire_gateway_contract -- --test-threads=1
```

The original `device-token-rotation-wire-gateway.log` is 29,902 bytes, SHA-256 `136f0a784ad135dc06be80b4d0aa1f196efb12b9da8b4b3d638dcc940aa3efd2`. Cargo exits 101 and capture exits 0; the complete libtest summary is 12 passed, 1 failed, 0 ignored, 0 measured and 0 filtered. The failed test is:

```text
token_rotate_wire_http_issued_nel_literal_never_becomes_pairing_or_admin_at_adjacent_consumers
```

Its panic is at `tests/device_token_rotation_wire_gateway_contract.rs:1146:13`, comparing `errors.len()` with `reports + 1`: observed 0, expected 1.

**Direct cause:** the new test asks for a 403 permission refusal, then demands one error-report callback. Exact-113 `gateway_server.rs` calls `on_command_error` only inside `if status >= 500`; it sends the 403 response without that callback. The fixture's callback simply records reports it receives. The client reads to connection EOF, and the production report decision occurs before response writing. This is neither a timing race nor a capture/parser failure. A delay or unexplained rerun cannot make an intentionally unreported 403 satisfy this assertion.

The following assertions in this test were reached before the failed report-count assertion: a real literal successor was issued and authenticated; ordinary-route authentication returned 200; the first management operation returned 403; response redaction passed; complete SQL state remained unchanged; and authentication/real-owner-hook counters matched. The failure prevents subsequent target/method iterations and the final genuine-admin positive controls in this one test from executing. Those unexecuted checks receive no success credit.

The safe correction is confined to the test oracle: preserve the exact 403, unchanged state/revision, real owner-hook, literal-scope and redaction assertions; expect no report increment for ordinary 403 under the current reporting contract. Retain the independent 500-report controls and ordinary positive cases. There is no basis to expand product reporting policy or remove the authorization checks.

| Source | Git blob | SHA-256 |
| --- | --- | --- |
| HTTP test | `e719636a00f80d18cc635ab7413a17b37c8d7046` | `d758e24465ab026b2b862450e2f2736ee04dc413c6ec1815c7e7738588b9bc4e` |
| Gateway server | `2cc83cf387ae37c3b1c28663f72d9d0492e84ccd` | `965935140600220ff2eaedc19c85ab325325b270677edc924663497db42b337a` |
| Portable capture script | `e683bb4494c3d28ca8bc466472eb3d198578198e` | `b4d9faeec15681a15301a5e8bc5701ae91cda20721ad86dfef0ab10110aff166` |

The durable pre-fix handoff is `113-dtr-http-failure-diagnosis.json`. Classification was delivered to the parent before any auditor edit or rerun; this auditor performed neither.

## Preserved positive evidence and authored selection

The original 50 ordered commands and required-name arrays match immutable 113 source. All 370 declared input SHA-256 values match the commit, including the current delegated script, four added DTR inputs, MSI/SGF inputs and the exact generated-protocol membership. All 50 original raw-log hashes, observed name sequences and parsed summaries agree with the original receipt. Evidence consistency is true; the captured product result correctly remains failed-or-incomplete.

| Selection | Authored requirement | Original observation |
| --- | --- | --- |
| Wire-rotation authority | Complete 44-case target | 44 passed; all names once; zero failed/ignored/measured/filtered |
| Wire-rotation HTTP | Complete 13-case target | 12 passed, 1 failed; target not approved |
| Retained streams | Complete 27-case target | 27 passed; all names once; zero failed/ignored/measured/filtered |
| Prior DRT selected-role authority/HTTP | 27 / 10 | Both pass, exact complete counts |
| Prior DVR whole-device authority/HTTP | 26 / 8 | Both pass, exact complete counts |
| Prior DPA authority/HTTP | 9 / 8 | Both pass with original authored gates |
| Other portable groups | Existing selections retained | Pass; raw logs and source lists preserved |
| Actual App | 94 outer cases, 191 commands, 249 inputs, including 3 new DTR cases | Source expectations only; step 19 skipped |

The retained-stream selection preserves all 20 prior names and adds seven DTR names. The portable group count grows from 48 to 50 without dropping prior groups. The App selection preserves all 91 prior cases and adds exactly the three shipping wire-rotation cases. Its expected 191 commands are two compile commands, 94 exact listings, 94 exact executions and one startup command. No App binary or App test pass is inferred from the portable library/integration results.

The same-H source extraction is `113-source-expectations.json`; the exact delta is `113-vs-e19-source-selection-delta.json`; the evidence proof is `113-portable-audit.json`. Each group retains command, required-name occurrence counts, raw-log SHA-256, summaries, exit state and strict-count checks. Owned child summaries and filtered selections remain distinct rather than summed into a misleading cross-group case total.

## Separate media and source evidence

Windows job `125709987605` fails `explicit-html-consumers`, with 22 passed and one failed. The raw 113 failure independently repeats the pre-correction assertion at `openclaw_web_media_contract.rs:253:48`: rooted drive expected valid but returned `invalid-file-url`. Raw target-log SHA-256 is `ce3defc5f940f974b367b580c3cb078ff9b544d41a908e564d0b531ba0680fd4`; cargo 101, capture 0. Its exact failed loop reference is absent from the original log. The previously documented locked-parser source inference classifies the unsupported triple-slash pipe-drive positive as TEST_FLAW; this is not a runtime-isolated identification of the iteration or proof that the owner's 78aa correction passes.

Both original 113 media artifacts independently match 12 source-selected commands, 37 input identities, all raw-log hashes, before/after identity stability and 13 oracle qualification controls. Linux passes all 12 groups; Windows passes the other 11. Raw Windows CRLF hashes and normalized Git blob identities remain separate.

Source run [41572993673](https://github.com/bhrumom/fabushi-desktop/actions/runs/41572993673) succeeds for exact 113. Its artifact has a consistent 62,130-entry inventory matching the raw Git-tree tuples. All 26 repository input hashes match immutable 113 and its one archived generated manifest matches as well. This differs from e19's three materialization differences; no stale assumption was carried forward. The initial source-audit draft documenting that expected-difference assumption is retained separately, and the corrected audit records the directly observed zero differences.

Tracked-tree and structural G02 pass; dependency/LFS closure stays open, final G02 is not run, independent review is pending and all eight completion flags remain false. The source artifact is structural evidence, not full source/semantic closure or product acceptance.

## Original artifact identities

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| 13202711485 — Linux before App | 454256 | `0162b1739d3cc812fa73377e818e3741d50641241447be8a2553cfc0aef6492c` |
| 13202815569 — Windows media | 90146 | `11eb22afdeb1b93ded828a44634e1d46faf589a71c6422677695bc454fc645e0` |
| 13201689142 — Linux media | 85829 | `f67c37cc879da4664bdf5fbe1324dcae817b1f76f9b82889699d86aab089fb07` |
| 13202660395 — source authority | 4951791 | `44c1ec838fe9e576d30a19fbad395406bb26100f211ed2ad24aeb28543847c64` |

All downloaded archives match their original GitHub metadata digests. Full raw API responses, original job-log observations and extracted receipts remain adjacent to this report. Intermediate Linux logs are labeled observations until the terminal job record is acquired; the aggregate machine proof tracks that status explicitly.

## Limits and continuation

No product files, workflow or GitHub state were changed by this auditor. No builds/tests/lint/generators/schema checks or synthetic qualification probes were executed locally or on htch; only immutable-source and original-evidence data were inspected. There was no rerun, dispatch or new campaign. The parent's later `78aa967`, `fee4f179` and R3 source work remain separate subjects. All eight global flags, unit verification, independent acceptance and global closure remain false. Queued macOS, skipped App and other skipped jobs remain non-passing.

Aggregate machine proof: `113-evidence-audit.json`. The next action is parent adjudication and owner correction followed by the authorized exact-subject recovery evidence; the failed 113 receipt remains historical evidence and is never rewritten into a pass.


## Terminal Linux observation and secondary capture

Linux job 125709987601 completed **failure**. Steps 12 (portable) and 20 (link-preview) fail; actual App step 19 remains skipped. Final artifact 13202776557 is 482,770 bytes, SHA-256 `61b1efe5b495fde38565487fba655a72b44e641d3e1c3c5492421d489dacc3f7`; all 51 portable members are byte-identical to the before-App artifact. The complete decoded terminal Linux log is 4,490,018 bytes, SHA-256 `58d38e66554a9233bd61bd57a053c1abe922b0af3c0633aa0eff298a373b23dd`. Earlier original partial logs remain preserved.

Step 20 retains three successful own command selections: Host, Coordinator and three filtered link-preview App cases. Their raw hashes, exact required names, source commands and 31 unchanged source inputs independently match. Its fourth entry, `reused-mcp-composition`, has no command and null cargo/capture exits; it fails because `.evidence/sqlite-native-path/automation-admission/receipt.json` is absent after the producer step was skipped. This is secondary missing evidence, not another product test failure. The three filtered App cases do not replace the skipped 94-case main App selection or grant any DTR App credit. Proof: `113-secondary-link-preview-audit.json`. macOS remains queued; all global flags remain false.
