# Media drive-pipe recovery — blind general review, round 2

## Verdict

**SOURCE-ONLY APPROVAL. No blocking or non-blocking defect finding was established in the declared three-path change.** The guarded Windows normalization implements the amended compatibility contract without bypassing the existing URL-admission or local-reader boundary. The oracle amendment is supported by independently inspected fixed-upstream and dependency source; it does not merely change an expectation to match the Rust library.

This is an independent source-review result, **not executable verification, bounded runtime acceptance, or full media/security acceptance**. Exact new-H Actions execution remains required. No product, parser, test, build, lint, generator, validator, harness, or acceptance code was run locally or on htch-runtime.

## Review identity and method

| Item | Identity |
| --- | --- |
| Reviewer | `/root/media_pipe_general_r2_blind` |
| Date | 2026-11-03 UTC |
| Model and effort | Inherited configured parent model **GPT-6 Astra Pro**, inherited **xhigh** effort; no model/effort override or adapter fallback applied |
| Effort rationale | Windows parsing and native conversion interact with security admission, lifetime/permission boundaries, an upstream compatibility oracle, and concurrent contract reconciliation |
| Repository | `/workspace/scratch/425c55809bda/fabushi-desktop` |
| Immutable review base | `fee4f179b640489774208fa3ce653a3868b641bf` |
| Starting HEAD recorded by manifest | `0e845ce1679e0cdf95346adc54f5d58bf06464e8` |
| Reviewed tree | `5a207a6a1914ba448f4511ed48e68787fb7f2d42` |
| Prior tree for supplied fix delta | `7ae5865f15158a5eeec67ac1a34e2a49f1171dd6` |
| Fixed OpenClaw U | `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700` |
| Scope mode | Exact three media paths, including reconciliation of the incoming historical oracle amendment |

The actual review mode inherited the configured model and effort. This worker did not request a substitute model. I read `manifest.json`, the complete frozen-scope patch, and the round-2 fix delta. Repository sources were read from the immutable tree with read-only Git operations. I did not read earlier/current reviewer reports or arbitration decisions. Parent-provided dependency locations and identities were acquisition pointers; the conclusions below come from my own source inspection.

The three scope blobs were independently read from the tree and match the manifest:

| Path | Reviewed blob |
| --- | --- |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `5733cb70842535cde24eb2bd0465a42c864e15ac` |
| `source/host/src/extensions/attachments/web_media.rs` | `e3af38b17fd7b91627493b89d6174636f719e109` |
| `source/host/tests/openclaw_web_media_contract.rs` | `1423e74eea806fe7656b29d14aa96dfd0c47174c` |

Applicable instructions and authority inspected: root `AGENTS.md`, the relevant current authority/execution sections of `projects/openclaw-rust/SOURCE_OF_TRUTH.md`, main migration spec §17, the media/module ownership mapping, and the complete FUA specification. The tree contains no more-specific `AGENTS.md` governing these paths.

## Independent compatibility trace

### Fixed upstream actually delegates this behavior

At fixed U, `src/media/web-media.ts` imports `safeFileURLToPath` from `@openclaw/fs-safe/advanced` and calls it for a file-scheme media reference before local read handling, wrapping failure as `invalid-file-url` (blob `f48bca504f6fb77c3d094864eb656cb2b1bf25e2`, lines 4 and 553–560). The fixed-U `package.json` binds fs-safe 0.27.0 and permits Node `>=24.16.0 <25 || >=26.1.0` (blob `6989c0618742d3c2b93cc336cbe1d9c3c4f0a657`, lines 2412 and 2533–2535). The root importer in `pnpm-lock.yaml` resolves 0.27.0 (blob `83b4975eb96942ec007a2eb9847ccfe98d380bda`, lines 293–295).

I inspected the exact fs-safe commit `67dee5ef4c03655a2aa98893539475ad041cb61b`: `src/advanced.ts` exports the local-file-access owner, and `src/local-file-access.ts::safeFileURLToPath` constructs a Node URL, checks scheme/host/encoded path separators, invokes `fileURLToPath`, and checks namespace/network aliases afterward. The local packaged `dist/local-file-access.js` was also read; its SHA-256 is `7530a957ca9d56f56deba8da1dd06be03c894c6f6472bc8571080937b57bfda8`, matching the spec's packaged-source identity. This review does not claim to have re-acquired or verified the full fs-safe archive.

