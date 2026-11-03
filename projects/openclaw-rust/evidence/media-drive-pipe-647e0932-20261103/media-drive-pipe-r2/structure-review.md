# R2 independent reuse, structure and performance review

Verdict: **APPROVE the declared source change for these review lenses. No actionable findings.** This is source-review approval only. It does not mark executable verification, native compatibility, the media responsibility, or shipping acceptance passed.

## Review identity and limits

- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Frozen base: `fee4f179b640489774208fa3ce653a3868b641bf`.
- Frozen R2 tree: `5a207a6a1914ba448f4511ed48e68787fb7f2d42`.
- Starting HEAD recorded by the manifest: `0e845ce1679e0cdf95346adc54f5d58bf06464e8`.
- Prior tree used only as the fix-delta identity: `7ae5865f15158a5eeec67ac1a34e2a49f1171dd6`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`, upstream tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Actual worker model/effort as supplied by the runner's launch configuration: **`gpt-6.1-sol`, `xhigh`**. This follows root `AGENTS.md`'s pinned reuse/structure/performance model. The complete assigned scope requires tracing Windows parser/native conversion, consumer fixtures and concurrent contract reconciliation; it is not a mechanical wording review.
- The manifest and Git tree independently agree that exactly the three paths below differ from the base. The current checkout contains unrelated concurrent changes; reviewed product sources and adjacent consumers were read using immutable Git object/tree references, not taken from that mutable checkout.
- Fresh independent review: no prior/current review reports or arbitration decisions were read. The incoming durable historical requirement was inspected as source, including its original `78aa967cc9400bb5c207441b2a78dbf7021d596c` contract text.
- Only Git/source/identity/archive data reads and this report write were performed. No product parser, test, build, lint, validator, generator, harness, benchmark, or other executable validation ran locally or on htch-runtime. No reviewed repository file was modified.

| Declared path | R2 blob |
| --- | --- |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `5733cb70842535cde24eb2bd0465a42c864e15ac` |
| `source/host/src/extensions/attachments/web_media.rs` | `e3af38b17fd7b91627493b89d6174636f719e109` |
| `source/host/tests/openclaw_web_media_contract.rs` | `1423e74eea806fe7656b29d14aa96dfd0c47174c` |

Both `complete.patch` and `fix-delta.patch` were read in full. Git reports 65 additions/0 removals in the contract, 23/0 in the owner and 28/15 in the integration contract. The focused delta reconciles the incoming historical contract and retains the incoming single-slash positive; it does not change the R1 production normalization implementation.

## Source requirement and parser/converter contract

The applicable authority is root `AGENTS.md`, the current project source-of-truth entry/constraints, OCRM's main migration specification §17, and the complete FUA contract. §17 preserves existing-owner-first implementation, separates source review from native execution/acceptance and requires exact-subject Actions evidence. The project module map's M23 placement agrees with the actual `extensions::attachments` owner; it is supporting navigation rather than independent ownership authority.

I independently checked the claimed compatibility requirement against primary source:

1. [Fixed-U `src/media/web-media.ts`](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/media/web-media.ts), blob `f48bca504f6fb77c3d094864eb656cb2b1bf25e2`, imports `safeFileURLToPath` from `@openclaw/fs-safe/advanced` and invokes it for a case-insensitive `file:` reference before subsequent local-path work (lines 554–560), wrapping failure as `invalid-file-url`.
2. [Fixed-U `package.json`](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/package.json), blob `6989c0618742d3c2b93cc336cbe1d9c3c4f0a657`, declares fs-safe `0.27.0` and permits Node `>=24.16.0 <25 || >=26.1.0`.
3. [fs-safe source `src/local-file-access.ts`](https://github.com/openclaw/fs-safe/blob/67dee5ef4c03655a2aa98893539475ad041cb61b/src/local-file-access.ts), blob `58acd2b6f17aa4bbae3eb59a54190a2fc18c9791`, structurally constructs a Node URL, checks the file scheme, empty/localhost host and encoded path separators, then calls Node `fileURLToPath` with the Windows option. The same source commit's `package.json`, blob `927f7cb02c483424dd253d2331c7806fd6bd43a3`, identifies version `0.27.0`. Its additional Windows alias/network checks remain broader obligations outside this narrow repair.
4. [Node v24.16.0 `deps/ada/ada.cpp`](https://github.com/nodejs/node/blob/v24.16.0/deps/ada/ada.cpp), blob `80bec6c67f313a78847dc3d88b59a5bf3a1b9f4d`, routes an empty file host through `PATH_START` (lines 13984–14025), consumes the leading separator there (13857–13903), and normalizes an initial Windows drive-letter pipe to a colon in file-path parsing (12167–12180 and the aggregator implementation at 15784–15801). [Its `ada.h`](https://github.com/nodejs/node/blob/v24.16.0/deps/ada/ada.h), blob `8f9089d2210a746a11141b1f6587ae4908ad24b3`, defines the drive check using an ASCII letter and `:`/`|` (1209–1214).
5. [Node v24.16.0 `lib/internal/url.js`](https://github.com/nodejs/node/blob/v24.16.0/lib/internal/url.js), blob `dfcd071073a8d31357b5885c03a6175acb4037c3`, `getPathFromURLWin32` rejects encoded separators, converts path separators, decodes percent escapes and requires the initial letter/colon native-drive form. The normalized rooted pipe input consequently has a source-supported route to the ordinary rooted Windows path.

The actual `/tmp/fabushi-647e-pairing/local-file-url-dependency/url-2.5.8.crate` archive was read without executing it. Its SHA-256 is `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`, matching both the frozen Host and App Cargo locks. `parser.rs::parse_file`/`parse_file_host` (512–560 and 1057–1112) use the empty-host result in a way that can put an extra leading slash through `parse_path`; the first-segment drive normalization at 1344–1363 is then missed before the leading-empty-segment removal at 1371–1378. `lib.rs::file_url_segments_to_pathbuf_windows` (3128–3202) accepts letter/colon or letter/encoded-colon first segments, not literal letter/pipe. This supports the narrow normalization requirement rather than requiring preservation of the old library limitation as permanent product behavior.

`Url::set_path` is also part of the inspected locked dependency contract (`lib.rs` 1745–1801): its documented and implemented behavior retains already percent-encoded path text, and it saves/restores the query/fragment suffix. Reusing it here does not double-encode the native fixture's Unicode/space/percent filename or turn query/fragment bytes into pathname admission input.

## Reuse and ownership assessment

The change extends the existing `web_media::file_url_to_path` owner in place. It continues to use the existing `Url` parser and `to_file_path` native converter; it introduces no dependency, public conversion API, persistence state, service arm, worker, cache or alternative path owner.

The ordering at `web_media.rs` lines 293–354 is appropriate: structural parsing and original-host violation capture, file-scheme/host admission, original leading-separator admission, encoded pathname separator refusal and the Windows bare-root guard all precede the new normalization. The byte-pattern match requires the already parsed leading `/`, exactly one ASCII drive letter, a literal pipe and a following `/`. The byte replacement is within known ASCII boundaries. It neither decodes an encoded drive pipe nor accepts longer/non-letter drive tokens. The converted result still reaches the existing final network-path refusal.

The existing channel attachment resolver is not a suitable reuse target for this repair. Its actual consumer path is `connectors::channel_attachment` → `media_file_context::resolve_local_media_path`, whose public result is `Option<PathBuf>` and whose single-slash/encoded-separator handling differs. Sharing or rerouting through it would broaden this repair and risk the FUA typed error contract. The current patch instead reuses the appropriate local owner and leaves that distinct consumer's behavior out of scope, as the contract requires. Targeted source search found no pre-existing equivalent drive-pipe normalization helper in the adjacent attachments/connectors owner that this patch unnecessarily duplicates.

The real consumer remains `load_web_media` → the private pure admission boundary → `load_local_web_media`, with filesystem operations after admission. The actual `AttachmentsService::dispatch_gateway("loadWebMedia", ...)` arm (2006–2025) constructs options and propagates this loader's result/error. No public Gateway registration changes appear in the declared diff. The source change does not move the filesystem boundary or introduce error retry/fallback behavior.

## R2 reconciliation and consumer coverage

The complete contract retains the historical FUA-WIN-ORACLE-01 text and explicitly identifies its original source commit. The immediately following amendment makes that text historical and supersedes the `file:///C|/` refusal only, preserving FUA-02 after normalization and every other valid control. This resolves the otherwise contradictory incoming requirement without erasing its provenance.

