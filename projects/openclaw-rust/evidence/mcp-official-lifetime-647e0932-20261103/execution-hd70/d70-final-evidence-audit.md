# Exact d70 MCP / PRID original Actions audit

## Outcome and authority

**The selected renderer MCP evidence passes. The shared Linux job fails with two portable test failures and a separate App test-binary compilation failure. Main App, Automation and PRID capture did not execute.** The source workflow completed successfully, while the runtime workflow remains queued with its macOS job queued. This is a completed bounded audit of the available original evidence, not global acceptance.

The subject is **`d70f86f33b5e3c67545954b270373e1548809f4b`**, tree **`4c4c9cfd3f5067fef57dca530042d149a8e2fc5c`**, parent **`972779ecd290b90e407ffe651aa5c456c31e9f7f`**. Original runtime is **41578381556**, attempt **1**; source workflow is **41578381536**, attempt **1**. All seven scoped blobs match the immutable publication tree and original commit API. The recorded configured model/effort is the parent assignment **gpt-6-astra/xhigh**, not a claim of model introspection.

| Original scope | Outcome | Direct evidence and limits |
|---|---|---|
| Renderer plugin-edge, job 125727154650 | **Pass** | Shipping bundle and Node TAP: 148 tests / 148 pass / zero fail, skipped, cancelled or todo; 144 top-level plus four nested |
| Linux portable, job 125727154700 step 12 | **Failure** | 52 commands/groups; 50 pass, two fail; 373 exact source input hashes |
| Main App step 19 | **Skipped** | 100 outer cases / 203 commands / 255 inputs are source selections only |
| Automation step 18 | **Skipped** | No producer receipt or borrowed MCP composition credit |
| PRID step 20 | **Skipped** | No new current-consumer tar or current-H database capture to inspect |
| Always-selected link-preview step 21 | **Failure** | Host scoped positives, Coordinator scoped positives with one interleaved ancillary line, App compilation failure; reused MCP receipt absent |
| Source structural workflow | **Success** | 26 immutable repository inputs plus one archived generated inventory; 62,130 tracked tuples; G02 final not run |
| Windows media / restoration jobs | **API success** | Original status recorded; their raw artifacts were not independently re-audited in this phase |
| macOS | **Queued** | No runtime or physical-device credit |

## Renderer MCP evidence

The original shipping build was `npm --prefix desktop run build:electron -- --plugin-skills-contracts`. The subsequent `node --test --test-reporter=tap` command ran five bundled targets under `set -euo pipefail`, piping output into the original TAP artifact. The original Actions build/test steps succeeded. The receipt has no separate numeric exit fields, so zero pipeline status is inferred from the successful pipefail shell step, not invented as a captured number.

The source-derived five target counts are **24 Host-loader + 19 SDK-loader + 2 Coordinator-control + 78 MCP-runtime + 21 Main-edge = 144 top-level cases**. Four explicitly nested Host-loader cases make the TAP total **148**. All **15 new MO cases appear exactly once**, all **63 prior MCP cases remain present**, and all **11 incoming materialization controls** appear exactly once. Exact names, indices, parent/child distinctions and complete TAP-to-full-log binding are recorded in [d70-plugin-audit.json](d70-plugin-audit.json).

All 20 paths declared by the plugin receipt match immutable H. All seven publication-scope blobs are separately API/Git bound. Only the MCP OAuth source and MCP contract test from those seven appear in the plugin receipt's 20-path map; the receipt is not a complete compiler dependency graph. The selected shipping bundle passes; broad renderer architecture/UI/build gates were skipped. Controlled-port evidence does not establish real provider/account behavior or global desktop acceptance.

## Linux failures and retained positives

The original before-App artifact and terminal artifact contain identical bytes for **all 53 portable receipt/log members**. The ordered 52-group source plan, raw logs, exact required-name occurrences and all 373 source hashes agree. The DTR groups retain exact positive selections of **44 owner / 14 HTTP / 27 stream regressions**. The three DTR App cases did not run.

**TEST_FLAW — service status oracle.** `plugin-uninstall-service` returns cargo 101/capture 0 because `desktop_refresh_and_sync_wait_for_the_actual_loader_and_status_reads_its_cache` expects only `authBlocked`. The actual status also includes `authRecovery`, required by immutable specification AR-08 for the retained refusal. The failed target reports **15 passed / 1 failed**. Preserve the exact-object assertion and add the required recovery projection; retain all loader/cache controls. Later assertions in that failed test were not reached.

**IMPL_BUG — poisoned-admission successor lifecycle.** `plugin-uninstall-state` returns cargo 101/capture 0 because `poisoned_admission_fails_closed_for_retained_and_successor_readers_without_resetting_truth` cannot construct the required successor. `retire_for_replacement` consumes the predecessor's one-shot claim, then its new `fence_owned_refresh` call propagates admission Mutex poison. The target reports **91 passed / 1 failed / 624 filtered**. Existing readers remain denied; the defect is lifecycle handoff against PS-LIVE-07/09 and retained shared-Arc requirements. The narrow repair boundary handles only already-terminal admission poison while preserving the poisoned Arc, no I/O/reset, healthy exact-owner fencing and every original successor assertion. It must not blanket-swallow future errors or change expected success to an error. See [d70-portable-failure-diagnosis.md](d70-portable-failure-diagnosis.md) and its machine proof for exact source blobs and excerpts.

