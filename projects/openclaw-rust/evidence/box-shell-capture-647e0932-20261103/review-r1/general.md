# BXR-SHELL-CAPTURE-v1 — fresh GENERAL review, round 1

Verdict: CHANGES_REQUESTED. One MUST_FIX finding (P2); no additional NON_BLOCKING findings. This is a source-only review, not execution or acceptance evidence.

Repository: bhrumom/fabushi-desktop
Repository identity independently read through GitHub API: repository ID 1375312893; owner bhrumom ID 281146136; public; active connection reports pull/push/admin permission.
Original immutable base: ca32f83f13a8aed439e9e3341d175405b1d1a655
Reviewed source commit: 152467880098cd87581ca279be4d2bc872a7620b
Reviewed source tree: 8fe5821505d342518628d56aa2920cc7b895284c
Scope manifest blob: 53fe2ada1c70a3b847a6fa7f435f1e8e81b9bd5d
Complete declared diff blob: deed9b45d7e6c168d93692a0da44a5b70dbadfc6
Contract: BXR-SHELL-CAPTURE-v1, SC01–SC07.
Requested review configuration: gpt-6-astra / xhigh, GENERAL including security, lifetime, concurrency, compatibility and oracle validity. This records the requested launch configuration; the reviewer has no separate hidden runtime/model attestation.
Review method: GitHub API source reads only. No local, htch, shell, build, test, linter, generator, schema or acceptance execution; no CI dispatch. No source, tree, commit, ref or PR mutation. No prior independent review report was accessed. The declared source-contract itself contains a pre-freeze decision entry; that scoped source was read, without following its historical report references.

## MUST_FIX

### GSC-R1-01 — P2 — PID reuse can overwrite a capture that still owns unjoined workers

Primary anchor: source/box-exec-daemon/src/server.rs:1904, `processes.insert(pid, ForegroundProcess::Capture(Arc::clone(&owner)))`.
Related anchors: server.rs:667–672 (try_wait records and reaps the child), 1941–1972 (reaped child remains pending while capture drains), 1842–1855 (removal waits for status plus capture_complete), 1795–1837 (stop snapshots and decides completion exclusively through the registry).
Current blob: c586ea57bec0bd0513a5f21f4532b98c08fbaf7b.
Requirement: SC02, retaining the owned request/capture registration through observed worker cleanup and refusing a clean stop while cleanup remains pending.

The registry now deliberately outlives the direct child's reap, but its sole key remains the recyclable OS PID. The insertion is unconditional and discards the previous entry for the same number. Arc identity is checked only during removal; that prevents an older capture from removing a newer entry, but it does not prevent the newer insertion from removing the older capture's registry custody.

Concrete triggering sequence:

1. Request A starts a foreground child with PID P. That child exits and is reaped by CapturedChild::poll, but a separately sessioned descendant retains one of its output pipes. A's capture workers therefore remain pending. A request with timeout 0 is permitted to wait indefinitely under the normal-output contract. The separately sessioned pipe holder allows the original process-group identity to disappear, so P can become reusable; no claim of descendant cleanup is needed.
2. Once the OS recycles P, another foreground request B spawns with that PID while A is still draining. run_process inserts B at P and silently replaces A in the only registry. A still exists in its request thread, but the shutdown owner can no longer enumerate it.
3. B completes and removes its own entry. stop() sets stopping and sees no Capture entries, so it returns Ok immediately. A has not yet observed cancellation and joined both workers; its normal cancellation/drain path can still require the 250 ms grace plus worker observation. The entrypoint can consequently treat shutdown as clean while the older request still owns capture work.

This is a newly material lifetime gap in the post-reap retention design. It does not depend on signalling an already reaped PID, and the existing signal() stale-PID guard does not address it. The same shared registry also admits Stream entries by numeric PID at lines 1125–1128, with unconditional numeric removals such as 1334–1337, so any repair must preserve identity across those adjacent users as well.

Required fix: retain a distinct, non-reusable registration identity for every foreground operation inside the existing registry, carrying the OS PID as process data rather than using it as the sole lifetime identity. Alternatively, preserve all distinct registrations for a reused PID and require identity-qualified removal. Do not add a second process registry. A newly spawned process must never replace an older registration that still owns workers. Review all Stream insertion/removal users of the same map when applying the fix.

