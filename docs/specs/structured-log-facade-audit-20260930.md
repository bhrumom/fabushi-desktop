# Structured-log facade parity ledger — 2026-09-30

Reference: frozen Grok 0.18 blob `5e19374e02f4984899ea48f06b1d70798bd17418`.

Audit lineage starts at `28eebfe882760decbabe2c8ce83bf27d110eefca`; this follow-up is prepared from exact PR HEAD `95b4f4c7ef67e9f3ef6420b78ace9404257aca55` and remains non-final until the resulting commit has exact-HEAD Actions evidence.

This ledger is intentionally stricter than file-existence or mapper-existence checks. A frozen `SandStructuredLogTelemetry` entry is accepted only when its Fabushi responsibility has a real shipping producer/call site, remains under the single Host structured-log owner, preserves mapping/settlement/confirmed/dispose semantics, and has focused contract evidence.

Status vocabulary:
- **verified-owner** — the shipping Host owner and the relevant transport/lifecycle behavior are evidenced.
- **delegated-nonfinal** — the facade delegates to another mapped module whose own architecture row is still `existing-needs-parity`; this facade cannot be final before that delegate is final.
- **producer-evidence-required** — a mapper/event shape exists, but this audit did not find a shipping producer through the canonical structured-log owner by the frozen facade method name; module presence alone is insufficient.
- **stateful-gap** — the frozen API carries state/lifecycle behavior that is not represented by the current generic `TelemetryService::report` boundary.
- **implemented-awaiting-exact-head-ci** — production wiring and focused contract were added after the audit rollback, but the row remains non-final until the new exact HEAD is accepted.

## SandStructuredLogTelemetry public surface

