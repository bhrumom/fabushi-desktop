# Bounded Rust MCP admission assessment

## Result

**The documented migration gap is still present at the exact assessed source. A universal Rust disabled-tool check is not code-ready from the current inputs.** The real routed Rust bridge has neither the stable legacy server-row identity nor an account/current-registration binding needed to interpret the persisted map. The Host settings copy is synchronized from the desktop; official connectors use their existing dedicated vault state. Treating a provider label or a resynced map as a grant would invent authority and can change supported behavior.

This is a bounded read-only source assessment, not an implementation or acceptance result. It establishes the missing contract needed for a subsequent **legacy account MCP tool disable → durable Rust Settings commit → execution admission** batch. It does not recommend creating another permission store, manager or runtime. The ongoing official lifecycle repair is independent and need not wait for this broader authority decision.

## Identity and method

- Task `647e0932-bea4-4113-9a61-65ae3db58955`, round 1; assigned assessment worker `/root/mcp_rust_admission_assessment`.
- Exact target base `8e5cee76ebbe6e748dc6a8dc87d73142bf04109b`, tree `33dad107f907edb77ec90b085a0bfee9e0ad0622`.
- Fixed upstream U `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`; no floating upstream or new upstream file acquisition/read claim.
- The worker was launched with `fork_turns=all` and no model/effort override, inheriting the parent configuration. Parent classified this phase as xhigh because account identity, durable Settings/vault authority, admission and entered effects are coupled. That classification was not treated as an independently applied host setting.
- All product reads used `git show` / `git grep` / `git ls-tree` at the exact commit. Mutable official-lifecycle edits were not review authority. No build, test, lint, format, generator, schema check, candidate execution, htch execution or GitHub mutation was performed.
- Read governing AGENTS spec-first/read-only and source-context/verification rules, SOT current identity/scope sections, main Spec §17, M16–M18, current P0 MCP lineage, and the applicable MD contract. This is not a new full read of unrelated historical task material or the full root policy document. No nested AGENTS applies to these paths.

## Actual producer, owners and consumers

### Routed execution is real, but does not carry permission identity

The shipping `CoordinatorRoutedToolBridge` is constructed at App `main.rs:10707–10825`. Its `mcp_review` closure invokes the existing AutoReview gate/classifier and possible user approval. That is a separate existing authorization responsibility, not a lookup of the disabled-tool setting. MCP mode `Off` explicitly returns no review refusal.

The bridge's `call_tool` (`main.rs:8260–8328`) runs that review, records pending Transcript activity, then invokes the existing `CoordinatorToolRelay::request_tool_execution`. Its request contains `providerIdentifier`, public `name`, raw `toolName`, arguments, tool-call ID and Agent ID. `RoutedToolDefinition` (`provider_session.rs:187–194`) contains only those tool identity/display/schema fields; it has no server-row ID, account scope, account-slot binding or registration generation. Both actual App catalog decoders preserve that contract (`main.rs:8331–8390`).

The Rust relay publishes a `runner-tool-request` event and owns its waiter/known-versus-unknown outcome (`coordinator_tool_relay.rs:83–205`). The production Coordinator receives that channel, uses its owned worker to call `control_command`, and resolves the original Host request (`source/node-agent-coordinator/src/main.rs:1153–1198`). Its parser accepts only list/execute methods and an object argument but does not interpret MCP account/server/tool permissions (`runner_tool_relay.rs`, complete 104-line file).

The Electron production adapter awaits the current runtime manager and forwards the request to the actual facade (`adapters/mcp-oauth.ts:350–360`). The facade selects official handling by the existing catalog ID, otherwise legacy discovery (`desktop-mcp-manager.ts:256–272`). Thus inserting a Rust predicate solely in the first App bridge would be earlier than legacy provider/account preparation and would lack the actual identity used by the existing policy.

### Legacy policy uses a resolved row, not the provider string as a key

`tools-discovery.ts:93–106` resolves a provider identifier against `lastAccountDisplayConfig().servers` using `displayRowOwnsIdentifier`. That helper accepts a row's own identifier or one of its account-slot identifiers (`mcp-definition-source.ts:5`). It is not an encoded numeric-ID parser or a prefix convention.

The actual disabled map is keyed by `displayServer.id`; the policy tests the raw `args.toolName` (`tools-discovery.ts:401–415`). After awaited HTTP provider resolution, that predicate is applied immediately before the existing backend execution port (`418–435`). Box execution waits for stdio resolution and config push, then applies the same predicate before the Box execution port (`437–451`). The earlier MD repair is present; no duplicate repair is needed.

Public tool name and raw tool name intentionally differ: the HTTP dispatch forwards `args.name` as the backend tool name while the disabled predicate uses `args.toolName`. A new Rust check must preserve that established distinction, not normalize names heuristically. Unknown/unmapped providers currently flow to the existing unavailable result, and built-ins do not become account row IDs by inference.

The manager publishes account display/runtime state after its existing generation/disposal checks. It calls `settingsStore.scopeToAccount(display.cacheScope)` and its existing scope-applied callback before adopting the resolved definitions (`mcp-manager.ts:125–177`). This is the producer of the relevant desktop identity association. No corresponding authoritative association is passed to the Rust routed definition/execute request.

### Settings exposes the record, but the full write/identity cutover has not happened

