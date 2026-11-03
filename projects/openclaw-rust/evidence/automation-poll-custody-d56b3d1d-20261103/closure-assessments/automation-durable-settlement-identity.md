# Automation durable settlement: identity, lifetime and ACK dependencies

## 1. Decision supported by this investigation

**There is an existing stable local account identity, but the shipping Sand transport does not expose a stable principal or a current-owner credential fence tied to that identity.** The desktop account slot reaches `ProductionSessionWorkers::local_human_id()`, and `AutomationRuntime` already owns those workers. The Electron Coordinator owner also has a real revocable lifetime. Neither fact currently accompanies the bearer token obtained by `ReqwestAutomationsBackendTransport`. Native Core credentials have a stronger account/session context, but the shipping Automations lifecycle deliberately uses the separate Sand inference credential path. Substituting one identity for the other would invent an authentication relationship. [I01–I08; A01–A05; B01–B03]

**The existing ACK retry policy can be persisted and resumed without adding a new remote operation.** The consumer already retries the same `ackRunUuids` after a failed poll or a successful poll that still returns those IDs. Persisting an ACK-owed phase can therefore close the local crash window after remote ACK and before local pin release. It is a continuation of an existing client policy, not proof of deployed idempotency. Its safety and eventual release still depend on a current, correctly bound account and on the meaning of a successful poll response. [B04:223–329; T01:1003–1077]

**The current checked ACK response is insufficient for an unconditional durable-release claim.** Shipping transport accepts any 2xx response that decodes as JSON. The consumer treats missing or non-array `events` as an empty list, then removes every Reported ID not present in the returned ID set. There is no per-ID ACK receipt or checked application success envelope. Even the separate Native adapter checks the poll protocol and events but does not check an acknowledged-ID receipt. This report does not upgrade successful absence into a proven service guarantee. [B01:65–101; B04:262–291; B05:100–117]

Consequently, a bounded batch can specify and prove **local durable custody and restart continuation of the current client retry policy**, with explicit account-binding and response assumptions. It cannot honestly claim general cross-account automatic settlement or remote exactly-once completion using only the present shipping interfaces. If those identity dependencies remain unresolved, the narrower option is durable pinning plus fail-closed recovery, with its liveness limitation stated plainly. Do not label that narrower option complete startup settlement.

This is a dependency investigation, not a second production defect report or acceptance decision. The one previously confirmed behavior remains the terminal coalesced-alias row being pruned while that alias still owes backend settlement, followed by healthy reopen and fresh execution on redelivery. The earlier full causal report is `/tmp/next-automation-closure-gap.md`, 58,921 bytes, SHA-256 `d217118675c8ddb8bd98e008d2b5e7b3a2f867efd94564a3ff27ee1b8a6a7430`. No new replay guarantee for already acknowledged history is proposed.

## 2. Frozen subject, authority and limits

| Subject | Exact identity |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| Inspected H | `71b039ae860c45e982dfbc8b1f401143d49d2454` |
| H tree | `3c30ec8ac78d0b8b549267a4b794a7cb177b8b23` |
| Read-only bare source | `/tmp/ocrm-source-84120837.git` |
| Earlier investigation H | `bb5a7f6098f952df6d388a672c4fb01045bc645c` |
| Earlier H tree | `5ad5f0e4a774947e22d6c3956775590503edac0c` |
| Fixed upstream U | `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` |
| U tree | `90c8aab39972da11cfad220361781d49e1bc6700` |

All report citations refer to the exact H path/blob in §11, unless expressly called prior read or contract review. A notation such as `[B04:262–291]` means those lines of that exact blob. Source can be independently reopened using `git --git-dir=/tmp/ocrm-source-84120837.git show 71b039ae860c45e982dfbc8b1f401143d49d2454:<path>`, or its immutable GitHub blob URL. No floating main is used.

Root AGENTS, project SOT, main Spec §17, current task, M19 map, full cron domain contract, full Runner domain contract and completion/ACK contract were read in the preceding investigation. Their blob identities were checked unchanged at H71. The relevant identity and actual shipping composition were read at H71, with exact full/bounded extents below. The previous full cron reads are reused as unchanged contract reads; they are not represented as a new full read or execution. This follow-up makes no additional U implementation or transitive-dependency closure claim. The fixed-U source work remains as indexed in the prior report. [S01–S09]

The bounded Git diff from the earlier H to H71 shows no change in the inspected Automation production modules, Auth modules, run path or automation store. The specified App and Session production files changed and their H71 identities and relevant composition ranges were read directly. This is not a full review of the incoming media changes. No incoming change was modified.

The assigned reasoning effort is xhigh, inherited from the parent task; no model override or subagent was used. Only source/Git reads and this report write were performed. No build, test, lint, generator, schema validator, project helper, local/htch validation, live service call, credential read/output, push or workflow dispatch was performed. No peers' private current reports were opened. All OCRM implementation/wiring/verification/acceptance responsibilities and gates remain open.

## 3. Which identities and fences actually exist

### 3.1 Stable desktop account identity reaches the existing Session owner

The current native account service produces `SandAuthStatus.authId` with `nativeAuthIdForIssuer(session.userId, session.issuer)`. Production and permitted loopback issuers use the user ID string; the isolated CI issuer uses a domain-separated hash with an issuer-qualified prefix. Device ID is deliberately separate. Thus the local stable identity is an account slot, not a token, machine ID or device session. The loopback behavior also means that durable network work must separately bind the actual backend origin; a local account slot alone is not a universal issuer namespace. [I01:47–64; I02:39–63]

The actual chain is:

1. `FabushiAuthService` emits the settled status. The native adapter installs the account credential path and obtains status before Coordinator/Host startup. It does not install that native bearer as a Sand inference renewal credential. [I01; I03:72–154]
2. `production-provider.ts` authorizes the account slot, sets `localHumanId = slot`, and places it in `processConfig.localHumanId`. Its own source comment identifies it as the settled native authId reused across devices. [I04:498–505,529–557]
3. Rust Coordinator reads `process_config.local_human_id` and passes `FABUSHI_LOCAL_HUMAN_ID` to its Host child. [I06:1357–1394; I07:1–85]
4. Shipping Host reads that environment value and constructs `ProductionSessionWorkers::production_with_identity_and_dependencies`. The workers normalize and retain the field; `local_human_id()` returns the immutable local account slot or an error. [I08:16139–16148; I09:322–347,410–454,4498–4502]
5. `AutomationRuntime` holds `Arc<ProductionSessionWorkers>` already. A small method or injected read callback in this existing owner can compare a saved local account binding with `self.sessions.local_human_id()`; no new account database or sidecar identity file is needed for that local comparison. This is a proposed use of the existing owner, not a method that already binds Automations network requests. [R01:300–311]

