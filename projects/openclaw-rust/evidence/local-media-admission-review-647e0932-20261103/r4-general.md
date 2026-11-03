# LFA/FUA current-source review — GENERAL, security and correctness — R4

## Verdict and scope

**APPROVED for source convergence of the 21 non-evidence paths in the frozen R4 scope.** I found no actionable correctness or security defect in the current changed hunks or the affected functions and adjacent consumers. This includes the five-path Windows drive-delimiter repair and the complete integrated LFA/FUA changes against the exact 3438 base. This is not executable validation, a claim that the earlier Windows failure has been observed fixed, or full product acceptance.

- Task: `647e0932-bea4-4113-9a61-65ae3db58955`, round 1, review round 4.
- Repository/branch: `bhrumom/fabushi-desktop`, `codex/openclaw-rust-fusion-chat-20261101-writer`.
- Frozen snapshot: `/workspace/scratch/7d27ea239237/media-admission-r4`.
- Exact base: `3438eb5bac755c399c9ec686414bbb4c0816888d`, tree `b544c1d377ca82618cdadcf5106cb3c10d82411b`.
- Candidate tree: `76e6c26d754179ee5af447ecec689a2de844b476`.
- Previous immutable source snapshot: R3, tree `e9a172d00139550e8a055a61bf06e225b9d4166c`.
- Fixed OpenClaw U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Lens/effort: fresh GENERAL/security/correctness review; configured model retained; the parent launch explicitly records actually applied `xhigh`. No sub-reviewer or replacement model was started by this worker.

The two frozen `projects/openclaw-rust/evidence/*.json` files were checked for byte/hash identity only. Their contents are explicitly excluded from this blind review and assigned to the root integrator for serial provenance/status audit. The source verdict does not approve their historical narrative or embedded decisions.

## Independence and method

I read the complete LFA specification, including LFA-03a, and the complete FUA specification before reviewing product code. Applicable authority was read through immutable Git objects: root AGENTS `e4682748087759b2d4bb2b3a47916b6148e8a6f2`, Source of Truth `3748d734b10c9ed5b2d948467fbaa9483f1dd306`, and main specification `69c9667af2328e20a1fba09ebf80c5aab4d5c336` §17. Exact-base tree inspection found no additional AGENTS file governing these paths.

No prior/other review report, root arbitration/decision file, or excluded evidence JSON was opened. Complete-patch reads skipped the two evidence JSON sections. The required specs themselves contain historical phrases such as “Prior FUA source approval” and “R1 source convergence was not runtime acceptance”; that limited exposure was disclosed to the parent. No earlier finding, rationale, or verdict was used as proof for the present implementation. All 21 current non-evidence changed-path dispositions below are based on source inspected in this review.

The work used source-byte/hash comparison, immutable Git reads, dependency-archive member reads and original CI-log archive reads. No product, URL parser, test, build, lint, capture, generator, schema checker or acceptance program was executed locally or on htch. No snapshot or foreign working file was modified. This report is the only authored file.

## Findings

**Blocking findings: none. Non-blocking findings: none.** There is no severity/file/function/line defect finding requiring a repair. The causal checks and exact source locations supporting that disposition follow; pending execution is recorded separately rather than converted into a defect or a passing result.

## Windows repair assessment

### Causal dependency boundary and intended behavior

The published `url 2.5.8` archive confirms the diagnosis independently of the prior panic text. `parse_file_host` returns the empty/localhost-host branch with a false flag (`parser.rs:1057–1079`). `parse_file` then adds a slash before calling `parse_path` with the remaining input (`parser.rs:532–542`). First-segment Windows-drive normalization requires `segment_start == path_start + 1` (`parser.rs:1344–1363`), while initial empty file segments are only removed afterwards (`parser.rs:1371–1378`). Consequently a triple-slash literal pipe drive can survive as `/C|/…`. The native Windows converter accepts a literal colon or `%3A`/`%3a` in its first drive segment, not a pipe (`lib.rs:3140–3165`).

The current repair in `media_reference.rs::resolve_local_file_url`, lines 119–140, operates at precisely that boundary. On Windows only, it requires the parsed literal prefix `/`, ASCII letter, `|`, `/`; copies that pathname; replaces bytes 2..3 with `:`; and calls the existing URL setter. It never constructs a native path or retries a failed conversion through a fallback.

### Security and language correctness

