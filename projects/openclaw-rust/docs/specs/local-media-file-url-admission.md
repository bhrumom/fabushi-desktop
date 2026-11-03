# Local media file URL admission — LFA-v1

Status: specified before implementation; independent review and exact-H Actions pending.
Project: OCRM-001. Task `647e0932-bea4-4113-9a61-65ae3db58955`, round 1.

## Problem, authority and current behavior

Fixed OpenClaw U remains `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`. Start from Fabushi H `947e980bd3c9dcb06535127f5cedfd193b83f0fc`, tree `686109128a616195e65d1c9018ccd5b0b9d2cc8a`, on the existing migration branch. Preserve the published HHP-v2/IPB-v1 and prior Runner, Session, channel, workflow, source and evidence changes.

## Concurrent implementation integration

The recovered full batch also incorporates FUA normalization recovery after
Linux H364 run `41567768358` proved the four-separator reader-boundary failure.
Preserve every original failure and zero-reader-entry assertion. Retain the
web adapter's pre-normalization four-or-more separator restriction for slash,
backslash, mixed separators and ignored ASCII tab/CR/LF spellings. Reuse this
spec's existing shared converter and native Windows drive-root precondition,
including refusal of a bare `file:///C:` before the dependency's conversion.
The unpublished FUA R2 tree `a2b9d3a154145d8071cc63909a5e6a20feeb7731` is a
historical source-review slice; its reviews do not approve this full LFA batch.
The recovered original LFA tree `6a6e8cf0c1b9e74df84a369dd3c639cd0b750d31`
remains immutable. New integrated review and exact-H Actions are required.

The R1 review base was `497c2c7a7bc5201aead83d3d36e44f52ba164d04`, tree
`97181e47831735417e4aeb8197d9a62a5d66b12d`. Preserve the intervening exact-947
terminal evidence and incoming generated SendMessage producer/recovery work.
Its strict shared SendMessage target retains every original assertion and now
includes both LFA facade cases: 17 expected passes instead of 15, with the
other target summaries and failure/ignored/uniqueness checks unchanged.

R2 integrates on `0e845ce1679e0cdf95346adc54f5d58bf06464e8`, preserving incoming Memory identity, device role revocation and the existing Attachments BR-v1 reader from `071aeb6af1b223cceee9d81b5f7df3bad484d5a4`. Reuse its raw-prefix guard unchanged, including localhost followed by repeated path separators and the drive-token interior-separator positive; invoke that guard only after shared parsing identifies a file source. Preserve every bounded-reader body, service cap conversion and test. The native capture unions both scopes, executing the common channel target once. Its 16 groups select 163 Linux/macOS and 158 Windows positives with 46 inputs; these are source expectations, not results. The inherited zero-job workflow-size refusal still requires the separate existing WFX repair before exact-H execution.

During authoring, live H advanced through evidence carrier `d26d44d1441c05e3dbb7dc18eb908ab042cbdae9` to FUA-v1 production H `3647531d1d47d1c690b98a06f0156ae2734ff7ad`, tree `a85fe54f86823ff30b682f91ff97b1daca3f1318`. FUA already removed the web converter's denylist/fallback, separated pure admission from its existing private synchronous reader, and added causal zero-reader-entry/native-service controls. LFA preserves those exact responsibilities and controls. It adds the shared Result boundary for actual Runner/AgentManagement and Channel consumers, and makes the existing web adapter delegate conversion to that same owner; it does not republish a second FUA implementation.

The earlier recovered LFA snapshot used `073b98b4826c8125d0a42d15f8ccdbfebb5eb488`, tree `f73e4982a23b357647b6fec38550c75bfe11f25e`, which adds only a root-verified evidence carrier after FUA. Preserve FUA's raw and resolved network refusals, their `network-path-not-allowed` code, absolute Windows verbatim-drive workspace positive, private reader boundary, and unmodified bounded read/HTML/image body. File URL refusals retain `invalid-file-url`; the added raw-path alias/NUL guard uses `path-not-allowed` after existing network classification. Unix converted double-slash paths remain subject to the web adapter's stricter existing restriction. All FUA selected cases remain in the same native capture alongside LFA cases. Prior FUA source approval and any earlier-H evidence do not approve this integrated candidate; fresh full-scope review and new exact-H Actions are required.

The initial facts below describe the 947 starting state; the preceding integration record supersedes that state where FUA has already repaired it.

