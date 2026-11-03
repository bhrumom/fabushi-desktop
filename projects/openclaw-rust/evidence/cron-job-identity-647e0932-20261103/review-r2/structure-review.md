# Cron job identity Round 2 — independent structure, reuse and performance review

Verdict: **SOURCE-ONLY APPROVE**. No falsifiable blocking defect found in the declared cron change or its composition onto the exact live integration base. This is source-review approval only; compilation, test execution, performance measurements and final acceptance are unknown in this review.

## Review identity and method

- Reviewer lens: structure / existing-owner reuse / performance integration; fresh and blind to other review reports and runner decisions.
- Actual launch metadata supplied by the parent: `gpt-6.1-sol`, effort `xhigh`. The repository pins this model for these lenses. The scope crosses canonical/legacy identity namespaces, durable execution/history owners and shared Actions composition; that warrants `xhigh`. No fallback was reported or applied in this review.
- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Reviewed immutable tree: `215733e43f8016e159ef1d676e433138272f6dc6`.
- Exact integration base: commit `71b039ae860c45e982dfbc8b1f401143d49d2454`, tree `3c30ec8ac78d0b8b549267a4b794a7cb177b8b23`.
- Original cron base: commit `972779ecd290b90e407ffe651aa5c456c31e9f7f`, tree `a75bb4706831a34b818e459f6be4df920560a2fe`.
- Prior snapshot retained as context: `f5e577cf9e119f417ce0cf071f03c214c5bf6190`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.

Read the root `AGENTS.md` and the complete candidate `projects/openclaw-rust/docs/specs/cron-job-identity.md` before judging. Read the Round 2 manifest, complete-scope patch and complete integration-delta patch. Reviewed the original Round 1 complete cron patch as context through the common complete-scope content and its complete textual comparison with Round 2; the comparison changes object identities, line offsets and incoming media context, without a cron algorithm/test/spec/source-contract change. No Round 1 reports, review decisions or integration-decision document were read. The mutable branch was not used as the review subject.

Used only source/Git reads and exact GitHub source reads. `GIT_NO_LAZY_FETCH=1` was set for Git object inspection. Two absent adjacent Git blobs were read through the exact GitHub blob API: App `Cargo.toml` `a2c58167a82657194b79aaf3e1e0d55577b2221c` and Session `production.rs` `c5912cab4f95d3a75fc13cb264a21b1deef9a733`. No local or htch build, test, lint, formatter, parser, generator, validator, benchmark or harness was executed. The only file written by this reviewer is this report.

## Complete declared scope

Git's exact-base name/status diff contains exactly the seven paths in the manifest:

1. `source/host/app/src/main.rs`.
2. `.github/workflows/rust-desktop-runtime.yml`.
3. `source/host/app/src/tests/cron_job_identity_tests.rs`.
4. `projects/openclaw-rust/docs/specs/cron-job-identity.md`.
5. `projects/openclaw-rust/evidence/cron-job-identity-647e0932-20261103/source-contract.json`.
6. `projects/openclaw-rust/parity-ledger.json`.
7. `projects/openclaw-rust/requirements-traceability-matrix.json`.

Adjacent source reads cover App registration and native `runAgentAutomationNow` dispatch, the existing stable ID mapper, Session automation/roster interfaces and shutdown, FileAutomationStore listing/creation/history, native AutomationRuntime creation/manual execution/disposal, AutomationRunPath terminal/retained handling, Gateway registry and authentication/operator/scope boundaries, App dependency/feature declarations, and the shared App Actions capture/execution step. Project context was bounded to the current Source of Truth entry, the relevant module-map owner rows, and OCRM-EXEC-02 §17. Fixed-U API reads reproduced all five declared identity slices with matching blob identities: jobs creation, exact job read, ID/access handoff, `cron.run`, and its CLI consumer.

## Structure and ownership assessment

`main.rs::list_openclaw_cron_jobs` (starting at line 2969) changes only the external job identity projection. It obtains the real Agent/local-routine pair from the existing Session/FileStore owners and calls `stable_automation_id(&agent.id, &r.id)`. It preserves the existing schedule, enabled/prompt/history and pagination projection. The mapper already exists in `source/host/src/automations/automation_id.rs` and is already imported/used by the App's telemetry; this batch introduces neither another mapper nor a durable alias registry.

