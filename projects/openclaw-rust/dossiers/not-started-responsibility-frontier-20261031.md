# OCRM Not-Started Responsibility Frontier — 2026-10-31

**Subject**: Fabushi Desktop working tree at the current local branch; upstream
authority remains the discovery-only candidate in `upstream.lock.json`.  
**Purpose**: Record why the remaining `not-started` ledger rows are not safely
implementable from the current production graph, and preserve the next direct
instrument required for each row. The tool-loop detector was removed from this
frontier after its production producer/consumer chain was wired; its frozen
verification remains pending in the parity ledger.

## Current frontier

| Responsibility | Current evidence | Why implementation cannot start from this tree |
| --- | --- | --- |
| `OCRM-RESP-AGENT-ACP-COMMANDS-001` | No production `AvailableCommand`/`get_available_commands` producer; current ACP manager is test-consumed only; historical source path moved to the current chat-command registry. | Requires exact upstream command-registry mapping and a real ACP producer/consumer before adding a second command owner. Blocker: `BL-PLUGIN-SDK-ACP-WIRING-001`. |
| `OCRM-RESP-AGENT-ACP-POLICY-001` | No production ACP dispatch-policy producer or consumer; current production runtime owns a separate Host turn lease/watchdog path. | Requires an exact ACP admission/settlement owner and a production caller; contract fixtures are insufficient. |
| `OCRM-RESP-AGENT-CONTEXT-COMPACTION-001` | `ContextWindowTracker` contract logic has no production `check_compaction` caller; production prompt code reads persisted compaction epochs through `HostRunnerComposition`. | Requires the current compaction producer and prompt/turn consumer; wiring a parallel tracker would create a second owner. |
| `OCRM-RESP-AGENT-LOOP-RUNNER-001` | `ProductionTurnRunShellAdapter` owns active-run lifecycle; `DeferredTurnQueue` exists only in the contract fixture. | Requires a production supersede queue owner and persistence/settlement consumer; do not copy the fixture queue into runtime without its producer. |
| `OCRM-RESP-AGENT-MODEL-PRICING-001` | No production pricing catalog or provider-fallback producer/consumer; production paths aggregate usage and retry transport failures only. | Requires provider pricing authority, currency/model schema, and fallback admission policy. |
| `OCRM-RESP-SESSION-AUTH-PROFILE-LIFECYCLE-001` | `session_profile_files.rs` serves agent profile/avatar data; no auth-profile preservation/removal producer or lifecycle consumer is identified. | Requires a real session cleanup/auth owner and credential-preserving contract; profile files cannot be repurposed as auth storage. |
| `OCRM-RESP-SESSION-MODEL-OVERRIDES-001` | Session owner now persists validated `sessionModelSelection` through `SandAgentSessionStore::apply_session_model_selection`; no production runner/provider consumer projects it into `SandSessionOptions`. | Requires a real session mutation route at the Host boundary and an inference consumer; the JSON fixture and KV owner alone are not runtime acceptance evidence. |
| `OCRM-RESP-SESSION-WORK-ADMISSION-001` | `ProductionSessionWorkers` provides general session storage but no work-admission/archive/review lifecycle producer or consumer. | Requires the owning workboard/task lifecycle and terminal review consumer; adding state fields alone would not close the path. |
| `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001` | `bundle_manifest.rs` defines parsing helpers, but no production bundle/archive producer or installer/discovery consumer calls them; Host extension registration currently publishes capability metadata only. | Requires a real plugin bundle ingress, archive custody, install transaction, and activation consumer. Blocker: `BL-PLUGINS-MANIFEST-WIRING-001`. |

## 2026-10-31 plugin-owner boundary recheck

The read-only source search was repeated against the current Fabushi checkout.
`source/host/src/extensions/plugins/bundle_manifest.rs` exposes the format
paths, ClawHub spec parser, archive-size limits, and SHA-256 verifier, and
`extensions/plugins/mod.rs` only exports that module. The only call sites for
`parse_clawhub_plugin_spec`, `verify_archive_limits`, and
`verify_sha256_integrity` are the `openclaw_plugins_contract.rs` tests; no
production Rust caller supplies bundle bytes or consumes a verified bundle.