- **Authority survives normalization as a refusal.** The original parse records `FileWithHostAndWindowsDrive` in a `Cell`, and lines 108–117 reject that flag, any remaining nonlocal host, userinfo or port before the new setter block. A remote host erased by the parser therefore cannot be revived as a local pipe-drive URL. The setter is not asked to establish authority.
- **The replacement has safe byte boundaries.** The predicate proves that all four relevant bytes are ASCII. `replace_range(2..3, ":")` therefore addresses one complete character. The owned pathname copy ends the relevant borrow before moving the `Url`; each branch yields one owned `Url`. I found no Windows-only borrow/type problem in the shadowing, and the integration targets use Rust edition 2024.
- **Escape validation remains after the rewrite.** Lines 142–178 still reject malformed percent escapes, encoded slash/backslash, invalid UTF-8 and NUL. The published setter does not double-encode existing percent escapes (`lib.rs:1751–1761`), uses path parsing, and restores query/fragment positions (`lib.rs:1767–1789`, 1804–1816). The later decoder still performs a single strict decode.
- **Drive admission remains strict.** Encoded pipe is not the literal byte pattern and still fails the decoded colon requirement. A bare pipe drive and a drive-relative pipe segment fail the required rooted prefix. The decoded Windows precondition at lines 179–193 precedes `to_file_path`, preventing the dependency's absolute-path assertion from seeing a bare drive. `path.is_absolute()` and `admit_local_media_path` remain mandatory after native conversion.
- **Aliases are still denied.** A colon after the drive position, including decoded ADS syntax, fails the path guard. Network and device namespace spellings retain the existing Windows guard. Interior pipe characters are not rewritten; the change does not claim to validate every possible Windows filename spelling.
- **The web restriction remains independent.** `web_media.rs:265–292` still inspects the original separator shape. It refuses repeated leading separators, including localhost and mixed/ignored-tab/CR/LF forms, while preserving the existing drive-token interior-separator exception. The guard runs only after the shared owner has classified a file source, and before the reader. It is byte-identical to the exact-base guard.
- **Unix is unchanged by this repair.** The setter block is absent under Unix compilation. The new Unix assertion explicitly preserves `/C|/safe/file|name.png`, and the previous colon and double-slash path-guard controls remain.
- **Admission remains pure.** The shared conversion/path guards contain no canonicalization, metadata, open, read, Box call, network operation or durable-state mutation. The rewrite adds only an in-memory URL mutation on the narrow Windows case.

### Five repair-path dispositions

