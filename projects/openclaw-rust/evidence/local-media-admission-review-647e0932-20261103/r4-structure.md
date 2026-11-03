# Media admission R4 — blind structure, reuse and performance review

**Terminal verdict: APPROVED.** No blocking or nonblocking concrete structure, reuse or performance finding was identified in the current 21 non-evidence paths, with the five-path Windows repair as the focus. Severity/affected path/function/line/trigger/impact/fix: none. This is a source-review verdict; it is not executable validation, native Windows acceptance, or full LFA/FUA/media/OCRM acceptance.

## Review identity and independence

- Repository: bhrumom/fabushi-desktop. Migration branch: codex/openclaw-rust-fusion-chat-20261101-writer. Task: 647e0932-bea4-4113-9a61-65ae3db58955, task round 1, frozen review round 4.
- Review configuration supplied by the runner: gpt-6.1-sol, xhigh, for the combined structure/reuse/performance lens. The shared path-security boundary, locked native semantics and multiple production consumers justify that effort. No alternative model or worker was launched by this reviewer.
- Frozen candidate tree: 76e6c26d754179ee5af447ecec689a2de844b476. Snapshot: /workspace/scratch/7d27ea239237/media-admission-r4.
- Exact base H: 3438eb5bac755c399c9ec686414bbb4c0816888d; base tree: b544c1d377ca82618cdadcf5106cb3c10d82411b; base snapshot: media-admission-r3-base.
- Prior candidate tree: e9a172d00139550e8a055a61bf06e225b9d4166c; prior snapshot: media-admission-r3. That snapshot was used only for source-byte identity/delta comparison, not as an approved implementation.
- Fixed U: 1440bb1771372334bea23d7642bb52deb7db1845, tree 90c8aab39972da11cfad220361781d49e1bc6700. No rebaseline is proposed.
- Governing immutable Git objects were read through read-only git show in /workspace/scratch/29e7135d8bfe/fabushi-desktop: root AGENTS e4682748087759b2d4bb2b3a47916b6148e8a6f2; SOURCE_OF_TRUTH 3748d734b10c9ed5b2d948467fbaa9483f1dd306 (applicable authority, fixed-source and state/evidence rules); main specification 69c9667af2328e20a1fba09ebf80c5aab4d5c336 §17. The full frozen LFA specification, including LFA-03a, and the full FUA specification were read. Exact-base module-map/media ownership and relevant build declarations/callers were also inspected.
- This was a fresh blind review. Neither frozen historical evidence JSON was opened for content. No prior/other review report or runner decision/arbitration file was inspected. The manifest and complete-diff hunk index expose evidence filenames and identities only. Normative specifications mention historical review/execution states; no earlier finding or approval was adopted as a premise for this verdict.

## Frozen identity checks

Independent source-byte/hash reads verified the byte count, SHA-256 and Git blob SHA-1 of every one of the 23 snapshot paths against the manifest. Every corresponding prior blob was also verified. Every base blob, including absence of newly added paths, matched the manifest. Exactly these five paths differ from R3:

1. projects/openclaw-rust/docs/specs/local-media-file-url-admission.md
2. source/host/src/extensions/attachments/media_reference.rs
3. source/host/src/extensions/attachments/web_media.rs
4. source/host/tests/local_media_file_url_contract.rs
5. source/host/tests/openclaw_web_media_contract.rs

The other 18 paths are byte-identical to R3. This is identity evidence, not inherited review approval. All current changed hunks in the 21 non-evidence paths against the exact base were reviewed with relevant adjacent source. The two historical evidence JSON paths remain identity-covered but content-excluded; their provenance/status adjudication belongs to the root reviewer.

| Frozen artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| media-admission-r4-manifest.json | — | 59b0db33b7d3b19593594de76ac502b7e96729853e180ce2699f0d2f4b9fb45d |
| media-admission-r4-complete.patch | 191292 | 900b7099d01a9b9bd61ee4d25a6f4522fe489fc4e8fb06f66028c11719b4ae28 |
| media-admission-r4-fix.patch | 12234 | 14c6a388c2d0140b9c02d87368444a9a7efc64250de586b7ba78824a579e5825 |
| url-2.5.8.crate | 86512 | ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed |
| fs-safe-0.27.0.tgz | 1101331 | 02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c |