Rust `SettingsService` exposes getters/setters for `mcpDisabledToolsByServerId` and account scope (`settings_service.rs:961–1035`). Its map keys are positive decimal strings (`1924–1928`), matching the legacy ID validator. The public getter reads through `load()`, which intentionally substitutes empty legacy defaults on unreadable/invalid Settings (`514–519`); a permission admission API cannot blindly treat this getter's empty map as positive authority. Checked snapshot and writer paths already exist and should be reused for any future authoritative decision.

A bounded Rust source search found the named `get_mcp_disabled_tools_by_server_id` accessor used only in its existing Settings test, not in production execution. This is corroborating search evidence, not a claim that name searching alone proves the entire Rust security model absent; the actual production chain above was read directly.

The current desktop `SandMcpInstructionsAndToggles` validates/looks up the server, computes the toggle, then writes the whole map through `SandSettingsStore` (complete `mcp-instructions-and-toggles.ts`, three dense lines). The store owns file-backed reads/persistence and the MCP fields (`sand-settings-store.ts:100–146`). Removal and account-scope transitions also mutate these fields. The real desktop IPC toggles first, then pushes the map to Host with bounded synchronization retries (`mcp-desktop.ts:12`). The Host `getHostSettings`/`setHostSettings` command uses the existing checked Rust Settings service (`main.rs:1648–1721`).

The actual reconnect resync (`coordinator-resync.ts`, complete file) compares scope, merges instructions, and unions disabled lists. It then writes the Host and mirrors the result to desktop. Therefore the two surfaces already have synchronization and reconciliation semantics. A new check cannot silently declare whichever copy it sees first to be the sole authoritative permission state. Account scope is a field supplied through this resync/update path; a generic Settings response is not by itself proof that the descriptor belongs to the current authenticated MCP manager.

`HostAuthExtension` exposes token/service/renewal access (`auth/extension.rs:1–90`), but these existing API definitions do not establish a stable MCP server/account-generation contract at the routed bridge. I did not read the full HostAuth internals or all account-transfer paths, so I do not claim there is no reusable current-account primitive elsewhere. That finite dependency must be traced before designing a replacement.

### Official connectors have a different existing record

`OfficialMcpService` stores `disabledTools`, credentials, connection and revision together in its existing account-scoped install state. It filters discovery and applies the disable check in `execute` before its own revision/scope fences (`official-mcp-service.ts:269–328`). `toggle` mutates that same existing state (`294–304`). The facade routes official toggles directly there, not to the legacy Settings map (`desktop-mcp-manager.ts:273–276`). The vault is `official-mcp/<scope>.json`, slot `OFFICIAL_MCP_INSTALLS`, through the existing native secrets store (`official-mcp-vault.ts`, complete 26 lines).

Official catalog IDs are consequently not legacy positive-decimal Settings IDs. Adding them to the legacy map, copying credential-bearing vault images to Host Settings, or accepting a caller-provided official disabled list would create an unapproved data/authority change. Official policy migration remains applicable work; it is not made N/A by limiting an earlier batch to legacy account connectors.

## Necessary next contract, before any implementation

The smallest coherent Rust-policy batch should be **legacy account connector tool disable and admission**, covering HTTP and Box variants that already share the same legacy row/map. It can use existing Settings and MCP/Runner owners, but requires these load-bearing decisions first:

1. **Prepared identity:** identify the existing authenticated account/lifetime owner and carry an exact account binding, resolved server-row ID, provider/account-slot identifier and raw tool name from the real current preparation path. A list label, optional new descriptor field or unchecked caller string cannot become an authority token. Preserve cold preparation and account alias behavior.
2. **One state owner:** make the existing Rust Settings mutation/checked read the authoritative legacy disabled-map operation, with an explicit cutover of the real toggle, removal, account clear/scope application and reconnect reconciliation writers. Desktop state may remain a projection, but cannot race an independently computed authoritative decision. Do not introduce another database or permission registry.
3. **Linearization and currentness:** define the point at which an execution is admitted after preparation and current account/registration validation. Distinguish a stale/missing binding or unreadable Settings from a positive allow. Recheck the actual bridge lifetime at dispatch. A disable or account switch before admission prevents a new dispatch; after an already-entered execution, preserve its result or true unknown outcome without replay.
4. **Compatibility boundary:** preserve existing raw/public names, per-account alias membership, return/result-arm classification, built-in/unavailable paths and AutoReview. Official policy stays in its current owner until its own complete migration cutover; this is scope sequencing, not an exemption.

This is **not implementation authorization or a finalized schema proposal**. In particular, I do not recommend immediately adding a `serverId` field and reading the resynced map in `CoordinatorRoutedToolBridge`. The identity producer and writer cutover are necessary dependencies of that code.

Existing paths likely required once those decisions are source-resolved:

- Rust owner/entry: `source/host/src/extensions/settings/settings_service.rs`; `source/host/src/extensions/mcp/mcp_service.rs` if its existing service owns the prepared binding; root-serialized `source/host/app/src/main.rs` and the existing inference tool definition/relay contracts only as required.
- Actual legacy producer/consumer bridges: `source/shared/node/mcp/mcp-manager.ts`, `mcp-instructions-and-toggles.ts`, `tools-discovery.ts`; `source/electron-main/mcp/mcp-desktop.ts`; `source/electron-main/coordinator/coordinator-resync.ts`; the existing adapter/Coordinator method route if a shared contract requires it.
- Verification owners: existing `settings_service_contract.rs`, actual App routed-bridge tests, current whole-file MCP runtime contract and existing Coordinator relay contracts. Root owns shared schema, composition and workflow selection. Do not edit the currently active official four-path batch concurrently.