| Path | Current repair and disposition |
| --- | --- |
| `projects/openclaw-rust/docs/specs/local-media-file-url-admission.md:95–142` | LFA-03a specifies the narrow platform rule, original-authority ordering, prohibited fallback, retained negative classes, unchanged test selection and required exact-H Windows execution. The implementation and test delta match that contract. |
| `source/host/src/extensions/attachments/media_reference.rs:86–204` | Approved: authority refusal, literal rooted-drive delimiter repair, strict decoding, rooted-drive precondition, native conversion and final path guard remain one ordered shared boundary. |
| `source/host/src/extensions/attachments/web_media.rs:572–627` | R4 changes test-table entries only. The added encoded/bare/relative pipe, ADS, separator, UTF-8, NUL and malformed-escape cases remain inside the existing zero-reader-entry test and all four roots/sandbox combinations. |
| `source/host/tests/local_media_file_url_contract.rs:31–77,147–188` | Approved: four Windows pipe spellings feed exact native path equality; remote-host pipe controls reject; Unix pipe preservation is explicit; Windows refusal cases extend the existing selected test. Iterator ownership is consistent on both platform branches. |
| `source/host/tests/openclaw_web_media_contract.rs:269–331` | All four original rooted-drive positives remain with exact `C:\` equality and source-labelled diagnostics. Four pipe variants extend the existing real loader/service byte, filename, cap-refusal, valid-retry and source-preservation test. Query/fragment variants still exercise the same paths. |

## Production callers, state and oracle assessment

The App resolver at `main.rs:639–705` now returns a Result. Its shared admission occurs before `fs::read`, `ForeverBoxService::get_status`, download and materialization. `Ok(None)` preserves ordinary non-file passthrough; `Err` cannot become a successful unchanged file URL. Both direct production callers propagate the Result (`main.rs:899–905` and 1180–1186). The separate pre-existing persistence/read-outcome fallback remains outside this admission claim and is explicitly left open in the specification.

SendMessage keeps its existing Session admission before resolution (`main.rs:885–898`). A later image error propagates through the real `build_sand_send_message` collection, and `SendMessageToolBridge::call_tool` invokes the existing abort hook (`send_message_tool.rs:260–276,405–410`). The production abort removes only that call's pending custody, rolls back its owned files and releases the recorded Session media admission (`main.rs:969–996`). The new App rollback oracle first observes a real materialized/admitted first image, then proves the later refused image causes that owned file and durable admission to be released. It is not a vacuous zero-file test.

AgentManagement has a distinct existing loop without SendMessage rollback. Its `parse_images` now admits every file URL while assembling the value vector (`sand_agent_management_tools.rs:93–105`), before `call_tool` begins any `resolve_image_source` iteration (`:61–65`). The native test observes zero resolver entries and zero sends for a valid first image followed by an invalid URL. The actual App test separately uses the real production sink, Session/Transcript/ForeverBox owners and messaging owner, checks unchanged media/transcripts/wakes across repeated refusals and owner reopen, then exercises successful native image materialization and a real outgoing transcript/wake projection. It does not pretend to execute a recipient agent or a live channel.

All six App test bodies and their fixture/observation port were read:

| App test | Causal oracle retained |
| --- | --- |
| `local_media_file_url_app_rejects_invalid_before_materialization` | Invalid native-owned URL variants fail through the real sink; no new media, admission, transcript publication, successful resolution or outward Host request. |
| `local_media_file_url_app_rolls_back_earlier_image_on_later_refusal` | First image actually acquires owned bytes/custody; later refusal triggers actual bridge abort, owned-file deletion and Session fence release, preserving the original source. |
| `local_media_file_url_app_commits_native_and_unicode_media_across_reopen` | Actual native/Unicode materialization, filename/bytes/dimensions, transcript identity and persisted media survive fresh owners. |
| `local_media_file_url_app_repeated_refusal_then_repair_survives_reopen` | Repeated denial preserves prior state; two fresh valid sends around a real reopen acquire distinct owned files and normal messages. |
| `local_media_file_url_app_agent_management_refuses_image_before_outward_send` | Direct sink refusal and whole-batch preflight preserve both Sessions and media; native success afterwards reaches real messaging and the observed wake port. |
| `local_media_file_url_app_preserves_https_without_local_custody` | HTTPS stays passthrough with normal persisted message behavior and no local media ownership. |

Channel admission continues through `media_file_context::resolve_local_media_path`, returning `None` on a shared refusal instead of manually reconstructing a file URL. The raw/home-expanded path guard remains before returning a local path. `resolve_channel_attachment` checks the final reanchored path at line 62 before metadata at line 63 and bounded reading at line 71. Exact-base reads confirm the real Attachments service delegates to this reader and the production cross-user attachment callback consumes the upload/None outcome. No new permission or delivery owner is introduced.

The web adapter maps shared failures to `invalid-file-url`, retains raw/resolved network error distinctions, and checks the final path before the private local-reader callback (`web_media.rs:294–385`). The callback's actual filesystem/HTML/image body is unchanged. Its direct service arm still delegates to `load_web_media` and retains the checked byte-cap conversion; no public Gateway registration is added.

## Complete 21-path current-source disposition

Paths are relative to the frozen snapshot. “Reviewed” means all changed hunks against exact 3438 plus the affected functions, fixtures or consumers needed for this assessment; it does not claim every unrelated line of the large App, ledger or workflow was reread.

| # | Path | Review disposition |
| --- | --- | --- |
| 1 | `.github/workflows/rust-desktop-runtime.yml` | Reviewed all changed selections/input bindings and actual native, portable-script and Linux App execution wiring. LFA names are added to existing gates; existing exact-subject and failure handling remain. |
| 2 | `projects/openclaw-rust/docs/specs/local-media-file-url-admission.md` | Read fully. Current LFA-01..06, AgentManagement batch preflight, custody/reopen requirements, integration constraints and LFA-03a align with the current source. |
| 3 | `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | Read fully. Shared-owner integration preserves FUA error classes, raw-prefix evidence, pure reader boundary and declared platform/read-authority limits. |
| 4 | `projects/openclaw-rust/parity-ledger.json` | Reviewed every changed hunk and mapped owner/consumer/test claims to source. The edits add narrow admission responsibilities, risks and references; they do not change status/global completion fields. Excluded historical JSON contents are not endorsed by those references. |
| 5 | `projects/openclaw-rust/requirements-traceability-matrix.json` | Reviewed every changed hunk. Additive spec/test/evidence references preserve existing requirement and acceptance status; no independent machine-schema result is claimed. |
| 6 | `projects/openclaw-rust/scripts/capture_media_cleanup.py` | Read current source fully. One shared channel target, all original BR/HHP/IPB/FUA selections, 16 groups, required-name uniqueness, exact summaries, command/capture exits, input identity and platform checks remain. |
| 7 | `projects/openclaw-rust/scripts/capture_portable_contracts.py` | Reviewed both changed hunks and actual selection/result consumers. Two LFA facade names accompany the change from 15 to 17 SendMessage positives; the other summaries remain 13 and 14, with no ignored/failure relaxation. Existing Device/Memory selections and WFX extraction are retained. |
| 8 | `source/host/app/src/main.rs` | Reviewed Result boundary, both actual sinks, Session/custody/abort adjacency and test-module registration. Invalid file URLs cannot reach local read or Box fallback. |
| 9 | `source/host/app/src/tests/local_media_file_url_tests.rs` | Read all 769 lines, including fixture lifecycle and delegation. All six actual App oracles above remain substantive and distinguish controlled observation ports from live service acceptance. |
| 10 | `source/host/src/connectors/channel_attachment.rs` | Read full file. Final reanchored admission precedes metadata/bounded read; remote HTTP forwarding and upload behavior remain. |
| 11 | `source/host/src/extensions/attachments/media_file_context.rs` | Reviewed all changed code and complete local resolver. Shared Result refusal replaces the separate reconstruction path; native and home-expanded local paths retain platform guards. |
| 12 | `source/host/src/extensions/attachments/media_reference.rs` | Deep review of new shared parser/guard and adjacent consumers/reader. The R4 repair is narrow; the existing BR reader implementation is byte-identical to exact base. |
| 13 | `source/host/src/extensions/attachments/web_media.rs` | Deep review of admission, local-reader transition and all changed table oracles. Raw prefix guard and actual bounded/HTML/image read body are byte-identical to exact base. |
| 14 | `source/host/src/runner/tools/sand_agent_management_tools.rs` | Read full file. Whole-value preflight finishes before all per-image resolution and send side effects; direct App sink remains independently guarded. |
| 15 | `source/host/src/runner/tools/send_message_tool.rs` | Reviewed changed facade plus real build/abort/delegation paths. Metadata Option projection delegates to shared admission; the production App uses Result directly. |
| 16 | `source/host/tests/channel_attachment_contract.rs` | Read full file. Added native byte/metadata, malformed encoding/retry and Windows pure refusal controls preserve original MIME/cap assertions. |
| 17 | `source/host/tests/local_media_file_url_contract.rs` | Read full file. Cross-platform owner/error distinctions and R4 pipe boundaries are covered without filesystem I/O. |
| 18 | `source/host/tests/openclaw_media_file_context_contract.rs` | Reviewed all changed functions and surrounding fixture/service assertions. Native exact-path positives replace the invalid Windows POSIX assumption while retaining Unix POSIX controls and adding Windows refusals. |
| 19 | `source/host/tests/openclaw_web_media_contract.rs` | Reviewed every changed hunk, complete affected tests, native fixture and real service helper. Four original root positives remain; exact bytes, cap, retry, trust absence and source preservation are asserted. |
| 20 | `source/host/tests/sand_agent_management_tools_contract.rs` | Read full file. Boundary-observation sink proves zero resolution before whole-batch admission and native/HTTPS positive recovery. |
| 21 | `source/host/tests/send_message_tool_contract.rs` | Reviewed added facade tests with facade/bridge integration and capture selection. Admitted metadata remains exact; unsafe file URLs lose local metadata without this Option helper being mistaken for the production Result boundary. |

