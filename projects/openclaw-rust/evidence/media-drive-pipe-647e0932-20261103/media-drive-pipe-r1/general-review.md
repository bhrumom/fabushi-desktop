# Independent GENERAL review — Windows rooted drive-pipe recovery

## Verdict

**APPROVE — source review only.** No falsifiable correctness, security, permission, compilation, or oracle defect was found in the frozen three-path change. This is not a build result, a passing Windows receipt, or runtime acceptance. Exact-H Actions recovery remains required.

## Identity, independence, and limits

- Reviewer: `/root/media_pipe_general_r1`, fresh independent GENERAL + correctness/security/permission/oracle review.
- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Frozen base: `3438eb5bac755c399c9ec686414bbb4c0816888d`.
- Starting HEAD recorded by the manifest: `0e845ce1679e0cdf95346adc54f5d58bf06464e8`.
- Reviewed tree: `7ae5865f15158a5eeec67ac1a34e2a49f1171dd6`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`.
- Launch record: inherited configured **GPT-6 Astra Pro**, with **xhigh** selected/inherited for the Windows URL/path, dependency-parser, consumer, permission, and oracle review. This child made no model/effort override and launched no further worker. The host exposes no separate runtime introspection for the inherited reasoning setting; this records the inherited launch setting, not a locally changed setting.
- Root `AGENTS.md` governs these paths; the tree has no applicable nested `AGENTS.md` for them. The complete frozen diff, complete FUA specification, affected owner, caller/reader, contract tests, and relevant consumer/dependency/capture source were inspected. Repository source was read with `git show`/`git diff` against the frozen objects, not from the mutable product worktree.
- No prior review report or runner decision was read. The existing dependency carrier index and archived dependency source were used as data only. No repository source was edited.
- No product code, URL parser, test, build, generator, linter, validator, or harness was executed locally or on htch-runtime. Git/source reads, archive-member reads, dependency byte hashes, and read-only GitHub source retrieval were the only inspection operations. This review did not run or independently audit a new Actions job.

The actual base-to-tree changed-path listing is exactly the following manifest scope; the unrelated plugin batch is excluded:

| Path | Reviewed blob |
| --- | --- |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `8b3bc15887b5dc6ebace7d7805ae63bc125d2a8e` |
| `source/host/src/extensions/attachments/web_media.rs` | `e3af38b17fd7b91627493b89d6174636f719e109` |
| `source/host/tests/openclaw_web_media_contract.rs` | `94d50b5e4a9c51f97da1e3f017052615edbc64ac` |

## Expectation and dependency semantics

The rooted literal-pipe expectation is supported by the actual dependency path, independently of whether the old test happened to pass. Fixed-U `src/media/web-media.ts`, blob `f48bca504f6fb77c3d094864eb656cb2b1bf25e2`, imports `safeFileURLToPath` from `@openclaw/fs-safe/advanced` and invokes it before local media loading, wrapping conversion refusal as `invalid-file-url`. Fixed-U `package.json`, blob `6989c0618742d3c2b93cc336cbe1d9c3c4f0a657`, requires `@openclaw/fs-safe` 0.27.0 and permits Node `>=24.16.0 <25 || >=26.1.0`.

The already acquired fs-safe package archive was read directly as data. Its SHA-256 is `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`; `package/dist/local-file-access.js` has SHA-256 `7530a957ca9d56f56deba8da1dd06be03c894c6f6472bc8571080937b57bfda8`. Its `safeFileURLToPath` uses Node `URL`, rejects nonlocal hosts and encoded path separators, then calls `fileURLToPath` before filesystem-alias/network checks. The exact source at `openclaw/fs-safe@67dee5ef4c03655a2aa98893539475ad041cb61b`, `src/local-file-access.ts` blob `58acd2b6f17aa4bbae3eb59a54190a2fc18c9791`, agrees. The adjacent `hasWindowsPathAlias` source does not reject an ordinary rooted `C:\` path.

Node v24.16.0 source was retrieved and its file/blob correspondence checked through GitHub:

- `src/node_url.cc`, blob `5f5241f6c6741218024452202c4a17c7d0239e6b`, binds URL parsing to `ada::parse<ada::url_aggregator>`.
- `deps/ada/ada.cpp`, blob `80bec6c67f313a78847dc3d88b59a5bf3a1b9f4d`, sends an empty file host to `PATH_START` (around lines 13984–14025). Path-start consumes the leading separator; the file-path consumer normalizes the first drive token to a colon (around lines 15783–15793). Localhost is emptied at the file-host step as well.
- `lib/internal/url.js`, blob `dfcd071073a8d31357b5885c03a6175acb4037c3`, `getPathFromURLWin32` converts separators and requires the resulting ASCII drive/colon prefix (lines 1472–1507). Thus the literal rooted-pipe form reaches an ordinary rooted drive path.

The exact locked Rust crate was also read directly. `/tmp/fabushi-647e-pairing/local-file-url-dependency/url-2.5.8.crate` has SHA-256 `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`, matching both frozen Host and App lockfiles. In `parser.rs::parse_file_host` an empty/localhost host returns the false flag; `parse_file` then inserts a slash while retaining the input's leading slash. Consequently the later `C|` segment can miss the `segment_start == path_start + 1` normalization in `parse_path`; removal of leading empty segments leaves `/C|/`. The Windows converter in `lib.rs::file_url_segments_to_pathbuf_windows` accepts a two-byte ASCII-letter/colon token or an ASCII-letter/encoded-colon token, but not a pipe. This independently supports the spec's identification of the third positive as the source-traced trigger; it is not evidence that the old failure log printed that reference.

## Production correctness and permissions

`web_media.rs::file_url_to_path` lines 333–347 implement the declared bounded repair. The `#[cfg(windows)]` shadow binding changes only a parsed path beginning with the literal four-byte shape `/[A-Za-z]|/`. The match guarantees a single-byte ASCII pipe at byte index 2, so `replace_range(2..3, ":")` is on valid UTF-8 boundaries. The temporary path is owned before the mutable setter call; no outstanding path borrow crosses mutation. The slice pattern, `u8` ASCII predicate, ownership/shadowing, and standard `Url` API use present no source-visible Rust compilation hazard under the existing edition-2024 crate. Actual compilation still requires Actions.