The actual Runner `SendMessage` path calls `ProductionSendMessageSink::resolve_attachment_source`, then `resolve_production_attachment_source`, then `fs::read`, existing per-agent media materialization, Session media custody, Transcript persistence and optional channel delivery. Agent-to-agent image resolution also calls this production resolver. The current `file_path_from_file_url` is only `Url::parse` plus `to_file_path`; Windows accepts a remote URL host as UNC. This can reach filesystem access before any network-path rejection. Merely returning `None` for an invalid file URL is insufficient: the App resolver currently converts `None` into successful original-URL passthrough.

The existing Channel attachment path independently resolves a local source and calls `fs::metadata` before its bounded reader. Its converter rejects remote file URL hosts but lacks the complete converted-path alias boundary. The existing direct Attachments `loadWebMedia` service has another converter with a manual fallback: parse/conversion failures can become local paths. Its finite external Gateway registration remains absent; this batch does not claim to register that service.

Relevant exact starting blobs: App main `4312a8ee014ccda8b44e7eab2a8b2feee9e0e1c3`; Runner SendMessage tool `08bfbe0bedc8138bb00804b5fbbd3b5ce3b046f2`; media reference `acbed906d06b496b481fd4d4b5c604d9fc3eda57`; media file context `2372f03102c106fdb93d3c5a50e71a74c80e55b3`; Channel attachment `356e901feb2328729f882afe487fbe0199f4a40a`; web media `61291cf57b42408cd98c7fe7676d2a0694fa44d0`.

The fixed-U dependency is `@openclaw/fs-safe@0.27.0`, archive SHA-256 `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`. Actual `dist/local-file-access.js` admits only empty/localhost authorities, rejects encoded pathname separators, invokes Node file conversion, then checks Windows aliases and network paths. Actual Windows syntax/alias modules define the platform distinction. U's web loader invokes this before local reading. Rust host and App both lock `url@2.5.8`, archive SHA-256 `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`; the acquired published conversion/parser source agrees with the reviewed exact source commit. Archive identity is not a complete dependency or execution verdict.

## Ownership and shared interface

The existing `attachments/media_reference.rs` owns shared media URI classification and bounded reading. Extend it with `resolve_local_file_url(raw: &str) -> Result<Option<PathBuf>, MediaReferenceError>` and a narrow local-path spelling guard reused by the actual local readers. No new module, crate, permission owner, scheduler, database, media registry or filesystem capability is introduced.

The result distinguishes ordinary non-file input (`Ok(None)`), an admitted file URL (`Ok(Some(path))`) and an invalid file URL (`Err`). The existing Runner Option facade delegates to this owner for metadata-only callers. The production App resolver consumes the Result directly; both real sinks propagate refusal. The local Channel resolver and web-media adapter reuse this same conversion and spelling policy, preserving their public error forms (`None` and `invalid-file-url`). The actual Channel reader checks the final reanchored path before metadata access.

## Required behavior

| ID | Contract and oracle |
| --- | --- |
| LFA-01 | Recognize a file URL using the same URL parser normalization, including mixed-case schemes and parser-accepted surrounding control/space normalization. Any recognized malformed file URL fails; it is never manually reconstructed as a local path or forwarded as successful original-URL resolution. Non-file HTTP/HTTPS and other existing non-file passthrough behavior remains unchanged. |
| LFA-02 | Only empty or localhost authority is local. Reject every other host, userinfo/port or malformed authority before filesystem, Box, persistence or delivery work. Validate pathname percent escapes; encoded `/` or `\\`, malformed escapes, invalid decoded UTF-8 and NUL are refused. Never use lossy decoding. |
| LFA-03 | Conversion must return a valid platform absolute path. On Windows reject network/share/device forms and alternate data stream or drive-relative colon aliases; ordinary rooted drive paths remain valid, and any admitted extended rooted-drive form follows the actual existing filesystem spelling contract. Apply the same Windows path guard to Channel local-path and reanchored-path admission before metadata/read. Unix colon filenames and valid Unix local paths remain available. |
| LFA-04 | `resolve_production_attachment_source` returns a Result. Both `ProductionSendMessageSink` and `ProductionAgentManagementSink` propagate invalid file URL errors before read/download/materialization. No error becomes an unchanged URL success. The existing SendMessage bridge invokes its current abort-custody path when a later image fails, preserving already-established cleanup and Session release ownership. |
| LFA-05 | Valid native local file URLs, localhost, mixed-case scheme, single-slash form where the native platform supports it, percent-encoded spaces and Unicode continue to reach the existing real reader. Verify exact returned paths and bytes, filename, owned materialization, original-source preservation and normal message behavior, without granting new roots/trust. |
| LFA-06 | Preserve all original valid assertions and HHP/IPB tests. Repair the existing cross-platform test flaw that asserts a drive-less POSIX file URL succeeds on Windows merely because a suffix is present: use an actual native absolute fixture, exact path equality, retain the Unix POSIX control, and add a Windows drive-less refusal. This is an oracle correction, not a product success claim. |