| # | Frozen API | Fabushi owner / delegation | Audit status |
|---:|---|---|---|
| 1 | `setHostBundleIdentity` | `HostTelemetryService::set_host_bundle_identity` → `HostStructuredLogTelemetry` → production transport identity tags | verified-owner |
| 2 | `startTurn` | `HostStructuredLogTelemetry::start_turn` now returns a Host-owned stateful turn handle; shipping provider thread creates it from the unique `worker_telemetry_logs`, binds stream request-id/model, accumulates real retry reports, and finalizes before renderer-visible terminal publication | **implemented-awaiting-exact-head-ci** |
| 3 | `reportAutoReviewApproval` | shipping auto-review settlement calls `HostStructuredLogTelemetry::report_auto_review_approval`; typed facade owns the frozen mapper and canonical transport; real JSONL facade contract covers field shaping | **implemented-awaiting-exact-head-ci** |
| 4 | `reportAutomationRun` | automation analytics/telemetry boundary exists, but frozen structured-log facade producer must be evidenced through canonical owner | producer-evidence-required |
| 5 | `reportAutomationFireDropped` | both production backend-fire rejection and Transcript dropped-fire reporter call `HostStructuredLogTelemetry::report_automation_fire_dropped`; typed facade owns mapper/transport | **implemented-awaiting-exact-head-ci** |
| 6 | `reportAutomationShadowPrune` | `automations/production_lifecycle.rs` calls `HostStructuredLogTelemetry::report_automation_shadow_prune` | verified-owner |
| 7 | `reportConversationGc` | production `ConversationGcReport` global reporter is pinned once by `ProductionHostExtensions` to `HostStructuredLogTelemetry::report_conversation_gc`; adapter preserves failed/skipped/collected and unresolved-ref/count fields; focused adapter contract covers skipped semantics; shutdown unpins before telemetry dispose | **implemented-awaiting-exact-head-ci** |
| 8 | `reportBoxDiskPressure` | `disk_pressure_telemetry.rs` mapper exists | producer-evidence-required |
| 9 | `reportHostEventLoop` | `HostTelemetryService` owns `EventLoopTelemetryRuntime` and reports `event_loop_window_telemetry` through its single log owner | verified-owner |
| 10 | `reportExperimentsDiagnostic` | pinned experiments diagnostics reporter calls `HostStructuredLogTelemetry::report_experiments_diagnostic` | verified-owner |
| 11 | `reportHostDiagnostic` | production HostDiagnostic global reporter is pinned once by `ProductionHostExtensions` to `HostStructuredLogTelemetry::report_host_diagnostic`; adapter preserves stage/agent/reason/error class and focused contract verifies representative frozen fields; shutdown unpins before telemetry dispose | **implemented-awaiting-exact-head-ci** |
| 12 | `reportHostEventBusFailure` | `host_event_bus_telemetry.rs` mapper exists | producer-evidence-required |
| 13 | `reportHostExtensionDiagnostic` | production extension composition calls `HostStructuredLogTelemetry::report_host_extension_diagnostic` | verified-owner |
| 14 | `reportSessionDiagnostic` | production SessionDiagnostic global reporter is pinned once by `ProductionHostExtensions` to `HostStructuredLogTelemetry::report_session_diagnostic`; adapter maps all four frozen families plus store-db recovery fields; focused adapter contract covers family/outcome/salvage semantics; shutdown unpins before telemetry dispose | **implemented-awaiting-exact-head-ci** |
| 15 | `reportSearchIndexHealth` | production content-search composition calls `HostStructuredLogTelemetry::report_search_index_health` | verified-owner |
| 16 | `reportLocalExecRefused` | production Local Exec refusal reporter calls the typed Host structured-log facade; frozen mapper and Sand error tags remain under the single owner | **implemented-awaiting-exact-head-ci** |
| 17 | `reportLocalExecProvider` | production provider lifecycle reporter calls the typed Host structured-log facade; mapper semantics remain unchanged | **implemented-awaiting-exact-head-ci** |
| 18 | `reportLocalExecFailed` | shipping Gateway Local Exec failure reporter calls the typed Host structured-log facade; mapper semantics remain unchanged | **implemented-awaiting-exact-head-ci** |
| 19 | `reportWebAuthnProxy` | repaired production WebAuthn callback now calls `HostStructuredLogTelemetry::report_webauthn_proxy` instead of stderr-only logging; typed mapper/error tags flow through canonical transport | **implemented-awaiting-exact-head-ci** |
| 20 | `reportMemorySynthesis` | production memory synthesis reporter and gate-disabled path use the unique Host logs; reporter now calls the typed facade method and preserves mapper/error semantics | **implemented-awaiting-exact-head-ci** |
| 21 | `reportHostStartup` | `lifecycle_telemetry.rs` mapper exists | producer-evidence-required |
| 22 | `reportHostLifecycle` | `HostLifecycleProgress` reports via the service log owner, but frozen facade equivalence still needs focused contract evidence | producer-evidence-required |
| 23 | `reportDaemonPing` | lifecycle mapper exists / generic TelemetryService event mapping exists | producer-evidence-required |
| 24 | `reportBoxImageCheck` | lifecycle mapper exists | producer-evidence-required |
| 25 | `enqueueBoxInfrastructureEvent` | `box_infrastructure_telemetry` + `HostStructuredLogTelemetry::report_box_log_record` cover infrastructure projection | verified-owner |
| 26 | `reportBoxBootStage` | generic TelemetryService event mapping exists | producer-evidence-required |
| 27 | `reportBoxBootFailure` | lifecycle mapper exists | producer-evidence-required |
| 28 | `reportEgressTunnel` | lifecycle mapper exists | producer-evidence-required |
| 29 | `reportHostBootFetch` | lifecycle mapper exists | producer-evidence-required |
| 30 | `reportExecDaemonRestart` | generic TelemetryService event mapping exists | producer-evidence-required |
| 31 | `reportSupervisorRestart` | generic TelemetryService event mapping exists | producer-evidence-required |
| 32 | `reportBoxBootStageConfirmed` | `HostStructuredLogTelemetry::report_box_boot_stage_confirmed` uses `ship_confirmed_projection` | verified-owner |
| 33 | `reportTurnInterrupt` | delegates to `turn_telemetry_mappers.rs`; that architecture row is non-final | **delegated-nonfinal** |
| 34 | `reportTurnAwait` | delegates to `turn_telemetry_mappers.rs`; row is non-final | **delegated-nonfinal** |
| 35 | `reportTurnRetry` | delegates to `turn_telemetry_mappers.rs`; row is non-final; frozen also mutates active-turn retry state | **delegated-nonfinal** |
| 36 | `reportUserMessageReceived` | delegates to `turn_telemetry_mappers.rs`; row is non-final | **delegated-nonfinal** |
| 37 | `reportClosingSendNudge` | delegates to `turn_telemetry_mappers.rs`; manifest explicitly says shipping closing-send producer remains missing | **delegated-nonfinal** |
| 38 | `reportSubagentRevival` | production completion revival runtime calls `HostStructuredLogTelemetry::report_subagent_revival`; typed facade owns the revival mapper and canonical transport | **implemented-awaiting-exact-head-ci** |
| 39 | `reportShellRevival` | shipping CompletionRevivals now branches on `RevivalReport.kind` and routes shell completions through `HostStructuredLogTelemetry::report_shell_revival`; the typed frozen mapper owns `sand.shell.revival` field/level semantics and the JSONL facade contract verifies the distinct shell event | **implemented-awaiting-exact-head-ci** |
| 40 | `reportTtft` | delegates to `turn_telemetry_mappers.rs`; row is non-final and manifest still requires complete dispatch-clock/trace/span provenance | **delegated-nonfinal** |
| 41 | `reportSendDispatch` | `queue_telemetry_mappers.rs` mapper exists | producer-evidence-required |
| 42 | `reportQueueAccepted` | shipping run-queue accepted callback calls the typed Host structured-log facade; JSONL contract covers frozen lane/depth metadata | **implemented-awaiting-exact-head-ci** |
| 43 | `reportQueueDequeued` | shipping run-queue dequeue callback calls the typed Host structured-log facade; JSONL contract covers timing/depth projection | **implemented-awaiting-exact-head-ci** |
| 44 | `reportQueueWatchdog` | shipping queue watchdog callback calls the typed Host structured-log facade after interrupt handling; JSONL contract covers warning/timing projection | **implemented-awaiting-exact-head-ci** |
| 45 | `reportAckObligation` | `queue_telemetry_mappers.rs` mapper exists | producer-evidence-required |
| 46 | `reportPendingWake` | shipping PendingWakeRearm runtime still publishes the Gateway pending-wake event and now additionally routes the same report through `HostStructuredLogTelemetry::report_pending_wake`; typed mapper preserves kind/work/age/reason/quiet-origin semantics and JSONL facade contract verifies rounding/event ownership | **implemented-awaiting-exact-head-ci** |
| 47 | `reportTurnUsage` | delegates to `turn_telemetry_mappers.rs`; row is non-final and provider token-usage propagation remains open | **delegated-nonfinal** |
| 48 | `reportTurnEmptyDelivery` | shipping provider terminal settlement calls `HostStructuredLogTelemetry::report_turn_empty_delivery`; typed mapper remains under canonical owner and JSONL contract covers empty-delivery fields | **implemented-awaiting-exact-head-ci** |
| 49 | `reportJournalOutcome` | `journal_outcome_telemetry.rs` mapper exists | producer-evidence-required |
| 50 | `reportComputerUseUsage` | shipping generated-subagent settlement now consumes `settled.computer_use_usage`, maps it with existing `computer_use_usage_telemetry`, and reports through the unique `worker_telemetry_logs`; exact-HEAD CI still required | **implemented-awaiting-exact-head-ci** |
| 51 | `reportToolCallError` | generic TelemetryService event mapping exists, but frozen field shaping/capping must be proven by shipping producer contract | producer-evidence-required |
| 52 | `reportToolCallStalled` | generic TelemetryService event mapping exists, but frozen field shaping/capping must be proven by shipping producer contract | producer-evidence-required |
| 53 | `reportToolCallStarted` | generic TelemetryService event mapping exists, but frozen field shaping must be proven by shipping producer contract | producer-evidence-required |
| 54 | `reportAgentError` | generic mapping exists; frozen emits error + optional detail event with Sand error tags/truncation | producer-evidence-required |
| 55 | `reportAutoReviewDisplayRecheckFailed` | direct frozen event has no canonical shipping producer evidence in current audit | producer-evidence-required |
| 56 | `reportAutoReviewExpireSweepFailed` | generic TelemetryService event mapping exists | producer-evidence-required |
| 57 | `reportBoxStoreSyncCycle` | direct frozen event; Box Store Sync producer path must be pinned | producer-evidence-required |
| 58 | `reportBoxStoreDbCapture` | direct frozen event; Box Store Sync producer path must be pinned | producer-evidence-required |
| 59 | `reportBoxStoreManifestConflict` | direct frozen event; Box Store Sync producer path must be pinned | producer-evidence-required |
| 60 | `reportChromeSessionStage` | direct frozen event; producer path must be pinned | producer-evidence-required |
| 61 | `reportMcpAuthCleanup` | direct frozen event; production MCP cleanup producer path must be pinned | producer-evidence-required |
| 62 | `reportMcpDiscoveryFailed` | direct frozen event; production MCP discovery producer path must be pinned | producer-evidence-required |
| 63 | `reportConnectorAuth` | frozen connector-auth mapper + host boundary; production producer path must be pinned | producer-evidence-required |
| 64 | `reportLocalToolPermissionStrandedRetirement` | direct frozen event; production permission retirement producer path must be pinned | producer-evidence-required |
| 65 | `reportSkillPublishEdgeFailed` | direct frozen event; production producer path must be pinned | producer-evidence-required |
| 66 | `reportPluginSkillsSync` | direct frozen event; production plugin-skills producer path must be pinned | producer-evidence-required |
| 67 | `reportTeachRecordingCapStopFailed` | Host structured-log helper exists | producer-evidence-required |
| 68 | `reportTeachRecordingStartFailed` | Host structured-log helper exists | producer-evidence-required |
| 69 | `reportBoxCopyIn` | direct frozen event; production producer path must be pinned | producer-evidence-required |
| 70 | `reportBoxRecreateDecided` | direct frozen event; production producer path must be pinned | producer-evidence-required |
| 71 | `reportInferenceCredentialRenewal` | `HostTelemetryService` owns auth renewal subscription and calls `report_inference_credential_renewal` | verified-owner |
| 72 | `reportGatewayCommandError` | direct frozen event; production Gateway producer path must be pinned | producer-evidence-required |
| 73 | `reportGatewayCommandTiming` | direct frozen event; production Gateway producer path must be pinned | producer-evidence-required |
| 74 | `reportAutomationLifecycle` | generic TelemetryService mapping exists; producer/field parity needs focused proof | producer-evidence-required |
| 75 | `reportHostLog` | Host console forwarder calls the unique `HostStructuredLogTelemetry::report_host_log` owner and truncates to frozen max length | verified-owner |
| 76 | `reportBoxLogBatch` | BoxLogShipper batches records into `HostStructuredLogTelemetry::report_box_log_record` | verified-owner |
| 77 | `reportBoxLogShip` | BoxLogShipper reports via `HostStructuredLogTelemetry::report_box_log_ship` | verified-owner |
| 78 | `reportDesktopHealth` | `DesktopHealthForwarder` is owned by `HostTelemetryService`; focused frozen facade projection still needs explicit ledger evidence | producer-evidence-required |
| 79 | `reportAgentOpen` | direct frozen event; production producer path must be pinned | producer-evidence-required |
| 80 | `reportHostCrash` | Host crash marker owner exists, but ordinary fire-and-forget crash projection producer must be distinguished from confirmed exit forwarding | producer-evidence-required |
| 81 | `reportHostProcessExitConfirmed` | crash-marker forwarding uses confirmed structured-log shipping under Host ownership | verified-owner |
| 82 | `reportInvariantViolation` | direct frozen event; producer path must be pinned | producer-evidence-required |
| 83 | `reportHostUpgrade` | Host Upgrade production dependency has a shipping telemetry callback | producer-evidence-required |
| 84 | `reportHostUpgradeConfirmed` | Host Upgrade production dependency exposes confirmed reporting and current Host sink has `ship_confirmed_projection` | verified-owner |
| 85 | `reportBoxHelp` | `HostStructuredLogTelemetry::report_box_help` exists; production call site still must be pinned | producer-evidence-required |
| 86 | `reportBotBlock` | generic mapping does not by itself prove frozen two-event summary/detail semantics | producer-evidence-required |
| 87 | `emitTurnEvent` | returned Host turn handle emits start/outcome/outcome_detail through the same `HostStructuredLogTelemetry::report_projection` owner | **implemented-awaiting-exact-head-ci** |
| 88 | `setFlushTickListener` | production transport is constructed with the pressure-profiler flush tick listener under Host ownership | verified-owner |
| 89 | `dispose` | `HostTelemetryService::dispose` disposes pressure profiler, structured-log transport, sink/tracing; transport drain is idempotent | verified-owner |
| 90 | `enqueue` | `HostStructuredLogTelemetry::report_projection` → `ProductionStructuredLogTransport::enqueue`; bounded/retry/identity semantics covered by focused transport contract | verified-owner |