Exact fs-safe source identities inspected:

- `src/local-file-access.ts`: `58acd2b6f17aa4bbae3eb59a54190a2fc18c9791`.
- `src/advanced.ts`: `e89dc96c9dd57eaa88cdfdeeb6877727c8769876`.
- `src/windows-path-alias.ts`: `9ddd757485447258e98b193dc1a07cbc171bd2b8`.

### Allowed Node source supports the positive

I independently fetched Node **v24.16.0**, without substituting the locally available older Node source:

- `lib/internal/url.js`, blob `dfcd071073a8d31357b5885c03a6175acb4037c3`: the URL constructor reaches the native parser (lines 824–850); `getPathFromURLWin32` rejects encoded separators, translates separators, percent-decodes once, and requires a drive colon for an empty-host local path (lines 1472–1507).
- `src/node_url.cc`, blob `5f5241f6c6741218024452202c4a17c7d0239e6b`: `BindingData::Parse` calls `ada::parse<ada::url_aggregator>` (lines 384–425).
- `deps/ada/ada.cpp`, blob `80bec6c67f313a78847dc3d88b59a5bf3a1b9f4d`: an empty file host goes to path-start; path-start consumes the separator; first-component file-drive handling replaces a literal pipe with a colon (lines 13857–13904, 13984–14025, 15783–15803).
- `deps/ada/ada.h`, blob `8f9089d2210a746a11141b1f6587ae4908ad24b3`: the drive predicate uses an ASCII letter followed by colon/pipe and a valid segment boundary (lines 1205–1218).

That source chain supports `file:///C|/` becoming the rooted Windows native path `C:\`, including the localhost form. This is source-derived compatibility evidence, not a claimed Node execution.

### The locked Rust discrepancy is real and narrowly addressed

Both host and app locks bind `url` **2.5.8**. The inspected archive `/tmp/fabushi-647e-pairing/local-file-url-dependency/url-2.5.8.crate` has SHA-256 `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`, matching both locks. Archive content was read directly; no parser was executed.

In that archive, `parser.rs::parse_file_host` returns the empty-host flag, which selects the extra-slash path in `parse_file` (lines 512–565 and 1057–1079). The first-component normalization condition in `parse_path` can consequently be missed before leading empty segments are removed (lines 1344–1378). The Windows converter accepts a literal colon or encoded colon in the drive component, but refuses the literal pipe (lib.rs lines 3128–3191). This independently supports the spec's traced cause. The earlier failed log did not identify a particular loop iteration, and this review does not turn that inference into a runtime observation.

## Correctness, security, lifetime, and permissions

The production change is confined to `file_url_to_path`, lines 333–347. Its placement and shape satisfy FUA-PIPE-01:

1. The existing real parser, file-scheme check, removed-original-host callback, parsed-host check, original leading-separator check, encoded path-separator check, and Windows bare-root check all run before the new branch (lines 298–330).
2. The branch is compiled only for Windows. It matches exactly the leading parsed slash, one ASCII letter, a literal pipe, and a root slash. Encoded pipes and longer/nonletter drive components do not match.
3. The slice pattern establishes the ASCII byte positions before `replace_range(2..3, ":")`, so the mutation cannot split a UTF-8 character. It changes an owned copy of the parsed path, then uses the existing URL setter and native converter. It neither fabricates a native path nor catches/retries a failed conversion.
4. The inspected `Url::set_path` implementation preserves the already encoded path and saves/restores query/fragment components (lib.rs lines 1721–1729, 1751–1789). Its parser percent-encoding set does not decode existing percent escapes. This supports preservation of the fixture's encoded Unicode, space, and literal-percent filename through the additional setter call.
5. The final converted-network-path refusal remains after native conversion (web_media.rs lines 349–354). Conversion failures still become `invalid-file-url`.

The actual load path remains `load_web_media` → private `load_web_media_with_local_reader` → `load_local_web_media`. Reader admission occurs at line 411, after URL conversion and resolved-path guards. Root canonicalization begins only in the local body at line 421; the shared bounded reader is reached at line 448. I also read `media_reference.rs::read_outbound_media_file_bounded`, its ordinary-file metadata admission, opened-file admission, and bounded read loop. The new URL branch adds no filesystem call before that boundary and does not alter those responsibilities.

The new allocations and URL mutation are synchronous local values. No escaping borrow, retained callback, shared mutable state, task, handle, worker, cache, cancellation state, or cleanup obligation is introduced. There is no lifetime/concurrency defect identified in this delta.

The direct `AttachmentsService::dispatch_gateway("loadWebMedia", ...)` arm still constructs ordinary loader options and delegates to the same loader (attachments_service.rs lines 2006–2025). The production gateway registry still has no `loadWebMedia` entry; `is_grok_gateway_command` remains driven by that finite registry. The patch does not add a public route or permission/capability bypass. Existing default-root, sandbox-authority, namespace/device/ADS, and file-identity/race limitations remain the explicitly deferred scope of FUA; this approval does not close them.

## Oracle reconciliation and test adequacy by source

The complete patch retains the historical FUA-WIN-ORACLE-01 text, then explicitly supersedes its `file:///C|/` refusal in FUA-PIPE-01. The later rule is precise about preserving FUA-02 failures after the guarded normalization. The fix delta restores that historical context and preserves the incoming `file:/C|/` positive. The resulting spec and tests are consistent rather than leaving two unqualified current expectations.