This is local-source admission, not a complete path capability or TOCTOU confinement contract. Keep the web adapter's pre-existing stricter cross-platform rejection of raw `file:////...` spellings explicitly separate from fs-safe's Windows-only network policy; do not claim full Unix equivalence for that retained restriction. NUL rejection moves the existing filesystem refusal before fallback; the fixed dependency's string converter itself does not explicitly reject NUL.

## State, errors, cancellation and recovery

The parser/guard is pure and owns no durable state. It must perform no `canonicalize`, metadata, open, network or Box operation. An invalid source creates no managed file, Session admission, transcript entry or channel request. If earlier images in the same SendMessage call have already acquired custody, the existing bridge failure path rolls back those owned files and releases their current Session media records. Tests must exercise the real production sink and the actual bridge rather than a replacement resolver.

AgentManagement uses a different existing image loop without SendMessage's
per-call custody rollback. Its existing `parse_images` must therefore admit
every file URL through the same pure media-reference owner before returning
the image vector and before any `resolve_image_source` call. A valid first
image followed by an invalid URL must leave no newly materialized file, no
source/target transcript change and no outward wake. Keep the actual App
sink's direct single-image check as well. This value preflight does not add
a state owner or claim cleanup for later read/persistence/delivery failures.
Retain native/HTTPS image positives and pin zero resolver-entry for a bad
later URL in the native bridge contract, plus the real App batch oracle.

The current Session work-admission check still precedes production SendMessage resolution. Existing closed/deleting owner refusals remain effective. Repeated invalid requests, including after a real Session reopen, remain refusals without replay or new persistent state. A later valid request is a fresh normal call. This batch introduces no persistence schema or migration and makes no claim about physical crash timing, external providers/devices or filesystem races.

The separate existing issue where failed media persistence may enter Box/original-URL fallback remains open. This batch changes invalid-source handling only; do not infer known or unknown write-outcome handling from a parsing refusal. Generated HTML provenance, full read authority, public `loadWebMedia` Gateway registration, MIME/codec equivalence and all other migration responsibilities remain required.

## Implementation and verification scope

Product paths are the existing media reference/file-context/web-media owners, Runner SendMessage facade, Channel reader and App production resolver. Root integrates App/composition, existing capture/workflow, ledger and RTM serially. Independent authoring uses exact base/allowed paths and the LFA-v1 interface. Existing App test helpers such as `channel_host_lane_tests::idle_runner_owners` may be reused for actual owners; do not introduce a second production owner or claim a mock is a real external channel.

Add native contract coverage for the shared owner, wrappers and real local Channel/web-media readers, including positive native paths, every denial class, decoded aliases and no-fallback controls. Add App tests using actual Session/ForeverBox owners and production SendMessage/agent-management sinks for early refusal, partial-image custody cleanup, positive local materialization and repair/reopen behavior. Controlled Box/provider/carrier plumbing is explicitly not live ecosystem acceptance.

Extend the existing exact-H media and Linux App capture rather than creating another runner. Bind the changed sources/tests and this spec to receipts. Require exact required-positive names, nonduplicate results, zero failures/ignored tests and command/capture exits; retain existing oracle qualification and original failure history. Native Windows must execute Windows path cases; a Linux pass cannot stand in for it. macOS/platform packaging and full product acceptance remain open if unavailable.

Before publication, freeze the complete source tree, full diff and manifest, obtain independent general/security and pinned structure/reuse/performance reviews, decide findings before fixes and review any accepted repair. All executable validation runs only in GitHub Actions after publication of exact H. Source/API/hash/archive work outside Actions is evidence acquisition, not validation. Completion of this slice requires actual production consumer results for its selected scope; no global flag or unit verified promotion precedes scoped authority/coverage.

## Native Windows drive delimiter recovery — LFA-03a

This amendment is specified before its R4 implementation. R3 integrates the
existing WFX extraction on base `3438eb5bac755c399c9ec686414bbb4c0816888d`, tree
`b544c1d377ca82618cdadcf5106cb3c10d82411b`; the full source candidate remains
unpublished and unexecuted. Preserve the extracted portable capture, incoming
canonical inventory, all BR reader bodies and the existing test selections.

