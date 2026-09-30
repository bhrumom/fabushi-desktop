# Structured-log facade parity ledger — 2026-09-30

Reference: frozen Grok 0.18 blob `5e19374e02f4984899ea48f06b1d70798bd17418`.

Audit lineage starts at `28eebfe882760decbabe2c8ce83bf27d110eefca`; earlier exact-HEAD claims were invalidated as PR #20 advanced, and this slice is rebuilt from exact PR HEAD `f771c289afd2dfbc4c20259560e515d5effe7379`; it remains non-final until the resulting commit has exact-HEAD Actions evidence.

This ledger is intentionally stricter than file-existence or mapper-existence checks. A frozen `SandStructuredLogTelemetry` entry is accepted only when its Fabushi responsibility has a real shipping producer/call site, remains under the single Host structured-log owner, preserves mapping/settlement/confirmed/dispose semantics, and has focused contract evidence.

Status vocabulary:
- **verified-owner** — the shipping Host owner and the relevant transport/lifecycle behavior are evidenced.
- **delegated-nonfinal** — the facade delegates to another mapped module whose own architecture row is still `existing-needs-parity`; this facade cannot be final before that delegate is final.
- **producer-evidence-required** — a mapper/event shape exists, but this audit did not find a shipping producer through the canonical structured-log owner by the frozen facade method name; module presence alone is insufficient.
- **stateful-gap** — the frozen API carries state/lifecycle behavior that is not represented by the current generic `TelemetryService::report` boundary.
- verified-owner — production wiring and focused contract were added after the audit rollback, but the row remains non-final until the new exact HEAD is accepted.

## SandStructuredLogTelemetry public surface