## Direct behavior oracles for that future batch

Use actual Settings persistence and actual default manager/row preparation plus App/Coordinator dispatch, with only account/remote execution I/O controlled. A helper-only predicate test cannot prove this behavior.

- Cold legacy HTTP and Box preparation resolve the same stable row/slot as current production. A disabled raw tool must produce the existing refusal with zero execution-port calls. A sibling enabled tool succeeds once with unchanged public/raw names and arguments.
- Hold preparation and, separately, AutoReview; commit disable or retire the account; release the hold. Require zero new dispatch, exact persisted denial state and a fresh current-account positive. Include two slots of one row and different accounts with an equal numeric row ID.
- Corrupt/unavailable Settings cannot be interpreted as an empty allow-map. Failures preserve raw bytes and recover through a fresh valid snapshot; no fallback rewrite of the file.
- Commit a real toggle, reopen the Settings owner, reconnect resync and reread through the actual UI/manager consumer. Prove the selected bit, unrelated rows, scope and unknown fields are retained. Explicitly pin enable as well as disable so union reconciliation cannot silently undo enable.
- Enter an execution before disable/reset and hold its remote result. Its actual success, server result, rejection or post-publication unknown disposition remains with the original call and is not retried. Preserve existing relay custody and AutoReview denial controls.
- Official IDs, built-ins and unsupported/unavailable providers retain their existing owners and behavior; the legacy gate must not manufacture row mappings for them.

These controls are authoring requirements, not executed cases. Exact-H Actions, independent review, scoped G01/G02 authority and real provider/account/platform acceptance remain required.

## Read boundaries and exact blobs

Deep-read ranges are cited above; other searches/tree listings only locate candidates. Some combined output was truncated, so unseen text is not credited. Source paths guessed incorrectly (`runner/provider_session.rs`, `production_coordinator.rs`) were not read; the real counterparts were then located and read. No peer implementation/review report was used as source authority.

| Path | Exact blob | Fresh reading |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/mcp-account-listing-retirement.md` | `2f6af6752039faee0418def3c9d3bf59c59002ac` | Applicable MD context/requirements, 200–265; not full unrelated RC history |
| `source/host/app/src/main.rs` | `ca038b1aa9f3a78a1951551e0a80d543142c08dd` | 1648–1721, 8200–8390, 10707–10825; targeted production chain |
| `source/host/src/extensions/inference/provider_session.rs` | `b2513672ca563e72efe0f4b56440b9aaf8f98c60` | 182–215 definition contract |
| `source/host/src/extensions/settings/settings_service.rs` | `7c17f9a08af73365b062e2f212e0b444bfa546f3` | 410–540, 920–1035, 1490–1535, 1915–1995; relevant persistence/normalization |
| `source/host/src/extensions/auth/extension.rs` | `afc6c89fc2d39d17b5cf20ea622cdabe639261fe` | 1–90 API/renewal boundary; deeper service not read |
| `source/host/src/runner/coordinator_tool_relay.rs` | `f0ddac250be056c5464e0208be0c873c8bfa63e1` | 1–205 request/publication/outcome boundary |
| `source/host/src/extensions/mcp/mcp_service.rs` | `e903112e400783e70e9735b41745223a0f29f06c` | 1–260 existing backend/service types; not whole service |
| `source/host/src/extensions/mcp/coordinator_relay.rs` | `4190c0bab004bf6c5191cef489e77b56d81cedb9` | 1–65 and 340–470 backend forwarding; partial first larger output |
| `source/host/src/extensions/mcp/production_box_state.rs` | `ed10c6b76df61a0f3fdf5aa1f60844ace5f172b2` | 1–180 raw Box adapter and existing tests; not executed |
| `source/node-agent-coordinator/src/main.rs` | `7fb056fc384fa705502a51a34cb7d94dc21fd358` | 1118–1200 worker dispatch/settlement |
| `source/node-agent-coordinator/src/runner_tool_relay.rs` | `54593a04ce88b9bc0b7e91d068b99024732eeab8` | Complete file |
| `source/shared/node/mcp/mcp-manager.ts` | `a06ba8949ecdd1a1228350125e376d9f13e7b68e` | 100–190 publication owner; other matches only located |
| `source/shared/node/mcp/mcp-definition-source.ts` | `ac85cb6f9f7f7d75b3cf6d749247ca0c3a43f5c3` | Complete dense file |
| `source/shared/node/mcp/mcp-instructions-and-toggles.ts` | `d0ade41ff69a8866f969aca183befefded04ab4f` | Complete dense file |
| `source/shared/node/mcp/mcp-server-id.ts` | `fe477dbe135a3333e5422b47136310c38556813e` | Complete file |
| `source/shared/node/mcp/tools-discovery.ts` | `6e4b944417ede0d30617bedb889adf71fc7fa284` | 1–220, 350–455; identity, disabled predicate and final preparation/dispatch |
| `source/shared/node/settings/sand-settings-store.ts` | `f45786daa8f7851bf967877e86e04d0d76e6d166` | 78–155 relevant persistence/MCP fields |
| `source/shared/node/cursor-token.ts` | `e64051586d4fba813fa477a4c22ff11c38604660` | 18–51 scope producer definition; not authenticated binding proof |
| `source/electron-main/coordinator/coordinator-resync.ts` | `3863ca3a3f933e611b4f58122ea6cd75274dd76c` | Complete dense file |
| `source/electron-main/adapters/mcp-oauth.ts` | `6653252d5f12ad3e345ee9104dd58c93734594df` | 350–382 actual routed/default manager delegation |
| `source/electron-main/mcp/desktop-mcp-manager.ts` | `17af10771dfaea32dec8626f5a9e3ab4d5be6ad8` | 183–285 official/legacy facade branches |
| `source/electron-main/mcp/mcp-desktop.ts` | `98d57679b13b215fafe02fa61860edb95c660197` | Relevant complete physical lines 8/12, actual toggle/sync paths |
| `source/electron-main/mcp/official-mcp-service.ts` | `3da590f7908870f1c7ac7806326e6ed04093882c` | 232–337 discovery/toggle/execute and scope/revision consumers |
| `source/electron-main/mcp/official-mcp-vault.ts` | `ca1a8beed13059998958030d399278c3c7e9a78d` | Complete file |

Additional bounded context read: MCP extension startup 1–190 and production cleanup/client declarations 1–230. They supplied no missing permission identity. No new source accounting or acceptance state was written.

## Concrete next dependency work

Trace the existing native account binding from `HostAuthExtension::service()` / `auth_service.rs` and the production account-transition/credential transfer into the Host, then reconcile it with the desktop `display.cacheScope` producer and the actual Settings writer/resync paths cited above. Record the exact existing owner and transition revision that a prepared legacy row may be bound to, and decide the one-writer cutover before root freezes a shared contract. If that existing identity cannot be established, leave Rust admission unimplemented rather than grant based on a label/cache. This is a source/contract dependency that can be investigated now; missing real service credentials block later compatibility acceptance, not that investigation.

## Dependency continuation — 2026-11-03 04:04:58 UTC

This separately timestamped continuation preserves the original assessment and its literal read limits. It resolves the specific account/currentness and writer dependencies requested by root. Product authority remains immutable `8e5cee76ebbe6e748dc6a8dc87d73142bf04109b`, tree `33dad107f907edb77ec90b085a0bfee9e0ad0622`; fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`. Root reported newer remote `fdb0f6ac` with disjoint media/plugin work; that report is not substituted for this assessment's exact source authority. No mutable official-worker files or peer reviews were read. The phase is classified xhigh by root; the agent actually retains its inherited configured model/effort, with no tool-applied override.

