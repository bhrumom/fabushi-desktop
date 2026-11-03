# Media persistence: current-consumer source audit

**Disposition: read-only audit complete; a bounded follow-on contract is ready to author after the current shared LFA repair converges. No implementation or runtime acceptance is claimed.**

The production gap is present in the assigned live source and remains in the frozen R4 candidate: once local or Box bytes have been acquired, a Session admission or storage error is erased into an optional result. The actual App attachment resolver can then try Box or return the original URL as a successful resolution. The narrow correction is to preserve that error through the existing messaging path, while explicitly preserving Browser and Computer's best-effort screenshot APIs.

This report is independent of the R4 blind review/publication work. It does not authorize changing the frozen candidate or advancing dependent production before the known shared contract fix. BR denotes the bounded reader; it is not evidence that this persistence path has been closed.

## 1. Identity, authority, and method

| Item | Exact identity |
| --- | --- |
| Repository / branch | bhrumom/fabushi-desktop / codex/openclaw-rust-fusion-chat-20261101-writer |
| Exact source base | 3438eb5bac755c399c9ec686414bbb4c0816888d |
| Base tree | b544c1d377ca82618cdadcf5106cb3c10d82411b |
| Live H supplied with this assignment | a27a3a6daec5377a56f61221b34ac3f3c8c4005e |
| Assigned H tree, read from its commit object | a15cb66584e1b51e82ae97098491657fe629bf59 |
| Frozen R4 candidate tree | 76e6c26d754179ee5af447ecec689a2de844b476 |
| R4 manifest | /workspace/scratch/7d27ea239237/media-admission-r4-manifest.json |
| R4 snapshot | /workspace/scratch/7d27ea239237/media-admission-r4 |
| Root AGENTS blob | e4682748087759b2d4bb2b3a47916b6148e8a6f2 |
| SOURCE_OF_TRUTH blob | 3748d734b10c9ed5b2d948467fbaa9483f1dd306 |
| Main Spec blob, operative §17 | 69c9667af2328e20a1fba09ebf80c5aab4d5c336 |

The applied review effort remains xhigh and the configured model is retained. I used exact Git-object reads, repository-wide symbol census, immutable-snapshot source reads, and byte/blob identity reads. Base-to-assigned-H Git diff lists only nine added media-br-fua-source-recovery evidence paths, with no source change. Candidate identity comes from the frozen manifest; the reviewed R4 App, fixture, bridge, contract, workflow, and capture-script blob identities below were independently reproduced from the snapshot bytes.

No product/parser/test/build/lint/schema/capture/generator was executed locally or on htch. No mutable working-tree source or other reviewer report was read. The earlier /tmp/fabushi-647e-pairing/media-persistence-next-contract.md was supporting context only; the caller and owner conclusions here were checked against actual source. This Markdown report is the only file written by this audit.

### Source citation key

All repository paths below are relative to the repository root. References such as [A:618–636] mean the exact blob in this table and its source lines, not a mutable checkout. R4 line numbers are used for changed files; unchanged files use the exact base object.

