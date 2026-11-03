# Generated HTML source authority: bounded producer and origin assessment

## Decision

**There is no existing generated-HTML source authority on the assessed shipping Rust path at H.** Shell can create an HTML file, and SendMessage can copy that file into Agent media, but no current owner captures an authoritative generation operation and carries its exact output bytes into staging. None of the inspected directories or capabilities supplies that missing grant. Setting `HostReadHtmlTrust::TempRoot` from a workspace/temp pathname, Agent ID, tool success, uploaded file, or JSON flag would invent authority.

The smallest useful extension belongs to the existing **Runner/Box execution owner, composed with the existing Session/Attachments owners**. It must capture a private output capability before a real producer runs, retain exact bytes and the captured lifetime, and make those bytes available to the actual `ProductionSendMessageSink`. It must not expose an operation that declares an arbitrary existing path or supplied buffer “generated.” A bounded output-capture mode on the real Shell execution path is one implementable first producer; ordinary Shell filesystem outputs remain unsupported until their independent filesystem-origin boundary is established. This is a proposed contract, not a callable capability present at H.

The pinned fs-safe implementation also has an important limit: **the actual `secure-temp-root` export does not perform Windows ACL admission**. Its Windows branch disables the descriptor repair adapter and bypasses the POSIX owner/mode predicate. The OpenClaw wrapper comment cannot supply that missing proof. Android's commented app-group policy likewise was not established by the inspected export. These limits must remain explicit in a Rust implementation decision.

## Subject, scope, and evidence

| Authority | Exact identity |
| --- | --- |
| Fabushi H | `871b033c5b9aabbe322cc7322af92648939db9b5` |
| H tree | `f641366eea64364f4d029487d04276c4c567790a` |
| H Host subtree | `59a5cb05d9e6754461be43589d037fd7f1d8c40c` |
| H Box executor subtree | `fc98ae5c40f709ff5c216447be3ddb0f2841cdb2` |
| H local executor subtree | `7e205d0af919cdd42b361ca14a193f071910d797` |
| Fixed OpenClaw U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| U tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| AGENTS | `e4682748087759b2d4bb2b3a47916b6148e8a6f2` |
| SOT / main spec / module map | `3748d734b10c9ed5b2d948467fbaa9483f1dd306` / `69c9667af2328e20a1fba09ebf80c5aab4d5c336` / `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3` |

The [source identity manifest](sandbox:/workspace/scratch/7d27ea239237/html-origin-boundary-inputs/source-identities.json) records each captured file's Git blob calculation, SHA-256, byte count, and line count. The [current semantic read index](sandbox:/workspace/scratch/7d27ea239237/html-origin-boundary-inputs/current-semantic-read-index.json) distinguishes bounded semantic reads from discovery/capture-only files. Complete files were captured for reproducibility; that is not a claim of complete semantic review. Exact H commit/tree responses and source subtree listings are retained in the same input directory.

H's commit object was not available in the local object database. Exact GitHub commit/tree responses established membership; matching immutable local blobs and the exact Host subtree were read where available. No mutable working-tree source was used. Prior fixed-U captures were copied only after matching their supplied Git blob identities. The prior bounded owner assessment was context, not evidence that its older H was this H. No MSU review reports were read.

This task performed source/identity reads and wrote this report and its input directory only. It did not execute product code, builds, tests, validators, generators, fixture/capture programs, htch work, commits, pushes, or Actions. Source/archive hashing and extracting selected regular archive members were data-only accounting. No governance, production, spec, ledger, or workflow changes were made.

## 1. Actual callable producer and SendMessage path

The following is the current path, not a proposed pipeline. Source IDs refer to the bounded read ledger below.

