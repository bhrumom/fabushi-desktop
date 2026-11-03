# Media admission R2 — independent GENERAL / security / correctness review

## Terminal verdict

**CHANGES_REQUIRED — one P2 source finding blocks the declared Windows contract.**

The complete-image admission repair for `SendToAgent` is causally correct by source inspection. The frozen R2 shared converter nevertheless retains a concrete compatibility defect for `file:///C|/`: the pinned `url` parser does not normalize that spelling to a colon drive, and R2 rejects it before native conversion. The retained Windows contract explicitly requires it to resolve to the root of drive C. The finding below identifies the dependency path and the R2 refusal without claiming that R2 was executed.

This is a terminal independent **source** review of the frozen tree below. It is not product acceptance, an Actions pass, or permission to promote any completion flag. No product, parser, build, test, lint, schema, capture, generator, benchmark, or acceptance command was executed locally or on htch.

## Review identity and authority

| Field | Exact identity |
| --- | --- |
| Reviewer | `/root/media_admission_general_r2` — fresh blind GENERAL/security/correctness R2 |
| Effort / model | Actual inherited launch effort `xhigh`, as assigned; configured model retained, no override. Scope includes parser security, Windows conversion, byte preservation, production custody, restart/recovery and capture integration. |
| Repository / branch | `bhrumom/fabushi-desktop` / `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Base and starting HEAD | `0e845ce1679e0cdf95346adc54f5d58bf06464e8` |
| Base tree | `4e69dc6d77f4ff4508e02cdd7064084530db9b6e` |
| Reviewed candidate tree | `a4f1c59eb577a45c1b05da1de0a8f13644dc1a5c` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Prior tree used only as allowed source context | `3129fb0c3897bdf78d50e1cd067cc3dd65310297` |
| Frozen current / before images | `/workspace/scratch/7d27ea239237/media-admission-r2` / `/workspace/scratch/7d27ea239237/media-admission-r2-base` |
| Manifest | `/workspace/scratch/7d27ea239237/media-admission-r2-manifest.json`, SHA-256 `6c2623df42cd54ea2cf8f5d5d15546b1bbc6b1cf647f174b6fe17ab670519e57` |
| Complete patch | `media-admission-r2-complete.patch`, 183,744 bytes, SHA-256 `4900c4d4d87ba403efa762a9bc6dc0ea9272805eb7b1c48a4b7e38f2b2988c3b` |
| Repair plus incoming patch | `media-admission-r2-fix-and-incoming.patch`, 202,088 bytes, SHA-256 `697877b87e297675e79b09d204b9210df020c4ef72a0d0bea3338fc90962c803` |

I independently checked the complete 22-path manifest against local SHA-256 and Git blob identities. API readback of the candidate recursive tree returned 10,501 entries with `truncated=false`; the base returned 10,496 entries with `truncated=false`. Their changed non-tree entries are exactly the 22 declared paths, with the declared candidate and before-image blobs/modes. These are source identity checks, not executable validation.

Governing reads included root `AGENTS.md` blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2`, OCRM `SOURCE_OF_TRUTH` blob `3748d734b10c9ed5b2d948467fbaa9483f1dd306`, main Spec blob `69c9667af2328e20a1fba09ebf80c5aab4d5c336` with particular attention to §17, the full-fusion-batch prompt blob `5ff7894d245588ed539ab075b09215a2f1a82716`, the module map and both scoped admission specifications. Existing-owner and exact-source/Actions-only requirements were also checked against the adjacent communication architecture guidance. Product and mixed source received deep review; tooling, specifications, evidence and ledger changes received change-specific evidence/contract review.

I did not read earlier or other reviewers' reports, did not inspect mutable foreign checkout source, did not alter the candidate, and did not spawn another reviewer. Adjacent unchanged source was read through exact-base Git objects or exact upstream/API/archive identities. This report is the only file written by this reviewer.

## Finding R2-GEN-01 — valid pipe-drive URL is rejected by the shared Windows owner

**Severity: P2; blocking the declared native Windows contract.**

**Affected source:** `source/host/src/extensions/attachments/media_reference.rs:156–173`, particularly the decoded drive check at line 163, as consumed by `web_media.rs:294–315` and the production App consumers. Required oracle: `source/host/tests/openclaw_web_media_contract.rs:269–271` in `test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references`.

