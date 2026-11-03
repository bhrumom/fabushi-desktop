# Hfc terminal Linux evidence and App execution boundary

## Result and launch implication

**Linux job 125740667771 completed with failure. The actual Automation/Settings/channel App step was skipped. Two concrete test defects require correction before a new candidate can claim those gates. Changing the COMMIT oracle alone does not address either failure.**

The portable command failed on a stale source assertion in source/host/tests/background_wakes_production_contract.rs. The later link-preview App test build failed to compile because source/host/app/src/tests/media_settlement_unknown_tests.rs references a nonexistent field. The root owns correction decisions and source edits. This worker made no repository edits.

This report applies only to Hfc, commit **fc187ecba404e2fa95d65871301aa8d83c7f0314**, tree **410414bcf3ad042cf49aeed86ce55836f4fbc1cd**. It does not promote evidence to R5, incoming f41, or later source. The earlier source, Linux media and Windows media evidence remains preserved at its original identity.

## One bounded observation

The single new API observation was made at **2026-11-03 05:32:17 UTC**. No subsequent run/job/artifact polling was performed.

| Identity or state | Observed value |
| --- | --- |
| Repository | bhrumom/fabushi-desktop |
| Run / attempt | 41582614116 / 1 |
| Tested commit | fc187ecba404e2fa95d65871301aa8d83c7f0314 |
| Tested tree | 410414bcf3ad042cf49aeed86ce55836f4fbc1cd |
| Fixed upstream commit | 1440bb1771372334bea23d7642bb52deb7db1845 |
| Fixed upstream tree | 90c8aab39972da11cfad220361781d49e1bc6700 |
| Linux job | 125740667771, sqlite-native-path |
| Linux job start | 2026-11-03T05:13:14Z |
| Linux job completion | 2026-11-03T05:30:26Z |
| Linux job conclusion | failure |
| Aggregate run state | queued; conclusion null |
| macOS job 125740668001 | queued |
| Windows job 125740667992 | completed, success |

The preserved [exact-attempt jobs response](sandbox:/workspace/scratch/7d27ea239237/media-fc-ci-app/api/jobs-attempt-1.json) records:

| Step | Result |
| --- | --- |
| 8 — Execute media cleanup owner and real service contracts | success |
| 12 — Portable recovery, Memory, pairing, palette, Session, MCP and channel controls | failure; completed 05:27:58Z |
| 18 — Typed Automation admission and actual owners | skipped |
| 19 — Actual Automation, Settings and channel App owners | skipped at 05:28:00Z |
| 20 — Historical prior-root recovery through current Host and shipping App | skipped |
| 21 — Native link-preview HTTP, Coordinator and app dispatch | failure; completed 05:30:22Z |
| 23 — Upload native SQLite path evidence | success |

The exact workflow makes link-preview eligible through its explicit always() condition after successful native paths. Thus it executed after the portable failure, while the ordinary Automation/App steps were skipped. The exact Hfc workflow and terminal job sequence support this distinction.

## Terminal archive and complete logs

New artifact **13207615800** is named ocrm-sqlite-native-path-fc187ecba404e2fa95d65871301aa8d83c7f0314-41582614116-1 and was created at 2026-11-03T05:30:23Z. Its API record names run 41582614116 and Hfc.

| Item | Bytes | SHA-256 |
| --- | ---: | --- |
| Downloaded ZIP | 493,564 | 67e6c96370531240a7d079a500b77860857f1bd144a6e62ab8d74d0ffa79b3c5 |
| Complete decoded job log | 4,989,438 | bc2980e3a6f52dea60fe20f057ad3309d574e643e6c583b1c6ec9055266e7493 |
| Portable receipt | 379,370 | a4674fa34aef061e605ff6dda391b62923543c7bacdd56b4810c6de7aa5f3c14 |
| Failed portable integration log | 50,298 | b4c853603bc9676993ebca8c57e143bed45d1cc3683371cd7aba7e583156a8b7 |
| Link-preview receipt | 26,167 | 10acd50d74c9f0e1a162f74045e0fb45a09209fc2927e7f3808e90975ca67be2 |
| Failed link-preview App log | 50,299 | c265d5f46491b731b3335823853bcce900fd23b655f0b07bf03ac0b0701eedbc |

The ZIP size and SHA-256 exactly match API metadata. All 69 archive members were saved as bytes. The archive contains 60 log files. They have no bare CR; the Coordinator log contains nine CRLF, while both failure logs use LF exclusively. All receipt JSON members use LF. No newline normalization was applied to archive members.

The complete job log is the connector's decoded UTF-8 representation, saved without text-mode newline conversion. It contains 71,007 LF and no CRLF or bare CR. This is distinguished from the archive's original member bytes.

All **52 portable raw-log hashes** and all **three available link-preview raw-log hashes** match their receipts. The missing reused Automation log is recorded as absent. Complete command arrays, selected/observed names, exits, errors and per-log bindings are preserved in [terminal-evidence-assessment.json](sandbox:/workspace/scratch/7d27ea239237/media-fc-ci-app/terminal-evidence-assessment.json), with original receipts alongside raw logs.