In shipping composition a missing localHumanId prevents Coordinator creation, although Session constructors used by other paths/tests permit `None`. A recovery implementation must not turn absence into a shared anonymous identity. It must either remain in custody without sending or use a separately specified supported owner source. [I04:498–505; I09:315–347,376–454]

### 3.2 Current desktop owner lifetime exists, but is not an Automations request permit

`CoordinatorAccountRuntime` owns the one Coordinator process for the settled account. `observe` increments its observation version and synchronously marks the active lifetime retired when the slot changes. `applyClaim` revokes the old owner, prepares account transition, stops the previous runtime and waits for confirmed exit before launching the replacement; unconfirmed shutdown blocks replacement. `captureCurrentBinding().assertCurrent()` checks the captured lifetime and runtime identity. This is a real existing ABA-resistant lifetime at this composition boundary: a retired lifetime is not revived just because the account string later matches again. [I05:124–150,190–276,391–458,501–536]

However, the interface itself says it is a native composition identity, not an account claim. The current Host fire callback does not carry this binding, observation version or revocation signal. Production's `captureMainDataSession` combines this binding with main-data ownership for that Electron edge; it does not make the Sand credential store account aware. [I05:41–45; I04:578–587]

Rust Coordinator has a `host_generation: AtomicU64` for child incarnation and routing rejection. It increments when spawning a Host. The inspected bootstrap carries the stable account slot, not that generation. On Coordinator input close it drops Host stdin; the Host's stdin-closed branch begins shutdown, cancels runners and stops Automations. This supports an existing process-lifetime boundary, but it is not a synchronous per-request authentication fence inside a detached Automations completion callback. It also does not prove that a remote HTTP side effect already in flight is cancelled. [I06:1371–1388,1518–1556,2968–2998; I08:17953–18045]

The native account service's `operationEpoch` is another genuine fence, but it is private to its asynchronous login/refresh/logout operations. Credential-file version 1 does not export this epoch. Logout increments it, waits for active operations, removes the native credential file and emits logged-out status. The file is replaced on valid account refresh/login. None of that changes the separate Sand inference credential record through an explicit current-account API. [I01:93–137,195–218,370–405,547–667]

### 3.3 Native credential context is useful, but belongs to a different authentication path

`HostAuthService::get_fabushi_core_credential()` reads the native file afresh on every request and validates its fields and expiry. `FabushiCoreCredential::context()` exposes, inside the crate, an equality-comparable tuple of issuer, user ID, device ID and session ID from the same credential image. Token rotation and expiry changes do not change this context. `authorize_origin` checks the native credential's approved issuer against the requested origin. [A01:29–95,112–159,482–496]

The separate `NativeAutomationsBackendTransport` already demonstrates a bounded native pattern: capture issuer/user/session/device at construction; read the current credential before a request; require the same identity; send the token from that validated image; read the credential again after response body reception; reject a changed context; reject redirects; check an exact 200 and native protocol. It also validates that a completion receipt echoes the requested ID/status/error. [B05:18–119]

This is **not the production backend selected by `ProductionAutomationsLifecycle` at H71**. The lifecycle constructs `ReqwestAutomationsBackendTransport` and a Sand Connect client. Replacing that with the native adapter would change protocol, headers, credential domain and service assumptions. That is not a harmless way to obtain the missing fence in this batch. The existing native adapter's comment itself requires new runtime creation for a new session and reconciliation under the original account. [B02:119–166; B05:35–50]

The native tuple is equality, not a persisted monotonic account generation. It detects the tested account replacement between request and response. It cannot independently prove that no intervening A→B→A restoration of the identical tuple occurred between two reads. The Electron retired lifetime provides stronger observed-lifetime semantics at its own boundary; that fact cannot be silently assigned to the native tuple or the shipping Sand transport.

`FabushiNativeMessagingClient` also exposes a non-secret claim containing issuer, user ID, device ID, session-binding hash and expiry, and an authenticated identity-verification method. Its account-scope hash follows the native local account namespace. This can inform reuse of existing native identity code if a native service batch is separately approved. It does not attest a Sand inference token. No verification endpoint was invoked in this investigation. [I10:123–166,337–386,971–1065]

### 3.4 Shipping Sand auth currently exposes token availability, not stable principal ownership

The Sand `InferenceCredential` contains only `access_token` and `expires_at_ms`. The credential parser accepts a nonempty token with explicit expiry, JWT expiry or a fallback expiry; it does not require a JWT subject or carry an account claim. The store protects only `Option<InferenceCredential>` with a mutex. `get_valid_access_token` returns a cloned token if not near expiry. There is no atomic public snapshot containing a stable principal and an ownership epoch. [A01:182–224; A02:30–47,74–126]

The Host auth source is captured when the service is constructed: a development token-file path or a Sand renewal credential from the environment. The production native credential path is held separately. The code explicitly forbids using it as a Sand/Connect renewal source. A fixed renewal source string is not a validated public principal; a development file can be replaced, and the renewal response parser itself does not establish that every returned token has one immutable subject. Hashing the source secret would create a source fingerprint, not the missing non-secret principal contract. [A01:306–362]

Each Reqwest Automation request obtains the current token anew through `HostAuthExtension::get_access_token()`. It sends that token with the configured backend URL and current `x-sand-box-namespace`. It does not capture or compare a principal before/after the HTTP request. The current auth-renewal callback delivered through `BrowserUaAuthApi` contains only `outcome: renewed/failed`; `ProductionAutomationsLifecycle` uses `renewed` to wake cloud reconciliation. It does not retire the backend owner or partition the pending fire map by principal. [A03:32–48,70–90; B01:65–99; B02:355–360]

`HostAuthRenewalEvent.is_first_credential` is computed from whether a valid token was present before the store write. It is not an account change or generation, and the Browser-UA projection drops it. Renewer `requested_epoch`/`completed_epoch` count renewal requests and waiter completion. Listener subscription IDs count subscribers. None is an authentication-owner generation. Preserve the current renewal-worker exit/waiter correction; repurposing those epochs would conflate a different state machine. [A01:350–390,498–505; A02:311–410]

The misleading constant name `DEFAULT_CURSOR_BACKEND_URL` now contains `https://api.ombhrum.com`; current origin configuration prefers `FABUSHI_API_BASE_URL`, then `SAND_BACKEND_URL`, and allows approved native/isolated origins or loopback. That current default does not change which credential path the Sand transport calls. The historical api2.cursor.sh default in the ACK spec's old baseline paragraph is not current-H runtime evidence. [A02:128–180; S05:33–38]

### 3.5 Values that must not be silently promoted into the missing principal

