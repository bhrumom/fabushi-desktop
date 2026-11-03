# Independent reuse, structure and performance review — media drive-pipe R1

Verdict: **APPROVE — source-only.** No actionable reuse, structure or performance defect was found in the complete frozen scope. This is a source review verdict, not a passing execution, CI, native-platform or release-acceptance claim.

## Identity, method and limits

- Reviewer: `/root/media_pipe_structure_r1`, fresh independent blind review. Other reviewer reports and arbitration were not read.
- Applied launch setting: `gpt-6.1-sol`, reasoning effort `xhigh`. The parent confirms the accepted exposed spawn call used `fork_turns="none"`, `model="gpt-6.1-sol"`, `reasoning_effort="xhigh"`; no fallback was reported. This records the launch setting, not independent runtime model introspection. The complete assignment includes Windows URL normalization, guarded filesystem admission, native consumer fixtures and capture integration, which warrants `xhigh`.
- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Base: `3438eb5bac755c399c9ec686414bbb4c0816888d`.
- Starting HEAD recorded by the frozen manifest: `0e845ce1679e0cdf95346adc54f5d58bf06464e8`.
- Reviewed tree: `7ae5865f15158a5eeec67ac1a34e2a49f1171dd6`.
- Scope: the following complete three-path diff against the base, with the adjacent frozen source and primary dependency/upstream sources described below. Product source was read with `git show TREE:path`; the mutable product worktree was not the review subject.

| Changed path | Frozen blob |
| --- | --- |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `8b3bc15887b5dc6ebace7d7805ae63bc125d2a8e` |
| `source/host/src/extensions/attachments/web_media.rs` | `e3af38b17fd7b91627493b89d6174636f719e109` |
| `source/host/tests/openclaw_web_media_contract.rs` | `94d50b5e4a9c51f97da1e3f017052615edbc64ac` |

The complete delta is 108 added and 14 removed lines: 55 specification additions, 23 owner-file additions including private tests, and 30 additions/14 removals in the existing integration target. No manifest, dependency, lock, build target, registry, module owner or public interface is changed.

Read-only source/Git/API operations were used. The embedded crate was read by streaming the 26 frozen carrier parts through data-only gzip/tar member reads. No collected file was executed or installed. No URL parser, product code, test, build, lint, validator, generator or benchmark was run locally or on `htch-runtime`. No repository path was edited. Only this requested review report was written outside the repository.

## Findings

None. There is no blocking or non-blocking preference-only finding to send into a fix round.

## Reuse and ownership assessment

The correction is placed in the existing `web_media::file_url_to_path` owner (`web_media.rs:293–355`). The inserted Windows-only block (`333–347`) changes the first literal parsed `/[A-Za-z]|/` component and then returns to the same `Url::to_file_path` and final network-path refusal. It neither constructs a native path nor decodes percent escapes itself. The previously introduced leading-separator helper and original-host callback remain the admission guards; this delta does not introduce another parser or second URL/business owner.

The ordinary loader still calls this function from its pure admission phase (`load_web_media_with_local_reader`, around `377`), then passes the result to the same private local body. The body retains root canonicalization, the shared `read_outbound_media_file_bounded` call, filename extraction, explicit HTML policy and image behavior. The delta cannot route around those consumers.

Actual adjacent production wiring was inspected: `attachments_service.rs:2006–2025` builds the existing options and delegates `loadWebMedia` to this loader; `attachments/extension.rs` forwards service dispatch; `host_production_extensions.rs` retains the existing Attachments extension composition. `attachments/mod.rs`, `extensions/mod.rs`, `lib.rs`, `source/host/Cargo.toml` and `source/host/app/Cargo.toml` establish the unchanged module/build path. The app depends on this existing host crate. No second endpoint or public registry entry is added. The service's default options are still distinct from an authorized local-root capability; the tests and this review do not claim otherwise.

Existing URL-related implementations were compared rather than assumed interchangeable. `media_file_context::resolve_local_media_path` and the channel attachment resolver have `Option`-based path resolution contracts, while `media_audio_transcode` has a normalization/fallback responsibility. Reusing those implementations here would not preserve this owner's typed admission errors, original-host violation handling and private no-I/O seam. Keeping this narrow fix at the actual failing owner is appropriate. No reusable shared implementation was duplicated by the new four-byte prefix recognition and one-character substitution.

## Windows behavior and dependency assessment

The source order is load-bearing and remains correct for the declared fix: parse failure and file-scheme refusal precede original-host/remote-host refusal; the existing raw leading-separator guard and encoded path-separator guard run next; the Windows missing-second-segment/bare-root guard follows; only then can the new literal-drive-pipe rewrite run. Native conversion and resolved network-path refusal remain last. Remote or malformed URLs do not gain a recovery route.

The new match requires a slash, exactly one ASCII alphabetic drive byte, a literal pipe byte and the root slash. Replacing byte range `2..3` is safe for that recognized ASCII prefix, including when a later filename contains Unicode. Encoded pipes and non-letter/long drive tokens cannot match. A bare `/C|` cannot reach the new block because the earlier Windows root-segment guard refuses it. Unix compiles without the inserted block.

Primary locked dependency source was inspected directly, not inferred from current library documentation. Both frozen host locks bind `url` 2.5.8 with checksum `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`. The actual `local-file-url-dependency/url-2.5.8.crate` bytes were read from the frozen `media-947-dependency-carrier-20261103` parts. Relevant reads covered `parser.rs` file-host/file/path-start/path handling and drive-token predicates, plus `lib.rs::set_path`, restoration of the query/fragment and the native Windows converter.

