# MSU R6 background-wakes wiring-oracle repair proposal

## Disposition

**PROPOSAL / EXECUTION UNVERIFIED.** This worker prepared a complete replacement candidate for `source/host/tests/background_wakes_production_contract.rs`. It has not applied the candidate to the writer checkout, changed production code, changed a workflow or ledger, or run executable verification. Root remains the integrating writer and must obtain the required fresh focused independent review before the next frozen-candidate Actions campaign.

Candidate: `/workspace/scratch/7d27ea239237/msu-r6-background-candidate.rs`.

| Identity | Value |
| --- | --- |
| Candidate Git blob | `1c38f0bfde3e23cb0d227d1c4dfb4bd5b4d0b713` |
| Candidate SHA-256 | `281cfdaa4b4c28a89dab0474340775c43d17b0b3b01808c9842c26f4a1460529` |
| Copied writer-draft Git blob | `b5ec669ec76774b00d84cd0611e47122ff03f1e8` |
| Copied writer-draft SHA-256 | `db07f1651a0d8840a187280f7da72c13dee59e818be0f7eb2d915663e6a3fcc6` |

The writer draft already contained the parent-authored bounded literal replacement for the two stale assertions. This candidate replaces that draft block and inserts one supporting AST predicate immediately before the same existing shipping test. The inspected diff against the copied draft has those two regions only. All other source is carried forward unchanged.

## Authority and failure

Task: `647e0932-bea4-4113-9a61-65ae3db58955`, round 1. Parent-supplied authority is current commit `9059d0c4493505851339ab228ec0d58a6023c904`, tree `63385ed61d0ef7074dab86baea7fab548224416d`, with original test blob `2da45cf1dbca8a80ef341f4624ea877b1629d9f9`. Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`. This worker did not independently re-resolve the remote commit or fetch the original test blob.

The existing authorized decision is `MSU-ROOT-R6-002`, classification `TEST_FLAW`, decision `FIX`, in:

`/workspace/scratch/7d27ea239237/media-settlement-work/projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/r6-fix-decision-before-edit.json`.

The preserved Hfc evidence is run `41582614116`, job `125740667771`, artifact `13207615800`. Reading the existing integration log confirms the existing shipping test failed at line 2930 on `SHIPPING_HOST.contains("for cleanup_error in custody.rollback()")`; the recorded target result was **47 passed, 1 failed**. The adjacent old diagnostic literal is also obsolete according to the pre-edit decision. No log was rewritten and no prior result was reclassified as passing by this proposal.

Existing log read:

`/workspace/scratch/7d27ea239237/media-fc-ci-app/linux-native-terminal-13207615800/channel-pending-recovery/integration.log`.

## Source basis and actual responsibility

Read the repository-wide authority `AGENTS.md`, the complete task specification `media-settlement-r5/projects/openclaw-rust/docs/specs/media-settlement-unknown.md`, the complete relevant shipping test, nearby existing syn helper precedents, the pre-edit decision, the actual App ownership and send paths, and the local custody implementation. The following immutable source identities were independently hashed:

| Source | Git blob | SHA-256 |
| --- | --- | --- |
| `media-settlement-r5/source/host/app/src/main.rs` | `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df` | `0ca4e2e81b459ac57c1a9aab9b31b54ef2d421d7131970519aa4f58faaa0011f` |
| `media-settlement-r5/source/host/src/attachment_paths.rs` | `6e1a2b8eb5752e005bbce30ed0b6eaaacfda0cda` | `ce4ac8bc522d1b20ff97b486942dfbe2aacd1ff0f65ee70a0b657f6960fff381` |

The App read covered its file header and the relevant owner region around lines 705–1285, including both actual methods and adjacent admission, locking, resolution, append, error and publication consumers. The custody read covered lines 1–112, including disposition, rollback, confirmed-refusal rollback and Drop.

At this source, ordinary abort calls `pending_media_entry(tool_call_id, false)`, takes the returned entry through `Self::try_pending_media(&entry)`, and delegates `rollback_pending_media(&mut pending, tool_call_id, false)`. Its body does not remove the map entry. The inherent helper obtains cleanup errors from a three-arm phase match: Collecting uses ordinary rollback; OwnerSettlementPending requires `confirmed_precommit_refusal` before using `rollback_after_confirmed_refusal`; all other phases return. It reports every returned error and returns before fence release if cleanup failed. The custody owner separately limits deletion to rollback-eligible disposition and permits the explicit confirmed-refusal transition. Those are source observations; they do not establish executed transaction or concurrency behavior.

## Proposed oracle

`shipping_abort_reports_guarded_media_cleanup(&syn::File)` is a deliberately bounded structural predicate. It uses the already declared `syn = 2.0.119` dependency with `full` and `parsing`; there is no new dependency and no use of `quote`, `extra-traits`, AST cloning, or a new source-parser framework. It reuses the existing unconditional-attribute helper for the file envelope.

The predicate requires:

- One unconditional top-level `SendMessageSink` implementation for the exact `ProductionSendMessageSink` type and its one `abort_attachment_custody` method; one unconditional inherent `rollback_pending_media` method for that type. Unrelated trait implementations are not inherent owners. Qualified, generic, conditional or ambiguous forms are conservatively rejected at this bounded seam.
- The direct three-statement abort sequence with the original argument bindings: existing-entry lookup with `false`, same-entry nonblocking helper call, and ordinary cleanup delegation with `false`. A dead branch, nested fixture, extra removal/recreation statement or changed receiver cannot satisfy that sequence.
- The direct guarded cleanup result, the terminal local field updates, `errors.is_empty()`, and iteration over that same `errors` value. The exact `eprintln!` format includes the loop-bound returned `error`, `self.agent_id`, and `tool_call_id`; a marker string elsewhere does not satisfy it.
- The failed-cleanup return before the final fence-release branch. The release must use the same pending entry's revision and work ID, clear its work ID only on accepted release, and retain the existing release-error diagnostic. The complete bounded statement shape disallows an extra earlier release hidden before the checked tail.

The real source is parsed once per positive or mutant. LF and CRLF forms must both be accepted for the actual-source positive. Mutation construction requires one occurrence of its intended source anchor, unequal before/after text and a changed source result. Every mutant must still parse before its refusal can count.

All controls are inside `shipping_send_message_reaches_host_owned_delivery_failure_broadcast_and_context_paths`. No `#[test]` registration was added or removed, so the target retains its **expected 48 named tests**. This is preservation of the source registrations, not a new executed count. The unrelated channel assertions, helpers and cases remain unchanged.