| Stage | Actual callable owner and bytes | Directory/admission consequence |
| --- | --- | --- |
| Turn composition | App constructs `ForeverBoxRunnerResourcePort::new(forever_box, agent_id)`, passes Box resources to the turn, and separately constructs `ProductionSendMessageSink` with captured Agent/session and Session/Box services. | The port captures an Agent string and a resource coordinator. The sink captures media/session custody. No generated-source capability connects them. [H01, H03, H10] |
| Agent-accessible file generation | `runner_box_tool_definitions` exposes **Shell and Read**. Shell defaults to `/workspace`, then passes command, cwd, tool-call ID, background request and blocking interval through `RunnerBoxResourcePort::execute_shell`. | Arbitrary commands can generate documents. There is no exposed Box Write tool and no generated-artifact result/grant in the Shell request or response. Approval/audit authorizes execution, not generated provenance. [H02, H03] |
| Box execution | `ForeverBoxRunnerResourcePort::execute_shell` uses its captured coordinator/accessor. The daemon validates the working directory then spawns the shell with pipes. Its `BoxExecRuntime` owns one workspace root and environment. | Local default root is `<app-data>/feature-host/runtime/workspace`, configurable by `SAND_BOX_WORKSPACE_ROOT`; daemon startup uses ordinary `create_dir_all`. Logical `/workspace` maps to that root. No private per-Agent/turn generated root is created or admitted. [H04, H05] |
| Actual produced bytes | Shell stdout/stderr are read as `Vec<u8>` then converted with `String::from_utf8_lossy` into `ProcessOutcome`. Files created by arbitrary shell commands stay in the chosen ordinary filesystem. | The current text result is not a byte-preserving document result or receipt. Process groups, cancellation and timeout supervision do not grant source identity to arbitrary resulting paths. [H05, H15] |
| Internal byte writes | `RunnerBoxWriteRequest` is used for generated images, browser-driver bootstrap, cloud transcript dumps and large MCP-output spill. It forwards path/data/tool ID through generic `execute_write`. | Known examples are `/workspace/assets/<filename>`, an owner-selected browser driver directory, JSONL dumps, and `.sand/tools/<UUID>.txt`. None is a general HTML producer. MCP spill bytes remain external tool content. The generic writer returns success, not an origin receipt. [H02, H03, H11] |
| User uploads | `upload_file_via_exec_daemon` creates the parent, writes a random `.part` through the same `execute_write`, then moves it into place. Shared desktop upload/download use a shared Box ID and ignore the supplied Agent argument. | A Box path or successful internal write cannot distinguish generated output from uploads. Generic upload and generic write must remain unable to mint a grant. [H06, H07] |
| Actual SendMessage source read | `resolve_production_attachment_source` tries a local URL/path with ordinary `fs::read`; on desktop fallback it calls `resolve_box_media_attachment`, downloads bytes and persists them. The latter accepts `/workspace`, `/home` or `/root`-shaped paths as Box locators. | These are read/delivery locators. They are not the secure OpenClaw temp root. The loopback download path performs its own host path resolution and direct read, rather than consuming a generated-output receipt. [H01, H07, H08] |
| Existing Session custody | `persist_runner_media_bytes` uses `with_agent_admitted_filesystem_mutation`; `persist_agent_media_bytes` publishes a UUID-named temporary file/rename under Agent media. The sink captures media admission/lifecycle revision and tracks owned paths. | These are valuable current admission and settlement owners. They establish the staged object's custody, not the origin of the source bytes. `ResolvedAttachmentSource` exposes only URL and filename. A later successful send cannot retroactively prove source generation. [H01, H08, H09] |

The App also has a genuine image producer that persists under `<agent_dir>/generated-images` before its image execution flow. That owner is image-specific; HTML cannot inherit its authority because it is adjacent in the module map. [H01, H11]

## 2. Existing directory/capability candidates do not supply the grant