## Returned SandTurnTelemetryImpl surface

`startTurn` is not just a one-shot report. The frozen returned object owns state that affects later telemetry:

| Frozen API | Frozen responsibility | Current audit |
|---|---|---|
| `noteRetry` | increments retry count, accumulates backoff, records retry cause until finalized | implemented on `HostStructuredLogTurnTelemetry`; shipping `ProviderRetryReport::Retried` calls it |
| `setModel` | latches model and emits turn-start once | implemented on Host turn handle; initial shipping `model_id` is supplied at start and later setter retains exactly-once start latch |
| `setRequestId` | first non-empty request-id wins | implemented on Host turn handle; shipping stream id is bound immediately after creation |
| `finalize` | one-shot finalization, duration, retry metadata, Sand error tags, outcome + optional detail event, active-turn removal | implemented; provider settlement maps waiting-user/cancel/completed/error before renderer-visible terminal publication |
| `baseTags` | canonical turn_type/conversation_id/request_id/model_intent projection | implemented on Host turn handle |
| `emitStart` | exactly-once turn-start emission | implemented by `start_emitted` latch; focused contract exercises late model + repeated model/finalize |

## Acceptance consequence

At audited HEAD `28eebfe882760decbabe2c8ce83bf27d110eefca`, the transport/backend/lifecycle work is real and should be retained, but the frozen facade is **not fully evidenced**. In particular:

1. `startTurn` / `SandTurnTelemetryImpl` stateful semantics were missing at the audited rollback HEAD; the follow-up implementation now restores them in the unique Host owner and shipping provider thread, pending exact-HEAD CI.
2. Eight facade entries delegate to `turn-telemetry-mappers.ts`, whose manifest row is still `existing-needs-parity`.
3. Numerous other frozen facade entries currently have mapper/event definitions without a method-specific shipping producer/call-site proof through the canonical Host owner.
4. A generic `TelemetryService::report(method, Value)` event-name switch is not sufficient evidence for frozen field shaping, multi-event behavior, settlement callbacks, or confirmed shipping.

Therefore `source/host/extensions/telemetry/structured-log-telemetry.ts` must remain `existing-needs-parity` until these facade responsibilities are either:
- wired through a real canonical Host facade/producer with focused contracts, or
- explicitly delegated to independently final mapped modules with their production call sites and semantic contracts recorded here.

This rollback does **not** invalidate the already-proven production transport work (AnalyticsService/SubmitLogs backend, auth/machine identity headers, identity hold/backstop, 3-second flush ownership, bounded buffer, failed-batch retry, replay-age filtering, confirmed shipping, pressure-profiler tick integration, and idempotent drain/dispose).


## 2026-09-30 exact-HEAD facade ownership follow-up