| # | Frozen API | Fabushi owner / delegation | Audit status |
|---:|---|---|---|
| 1 | `setHostBundleIdentity` | `HostTelemetryService::set_host_bundle_identity` → `HostStructuredLogTelemetry` → production transport identity tags | verified-owner |
| 2 | `startTurn` | `HostStructuredLogTelemetry::start_turn` now returns a Host-owned stateful turn handle; shipping provider thread creates it from the unique `worker_telemetry_logs`, binds stream request-id/model, accumulates real retry reports, and finalizes before renderer-visible terminal publication | verified-owner |
| 3 | `reportAutoReviewApproval` | shipping auto-review settlement calls `HostStructuredLogTelemetry::report_auto_review_approval`; typed facade owns the frozen mapper and canonical transport; real JSONL facade contract covers field shaping | verified-owner |
| 4 | `reportAutomationRun` | automation analytics/telemetry boundary exists, but frozen structured-log facade producer must be evidenced through canonical owner | producer-evidence-required |
| 5 | `reportAutomationFireDropped` | both production backend-fire rejection and Transcript dropped-fire reporter call `HostStructuredLogTelemetry::report_automation_fire_dropped`; typed facade owns mapper/transport | verified-owner |
| 6 | `reportAutomationShadowPrune` | `automations/production_lifecycle.rs` calls `HostStructuredLogTelemetry::report_automation_shadow_prune` | verified-owner |
| 7 | `reportConversationGc` | production `ConversationGcReport` global reporter is pinned once by `ProductionHostExtensions` to `HostStructuredLogTelemetry::report_conversation_gc`; adapter preserves failed/skipped/collected and unresolved-ref/count fields; focused adapter contract covers skipped semantics; shutdown unpins before telemetry dispose | verified-owner |
| 8 | `reportBoxDiskPressure` | shipping ForeverBox disk-pressure producer already emits complete fields through the HostDiagnostic domain; the unique production reporter adapter now recognizes `kind=disk_pressure`, converts the full payload to `DiskPressureReport`, and calls typed `report_box_disk_pressure`, whose facade restores frozen event `sand.box.disk_pressure`; adapter + real JSONL contracts verify hard/error and `used_percent` formatting | verified-owner |
| 9 | `reportHostEventLoop` | `HostTelemetryService` owns `EventLoopTelemetryRuntime` and reports `event_loop_window_telemetry` through its single log owner | verified-owner |
| 10 | `reportExperimentsDiagnostic` | pinned experiments diagnostics reporter calls `HostStructuredLogTelemetry::report_experiments_diagnostic` | verified-owner |
| 11 | `reportHostDiagnostic` | production HostDiagnostic global reporter is pinned once by `ProductionHostExtensions` to `HostStructuredLogTelemetry::report_host_diagnostic`; adapter preserves stage/agent/reason/error class and focused contract verifies representative frozen fields; shutdown unpins before telemetry dispose | verified-owner |
| 12 | `reportHostEventBusFailure` | shipping `SandHostEventBus::default()` forwards listener/subscriber failures through a production-pinned reporter owned by `ProductionHostExtensions`; the adapter preserves optional topic semantics and calls typed `HostStructuredLogTelemetry::report_host_event_bus_failure`; behavior contracts trigger both real failure paths | verified-owner |
| 13 | `reportHostExtensionDiagnostic` | production extension composition calls `HostStructuredLogTelemetry::report_host_extension_diagnostic` | verified-owner |
| 14 | `reportSessionDiagnostic` | production SessionDiagnostic global reporter is pinned once by `ProductionHostExtensions` to `HostStructuredLogTelemetry::report_session_diagnostic`; adapter maps all four frozen families plus store-db recovery fields; focused adapter contract covers family/outcome/salvage semantics; shutdown unpins before telemetry dispose | verified-owner |
| 15 | `reportSearchIndexHealth` | production content-search composition calls `HostStructuredLogTelemetry::report_search_index_health` | verified-owner |
| 16 | `reportLocalExecRefused` | production Local Exec refusal reporter calls the typed Host structured-log facade; frozen mapper and Sand error tags remain under the single owner | verified-owner |
| 17 | `reportLocalExecProvider` | production provider lifecycle reporter calls the typed Host structured-log facade; mapper semantics remain unchanged | verified-owner |
| 18 | `reportLocalExecFailed` | shipping Gateway Local Exec failure reporter calls the typed Host structured-log facade; mapper semantics remain unchanged | verified-owner |
| 19 | `reportWebAuthnProxy` | repaired production WebAuthn callback now calls `HostStructuredLogTelemetry::report_webauthn_proxy` instead of stderr-only logging; typed mapper/error tags flow through canonical transport | verified-owner |
| 20 | `reportMemorySynthesis` | production memory synthesis reporter and gate-disabled path use the unique Host logs; reporter now calls the typed facade method and preserves mapper/error semantics | verified-owner |
| 21 | `reportHostStartup` | shipping `source/host/app/src/main.rs` calls the typed Host facade only after background-ready work, HostUpgrade resume and ACK-redrive setup; metadata comes from the real ForeverBox auto-update owner plus HostUpgrade bundle identity, and a focused JSONL facade contract verifies frozen startup mapping including `host_built_at_ms` | verified-owner |
| 22 | `reportHostLifecycle` | shipping Host main owns one `HostLifecycleProgress` across `plugin_graph → identity → log_catchup → transcript_read → ready`, uses the frozen 5-minute stuck watchdog, routes startup failure exits through `fail()`, and sends reports through the typed Host facade using `host_lifecycle_telemetry` so failed/stuck SAND error tags are preserved; the existing exact Host-owner contract plus progress contracts cover mapping/state semantics | verified-owner |
| 23 | `reportDaemonPing` | lifecycle mapper exists / generic TelemetryService event mapping exists | producer-evidence-required |
| 24 | `reportBoxImageCheck` | lifecycle mapper exists | producer-evidence-required |
| 25 | `enqueueBoxInfrastructureEvent` | `box_infrastructure_telemetry` + `HostStructuredLogTelemetry::report_box_log_record` cover infrastructure projection | verified-owner |
| 26 | `reportBoxBootStage` | frozen method delegates to infrastructure mapping; shipping `BoxLogShipper` parses `boot_stage` from the production box telemetry stream and the unique Host log owner calls `report_box_log_record -> box_infrastructure_telemetry`; production contract verifies delivery/offset settlement and exact event | verified-owner |
| 27 | `reportBoxBootFailure` | delegated to the same shipping `BoxLogShipper` infrastructure owner; focused production contract now exercises strict `boot_failure` schema and exact event | verified-owner |
| 28 | `reportEgressTunnel` | delegated to shipping `BoxLogShipper` infrastructure owner; production contract exercises `egress_tunnel` including attempt/exit/runtime fields | verified-owner |
| 29 | `reportHostBootFetch` | delegated to shipping `BoxLogShipper` infrastructure owner; strict parser validates outcome/reason/version provenance and production contract verifies exact event | verified-owner |
| 30 | `reportExecDaemonRestart` | delegated to shipping `BoxLogShipper` infrastructure owner; production contract exercises restart attempt/runtime/cause/exit status and exact event | verified-owner |
| 31 | `reportSupervisorRestart` | delegated to shipping `BoxLogShipper` infrastructure owner; production contract exercises restart attempt/runtime/cause/exit status and exact event | verified-owner |
| 32 | `reportBoxBootStageConfirmed` | `HostStructuredLogTelemetry::report_box_boot_stage_confirmed` uses `ship_confirmed_projection` | verified-owner |
| 33 | `reportTurnInterrupt` | delegates to `turn_telemetry_mappers.rs`; that architecture row is non-final | **delegated-nonfinal** |
| 34 | `reportTurnAwait` | delegates to `turn_telemetry_mappers.rs`; row is non-final | **delegated-nonfinal** |
| 35 | `reportTurnRetry` | delegates to `turn_telemetry_mappers.rs`; row is non-final; frozen also mutates active-turn retry state | **delegated-nonfinal** |
| 36 | `reportUserMessageReceived` | delegates to `turn_telemetry_mappers.rs`; row is non-final | **delegated-nonfinal** |
| 37 | `reportClosingSendNudge` | delegates to `turn_telemetry_mappers.rs`; manifest explicitly says shipping closing-send producer remains missing | **delegated-nonfinal** |
| 38 | `reportSubagentRevival` | production completion revival runtime calls `HostStructuredLogTelemetry::report_subagent_revival`; typed facade owns the revival mapper and canonical transport | verified-owner |
| 39 | `reportShellRevival` | `ProductionCompletionRevivalRuntime::report_revival` branches on `RevivalReport.kind == "shell"` and routes to `HostStructuredLogTelemetry::report_shell_revival`; the typed mapper fixes `sand.shell.revival` and preserves level/fields. Exact-HEAD `3a930215…` CI disproved the prior contract because it read the event from `payload.event` instead of the persisted record event field. Commit `56afc178…` corrects that contract, but this row remains non-final until a new exact-HEAD rust-host run passes. | verified-owner |
| 40 | `reportTtft` | delegates to `turn_telemetry_mappers.rs`; row is non-final and manifest still requires complete dispatch-clock/trace/span provenance | **delegated-nonfinal** |
| 41 | `reportSendDispatch` | `queue_telemetry_mappers.rs` mapper exists | producer-evidence-required |
| 42 | `reportQueueAccepted` | shipping run-queue accepted callback calls the typed Host structured-log facade; JSONL contract covers frozen lane/depth metadata | verified-owner |
| 43 | `reportQueueDequeued` | shipping run-queue dequeue callback calls the typed Host structured-log facade; JSONL contract covers timing/depth projection | verified-owner |
| 44 | `reportQueueWatchdog` | shipping queue watchdog callback calls the typed Host structured-log facade after interrupt handling; JSONL contract covers warning/timing projection | verified-owner |
| 45 | `reportAckObligation` | `queue_telemetry_mappers.rs` mapper exists | producer-evidence-required |
| 46 | `reportPendingWake` | shipping PendingWakeRearm runtime still publishes the Gateway pending-wake event and now additionally routes the same report through `HostStructuredLogTelemetry::report_pending_wake`; typed mapper preserves kind/work/age/reason/quiet-origin semantics and JSONL facade contract verifies rounding/event ownership | verified-owner |
| 47 | `reportTurnUsage` | delegates to `turn_telemetry_mappers.rs`; row is non-final and provider token-usage propagation remains open | **delegated-nonfinal** |
| 48 | `reportTurnEmptyDelivery` | shipping provider terminal settlement calls `HostStructuredLogTelemetry::report_turn_empty_delivery`; typed mapper remains under canonical owner and JSONL contract covers empty-delivery fields | verified-owner |
| 49 | `reportJournalOutcome` | `journal_outcome_telemetry.rs` mapper exists | producer-evidence-required |
| 50 | `reportComputerUseUsage` | shipping generated-subagent settlement now consumes `settled.computer_use_usage`, maps it with existing `computer_use_usage_telemetry`, and reports through the unique `worker_telemetry_logs`; exact-HEAD CI still required | verified-owner |
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
| 61 | `reportMcpAuthCleanup` | shipping `start_mcp` owns legacy-auth cleanup outcome/count and now calls typed `report_mcp_auth_cleanup`; direct JSONL contract verifies error=>warn and removed-count shaping | verified-owner |
| 62 | `reportMcpDiscoveryFailed` | direct frozen event; production MCP discovery producer path must be pinned | producer-evidence-required |
| 63 | `reportConnectorAuth` | frozen connector-auth mapper + host boundary; production producer path must be pinned | producer-evidence-required |
| 64 | `reportLocalToolPermissionStrandedRetirement` | shipping transcript bind already receives the real stranded-retirement callback; it now calls typed `report_local_tool_permission_stranded_retirement`, which owns the frozen `sand.client_resource` six-field failure projection | verified-owner |
| 65 | `reportSkillPublishEdgeFailed` | production `SandSkillPublishService` already exposes its edge-failure reporter; `start_mcp` now binds it to typed Host structured-log `report_skill_publish_edge_failed`; producer tests plus JSONL facade contract cover stage/error shaping | verified-owner |
| 66 | `reportPluginSkillsSync` | production `SandPluginSkillsService` already emits real sync outcome/changed/count/error/duration reports; `start_mcp` now installs that reporter on the unique Host logs and calls typed `report_plugin_skills_sync`; service tests plus JSONL facade contract cover success/failure shape | verified-owner |
| 67 | `reportTeachRecordingCapStopFailed` | shipping TeachRecordingServiceDeps captures the unique Host logs and calls `report_teach_recording_cap_stop_failed`; dedicated JSONL facade contract verifies warn/event/error-class semantics | verified-owner |
| 68 | `reportTeachRecordingStartFailed` | shipping TeachRecordingServiceDeps captures the unique Host logs and calls `report_teach_recording_start_failed`; dedicated JSONL facade contract verifies warn/event/window/entry-point semantics | verified-owner |
| 69 | `reportBoxCopyIn` | direct frozen event; production producer path must be pinned | producer-evidence-required |
| 70 | `reportBoxRecreateDecided` | direct frozen event; production producer path must be pinned | producer-evidence-required |
| 71 | `reportInferenceCredentialRenewal` | `HostTelemetryService` owns auth renewal subscription and calls `report_inference_credential_renewal` | verified-owner |
| 72 | `reportGatewayCommandError` | direct frozen event; production Gateway producer path must be pinned | producer-evidence-required |
| 73 | `reportGatewayCommandTiming` | direct frozen event; production Gateway producer path must be pinned | producer-evidence-required |
| 74 | `reportAutomationLifecycle` | generic TelemetryService mapping exists; producer/field parity needs focused proof | producer-evidence-required |
| 75 | `reportHostLog` | Host console forwarder calls the unique `HostStructuredLogTelemetry::report_host_log` owner and truncates to frozen max length | verified-owner |
| 76 | `reportBoxLogBatch` | BoxLogShipper batches records into `HostStructuredLogTelemetry::report_box_log_record` | verified-owner |
| 77 | `reportBoxLogShip` | BoxLogShipper reports via `HostStructuredLogTelemetry::report_box_log_ship` | verified-owner |
| 78 | `reportDesktopHealth` | `HostTelemetryService` owns the production `DesktopHealthForwarder`; forwarder now calls typed `report_desktop_health`, preserving computed frozen level/metadata and fixed `sand.box.desktop_health`; existing JSONL/heartbeat contracts exercise revision suppression and persisted payload | verified-owner |
| 79 | `reportAgentOpen` | direct frozen event; production producer path must be pinned | producer-evidence-required |
| 80 | `reportHostCrash` | Host crash marker owner exists, but ordinary fire-and-forget crash projection producer must be distinguished from confirmed exit forwarding | producer-evidence-required |
| 81 | `reportHostProcessExitConfirmed` | crash-marker forwarding uses confirmed structured-log shipping under Host ownership | verified-owner |
| 82 | `reportInvariantViolation` | direct frozen event; producer path must be pinned | producer-evidence-required |
| 83 | `reportHostUpgrade` | production HostUpgrade dependency now calls typed `HostStructuredLogTelemetry::report_host_upgrade`; facade owns frozen failed=>warn/info event semantics and dedicated JSONL contract verifies ordinary shipping | verified-owner |
| 84 | `reportHostUpgradeConfirmed` | production HostUpgrade dependency calls typed `report_host_upgrade_confirmed`, which retains the same frozen event/level semantics but delegates to transport-owned `ship_confirmed_projection`; Host Upgrade service contract verifies the confirmed settlement path | verified-owner |
| 85 | `reportBoxHelp` | shipping box handoff callback captures the unique Host logs and calls typed `report_box_help`; existing Host telemetry JSONL contract verifies frozen event/conversation/snapshot/reason shaping | verified-owner |
| 86 | `reportBotBlock` | generic mapping does not by itself prove frozen two-event summary/detail semantics | producer-evidence-required |
| 87 | `emitTurnEvent` | returned Host turn handle emits start/outcome/outcome_detail through the same `HostStructuredLogTelemetry::report_projection` owner | verified-owner |
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