The supplied manifest records a nontruncated remote candidate tree with 10,502 entries and all 23 candidate blobs API-checked. This reviewer independently verified the supplied snapshot identities; the candidate tree object is not present in the local read-only Git object store, so no independent local full-tree/API attestation is claimed. Exact-base Git objects were accessible and used for adjacent production source and lock reads. The later remote advance a27a3a6daec5377a56f61221b34ac3f3c8c4005e is outside this frozen subject; preserving its nine disjoint evidence paths is a publication responsibility, not a reason to substitute a different review subject.

## Windows repair and dependency semantics

The repair is in the correct existing owner: media_reference.rs::resolve_local_file_url, lines 86–204. The only R4 production addition is lines 119–140. It does not create a new parser, path model, module, permission owner, native-path constructor, fallback route, or new dependency. It operates on the already parsed URL, uses the existing URL setter, and then reaches the same native converter and path guard as every other admitted file URL.

The order is significant. Original parsing and the FileWithHostAndWindowsDrive callback run at lines 87–105. Original-host erasure, nonlocal host, userinfo/password and port refusals run at lines 106–117. Only after those checks can Windows inspect the parsed pathname. Thus a remote authority cannot be rehabilitated by the setter or by normalizing a drive token. Hosts retained by parsing are refused directly; hosts erased by the locked parser are refused by the retained syntax violation. Authority-position drive tokens such as file://C|/ are intentionally parsed as local drive path syntax by the dependency, rather than treated as an arbitrary admitted hostname.

The new predicate requires four literal bytes: slash, ASCII letter, pipe, slash. It therefore matches a rooted leading drive delimiter only. The replacement range is safe on UTF-8 boundaries because the inspected bytes establish ASCII at the exact replaced position. It changes one delimiter while preserving the drive letter and the remaining serialized pathname. An encoded pipe does not satisfy the predicate; a bare pipe drive and a drive-relative spelling do not satisfy its trailing-slash condition. It cannot rewrite an interior pipe. On Unix the entire addition is removed by cfg(windows).

The locked archive, not memory of a newer URL implementation, supports the diagnosis and choice of API:

- parser.rs 512–550 invokes parse_file_host and chooses the path-start route from its returned flag. At 1057–1079, the empty/localhost case returns a false flag. In parse_file, that route inserts a slash before parsing the still-leading slash.
- parser.rs 1344–1363 normalizes a Windows drive token only when its segment starts at path_start + 1, and logs FileWithHostAndWindowsDrive before clearing an actual host. Initial empty path segments are removed afterward at 1371–1378. This explains how a literal leading pipe delimiter can survive the earlier normalization opportunity for file:///C|/.
- lib.rs 3128–3191 accepts a literal colon or percent-encoded colon in the native Windows drive token, decodes later segments using strict UTF-8, and asserts the resulting path is absolute. It has no corresponding pipe-drive conversion path.
- lib.rs 1751–1760 documents that set_path does not double-encode existing percent escapes. The implementation at 1767–1789 uses the library's setter parser; take_after_path at 1721–1729 saves query/fragment serialization, and restore_after_path at 1804–1816 restores it and adjusts indices. The owned serialized pathname contains no raw query/fragment component to be reclassified as path data. Changing a pipe to a colon introduces no new escaping expansion.

The selected parser.rs, lib.rs, Cargo.toml and Cargo.toml.orig were byte-compared with their actual published archive members. Host and App Cargo.lock at the exact base both bind url 2.5.8 to the matching archive checksum. Existing Host dependencies already provide url, percent-encoding and regex; the library's normal std feature provides the used conversion API. No dependency or build-reach growth is introduced by this repair. The dependency is MIT OR Apache-2.0; this review does not claim recursive dependency closure or a complete package audit.

The remaining validations still execute after the setter: strict percent syntax and encoded slash/backslash refusal at media_reference.rs 142–170; strict decoded UTF-8 and NUL refusal at 171–178; the decoded absolute Windows drive-root precondition at 179–193; native conversion and absolute-path check at 194–201; and the existing shared local-path spelling guard at 202. Consequently the repair does not revive malformed escape, encoded separator, invalid UTF-8, NUL, ADS, bare-drive or drive-relative inputs. The native-path spelling guard retains its existing network/device distinction and extra-colon refusal. The web adapter separately retains its stricter original leading-separator policy after shared file classification, so parser normalization cannot erase the evidence that policy needs.