### Capture and preservation conclusion

The native capture source selects **163 Linux/macOS** and **158 Windows** required positives across **16 groups**, bound to **46 input paths**. These are source expectations, not observed R4 results. The R4 additions change table entries, not function selections or expected test counts. The current extracted portable script requires summaries `(13,0,0)`, `(14,0,0)`, `(17,0,0)` and unique required results. The actual workflow invokes that script at line 3604; native media capture is wired on macOS, Linux and Windows at lines 157, 3329 and 4417.

The six App cases are included in the existing Linux App required list at workflow lines 4160–4169. That lane compiles the actual shipping binary, identifies one linked App test executable, requires exact test-list identity and executes each selected name with a one-passed/zero-failed/zero-ignored completion. The execution remains pending. Workflow bytes are 431,076; no YAML parser or Actions validator was run here.

Independent source-byte comparisons confirm that the BR reader slice from `MEDIA_READ_CHUNK_BYTES` through the implementation before its test module, `load_local_web_media`, and the original raw-prefix guard are byte-identical to the exact 3438 base. The service cap conversion remains `usize::try_from`; R4 changes no reader, service, state, inventory or workflow path beyond its five declared files relative to R3.

## Independent identities and original execution evidence

I independently recomputed SHA-256, Git blob SHA-1 and lengths for all 23 candidate paths, compared all 23 prior-snapshot blobs, compared the 18 existing base blobs, and confirmed the five newly added paths are absent from the base snapshot. The snapshot's complete file set exactly matches the 23 manifest entries. There were no mismatches. Exactly the five declared repair paths differ from R3; the other 18 are byte-identical. A final source hash read before report authoring still matched the manifest.