**Trigger and observable failure:** on Windows, call `resolve_local_file_url("file:///C|/")`, or the public `file_url_to_path` wrapper. The retained oracle requires `PathBuf::from(r"C:\")`. In the frozen R2 source, the shared owner instead returns `InvalidPath("Local file URL must contain an absolute Windows drive path")`; the web wrapper returns `LocalAccess` with code `invalid-file-url`. A nested local file such as `file:///C|/safe/image.png` has the same refusal. No filesystem or remote service is needed to trigger this defect.

**Source-causal trace, using the hash-verified published `url@2.5.8` archive:**

1. `src/parser.rs:1062` discards the first boolean returned by `Parser::file_host`. For the empty host in the triple-slash spelling, `remaining` still begins with `/C|/`; `parse_file_host` returns `has_host=false` at line 1079.
2. `parse_file`, lines 532–541, treats that false value as the path-start choice. It inserts a slash into the serialization and then calls `parse_path` with the still-leading slash in `remaining`.
3. `parse_path` sets `segment_start` to the current serialization length at line 1221. The remaining leading slash creates an empty initial segment. Consequently the subsequent `C|` segment begins at `path_start + 2`, not `path_start + 1`.
4. The only pipe-to-colon normalization is guarded by `segment_start == path_start + 1` at lines 1345–1347, so this `C|` is not normalized. Lines 1371–1378 subsequently strip initial empty slash segments, but do not revisit the drive token. The resulting pathname is `/C|/`.
5. R2 strictly decodes that pathname, then requires `decoded[2] == b':'` at `media_reference.rs:163`. It returns an error for `|`. Merely loosening this precondition is insufficient: the same pinned dependency's Windows conversion at `src/lib.rs:3142–3146` rejects a two-byte first segment unless its second byte is `:`.

This is distinct from `file:///C%3A/`. The dependency explicitly supports `C%3A` and `C%3a` in `src/lib.rs:3151–3162`, and has an encoded-colon unit oracle at `tests/unit.rs:217–219`. It is also distinct from the single/double-slash pipe spellings covered by the dependency's `tests/urltestdata.json:6704–6729`; those paths do not take the same extra-empty-segment branch. My initial broad assumption that the dependency normalizes every pipe-drive spelling was corrected by the complete source trace above.

**Why this is in scope:** LFA-03 requires ordinary rooted-drive compatibility, LFA-06 preserves valid original assertions, and the incoming four-element rooted positive matrix is explicitly retained in this candidate. The defect survives the newly centralized converter and leaves its mandatory Windows suite unable to satisfy that oracle. It is an inherited dependency compatibility defect carried into the shared integration, not a regression caused by the `SendToAgent` preflight itself.

**Concrete repair:** reconcile this parser normalization gap inside the shared local-file owner before its Windows drive precondition and native conversion. Keep authority admission and the web adapter's original-spelling separator guard authoritative. A narrowly recognized literal initial `/[A-Za-z]|/` drive token can be normalized through the parsed URL/native-conversion path; do not normalize arbitrary interior pipes or treat percent-escaped `%7C` as a raw drive delimiter. Do not fall back to an original URL or an unchecked reconstructed path when conversion fails. Preserve bare-drive, remote/erased authority, malformed percent, encoded separator, NUL, UTF-8 and ADS refusals.

Retain all four existing rooted positives and add a direct shared-owner Windows assertion for the repaired spelling, with a variant-bearing assertion message so a future failure identifies its input. The existing native Windows Actions gate must then execute the corrected exact subject; a Linux source/read or test result cannot establish Windows conversion.

**Adjacent runtime evidence, kept separate:** the integration owner reported exact subject `e19a0a265e400a4c9b1d96520c84fb612a3843e8`, run `41571103928`, Windows job `125703932910`, with 22 passed and 1 failed in the older web target. Its panic at the old test line 253 was `rooted drive remains valid: LocalAccess { code: "invalid-file-url", message: "File URL cannot be converted to an absolute local path" }`; the loop did not print the variant. I independently read that subject's immutable web converter, blob `9912720f7c6eff6b53444b1912e3b291613d33e4`, and confirmed its old segment precondition and `to_file_path` path. I did not independently fetch or execute the raw job log. The source trace identifies a concrete failing R2 input consistent with that adjacent failure; it does not relabel the adjacent result as R2 execution or claim that the unlabeled panic itself proved the variant.

## Repair assessment: the decided SendToAgent custody scenario

The change to `sand_agent_management_tools.rs:93–105` resolves the supplied failure scenario at its actual cause. `parse_images` completes schema and shared local-file admission for the whole value vector. Only after it returns successfully does `call_tool`, lines 61–65, enter the iterator that invokes `resolve_image_source`. A valid first image followed by an invalid second file URL therefore fails before the first resolver can materialize anything.