| Candidate | What the inspected code actually establishes | Why it is insufficient here |
| --- | --- | --- |
| Agent assets/attachments and subagent attachment helpers | Agent-derived paths; subagent helper includes normalized Agent ID and a session hash. Generic media publication creates directories and atomically names the destination. | Naming is not private-root admission. The subagent helper had no production generated-document caller in the exact Host-tree search. Staging is also used for ordinary attachments. [H09] |
| Box atomic writer | POSIX component traversal uses no-follow directory opens; leaf regular-file check; exclusive no-follow 0600 temporary file; sync/rename. Non-Unix writer is unsupported. | Useful write-confinement primitive; root is initially opened as a supplied directory and is not subjected to secure-temp ownership/ancestor admission. The same writer accepts uploads. It does not attest the producer. [H06] |
| Native local executor | Effective root is `SAND_LOCAL_EXEC_ROOT`, `SAND_AGENT_PROJECT_DIR`, or the user's home. Native file operations enforce relative traversal, handle/reparse or symlink restrictions and device checks; uploads use the same protocol. | Root containment and upload hashes protect file access/integrity, not generated origin. POSIX root ownership/mode privacy is not established; Windows handle opening is not a private DACL grant. [H12] |
| Runner checkpoint directory | `ProductionRoutedProviderCheckpointStore` has per-Agent/per-stream digest paths, creates new directories as 0700 and checkpoint temporary files as 0600, and publishes checkpoint bytes. | This is a checkpoint owner, not an HTML producer. Existing directories are not admitted with the whole fs-safe root/ancestor contract. Moving document state into this store would create the wrong ownership and would still not prove source origin. [H13] |
| `RoutedProviderAttemptControl` and `ToolEffectReceipt` | Captured generation, cancellation and admission state fence tools; unresolved outcomes deny later admission. | This is useful lifetime authority. It contains no document bytes or source-object identity. The bridge call currently receives tool name/arguments/ID without passing this token into the Box producer. The timeout wrapper can return while its worker continues, so a new producer must carry the original token into completion/capture. [H13, H14] |
| `NodeWorkerWorkspaceLease` | Provided canonical root is bound to environment/session/owner epoch and generation; access checks currentness and containment. | The owner explicitly does not create directories or processes or establish root permissions. It accepts a preprepared root; no ordinary Runner→generated HTML→SendMessage route was found. Do not introduce a separate node execution route merely to borrow this type. [H16] |
| Cloud-worker artifact publisher | Digested immutable artifacts for `WorkerBinary`, `RuntimeBundle`, and `PluginBundle`. | These are deployment artifacts, not user documents. Reusing that store would introduce a second document truth without solving producer admission. [H17] |

A chmod-0700 directory alone is also not a claim of isolation between arbitrary processes running as the same OS user. The inspected Box shell is shared execution, with a process-wide environment; setting a mutable global output-directory environment variable is not a per-turn capability. A new implementation must state the principals it excludes and must not claim stronger isolation than its actual executor provides. [H05]

## 3. Fixed-U source contract and dependency closure

### The upstream origin decision

U's `resolveTrustedGeneratedHostReadHtml` first observes an actual regular non-symlink file with `nlink === 1`, then obtains canonical file, temp-root and outbound-root paths. **Outbound is checked first and requires a stored marker even if outbound is nested inside the trusted temp root.** Only a non-outbound file under the root returned by `resolvePreferredOpenClawTmpDir` receives the temp-root source. This is not a generic OS-temp allowlist. [U01]

`resolveOutboundAttachmentFromUrl` loads/validates bytes through the real media loader, stages those same bytes, and writes provenance only when that loader returned trusted generated source. Ordinary `resolveOutboundAttachmentFromBuffer` has no such grant. Marker creation captures the state context and computes SHA-256/size before asynchronous path/storage work. [U01, U02]

The wrapper uses `@openclaw/fs-safe/secure-temp-root`, with preferred `/tmp/openclaw`, fallback prefix `openclaw`, and preferred-root skipping on Windows. Workspace use adds canonical POSIX ancestor checks: group/world writable ancestors need sticky protection. Failed ancestor admission tries a state-directory temp fallback. The `coordination` option bypasses this added workspace ancestor check and must not accidentally be used for document source admission. [U03]