### Decision after the additional source trace

There **is** an existing Native account/session identity and an existing Coordinator lifetime owner. A new account store or permission owner is unnecessary. The production legacy HTTP/Box admission batch is nevertheless **not yet code-ready as a small Rust predicate insertion**: two necessary contracts are absent, and their absence is now demonstrated at the real producer/writer boundaries rather than inferred from an unread auth service.

1. The producer must bind a server-row registration and the credential used to fetch it to the existing authenticated account and exact active Coordinator/MCP lifetime. Native credential context, authorization account scope, and legacy token-derived `display.cacheScope` are not interchangeable. No source contract proves the latter two equal, and no existing MCP descriptor carries the necessary currentness binding.
2. The cutover must give the existing Rust Settings owner sole durable control of the legacy disabled record. On the local Host path, Electron and Rust both write `settings.json` as complete documents. Other Electron settings updates and a read-triggered settings migration can still replace the record even if the MCP toggle setter and resync union are changed. The existing Rust mutex/CAS only serializes Rust owner operations.

These are actionable shared-contract/implementation decisions within the authorized migration, not missing-device/account blockers and not a reason to wait for global source closure. The bounded next implementation must settle both, or perform a necessary public-contract batch that does not claim Rust admission is already authoritative. No bypass based on provider-name syntax, a caller-supplied account hash, current settings scope alone, or “last synced” map is supported by this source.

### Existing account owners and the actual transfer

**Native durable identity producer.** `source/product/fabushi/fabushi-account-service.ts` owns durable Native sessions and its private `operationEpoch`, `logoutInProgress`, active-operation set and login cancellation. Its `writeHostCredential()` writes a private temporary file and renames it to `fabushi-host-access-credential-v1.json`, with version, issuer, access token, expiry, session ID, device ID and user ID. It does not serialize `operationEpoch` or a generic account-transition generation. Login validates the backend identity, persists the session, writes the Host credential and emits status; refresh checks its captured epoch around asynchronous work; logout advances the epoch and waits for captured operations before clearing secret/file state. Token or file presence by itself cannot prove that a transition has not begun: the epoch changes before every old credential file has necessarily been removed/replaced. `peekAccessToken()` explicitly refuses an active login; the Host file reader has no access to that latch.

**Credential path carrier.** `fabushi-account-adapters.ts` installs the credential path into `context.env` and creates the Native service with that path. Production services use `bindings.env ?? process.env`. The existing Coordinator launcher invokes Electron's utility process without replacing its environment; `rust-coordinator-carrier.ts` explicitly spreads `process.env` when spawning the Rust Coordinator. Rust `spawn_host()` inherits that environment and sets the same selected data root plus optional local human ID. Thus the file is an existing production credential transfer, not a proposed side channel. The source trace is not a platform runtime execution claim.