| Key | Path | Exact blob |
| --- | --- | --- |
| A | source/host/app/src/main.rs, R4 | bf65f7ee95cfe1dd76568383eb7f81fe13f06b08 |
| F | source/host/app/src/tests/local_media_file_url_tests.rs, R4 | 5682e951f1eac8f107efd3a3948b0748870912bd |
| E | source/host/src/runner/tools/send_message_encoding.rs | 22e2700f4ea2f894b92f3233d0a399a296f6fa5b |
| P | source/host/src/attachment_paths.rs | c66a2d1ce8dfb70532a16857a0fe02d31788ca62 |
| S | source/host/src/extensions/session/production.rs | f585fae13720e08f34323bed82759253590a3753 |
| D | source/host/src/extensions/session/agent_db.rs | 78740768f7f1b0307dfd6756c4d50c368a91b721 |
| T | source/host/src/runner/tools/send_message_tool.rs, R4 | 9e12d7d2b457715f0fac168e5e4f262e6ef15985 |
| M | source/host/src/runner/tools/sand_agent_management_tools.rs, R4 | 22e7f260ca0d23250802af3d862cf836b21d675b |
| C | source/host/tests/send_message_tool_contract.rs, R4 | 8c334d36e4b507842a38509b945956fe5085beb7 |
| B | source/host/src/runner/tools/sand_browser_tools.rs | 191eda1bfb8c2958716a1c4635b277d09f87495c |
| K | source/host/src/runner/tools/sand_computer_tool.rs | 2de35a7a5a5db90f5e53c040318b849d7fdeccef |
| X | source/host/src/runner/host_computer_tool_dependencies.rs | 870ac966147cce88a688a526835f7bd6c68d716b |
| H | source/host/src/extensions/forever_box/host_box.rs | c841d53cc412e8813ce1904cfb90e329fff9dd30 |
| I | source/host/app/src/tests/channel_host_lane_tests.rs | ff4ebba71eae35f4be98a695654a0d33b84f1818 |
| W | .github/workflows/rust-desktop-runtime.yml, R4 | 9419b5467e6882a7aeff16c91aa97317a4e001eb |
| N | projects/openclaw-rust/scripts/capture_media_cleanup.py, R4 | dfc83dd491046de7708643ce48b0fbebd74a19bb |
| Q | projects/openclaw-rust/scripts/capture_portable_contracts.py, R4 | dce650d0425bc35db032234753cf5e4ec298896c |

The exact base App blob is fb7d24ab8996f8c444eb851d4e5f8c41e0627ff6. R4 App SHA256 is 6010ec87aaef917a203256a8d917df170a454bcf440e7c483a1d3f3896d16e07; the new App fixture SHA256 is f2097d754193bd21cc1b7911829f37ba395f59750680e1e560d682e6a8f49d80.

## 2. Complete direct-caller census

Repository-wide exact-base Rust symbol search, followed by the changed R4 source reads, finds **five calls to the private persistence helper, two production consumers of the private App resolver, one production invocation plus four contract invocations of the generic Box resolver, and one production invocation plus three contract invocations of the filesystem writer**. Imports and declarations are excluded from those counts.

### All five persist_runner_media_bytes callers

| Caller | R4 App line; exact-base line | Current effect of helper None | Required migration |
| --- | --- | --- | --- |
| resolve_production_attachment_source, local-read branch | 665; 662 | Falls through to Box status/eligibility even after bytes were read; can ultimately return original URL successfully | Propagate helper Err immediately; managed URL on Ok |
| Same resolver, Box image callback | 685; 682 | Generic Box resolver returns None, then App returns original URL | Return helper Result through generic Box resolver |
| Same resolver, Box attachment callback | 692; 689 | Same collapse as image callback | Same Result propagation |
| BrowserPersistImageCallback closure | 11880; 11875 | Logs admission-or-persist-failed and returns unit; browser output continues | Change the failure predicate to is_err(); preserve unit callback, PNG filter, redacted log, and tool success behavior |
| ComputerPersistImageCallback closure | 11983; 11978 | Returns None; completed computer action retains its screenshot bytes without adding a saved path | Deliberately convert Result to Option at this adapter with ok(); preserve WebP filter and best-effort API |

The helper is declared at [A:618–637]. Its operation already returns Result<String, String> inside Session's admitted filesystem boundary; only the final ok() erases admission, storage, and file-URL construction errors. The proposed change removes this erasure at the shared helper, not from the intentionally optional Computer API.

### Resolver consumers and remaining invocation sites