| Existing value | Actual role | Why it is insufficient for this durable Sand boundary |
| --- | --- | --- |
| Session `local_human_id()` | Stable authorized desktop account slot | Good local storage binding; no established equality with the Sand request's token owner |
| Native issuer/user/session/device | Current native credential context | Different credential domain; full tuple is session lifetime, not stable across reauthentication |
| `principal_from_access_token()` in full-name service | Decoded JWT `sub` used to partition a display-name cache | Private helper, no signature/issuer/audience validation, and applies display normalization that collapses whitespace and truncates to 200 characters; current credential parser permits opaque tokens |
| Cursor backend `AccountScope` | Private cache partition from JWT subject or token fallback | Fallback changes on token rotation; private cache hashing is not a stable principal or current-owner lease |
| `get_machine_id()` | Installation identity | Not account ownership |
| `sand_box_namespace()` | dev/lab/prod environment namespace | Not account or device ownership; bind it as request namespace if used, not as principal |
| auth `is_first_credential` | Previous valid-token availability | Can change on expiry/recovery for the same account; not ownership |
| renewal requested/completed epochs | Renewer waiter progress | Not account change; not a request credential snapshot |
| Electron auth-status freshness / private display resolver generation | Event/cache freshness | Different owner and purpose; not exported to the Host request |
| Rust Coordinator host generation | Child-process incarnation | Useful routing lifetime; not a stable cross-restart principal and not currently given to Automations |

[A01–A06; I05–I09; I11]

## 4. Actual callbacks, switching and comparison points

### 4.1 Production callback inventory

| Actual producer/caller | Actual consumer | Identity and lifetime available today |
| --- | --- | --- |
| `HostProductionExtensions::start_automations` | `ProductionAutomationsLifecycle::start` | Host backend URL, HostAuthExtension, notify bus and fire callbacks. No Session account claim argument. Poll readiness is `fire_inference.is_ready()`. [B03:1269–1318] |
| Lifecycle construction | `ReqwestAutomationsBackendTransport::new` and `ProductionAutomationsBackendRuntime::start` | Same HostAuthExtension; no captured Sand principal. [B02:136–166] |
| Backend poll worker | `FireDispatch` after releasing the backend state lock | Fire envelope has ID, agent, automation, timestamps/revision/event; no account or lease. Backend inserts Running before queueing delivery. [B04:71–95,294–305; B06:157–201] |
| Shipping Host `fire_dispatch` closure | `dispatch_production_backend_fire` | Gateway has Session workers and AutomationRuntime; current callback type remains `Fn(Option<FireCompletion>)`. [I08:17682–17685,4022–4131] |
| Checked shipping ingress, known terminal or a classified rejection | Same completion callback immediately | `Complete` calls `completion(Some(terminal))`; `Abandon` returns false. Missing definition, disabled or nonmatching trigger can have no owned execution row. [I08:5997–6040; B07:41–153] |
| Detached schedule worker | Same completion callback after actual run-path return | Calls `run_server_scheduled_automation_attempt_with`, then `automation_fire_completion`. No callback generation capture. [I08:4042–4078] |
| Detached event worker/coalesced caller | Same completion callback after enqueue result | Calls `enqueue_event_automation_fire_with`; each backend caller retains its own ID-bound completion closure. [I08:4080–4123; R02] |
| Backend completion closure | `AutomationFireBackendRuntime::complete(id, value)` or `abandon(id)` | Captures fire-state Arc and ID. It does not capture WakeState, account scope or generation. `None` releases only the volatile claim. [B06:180–201] |
| Consumer direct complete / retry loop | Transport `post_json` | Both completion paths advance to Reported only on success. No durable callback or typed remote receipt currently exists. [B04:187–204,332–365] |
| Consumer successful poll | In-memory Reported deletion | No callback reaches FileAutomationStore or Session when an ID leaves the map. [B04:262–291] |
| Host shutdown / lifecycle stop | Backend stop/join | Polling stops and its worker is joined; detached execution callbacks retain their Arcs. Consumer complete only checks map membership and can still POST after `stop()`. This proves a source-reachable late call, not a demonstrated cross-account server breach. [I08:18013–18025; B02:460–494; B06:264–310; B04:159–163,187–204] |

These are the actual fire/completion/ACK callback paths, not a proposed alternate harness. New persistence must pass through them; adding a helper unused by these callers would not close the behavior.

### 4.2 Account-change callback chain and its limits

The native service publishes status; the adapter's existing auth-wiring edge forwards it to the Coordinator account runtime when available. The production Coordinator provider also subscribes and refreshes status with an observation-sequence check. The account runtime invalidates the captured lifetime on a slot change, prepares account cleanup, retires the current process and starts a replacement only after confirmed exit. Those existing owners should remain authoritative. [I01; I03; I04:529–562; I05; I11:41–104]

The Host auth renewal path is separate: a credential write triggers `HostAuthRenewalEvent`, the extension's Browser-UA adapter reduces it to renewed/failed, and the Automations lifecycle wakes cloud reconciliation. This path has no native account slot or account-switch callback. Adding an Automations subscriber that merely increments a counter on every renewal would be an invented account generation and would invalidate same-account refreshes without proving ownership. [A01:350–390; A03:70–90; B02:355–360]

The existing scopes offer three useful places to compare, with distinct jobs:

- **AutomationRuntime/Session admission and mutation:** compare the durable local account scope to `sessions.local_human_id()` while using existing per-agent mutation and admitted filesystem mutation boundaries. This prevents accidental local-record reuse across a mismatched current Session owner.
- **Production backend lifetime:** capture the actual runtime lifetime and make its permanent retirement observable by callbacks. The existing backend WakeState has a permanent `stopped` AtomicBool; a closure can retain this owner signal rather than inventing a global registry. Transient suspension and permanent retirement are different operations and need separate contract treatment. A retired callback may preserve already-owned durable custody, but must not authorize a new-account request or clear a replacement owner's entry.
- **HostAuth/transport request:** obtain one credential/context snapshot from the actual credential owner and validate currentness before send and before accepting the response. For Sand this interface is missing; a second token fetch in Runtime followed by a third fetch in transport is not an atomic binding.

[R01:300–405,568–597,1268–1292; B06:32–76,228–293; A01:182–218,445–469; B01:65–99]

A before/after local check cannot retract a remote operation that already committed. If the account changes while a completion or ACK is in flight, the response must not be used to release custody under another account. Preserve an unresolved phase for the original binding and let only a properly reauthorized original owner continue it. This is a required local-state rule, not a claim that the service supports reconciliation or cancellation.

## 5. What completion and poll ACK mean in actual current code

### 5.1 Two separate client transitions

| Operation | Request | Current accepted evidence | Local consequence |
| --- | --- | --- | --- |
| Completion report | POST `/sand/automation-runs/complete` with `runUuid`, `status`, `errorMessage` | Transport returns Ok: HTTP 2xx plus JSON decode; body otherwise ignored | Completed → Reported. ID remains pending and cannot redispatch inside this runtime. |
| Poll carrying ACK | POST `/sand/automation-events/poll` with Reported IDs in `ackRunUuids` | Same transport Ok; consumer makes a returned ID set from `events` | Reported IDs absent from that set are removed. IDs still returned remain Reported and are ACKed again later. |