**Existing Host auth identity.** `HostAuthExtension::service()` gives the existing `HostAuthService`. Its `get_fabushi_core_credential()` reads and validates the configured credential file on each use; it does not cache the Native account. The same parsed image yields `FabushiCredentialContext { issuer, user_id, device_id, session_id }`. This type is crate-private and its fields are private; an App API cannot simply assume it is already a public MCP contract. Token/expiry rotation alone does not change this context. Legacy Cursor/Sand inference credentials and their renewal listeners are a separate domain and must not be substituted for Native identity. The auth listener counter is not an account transition revision.

**Existing request-currentness precedent.** `provider_session.rs:1080–1225` already captures a `FabushiResponsesRequestBinding` using the context, endpoint and credential path, then rereads credentials and refuses changed context before transport. Its binding also handles concurrent first captures. This is evidence for a reusable current-credential pattern, not permission to move inference-owned request state into MCP. The next MCP contract should obtain a bounded non-secret claim from the existing auth owner and retain independent MCP registration lifetime under the existing MCP owner.

**Existing Native scope claim.** `session/native_messaging.rs` already has public `FabushiNativeCredentialClaim`: user ID, issuer, device ID, hashed session binding and expiry. Its `account_scope_hash()` matches the issuer-qualified Native account slot, including isolated CI issuer qualification. `credential_claim()` checks the existing private regular credential file, configured issuer and expiry. `verify_account_identity()` uses the existing authenticated user-info endpoint; App mini-app scope uses that proof and its own install-ledger generation. The mini-app generation is a mini-app ledger authority; it must not become a general MCP permission revision. The full Native claim includes expiry, while `FabushiCredentialContext` deliberately ignores token/expiry changes. Which representation a prepared MCP request compares is a contract decision: naively comparing full claims would retire otherwise identical work on refresh, and comparing only a user hash would miss a changed session/device/issuer.

**Existing account/lifetime owner.** `CoordinatorAccountRuntime` already serializes account transitions and owns the active runtime, its `RuntimeLifetime`, `observedVersion` and applied version. `observe()` synchronously marks the old/pending lifetime retired when the account slot changes, before queued cleanup runs. `captureCurrentBinding()` returns a closure bound to the exact active session and lifetime; `assertCurrent()` refuses retirement, replacement or disposal. `production-provider.ts` composes this with the adopted MainData port in `captureMainDataSession()`. This is an existing proof mechanism, not a new epoch registry. The counters and JavaScript object identity are not an authenticated serializable MCP binding as currently exposed.

The actual MCP executor callbacks in `production-provider.ts:419–451` still delegate through `context.requireMcp()` and are created outside `createRuntime()`. The legacy MCP runtime separately fences manager construction with its `Attempt` identity, and the manager has its own discovery `generation`, but neither is carried in a routed Rust tool. Merely calling `captureCurrentBinding()` when an old request arrives could capture a replacement runtime; the request must retain the originating lifetime/port binding. A new wire field supplied by the caller cannot create that proof.

### Why `display.cacheScope` is not the Native authorization scope

The exact source chain is:

| Producer / consumer | Identity actually used | Consequence |
| --- | --- | --- |
| Native `statusFromSession()` via `nativeAuthIdForIssuer()` | Exact user ID for production/loopback; issuer-qualified derived ID for isolated CI | Stable Native account slot independent of token encoding |
| `account/account-authorization.ts` after host binding authorization | `accountCacheScope(slot)` | Hash of the authorized Native slot; store scope is changed only after the existing host authorization succeeds |
| `fetchAccountMcpServers()` | `accountCacheScope(accessToken)`, where it captures the actual token before backend enumeration | Hash of parsed JWT `sub`, or of the complete opaque token when no usable JWT payload exists |
| `SandMcpManager.loadAccountServers()` | `display.cacheScope`, then `settingsStore.scopeToAccount()` | A discovery result can change the persisted scope independently of Native account authorization |
| Existing Native Rust claim | Hash of issuer-qualified user ID | Matches Native authorization semantics; does not prove equality with arbitrary JWT `sub` or opaque token hash |

The account MCP client uses the captured token for its enumeration/config responses; it does not reread a potentially different account's token between those calls. That is useful provenance, but the returned display object contains only the derived scope and rows, not a Native claim or captured Coordinator binding. Its source permits opaque bearer tokens; Native session validation does not require JWT `sub === userId`. Actual backend access is through `getSandInferenceBackendUrl()` and the configured first-party origin; the production Native auth service refuses a different issuer target. None of those checks establishes the scope-equivalence assumption.

The public row ID still comes from the backend's `AvailableMcpServer.id`; account-slot identifiers come from that row's account list. The mapping must remain an explicit captured row/slot relation and preserve HTTP's routed public `name` versus raw disabled-policy `toolName`. Neither the provider label nor a token-derived cache key proves ownership by itself.

**Exact unresolved migration contract:** how a legacy disabled map stored under a token-derived scope is adopted into the authenticated canonical account scope without clearing valid user choices, granting from another account's map, or assuming token format. A currently authenticated row enumeration, captured credential context and exact old scope can supply a possible migration witness, but the existing APIs do not return that combined witness. Blindly renaming `cacheScope` or resetting differing scopes is not a compatibility-preserving implementation.