This placement matters because `ProductionAgentManagementSink::resolve_image_source` uses the real production attachment resolver but does not have the separate `SendMessage` per-call media rollback map. Adding a later error propagation alone would leave the earlier managed file without that custody. The new preflight introduces neither a new state owner nor a competing cleanup protocol; it prevents the deterministic admission failure from creating effects in the first place.

The new App case, `local_media_file_url_app_agent_management_refuses_image_before_outward_send`, is a causal regression oracle rather than a resolver-only fake. It uses the actual `ProductionAgentManagementSink`, `ProductionAgentToAgentMessaging`, Session and transcript owners. It repeats valid-first/invalid-second attempts across two freshly constructed owner cycles, retains source and target transcript/admission snapshots and managed-file inventory, checks original source bytes, and requires no wake or outgoing-message observation. The successful control then materializes one real local image, verifies its exact bytes and the source outgoing transcript, inspects the actual messaging wake's selected-image byte payload, and reopens again to verify persisted transcript and bytes. The controlled wake port does not execute a target Agent, which the test explicitly acknowledges.

The paired `SendMessage` test separately proves its existing custody mechanism: the first image is actually materialized and registered under the real per-call Session media identity; the later refusal invokes the actual abort path and removes only the owned materialized file while preserving the source. The native/unicode success and retry cases explicitly drop bridge/observer references before reopening owners. These are sufficient source-level causal oracles for the stated repair; they are not execution evidence and do not close unrelated late read, persist, delivery, crash or unknown-commit cleanup responsibilities.

## Other security, compatibility and byte-path assessment

Except for R2-GEN-01, I found no additional concrete blocking source defect in the declared scope.

- The shared owner preserves `Result<Option<PathBuf>, MediaReferenceError>`: non-file sources are distinct from invalid file sources. Actual App call sites consume the error before local read, Box lookup, persistence or original-URL passthrough. The Option facade is limited to the existing metadata-oriented `SendMessage` helper.
- The URL parser's `FileWithHostAndWindowsDrive` callback preserves refusal when the pinned parser erases a remote host. Normalized authority, userinfo and port checks remain before conversion. The malformed-file classifier accounts for the parser's C0/space trimming and ignored ASCII tab/CR/LF without becoming a second path parser.
- Percent checks are confined to the pathname. Malformed octets, encoded slash/backslash, invalid UTF-8 and NUL are refused; valid query/fragment percent text does not become a path error. Decoding occurs once. The Windows absolute-drive precondition still prevents the dependency's bare-drive debug assertion; fixing the pipe-drive mismatch must preserve that protection.
- `admit_local_media_path` rejects NUL on all platforms and Windows network/device/drive-relative/extra-colon aliases before metadata/read. Rooted extended-drive spelling is retained where the existing filesystem contract allows it. Unix colon filenames remain admitted. The Channel path is guarded again after legacy reanchoring.
- The web raw-prefix helper is retained unchanged and runs only after the shared owner identifies a file source. It preserves localhost followed by repeated path separators, all-platform strict network-like spelling refusals, the ordinary `file://C://...` interior separator positive and query/fragment positives. The local reader callback remains the only reader boundary; no new public Gateway command or caller-provided reader bypass is introduced.
- The incoming BR-v1 read implementation, private/public/service tests and cap conversions remain intact. The reader still rechecks opened-file metadata, bounds actual bytes with a one-over sentinel, supports short/interrupted reads, discards prefixes on late/allocation errors and uses Unix nonblocking open for the existing FIFO replacement scenario. No `max + 1` overflow or metadata-sized allocation was introduced.
- The existing App Session/transcript/media owners remain responsible for custody and persistence. The resolver still returns owned-path information to `SendMessage`; its exact-call abort and commit paths remain composed. No second media lifetime, transcript, Session or messaging owner appears.

These conclusions do not certify root-capability or TOCTOU confinement, complete native filesystem alias parity, provider/channel/device compatibility, full MIME/codec policy, generated-HTML trust closure, arbitrary write-failure cleanup or crash/unknown-effect acceptance. The scoped specifications and evidence correctly leave those responsibilities open.

## Capture and real App integration disposition

The native capture source selects the following exact group cardinalities. They are **expectations, not passing results**.