`main.rs::trigger_openclaw_cron_job` (starting at line 3037) retains one selected `(agent_id, local_id, canonical_id, previous_run_id)` tuple. Its joint canonical/local match predicate counts a record once even when both namespaces match that same pair. A second matching pair refuses before the `FnOnce` run port. A unique local-only UUID-v5 input refuses through the existing UUID library, preserving the documented stale-canonical non-redirection rule. A misleading request `agentId` is not copied into execution arguments.

The run port receives the selected Agent and **local** automation ID. The registered App wrappers at lines 5417–5433 call these actual private functions, and the trigger closure still calls `self.call("runAgentAutomationNow", run_args)`. Native dispatch at line 14086 continues to select the configured routed Runner or existing Host lane and uses the same existing AutomationRuntime. The after-dispatch history read uses the selected local pair; the response carries its canonical job ID and preserves the existing observed-run projection. No new scheduling, cancellation, persistence, replay, completion or permission owner is introduced.

The new test module executes the production functions rather than a resolver copy. It creates actual Sessions and routines through the existing native creation dispatcher, uses the actual file-backed AutomationRuntime, controls only final execution results, checks real histories/observations and non-target bytes, reopens owners, and deletes through the existing runtime. Fixture roots are exclusively created; disposal/shutdown precedes removal of the fixture's own root. Source wiring assertions are expressly supplementary. Compile/run validity remains unknown until exact-candidate Actions execute.

## Performance assessment

Listing adds one existing stable-ID calculation per listed routine. Trigger resolution must now inspect all visible pairs rather than stop at the first local-name match, because ambiguity across Agents and canonical/legacy namespaces must be rejected. That is the required causal cost of CJI-02 with the current non-reversible stable ID and no alias state. The implementation keeps only one candidate tuple, adds no cache/invalidation machinery, polling, background worker, persistent write or additional scan before dispatch. Existing Session/FileStore listing and history costs remain owned by those existing interfaces. No source-evidenced avoidable new asymptotic cost or materially heavier dependency/build reach was found. This is a source assessment, not a latency or memory benchmark.

## Integration assessment

Relative to exact live tree `3c30ec8…`, the App patch contains only the cron helper extraction/identity behavior, the two wrappers and the cron test-module registration. The incoming `ProductionSendMessageSink` media implementation is therefore retained byte-for-byte outside those cron hunks. The larger Round 1→Round 2 media delta represents the incoming base, not an expansion of this cron review into independent media implementation acceptance.

The shared workflow retains the incoming media inputs, `media_settlement_contract`, SQLite environment capture, scope, and all eight exact media test names. It also retains `host_feature_observation`, the shipping Host dependency observation without `test-support`, and App-test dependency observation with `test-support`. The cron additions are adjacent distinct dictionary fields, three source inputs and five exact required cases. Both required-case lists append to the existing combined `required` list; neither overwrites the other. Exact test listing, one-passed/zero-failed/zero-ignored execution summaries, actual executable identity, command/log digests and H/tree/U/run/attempt fields remain present in the composed capture path.

The narrow ledger/RTM delta against the live base adds cron evidence references only: one existing dispatch-registry responsibility and the existing R04/AC11 rows. It changes no read, mapping, implementation, wiring, verification or acceptance status, gate list or global completion flag. Incoming media responsibility/RTM records are retained from the base. The cron source contract continues to state that source rows/global flags/unit verification are not promoted and exact-H execution is pending.

## Remaining execution uncertainty

No exact-candidate Actions result was inspected or asserted here. The shipping App check, linked test build, five required cron cases, retained native/Gateway/media regressions and executable feature observations must still pass on the published exact H. Full HTTP/App construction, real backend/provider/channel behavior, broader upstream cron access/queue/wait equivalence, concurrency correlation beyond the existing history projection, filesystem enumeration strength and supported-platform/package acceptance remain outside this bounded source verdict, consistently with the candidate spec.

Findings: **none**. **SOURCE-ONLY APPROVE** for the declared Round 2 scope and its reviewed integration boundaries.