[B01:65–101; B04:187–204,262–291,351–365]

Completion therefore reports a terminal outcome; it is not the same local event as relinquishing delivery custody. A successful completion does not by itself remove the pending ID. Conversely the ACK request does not contain or validate the outcome. For aliases each backend ID is independently reported/ACKed, even though the executor ran only once. A durable record must not infer that confirming primary P settles alias A. [B04; T01:1003–1077]

### 5.2 Failure and unknown-response rules already required

| Observed result | Current state behavior | Durable extension must retain |
| --- | --- | --- |
| Completion 401/404/409 or any transport error | Completed remains; direct error propagates, retry errors retained | Exact completion payload and original ID; no ACK based on error |
| Completion HTTP 204 with empty body | JSON decode fails in current transport | Unknown/unconfirmed completion; no invented success interpretation |
| Completion accepted remotely but response lost | Indistinguishable locally from failure | Retry according to existing policy; no new execution or ACK until accepted local completion evidence |
| Poll fails before a valid response | Reported set unchanged because the `?` returns before retain | ACK obligation retained; repeating same ACK is already client behavior |
| Poll succeeds and returns an ACKed ID | That ID stays Reported | Repeat its ACK; do not resend completion merely because the poll returned it |
| Poll succeeds and omits an ACKed ID | That Reported ID is removed | This is the current successful-absence release rule, not an independently proven server receipt |
| Poll JSON is `{}` or `events` has wrong type | Currently treated as empty events | Must not silently count this as a stronger durable ACK receipt; specify whether to reject malformed successful poll bodies before any durable release |
| Poll succeeds with malformed individual events | Returned set uses any string IDs, dispatch parser skips invalid full envelopes | Typed envelope validation is incomplete in shipping consumer; validation choice must be stated before durable clearing |

[B01; B04; S05:12–22; T01:467–507]

No source in this inspected production chain proves that absence is due to durable ACK rather than pagination, delivery filtering, authorization scope, leasing or server error. No source here proves how a deployed server handles repeated ACKs after retirement. A list named `ackedRunUuids` must not be invented as an existing response field. The separate native client also does not read such a field. [B04:262–329; B05:100–117]

The local native transport tests are useful evidence of the native adapter's checked completion and context behavior. They use loopback fixtures, not the deployed Sand service. Their success receipt body is deliberately native; it cannot be retrofitted as the existing shipping Sand wire contract. [T02]

## 6. Pin-only safety versus durable completion and ACK recovery

### 6.1 Pin-only option

The minimal safety candidate from the first report remains coherent: mark the actual admitted backend IDs, including all coalesced aliases, as unresolved custody in the same FileAutomationStore run image before executing; retain such a row beyond ordinary terminal history pruning; release only the exact ID when the existing backend owner confirms its ACK under the specified local criterion and scope. All writes remain under the existing Session filesystem admission and Runtime mutation boundary. [R01:319–405; R03:224–335,488–512]

This protects the terminal row while completion is still unconfirmed and prevents the previously demonstrated eviction/reopen/redelivery execution. It does not, by itself, make progress after a successful remote ACK whose local pin release never committed. On restart the server may have no further delivery to provoke release, while the only durable fact says the pin is owed. Automatically clearing by age, history pressure, or absence on an unrelated poll would discard uncertainty. Keeping it forever is not an authorized product guarantee either; it is a limitation to expose before selecting pin-only scope.

Pinning must not manufacture backend obligations for manual/local-only runs or for every historical row. Existing immediate rejections with no owned run image are not automatically part of a history-backed persistence batch. Definition deletion and user agent deletion also need explicit compatibility treatment; no new hidden tombstone subsystem is implied.

### 6.2 Minimal durable phases through the same owner

A fuller candidate uses the same protected run image and an exact per-ID phase:

| Durable phase, illustrative name | Evidence already owned | Permitted recovery action |
| --- | --- | --- |
| Accepted / running custody | Fresh backend identity was durably admitted before execution | Preserve current Running/history-unknown Abandon. No execution replay, no invented terminal completion. |
| Completion owed | Safe local terminal settlement plus frozen exact completion payload for this backend ID | Retry that completion under the original authorized scope; no Runner execution. |
| Reported / ACK owed | Accepted completion evidence durably recorded for this exact ID | Include ID in the existing ACK poll; do not report completion again solely to clear the pin. |
| Released | Checked current-owner ACK-release condition committed locally | Ordinary terminal history retention applies; no perpetual replay protection or promise about arbitrary future server replay. |

These names are design descriptions, not implemented schema or protocol. Persistent Running custody remains Running unless an existing legitimate settlement path settles it. Terminal history by itself must not be treated as a ready outbox entry if the exact terminal callback payload was never durably frozen; source currently projects stored error detail differently from some live callback strings. The new spec must choose the write order and exact payload ownership, not regenerate a potentially conflicting error after restart. [B07:162–197; I08:6043–6060]

### 6.3 Crash ordering and the actual liveness argument

| Crash/failure boundary | Required retained local fact | Restart behavior and limit |
| --- | --- | --- |
| After admission before executor terminal | Running custody and IDs | Abandon redelivery; do not rerun unknown side effects. This remains intentionally unresolved. |
| After safe local terminal but before completion-owed publication | At minimum the pin must already exist; no fabricated report | The spec must make the terminal/payload publication part of the existing settlement ordering or explicitly retain uncertainty. A standalone best-effort after-callback write leaves a gap. |
| Completion may have committed but its response is lost | Completion owed, same frozen payload | Reuse current completion retry policy under original scope. Remote duplicate-completion semantics remain external. |
| Completion success observed but Reported phase write fails/crashes | Prior completion-owed record remains | Never ACK based solely on volatile success. Retrying completion is the preexisting uncertainty policy; no execution replay. |
| Reported phase committed before poll | ACK owed | Startup can ACK without waiting for server redelivery and without posting completion again. |
| ACK poll may have committed but response is lost | ACK owed | Repeat the same ACK on the next eligible poll, as the current client already does after poll failure. |
| ACK-release response observed but local release fails/crashes | ACK owed | Repeat ACK after restart; a later acceptable response allows exact-ID local release. No new idempotency claim is needed to describe the client algorithm. |
| Durable exact-ID release committed | No unsettled pin for that ID | Normal bounded history may prune. No promise for arbitrary replay of already released historical IDs. |
| Owner/account changes at any network boundary | Original obligation remains bound to original scope | No send or release under mismatched current scope; only a reacquired legitimate original owner may continue. |