### Exact dependency and license identities

| Item | Exact identity / observation |
| --- | --- |
| U package / lock blobs | `6989c0618742d3c2b93cc336cbe1d9c3c4f0a657` / `83b4975eb96942ec007a2eb9847ccfe98d380bda` |
| Locked package | `@openclaw/fs-safe@0.27.0`; actual export `./secure-temp-root` → `./dist/secure-temp-dir.js` |
| Locked SRI | `sha512-v9RWszhsnswr7kAg/6ZM1fUxtyY1PItj7uBarnQ2UdeTPgDdUZFS5hXIGUdLtNvSty/JEPJvjE3MWqbKbx9dng==` |
| Npm archive | 1,101,331 bytes; SHA-256 `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`; directly rehashed, SRI matched |
| Exported resolver bytes | `package/dist/secure-temp-dir.js`, 10,351 bytes; SHA-256 `2f37e8e6b1bb31fcab349986d08f9a2da074b00500ac2e66ec7cc524dd83ff04` |
| Repair implementation bytes | `package/dist/secure-temp-repair.js`, 5,052 bytes; SHA-256 `097934801db8ef99bb3062d55f5639953b347ee55b2db146751141dfffeedbac` |
| Upstream fs-safe tag | `v0.27.0`, annotated tag object `ba5bbdaf7075ce4e0501d00d26b4864f8c7ad3d9` |
| Source commit / tree | `67dee5ef4c03655a2aa98893539475ad041cb61b` / `6d2ace92770970c1f11479bb8ee0ccb1bf061e6d` |
| Source archive | 3,269,313 bytes; SHA-256 `6a8b657cc3da40d3092974969a1aa39061bcf38d41fd86467f296128342f156b`; directly rehashed; four selected members match exact GitHub source captures |
| Source resolver / repair blobs | `965024c145d83b1a68594b67f2aa6c16765e188b` / `b7f7c617a7859a73bf1c92885969d50f0ffc824f` |
| License | MIT; actual archive and source `LICENSE` bytes match, Git blob `88706eede2b53e494ba1c5b5ad28243393fbb378`, SHA-256 `8d703995c48aeb3726ab83f111a10ee696835e096802ee0329ac50216ab9434e` |

See [npm archive accounting](sandbox:/workspace/scratch/7d27ea239237/html-origin-boundary-inputs/fs-safe-archive-identity.json), [source archive accounting](sandbox:/workspace/scratch/7d27ea239237/html-origin-boundary-inputs/fs-safe-source-archive-identity.json), the [captured npm metadata](sandbox:/workspace/scratch/7d27ea239237/html-origin-boundary-inputs/fs-safe-npm-metadata.json), and the [actual exported implementation](sandbox:/workspace/scratch/7d27ea239237/html-origin-boundary-inputs/fs-safe-npm/package/dist/secure-temp-dir.js).

These facts establish exact package bytes and inspectable source identities. They do **not** establish independently verified attestation signatures or a reproduced source-to-package build. The registry exposes an attestation URL; no signature verification was performed. The tag API's verification metadata is not substituted for independent verification. The package also declares jszip and optional platform-native packages; this bounded resolver review is not full transitive/native/license closure. A source-informed Rust implementation need not introduce the npm runtime. Substantial adaptation must retain the captured MIT notice through the existing notice process.

### What the actual export guarantees, and what it does not

On the inspected POSIX path, the resolver captures filesystem hooks, descriptor operations and user identity before work. It rejects non-directory/symlink candidates, mismatched ownership and group/world write bits when a user identity is available; access must allow write/search. New roots are created as 0700 and re-observed; a concurrent creator is not automatically trusted. Preferred and named fallback roots are admitted individually. Supplied virtual observations cannot silently authorize host creation. [D01, D02]