Required focused evidence: an ownership regression controlling a reused numeric PID while the old registration is reaped but capture-incomplete, with two distinct operation owners. Require both registrations to remain visible, require an older completion not to erase the newer owner, and require stop to refuse clean completion until all relevant capture workers have been observed. The test may use a narrow registration seam to control numeric reuse rather than relying on the OS to wrap PIDs nondeterministically. Execute only in the authorized Actions lane.

## NON_BLOCKING

None. Deferred raw-byte caps, descendant-process closure, hostile process/filesystem changes, real service/device/platform/package behavior, generated HTML provenance and full OCRM acceptance are retained limitations, not newly invented blockers.

## Source findings that support the rest of the change

The pre-spawn/post-spawn distinction is carried consistently through ProcessFailure, shell_with_abort and both ordinary ShellArgs and MiniSweAgentBashArgs dispatches. After successful spawn, pipe acquisition, nonblocking setup, worker creation, read failure, wait failure and join failure all reach the typed unknown path. Interrupted reads retry. No observed path substitutes partial or empty bytes as a definitive result after those errors.

The ordinary production capture path contains no fault selector. CaptureProbe, its runtime field, observation helpers, and the ignored child test are cfg(test). Fault controls run after actual prefix observation from a real spawned child, and the shell script appends its counted effect before printing that prefix. Both stdout and stderr and both foreground dispatch variants are covered in the new unit matrix.

The cleanup implementation retains the Child in an Arc-protected owner. Child polling and signalling share the mutex; once status is observed, signal returns without using that PID again. The error boundary sets capture cancellation, attempts bounded child cleanup, and calls observe(true) for both workers without short-circuiting after the first result. Pending terminal ownership is retained when cleanup cannot establish reap. These mechanisms are sound by source inspection apart from the registry identity finding above.

Successful and nonzero exit output still uses the existing lossy UTF-8 projection. Ordinary EOF waiting has no newly invented independent deadline. Explicit request timeout, abort or stop bounds an incomplete pipe drain. The helper caller in shell_environment_and_background_stdin_match_daemon_contract was adapted to the Result return type.

Unknown diagnostics are bounded static phase strings, with an optional static unresolved-cleanup suffix. The Throw constructor carries matching request identity and the existing error code, with no stack trace, command or captured output. Both foreground dispatches proceed to the existing StreamClose; socket failure remains transport uncertainty.

The actual Host chain was traced: generated_production's ordinary Shell request uses BoxRemoteExecManager; matching Throw becomes RemoteThrow; outcome_unknown classifies it as unknown; ForeverBox projects it to ToolOutcomeUnknown; RunnerBoxToolBridge returns that error unchanged. The real tool guard marks DownstreamOutcomeUnknown before receipt.complete. StreamAttemptRuntime keeps that unknown across later generations and rejects further tool admission and checkpoint revision admission. ProductionTurnRunShellAdapter checks custody before persistence and before retry; its unknown/unsettled path produces SAND-E0407 and cannot replay the first effect.

The new consumer tests exercise those actual owners over an authenticated loopback daemon built as a cfg(test) library executable. Their ProbeAttempt is a controlled provider-attempt executor, not an actual external provider. The tests require the producer-specific error phase, real prefix observation, one effect, zero later dispatches, one provider attempt, rejected checkpoint I/O, and preserved unknown custody. Known success/nonzero/pre-spawn refusal controls use the actual checkpoint store and read the stored bytes. They are scoped consumer evidence, not real ecosystem or shipping acceptance.

The disconnect control now parses a positive PID, obtains that same captured Child owner from the registry, observes it live with zero termination signals before disconnect, then observes terminal status with a signal after disconnect. This is materially stronger than an external kill-status probe or an invalid PID string.

The CI capture independently builds the normal daemon and the cfg(test) library executable, uses --locked with the tracked daemon lock, selects one successful library test artifact from Cargo compiler metadata by source path, target, kind and profile, rejects absent/duplicate/foreign/failed artifacts, and binds executable and lock bytes before and after. The consumer environment receives only the selected test executable. Build failure prevents the corresponding live controls from using a stale executable. The required-name oracle requires exactly the expected positives and a matching complete libtest summary. The three added cases require six unit, two consumer and one disconnect positives. Existing BXR case requirements remain present.

The existing workflow invokes capture_portable_contracts.py in the Linux portable lane and uploads the containing .evidence/sqlite-native-path directory. These source observations do not claim that the new commands compiled or passed.