**Conditional local liveness:** if the original account is valid again, the existing readiness and poll scheduling conditions are eventually satisfied, filesystem writes eventually succeed, and the service eventually returns a response acceptable under the specified existing completion/ACK policy, the durable phases can advance to local release without another executor invocation. This is a useful bounded claim. It is not guaranteed progress during permanent auth loss, deletion, server rejection, indefinitely malformed responses or service ambiguity.

The existing consumer already keeps polling when it has owed work, even if no server-schedulable automation remains. It still returns early when `is_ready` is false, and completion retry happens after a successful poll. Shipping readiness comes from the inference owner. Restoring pending states does not bypass these gates. If the intended new requirement is settlement even while execution/inference is unavailable, that is a further explicit readiness-policy change; it is not an automatic consequence of persisting Reported. [B04:232–260,307–329; B03:1288–1299]

The existing policy and test with two repeated ACK polls are sufficient to justify *reusing client retry semantics*. They are insufficient to prove safe server effects for repeated ACKs. The report therefore does not require inventing a new server protocol before describing a bounded local implementation, and equally does not claim the current deployed protocol is verified.

## 7. Compatible minimum interface through existing owners

### 7.1 Separate durable identity from current authority

The durable binding needs enough non-secret identity to distinguish the local account and remote namespace used when the backend ID was accepted: existing Session account slot, selected transport/protocol kind, normalized backend origin, namespace where applicable, and the actual credential owner's stable principal if that principal is distinct. A transient owner-lifetime token must not be the durable principal. Requiring a process generation to be identical after restart would make all recovery impossible; discarding all lifetime checks because the account string matches would admit stale callbacks.

The current-owner permit must instead be reobtained from existing owners for each resumed attempt. It must bind the credential used for the request to the original durable scope and detect permanent retirement of that runtime. The token remains private to HostAuth/transport. No token, renewal secret, raw credential file, or token-derived pseudo-principal needs to be stored in run history.

For the **native** credential path there is existing context code capable of supplying most of this. For the **shipping Sand** path there is no completed equivalent. Implementing a new local method named `current_principal` that calls the full-name JWT helper would not resolve that missing contract. A canonical JWT subject can be a candidate only after the actual Sand credential contract establishes the token shape, issuer namespace, subject meaning, rotation and account-change behavior. Opaque tokens are currently valid inputs; silently failing them or falling back to token hashes would be a compatibility decision requiring specification.

### 7.2 Small internal seams, not a new owner

| Existing owner | Minimum additional responsibility if the fuller batch is chosen | Boundary to preserve |
| --- | --- | --- |
| `HostAuthService` and `HostAuthExtension` | Expose a coherent non-secret context plus private request credential snapshot/currentness operation from the actual Sand store, **after** principal semantics are established | No duplicate auth store, no native/Sand substitution, no reuse of renewal waiter epochs as account epochs |
| `ReqwestAutomationsBackendTransport` | Use the snapshot for request and context-check response; expose typed local poll/completion outcome sufficient for consumer decisions | Same existing endpoints unless separately authorized; no invented ACK receipt; explicit successful-response checks |
| `ProductionAutomationsLifecycle` / backend runtime | Bind original scope/lifetime to incoming fires and completion closures; stop/retire invalidates sending and release authority; restore pending phases before fresh delivery processing | One backend owner and existing wake loop; transient suspend remains distinct from permanent stop |
| `AutomationRuntime` with `ProductionSessionWorkers` | Compare the saved local account to existing Session identity; enumerate checked local pending entries; serialize exact-ID phase/release mutations through current per-agent and Session admission boundaries | No independent file writer; no scan of display projections; missing/unknown owner/history fails closed |
| `FileAutomationStore` and `AutomationRun` storage representation | Validate and preserve bounded versioned custody metadata, frozen completion and per-ID phase; protect unresolved rows from ordinary pruning; release only the exact matching ID and binding | Existing single history image and mutation owner; display history cap remains a separate projection |
| `AutomationFireBackendRuntime` | Restore Completed or Reported from checked durable entries; perform phase write before corresponding outward ACK or volatile removal; retain earlier phase on any local write error | Existing completion/ACK retry algorithm; no execution replay; no ACK from 401/404/409/unknown response |
| Shipping Host callback composition | Carry the original internal scope/permit and route outcomes through the actual callbacks in §4 | No new parallel scheduler/helper/demo path; known-terminal callback does not become new Runner work |

The exact internal type names are for the implementer to choose. A compatible shape is a durable owner binding plus an opaque per-lifetime permit returned with the checked poll result, carried alongside `BackendAutomationFire`, and retained with the exact ID in backend state. Completion and ACK-release callbacks can call Runtime's new exact-ID methods. The same originating permit must be checked when committing phase changes; an unbound callback that just says “ID succeeded” is insufficient.

Do not use two independently sampled identities: reading Session/native identity in Runtime while the transport later fetches an unrelated current token has a race and a domain mismatch. Nor should a generic transport trait default silently treat “no identity support” as globally authorized. Test mocks may use an explicit deterministic test context; production must supply the real context or retain custody without sending.

### 7.3 Ordering, locks and stale callbacks

Network I/O must not occur while holding the admitted filesystem mutation lock. Read/persist an owned phase first, leave the local critical section, perform the existing request with its captured permit, then reacquire the existing mutation path and compare the exact owner binding, run/alias identity and phase before applying the result. The backend currently serializes poll/complete under its fire mutex and invokes dispatch outside that mutex. New persistence callbacks must preserve a consistent lock order; a callback from a backend-held mutex into Runtime must not form a reverse Runtime→backend lock cycle. This is a concrete design constraint for the integrator, not a claim that a proposed interface has already been implemented or deadlock-tested. [B06:157–202; R01:1272–1292]

At minimum, a late callback from an old runtime must not overwrite a newer phase, change a newly accepted account binding, clear a different alias, or send with the replacement account token. A known terminal result may still need to be recorded for the original already-owned run after stop, but recording that local result and authorizing remote send/release are separate permissions. Unknown provider/tool effects retain the existing Running custody and Abandon behavior. This does not authorize cancellation-driven fabrication of failed completions.

### 7.4 Compatibility and boundedness

The old parser reads only known `AutomationRun` fields and `run_to_json` writes only those fields. An optional new metadata field is therefore not transparent downgrade compatibility: an older binary can rewrite history, drop the pin/phase and resume old pruning behavior. The new batch must specify a downgrade guard, compatible writer strategy, or an explicit unsupported rollback condition while obligations remain. No such guard was found or implemented in this investigation. [R03:511–512; R04]

Existing files without metadata cannot be reinterpreted as proof that historical runs were never ACKed. Limit migration to explicit new admissions or a separately justified current unACKed redelivery whose owned terminal row is still available. Do not create perpetual outbox work for every retained historical row. Previously evicted aliases are unrecoverable from this file alone. Manual runs must retain their existing capacity/retention semantics; protected pending custody must have an explicit capacity policy that refuses new work rather than silently evicting unsettled obligations. [R03:234–295,488–503]

