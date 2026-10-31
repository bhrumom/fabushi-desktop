# OCRM Not-Started Responsibility Frontier — 2026-10-31

**Subject**: Fabushi Desktop working tree at the current local branch; upstream
authority remains the discovery-only candidate in `upstream.lock.json`.  
**Purpose**: Record why the remaining `not-started` ledger rows are not safely
implementable from the current production graph, and preserve the next direct
instrument required for each row.

## Current frontier

| Responsibility | Current evidence | Why implementation cannot start from this tree |
| --- | --- | --- |
| `OCRM-RESP-AGENT-ACP-COMMANDS-001` | No production `AvailableCommand`/`get_available_commands` producer; current ACP manager is test-consumed only; historical source path moved to the current chat-command registry. | Requires exact upstream command-registry mapping and a real ACP producer/consumer before adding a second command owner. Blocker: `BL-PLUGIN-SDK-ACP-WIRING-001`. |
| `OCRM-RESP-AGENT-ACP-POLICY-001` | No production ACP dispatch-policy producer or consumer; current production runtime owns a separate Host turn lease/watchdog path. | Requires an exact ACP admission/settlement owner and a production caller; contract fixtures are insufficient. |
| `OCRM-RESP-AGENT-CONTEXT-COMPACTION-001` | `ContextWindowTracker` contract logic has no production `check_compaction` caller; production prompt code reads persisted compaction epochs through `HostRunnerComposition`. | Requires the current compaction producer and prompt/turn consumer; wiring a parallel tracker would create a second owner. |
| `OCRM-RESP-AGENT-LOOP-DETECTION-001` | No production repeated-tool/hash detector or loop-breaker producer; `TurnOwnerToken` is consumed for cancellation/generation only. | Requires a real tool-call observation stream and cancellation consumer; static token helpers do not prove loop detection. |
| `OCRM-RESP-AGENT-LOOP-RUNNER-001` | `ProductionTurnRunShellAdapter` owns active-run lifecycle; `DeferredTurnQueue` exists only in the contract fixture. | Requires a production supersede queue owner and persistence/settlement consumer; do not copy the fixture queue into runtime without its producer. |
| `OCRM-RESP-AGENT-MODEL-PRICING-001` | No production pricing catalog or provider-fallback producer/consumer; production paths aggregate usage and retry transport failures only. | Requires provider pricing authority, currency/model schema, and fallback admission policy. |
| `OCRM-RESP-SESSION-AUTH-PROFILE-LIFECYCLE-001` | `session_profile_files.rs` serves agent profile/avatar data; no auth-profile preservation/removal producer or lifecycle consumer is identified. | Requires a real session cleanup/auth owner and credential-preserving contract; profile files cannot be repurposed as auth storage. |
| `OCRM-RESP-SESSION-MODEL-OVERRIDES-001` | No production session model-override producer; `apply_session_model_selection` has no non-test caller. | Requires a real session mutation route and runner/provider consumer; the JSON fixture is not admission evidence. |
| `OCRM-RESP-SESSION-WORK-ADMISSION-001` | `ProductionSessionWorkers` provides general session storage but no work-admission/archive/review lifecycle producer or consumer. | Requires the owning workboard/task lifecycle and terminal review consumer; adding state fields alone would not close the path. |
| `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001` | `bundle_manifest.rs` defines parsing helpers, but no production bundle/archive producer or installer/discovery consumer calls them; Host extension registration currently publishes capability metadata only. | Requires a real plugin bundle ingress, archive custody, install transaction, and activation consumer. Blocker: `BL-PLUGINS-MANIFEST-WIRING-001`. |

## Evidence boundary

These are implementation gaps, not acceptance approvals. No row is promoted by the
presence of a helper, contract test, capability metadata, or historical upstream
source path. The next direct instruments are a fresh exact-subject source/dependency
closure and, for each applicable row, a production call graph showing producer,
owner, state/storage boundary, and consumer. Until then the ledger states remain
`not-started/not-wired` and all global completion flags remain unchanged.

## References

- `projects/openclaw-rust/parity-ledger.json`
- `projects/openclaw-rust/upstream.lock.json`
- `docs/specs/openclaw-rust-equivalence-migration.md`
- `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md`