Repair requires a complete captured descriptor adapter with safe open flags. It binds the original directory device/inode/uid receipt, opens without following a symlink, compares descriptor and pathname identity, changes permissions through the descriptor, and rechecks around the access/finalization boundary. These are useful invariants for a Rust private-root extension; a path-only chmod is not equivalent. [D01, D02]

**Windows limitation:** `dist/secure-temp-dir.js` lines 14–18 set the descriptor adapter to undefined on Windows. Lines 60–68 return true from the owner/mode predicate on Windows or when uid is unavailable. Directory/symlink/access/path-alias checks remain, but the inspected resolver/repair call graph contains no DACL admission. The wrapper's line-63 comment about fs-safe owning Windows ACLs is not implemented proof. The fixed-U sealed bootstrap registers the same resolver export and configures fs-safe native mode off, so that bootstrap does not close the gap. [U03, U04, D01, D02]

**Android limitation:** the wrapper skips its POSIX ancestor walk for Android and comments about a trusted app-group policy. The inspected export does not establish that additional policy. No Android or Windows execution evidence was produced. Do not promise either private-origin platform contract from this source reading alone.

## 4. Smallest existing-owner extension

### Concrete proposed callable contract

The authoritative producer must be a real operation in `RunnerBoxResourcePort` / `ForeverBoxRunnerResourcePort`, backed by the existing Box execution resource. It cannot be a marker-only helper, a test fixture, a new artifact service, or an API that accepts arbitrary HTML bytes as already trusted. Suggested names below describe the contract; they are not present symbols.

1. **Compose one captured producer authority.** App passes the captured Agent, Session key/lifecycle revision and turn identity to the existing per-turn Box owner and shares that same owner with `ProductionSendMessageSink`. On the actual tool invocation, bind the original `RoutedProviderAttemptControl`, resource/execution identity and tool effect receipt before execution. The Session owner supplies the lifecycle facts; request JSON cannot supply or replace them. The attempt token must reach the worker, not just the outer timeout wrapper.

2. **Add an actual generated-document output mode to Shell execution.** A model-supplied request may ask for output capture, but it grants nothing. The Rust owner creates a private `GeneratedOutputCapture` and invokes the real authorized Shell producer. For the narrowest first implementation, let the existing supervisor retain a dedicated bounded byte output stream or its raw stdout bytes **before** lossy text conversion; the child command is the producer. The capture requires a correlated completed execution, producer EOF, a byte bound, and acceptable terminal/lifetime state. The current text JSON result is insufficient. A background/timeout/cancelled/unknown worker cannot return a ready trusted output merely because it later writes bytes.

3. **Mint the private receipt only from actual owned capture.** Return an internal, non-deserializable receipt binding producer operation, captured owner/lifetime, exact byte buffer and byte identity. A model-visible locator is only a lookup key; it is not the capability. Resolve it through the same captured owner, never by decoding caller JSON into a receipt. A marker type with a public constructor from `path + bool`, `path + sha256`, ordinary upload bytes, or tool-result JSON would fail this requirement. Existing per-turn resource state can retain these bounded outputs; do not add another durable artifact store.

4. **Stage the captured bytes through current Session media custody.** The sink recognizes a locator only when its captured owner can supply the matching current receipt. It consumes those exact retained bytes, calls the existing Attachments HTML MIME/shape policy, and publishes under the existing Session admitted filesystem/media mutation. Preserve the current pending-media ownership, cancellation, settlement and recovery contracts. Do not reopen a model-supplied source path after capture or derive origin from the destination. Any retained source receipt must stay internal to the materialization boundary; URL/filename projection is not an origin token.

5. **Persist provenance in the current Session/Agent database and consume it through current Attachments.** Write the generated marker for the exact staged object/bytes only after valid producer admission and successful bounded publication. Bind canonical staged path/object, SHA-256, size, version, and captured owner. Use existing Session/Attachments transaction and cleanup owners, including their established media custody semantics; do not create a parallel DB. A later outbound read still needs marker lookup and byte/shape validation, with outbound precedence over any trusted temp scope. Missing or uninspectable authority narrows trust.