The existing atomic history writer uses temporary publication/rename with its current fallback behavior, not a demonstrated fsync-backed power-loss transaction. Preserve the prior report's distinction between process crash/reopen source reasoning and storage/power-loss guarantees. No new durability guarantee is claimed merely because a field is named persistent.

Immediate classified rejections can lack an admitted run row, including `automation_missing`. A batch that only extends existing execution history must explicitly bound its restart guarantee to actual owned execution rows and their aliases. It cannot promise durable settlement of every rejected delivery without also designing a new legitimate storage location and owner; that broader design is outside the confirmed pruning defect. [B07:47–153; I08:6025–6038]

## 8. Required contract repairs and genuine external unknowns

### 8.1 Repair the existing spec/task before code

The existing ACK safety contract expressly limited its correction to volatile state and left persistence/account binding unresolved. The cron contract already protects Running/unknown custody and establishes the real owners; it does not grant a new cross-account outbox protocol. Add a bounded requirement set in that existing authority before implementation. [S04; S05:18–22]

It must decide:

1. Whether the selected batch is pin-only safety or completion/ACK recovery for newly admitted backend execution rows. Give each a distinct acceptance statement.
2. The durable binding and current-owner permit, which existing owner supplies each, and what happens for missing/opaque/unverifiable Sand principal identity.
3. Whether same-account token refresh preserves authority, whether a new native session requires explicit reacquisition, and how stop, suspend, logout, account switch and ABA are separated.
4. The precise terminal/payload and phase write order, including failure after a remote success but before local publication.
5. The exact successful poll response condition used for durable release. A shape check can narrow local acceptance without inventing a wire field; it does not prove the external successful-absence semantics.
6. Per-alias completion and ACK independence, old-row behavior, deletion behavior, protected capacity, backward serialization and rollback.
7. Startup ordering relative to fresh poll dispatch, existing inference readiness, checked Session enumeration and unavailable history. Running/history unavailable remain Abandon.
8. A causal oracle crossing the shipping callbacks, actual FileAutomationStore pressure, owner reconstruction and exact outbound request bodies; mock-service limits remain explicit.

The old ACK-spec source paragraph describing api2.cursor.sh is historical. If touched in a later spec repair, label it as that correction's baseline or update the current-H source statement; do not use it to infer a new service owner.

### 8.2 Unresolved service facts, not local implementation facts

| External fact needed for a stronger claim | Why the current source is insufficient |
| --- | --- |
| Actual deployed Sand endpoint owner/revision and route contract | Configured origin/path is a client selection, not deployment provenance; no live/API authority was queried here |
| Stable principal carried by or associated with the inference credential | Current parsed credential is token+expiry only; no checked subject/account response field is owned by HostAuth |
| Relation between native account slot and Sand principal | Code explicitly separates the credential domains; equal endpoint origin is not an identity equivalence proof |
| Duplicate completion behavior after response loss | The client retries the same payload, but no deployed idempotency/conflicting-payload rule is proven |
| Repeated ACK behavior for already retired or unknown IDs | Current client retries; absence of a server error in a mock is not service evidence |
| Meaning and completeness of successful poll `events` absence | No explicit per-ID ACK receipt is checked; pagination/filter/lease/status behavior is unknown |
| Cross-account run-ID namespace and authorization | Fire envelope lacks account identity; current transport performs no credential-context comparison |
| Revocation and in-flight request semantics | Local epoch/tuple checks can refuse publication after change, but cannot revoke a request already committed remotely |
| Remote retention/TTL and reconciliation after long outage | Cannot be derived from the local 20-row display/history policy |

These unknowns need not be converted into an invented new protocol to implement a conservative local client improvement. They do prohibit claiming deployed end-to-end settlement or guaranteed liveness. A future real-service receipt must be bound to the exact endpoint/revision/account context and actual response behavior, not an independent reconstructed server or a loopback fixture.

## 9. Existing valid assertions and meaningful future oracles

### 9.1 Existing source assertions to retain

`fire_runtime_never_acks_unconfirmed_completion_or_redispatches_redelivery` checks 401, 404, 409 and lost completion response; each retains the same completion, sends no ACK until later success, and dispatches once. It is source evidence, not an execution result from this investigation. [T01:467–507]

`fire_runtime_polls_delivers_reports_and_acks_on_next_poll` proves the intended local separation of completion and next-poll ACK under fixture responses. [T01:185–230]

`explicitly_settled_history_replays_terminal_success_or_failure_after_reopen` starts with Running/Abandon, explicitly settles the actual existing DB/history owners, reopens, reports both primary and alias from known terminal history with zero executions, and sends the identical ACK list twice while returned IDs remain present. This is direct existing evidence that ACK retry is already intended client policy. Preserve it. [T01:1003–1077]

The native transport fixture tests validate origin refusal, redirect refusal, checked native completion response and rejection of an in-flight poll response after account file replacement. Preserve them as native-adapter evidence; do not call them shipping Sand account proof. [T02:61–137]

The earlier report's valid fresh-admission, checked-history, unresolved-tool/terminal custody, legacy-slot Abandon and per-alias terminal replay oracles remain in force. The source files used by those Automation oracles are unchanged at this H. None was executed here, and the incoming media task's execution status is not evaluated here.

### 9.2 Causal oracle for the proposed durable phases

A meaningful new oracle would use the existing production callback/Runtime/Session/FileAutomationStore composition and a controlled backend transport. It should not pass by testing an unused serialization helper.

The principal scenario is still: accept P and coalesced A through actual backend event dispatch; execute once; safely settle the owned local run; make A completion unconfirmed; complete enough distinct manual runs through the real owner to pressure ordinary history; drop/reopen all relevant owners; redeliver A; assert no second executor invocation and no fresh side-effect nonce/turn. A's exact retained terminal/payload must remain reachable. Then allow completion success and ACK according to the existing policy, assert exact per-ID phase release, and show that ordinary history can again prune the released row. P success must not release A. This is the original defect's causal test, not arbitrary old-history replay protection.

Add the two distinguishable crash windows:

- Reopen after completion success but before local Reported publication: preserve completion owed, never ACK from volatile success, retry the exact frozen completion, execute zero additional turns.
- Reopen after ACK may have succeeded but before local release: restore Reported, send the same ACK through the existing poll operation, do not send a fresh completion or execute; retain on failed/returned-ID poll, release only on the specified checked response under matching owner scope.

Identity assertions must use the **same credential snapshot that the production request consumes**. Test account change before request, while response is in flight, and before local phase/release publication; same-account token refresh; permanent old runtime retirement with late callback; A→B→A with distinct owner lifetimes; restart under a different Session account; missing identity; expired credential and later legitimate recovery. A fake `principal` string supplied only to a helper while real Reqwest still fetches an unrelated token would not be a valid oracle.

