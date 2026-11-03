# Windows media failure diagnosis and scoped repair

## Verdict

**IMPL_BUG — Windows rooted literal drive-pipe compatibility in the existing Rust Attachments owner.** The failure is not media cleanup, a runner failure, or a reason to remove a positive assertion. The third rooted-drive positive, `file:///C|/`, is the triggering value identified by static tracing of the actual locked dependency. The runtime panic did not print its input, so the exact value is a source-trace conclusion, not a claimed runtime observation.

Selected effort: xhigh for platform URL/path semantics and the admission boundary; inherited model/session setting retained. Repository instructions and the full FUA-v1 spec were read, together with OCRM §17 and relevant current owner/source records. No local or htch product code, URL parser, validator, test, build, generator or linter was executed. Source/archive bytes were read as data; Git operations did not modify HEAD, index, or staging.

## Exact failure evidence

| Identity | Value |
| --- | --- |
| Target H | `3438eb5bac755c399c9ec686414bbb4c0816888d` |
| Target tree in capture | `b544c1d377ca82618cdadcf5106cb3c10d82411b` |
| Fixed OpenClaw U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| U tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Run / attempt | [41571310630 / 1](https://github.com/bhrumom/fabushi-desktop/actions/runs/41571310630) |
| Job | [125704599418 — OCRM media cleanup / Windows](https://github.com/bhrumom/fabushi-desktop/actions/runs/41571310630/job/125704599418) |
| Failed step | `Execute media cleanup owner and real service contracts` |
| Artifact | [13201906203](https://github.com/bhrumom/fabushi-desktop/actions/runs/41571310630/artifacts/13201906203) |
| Artifact name | `ocrm-media-cleanup-Windows-3438eb5bac755c399c9ec686414bbb4c0816888d-41571310630-1` |
| Artifact ZIP | 90,120 bytes; reported SHA-256 `8309821e43697663608552956c136e7335f5848f7fc056904a758e0a3cd68655` |
| Runtime | Windows Server 2025, x86_64-pc-windows-msvc; rustc 1.97.1 |

The job logs were retrieved directly through GitHub's job-log connector. The job-steps API confirms the test step failed and artifact upload succeeded. The artifact API binds the preserved archive to H. The decoded job log includes raw test output and the capture receipt.

At `2026-11-03T03:22:35.7587518Z` the log reports:

```text
thread 'test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references' (6504)
panicked at tests\openclaw_web_media_contract.rs:253:48:
rooted drive remains valid: LocalAccess { code: "invalid-file-url", message:
"File URL cannot be converted to an absolute local path" }
test result: FAILED. 22 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

The recorded failing capture case is `explicit-html-consumers`. The command already executed in Actions was:

```text
cargo test --locked --manifest-path source/host/Cargo.toml --test openclaw_web_media_contract -- --test-threads=1
```

Its recorded cwd is `D:\a\fabushi-desktop\fabushi-desktop`, `cargo_exit=101`, `capture_exit=0`, `execution_error=null`, and `oracle_accepted=false`. Its sole missing positive is the failing test above. The receipt says `inputs_unchanged=true`; the capture process exits 1. The other 11 cases have `oracle_accepted=true`, including cleanup owner 8/8, cleanup consumers 16/16, and pure file-URL admission 6/6. Those remain positive evidence only at their original H and scope.

## Causal source trace

At H, `source/host/tests/openclaw_web_media_contract.rs:252–253` iterates, in order:

```text
file:///C:/
file://C:/
file:///C|/
file:///C%3A/
```

All four must equal `C:\`. In `source/host/src/extensions/attachments/web_media.rs:293–338`, the third spelling passes scheme, original-host, leading-separator, encoded-separator, and required-root-segment checks. It then reaches the existing `Url::to_file_path` call, whose error is mapped to the observed message.

The actual `url-2.5.8.crate` is embedded in H's `projects/openclaw-rust/evidence/media-947-dependency-carrier-20261103` parts, member `local-file-url-dependency/url-2.5.8.crate`. Its recorded SHA-256 matches `source/host/Cargo.lock:3717–3720`: `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`. The relevant packaged source was read directly from this member, not inferred from a floating tag:

1. `src/parser.rs:1057–1079`: an empty file host returns `false`, with the remaining leading slash intact.
2. `src/parser.rs:512–565`: that flag selects the branch which appends a slash and calls `parse_path` on the remaining slash.
3. `src/parser.rs:1344–1378`: drive normalization requires the first segment position. The initial empty segment prevents `C|` from matching that position; the later leading-slash trim does not normalize it. The resulting parsed path is `/C|/`.
4. `src/lib.rs:3128–3199`: the Windows converter accepts a two-byte ASCII drive only when its second byte is `:`. Thus literal `C|` returns `Err(())`.

The same converter has an explicit four-byte `C%3A`/`C%3a` branch. The fourth positive does not need a new percent decoder. Adding percent-decoding machinery would not address the identified failure.

The version-source links matching the packaged behavior are [rust-url parser](https://github.com/servo/rust-url/blob/d6ea13c5f8e7e6e627f6390161b3e185bda5e5ce/url/src/parser.rs) and [rust-url native converter](https://github.com/servo/rust-url/blob/d6ea13c5f8e7e6e627f6390161b3e185bda5e5ce/url/src/lib.rs). Recorded packaged member SHA-256 values are `2dd4210e1773fc5d3fdc15ce506ecff30dbf8eb358a79bbbc2835cc30d513c3a` and `51f47a93654528f3b8cb92c94fc26ad25775860d84197fff0c3124a687f8c5e3`, respectively.

## Why the positive remains valid

Fixed U's [web-media.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/media/web-media.ts), blob `f48bca504f6fb77c3d094864eb656cb2b1bf25e2`, calls `safeFileURLToPath` at lines 555–560. Its package pins `@openclaw/fs-safe` 0.27.0 and allows Node `>=24.16.0 <25 || >=26.1.0`.

The exact dependency source [local-file-access.ts](https://github.com/openclaw/fs-safe/blob/67dee5ef4c03655a2aa98893539475ad041cb61b/src/local-file-access.ts), blob `58acd2b6f17aa4bbae3eb59a54190a2fc18c9791`, parses a Node URL, refuses remote hosts and encoded separators, then delegates to Node `fileURLToPath`. Its [namespace admission](https://github.com/openclaw/fs-safe/blob/67dee5ef4c03655a2aa98893539475ad041cb61b/src/windows-path-alias.ts) permits an ordinary rooted drive result.

In supported Node 24.16.0, [bundled Ada source](https://github.com/nodejs/node/blob/v24.16.0/deps/ada/ada.cpp), blob `80bec6c67f313a78847dc3d88b59a5bf3a1b9f4d`, sends an empty file host through `PATH_START` (13993–14003), consumes the leading separator (13857–13904), then normalizes the first literal ASCII drive pipe to a colon (15784–15793; the alternate URL representation has the same rule at 12167–12176). [Node's Windows converter](https://github.com/nodejs/node/blob/v24.16.0/lib/internal/url.js), blob `dfcd071073a8d31357b5885c03a6175acb4037c3`, converts the normalized pathname to the rooted native path at 1472–1507.

This is primary-source compatibility evidence. The old assertion's generic diagnostic is an observability weakness, but does not make its expected result wrong. No Node/Windows parser was executed during this diagnosis.

## Authorized repair delivered for independent review

Before editing, scoped `git status --short` and `git diff 3438…` were empty for all three assigned files. The spec amendment was written first. Only these repository paths were changed:

| Path | Exact3438 blob | Proposed blob from full-index diff |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `af7a5ffac2435d673923c2b55010bcf84e23faf0` | `8b3bc15887b5dc6ebace7d7805ae63bc125d2a8e` |
| `source/host/src/extensions/attachments/web_media.rs` | `9912720f7c6eff6b53444b1912e3b291613d33e4` | `e3af38b17fd7b91627493b89d6174636f719e109` |
| `source/host/tests/openclaw_web_media_contract.rs` | `4c7f27cc2ac3e8533a5befd5e2a9e7f841113ac6` | `94d50b5e4a9c51f97da1e3f017052615edbc64ac` |

The Windows-only owner change recognizes exactly the already-parsed first `/[A-Za-z]|/` component, replaces only its pipe with a colon via the existing URL path setter, and keeps `Url::to_file_path` as native conversion. It runs after all existing admission guards and before the final network check. It does not manually construct a native path, decode another spelling, introduce a parser/dependency/owner, or change Unix behavior. The locked `Url::set_path` implementation preserves the saved query/fragment and parses the replacement through its existing path machinery.

All four rooted positives and all pre-existing negatives remain. New negative cases cover encoded pipes, a numeric/long drive token, four leading separators, and encoded suffix separators. The existing zero-reader-entry checks exercise applicable negatives with default/explicit roots and sandbox variants. The real native fixture now also exercises ordinary and localhost literal-pipe references, both with and without query/fragment, through actual file bytes and `AttachmentsService::dispatch_gateway("loadWebMedia", ...)`, retaining cap refusal and fresh recovery. Rooted conversion and native/service failure diagnostics now include the reference.

No named test was added or removed; the affected existing names are:

- `extensions::attachments::web_media::file_url_admission_tests::remote_file_authorities_never_enter_local_reader`
- `extensions::attachments::web_media::file_url_admission_tests::invalid_file_urls_never_enter_local_reader`
- `test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references`
- `test_ocrm_web_media_file_url_native_bytes_service_and_cap_recovery`

Complete three-path diff: `reviews/media-windows-3438.patch` relative to the workspace root. Independent review and new frozen-H Actions evidence are still pending. Root owns capture/workflow/ledger publication. The direct next instrument is the changed URL admission controls plus the full web-media contract in Windows Actions; preserve still-valid unrelated evidence at its original subject. No rerun-to-green, assertion deletion, generic fallback, or global acceptance upgrade is warranted.