The repair delegates the final conversion to `Url::to_file_path`; it neither manually assembles a native path nor reintroduces fallback conversion. `Url::set_path` preserves existing percent escapes and restores the old query/fragment suffix (`lib.rs` lines 1757–1789 and 1804–1816). Therefore replacing the drive delimiter does not double-encode the fixture's Unicode, spaces, or literal percent sign, and query/fragment separator escapes do not enter the pathname scan. Non-Windows compilation omits the whole new binding, retaining the previous conversion path.

Every pre-existing admission condition still runs before normalization: scheme check; original-host violation callback and parsed-host policy; original leading-path-separator guard; case-insensitive encoded slash/backslash scan; and the Windows second-segment/root guard. The final converted-network-path check also remains. In particular, remote host/drive combinations cannot be rescued by this normalization, encoded `C%7C`/`C%7c` do not match it, nonletter/long drive tokens do not match it, and bare `C|` still stops before native conversion. Interior literal pipes are not changed.

The production path remains `load_web_media` → private `load_web_media_with_local_reader` → `load_local_web_media`. Admission and the resolved Windows namespace/network guard precede the one private `FnOnce` reader transition. Root canonicalization, metadata/open/read, format handling, and the byte cap remain inside the unchanged local body and its bounded-reader callee. The new path normalizer receives no permission option and cannot bypass roots or admission through `sandbox_validated`.

The actual `AttachmentsService::dispatch_gateway("loadWebMedia", ...)` arm at `attachments_service.rs` lines 2006–2025 still parses its established cap/image/HTML options, calls the same public loader, and serializes that result or forwards its error. It has no added read override or trust field. `gateway_protocol::is_grok_gateway_command` and the finite registry in `host_gateway_api.rs` are unchanged and do not register `loadWebMedia`.