Parser assertions should show malformed successful poll responses retain custody, if that narrowing is adopted. Unknown completion status codes retain the current no-ACK rule. Restore order must suppress fresh redispatch before the durable obligations are visible to the backend owner. Local write failures at each phase must keep the earlier phase/pin; release of one alias must not modify another. Running/history-unavailable/unknown-effect cases must continue to exercise Abandon and zero completion/ACK.

All such tests and shipping compile/wiring verification remain future exact-H GitHub Actions work under the governing project contract. No source-only report can mark them passing.

## 10. Bounded implementation paths and decision boundary

The possible change surface is the existing Automation row/storage parser/retention, Session-admitted Runtime façade, actual backend fire state machine, lifecycle/production callbacks, shipping Host composition, and a small HostAuth/Reqwest internal interface if the actual principal contract can be established. No new standalone outbox file, general scheduler, account registry, experimental native-service cutover, or broad refactor is needed to describe the bounded behavior.

Specific files for an integrator's later independently reviewed patch are:

- `source/host/src/automations/automation.rs` and `automation_store.rs`: optional but validated custody representation, phase/payload preservation, exact-ID mutation and retention.
- `source/host/src/extensions/transcript/automation_runtime.rs` and the necessary admitted run-path interface: reuse Session identity, serialize the existing storage boundary and carry original backend custody through actual acceptance/terminal settlement.
- `source/host/src/extensions/session/agent_session.rs` / existing Session façade only if a narrow forwarding method is necessary; preserve incoming H71 Session work.
- `source/host/src/extensions/automations/sand_automation_fire_consumer.rs`, `production.rs`, `production_lifecycle.rs`, `backend_transport.rs`: restored state, exact phase callbacks, lifetime binding and checked current response.
- `source/host/src/extensions/auth/auth_service.rs` / `extension.rs`: only the minimal actual-credential context interface if justified; preserve all renewal worker behavior and other existing subscribers.
- `source/host/src/host_production_extensions.rs` and current `source/host/app/src/main.rs`: real composition/callback plumbing; do not overwrite incoming media work.
- Existing focused Automation/backend/App contract files for causal oracles, plus relevant Auth interface tests if changed; existing spec/task repairs first, executable work only through the authorized workflow later.

The missing Sand principal is a real design dependency. It is not solved by renaming the Session account ID, logging a bearer hash, or activating the unused native transport. The strongest currently supportable bounded outcome is local custody plus exact continuation of the existing client policy under an explicit supported owner binding. The parent should decide scope only after rereading the cited owner contracts; this report does not accept either candidate or authorize broadened service guarantees.

## 11. Exact source identities and actual read extent

### 11.1 Governing contracts, unchanged from the preceding read

| Ref | Path | H blob | Read extent/classification |
| --- | --- | --- | --- |
| S01 | `AGENTS.md` | `e4682748087759b2d4bb2b3a47916b6148e8a6f2` | Prior current root instruction read through end; identity rechecked at H71 |
| S02 | `projects/openclaw-rust/SOURCE_OF_TRUTH.md` | `3748d734b10c9ed5b2d948467fbaa9483f1dd306` | Prior 1–85,453–510 and headings; unchanged identity, not full body |
| S03 | `docs/specs/openclaw-rust-equivalence-migration.md` | `69c9667af2328e20a1fba09ebf80c5aab4d5c336` | Prior full §17 671–720 plus 170–198,347–356,371–395,414–427; unchanged identity |
| S04 | `projects/openclaw-rust/docs/specs/cron-session-reaper.md` | `b0a62da2cc86b5479047458019bbb3adfc2b16d1` | Prior full 1–1475; unchanged contract identity reused |
| S05 | `docs/specs/sand-completion-ack-safety.md` | `cbe3ceeb072ee9756235ac926f0c09021448ab8c` | Full 1–60 reread at H71; historical baseline statements remain historical |
| S06 | `docs/specs/fabushi-cursor-service-independence.md` | `b857b20ee4fbf92610caac7ae8b19034e559f53f` | Prior bounded 31–55, shipping/native distinction; identity rechecked |
| S07 | `projects/openclaw-rust/docs/specs/agent-loop-runner.md` | `254d7c375ceb7dab3ec008aa7b859ea395e04745` | Prior full 1–190; unchanged identity, no new Runner acceptance |
| S08 | `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md` | `1f0ecfb239f840df5e2ce5237029a78d945b4b9e` | Prior current 1–381; historical remainder not full-read; unchanged identity |
| S09 | `projects/openclaw-rust/module-map.md` | `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3` | Prior 1–64 and M19 row at 80; unchanged identity, navigation only |

### 11.2 Host auth and shipping backend

| Ref | Path | H blob | Read extent/classification |
| --- | --- | --- | --- |
| A01 | `source/host/src/extensions/auth/auth_service.rs` | `56cc268d11369f3d941bc1a5f007d8c97c8b82c4` | Full production 1–548 at H71; tests only initial 549–585 glimpse/search, not full test audit |
| A02 | `source/host/src/extensions/auth/credential_renewer.rs` | `911bab17bc9c675f861b49194734b78546b213dd` | Bounded 1–245,269–410; token/parser/origin/source/hooks/epoch/exit-guard dependency read; no whole worker/test claim |
| A03 | `source/host/src/extensions/auth/extension.rs` | `afc6c89fc2d39d17b5cf20ea622cdabe639261fe` | Full 1–132 at H71 |
| A04 | `source/host/src/extensions/auth/user_full_name_service.rs` | `a1bd19df957da3b55a49e3abd0771cedabb1ad23` | Full 1–148 at H71 |
| A05 | `source/host/src/sand_user_identity.rs` | `67eeb1da3c89a7d0e0740e44d7eabcb272f7b9b2` | Full 1–13 at H71 |
| A06 | `source/host/src/cursor_backend.rs` | `4b48a00341229a2500fe444293455339b6e42552` | Bounded 40–84,110–148,506–580 and principal/cache searches; not full transport |
| B01 | `source/host/src/extensions/automations/backend_transport.rs` | `6dc4477c72a38179d71ab33013c9b7e2c3b89991` | Full 1–101 at H71 |
| B02 | `source/host/src/extensions/automations/production_lifecycle.rs` | `acc5d67aef35ede93a0f87e317f966013ee2eb8b` | Full 1–494 at H71 |
| B03 | `source/host/src/host_production_extensions.rs` | `36b5dbac27d346f883150df179372034e0bea223` | Bounded 1260–1345 plus exact start/stop searches; unchanged from prior owner read |
| B04 | `source/host/src/extensions/automations/sand_automation_fire_consumer.rs` | `7d5f3e50f3a20aa5f142fdfa029cdcc6d40e06c9` | Full 1–366 at H71 |
| B05 | `source/host/src/extensions/automations/native_backend_transport.rs` | `9e7a811165610e81719ae26fcf4513a75088074c` | Full 1–119 at H71; separate nonshipping adapter |
| B06 | `source/host/src/extensions/automations/production.rs` | `3927e9736bebd27302cd9c4c2139b583e80b7aa4` | Full 1–310 at H71 |
| B07 | `source/host/src/extensions/automations/fire_delivery.rs` | `17a4466130d516d132db59f2ded772bead34935b` | Prior full 1–541 unchanged; 1–242 reread at H71 |

