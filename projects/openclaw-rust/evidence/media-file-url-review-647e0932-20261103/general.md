# FUA-v1 round 1 — independent general / correctness / security review

## Verdict

**No findings in the complete declared FUA-v1 change scope.** I found no concrete correctness, security, integration, capture-selection, or evidence-state defect that requires a source change before the next gate.

This is a source-review verdict only. It is not a compiler result, native runtime result, FUA acceptance, full media-equivalence verdict, or shipping acceptance. The candidate still requires its own exact-H Linux, macOS, and Windows Actions evidence. Earlier-H results do not validate this candidate.

Effort: **xhigh**, inherited as assigned, because the scope crosses URL normalization, native Windows path semantics, the production filesystem boundary, and acceptance evidence. The configured/inherited model was retained. No other review report or prior review-runner decision was read. No subagent was used.

## Frozen identity and integrity

- Repository: `bhrumom/fabushi-desktop`.
- Base and starting HEAD: `947e980bd3c9dcb06535127f5cedfd193b83f0fc`.
- Base tree: `686109128a616195e65d1c9018ccd5b0b9d2cc8a`; independently confirmed using the existing Git object database.
- Candidate tree: `27b6ac5b5e90023dff847e73e8c564743dd8e109`, as bound by the frozen manifest and the parent's GitHub tree readback.
- Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`.
- Manifest: `/workspace/scratch/7d27ea239237/file-url-r1-manifest.json`.
- Candidate snapshot: `/workspace/scratch/7d27ea239237/file-url-r1/`.
- Base snapshot: `/workspace/scratch/7d27ea239237/file-url-r1-base/`.
- Complete diff: `/workspace/scratch/7d27ea239237/file-url-r1-complete.patch`, 900 lines; independently measured SHA-256 `60f999d9097f70f0f0225e28e68be0b82a42eb6dd40f3587e750ff787b0d8411`, matching the manifest.

I independently hashed all seven candidate files with SHA-256 and Git blob hashing, and measured their byte lengths. Every value matched the manifest. I also hashed all five modified-file base snapshots; every base blob matched the manifest. The declared changes comprise two additions and five modifications, all read in the complete diff:

| Path | Candidate Git blob |
| --- | --- |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `8057987d1ec0ed53f87343f1ff65063808eca44a` |
| `projects/openclaw-rust/evidence/media-file-url-admission-647e0932-20261103.json` | `95b07bf1b78ef149b7dabb5ed2926693247219fd` |
| `projects/openclaw-rust/parity-ledger.json` | `df9cba558cccc210b35a811e9b5f9f3f8817cd79` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `1d74ab6a725b6bab93428ce571d16944a05afa92` |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | `7eed829018250ca297538df614f3929ea5a09058` |
| `source/host/src/extensions/attachments/web_media.rs` | `3e26c08770379911d6502bb778c0078deda822ed` |
| `source/host/tests/openclaw_web_media_contract.rs` | `752687912e16ceb804fa63b6a172251dc53d0943` |

The root `AGENTS.md` was read from the explicitly authorized foreign worktree path. Product dependencies were read from frozen snapshots or `git show` of the exact base; no changed product source in that mutable worktree was used.

## Source assessment

### Admission and filesystem ordering

`web_media.rs::file_url_to_path` (261–286) now uses the existing `Url` parser, requires the file scheme, applies an empty/localhost host allowlist, rejects encoded slash/backslash in the parsed path, and propagates native conversion failure as `LocalAccess` / `invalid-file-url`. The old one-host rejection and authority-stripping/lossy fallback are removed. The converted-path network guard remains before any local-reader entry. Constant error messages also avoid reflecting rejected URL credentials.

`load_web_media_with_local_reader` (299–344) performs reference cleanup, raw-path rejection, URL/workspace resolution, and the final resolved-path check before its sole callback invocation. Its operations are string/URL/path operations; no canonicalization, metadata, open, or read occurs in this admission function or its admission callees. On Windows, the component match rejects UNC, verbatim UNC, device, and other prefix namespaces while permitting only the explicitly scoped absolute `VerbatimDisk` workspace case. The initial raw-path contract remains in place.

Production `load_web_media` always supplies `load_local_web_media` (290–295). Both the seam and body are private, and no public option, service argument, or alternate public registration supplies a reader. `FnOnce` also prevents this function from retrying the reader. Existing root canonicalization starts at 353; the bounded read is at 380. The exact-base `media_reference::read_outbound_media_file_bounded` owns metadata, file open, and bounded reading, so those effects remain below the seam as well.

The extracted body retains the previous root, sandbox, byte-cap, MIME/HTML, filename, and image behavior. This preserves the previously documented limitations of default roots, the sandbox boolean, filesystem identity/races, and broader namespace/alias parity; the change does not claim those limitations are solved.

### Existing owner and real consumer

The change stays in the established Attachments web-media owner. The exact-base direct service arm at `attachments_service.rs::dispatch_gateway`, 2006–2022, still calls `load_web_media`, consumes the existing bounded options and exact HTML mode, and propagates its error/result. Source search of exact-base Host production paths found that service call as the existing production caller of this loader. The separate `media_file_context` parser keeps its existing optional-path contract and is not rerouted.

The public Gateway registry remains finite and unchanged. `gateway_protocol::is_grok_gateway_command` delegates to the unchanged registry/compatibility lists; `host_gateway_api` uses literal registrations with a `None` fallback and has no `loadWebMedia` entry. The new service test checks that existing admission surface without claiming HTTP, renderer, Runner, or channel execution.

### Oracles, platform fixtures, and recovery

The five common private-boundary tests cover unrelated hosts, localhost suffixes, IPv4/IPv6 authorities, mixed-case schemes, raw backslash file URLs, four-slash URLs, parser failures, encoded separators, raw and workspace-derived network paths, exact native path/options delivery, and original reader-error propagation without retry. The remote-host matrix exercises both root-presence states and both sandbox states. The Windows-only test pins the deliberately admitted verbatim-drive workspace, alongside retained Windows UNC/device negatives.

The integration changes replace the Unix-specific Windows positive with a URL derived from the actual native temporary path. The new real-file test checks ordinary, uppercase single-slash, mixed-case, and localhost forms; exact Unicode/space/percent filename and bytes; real loader/service results; cap refusal; and successful subsequent calls. The Unix alias fixture creates a file that the removed fallback could actually read and proves refusal with roots active, with both sandbox settings, and through the real service. Its valid local URL remains a positive control. The service parse-refusal test checks typed propagation, credential non-reflection, unchanged registry admission, and successful recovery.

These controls are causally useful for the stated regressions: unrelated-host negatives defeat restoration of the denylist; the Unix alias defeats the old fallback; native positives defeat reject-all behavior; and callback-entry counters detect admission moved after the local body. The counters measure that private boundary, not arbitrary OS syscalls. This review establishes the source side of that limitation; native execution remains outstanding.

### Capture and retained contracts

The complete capture script was read. It adds the owner group and three common integration names, plus the Unix alias and Windows verbatim-workspace selections. All prior cleanup, HHP-v2, host-format, and IPB-v1 command groups/names remain. The script retains exact HEAD/dispatch/platform binding, before/after input identities, raw-log hashes, command and capture exits, exact-once positive names, no unexpected positives, one exact summary, and its 13 synthetic oracle qualification controls.

By source selection, the planned totals are Linux 79, macOS 79, and Windows 76, matching the new evidence document. These are planned counts, not observed results. The unchanged exact-base workflow has capture entry points and artifact upload paths on `macos-15`, `ubuntu-24.04`, and `windows-2025`; no separate workflow or execution framework was introduced.

The retained explicit HTML body still validates captured bytes before returning original HTML bytes with absent generated trust. Existing HTML and image regression groups are preserved; no HHP/IPB acceptance status is upgraded by this change.

### Provenance and ledger truthfulness

The new evidence distinguishes scoped source reading, implementation, direct service reachability, pending exact-H execution, and broader acceptance. The ledger change is confined to the existing web-media responsibility's descriptive behavior/invariants, evidence references, risks, and test references. Its full-unit states remain `verification=not-run`, `wiring=not-wired`, and `acceptance=pending`; historical read facts are retained. RTM changes only append the new spec/evidence references to the existing project rows. The complete diff does not promote global completion flags or alter historical receipts.

I independently matched the fixed-U `web-media.ts`, Windows file-URL test, package manifest, and lock blobs to the identities in the evidence. The actual `fs-safe` archive SHA-256 and all three declared member hashes match; its `advanced` export leads to the inspected `local-file-access.js` implementation. I also matched the version-source Rust `url` blob and read its native conversion contracts; both exact-base Host/App locks pin `url` 2.5.8 with the recorded checksum. This is bounded API/security-contract review, not complete package/dependency/legal closure.

## Read bounds and remaining acceptance boundary

Fully read: all 900 lines of the complete seven-path diff; the complete FUA specification and new evidence JSON; all 607 lines of candidate `web_media.rs`; all 827 lines of candidate `openclaw_web_media_contract.rs`; all 336 lines of the capture script; the complete HHP-v2, IPB-v1, and media-store read-scope specifications; and the supplied root instructions.

Focused adjacent reads: the entire affected ledger responsibility and the changed RTM contexts; project source-of-truth authority/current-subject sections; migration specification revision 2 §17; the media owner-map row; exact-base service dispatch, bounded reader, alternate optional-path parser, Gateway registry/dispatch predicates, Host/App lock entries, and relevant three-platform workflow/capture sections. The large unchanged ledger/RTM, full workflow, full service, and all unrelated product modules were not reread wholesale.

Fixed-source reads: the matching `web-media.ts` URL-admission and local-read ranges 530–562 and 638–756; the complete 48-line native Windows test; package/lock binding ranges; the complete packaged `local-file-access.js` plus export/license identity references; and matching `url` source conversion ranges 2694–2757 and 3060–3191. The candidate already records that these are scoped reads and that full Node/Rust/filesystem-alias parity remains open.

No builds, tests, compilers, syntax checks, project parsers, validators, generators, installation, or product execution were run locally, on htch-runtime, or in Actions by this reviewer. Only text/source/evidence reading, hashing, archive-member reading, and read-only Git operations were used. The only written file is this review report.