The fixed-U package contract was also inspected from the hash-verified fs-safe archive: local-file-access.js delegates to Node fileURLToPath after structural local-host and encoded-separator admission, then applies Windows alias/network checks. Its actual local-file-access member SHA-256 is 7530a957ca9d56f56deba8da1dd06be03c894c6f6472bc8571080937b57bfda8. windows-path-syntax.js and windows-path-alias.js establish the rooted drive/namespace and colon distinctions reused by the Rust guard. Fixed-U web-media.ts 553–560 and 638–650 show actual safe conversion before local loading and the typed invalid-file-url mapping; local-media-path.ts and its Windows web-media test were inspected. This is scoped dependency/actual-use review, without claiming that Node and Rust have identical behavior for every path spelling or dot-normalization case.

## Production integration and structural quality

The full current batch converges on one existing media-reference owner. media_file_context.rs 169–207 delegates file sources to the Result boundary before its existing non-file/home/native-path handling. Its Option wrapper returns None on refusal. channel_attachment.rs 60–75 checks the final reanchored path before metadata and the existing bounded reader. Exact-base host_paths.rs::reanchor_sand_path was inspected: it does not itself access the filesystem; the added post-reanchor guard protects the actual path passed onward. AttachmentsService::resolve_channel_attachment delegates to this consumer, and cross_user_sharing/production.rs 385–397 consumes its image upload result while preserving the existing None/omission behavior.

web_media.rs 294–324 maps shared errors to invalid-file-url and retains its original prefix restriction and resolved-network refusal. The admission function at 336–385 still performs no filesystem operation before invoking the private local reader. The final raw/resolved path spelling check is before that callback. Roots, sandbox options, HTML policy, image handling and byte caps remain with their current owners. The service's exact-base loadWebMedia arm passes the existing size conversion/options and calls this same loader. The resolveLocalMediaPath arm retains its current string/null result. No new public Gateway registration is introduced.

App main.rs::resolve_production_attachment_source now consumes the shared Result at lines 639–653. A refused file URL returns an error before fs::read at line 659, ForeverBox status/download at 678–699, or media persistence. Only Ok(None) takes the existing non-file passthrough. Both actual sinks propagate that Result at 899–905 and 1180–1186. file_path_from_file_url remains an Option facade for metadata/derived-local-URL use, not the App's refusal boundary. The existing SendMessageToolBridge error branch at send_message_tool.rs 405–409 invokes abort_attachment_custody when a later image fails; the existing App abort path owns rollback and durable admission release. The batch adds no second cleanup owner.

AgentManagement has a distinct per-image loop and no corresponding SendMessage per-call rollback. sand_agent_management_tools.rs::parse_images, lines 93–105, now validates every file URL before it can return a vector; the caller at 61–65 starts resolve_image_source only after that full Result succeeds. This is a pure value preflight through the same owner. It prevents an earlier valid image from materializing before a later invalid URL is discovered, while leaving later read/persistence/send failure handling outside this narrow claim. Keeping the actual App single-image check is appropriate because the sink also has direct callers.

The actual App constructs both private production sinks at main.rs 11554–11589 and supplies them to the production composition at 12319 and 12331. Exact-base runner_production_bridge.rs 191–221, TurnAgentComposition's sink projection, and runner/tools/turn_toolset.rs 752–754 and 824–830 confirm that the production bridge installs the existing AgentManagement and SendMessage tool adapters. The change is therefore reachable through current composition, not merely exposed as an unused helper.

No new dependency direction, global state, lock, cache, persistent schema, async lifecycle or registration is introduced. The shared error type already belongs to the media-reference owner; adapters retain their existing public Result/Option/typed-string forms. The narrowly retained raw-prefix helper encodes a stricter web-only policy and does not duplicate structural URL parsing. The AgentManagement preflight repeats admission at a distinct before-side-effects boundary, rather than copying URL implementation logic.

## Performance and preserved readers

The R4 condition examines four bytes. On nonmatching Windows paths it adds no explicit pathname clone or setter call. On Unix it adds no runtime work. The matching branch clones the serialized pathname, performs an equal-length replacement, and calls the URL owner's setter. That setter also saves/restores any query/fragment and uses its existing path parser. The repair adds a bounded number of passes and linear temporary storage relative to the affected serialized URL; it introduces no loop over previously visited prefixes, per-byte heap allocation, repeated filesystem probe, network operation or extra native read. This is a source complexity assessment, not a measured benchmark or a complete complexity proof of the dependency.

Across the broader batch, centralized admission replaces separate conversion/fallback logic rather than adding a second policy implementation. AgentManagement can parse a URL in preflight and again at actual resolution; the work remains proportional to the image URLs, and the repeated boundary protects against early materialization. No unrelated caching or abstraction is warranted by this repair.

