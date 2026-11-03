# BXR-SHELL-CAPTURE-v1 / SC02-R2 — independent STRUCTURE review

Verdict: MUST_FIX. One concrete ownership defect in the changed stream spool naming prevents structure approval of this frozen candidate. The process registration, actual Child signaling, exceptional resource guard, handoff transfer and completed-recovery observation otherwise follow the existing owner. This is a source-review verdict only; no executable verification or product acceptance is asserted.

## Review identity and independence

- Task: 647e0932-bea4-4113-9a61-65ae3db58955.
- Reviewer lens: STRUCTURE, including reuse, performance and lifetime integration.
- Model/effort: parent launched this independent reviewer with gpt-6.1-sol / xhigh; lifetime/concurrency and real producer/consumer custody justify xhigh.
- Repository independently resolved through GitHub: bhrumom/fabushi-desktop, repository id 1375312893. Authenticated login bhrum; repository permissions include push=true; independent collaborator check returned admin.
- Actual source subject: 00bb59e487e55b89cb41bc9d25b14e3b707e9b6a, tree 206cb590550ac71497b5a79227d3f752368ea90c. Git commit metadata and the complete, nontruncated recursive tree were fetched independently.
- Original base: ca32f83f13a8aed439e9e3341d175405b1d1a655, tree f609dcdc22418f313177c3d86e01103a226799de.
- Prior source subject, used solely as the proper-delta base: 152467880098cd87581ca279be4d2bc872a7620b, tree 8fe5821505d342518628d56aa2920cc7b895284c.
- Packet carrier independently resolved: 4b47a31ab70b85bf807460a726fe9e8e3feb5728, tree 8280e61f283fd309b90fa1b5e14cb4071506def5. Its scope.json/full.diff/fix-delta.diff path blobs independently match 85556822e87d69d380da661bea98b1ca95c9a1eb / 78518c73c7f43a5c53535b08c0dbb8ed70ba4f57 / b725746ac49ae28eef283cf955b8ca4d607fb39f.
- Fixed U independently resolved: openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845, tree 90c8aab39972da11cfad220361781d49e1bc6700.
- Root AGENTS.md read; no more-specific applicable AGENTS.md was found in the exact source tree. Relevant SOURCE_OF_TRUTH, master spec §17, module-map ownership boundaries, locked-source declaration, BXR-SHELL-CAPTURE SC01–SC07 and SC02-R2 were inspected.
- No prior reviewer report, arbitration/decision file or another lens verdict was opened. References and historical author notes embedded in the required current source-contract were not used as independent proof.
- No local/htch execution, compilation, lint, parser/generator/test validation, Actions dispatch, production/shared-source edit or ref change occurred. Read-only GitHub retrieval and source/metadata text inspection were used. This new report is the only proposed write.

All seven original-scope identities were independently matched against the actual subject:

| Path | Blob |
| --- | --- |
| source/box-exec-daemon/src/server.rs | 20b6a1635c091a5cacbc0bbdabff17cdc041e90f |
| source/host/tests/box_exec_daemon_process_contract.rs | 23941b650376d23c07c67064f49c083bf7db8689 |
| docs/specs/g11-box-browser-wire-completion.md | 707256fe48cae8ac3667464bbf1bfa5c1ece6e53 |
| projects/openclaw-rust/evidence/box-shell-capture-647e0932-20261103/source-contract.json | 85c03a40ad4e65993e26cc09f1b7f00d50885fc6 |
| projects/openclaw-rust/scripts/capture_portable_contracts.py | 40cb3b4e8a629e36b04ecb6654035d868b6b57bb |
| projects/openclaw-rust/parity-ledger.json | f8c163b0e3679fe7ac7fbd8d6d2103c037b22886 |
| projects/openclaw-rust/requirements-traceability-matrix.json | 073e031c67647c06a848f6ade737b8fca5b1399b |

## MUST_FIX

### SC-R2-STRUCT-01 — a runtime-local registration id is used in a shared spool pathname

Priority: P2. Classification: implementation ownership defect. Exact changed anchor: source/box-exec-daemon/src/server.rs:1298–1308, particularly the pathname at 1300 and cleanup registration at 1301. Supporting anchors: next_foreground_id initialization at 927; default terminal root at 2240–2243; normal spool unlink at 1481; timed handoff pathname read at 1535; exceptional cleanup unlink at 694–696.

ForegroundId correctly prevents reuse inside one BoxExecRuntime, but its value starts at 1 in every runtime. The spool pathname now contains only that runtime-local value: .shell-stream-<id>.tmp. Public start_box_exec_daemon accepts distinct workspaces and ports while defaulting all unspecified terminal directories to the same temp_dir()/sand-box-terminals. There is no singleton/root-exclusivity admission check. Thus two ordinarily configured runtime instances can both own foreground invocation 1 and address the same spool file.

Concrete failure sequence:

1. Start daemon/runtime A and B with different workspace roots and ephemeral ports, and omit terminals_directory for both, or explicitly give both the same valid terminal root.
2. Start the first actual stream in A; it opens .shell-stream-1.tmp and writes a prefix while its child remains live.
3. Start the first actual stream in B. Its create(true), truncate(true) open truncates A's spool. Both runtimes then write through independent File values/mutexes referring to the same pathname/inode, so stream data can overwrite or mix.
4. Let A finish before B's timed-background transition. A's normal completion removes .shell-stream-1.tmp while B still owns its live child and file.
5. B's handoff reads the pathname at 1535 and receives NotFound, or, under another ordering, copies mixed/truncated bytes into its terminal. The first outcome forces an entered execution into exceptional unknown/termination because another valid invocation removed its spool.

The new cleanup_paths.push occurs before the file open establishes ownership. Exceptional cleanup can therefore also unlink a colliding/pre-existing pathname that this invocation did not exclusively create. This is an ownership error in the changed resource lifetime, not a request for cross-platform acceptance or hostile-filesystem hardening. The earlier PID-based name did not deterministically collide between two live first invocations in separate runtimes.

Smallest owner-preserving correction: keep ForegroundId and the existing registry/StreamRequest guard, but create each spool exclusively in a namespace safe across runtime instances sharing the configured terminal directory; record the path for cleanup only after successful owned creation. A creation collision must not truncate or later delete another invocation's file. No second registry, scheduler, protocol field or new runtime is needed. Merely adding a runtime-local counter to another runtime-local name is insufficient.

Focused validation after an accepted correction should use two actual runtimes sharing a terminal root, real child prefixes, and explicit fixture releases. Hold B's transition until A's completion/cleanup, then require B's timed handoff and complete output to remain intact and each invocation to clean only its own spool. Include a pre-existing candidate-path control if exclusive creation is chosen. This scenario is source-derived and was not executed by this reviewer.

## NON_BLOCKING

### SC-R2-STRUCT-NB01 — polling and aggregate shutdown latency remain unmeasured

Anchors: server.rs:2093–2155 and 2785–2808 contain the caller and two capture-reader polling loops; 1881–1932 and 818–854 contain stop and per-child cleanup.

Idle ordinary capture uses a 10 ms sleep in each nonblocking reader and in the caller. This provides a cancellation observation point but introduces recurring wakeups whose actual CPU/power/latency cost has no execution evidence here. The spec explicitly keeps active reader wakeups as an open performance/soak question (g11-box-browser-wire-completion.md:465).

Also, the outer stop deadline is checked after iterating the snapshot; each newly encountered captured owner may consume its own cleanup limit. With several retained live children whose termination cannot be observed, a stop call can take multiple per-child cleanup windows. The source does not establish an aggregate two-second shutdown SLA. This observation does not show false clean shutdown and is not a fix-round trigger for this slice. A future global shutdown SLA should use a shared deadline/remaining budget or bounded round-robin observation inside this same owner, and be qualified with retained-owner stress controls. Do not infer performance/soak acceptance from the functional tests.

## Structure and lifetime findings supporting the rest of the design

1. **Existing ownership is reused.** The only foreground process map is BoxExecRuntime.foreground_processes at 891–914. ForegroundId allocation at 1966–1971 uses checked atomic increment without wrap; both capture admission (2040–2056) and stream admission (1261–1282) allocate before spawn while holding the map admission fence. Numeric PID is observational data. remove_completed_capture (1984–2003) checks both the exact id and Arc identity, so a reaped older capture cannot erase a successor.

2. **Signals use the actual owned child.** CapturedChild.poll/signal at 705–739 observes status and signals only before an observed reap, under the same owned state. stop no longer signals foreground stream PID data; active streams consume stopping in shell_stream_live (1408–1418), and unresolved Stream markers prevent clean stop (1914–1930). Captured-owner locks are released before map removal, and snapshot readers do not hold the map while acquiring a Child mutex (1935–1964). I found no reverse foreground/Child lock cycle in the affected paths.

3. **The actual stream resources survive exceptional returns.** StreamRequest is armed immediately after spawn with CapturedChild and StreamRecovery, before fallible spool setup or worker creation (1268–1277). Every started pipe worker is stored in that guard. Setup/emitter/poll/handoff errors and controlled unwind enter Drop (784–812), which replaces the exact Stream registration with a Capture holding the real Child/readers. Direct-child cleanup is bounded and does not synchronously join a reader still blocked on an inherited pipe.

4. **Thread creation does not take the sole child owner prematurely.** handoff_shell_stream_to_background (1505–1632) creates a receiver thread before taking resources from the guard. The closure initially owns the channel receiver rather than the only Child/readers. On send error, resources return to the guard; locks are explicitly released before joining that failed worker. The foreground/background lock order is consistent. After successful send, the receiver waits for the background map lock, preventing reap/removal before insertion commits. Background completion remains the existing owner.

5. **Completed exceptional recoveries can be reclaimed.** observe_stream_recovery (682–703) joins only finished handles, requires actual reap plus observation of both started readers, closes parent pipe handles, and marks capture_complete. Drop observes immediately after bounded cleanup (803–804). Both ordinary capture admission (2036) and stream admission (1245) inspect retained stream recoveries through 1935–1964. Ordinary active Capture workers remain with their caller. Removal requires both terminal state and exact identity/Arc; a pending reader or poll error is retained. Later cleanup does not change the already unknown request result.