### Dedicated disk-pressure facade repair

Frozen `reportBoxDiskPressure` is not equivalent to generic Host diagnostics: it uses the disk-pressure mapper and then explicitly enqueues event `sand.box.disk_pressure`. The shipping ForeverBox already emits the complete disk fields into the HostDiagnostic domain, so the single global Host reporter adapter now detects only `kind=disk_pressure`, projects those fields to `DiskPressureReport`, and calls the typed facade. The facade restores the frozen fixed event name rather than relying on the mapper's intentionally event-less projection. Focused adapter and JSONL contracts cover field preservation, hard-pressure `error` level, fixed event ownership, and one-decimal `used_percent`.

### Existing direct facade evidence and Host Upgrade facade ownership

The facade audit distinguished API presence from real shipping calls. Teach Recording cap-stop/start failures are wired by production `TeachRecordingServiceDeps`; Box Help is wired by the production handoff callback. A focused JSONL contract now exercises both Teach failure helpers, Box Help, and ordinary Host Upgrade semantics. Host Upgrade production no longer owns a private duplicate projection helper: both ordinary and confirmed reporting are methods on `HostStructuredLogTelemetry`; confirmed delivery still uses transport-owned confirmed shipping and remains separately covered by the Host Upgrade service settlement contract. `reportHostLifecycle` was deliberately not advanced because `create_host_lifecycle_progress` is currently not instantiated by shipping composition.