### Complete relevant writer classes located in this bounded campaign

This table covers the literal legacy disabled-field and scope writer references found in production Electron/shared code, their concrete implementations, and the actual Rust Settings entry. It is not a claim to have read every settings consumer in the repository.

| Writer / entry | Current behavior and store | Necessary cutover treatment |
| --- | --- | --- |
| `SandMcpInstructionsAndToggles.toggleMcpToolDisabled()` | Resolves row; reads the whole desktop map; inverts the bit; writes the whole map | Commit one explicit desired disabled value through Rust; acknowledge only the committed snapshot; no retry of a blind toggle |
| `deleteDisabledToolsForServer()` called by manager removal/uninstall | Removes rows after backend config/plugin mutation, writes desktop map | Use the same Rust owner for row cleanup and retain actual backend outcome when later policy persistence fails |
| `SandMcpManager.loadAccountServers()` | Applies `display.cacheScope`, which may clear maps and other account-sensitive settings | Bind enumeration to canonical account/lifetime; do not let the cache label choose durable account ownership |
| `createProductionAccountAuthorization()` / `account-authorization.ts` | Scopes local Settings after host/account authorization | Reuse this authority for binding/import; avoid independent MCP authorization |
| `account-transition-cleanup.ts` | Begins departure; clears Host scope/maps through resync; finally clears desktop scope, descriptor and manager | Retire admissions before awaiting cleanup; no stale clear may clear a replacement account; failed clear cannot re-authorize old work |
| `mcp-desktop.ts` toggle, removal and instruction sync | Desktop writes precede Host updates; toggle retries map push three times and returns tools even without a successful Host receipt | Thin bridge to the authoritative semantic command; retain explicit failure/unknown commit outcome; no stale whole-map retry |
| `coordinator-resync.ts` startup/reconnect | Initializes missing Host scope from desktop or unions same-scope disabled maps; writes Host, then desktop projection | After cutover use authoritative read/projection; union cannot revive disabled entries after a committed enable; import must be explicit and bounded |
| `production-root-auxiliary-provider.ts` resync setters | Real callbacks directly invoke `SandSettingsStore` map setters | Must change with the resync contract, not merely change an interface test double |
| Rust Settings public/trusted update and specialized scope/map methods | Checked read, Rust owner lock, whole-document commit; `config.set` / `setHostSettings` accept scope/map fields | All supported mutation entries must participate in the same account/currentness contract and receipt; generic writes cannot silently bypass semantic admission state |
| `SandSettingsStore.persist()/update()/applyPendingMigrations()` | Every ordinary setter reads and rewrites a complete parsed document; `load()` can persist a pending migration | On the local Host path these can replace the same permission record despite removal of direct MCP setters; this cross-process writer must be handled |

`production-binding-providers.ts` creates the real Electron `SandSettingsStore(join(getSandRootDir(), "settings.json"))`. Rust `SettingsService::production()` selects the same filename under the data root supplied to the local Host. `SandSettingsStore.persist()` uses process-local temp-and-rename with no shared Rust lock; `SettingsService` has an in-process mutex. Atomic rename protects document publication, not lost updates across the two writers. A failing TS parse also projects defaults before an ordinary write; the Rust checked writer deliberately refuses invalid sources. Permission admission must use the checked source and must not translate corrupt/unreadable state into an empty allow map.

**Useful existing revision, with its limit.** Rust `get_public_host_settings_snapshot()` / `set_public_host_settings_if_current()` already provide a keyed public normalized revision/hash and checked compare/commit under the Settings mutex, with a receipt captured before listener callbacks. The trusted form also exists. The base hash is optional on legacy setters; it is a whole-configuration content revision, not an account-lifetime proof or a serialized MCP generation. It can support a typed policy commit once all relevant writers obey the contract. Adding a second “policy revision” without first closing the writer path would not make it authoritative.

### Minimal shared contract that must be frozen before enforcement

The owner selection is known from source: **Native Account/HostAuth owns credential identity; CoordinatorAccountRuntime owns departure/currentness; existing MCP manager/service owns registered row/slot/transport preparation; Rust Settings owns the durable disabled state and commits; the existing relay owns publication/outcome custody.** No new account store, scheduler, permission registry or Agent loop is needed.

The following are minimal semantic types, not an applied patch or names of existing implemented APIs:

| Contract | Required data / provenance | Required behavior |
| --- | --- | --- |
| Bound legacy registration | Backend row ID, explicit provider/slot relation, raw tool identity and routed name; originating authenticated Native context or supported legacy auth context; exact originating Coordinator/MCP lifetime | Issued by the existing registration producer after its authenticated enumeration; rejected if retired, changed or unknown; never reconstructed from provider-label syntax |
| Policy read / set-disabled receipt | Owner-established account binding, exact row/tool, desired boolean, current Settings revision, committed revision and state | Checked Rust read/commit; no UI-chosen scope; no successful UI return on failed commit; conflict reconciles intent rather than replaying an inverse toggle |
| Prepared execution | Immutable registered identity, the already-selected HTTP client or Box configuration preparation, exact lifecycle binding, current Rust policy decision | Check after all pre-effect awaits; stale/disabled/unmapped fails before the actual HTTP execute or Box execution handoff; a later replacement is never selected for the original request |
| Effect settlement | Existing relay request ID and publication boundary, actual backend result or unknown outcome | Once entered, preserve the actual result even if account, manager or policy later changes; no post-effect stale denial, fabricated known-no-effect or automatic retry |