Those sources support the stated repair mechanism. Empty/localhost host handling can leave an extra initial empty segment before file path cleanup, causing the first-drive normalization opportunity to be missed. The Windows converter accepts the colon drive token and the already-supported encoded colon, but rejects the literal pipe token. `Url::set_path` explicitly preserves already-percent-encoded path strings, uses the existing path setter and restores the query/fragment. Therefore this correction retains escaped Unicode, spaces and literal percent filenames instead of double-encoding them. Conversion still belongs to the existing native converter. This is static tracing; no parser execution was performed, and archive checksum validation was not independently executed in this review.

The source compatibility requirement was also checked against primary upstream sources fetched independently:

- Fixed U `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`, `src/media/web-media.ts`, blob `f48bca504f6fb77c3d094864eb656cb2b1bf25e2`: the actual file-URL branch calls `safeFileURLToPath` and maps failure to `invalid-file-url` (`555–559`).
- Fixed-U `package.json`, blob `6989c0618742d3c2b93cc336cbe1d9c3c4f0a657`: `@openclaw/fs-safe` is `0.27.0`; Node `24.16.0` is within the declared engine range.
- `openclaw/fs-safe@67dee5ef4c03655a2aa98893539475ad041cb61b`, `src/local-file-access.ts`, blob `58acd2b6f17aa4bbae3eb59a54190a2fc18c9791`: `safeFileURLToPath` uses Node structural URL/native conversion after scheme, host and encoded-separator admission, then checks prohibited native forms.
- `nodejs/node@v24.16.0`, `deps/ada/ada.cpp`, blob `80bec6c67f313a78847dc3d88b59a5bf3a1b9f4d`: independently read `13857–13904`, `13984–14025` and `15770–15805`; empty file-host handling goes through path-start, and the first literal drive token is normalized to a colon.
- That Node version's `lib/internal/url.js`, blob `dfcd071073a8d31357b5885c03a6175acb4037c3`, `getPathFromURLWin32`: the local native result requires an alphabetic drive plus colon after separator refusal and decoding.
- Fixed-U Windows web-media tests, blob `bdaedcde111ea2880b96d9afa547a9c1d2a36b42`, were read as the existing uppercase/single-slash and unsafe-file-URL precedent. They do not themselves provide a dedicated pipe runtime receipt; the pipe requirement is source-supported by the dependency/Node chain above.

## Test and capture integration assessment

The complete changed integration file and private owner-test module were read. The four original Windows rooted-drive positives are retained verbatim: `file:///C:/`, `file://C:/`, `file:///C|/` and `file:///C%3A/`. Diagnostics now identify the failing reference. Every existing negative remains; added cases cover the prohibited leading separators, encoded separators in pipe URLs, upper/lower encoded pipes, numeric drive tokens and long drive tokens.

The additions to the private owner tests reuse `assert_reader_not_entered` under the pre-existing roots/default and sandbox combinations. Those checks are causally stronger than letting a network filesystem access fail: the callback entry count must remain zero, and the exact admission error category is asserted. Their claim remains reader-boundary entry, not arbitrary OS syscall measurement.

`test_ocrm_web_media_file_url_native_bytes_service_and_cap_recovery` adds two Windows pipe spellings to its existing native temp-file reference iterator: empty host and localhost. The existing iterator then creates both plain and query/fragment forms for each. Each new case therefore exercises the same exact native `PathBuf`, real Unicode/space/percent filename, real bytes, root-constrained loader, explicit HTML MIME/name behavior, actual `AttachmentsService::dispatch_gateway`, cap-minus-one refusals and successful subsequent service read. The existing equality helper still requires the exact object and absence of generated HTML trust. No parallel fake service or synthetic-only pipe test is substituted.

The unchanged capture source already runs the owner module and full integration target. `capture_media_cleanup.py` declares the owner prefix at `57`, records both changed source files, the spec and host locks in `INPUTS`, and includes `file-url-admission` and `explicit-html-consumers` in the command cases around `368–369`. Existing required positive names still match because this delta adds cases inside existing tests. Its assessment checks unique positive names, exact passed/failed/ignored summary counts and cargo/capture exits. The frozen Actions workflow invokes that capture in its existing Linux/macOS/Windows jobs and uploads exact-subject artifacts. These source declarations establish integration reach; they do not establish any new job's outcome.

## Performance assessment

Ordinary non-Windows URLs compile through the previous behavior. On Windows, an ordinary admitted URL adds a bounded prefix check and a move/shadow of the existing `Url`, with no added path clone or setter call. Only the compatibility form pays for a path-sized `String`, one ASCII replacement and the existing setter's path reparse. This remains linear in URL length and adds no I/O, filesystem probe, dependency, cache, retry, background work or process. Using the existing setter avoids maintaining an independent serialization/parser implementation. No concrete performance regression requiring a change or benchmark was identified from this scope.

## Recorded acceptance limits

The spec amendment accurately keeps the historical failed Windows receipt as failed and describes its missing-reference attribution as static source tracing. This review did not independently audit that historical Actions run and does not promote its other case results. Exact-H Actions evidence for the affected owner controls and full integration target remains required after publication.

Broader percent-decoding and encoded drive-letter behavior, dot-path normalization parity, namespace/device policy, native read-capability/default-root authority and file-identity/race closure remain deferred under the existing specification. Public HTTP/UI/Runner/channel behavior, native 32-bit, hostile Windows open deadlines, packaging, codec/full-media parity and global OCRM gates are not established by this source-only approval. No global or whole-media status is advanced.