| Function / contract | Exact source location | Effect / migration obligation |
| --- | --- | --- |
| ProductionSendMessageSink::resolve_attachment_source | [A:880–966], resolver call 899; base call 896 | Already maps String error to ProviderSessionError::Tool. Only a successful owned path is added to OutgoingMediaCustody and durable media admission |
| ProductionAgentManagementSink::resolve_image_source | [A:1175–1191], resolver call 1180; base call 1176 | Already maps resolver error to Tool; preserves image alt. Discards returned owned_path, so it has no SendMessage-style rollback custody |
| Production construction of these sinks | [A:11554–11589] | Uses actual worker Session and ForeverBox owners; SendMessage wraps the real sink in CountingSendMessageSink |
| resolve_box_media_attachment production call | [A:681–700]; base 678–697 | Only production invocation; both persistence closures must adopt Result and the outer call must propagate Err before original-URL fallback |
| Existing Box image control | [C:480–487] | /workspace/a.png with desktop and bytes; migrate Some(image URL) to Ok(Some(image URL)) |
| Existing Box attachment control | [C:490–497] | /workspace/movie.mp4; migrate both callbacks and expected media URL |
| Existing outside-root refusal | [C:499–505] | /tmp path; migrate None to Ok(None); callbacks must remain uncalled |
| Existing no-desktop refusal | [C:506–512] | /workspace path without desktop; same Ok(None) compatibility |
| persist_agent_media_bytes production call | [A:631] | Sole production invocation of this writer. Existing io::Result remains unchanged |
| Direct writer contract calls | [C:575,581]; openclaw_crash_guard_media_contract.rs:303 | Positive Image/Attachment storage and OCRM-MEDIA-O09 tests; no API migration required |

The positive writer test at [C:570–596] verifies roots, names, bytes, and absence of a temporary file on a successful image write. Its existing name containing “atomic” does not supply failure, crash, or durability evidence.

## 3. Narrow Result contract

Recommend retaining the existing String error boundary: private helper returns **Result<String, String>**; generic resolve_box_media_attachment returns **Result<Option<String>, String>**, with both persistence callbacks returning Result<String, String>. Keep Download as FnOnce(&str) -> Option<Vec<u8>> and keep both persistence callbacks FnOnce. A new error taxonomy, runtime, store, or permission owner is unnecessary for this closure.

| State reached | Result / next action |
| --- | --- |
| Shared local-file URL admission rejects the input | Existing invalid-local-media Tool error; no materialization or Box fallback |
| Input is not a local file URL | Existing original URL and metadata behavior |
| Local read succeeds; Session/writer/URL conversion succeeds | Return managed URL and its owned path |
| Local read succeeds; persistence returns Err | Terminal resolver Err; no Box status, download, retry, or original-URL success |
| Local read fails, with any error kind | Preserve the current Box eligibility path exactly |
| Box path is outside the existing roots, desktop is absent, or Download returns None | Ok(None); App retains existing original-URL fallback |
| Box download yields bytes; selected persistence succeeds | Ok(Some(managed URL)) |
| Box download yields bytes; selected persistence fails | Err; no alternate persistence callback and no original-URL success |

The distinction is **before versus after bytes are acquired**, not NotFound versus other read errors. The current local branch is if let Ok(bytes) = fs::read(...), so every read error reaches the existing Box eligibility logic. Introducing a NotFound-only fallback would change compatibility beyond this contract. Likewise, the existing Box download closure maps all download errors to None; this proposal deliberately leaves that pre-bytes contract intact. [A:645–704]

Keep existing root checks (/workspace, /home, /root with exact-root or slash boundaries), desktop detection, extension-based Image/Attachment selection, source filename handling, shared URL admission, and original-URL metadata behavior. The generic helper calls only the selected persistence callback after its single download. Preserve that selection and do not add a retry/alternate branch. [E:217–267]

### Browser and Computer are explicit best-effort adapters

Browser's public alias is Arc<dyn Fn(&[u8], &str) + Send + Sync>, with no result channel. Its executor invokes the callback after it has read a successful screenshot, then base64-encodes those original bytes and renders the successful response; navigation notification follows normally. [B:620,667–672,849–875] The App accepts only image/png and logs a generic failure on persistence refusal. Its installation remains conditional on a non-group-member turn. [A:11875–11895,11971–11974] Preserve all of this; widening Browser's callback to Result would introduce a new failure after an action has already occurred.

Computer's alias is Arc<dyn Fn(&[u8], &str) -> Option<String> + Send + Sync>. persist_computer_screenshot only updates screenshot_path when the callback returns Some; None returns Ok and leaves the existing result intact. [K:439–469] The production executor has already executed the computer action before invoking persistence; it then runs its post-action callback and returns the result. [X:649–697] Keep the App's image/webp filter and non-group installation. An intentional ok() at [A:11983–11989] preserves that API and prevents a persistence failure from becoming an apparent failed action/replay trigger. [A:12150–12153]