A viable one-writer cutover has to select **one** storage strategy within Settings before code is frozen: either route every whole-document writer through the Rust owner, or move the one legacy MCP policy record into a Rust-owned record under that same Settings owner with an explicit, one-time checked migration and retire the old field as authority. These are alternative implementation decisions, not two parallel authorities and not a recommendation to create a new module. Merely deleting the TS MCP setter is insufficient because unrelated complete-document writes remain. A side record would also require defining crash/rollback/import ordering and how public configuration reads/updates reach the canonical record; this assessment does not claim that migration contract exists already.

The exact still-unresolved decisions are therefore finite: **canonical account/legacy-scope import witness; originating runtime binding transfer; storage cutover and CAS receipt semantics.** The auth service itself is no longer unread. The remaining uncertainty is not the location of the owner: it is the absent contract joining those existing owners without weakening legacy compatibility or write serialization. Root can decide and implement these within scope; source evidence does not justify silently selecting defaults and calling the admission batch complete.

### Causal verification required for the eventual real-path batch

These are test oracles for root's eventual frozen-H Actions selection. None was executed here, and no local harness or schema/validator was invoked.

1. **Real toggle and restart:** enter through `sand:mcp-toggle-tool-disabled`, hold/fail the real Rust commit port, assert no success or local-only permission mutation; on success reopen the authoritative store, list the real routed tool, invoke via App Runner bridge, and observe zero HTTP/Box execution for the committed disabled tool. A committed enable survives reconnect/resync and restart without union resurrecting it.
2. **Scope non-equivalence:** use a Native session with an opaque token and another with JWT `sub` different from the user ID; retain authenticated backend rows. Prove adoption preserves only the current account's known disabled entries or produces an explicit migration refusal. It must neither clear silently nor treat a cache string as authorization. Include issuer-isolated account scope and token rotation within unchanged Native context.
3. **Transition before credential replacement:** pause login/logout after the Native epoch/lifetime is retired but before file replacement/cleanup. Release a pending registration, policy commit and prepared execute from the old runtime. Each must refuse before effect/commit, even while the old credential file is still readable. Repeat same account with a new session/device and stale origin requests arriving after replacement runtime activation.
4. **Cross-process writer:** pause an unrelated real TS Settings write after its read, commit disabled state in Rust, then release TS publication; policy must remain committed under the selected cutover. Also cover TS read migration, `config.set`, row removal, and account-clear ordering. This is a concrete risk that an isolated Settings unit test cannot discharge.
5. **Last awaited preparation:** for HTTP delay actual provider preparation; for Box delay configuration load/push. Disable or retire during that await; then release it and assert the real executor was not called. Alias row lookup uses the returned explicit slot relation, including colliding-looking provider strings. Raw `toolName` policy and routed HTTP `name` remain distinct.
6. **Already-entered effects and commit uncertainty:** latch the actual HTTP/Box execution call; then retire the manager/account or disable the tool. Return the actual backend result and prove it is preserved with exactly one execute call. Transport loss after effect publication remains unknown. A lost policy commit reply must reconcile by receipt/revision/read, not replay a blind toggle or report success from desktop state.
7. **Checked state and compatibility:** unreadable, corrupt, read-only or stale policy source refuses admission/commit without replacing stored bytes; official MCP still uses its existing scoped vault/install lifecycle until its own explicit Rust migration; existing legacy HTTP and Box success/error contracts and AutoReview semantics remain exercised.

Unit doubles can expose ordering and byte/dispatch invariants, but real backend account-mapping claims, actual HTTP/Box compatibility and supported platform file/credential behavior remain blocked until their corresponding account/service/device/platform evidence exists. Those external gates do not block implementing the typed owner cutover once the shared decisions above are explicit.

### Concrete implementation paths and integration boundaries

If root chooses this complete next batch, the real edit surface is bounded to the already identified owners: `source/host/src/extensions/auth/auth_service.rs` (bounded non-secret context access only), `source/host/src/extensions/settings/settings_service.rs` (canonical commit/read/migration contract), `source/host/src/extensions/mcp/` existing service/relay files (registration/prepared decision), `source/host/src/extensions/inference/provider_session.rs` (routed descriptor contract), `source/host/src/runner/coordinator_tool_relay.rs` and `source/host/app/src/main.rs` (real typed relay/composition), plus the existing Coordinator runner control carrier if the shared contract changes it. On the producer/UI side: `source/shared/node/cursor-backend/account-mcp.ts`, `source/shared/node/mcp/{mcp-manager,tools-discovery,mcp-instructions-and-toggles}.ts`, `source/shared/node/settings/sand-settings-store.ts`, `source/electron-main/mcp/{mcp-runtime,desktop-mcp-manager,mcp-desktop}.ts`, `source/electron-main/adapters/mcp-oauth.ts`, and the existing account/Coordinator composition, resync and transition files named above. Tests belong beside those actual owner paths and the existing real-path contract suites. This is a prospective root-owned allowlist, not permission for this read-only agent to edit them.