The rooted-drive loop now retains all five required positives at test lines 258–263:

| Reference | Required Windows result |
| --- | --- |
| `file:///C:/` | `C:\` |
| `file://C:/` | `C:\` |
| `file:/C|/` | `C:\` |
| `file:///C|/` | `C:\` |
| `file:///C%3A/` | `C:\` |

The one superseded literal-pipe refusal becomes its source-supported positive. The other previous negatives are retained. Added negative controls cover encoded pipes in both percent-hex cases, a nonletter drive, a long drive token, prohibited leading separators, and encoded path separators in literal-pipe URLs. In the private boundary tests these run with the existing roots-present/absent and sandbox-false/true combinations and assert zero reader entries before checking the error code (web_media.rs lines 530–633). These are no-I/O boundary controls, not live share probes or claims about arbitrary OS syscalls.

The actual native temp-file fixture adds both ordinary and localhost literal-pipe references, then tests each with and without query/fragment components (test lines 266–321). It preserves the filename `café 100% report.html`, verifies exact decoded path/bytes/name/content type through the rooted loader, invokes the actual service via the common helper, checks byte-cap refusal through loader and service, and verifies successful subsequent reads. The helper compares the whole serialized result, including the absence of generated-HTML trust (lines 513–547). The helper edits otherwise change failure diagnostics only. Both new pipe references are derived from the actual native temporary path; the hard-coded `C:\` assertions are conversion-only and perform no filesystem access.

I found no source-level weakness that makes these controls pass without exercising the changed branch and real consumer behavior. Removing the new branch would leave the triple-slash native pipe fixture and retained root positive unable to convert on Windows. Broadening admission to encoded/nonletter/long tokens or moving failures after the reader is constrained by the new and retained negative controls.

## Execution gate still outstanding

The existing capture source was inspected, not run. `capture_media_cleanup.py` retains exact positive-name uniqueness, complete summary/count checks, zero command/capture exits, before/after input identities, raw-log hashes, and exact subject/upstream receipt fields. It still selects the changed private URL owner controls and the complete `openclaw_web_media_contract`. Expected counts inferred from the frozen source are **6 URL-owner tests and 23 web-media tests on Windows**, and **5 and 25 respectively on Linux/macOS**; these are not observed passes.

The current Windows workflow checks out and binds the requested subject, uses the existing capture, and uploads its evidence even after a failed capture. No changed test name or weakened capture selector was introduced by this patch.

Before accepting the repair, obtain the required focused Actions results on the new frozen H, including native Windows execution of the affected owner controls and full web-media target, exact input/lock identities, complete unique positive results, summaries, command and capture exits, and raw logs. Reuse unrelated still-valid evidence only with its original subject and applicability intact. Preserve the historical failures. Source approval here must not be promoted to executed, verified, accepted, or a global completion flag.

## Findings

**None established in the frozen change.** No repository source was edited by this reviewer. This report is the only file written.