6. **Unknown result classification stays in the existing chain.** run_process (2029–2182) retains post-spawn typed uncertainty, observes both capture workers even after the first failure, and does not substitute partial or empty capture. Both foreground dispatches emit Throw, then existing Close (1804–1830, 1868). Actual consumer functions read include BoxRemoteExecManager.create_exec_instance, ProductionBoxResourceAccessor.execute_shell_result/outcome_unknown, ForeverBoxRunnerResourcePort.execute_shell/project_box_exec_error, RunnerBoxToolBridge.call_tool, the real guarded tool worker, StreamAttemptRuntime custody/checkpoint admission and ProductionTurnRunShellAdapter's actual checkpoint/retry paths. The guard marks DownstreamOutcomeUnknown before receipt.complete (mcp_meta_tools.rs:198–207). Existing checkpoint/retry fences remain authoritative.

## Oracle and capture inspection

All thirteen daemon SC test functions, their real-child helpers, both new Host consumer functions and their actual producer/guard/checkpoint fixture were read in full. The corrected disconnect function was read with its production monitor/dispatch caller.

- Reused-PID controls retain two real child generations; they alter registry observational PID only, and explicitly test exact-id/Arc removal and no signal after reap.
- The stream stop control presents an actual live sentinel's PID as observational data, then proves the actual request performs termination and a blocked request prevents clean stop.
- The handoff control waits on owned child stdin until after Backgrounded; an independent countdown cannot settle it before the assertion.
- Repeated setup-error control installs TERM-ignore before its observed effect, induces a real filesystem setup failure, repeats three times without stop, and inspects the actual transferred Child state and released parent pipes.
- The inherited-pipe fixture keeps pipes open until an explicit owned release. It labels the unwind at successful actual reap as controlled rather than an OS wait-failure replay. Both ordinary and stream admission controls wait for real worker completion, then require the normal production request boundary to perform the joins/removal.
- The pre-aborted control proves its bounded abort/result shape; it does not establish TERM-ignore trap readiness for that same abort.
- Host negatives require actual loopback producer Throw, one counted effect, the typed error through the real tool guard before its deadline, sticky downstream custody, refused later tools/checkpoint/replay, one provider attempt and no unknown checkpoint directory. Known controls exercise actual persisted checkpoint bytes.
- Disconnect now resolves the published positive PID to the registered Capture, proves that exact Child alive with zero prior signals, disconnects, and requires its actual observed reap and termination signal.

capture_portable_contracts.py was read at the affected source-binding lists, all changed hunks, full relevant artifact selection/build/result functions and final acceptance. It requires 13 distinct daemon unit names + 2 actual Host/Runner names + 1 disconnect name. Its BXR oracle compares exact sorted names, requires unique required names, one complete summary and zero failed/ignored/measured results, rejecting missing/extra/duplicate positives. The retained eight BXR cases still declare the existing 77 direct names, and the 14 downstream custody names remain required. Plugin additions and LXR metadata amendments were inspected for preservation, not accepted as product work by this review.

The separate cfg(test) producer build requires tracked Cargo.lock, --locked, successful current compiler metadata, one library-test target with the exact lib.rs source and an executable inside its declared target directory. Missing/ambiguous/foreign artifacts fail closed. Sources, lock and both shipping/test binaries are bound before and after the cases, and a failed build does not fall back to an unbound executable. The existing Linux workflow still invokes this capture at its established exact-H step; no new execution lane is introduced. These are authored evidence contracts, not successful execution evidence.

## Actual reading depth and remaining limits

Deep reading covered complete affected server ownership/cleanup/dispatch/run_process/stream/handoff/stop/admission functions, capture worker implementations, their relevant actual network/entrypoint callers, all affected tests and needed consumer functions. The original-base and previous-subject diffs were inspected, including removed logic and all seven original paths. I do not claim a full-file deep read of every unchanged protobuf declaration, filesystem operation, unrelated daemon test, full capture script, large ledger/RTM, workflow or Host consumer module. Their affected contracts/hunks and relevant complete rows/functions were inspected.

The current source-contract was read, the complete affected generated-HTML ledger row and R23/AC23 RTM rows were read from the actual full JSON, and both metadata files retain all eight global flags false. Existing generated-HTML wiring blockers, pending acceptance and not-run verification remain present. Independent Plugin/LXR referenced review/decision files were not opened.

Normal active stream TERM-only termination and synchronous post-reap reader joins (1415–1452), and BackgroundProcess raw-PID/reap/setup/stop/input ownership, remain open existing-owner follow-on work. Exceptional cleanup approval would not accept them. Raw capture byte bounds, Shell generator authority, private generated-origin grants, actual durable Session/Attachment consumers, fixed-U dependency/license/source closure, real ecosystems/platform packages, performance/soak and full OCRM acceptance remain unaccepted.

No exact-H compilation, contract execution, negative/wiring result or final acceptance has been produced by this review. Resolve SC-R2-STRUCT-01 through the ordinary runner decision/fix/re-review lifecycle, then obtain the separately required exact-H Actions evidence.