### Host event-bus failure ownership repair

Frozen `SandHostEventBus` reports listener failures without a topic and capability-subscriber failures with a topic. The shipping Rust bus previously defaulted its failure reporter to a no-op even though the mapper existed. Production composition now pins exactly one failure reporter alongside the other structured-log domain reporters; `SandHostEventBus::default()` forwards real listener/subscriber failures into that reporter, which projects optional-topic semantics and calls the typed Host structured-log facade. Shutdown unpins the callback before telemetry disposal. The event-bus contract triggers both failure modes through the real default bus, while the projection contract verifies frozen field shaping.


### Box infrastructure delegation and DesktopHealth facade ownership

Frozen boot-stage/failure, egress-tunnel, host-boot-fetch, exec-daemon-restart and supervisor-restart methods all immediately delegate to the same infrastructure-event mapper. The Rust Host preserves that boundary through the shipping `BoxLogShipper`: the production box telemetry stream is strictly parsed into `BoxInfrastructureEvent`, then the unique Host structured-log owner maps and ships it. The production shipper contract now feeds every frozen infrastructure family through the real offset/delivery path and verifies the exact event names, so these rows are explicit architectural delegation rather than mapper-only evidence. DesktopHealth already had a real `HostTelemetryService` worker; it now calls a typed `report_desktop_health` facade instead of bypassing the facade via raw projection.