This is a local URL syntax repair, not proof of a complete authorization system. The existing default-root/sandbox-capability, pathname identity/race, namespace/device, and wider parity limitations explicitly retained by the FUA spec remain outside this approval; none was silently closed by the patch.

## Oracle and consumer assessment

The four rooted-drive positives remain exactly `file:///C:/`, `file://C:/`, `file:///C|/`, and `file:///C%3A/` in `test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references` (lines 258–263). They are not weakened, and their failure messages now identify the reference. The new negative cases retain the exact `invalid-file-url` classification and add literal-pipe forms with prohibited leading separators and encoded path separators, plus Windows encoded-pipe/nonletter/long-drive refusals.

The private no-I/O tests run these additions under both values of `sandbox_validated` and both absent/present local-root options. Their reader counter must remain zero; if admission is moved after the reader or the normalizer admits a bad token, the sentinel/counter makes that observable without opening a share. Existing remote-host, mixed-separator, bare-root, raw/workspace UNC/device, one-entry positive, original-options, and error-without-retry controls remain in place.

`test_ocrm_web_media_file_url_native_bytes_service_and_cap_recovery` (lines 267–321) now builds both empty-host and localhost pipe URLs from the current platform's actual native temp-file URL. It does not hard-code a Unix path as a Windows positive. Each form is also tested with query/fragment escapes. The loop checks exact native `PathBuf`, exact file bytes, Unicode/space/percent file name, and `text/html`, then exercises the actual Attachments dispatcher through `assert_explicit_html_loads`.

That helper (lines 513–548) requires exact serialized object equality for both direct loader and real service, including `kind: document` and absence of generated-HTML trust. The same pipe reference is refused at `bytes.len() - 1` by both loader and service, then succeeds again, and the original file remains unchanged. This is an actual file/service consumer control, not only a conversion assertion. The loop's post-cap retry uses the same file bytes; independent retained tests at lines 872–891 and 973–1002 explicitly replace file contents to test fresh reads. Source tracing confirms every admitted call invokes the unchanged reader afresh; the new normalizer adds no cache, persisted state, retry, or alternate reader.

The existing capture script still selects the changed owner module through `FILE_URL_OWNER_PREFIX` and the complete `openclaw_web_media_contract` through `WEB_MEDIA_NAMES`; no renamed or newly added test name escapes capture. On Windows this is six owner tests and 23 web-media tests. The script's `assess` requires unique exact positive names, one matching zero-failure/zero-ignored summary, and both cargo and capture exit zero; its input capture binds committed blobs before/after. The three-platform workflow still calls this script. These are source observations about the existing evidence mechanism, not claims that it has executed against this tree.

## Spec compliance and remaining gate

| Requirement | Source disposition |
| --- | --- |
| FUA-PIPE-01 | Satisfied by the Windows-only literal first-drive normalization after the prior guards, followed by native conversion and final network refusal. |
| FUA-PIPE-02 | All four rooted positives and existing negatives retained; new literal/encoded-pipe and invalid-drive negatives cover the private boundary and pure converter; failure diagnostics identify the reference. |
| FUA-PIPE-03 | Native temp-file/actual-service fixture extended for literal-pipe, localhost, and query/fragment variants with path, byte, name, HTML, cap-refusal, and subsequent-load assertions. |
| Historical evidence and acceptance separation | The amendment preserves the failed run/job/artifact identity and labels the trigger as source-traced. It requires new exact-H recovery and does not promote whole-media or global acceptance. Historical outcome counts are preserved from the contract; this reviewer did not independently re-audit that old receipt. |

**Findings: none.** Proceed to the declared focused Actions recovery for the changed URL owner controls and full web-media contract on the newly frozen H, preserving unrelated valid evidence at its original subject. Do not report CI success or bounded runtime acceptance from this source-only approval.