## 4. Ownership, error effects, and recovery limits

The existing owner stack stays intact:

- Session owns the per-agent filesystem/deletion lock and durable admission check. The admitted wrapper executes the check while holding the same lock, then calls the operation. Its public admission facade is non-reentrant; do not call it recursively inside that operation. [S:2290–2343]
- The writer chooses assets or attachments, creates the directory, writes a UUID temporary file, and renames to a UUID target. Its existing io::Result is the storage error source. [P:236–255]
- SendMessage creates OutgoingMediaCustody only after receiving an owned path. It records durable media admission and absorbs that custody into the existing per-tool-call pending entry. A later message-building error invokes abort_attachment_custody through the existing bridge; it can roll back earlier paths whose custody it actually holds. [A:899–966,969–996; T:260–276,405–410]
- AgentManagement resolves its images before send_to_agent_with_reply. A propagated persistence error prevents the actual outward messaging call and wake. It has no abort method and ignores owned_path, so an earlier successfully persisted image is not rolled back if a later image's persistence fails. Preserve that limitation in acceptance language. LFA's all-URL preflight prevents a later invalid-URL side effect; it does not make persistence of the image batch transactional. [A:1180–1189; M:55–65,93–105]

The controlled write obstacle described below fails at create_dir_all, before the temporary write. That permits a strong no-new-file oracle for that exact case. A general persistence error does **not** prove that nothing was written: fs::write can leave a temporary file on error, rename failure cleanup is best-effort, and conversion to a file URL occurs after the final target exists. The Result change is not a cleanup guarantee. [P:246–258; A:631–634]

Pending-media recovery only reconciles the existing durable work identity and recorded paths. A valid settlement receipt retains media; without one it verifies recorded paths against the agent roots, refuses uncertain paths, rolls back known paths, and clears the matching fence only after successful reconciliation. It cannot discover every unrecorded temporary/target file from a failed writer. [S:1752–1889]

Keep unknown database commit outcomes separate. The media-settlement method currently maps a COMMIT failure after an ignored ROLLBACK attempt into its ordinary error path; App send error handling aborts pending custody. This source observation warrants a separate outcome/ownership contract, not a new claim in this bounded patch. Nothing here establishes whether a failed COMMIT was durable, authorizes deleting possibly committed media, or proves crash safety. [D:4899–4917; A:1036–1095]

## 5. Actual App failure oracle using the existing fixture

### Reuse and safe dependencies

LocalMediaAppFixture already constructs the real ProductionSessionWorkers, actual materialized Session, TranscriptManager/runtime, real ProductionSendMessageSink, and the first-party SendMessageToolBridge. ObservedProductionSendSink delegates actual resolution and sending; it records successful resolution only after the production resolver succeeds. It also records aborts, sends, real bytes, and media admission. Reuse these owners and observations. [F:76–159,161–241]

The fixture provides source(), entries(), admission(), agent_dir(), assert_custody_idle(), assert_no_host_request(), reopen(), and close(). Reopen disposes and reconstructs the actual owners against the same owned files. Drop external bridge/observer/sink Arcs and unsubscribe observation callbacks before reopening; otherwise an old owner can remain alive and weaken the fresh-owner claim. [F:227–321]

The current AgentManagement test defines bridge_for as a nested function at [F:597–622]. Lift that helper only within the test module so new cases reuse the same actual ProductionAgentManagementSink and ProductionAgentToAgentMessaging with the existing recording wake callback. Its positive control observes the real source transcript plus the selected image in the wake request; it does not execute a recipient Runner or prove external delivery. [F:624–732]

**Deterministic storage refusal:** on a fresh fixture, create an owned regular file at agent_dir()/assets for a PNG source, or at agent_dir()/attachments for a source with a non-image extension. Verify the source is readable and its bytes unchanged, and verify the obstacle is a regular file containing the expected marker. The real create_dir_all must refuse this shape before any temporary write. This avoids permission-bit/root-user assumptions, symlinks, arbitrary outside paths, quota simulation, or a fake persistence callback.