Prepared from PR #20 exact HEAD `95b4f4c7ef67e9f3ef6420b78ace9404257aca55`. This slice deliberately does not edit `turn_telemetry_mappers.rs` and does not mark the structured-log row final. It adds method-specific `HostStructuredLogTelemetry` facade ownership for auto-review approval, automation-fire-dropped, Local Exec refused/provider/failed, WebAuthn proxy, memory synthesis, subagent revival, queue accepted/dequeued/watchdog, and turn-empty-delivery. Their existing shipping producers now invoke those facade methods rather than bypassing the facade with mapper + generic projection.

The audit also found and repairs a real WebAuthn production hole: the prior shipping callback mapped the event but only wrote it to stderr, so no structured-log transport owned the event. The repaired callback captures the unique Host telemetry logs and reports through `report_webauthn_proxy`. `host_telemetry_service_contract.rs` exercises all twelve facade methods against the real JSONL structured-log sink and asserts event names plus representative frozen rounding/error/level fields; this is behavior evidence, not source-string evidence.

Rows not advanced above remain unresolved. In particular shell revival, send dispatch, ack obligation, pending wake, journal outcome, Host/session diagnostics, conversation GC, disk pressure and many direct-event facade methods still require production ownership proof or implementation. The eight turn-telemetry delegate entries also remain non-final. Therefore the parent structured-log manifest row must remain `existing-needs-parity`.

### Shell revival / pending-wake ownership repair

The follow-up audit found two production defects rather than documentation gaps. First, `ProductionCompletionRevivalRuntime::report_revival` projected every completion through the subagent mapper even when the frozen domain report was a shell revival. It now preserves the domain split: shell reports use the typed `report_shell_revival` facade and subagent reports keep `report_subagent_revival`. Second, the PendingWake production runtime previously published only the Gateway event; it now also sends the same lifecycle report through the typed Host structured-log facade while preserving the existing Gateway publication. Both paths are covered by the real JSONL facade contract. These entries remain awaiting exact-HEAD Actions until the repair commit is tested.

### Host / Session / Conversation-GC global reporter ownership repair

The audit confirmed that all three domains already had real business producers, but their global reporter slots were never pinned by the shipping Host, so the events vanished before reaching structured-log. `ProductionHostExtensions` now installs exactly one adapter for Host diagnostics, Session diagnostics and Conversation GC only after the production extension graph has started successfully. The adapters call typed `HostStructuredLogTelemetry` facade methods, and shutdown explicitly unpins them before telemetry disposal; `Drop` repeats the unpin idempotently as a failure-safe. Focused adapter behavior tests cover Host field aliases, Session family/store-db recovery fields, and GC skipped/unresolved-ref semantics. This is real production composition plus semantic tests, not a mapper/string-only claim.