The existing BR reader production slice, from MEDIA_READ_CHUNK_BYTES through the bounded-reader functions before its test module, is byte-identical to the exact base. Independently compared web_media.rs::load_local_web_media and file_url_has_extra_leading_path_separator are also byte-identical to the base. The 64 KiB scratch read, one-byte over-cap probe, opened-file metadata admission, bounded/geometric reservation and error handling are preserved. Channel still uses that shared reader. The App's existing fs::read/materialization/fallback body remains its existing responsibility; this source approval does not claim the App has acquired the BR reader's byte-bound or unknown-write-outcome guarantees.

## Current test/oracle and capture review

The R4 test changes extend existing selected tests rather than changing selection counts. local_media_file_url_contract.rs 32–56 adds the four native pipe-drive URL shapes with exact PathBuf equality. Lines 70–74 add remote pipe authorities. Lines 147–161 retain Unix colon/pipe preservation. Lines 166–188 add encoded/bare/relative pipe, ADS, encoded separators, invalid UTF-8, NUL and malformed-percent negatives. web_media.rs 572–627 exercises those Windows negatives through the existing zero-reader-entry seam for the four root/sandbox option combinations.

openclaw_web_media_contract.rs 269–275 preserves all four rooted-drive positives and labels the failing source in diagnostics. Its actual native byte/service/cap/retry test at 279–331 adds triple-slash, localhost, single-slash and authority-position pipe forms, each with and without encoded separator text in query/fragment. It checks the exact path, bytes, filename, HTML type, real service result, size-cap refusal and later valid recovery. The existing no-share negative seam remains appropriate; none of these negative controls opens a live share or device.

The App test file was read in full. Its observation sink delegates to the actual private production sink and records real admitted bytes and custody before the next image, so the rollback control cannot pass because the first image never materialized. The six causal oracles are: early invalid-source refusal with unchanged Session/transcript/media/source and zero send; successful first-image custody then actual bridge rollback on later refusal; native/Unicode materialization and persisted exact metadata across real owner reopen; repeated refusal plus a fresh valid send before and after reopen; direct and batch AgentManagement refusal with unchanged source/target state, no materialized earlier image and no wake, followed by a real positive materialization/wake; and HTTPS passthrough without local custody. Existing idle_runner_owners and its scoped disposal are reused. Controlled final transport/wake observation is explicitly narrower than real provider or channel acceptance.

Static reading/counting of the capture source, without importing or executing it, confirms 16 native command groups, 46 input paths, 163 Linux/macOS and 158 Windows expected positives. The shared Channel target executes once. Existing HHP/IPB/BR controls remain selected. capture_media_cleanup.py 268–315 still binds committed/normalized inputs and requires exact unique positives, no unexpected positives, exact zero-failure/zero-ignored summary and both command/capture success. Lines 381–407 retain exact H, dispatch, native OS and oracle-qualification checks; lines 463–477 retain before/after input identity and fail-closed completion.

The current WFX extracted portable script remains called by the existing Linux workflow step at workflow lines 3594–3604. Its generated-send-message-contracts case retains all three targets and now includes the two facade cases; its exact summary requirement remains 13/14/17 at script lines 974–976, with required-name uniqueness enforced by the surrounding gate. The workflow's existing macOS required-positive list also includes both facade cases. Native media capture remains wired on Linux, macOS and the explicit windows-2025 job at workflow 4381–4425. The App workflow includes the added source/test input bindings, selects the actual App test executable, and requires all six exact App cases through listing and one-pass/zero-fail/zero-ignored execution at 4160–4183. Source-reviewed wiring and planned counts are not pass receipts.

## Disposition of all 21 current non-evidence paths

All paths below are relative to the repository root. APPROVED means the current changed hunks and required adjacent context satisfy this source-review lens, subject to the execution limits below.