**Fixture trap:** media_files() assumes existing roots are directories and calls read_dir(...).expect(...). It will panic on this regular-file obstacle. Inspect the obstacle and the other media root directly during refusal, then call media_files() after removing the obstacle, or narrowly adapt this test helper to represent the obstacle separately. Never treat the obstacle itself as successfully persisted media. [F:259–275]

assert_invalid_source currently requires the LFA-specific error prefix. Add an appropriate persistence/admission error assertion; do not relabel a filesystem failure as invalid URL. For storage errors, assert Tool failure and the deterministic obstacle/preconditions without depending on an OS-specific error string. [F:330–338]

### Existing observation port for no fallback

Use fixture.deps().forever_box.box_().subscribe with an agent-filtered counter. HostBox::get_status always emits a status notification before returning, including an absent/no-desktop status; subscriptions are existing public API. Qualify the observation with one explicit get_status call, require one notification, then reset before the refusal. [H:25,56–68,103–111,165–194]

The fixture's idle owners use an unstarted ForeverBox and do not start unrelated Box-store polling. No ensure, VNC setup, network server, or new production test seam is needed. [I:182–183,221–264] A readable-source persistence refusal must leave this counter at zero. Since the actual resolver's sole download invocation follows get_status, this is a causal assertion that its Box fallback/download path was not reached. It is not a direct measurement of every network or filesystem syscall. [A:678–684]

For an affirmative compatibility control, make an owned directory with a valid native file URL, confirm fs::read fails with an error kind other than NotFound, then invoke the actual App resolver/sink. Require its existing original-URL result and one observed Box-status evaluation. Also retain a missing-file control. With this idle fixture, no desktop is available, so this proves retention of the pre-read-error eligibility path, not successful remote downloading.

### Proposed exact App cases

All five names below belong in source/host/app/src/tests/local_media_file_url_tests.rs and therefore have the prefix tests::local_media_file_url_tests::. They are proposed names, not tests already implemented or executed.

| Proposed test name | Required causal oracle |
| --- | --- |
| media_persistence_app_refuses_writes_then_repairs_across_reopen | Exercise PNG/assets and standalone attachment/attachments on separate fresh fixtures. Repeated failed real SendMessage calls return Tool errors, produce no successful-resolution observation, send count zero, no Host-lane request, unchanged transcript/admission, no pending custody, zero Box fallback notifications, and unchanged source/obstacle bytes. Remove only the owned obstacle; a fresh call succeeds with a new managed URL, exact bytes, recorded admission then settled custody, and one actual transcript message. Drop owners, reopen, and verify the committed message/path/bytes persist |
| media_persistence_app_rolls_back_prior_media_on_later_write_refusal | First image resolves successfully into assets and its real pending/durable custody is observed. A second readable, admitted source with a non-image extension targets blocked attachments. The extension-based branch is intentional; it can contain the same PNG bytes. Require a terminal failure, no send, one abort, rollback of the earlier known owned path, released fence/pending custody, unchanged sources/transcript, and zero Box fallback. This proves rollback of earlier tracked media, not cleanup after a partial failed write |
| media_persistence_app_agent_management_refuses_write_before_wake | Fail persistence of the first image through the actual AgentManagement bridge. Require source/target transcripts and admissions unchanged, no recorded agent message, zero wake requests, and zero Box fallback. Repair, use a fresh call, observe the real source entry and one wake carrying the managed image, then reopen and verify bytes/source entry. Do not claim a later-image failure rolls back earlier AgentManagement media |
| media_persistence_app_keeps_pre_read_errors_box_eligible | Use owned missing-file and directory-read-error controls. Qualify the latter as non-NotFound, invoke the actual resolver/sink, and require the existing original-URL result plus observed Box-status evaluation. No acquired-bytes persistence is attempted; this pins the compatibility fence |
| media_persistence_app_admission_refusal_is_terminal_before_box_fallback | Use existing Session begin_agent_delete/end_agent_delete on the fixture's materialized agent; do not delete its directory. Invoke the actual AgentManagement resolution/bridge with readable local bytes so the helper's admitted mutation is the refusal boundary. Require the admission error, no write/send/wake/fallback, then clear the fence and exercise a fresh successful control. SendMessage alone is insufficient to prove this helper path because it has an earlier admission preflight |