**TEST_FLAW — incoming App fixture compilation.** The link-preview App command returns cargo 101/capture 0 before executing tests. Original diagnostics contain **12 missing `json!` macro errors, six missing `Value` type errors and one `f64` timestamp mismatch** in `source/host/app/src/tests/local_media_file_url_tests.rs`. Immutable source blob is `5682e951f1eac8f107efd3a3948b0748870912bd`, SHA-256 `f2097d754193bd21cc1b7911829f37ba395f59750680e1e560d682e6a8f49d80`. The exact source remedy is explicit JSON imports and `1_000.0` for the existing fixture timestamp; all behavior assertions remain necessary. This original report grants no pass to a later repair. The failing module's identity is derived from exact H, not falsely presented as a member of the 31-path link-preview receipt map.

The link-preview Host command separately has **52 complete positive names**, target summaries **9 + 43**, and all ten required names once. The Coordinator command has **cargo/capture 0**, summaries **58 + 14 + 1**, and all four required scoped names once. Its raw output has only **72 complete standalone success lines**: stdout/stderr interleaves `gateway_reachability_matches_http_network_and_base_url_semantics` into a numeric diagnostic. This limits full-target name pairing; it does not erase the four directly paired scoped positives or create a new product failure. The three App names have **zero executions**. The final reused-MCP record has no command and reports the absent Automation receipt, a secondary consequence of step 18 being skipped. [d70-terminal-secondary-audit.json](d70-terminal-secondary-audit.json) preserves every command, log hash, diagnostic and count distinction.

## PRID result and historical correction

At d70, the new `consumer_payload` and `current-consumer.tar.gz` handling is source-authored only. The preparation derives its exact member/inventory checks and immutable inputs, but step 20 was skipped. No archive safety, hidden-file inclusion or current-H consumer result is claimed from a non-existent d70 artifact.

The separately requested re-examination of **original 0c artifact 13204320664** corrects a prior audit overstatement. Both supposedly absent current-consumer database byte sequences are fully present in the original uploaded `persistent-image-after.json` files as `other_files["agents/.call-sessions.db"]` integer arrays. Each reconstructs to **32,768 bytes**, SHA-256 **`678a56ce2376b5996556e3f692199423d285feaa36a55aa04076b25370e63974`**, matching its original inventory. The container hashes match the original 89-file consumer inventory, which matches the PRID receipt and exact 0c/run/Host/App source context. **87 direct members plus two original JSON reconstructions provide complete inventory byte coverage.** Standalone hidden DB paths and dedicated consumer-tar packaging were still missing.

The earlier broad raw-absence/unrecoverability claim is withdrawn as **AUDITOR_ERROR**. Original reports remain unchanged and are hash-recorded. This reconstruction uses neither historical producer tar nor regenerated fixtures. [0c7-original-json-carrier-correction.md](0c7-original-json-carrier-correction.md) and its machine proof append the corrected conclusion; they grant no d70 or later-H runtime credit.

## Original artifact custody

| Artifact | ID | Bytes | SHA-256 |
|---|---:|---:|---|
| Before-App portable | 13204704124 | 464,788 | `37a64bad37cedad67277ceb7a71ffe40b47aa30bc38a778c708fddd7e635d005` |
| Terminal native | 13205178129 | 493,551 | `85792b9303ce1be988369dcebcf4620f89f5afcd34c4b50c3f410d53a02ecb31` |
| Renderer plugin edge | 13205176091 | 10,450 | `9c4e019634217058bf7d17d2a3edb7b00d68d286f426f76a1b6178af3ed09e58` |
| Source structural | 13204927153 | 4,951,794 | `abc732278612587a7915db4eb05e6a3d93a13e08f393f8bccdd532c976dcf0f1` |

The complete terminal Linux log is **4,805,823 bytes**, SHA-256 `c60c7a14aa2593582b38e77960e908c4636a520e603b68b0f13a933fcb823ef2`; renderer log **202,715 bytes**, SHA-256 `ad8c1e7ee22f185599a6edaa2e230b825948bf5c20ec875f099eeefdd01d1ae6`; source-authority log **31,525 bytes**, SHA-256 `aeae7f2b6a97f5153da4b0a5e6f2b7bc48bd872a91146dc5b7d5ca075dfc394b`. The [aggregate machine proof](d70-final-evidence-audit.json) binds original API observations, commands, all seven scope identities, reports and artifact/log digests.

All eight global flags, `unit_verified`, independent acceptance and global acceptance remain **false**. Prior e19/113/0c failures and positives remain separate. No local/htch product validation, repository edits, GitHub mutation, dispatch, rerun, or later-H evidence acquisition occurred in this audit. Subsequent corrected source requires its own original evidence; no further run is requested or started here.