Two nearby paths are deliberately not promoted to consumers:

- `extensions/mcp/installable_package/ledger.rs` owns the separately scoped
  mini-app install/activation ledger and `VerifiedPackage` transaction. It is
  not a plugin bundle ingress and must not be reused as a second interpretation
  of plugin manifests.
- `extensions/mcp/skill_publish.rs::read_manifest_name` reads only the
  `plugin.json` name from an already-installed skill directory for duplicate
  protection. It does not discover archives, enforce bundle limits, verify
  bundle digests, or activate plugin code.

This recheck strengthens the existing disposition: the bundle-manifest helper
is implementation-shaped test support, not production wiring. The next direct
instrument remains a real marketplace/ClawHub or local-bundle producer through
the existing plugin install owner, followed by one state/receipt boundary and
the activation consumer. No ledger status, completion flag, or acceptance
claim changes from this evidence.

## 2026-10-31 session model-override boundary recheck

The session override path has a real persistence owner but still lacks the
required consumer edge. `agent_session.rs` reads and writes the
`sessionModelSelection` KV through the per-agent SQLite owner and applies the
validated transition at `apply_session_model_selection`; the only production
references to `read_session_model_entry` are its definition and the store
method that exposes the read. No turn/request composition path loads that
record and builds `SandSessionOptions` from it.

The inference owner does consume an explicitly supplied `SandSessionOptions`
through `ProductionAgentInferenceOwner::resolve_requested_model` and
`cursor_auth_for_session`, but its persisted inputs are agent defaults,
computer/browser-use settings, environment and experiment overrides. The
session KV is not among those inputs. Therefore the current chain ends at
SQLite state and does not reach the inference consumer required by
`OCRM-RESP-SESSION-MODEL-OVERRIDES-001`; the row remains
`implementing/not-wired/not-run/pending`. The next direct instrument is the
existing Host turn/session composition boundary that can project the
validated per-session state into one `SandSessionOptions` instance, followed
by a negative test proving parent-agent defaults remain unchanged.

## 2026-10-31 session work-admission boundary recheck

The current source also does not contain a production equivalent of the
upstream `assertAgentSessionWriteAdmission` contract. The session runtime has
active-session switching, shutdown draining, deletion and deferred activation
paths, but none checks an archived-session state before admitting new agent
work. The `archive`/`unarchive` branches found in `main.rs` are cloud-agent
lifecycle review actions and do not consume `ProductionSessionWorkers` as a
session work-admission owner. No production caller for the contract's
background-session result or archive/restore review transition was found.

These adjacent lifecycle paths therefore cannot close
`OCRM-RESP-SESSION-WORK-ADMISSION-001`; its status remains
`not-started/not-wired/not-run/pending`. The next direct instrument is a
session-owned admission API that fences writes against archive/delete
generations and returns a durable terminal/review outcome to the existing
turn consumer. No second session store or cloud-agent archive shim is
authorized by this finding.

## Evidence boundary

These are implementation gaps, not acceptance approvals. No row is promoted by the
presence of a helper, contract test, capability metadata, or historical upstream
source path. The next direct instruments are a fresh exact-subject source/dependency
closure and, for each applicable row, a production call graph showing producer,
owner, state/storage boundary, and consumer. The rows listed above remain
`not-started/not-wired` and all global completion
flags remain unchanged. `OCRM-RESP-AGENT-LOOP-DETECTION-001` is intentionally
not listed: its current ledger state is `implemented/wired/not-run/pending`.

## References

- `projects/openclaw-rust/parity-ledger.json`
- `projects/openclaw-rust/upstream.lock.json`
- `docs/specs/openclaw-rust-equivalence-migration.md`
- `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md`