| Path | Current-source disposition |
| --- | --- |
| .github/workflows/rust-desktop-runtime.yml | APPROVED: facade positives, App input bindings and six exact App case selections; current extracted portable/native wiring retained. |
| projects/openclaw-rust/docs/specs/local-media-file-url-admission.md | APPROVED, R4 focus: full LFA contract and LFA-03a define the narrow Windows repair, unchanged negative classes and execution boundary without relabeling the failed raw assertion. |
| projects/openclaw-rust/docs/specs/media-file-url-admission.md | APPROVED: full FUA contract retains prefix/host-erasure/native-root and reader boundaries, typed errors and open broader scope. |
| projects/openclaw-rust/parity-ledger.json | APPROVED: four affected responsibility rows describe shared admission, actual consumers, tests, risk and safe rollback. Source comparison found no changed status/global flag fields. Historical evidence JSON content remains outside this review. |
| projects/openclaw-rust/requirements-traceability-matrix.json | APPROVED: six rows add relevant evidence/source references only; no status/global flag promotion. Reference content in excluded evidence files is root-owned. |
| projects/openclaw-rust/scripts/capture_media_cleanup.py | APPROVED: extends existing source-bound native harness, preserves exact/unique positive and error gates, one shared Channel target and honest scope. |
| projects/openclaw-rust/scripts/capture_portable_contracts.py | APPROVED: two facade names added to current extracted three-target command; matching 13/14/17 gate preserves original targets and assertions. |
| source/host/app/src/main.rs | APPROVED: Result boundary before read/Box/persistence; both real sinks propagate refusal; existing composition/custody preserved; App tests registered in existing test module. |
| source/host/app/src/tests/local_media_file_url_tests.rs | APPROVED: full six-case actual-owner fixture/oracles reviewed; observation wrapper delegates real resolution; reopen and cleanup are fixture-owned. |
| source/host/src/connectors/channel_attachment.rs | APPROVED: final reanchored spelling guard before metadata/read; existing 50 MiB bounded consumer, MIME/result handling retained. |
| source/host/src/extensions/attachments/media_file_context.rs | APPROVED: delegates file URLs to shared Result owner, preserves Option surface and ordinary non-file/home handling, guards resolved local spelling. |
| source/host/src/extensions/attachments/media_reference.rs | APPROVED, R4 focus: single pure owner, original authority evidence retained, narrow cfg(windows) setter normalization before strict decode/native/alias checks; BR bodies preserved. |
| source/host/src/extensions/attachments/web_media.rs | APPROVED, R4 focus: only test-table R4 change; whole batch retains adapter errors/raw-prefix policy/private no-I/O seam and unchanged local read body. |
| source/host/src/runner/tools/sand_agent_management_tools.rs | APPROVED: complete image-value preflight through shared owner before any resolver entry; no new state or rollback owner. |
| source/host/src/runner/tools/send_message_tool.rs | APPROVED: existing Option metadata facade delegates shared conversion; real App consumes Result separately; existing abort path remains applicable. |
| source/host/tests/channel_attachment_contract.rs | APPROVED: real native bytes/metadata, strict invalid encoding with valid retry, pure Windows network/alias refusals. |
| source/host/tests/local_media_file_url_contract.rs | APPROVED, R4 focus: typed shared classification, exact native path positives, remote/encoding refusal, stateless recovery and platform-specific pipe/alias controls. |
| source/host/tests/openclaw_media_file_context_contract.rs | APPROVED: correct native cross-platform positive, Unix retained controls, Windows drive-less refusal, actual service/Option and alias tests. |
| source/host/tests/openclaw_web_media_contract.rs | APPROVED, R4 focus: retained root positives with diagnostics, real pipe-variant path/byte/service/cap/retry oracle and existing alias/encoding controls. |
| source/host/tests/sand_agent_management_tools_contract.rs | APPROVED: later invalid URL proves zero image-resolver/send entries, with local/HTTPS admitted control. |
| source/host/tests/send_message_tool_contract.rs | APPROVED: facade preserves admitted filename/original URL and refuses unsafe sources without claiming metadata facade is the production refusal boundary. |

## Limits and required next evidence

No product/parser execution, test, build, lint, benchmark, capture, schema check, generator or acceptance command was run locally or on htch. Commands used source/Git/archive reads, hashing and static textual/JSON field comparison only; neither capture script nor any product dependency was imported or executed. No product/shared/snapshot file was changed. The only reviewer write is this report.

The two adjacent original Windows subjects e19a0a265e400a4c9b1d96520c84fb612a3843e8 and 3438eb5bac755c399c9ec686414bbb4c0816888d retain the supplied 22-pass/1-fail web-target history. The raw failing assertion does not identify a particular variant; this report does not infer such a label. The pipe-delimiter diagnosis above is derived from the locked source and repair, not an executed R3/R4 result. Those prior results do not execute this candidate.

Exact published-H GitHub Actions, including native Windows and every selected native/App/capture gate, remain required. The source verdict does not approve broad read capability/root identity, filesystem races, all Windows aliases/device opens, persistence unknown outcomes, real provider/channel/node/device behavior, public Gateway exposure, packaged platforms, full media parity, or any global completion flag. No earlier source approval or different-H runtime receipt is carried forward as current acceptance.

**Final source verdict: APPROVED.** Findings: none.