### 11.3 Actual account/Coordinator/Session composition

| Ref | Path | H blob | Read extent/classification |
| --- | --- | --- | --- |
| I01 | `source/product/fabushi/fabushi-account-service.ts` | `aa2358143079451cfa32b4c4dd64a148e20dfb2d` | Bounded 1–260,346–493,540–667; account epoch/status/native credential/refresh/logout, not full remote-computer flows |
| I02 | `source/product/fabushi/fabushi-account-policy.ts` | `31da18303e7a482ed5199e60936f601c0ada0af1` | Bounded 1–84; stable authId and device-scope policy only |
| I03 | `source/product/fabushi/fabushi-account-adapters.ts` | `b2e0fa5a88d6d0f19f7637ba62f79c9e4fc6cf0e` | Full 1–215 at H71 |
| I04 | `source/electron-main/coordinator/production-provider.ts` | `8436339d3e2bfdf05f1001f284dfadee72ecf8ef` | Bounded 475–610 and localHumanId search; actual processConfig/account subscription/main-data capture |
| I05 | `source/electron-main/coordinator/coordinator-account-runtime.ts` | `84e0e0ee6f83c0932a66ec4f89167b32e3f0e4ef` | Full 1–579 at H71, read in overlapping ranges |
| I06 | `source/node-agent-coordinator/src/main.rs` | `7fb056fc384fa705502a51a34cb7d94dc21fd358` | Bounded 1320–1405,1518–1556,2960–2998 plus host-generation/child searches; not full Coordinator |
| I07 | `source/node-agent-coordinator/src/carrier.rs` | `4e78824385930e66e145e5eacc737319b411675b` | Bounded 1–85; bootstrap/process config |
| I08 | `source/host/app/src/main.rs` | `7ec9bd8a22fd595f63cb3f4b632673e298294dfa` | Bounded 3995–4200,5980–6070,16115–16165,17630–17855,17940–18045 and exact callback/startup searches; no full App/media review |
| I09 | `source/host/src/extensions/session/production.rs` | `c5912cab4f95d3a75fc13cb264a21b1deef9a733` | Bounded 300–465,4480–4525 plus identity/shutdown method searches at H71; prior-H cron settlement ranges are not relabeled current full read |
| I10 | `source/host/src/extensions/session/native_messaging.rs` | `08e0921e9826006e109ed4253d518fad8bf4c68c` | Bounded 85–166,285–430,960–1065 and credential-call searches; no live identity verification or full messaging review |
| I11 | `source/electron-main/account/cursor-auth-wiring.ts` | `21cea3702d46d78002ed550786c0ab0c83ef830a` | Bounded 1–116; status delivery and runtime observation only |
| I12 | `source/electron-main/account/env-descriptor-account-binding.ts` | `7584b45fd36b5298b11023bacc09d39e1ce5150c` | Full 1–82; local descriptor/account hash binding, not Sand principal |
| I13 | `source/electron-main/account/production-account-authorization.ts` | `10ed5490a4324eba60957fdaf4edd2a46aac769d` | Full 1–69; existing descriptor/devbox authorization |
| I14 | `source/electron-main/coordinator/account-transition-cleanup.ts` | `9a5d5701ceefbeb137b407a8891db06737577a8f` | Full 1–74; account transition cleanup only |
| I15 | `source/electron-main/coordinator/production-root-provider.ts` | `4c5a4f73315994da7258d1d605ba4f5a4d438605` | Bounded 1–135; actual account transition port composition |

### 11.4 Existing persistence and oracle dependencies

| Ref | Path | H blob | Read extent/classification |
| --- | --- | --- | --- |
| R01 | `source/host/src/extensions/transcript/automation_runtime.rs` | `044f53c8eb94924168cc7dd953f3732cd70fa668` | H71 1–250,286–405,535–606,1260–1310; prior unchanged 301–494,571–654,773–1085,1152–1185,1268–1310 reused |
| R02 | `source/host/src/extensions/transcript/automation_event_fires.rs` | `0cab169c66da397a9bb2777995c67178461edc43` | Prior full 1–337; H71 identity rechecked, no fresh full read |
| R03 | `source/host/src/automations/automation_store.rs` | `57f39e545e525a36de3c7e9830970623862ee1fd` | Prior full 1–515 unchanged; H71 224–359,410–515 reread |
| R04 | `source/host/src/automations/automation.rs` | `4b27d9f5a7c4174a26e1244c50bef683d2c01482` | Prior full 1–87; H71 identity rechecked |
| R05 | `source/host/src/extensions/session/agent_session.rs` | `0a54da8349bc3edc9ef21c5f1c1862e13a9e3039` | Prior 875–935 history mutation façade; H71 identity rechecked |
| R06 | `source/host/src/extensions/transcript/automation_run_path.rs` | `a80ca919c1d05c30be29ad60e109f3a54f324e19` | Prior 1–861 settlement/admission read; unchanged H71 identity; no new full prompt-helper audit |
| T01 | `source/host/tests/automations_backend_contract.rs` | `45174d8010a61d631bd4d605ecc1df515a619bbc` | H71 150–242,450–522,1000–1077; named relevant tests fully read, rest searched; no execution |
| T02 | `source/host/tests/native_automation_transport_contract.rs` | `1a8bf2e109c9b067349d123ca7c61801c9e38cd5` | Full 1–137 at H71; loopback fixture source only, no execution |

Search-only inventory covered actual renewal subscribers and fire/completion/ACK call sites in current source. Those search results are navigation, not full-read claims for every neighboring owner. A broad tracked-text search also surfaced historical repository dossiers/evidence; no historical reconstructed-server assertion is used as deployed authority, and no private current peer report was opened.

## 12. Handoff status

The requested identity, callback, startup-recovery and ACK dependencies are resolved as source facts to the extent possible without an external deployed protocol. The available local account slot and lifetime owners are concrete; the shipping Sand principal/context interface and service ACK evidence are genuinely absent from the inspected path. The report supplies the narrow internal seams and conditional retry argument without inventing a protocol or promoting pin-only retention into complete recovery.

No repository file, runtime state or external service was changed. No validation was run. This report is the only new artifact of this follow-up. The parent retains responsibility for scope selection, spec repair before code, independent source reread, integration, review, exact-H Actions, real-service evidence where required, and all OCRM acceptance gates.