| Group | Linux / macOS | Windows |
| --- | ---: | ---: |
| cleanup-owner | 7 | 8 |
| cleanup-consumers | 17 | 16 |
| html-policy | 12 | 9 |
| host-read-format | 1 | 1 |
| file-url-admission | 5 | 6 |
| explicit-html-consumers | 28 | 26 |
| image-probe-consumers | 16 | 16 |
| bounded-reader-owner | 11 | 8 |
| bounded-reader-public | 17 | 17 |
| bounded-reader-box-consumer | 1 | 1 |
| bounded-reader-image-consumer | 1 | 1 |
| bounded-reader-channel-consumers | 7 | 8 |
| local-file-url-owner | 10 | 11 |
| file-context-consumers | 10 | 10 |
| send-message-consumers | 17 | 17 |
| agent-management-consumers | 3 | 3 |
| **Total: 16 groups** | **163** | **158** |

There are 46 bound native inputs. The common Channel target executes once with both LFA and incoming BR controls. `assess` requires each exact positive name once, no extra positives, one exact zero-failure/zero-ignored summary, successful cargo/capture exits and a nonempty unique expected-name list. The 13 synthetic result-oracle qualification controls remain. Exact H/U/platform checks and before/after input identities remain in the capture source. I did not execute the script or its controls.

Both workflow `SendMessage` required-name lists include the two added facade cases. The `generated-send-message-contracts` receipt keeps the full three-target summary oracle at **17 / 14 / 13**, including zero failures and ignored cases; it does not silently replace the full target with a new-name filter.

The existing Linux actual-App step binds the new inputs, compiles the shipping non-test App and linked test harness, verifies exact test selection and selects these six complete names:

1. `tests::local_media_file_url_tests::local_media_file_url_app_rejects_invalid_before_materialization`
2. `tests::local_media_file_url_tests::local_media_file_url_app_rolls_back_earlier_image_on_later_refusal`
3. `tests::local_media_file_url_tests::local_media_file_url_app_commits_native_and_unicode_media_across_reopen`
4. `tests::local_media_file_url_tests::local_media_file_url_app_repeated_refusal_then_repair_survives_reopen`
5. `tests::local_media_file_url_tests::local_media_file_url_app_agent_management_refuses_image_before_outward_send`
6. `tests::local_media_file_url_tests::local_media_file_url_app_preserves_https_without_local_custody`

Each is listed by exact name before execution and requires successful one-passed/zero-failed/zero-ignored completion. The real App module declaration, helper visibility and production composition were source-checked. The fixture controls external ports and owns its temporary root; this does not establish real external service compatibility.

The frozen workflow is 532,528 bytes and retains the supplied inherited admission-size blocker. H497 run `41569288081` failed with zero jobs. A separate existing WFX owner is repairing extraction; this review does not request a duplicate source repair or transfer any pass from that work. The extraction/integration's final exact H must be reviewed and run after R2-GEN-01 is resolved. The later adjacent e19 Windows failure is a separate runtime fact, described above.

## Complete 22-path disposition

All paths below are relative to the frozen candidate root identified above. Exact blob/SHA identities are bound by the independently checked manifest. “Source acceptable” is limited to this review and does not mean executed or fully accepted.