The native Windows receipts for subjects
`e19a0a265e400a4c9b1d96520c84fb612a3843e8` (run `41571103928`) and
`3438eb5bac755c399c9ec686414bbb4c0816888d` (run `41571310630`) both retain a
22-pass/1-fail full web target. The failing rooted-drive assertion does not
label the particular input. Source reading separately identifies a causal
dependency boundary: `url 2.5.8`'s `parse_file_host` returns a false path-start
flag for the empty/localhost host; `parse_file` inserts a slash before passing
the still-leading slash to `parse_path`. For `file:///C|/`, the drive token is
then beyond the first-segment normalization condition. Leading empty segments
are removed later, but the literal pipe remains. The Windows converter accepts
only a colon or percent-encoded colon in that drive position. R3's shared
absolute-drive precondition also refuses this source. This is source-derived
diagnosis, not an executed R3 result or a variant label from the failed log.

LFA-03a requires the existing shared owner to normalize only a parsed Windows
pathname's literal leading `/<ASCII letter>|/` drive delimiter to `:` before
native conversion, after the original authority/violation checks. Use the
existing URL path setter and native converter; do not reconstruct a native
path or add a fallback after conversion failure. Keep percent escapes, Unicode,
query/fragment separation, root/size policy and downstream owner checks intact.
The URL setter's published contract preserves existing percent escapes and
restores query/fragment positions. No such rewrite runs on Unix.

Encoded pipe, bare pipe drive, drive-relative path, remote authority, encoded
separator, malformed escape, invalid UTF-8, NUL and ADS remain refusals. The
web adapter's existing raw leading-separator guard remains authoritative even
after this narrow normalization. Preserve every existing four rooted-drive
positive and add the source value to its failure diagnostics. Extend existing
shared-owner variants and the real native web/service byte/cap/retry case with
triple-slash, localhost, single-slash and authority-position pipe forms. Extend
existing zero-reader-entry negatives around encoded/bare pipe and aliases.
These are added table entries within the already selected tests, not reduced
coverage or synthetic success counts.

Completion requires fresh focused general/security and pinned structural
reviews, unchanged shared capture gates, and exact published-H native Actions,
including actual Windows execution. Retain both original failed receipts.
Do not rerun an unchanged failed subject, upgrade a ledger status/global flag,
or claim App/real-service/platform-wide acceptance from this normalization.

## Delivery, rollback and remaining acceptance

### Concurrent native-converter oracle integration — LFA-03b

Integrate on `78aa967cc9400bb5c207441b2a78dbf7021d596c` while preserving its
incoming Device token-rotation production, shared workflow selections, ledger
and evidence. Its FUA-WIN-ORACLE-01 correction is valid for the older production
converter: locked `url` 2.5.8 directly refuses `file:///C|/` while normalizing
`file:/C|/`. LFA-03a is an explicit product compatibility extension before
native conversion, not proof that the old converter violated FUA-02.

Keep the incoming single-slash pipe positive and labelled diagnostics, plus
LFA-03a's triple-slash pipe positive and all three colon/encoded-colon positives.
Retain the unchanged dependency refusal as a direct Windows converter control
inside the same selected test. Do not require the current product function to
both accept and reject the same spelling. Keep every other refusal, real
native service/byte/cap/retry case and zero-reader-entry control. No test-name,
selected-count or strict-gate change is required by this integration.

The inspected fixed-U Windows tests do not contain a pipe-specific oracle;
the compatibility extension is explicitly specified here, with exact-H native
execution still required. Preserve both older failed receipts and their
unlabelled iteration, without converting source inference into an observed
runtime result or full Node/Rust parity. The production normalization owner
and its R4 guard/order remain unchanged by this spec/test reconciliation.

Publish through the existing branch with a compare-and-swap lease, preserve concurrent work, and read back the full remote commit/tree/content. A rollback reverts this admission change without rewriting previous receipts or durable Session data. Reverting to unsafe URL acceptance is not a security acceptance path; hold affected delivery if this repair cannot meet its contract.

G01–G12, AC01–AC24, real provider/channel/plugin/node/device, UI/timing/security/performance/soak, supported package/install/upgrade/rollback, existing Fabushi regression and independent ACCEPT remain globally unaccepted. No dependency discovery, helper pass or test count establishes full migration completion.