This is an origin claim for the output of an admitted host producer operation; it is not a proof that HTML is benign or semantically novel. The new producer route must not allow a caller simply to submit an upload buffer or label an existing uploaded file. If an implementation instead wants to claim arbitrary existing Shell-file output as trusted, the filesystem alternative below is a separate required boundary.

### Why ordinary file-output support remains blocked

For existing `Shell → /workspace/report.html → SendMessage(path)`, there is no private root receipt and no connection from execution to source object. An implementation that retains this path must extend the existing Box/native filesystem owner to allocate and admit an output scope **before** the producer runs, carry the captured scope into execution, and capture a bounded regular single-link object from the admitted root. It must reject generic upload/write registration, path/root replacement, symlink/reparse/hardlink ambiguity, stale execution/session ownership and unsupported platform admissions. POSIX owner/mode/ancestor checks must be real; Windows privacy requires a real platform contract, not the wrapper comment.

The current shared shell can address other filesystem locations as the same OS user. A fresh 0700 path does not by itself establish isolation from that principal, and a global environment mutation does not bind one concurrent turn. If scope exclusivity is part of the asserted source authority, it must be enforced by the actual executor or a handle-based output route. Merely borrowing the checkpoint directory, Node lease, or native read sandbox cannot supply it.

The bounded output-capture route above avoids trusting an existing pathname and gives one real producer an implementable source receipt without adding a runtime or durable store. It **does not** close arbitrary filesystem-temp origin, Box-provider/transport isolation, Windows/Android private roots, or complete fixed-U parity. Unsupported executor/resource/platform paths must remain ordinary untrusted media until their own producer/capture contract exists.

## 5. Required positive and negative oracles

These are implementation acceptance requirements only. No oracle was run in this task. Execution remains in the existing exact-subject Actions lanes.

| Oracle | Required observation |
| --- | --- |
| Real producer positive | The actual Runner Shell producer under captured authority emits valid HTML; actual Box capture returns the exact bytes; real `ProductionSendMessageSink` stages them; a fresh existing Session/Attachments owner reloads and validates their marker. A fabricated row or fake producer cannot substitute. |
| Identical ordinary bytes | The same HTML supplied by upload, generic internal write, arbitrary local path, generic temp root, ordinary buffer publication or explicit HTML mode gets no generated marker/grant. Successful send changes none of these facts. |
| Forged or replayed locator | Invented locator, JSON origin fields, another Agent/session/turn, stale attempt and a repeated locator after retirement cannot resolve a receipt. A hash supplied by the caller cannot establish it. |
| Producer failure and lifetime | Cancel/timeout/unknown result, attempt replacement, Session revision change, late worker completion, closed owner and shutdown never transfer authority to a successor. A paused old worker must not publish into the next turn's store. |
| Exact bytes | Size cap, malformed HTML/MIME mismatch, output truncation, lossy UTF-8 substitution, source replacement after capture and mismatched marker hash/size cannot produce trusted staging. Staging consumes the captured buffer, not a reread path. |
| Filesystem alternative | On each supported platform, real symlink/reparse/hardlink, writable root/ancestor, wrong owner, replacement/concurrent creator and scope-admission failures deny origin. Unsupported platform checks are visible failures/open cells, not passing placeholders. |
| Outbound priority and recovery | Ordinary outbound HTML remains untrusted even under a trusted temp root; missing marker/store refusal denies trust; stale observation cannot delete a newer marker. Current media-custody settlement and fresh-owner recovery remain intact. |

The immediate blocker is therefore not “choose a temp directory.” It is **add and specify a real producer-output authority at the existing execution owner, then carry it into the actual sink**. The existing `BL-MEDIA-HTML-PROVENANCE-CHAT-WIRING-001` / generated-HTML responsibility remains open. No new completion or acceptance status is asserted by this report.