The base tree was independently resolved through Git to `b544c1d377ca82618cdadcf5106cb3c10d82411b`. The candidate's full 10,502-entry remote tree/readback identity is supplied by the frozen manifest/controller; I did not repeat that complete remote API retrieval. My independent check covers every scoped blob against that manifest.

| Artifact | Independently observed SHA-256 |
| --- | --- |
| R4 complete patch, 191,292 bytes | `900b7099d01a9b9bd61ee4d25a6f4522fe489fc4e8fb06f66028c11719b4ae28` |
| R4 fix patch, 12,234 bytes | `14c6a388c2d0140b9c02d87368444a9a7efc64250de586b7ba78824a579e5825` |
| `url-2.5.8.crate`, 86,512 bytes | `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed` |
| Archive `url-2.5.8/src/lib.rs` | `51f47a93654528f3b8cb92c94fc26ad25775860d84197fff0c3124a687f8c5e3` |
| Archive `url-2.5.8/src/parser.rs` | `2dd4210e1773fc5d3fdc15ce506ecff30dbf8eb358a79bbbc2835cc30d513c3a` |
| `fs-safe-0.27.0.tgz`, 1,101,331 bytes | `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c` |
| Archive `package/dist/local-file-access.js` | `7530a957ca9d56f56deba8da1dd06be03c894c6f6472bc8571080937b57bfda8` |

The extracted URL parser/lib files used for line reads exactly match their archive members. Immutable host and App lock reads both bind `url 2.5.8` to the archive checksum above. The fs-safe local admission, Windows syntax and alias modules were read directly from the archive. Fixed-U `web-media.ts` and its Windows file-URL test independently hash to Git blobs `f48bca504f6fb77c3d094864eb656cb2b1bf25e2` and `bdaedcde111ea2880b96d9afa547a9c1d2a36b42`; its actual web/local-path uses of `safeFileURLToPath` were inspected. This is dependency-contract/actual-use evidence, not full recursive dependency or Node/Rust parity closure.

I also read the original `explicit-html-consumers.log` member from each original Windows archive. Both show the unchanged rooted-drive assertion failing with **22 passed, 1 failed, 0 ignored**. Neither panic identifies the particular URL variant. The literal-pipe diagnosis above is consequently source-derived, not a fabricated variant label from those logs.

| Original archive/log | SHA-256 |
| --- | --- |
| `windows-3438.zip` | `8309821e43697663608552956c136e7335f5848f7fc056904a758e0a3cd68655` |
| Its `explicit-html-consumers.log` | `8613e71bae205445abdde7e1937723a0edf84aa098afb6ad009aec35445e91fe` |
| `windows-e19.zip` | `e6724c09ab10215588c4a9ccb9c632b477b5af66cdd4e94af8e70c58ef3c66e2` |
| Its `explicit-html-consumers.log` | `7bf08cb08fb238ba582f12b4c323f18aa2209444346b40a53fab3e83e9d089d0` |

## Remaining execution and acceptance limits

1. No R4 program or parser has run in this review. Exact published-H Actions must still establish compilation, actual Windows pipe-drive behavior, native loader/service bytes/caps/retry, retained negative reader-boundary controls, the complete selected native capture and the six actual Linux App cases. An earlier-H pass cannot substitute for those results.
2. Preserve both original failed Windows receipts. Their failure is not erased by this source approval or by the strengthened variant diagnostic. Do not rerun an unchanged failed subject as proof of this repair.
3. The root integrator must separately audit the two excluded historical evidence JSON files and preserve the newer disjoint evidence paths at publication. Any source change to a reviewed blob invalidates the corresponding current-source disposition.
4. This review does not close general root authority/permission, opened-handle identity/TOCTOU, full filesystem alias parity, unknown persistence outcomes, live provider/channel/device execution, public `loadWebMedia` registration, full codec/media equivalence, packaging, performance/soak or global G01–G12/AC01–AC24 acceptance. The current specs and captures accurately keep those claims separate.

Terminal verdict: **APPROVED**.