The existing App refusal tests already use fixture.deps().events.subscribe() and assert an empty receiver. Reuse that same real GatewayEventHub observation around the bounded failed call, alongside transcript/admission, pending custody, sends/aborts, Host-lane requests, and AgentManagement wakes. [F:385,413,428,452] This establishes absence on the observed hub during the invocation; it is not a claim about every process-level event.

## 6. Generic and callback contract coverage

Migrate all four existing Box resolver calls listed in section 2. Preserve their positive metadata and root/desktop fences. Add one focused test, **box_media_persistence_refusal_is_terminal_after_download**, covering both image and ordinary-media persistence errors with exact error preservation, one download, one selected persistence callback, and zero alternate callback calls. Include a Download=None control with neither persistence callback invoked and Ok(None). Existing root/no-desktop cases should use fail-on-call callbacks to prove the early fence. These closures observe the real generic helper; they do not substitute for the actual App tests.

The existing Browser test **production_browser_executor_uses_only_host_box_port**, source/host/tests/sand_browser_tools_contract.rs:519–608, verifies the production executor's unit callback, original screenshot bytes, rendered image, and navigation notification. Preserve it and the public callback type.

The existing Computer test **production_computer_executor_uses_host_box_transport_and_persists_screenshot**, source/host/tests/computer_tool_projection_contract.rs:344–390, verifies one real executor transport invocation, callback bytes/MIME, raw screenshot, saved path, and post-action. Extend that same named contract with a callback returning None and require successful result/raw screenshot, no added path, one transport call, and the post-action callback. The lower-level **outcome_render_and_screenshot_persistence_match_frozen_contract**, lines 151–175, is additional coverage. Neither test alone proves execution of the App's inline adapters; that binding is the source review plus actual App compilation, not a copied closure in a test.

## 7. Small authoring scope and exact selection touchpoints

### Production and tests

The production change requires only source/host/app/src/main.rs and source/host/src/runner/tools/send_message_encoding.rs. The App helper, local/Box propagation, and two screenshot adapters change together. The existing SendMessage/AgentManagement trait APIs, filesystem writer, Session locks/admission, transcript owner, Box transport, and custody owner require no production change.

Test edits belong in the existing send_message_tool_contract.rs and local_media_file_url_tests.rs. Extend computer_tool_projection_contract.rs only for the meaningful None-result control above; the Browser contract can remain unchanged. Reuse the existing fixture and lift its nested AgentManagement helper locally. Avoid a new fixture/runtime/storage service or a duplicate resolver.

Write the bounded contract and evidence/traceability in the project's established locations as a later, separately frozen batch. Distinguish terminal messaging refusal from writer cleanup, crash recovery, and durable commit closure. No success/status promotion follows from this audit.

### Selection and input binding

| Current location | Follow-on obligation |
| --- | --- |
| [N:182–200,457], SEND_MESSAGE_NAMES / send-message-consumers | Add the one new Box-persistence test; retain all 17 existing names. The existing exact-name and exact-summary oracle must remain strict [N:285–307] |
| [N:208–254], INPUTS | Add the changed send_message_encoding.rs, currently absent. Include attachment_paths.rs when claiming this writer dependency; if adding screenshot contract selections, bind their test files and the corresponding Browser/Computer source files |
| [Q:542–559], generated-send-message-contracts | Add the same new Box-persistence name; retain existing names and all three test targets |
| [Q:974–976] | With exactly one added SendMessage contract test, the required sorted target summaries become (13,0,0), (14,0,0), (18,0,0), replacing only the current 17 count. Retain uniqueness and complete required-name checks |
| [W:1640–1647,1806–1845], macOS recovery integration command and required list | The target is already selected; add the new required name to this independent list as well |
| [W:4160–4169], actual Linux App required list | Retain all six LFA cases and append the five proposed media_persistence App names. Preserve exact executable selection, one-passed/zero-failed/zero-ignored completion, and case completeness [W:4173–4185] |
| [W:3876–3969], actual App source inputs | App main, the fixture, bridge sources, Session owners, shared fixture, and send_message_encoding.rs are already bound. Add the new bounded contract and missing directly relied-on sources: attachment_paths.rs, HostBox/ForeverBox status source, and agent_to_agent_messaging.rs |
| Screenshot acceptance selections | If this batch claims the Browser/Computer executor controls were run, explicitly select the two named existing contracts above, preferably as exact cases through the existing capture machinery, and bind their inputs. These targets are absent from the current focused native/portable selections |