## Portable failure: stale source assertion

### Actual command and outcome

The enclosing Actions step ran:

    python3 projects/openclaw-rust/scripts/capture_portable_contracts.py

Its integration case ran:

    cargo test --locked --manifest-path source/host/Cargo.toml --no-fail-fast --test durable_recovery_stores_contract --test pending_wake_rearm_contract --test transcript_manager_contract --test memory_gateway_authorization_contract --test channel_gateway_authorization_contract --test cloud_agent_pending_wake_rearm_production_contract --test connector_secret_store_contract --test session_gateway_contract --test widget_responses_full_contract --test agent_content_search_contract --test roster_search_contract --test palette_message_gateway_contract --test memory_service_contract --test turn_memory_contract --test memory_synthesis_store_contract --test memory_synthesis_service_contract --test background_wakes_production_contract -- --test-threads=1

The integration command returned **cargo_exit 101**, **capture_exit 0**, with no execution error. The enclosing Actions step exited 1. Its receipt rejected the execution even though missing_or_nonunique_positive_results was empty: the command's failure remained decisive.

The exact failed test is:

    shipping_send_message_reaches_host_owned_delivery_failure_broadcast_and_context_paths

[Raw integration.log](sandbox:/workspace/scratch/7d27ea239237/media-fc-ci-app/linux-native-terminal-13207615800/channel-pending-recovery/integration.log) records the panic at tests/background_wakes_production_contract.rs:2930:

    assertion failed: SHIPPING_HOST.contains("for cleanup_error in custody.rollback()")

That background-wakes target finished with **47 passed, 1 failed, 0 ignored**, in 12.90 seconds. The 47 positive results are retained. Of the portable receipt's 52 command executions, 51 are recorded as accepted and this integration execution is rejected. This report does not turn the rejected aggregate into approval.

### Exact-source causal classification: TEST_FLAW

The failing fixture is Hfc Git blob **2da45cf1dbca8a80ef341f4624ea877b1629d9f9**, SHA-256 **3cf7721a1c8545c360ebc831021cdfe63b5aafe98aeb33b467b1cecd9ca45720**. Its bytes were acquired through that exact GitHub blob and match the portable receipt.

Production main.rs is Hfc Git blob **ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df**, SHA-256 **0ca4e2e81b459ac57c1a9aab9b31b54ef2d421d7131970519aa4f58faaa0011f**. Its bytes match the exact tree and both portable and link-preview receipt maps.

Direct source reading establishes the actual chain:

1. ProductionSendMessageSink::abort_attachment_custody at main.rs:1087 obtains the existing entry through pending_media_entry(tool_call_id, false).
2. It obtains that entry's guard through Self::try_pending_media(&entry).
3. It calls self.rollback_pending_media(&mut pending, tool_call_id, false).
4. rollback_pending_media at main.rs:856 obtains errors from pending.custody.rollback() in the Collecting phase. OwnerSettlementPending can use rollback_after_confirmed_refusal() only when confirmed_precommit_refusal is true. Every other phase returns before cleanup.
5. The helper iterates those errors and publishes each through eprintln with the actual agent and tool-call identity.
6. Failed cleanup returns before release_session_media_admission, retaining the durable fence.

The actual error format is:

    mahayana-host outgoing_media_cleanup_failed agent={} tool_call={} error={error}

The adjacent fixture assertion at line 2933 still expects:

    outgoing_media_cleanup_failed agent={} tool={} error={cleanup_error}

Simply repairing the first literal would therefore expose the second stale assertion. Both expectations describe the old local loop shape; the cleanup and error-publication responsibilities remain in the current owner.

The retained-unknown boundary matters. pending_media_entry returns a cloned Arc without removing or replacing the map entry. Its map lock ends inside that helper. try_pending_media uses try_lock; WouldBlock returns ToolOutcomeUnknown. Poison recovery seals collecting custody as OwnerSettlementPending and returns ToolOutcomeUnknown, without continuing cleanup or execution. Ordinary abort supplies false, so pending/unknown, committed and previously aborted identities are retained. Underlying OutgoingMediaCustody only removes paths while RollbackEligible; ordinary rollback and Drop do not recover deletion authority from pending/committed disposition.

The Tool caller also suppresses abort on ToolOutcomeUnknown when message construction fails. These are source facts, not newly executed App/runtime proof.

### Narrow replacement recommendation

Keep the existing shipping_send_message… test and every unrelated assertion. Replace only the two obsolete cleanup-loop/log-string assertions with a bounded proof of the actual owner chain:

- Resolve exactly one unconditional abort_attachment_custody method in impl SendMessageSink for ProductionSendMessageSink, and exactly one applicable inherent rollback_pending_media method.
- Require existing-entry lookup with create=false, same-entry try-lock admission, and delegation to rollback_pending_media with confirmed_precommit_refusal=false, in order.
- Require the Collecting custody rollback, guarded confirmed-refusal rollback, and default return preserving unconfirmed/pending and terminal identities.
- Require that the returned cleanup-error collection is actually iterated and published with agent_id and tool_call_id. Reject an unused/dead logger or unconditional success substitute.
- Require cleanup failure to return before Session media-admission release.
- Preserve Arc cloning without remove/recreate; WouldBlock refusal; and poison sealing followed by unknown refusal.

The file already uses syn and bounded source helpers. Extend that approach to the two identified impl owners rather than restoring a whole-file literal that unrelated or dead code could satisfy. Existing channel_failure_shipping_function handles free functions only; it cannot identify impl methods unchanged.

Qualify the replacement against actual positive source in LF and CRLF, with each negative first confirming one real mutation site. Targeted negatives should remove delegation, change ordinary abort's false to true, bypass the pending-phase guard, remove error publication, move release before the cleanup-failure return, or remove/recreate the retained entry. Preserve the 47 other target cases and all existing channel, delivery, inbound and context assertions.

This recommendation is source-only and unapplied. The root retains durable FIX, implementation, review and Actions execution responsibility.

## Link-preview failure: App test fixture does not compile

The exact App command was:

    cargo test --locked --target-dir source/host/target --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host tests::link_preview_gateway_tests::

It returned **cargo_exit 101**, **capture_exit 0**, with no execution error. No selected App test ran.

[Raw app.log](sandbox:/workspace/scratch/7d27ea239237/media-fc-ci-app/linux-native-terminal-13207615800/link-preview-native-gateway/app.log) reports E0609, no field duplicate on type PersistedSendContext, at src/tests/media_settlement_unknown_tests.rs:805:68. The offending source is:

    assert!(accepted.context.accepted_durably && !accepted.context.duplicate);

The compiled MSU fixture is Hfc blob **f525710de5833559de608e1cb8aaa89a8d58e2b8**, SHA-256 **f1ec56ebb6b7b31bcb9809645a60221e488269bbb841780d65da01d35da5fc68**. Those bytes match the exact Git tree and the portable receipt, which includes this compilation dependency. The link-preview receipt's narrower input list does not itself list this MSU fixture.

This is a **TEST_FLAW in the App test fixture**, directly evidenced by compilation of an access to a nonexistent field. It is not evidence of infrastructure failure or a production runtime failure. The root independently identified the same defect and owns inspection of the context/caller contract and correction. This report does not prescribe a replacement field without that ownership review.

Link-preview Host and Coordinator returned cargo_exit 0 and capture_exit 0 and are recorded as accepted. Its reused-mcp-composition evidence could not open .evidence/sqlite-native-path/automation-admission/receipt.json because the producer was skipped. That is a missing dependency caused by the earlier gate failure, not an independently established environmental root cause or permanent blocker.

## Actual Automation/App evidence remains unexecuted

The automation_app step's explicit outcome is **skipped**, not pending or passed. No automation-app receipt or logs are in the terminal artifact.

| Required observation | Hfc evidence from this run |
| --- | --- |
| Ordinary shipping-feature-off App compilation in automation_app | Not executed |
| Compiled feature observations in automation_app | Not executed |
| MSU eight private App/Session/Runner cases | Not executed |
| LFA six App cases | Not executed |
| SGF three App cases | Not executed |
| Cron five selected App cases | Not executed |
| Link-preview three App cases | Test build attempted; compilation failed; no tests executed |

The separate link-preview command had no explicit --features argument, but it was an App test build that failed. It does not supply the skipped ordinary compilation/probe receipt or the named Automation/App runtime results.

## Input binding, retained evidence and limits

[causal-input-bindings.json](sandbox:/workspace/scratch/7d27ea239237/media-fc-ci-app/causal-input-bindings.json) records 11 exact source inputs covering the failed portable fixture, production cleanup chain, App compilation fixture, capture script, Cargo declarations and workflow. Every saved input's Git blob matches the preserved untruncated Hfc GitHub tree, and every corresponding receipt SHA-256 matches.

The portable receipt contains 377 input hashes. This bounded diagnosis did not independently acquire all 377 source bytes. The link-preview receipt has 31 before and 31 after hashes; those maps are identical. Only the corresponding causal subset above is independently byte-bound here. No full-receipt input approval beyond that subset is implied. There is no automation_app receipt whose inputs could be bound.

The prior source/Linux report, assessment and binding/index, plus the Windows report/assessment/binding, retain their previously recorded hashes. Those seven preservation hashes are recorded in the new terminal assessment. No previous artifact, report, snapshot or diagnosis was rewritten.

The complete archive, raw members, decoded job log, source copies, API snapshot and assessment are under media-fc-ci-app. The phase retained the configured parent model and inherited xhigh effort because exact-subject artifact judgment and source-causal classification were required. No local or htch source parser, compiler, test, capture, validator, generator or test listing ran. Work was limited to API/Git acquisition, direct source/log reading, JSON/ZIP decoding, byte copies and hashes. No dispatch, rerun, cancellation or repository mutation occurred.