Root must serially freeze the account/descriptor/receipt schema, Settings storage cutover and composition because those are shared. The official lifetime worker's mutable four-path work remains outside this assessment; any overlap in shared facade/adapter source must be integrated from its actual diff by root before H is frozen. No new workflow, OCRM inventory/ledger or governance document was changed here.

### Additional immutable identities and literal read coverage

Every path below was read via exact `git show 8e5cee76…`, not mutable worktree source. Ranges describe observed source, not full-file comprehension where omitted. Broad search output that was truncated was not counted as full reading; required manager/adapter/storage segments were reread separately.

| File | Blob at exact base | Additional observed coverage |
| --- | --- | --- |
| `source/host/src/extensions/auth/auth_service.rs` | `56cc268d11369f3d941bc1a5f007d8c97c8b82c4` | 1–550, production service before tests; focused context/current-reader reread |
| `source/host/src/extensions/session/native_messaging.rs` | `08e0921e9826006e109ed4253d518fad8bf4c68c` | 95–165, 245–375, 970–1080; claim/file/issuer/currentness only, not full messaging client or tests |
| `source/product/fabushi/fabushi-account-service.ts` | `aa2358143079451cfa32b4c4dd64a148e20dfb2d` | 1–230, 350–end; intervening remote-computer registration internals not deeply read |
| `source/product/fabushi/fabushi-account-adapters.ts` | `b2e0fa5a88d6d0f19f7637ba62f79c9e4fc6cf0e` | 40–135 credential path / native service construction, relevant environment matches |
| `source/product/fabushi/fabushi-account-policy.ts` | `31da18303e7a482ed5199e60936f601c0ada0af1` | Complete file |
| `source/electron-main/coordinator/coordinator-account-runtime.ts` | `84e0e0ee6f83c0932a66ec4f89167b32e3f0e4ef` | 1–465, 470–end across bounded reads; no claim for the intervening few unshown lines |
| `source/electron-main/coordinator/production-provider.ts` | `8436339d3e2bfdf05f1001f284dfadee72ecf8ef` | 419–481, 535–625 and scoped binding searches; not complete file |
| `source/electron-main/coordinator/production-root-provider.ts` | `4c5a4f73315994da7258d1d605ba4f5a4d438605` | 1–96 actual account cleanup/authorization composition |
| `source/electron-main/coordinator/production-root-auxiliary-provider.ts` | `3afe596fea71772b463454d3350c8841d9a3335a` | 99–126 actual Settings resync setters |
| `source/electron-main/coordinator/account-transition-cleanup.ts` | `9a5d5701ceefbeb137b407a8891db06737577a8f` | Complete file |
| `source/electron-main/coordinator/coordinator-launcher.ts` | `1b6192d01594e691171c36397a63dda154b41565` | Complete file |
| `source/electron-main/coordinator/rust-coordinator-carrier.ts` | `375a706177e9bfa7aa7c9bf591d6688887af12fb` | Complete file |
| `source/electron-main/adapters/coordinator-native.ts` | `319a06fcca678e546b594abe40ebf5a895570299` | Complete file |
| `source/electron-main/account/account-authorization.ts` | `5470896e1a623e69ec3b326cb08a5fb9865fb737` | Complete file |
| `source/electron-main/account/production-account-authorization.ts` | `10ed5490a4324eba60957fdaf4edd2a46aac769d` | Complete file |
| `source/electron-main/main-production-services.ts` | `d194b12a676ca93e992a3638e5d5dde286d03c53` | 430–449, 625–675 plus scoped `bindings.env ?? process.env` producer search; not complete file |
| `source/electron-main/mcp/mcp-runtime.ts` | `4236b468306aee00d727bbd59d27c15537fbf768` | 1–185; manager construction/attempt/credential forwarding, not remaining cleanup tail |
| `source/electron-main/production-binding-providers.ts` | `e65c2b1e9160d30dad1b549558e3d01826069dce` | 231–263 concrete Settings file construction |
| `source/shared/node/cursor-backend/account-mcp.ts` | `ffbb57149a5f992cf1746875be4e19a2ca1d08ef` | Complete file |
| `source/shared/node/cursor-backend/cursor-inference.ts` | `296eeb02b7b0bc37256fc0d89b5c215d7cd1d501` | Imports, backend producer at 109 and adjacent client construction from 1–180 output; no full-file claim, output was partially truncated |
| `source/shared/node/sand-data-root.ts` | `53310a97346d7e11c42596d339f57bb51f6802a1` | 1–85 root selection; no broader startup validation claim |

Previously identified blobs received these additional reads: `provider_session.rs:1080–1225`; App `main.rs:15305–15428`; Rust Coordinator `main.rs:1320–1410`; manager `mcp-manager.ts:345–497`; `desktop-mcp-manager.ts:1–182`; adapter `mcp-oauth.ts:75–119,205–343`; `cursor-token.ts` complete; `mcp-desktop.ts` and `coordinator-resync.ts` complete physical files; TS Settings 75–117; Rust Settings 30–125, 410–506, 512–582, 575–682, 1330–1534, 2010–2073. The earlier `HostAuthExtension` limit “deeper service not read” remains a historical statement in the original table and is superseded only for this later continuation by the precise auth-service range above. No historical read fact or verification level was upgraded.

**Result:** immediate auth/source dependency traced; existing owners and real storage/currentness gaps named; no production changes, executable validation, real-account acceptance, OCRM flags or GitHub delivery claimed by this assessment.