### MCP / Plugin Skills / local-permission direct facade repair

The shipping domains already owned the required business signals but did not consistently route them through structured-log. `start_mcp` now reports legacy-auth cleanup outcome/count, installs the existing Plugin Skills sync reporter on the unique Host logs, and binds the existing Skill Publish edge reporter to the typed structured-log facade. The transcript/local-permission stranded callback no longer constructs an ad-hoc projection; it calls the frozen client-resource facade, restoring the fixed domain/operation/state/failure/boundary/retry-owner metadata. Focused JSONL contracts verify all four direct-event shapes while existing domain tests continue to exercise the producer callbacks.


## Exact-HEAD producer re-audit after shell-revival CI counterexample

The exact-HEAD `3a9302151c72f6c07d604dc0c35bd1d6c1a5473a` Rust runtime run `36726787963` failed in `host_telemetry_service_contract` because the test read `sand.shell.revival` from `payload.event`. The frozen Grok facade sends the mapper event as the transport message/event argument; Fabushi persists that value in `PersistedHostTelemetryRecord.event`, while `payload` contains `level` and `metadata`. The shipping producer, typed mapper and unique Host owner were already correct; the contract was wrong. The corrected contract now asserts the persisted record event and retains independent metadata/level assertions. This is not accepted as final until the new exact HEAD passes rust-host CI.