The general full Host cargo test at [W:260–263] is excluded for this writer branch's push and ocrm-config dispatch. Therefore its presence is not evidence that the focused lanes execute the Browser/Computer targets. Do not infer callback acceptance from that unrelated command. Preserve all existing shell, environment, source-identity, capture-exit, artifact, and oracle controls; append bounded selections without relaxing counts. Recompute any aggregate totals from the final frozen selections rather than carrying forward a stale count.

The current native captures execute Host contracts. The existing exact actual-App execution lane described above is Linux. Native Host success does not establish actual App behavior on macOS or Windows, packaged-platform behavior, or a real external Box/provider interaction.

## 8. Additional immutable dependency identities

These files were read as dependencies, not proposed production edits:

| Path | Exact base blob |
| --- | --- |
| source/host/src/extensions/session/agent_session.rs | 0a54da8349bc3edc9ef21c5f1c1862e13a9e3039 |
| source/host/src/extensions/transcript/production_runtime.rs | 1f336135cb1c521d16dc0d8ad9e42ef0cf6bccba |
| source/host/src/extensions/transcript/agent_to_agent_messaging.rs | e1b3769993c5086f3fbce654d431a42b9bfa1783 |
| source/host/src/extensions/forever_box/forever_box_service.rs | 2c549d54d607b567289a21e5c5a370f834bdbe37 |
| source/host/src/box/production.rs | 96aa8c1ccf7ca2c0f8a5085a11279d29c113706e |
| source/host/src/box/box_factory.rs | b0d446757dcb036f91a835f947db53b74f83aa45 |
| source/host/src/box/loopback_sand_box.rs | ac2fcd557f09eb3c5fce9e8d22001dd6600266d3 |
| source/host/src/box/shared_desktop_sand_box.rs | dfce60df4441cd96118d6a844e06fba48021f10d |
| source/host/tests/sand_browser_tools_contract.rs | 0a2153eb365375e331f504f5c4e80eae51c30fdc |
| source/host/tests/computer_tool_projection_contract.rs | ea466f28fdb74a5ff80cbc310d701e2c5ec6dd25 |
| source/host/tests/openclaw_crash_guard_media_contract.rs | d753d932b3b60a35230ea6c827033b7a349d9998 |

The Box chain remains HostBox::download_file (289–295) → ProductionBoxEnvironment (175–180) → existing composition (box_factory.rs:130–139) → Loopback download (530–547), with SharedDesktop delegating to that same inner owner (454–461). The loopback path retains its existing file-read admission and error mapping. This audit does not propose changing or independently duplicating that chain.

## 9. Convergence gate and remaining work

This follow-on is logically distinct from LFA's URL/native-path admission repair. It physically overlaps App main, the App fixture, SendMessage contracts, workflow/capture scripts, and project accounting. Consequently it must be authored from the converged post-R4 source, with a fresh manifest/diff and reviews; it must not be merged into or inferred from R4's frozen review identity.

Before production acceptance, the root task still needs the current shared repair's joined source reviews/publication and exact-H Actions evidence. The later persistence batch then needs its own complete caller migration, focused independent review, exact published-H source/test bindings, the named actual-App negative/repair/reopen controls, and preserved native/portable oracles. No execution or publication gate has been satisfied by this read-only audit.

Remaining explicit exclusions are partial-write/rename/URL-conversion cleanup, unknown-COMMIT settlement ownership, crash/fail-stop durability, AgentManagement multi-image rollback custody, actual remote Box download behavior, and packaged/native App acceptance beyond the executed lane. Each requires evidence at its existing owner boundary rather than a broader success claim attached to the narrow Result propagation change.