## 6. Bounded semantic read ledger

All H paths below are in the captured `current/` subtree of the [source manifest](sandbox:/workspace/scratch/7d27ea239237/html-origin-boundary-inputs/source-identities.json); all exact hashes and other capture-only files are recorded there. The ranges below are the substantive reads used for the conclusions. Search hits and captured bytes outside them do not add semantic review credit.

| ID | Source path | Git blob / substantive ranges |
| --- | --- | --- |
| H01 | `source/host/app/src/main.rs` | `7ec9bd8a22fd595f63cb3f4b632673e298294dfa`; 558–564, 619–723, 1004–1097, 9987–10042, 10952–10961, 11683–11720, 11935–11983, 12330–12355 |
| H02 | `source/host/src/runner/box_tool_access.rs` | `9a84091770cd1c23debbdf5fbf6eb33a953158f7`; full 1–351 |
| H03 | `source/host/src/extensions/forever_box/runner_tools.rs` | `c38b3dda53e5d9eccb1581ed3bd6ec38599f7ff6`; 1–240, 428–560, 850–886 |
| H04 | `source/host/src/box/exec_daemon_process.rs` | `66e218e389db0352bde7f1917aa0c14ef8a8c78c`; 1–160; 185–208 discovery |
| H05 | `source/box-exec-daemon/src/server.rs` | `a40dc3b5f78e414c3ac302137668f1d5be8dc73f`; 630–811, 868–942, 1625–1849 |
| H06 | `source/box-exec-daemon/src/workspace_write.rs`; `source/host/src/box/box_file_transfer.rs`; `source/host/src/box/generated_production.rs` | `e91358cb84271251a8d271ef92a200f02be41c90` full; `eeb9f73515de672ba6236272650743962c5d6035` 1–200, 215–325; `81f5a74dbf0ebc3700f177639bdd875f8c00eb56` 708–760 |
| H07 | `source/host/src/box/loopback_sand_box.rs`; `shared_desktop_sand_box.rs` | `ac2fcd557f09eb3c5fce9e8d22001dd6600266d3` 1–140, 400–558; `dfce60df4441cd96118d6a844e06fba48021f10d` 421–461 |
| H08 | `source/host/src/runner/tools/send_message_tool.rs`; `send_message_encoding.rs` | `86a9e0e394b0145173bc033ceb757b7fb10835e5` 25–100, 276–342; `22e2700f4ea2f894b92f3233d0a399a296f6fa5b` 217–267 |
| H09 | `source/host/src/attachment_paths.rs` | `6e1a2b8eb5752e005bbce30ed0b6eaaacfda0cda`; 107–169, 225–299 |
| H10 | `source/host/src/runner/turn_agent_composition.rs`; `tools/turn_toolset.rs` | `c988b9a02db95ed5106a34d67bdd5c9ff30447d6` 300–345; `d2f4af4346d3576bdf0c4c43c6371b38e99c9069` 650–745 |
| H11 | `runner/large_output_spill.rs`; `cloud_agents/cloud_agent_tool.rs`; `runner/tools/sand_browser_tools.rs`; `sand_generate_image_tool.rs` under Host `src` | `f33b9a2964e3c4ee9ca75be847aa2354930261f8` full; `6a5a1aacbe87bd302e99e730211d2d464eb56046` 477–515; `191eda1bfb8c2958716a1c4635b277d09f87495c` 691–724; `6cbf93921b0e51c787503f68282fbc7d86b27aac` 149–205 |
| H12 | `source/local-exec-daemon/production-executor.ts`; Host `local-exec/local-exec-machine.ts`, `local-exec-native-file-access.ts`; Host `src/extensions/local_exec/native_file_sandbox.rs`, `gateway_local_exec_sand_box.rs` | `ab36e8824bd960750bcf6e0f7b386ed524609244` 1–230; `863c8ab5e02744cfd380e478c8be5db6a3ce7f55` full; `9957954ba774811cab99f500ae0d1105440df80d` 87–167; `4f0f5a6d172bf3e555ddcdc74b6b9bbb80afb5f3` 1–230, 258–475, 658–775, 810–938, 1040–1152; `fd3f23c69ef5841980df39e6c486f311b97c33d1` 333–505 |
| H13 | `source/host/src/runner/routed_provider_runtime.rs`; `production_turn_run_shell_adapter.rs` | `a351081d6e8e9e9c1fb51439f4d67b48e4ace886` 215–640; `d0fe2714923338dd717284f08f05eef438db69d9` 165–232 |
| H14 | `source/host/src/runner/tools/mcp_meta_tools.rs` | `923288198b898083cbfe5da9a1f76e9f309ee516`; 131–245 |
| H15 | Shell bytes / completion | Same H05; 1625–1745 are especially material |
| H16 | `source/host/src/extensions/node_host/worker_workspace.rs` | `8147cda74d71db159e936988bbad30802da09612`; full 1–267, including unexecuted test source |
| H17 | `source/host/src/extensions/cloud_workers/artifact_store.rs`; `extension.rs` | `f88eba22077c0849dba9e7d3497084c82fd645f4` 1–245; `0d54fa210fa844b6942ab99600d47f6d837d9b5d` 726–756 |
| U01 | `src/media/web-media.ts` | `f48bca504f6fb77c3d094864eb656cb2b1bf25e2`; 141–259, 664–756 |
| U02 | `src/media/outbound-attachment.ts` | `d2c00c2905323763fb58cb61b4e3aeb5bf37a250`; full 1–65 |
| U03 | `src/infra/tmp-openclaw-dir.ts` | `cf059c7a37029556ba07f40f73563dcea9b39adb`; full 1–128 |
| U04 | `src/infra/sealed-runtime-registry.ts`; `sealed-runtime-bootstrap.ts` | `97ead5bce1e79fc2f102e10f8b1fc5a8b21e36ff` full 1–17; `b4b289981a3c195da0e3633698bf7dd7b11c9ff7` full 1–9 |
| D01 | Actual npm `dist/secure-temp-dir.js` and corresponding pinned source | npm SHA-256 above; full 1–188; source `965024c145d83b1a68594b67f2aa6c16765e188b`, full 1–238 |
| D02 | Actual npm `dist/secure-temp-repair.js` and corresponding pinned source | npm SHA-256 above; full 1–110; source `b7f7c617a7859a73bf1c92885969d50f0ffc824f`, full 1–130 |

Additional bounded dependency reads: actual npm `directory-entry-path.js` 1–21, `recursive-mkdir-path.js` 1–8, `safe-path-segment.js` 1–76, `windows-path-alias.js` 1–140, `temp-directory-diagnostics.js` 1–29, `path.js` 1–62, and `secure-temp-dir.d.ts` 1–27. `device-path.js`, `errors.js`, `root-path-existing.js` and `windows-path-syntax.js` were captured/discovery only; unrelated native/package members were not semantically inventoried. Package export/dependency/engine fields and the complete 22-line MIT license were read. Fixed-U lock reads were 293–295, 5154–5156 and 13230–13242; package dependency at 2412. This is not a whole lockfile review.

Governance semantic reads: full supplied AGENTS; SOT 1–53; main spec §17 at 671–720; module-map relevant M05/M07/M10/M11/M23/M24 rows and unique-owner rule; full HHP-v2 contract blob `f0892088bfa3448441f7b92df6344692ac535932`. Exact Host-tree searches located generated-HTML/temp-root candidates, all `RunnerBoxWriteRequest` users and subagent attachment helper references; search results were discovery, not proof of exhaustive repository semantics. The prior 169-line bounded assessment was read completely. No source-reading, implementation, verification or global completion ledger state was promoted.