The rooted-drive integration loop at `openclaw_web_media_contract.rs` lines 256–263 contains five positives: `file:///C:/`, `file://C:/`, incoming `file:/C|/`, repaired `file:///C|/`, and `file:///C%3A/`. It preserves the incoming single-slash positive and adds the source-required spelling with reference-bearing diagnostics. The only removed negative is the explicitly superseded literal-pipe refusal. Encoded pipes, non-letter/long tokens, bare roots, encoded separators and prohibited leading separators remain explicit negatives, including zero-reader-entry tests under all existing root/sandbox option combinations.

The native fixture at lines 267–322 derives URL spellings from an actual native temp path rather than hard-coding a Unix-positive path on Windows. The Windows additions construct both empty-host and localhost literal-pipe URLs and pass each through the existing query/fragment expansion. Each variant checks exact path, Unicode/space/literal-percent filename, bytes and explicit HTML behavior through the loader and actual Attachments service; checks cap refusal; then repeats a valid call for recovery and confirms original bytes. The modified `assert_explicit_html_loads` helper preserves its exact expected object and changes only failure diagnostics, so its other HTML consumers retain their prior assertions.

The existing Actions capture already selects the affected owner tests and complete web-media integration target. `capture_media_cleanup.py` records the exact changed source/spec inputs and lock identity, requires positive test names once each, exact passed/failed/ignored summaries, and both command/capture exits. Its Windows selection remains six URL owner tests and 23 web-media tests; non-Windows remains five URL owner tests and 25 web-media tests. No test rename or addition changes these selected counts. The frozen workflow invokes that capture from the macOS, Linux and Windows jobs with exact-HEAD binding. No capture/schema/build declaration change is needed to discover these modified existing tests.

## Performance assessment and remaining execution gate

The prefix decision adds constant-sized work on Windows. Only the matching legacy spelling allocates a copy of the parsed pathname and asks the existing URL setter to rebuild that path before ordinary native conversion. That branch adds linear pathname work to an already linear URL conversion, with no filesystem operation, retained allocation, shared lock, background activity or work proportional to file bytes. Unix does not compile the normalization block. I found no concrete material performance regression requiring a benchmark or structural abstraction in this scope.

**Actions evidence on the new exact H is still required**, particularly the Windows URL owner controls and full `openclaw_web_media_contract`, with matching input blobs, positive-name/summary evidence and command/capture exits. The source-only reasoning above is not a native execution result. Historical CI outcomes described by the contract were not independently certified in this review and are not relabeled as R2 results. Existing broader percent/drive spelling, dot normalization, namespace/device, root/capability, identity/race, transport, codec and whole-product acceptance obligations remain open as the contract states.

No blocking or non-blocking defect was established in the declared R2 source under the reuse, structure or performance lenses.