The 29 `producer-evidence-required` rows were re-audited against the clean exact HEAD rather than by mapper/facade name presence. The evidence threshold remains: real shipping producer/call site → unique Host structured-log owner → frozen event/field/level/multi-event/confirmed/failure/dispose semantics → focused contract.

| Frozen facade | Shipping producer finding | Owner / semantic finding | Result |
| --- | --- | --- | --- |
| `reportAutomationRun` | no shipping call site found; only AnalyticsService/ports registration and generic mapping | generic Host report mapping is not producer evidence | keep gap |
| `reportHostStartup` | no shipping call site found on clean exact HEAD | typed method/mapper exists, but no production producer | keep gap |
| `reportHostLifecycle` | `HostTelemetryExtension::create_host_lifecycle_progress` closure calls typed Host lifecycle reporter; lifecycle object emits completed/failed/stuck and disposes/rearms watchdog | real producer exists; existing lifecycle contracts prove phase/failure/stuck/dispose, but one focused shipping-owner JSONL contract is still missing | keep gap pending focused owner contract |
| `reportDaemonPing` | no shipping call site found | generic registration only | keep gap |
| `reportBoxImageCheck` | no shipping call site found | mapper exists only | keep gap |
| `reportSendDispatch` | no shipping call site found | queue mapper + generic registration are insufficient | keep gap |
| `reportAckObligation` | no shipping call site found | queue mapper + generic registration are insufficient | keep gap |
| `reportJournalOutcome` | no shipping call site found | journal mapper + generic registration are insufficient | keep gap |
| `reportToolCallError` | no shipping call site found | frozen error event and capped duration shaping are not proved by generic raw mapping | keep gap |
| `reportToolCallStalled` | no shipping call site found | frozen warn + elapsed cap not proved by generic raw mapping | keep gap |
| `reportToolCallStarted` | no shipping call site found | frozen field shaping not proved by generic raw mapping | keep gap |
| `reportAgentError` | no shipping call site found | frozen summary + optional detail dual-event, Sand error tags and truncation are not proved by generic raw mapping | keep gap |
| `reportAutoReviewDisplayRecheckFailed` | no shipping call site found | no typed Host producer; fixed `surface=computer` remains unevidenced | keep gap |
| `reportAutoReviewExpireSweepFailed` | no shipping call site found | generic registration only | keep gap |
| `reportBoxStoreSyncCycle` | no shipping call site found | no typed Host producer found | keep gap |
| `reportBoxStoreDbCapture` | no shipping call site found | no typed Host producer found | keep gap |
| `reportBoxStoreManifestConflict` | no shipping call site found | no typed Host producer found | keep gap |
| `reportChromeSessionStage` | no shipping call site found | no typed Host producer found | keep gap |
| `reportMcpDiscoveryFailed` | no shipping call site found | no Host producer proving warn/rounded elapsed/served-stale fields | keep gap |
| `reportConnectorAuth` | Electron-main has a separate desktop telemetry producer, but no Host shipping producer through the frozen Host facade was found | Electron-main telemetry is not evidence for the unique Host structured-log owner | keep gap |
| `reportBoxCopyIn` | no shipping call site found | no typed Host producer found | keep gap |
| `reportBoxRecreateDecided` | no shipping call site found | no typed Host producer found | keep gap |
| `reportGatewayCommandError` | no shipping call site found | no typed Host producer found | keep gap |
| `reportGatewayCommandTiming` | no shipping call site found | no typed Host producer found | keep gap |
| `reportAutomationLifecycle` | no shipping call site found | generic registration does not prove optional scheduling flags/age/run-count shaping | keep gap |
| `reportAgentOpen` | no shipping call site found | no typed Host producer found | keep gap |
| `reportHostCrash` | no ordinary fire-and-forget frozen producer found; confirmed process-exit forwarding is a distinct path | confirmed exit semantics cannot substitute for ordinary `reportHostCrash(kind)` | keep gap |
| `reportInvariantViolation` | no shipping call site found | no typed Host producer found | keep gap |
| `reportBotBlock` | no shipping call site found | generic mapping does not prove frozen two-event summary/detail behavior | keep gap |