The ledger and RTM changes add SC01–SC07 references and describe the narrow capture prerequisite. The reviewed provenance responsibility retains acceptance=pending, verification=not-run and wiring=not-wired. This change does not upgrade full generated-HTML provenance or global acceptance.

## Read depth and exact authority

Read the root AGENTS.md at the subject. The complete recursive subject tree was obtained with truncated=false; there is no applicable nested AGENTS.md for these paths. Independently read the commit object and confirmed its tree, then matched every one of the seven manifest path/blob pairs against that tree.

All declared changed hunks were inspected from the complete diff and current source. Deep read: all changed producer structs/functions, run_process, stop/handle stop, signal/cleanup, capture reader/workers/faults, foreground dispatch, all new unit tests, changed disconnect test and helper call, complete Host process-contract test file and its new consumer fixtures. Adjacent streamed foreground registry insertion/removal and stream-to-background handoff were inspected because they share ownership state. Read the scoped source-contract and applicable BXR/SC specification, capture changes plus input/build/result/receipt consumers, and all changed ledger/RTM hunks with their owning record context.

Seven scoped blobs:
- server.rs: c586ea57bec0bd0513a5f21f4532b98c08fbaf7b
- Host process contract: 23941b650376d23c07c67064f49c083bf7db8689
- capture_portable_contracts.py: 4084e2cc84a2a2957e2a7829dad185ad1ef55f09
- parity ledger: 3c947cf0ce6f87a441104a414023f7e8fbfdf293
- RTM: 2f247e19440928eeeaf7cec5fa9fef6c47bd16a8
- source-contract: 56f2b90582c2de7b1fe8e04efd7bd397e9a25f35
- unchanged specification: f495d1a04dcf39453f0a6da5b7a2bce310e7dd44

Adjacent dependency/consumer contracts read at the same source commit:
- generated_production.rs 18ada294465dd3f7defa6772fa05858ecb2fbea8 — Shell execution, transport boundary and error classification.
- box_remote_accessor.rs e6470d1769972f73c90835c368f2164c7055c7e0 — response identity, matching Throw and completion.
- forever_box/runner_tools.rs 0051f2ae387a036f5cfe78afb9b329b3efe47e48 — actual connection, lifecycle selection and Shell/error projection.
- runner/box_tool_access.rs 35f217614fd832b3a482394cd70638f67d865efc — registered Shell bridge and error forwarding.
- runner/tools/mcp_meta_tools.rs aed2d181ba495ad529f4eb083a42a76b6c460439 — guarded admission, unknown-before-completion and wait.
- runner/stream_attempt.rs 6eb8a3eae5556eb6d6a6c03d00059d7c1180190f — sticky custody, admission, completion and checkpoint fences.
- runner/production_turn_run_shell_adapter.rs d0fe2714923338dd717284f08f05eef438db69d9 — attempt execution, checkpoint callback and retry refusal.
- runner/routed_provider_runtime.rs a351081d6e8e9e9c1fb51439f4d67b48e4ace886 — actual checkpoint store publication.
- HostBox c841d53cc412e8813ce1904cfb90e329fff9dd30, ForeverBoxService 2c549d54d607b567289a21e5c5a370f834bdbe37, ProductionBoxEnvironment 96aa8c1ccf7ca2c0f8a5085a11279d29c113706e, box_factory b0d446757dcb036f91a835f947db53b74f83aa45 and LoopbackSandBox ac2fcd557f09eb3c5fce9e8d22001dd6600266d3 — constructed test chain and actual accessor selection.
- remote_box_resources.rs fdcde82da1714113fd19442ec85cd826e3244232 — ordinary Shell lifecycle delegates once.
- workflow 0c2b44b51a129ce2deb12599676a94a87ed7a91d — ordinary daemon/test commands, exact-source setup, portable capture invocation and artifact collection.
- Daemon manifest 477f8705412d38ed775bcd2f17058689b1ef0c0e, lib 441d1bd8c692c8db3548154f7bccbb1fa4f60082, entrypoint eca0d794be7f61fd3906b41697fa2b2767677846 and cli 553d7bd05bef899d3c9ff7cf1c9a2b9a1ace68b5 — normal/test build identity and shutdown caller.

This is risk-directed contract reading of adjacent files, not a claim that every unrelated function, upstream source, registry crate or platform implementation was deeply reviewed. No test was executed and no runtime, packaged, external service, full migration or final acceptance verdict is granted.