| Changed path | Review disposition |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | All changed capture/name/count/App-input hunks checked against adjacent execution/receipt code. 17/14/13 and six actual-App cases preserved. Source integration acceptable; inherited size blocker and exact-H execution remain open. |
| `projects/openclaw-rust/docs/specs/local-media-file-url-admission.md` | LFA contract, actual-owner placement, R2 batch preflight, incoming preservation and explicit remaining scope read. Source declaration is coherent; R2-GEN-01 violates its rooted-drive/preserved-oracle obligation. |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | FUA normalization recovery, authority/reader boundary and retained restrictions checked. No false global closure; required native Windows positive still needs the finding's repair. |
| `projects/openclaw-rust/evidence/local-media-file-url-647e0932-20261103.json` | Contract/oracle/input links and 16/163/158/46/6 source expectations checked. Explicit `not-run`, no scoped/full-unit authority or completion promotion. Not execution evidence. |
| `projects/openclaw-rust/evidence/media-file-url-normalization-recovery-647e0932-20261103.json` | Historical failure subjects/results remain historical; current repair state remains pending. Acquired archive identity is separated from runtime verification. No prior reviewer report was opened. |
| `projects/openclaw-rust/parity-ledger.json` | Four affected responsibility rows reviewed against their exact-base delta and current owners. Dependencies/tests/evidence/invariants extended without status or global completion promotion. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Changes append evidence references for R15/R23/R24 and AC15/AC23/AC24; no requirement or acceptance status promotion. |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | Full capture source, platform selections, 46 inputs, strict result oracle and 13 qualification controls reviewed. Complete LFA/FUA/BR union preserved; R2-GEN-01 remains observable in its Windows web target. |
| `source/host/app/src/main.rs` | Production resolver, both real sinks, custody/abort/commit, messaging composition and new test-module hookup reviewed with adjacent exact-base owners. Typed refusal before effects is correct; shared-converter finding propagates to its Windows input boundary. |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | Entire new actual-App fixture and six cases reviewed. Real materialization, owned bytes, transcript/admission snapshots, repeated refusal, owner reopen and successful controls are causally meaningful; external ports remain controlled. |
| `source/host/src/connectors/channel_attachment.rs` | Complete consumer reviewed; admitted source plus post-reanchor guard precede metadata/read. Existing URL forwarding, caps, bytes and metadata behavior preserved. |
| `source/host/src/extensions/attachments/media_file_context.rs` | Shared tri-state replaces duplicate URL conversion; refusal cannot become fallback path. Native/plain-path projections and existing service semantics retained. |
| `source/host/src/extensions/attachments/media_reference.rs` | Complete new admission and adjacent retained BR reader/private tests reviewed. **R2-GEN-01** in Windows normalization/conversion boundary; other concrete safety checks and BR retention acceptable by source. |
| `source/host/src/extensions/attachments/web_media.rs` | Complete admission path and adjacent reader/format/optimization behavior checked. Retained raw-prefix helper and incoming BR code preserved. Wrapper exposes **R2-GEN-01** without masking it. |
| `source/host/src/runner/tools/sand_agent_management_tools.rs` | Full bridge/schema/preflight/resolution flow read. Decided later-invalid image scenario repaired before all resolver entry. No second cleanup/state owner. |
| `source/host/src/runner/tools/send_message_tool.rs` | Shared Option facade, original source metadata, sequential resolution, abort/error and observed shipping paths checked. Incoming SGF behavior preserved. |
| `source/host/tests/channel_attachment_contract.rs` | Full existing and new native URL/bytes/refusal/retry/Windows cases reviewed. Shared target retained once in native capture. |
| `source/host/tests/local_media_file_url_contract.rs` | Entire shared-owner matrix reviewed. Non-file, authority, malformed/path byte, Unicode, stateless and native Windows cases present. Add direct raw pipe-drive coverage when repairing R2-GEN-01. |
| `source/host/tests/openclaw_media_file_context_contract.rs` | Native full-path replacement for the old cross-platform POSIX oracle, retained Unix control, Windows refusal and new shared-owner service/projection cases checked. No valid assertion was removed to pass Windows. |
| `source/host/tests/openclaw_web_media_contract.rs` | New and retained admission/service/native byte/BR/root-alias controls reviewed. Four rooted-drive positives remain at lines 269–271; their third input is the causal oracle for **R2-GEN-01**. |
| `source/host/tests/sand_agent_management_tools_contract.rs` | Existing two tests retained; new whole-batch test requires zero resolver/send entries for a later-invalid URL and ordinary valid recovery. Matches the production preflight order. |
| `source/host/tests/send_message_tool_contract.rs` | Existing complete 15-case contract preserved; two shared-facade cases bring the target to 17. Full-target SGF/native capture expectations updated consistently. |

## Dependency and adjacent-owner disposition

The published `url-2.5.8.crate` at `/tmp/fabushi-647e-pairing/local-file-url-dependency/url-2.5.8.crate` has SHA-256 `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`. Its actual parser, native conversion and relevant unit/WPT source were read directly from the archive; both Host and App lock this version/checksum. The pipe-drive finding depends on those exact bytes, not on remembered URL behavior.

The published `fs-safe-0.27.0.tgz` at `/tmp/fabushi-647e-pairing/media-fixed-dependency/fs-safe-0.27.0.tgz` has SHA-256 `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`. Its actual local-file-access, Windows-alias and Windows-syntax modules were read. Fixed-U local media path, Windows tests and web-media source were obtained by exact GitHub blob identity. The upstream contract delegates URL conversion to Node after its authority/path checks; the target must reconcile the pinned Rust dependency's relevant observable mismatch without weakening those checks.

Adjacent exact-base reads covered attachment persistence/root reanchoring, tool URL schema, Session/media/transcript ownership, agent messaging and selected-image loading, real App fixture construction/disposal, attachment service dispatch and public Gateway registration, Cargo declarations/locks and existing byte-reader consumers. There is no new dependency or product owner in the patch. The only requested repair is the shared conversion boundary above, followed by fresh focused review and exact-H execution.

**Final disposition: CHANGES_REQUIRED. One concrete P2 finding; SendToAgent custody repair source-confirmed; execution and full acceptance remain open.**