Therefore none of the 29 rows is promoted by this re-audit. `reportHostLifecycle` is the only row in this set with a clearly evidenced production producer at this exact HEAD, but it still lacks the focused single-owner shipping contract required by this ledger. The parent structured-log architecture row must remain `existing-needs-parity`, and `turn-telemetry-mappers.ts` must not start yet.


## Exact-HEAD acceptance at `33ec024399340542fb5fa9d25f1aeb14c8eff9f8`

Exact-HEAD GitHub Actions closed the prior awaiting-CI bucket:

- Desktop Chat Parity CI run `36732506489` completed successfully, including Focused Electron chat E2E and renderer typecheck/build.
- Rust desktop runtime run `36732506428` bound the exact tested HEAD. Its `rust-host` job `109945560052` completed successfully: independent Mahayana Coordinator, independent box-exec daemon, shipping Host box-exec supervisor, independent Mahayana Host Runner, prompt attachment projection, human Computer takeover, and ConversationActor/CapabilityBroker ownership all passed.
- The Host Runner test log explicitly records `frozen_facade_methods_route_mapper_semantics_through_single_host_owner ... ok` and `shipping_host_lifecycle_progress_routes_through_single_structured_log_owner ... ok`; `host_telemetry_service_contract` finished 9/9 and `structured_log_lifecycle_facade_contract` passed.
- Linux and Windows pressure-profiler jobs passed. The renderer job fails only at the final architecture gate because the manifest still contains 59 `existing-needs-parity` rows; all preceding build, boundary, renderer and inventory checks in that job passed.
- Therefore every facade row previously classified `implemented-awaiting-exact-head-ci` is now reclassified `verified-owner`. This does **not** advance the 27 `producer-evidence-required` rows or the 7 `delegated-nonfinal` rows.
- The historical `3a930215...` producer re-audit below remains provenance for the earlier counterexample; where it conflicts with the current main table, the current main table and this exact-HEAD acceptance section are authoritative.

Current facade totals after this acceptance: **56 verified-owner / 27 producer-evidence-required / 7 delegated-nonfinal = 90**. The parent `structured-log-telemetry.ts` architecture row must remain `existing-needs-parity`; `turn-telemetry-mappers.ts` must not start until all 27 direct producer gaps close and only the 7 delegated rows remain.