## Authored mutation controls

Each of these sixteen controls is exercised in both LF and CRLF form by the proposed test:

1. Keep the correct delegation text inside `if false`, skipping actual cleanup.
2. Change ordinary abort's confirmed-refusal argument from `false` to `true`.
3. Change ordinary abort's entry-creation argument from `false` to `true`.
4. Replace the same-entry nonblocking helper with a blocking `entry.lock()`.
5. Remove the confirmed-precommit guard from the pending-phase arm.
6. Replace the unknown/terminal return arm with cleanup.
7. Discard each returned cleanup error instead of logging it.
8. Keep the diagnostic marker but omit the returned error from the message.
9. Release the durable fence before cleanup.
10. Let cleanup failure fall through to the release branch.
11. Remove and recreate the indexed entry while retaining the old entry.
12. Exclude the shipping file conditionally.
13. Exclude the shipping trait implementation conditionally.
14. Exclude the inherent cleanup helper conditionally.
15. Add an ambiguous second shipping trait implementation.
16. Add an ambiguous second inherent cleanup helper.

These are authored expectations only. None was executed locally. The two ambiguity mutants deliberately remain syntactically valid while representing compiler-invalid duplicate declarations; their claim is source-owner uniqueness, not a runnable alternative application.

## Limits, review and required execution

This is a wiring supplement. It does not expand macros, resolve arbitrary aliases or prove Rust typing. It is intentionally conservative about exact method shape and attributes. A legitimate future restructuring of these two methods requires a reviewed oracle update rather than being silently accepted through unrelated text.

It does not assert full behavior of `pending_media_entry`, `try_pending_media`, every helper caller, filesystem cleanup, DB rollback/COMMIT, owner retirement, reentrant callbacks or recovery. Those responsibilities remain with the real App/Session/DB/custody tests required by MSU-01 through MSU-10. This repair neither substitutes for those cases nor promotes any responsibility or acceptance ledger.

Only source reads/searches, copying/editing the proposal, diff inspection and file hashes were performed. No cargo invocation, compilation, test, executable parser, validator, generator, lint, capture, test listing, or htch-runtime check was run. The authored code's future `syn::parse_file` calls were not executed by this worker.

Root should review and integrate the exact candidate, then obtain the fresh independent focused review of this delta plus the affected real owner/caller paths. The required actual-source positives, mutation refusals, compiler compatibility and unchanged named-test result remain unverified until the newly frozen candidate runs in the permitted Actions campaign. Prior passing evidence carries forward only under the existing causal invalidation and campaign-budget decision.

Implementation phase requested effort was **xhigh** for owner/phase semantics and source-guard controls. The configured model and effective session settings were inherited; this worker made no effort/model override and does not claim that prompt wording changed an unexposed runtime setting. This is the implementer's handoff, not an independent review or acceptance verdict.
