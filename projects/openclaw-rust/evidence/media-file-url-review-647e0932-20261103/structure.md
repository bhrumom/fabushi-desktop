# FUA-v1 round 1 — independent structure, reuse, and performance review

Verdict: **NO FINDINGS** within these lenses. No actionable defect was established in the complete frozen seven-path candidate. This is source-review convergence for this reviewer; it is not compilation, native execution, full URL/filesystem parity, or runtime acceptance.

Review configuration requested by the assignment: `gpt-6.1-sol`, `xhigh`, because the complete scope includes a native-platform filesystem/security boundary. The worker used the host-selected session configuration without launching another AI process or changing configuration. No product code, tests, scripts, parsers, validators, generators, linters, or builds were executed. Only read/hash/Git operations were used before writing this report. No other review reports or prior review-runner decisions were read.

## Frozen subject and identity checks

- Repository: `bhrumom/fabushi-desktop`.
- Base and starting HEAD: `947e980bd3c9dcb06535127f5cedfd193b83f0fc`, tree `686109128a616195e65d1c9018ccd5b0b9d2cc8a`.
- Candidate tree declared by the frozen manifest: `27b6ac5b5e90023dff847e73e8c564743dd8e109`.
- Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Manifest: `/workspace/scratch/7d27ea239237/file-url-r1-manifest.json`.
- Candidate and baseline read from `file-url-r1/` and `file-url-r1-base/`; no mutable product worktree source was used.
- The complete binary-safe diff SHA-256 independently matched `60f999d9097f70f0f0225e28e68be0b82a42eb6dd40f3587e750ff787b0d8411`.
- All seven candidate SHA-256 values and raw Git blob hashes independently matched the manifest. All five modified baseline file blob hashes independently matched the manifest's `base_blob` values. The manifest's remote tree acquisition/readback remains the runner's evidence; this reviewer independently checked the supplied bytes and blob identities rather than repeating remote tree acquisition.

| Changed path | Candidate Git blob | Review disposition |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `8057987d1ec0ed53f87343f1ff65063808eca44a` | Complete new contract read, including requirements, native seam, oracles, exclusions and provenance. |
| `projects/openclaw-rust/evidence/media-file-url-admission-647e0932-20261103.json` | `95b07bf1b78ef149b7dabb5ed2926693247219fd` | Complete evidence declaration read; source, execution and acceptance claims checked. |
| `projects/openclaw-rust/parity-ledger.json` | `df9cba558cccc210b35a811e9b5f9f3f8817cd79` | Every changed hunk plus complete affected responsibility row read. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `1d74ab6a725b6bab93428ce571d16944a05afa92` | Both changed reference additions and surrounding requirement/acceptance contexts read. |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | `7eed829018250ca297538df614f3929ea5a09058` | Complete script read against the baseline diff, including old qualification/oracle logic. |
| `source/host/src/extensions/attachments/web_media.rs` | `3e26c08770379911d6502bb778c0078deda822ed` | Complete candidate read against the baseline; admission, moved body and unit tests deeply reviewed. |
| `source/host/tests/openclaw_web_media_contract.rs` | `752687912e16ceb804fa63b6a172251dc53d0943` | Complete candidate read; old tests retained and new fixture/capture contracts checked. |

## Structure and reuse assessment

`web_media.rs::file_url_to_path` (line 261) repairs the existing public converter in place using the already-declared `url::Url`; it removes the host-name denylist and failed-conversion manual fallback. The target Host and App locks both retain `url` 2.5.8 with checksum `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`. No dependency, platform bridge, parser runtime, registry, permission store, or persistence owner is added. Removing this function's manual percent-decoding does not strand the `percent-encoding` dependency: existing `media_reference`, `media_fetch`, and settings code still use it.

`load_web_media` (line 290) always supplies `load_local_web_media` to the private `load_web_media_with_local_reader` seam (line 299). The public signature and options stay unchanged, and no service argument or public option can inject a reader. The callback is synchronous and `FnOnce`, with no heap-allocated trait object, lock, task, shared mutable state, or retry loop. It is a narrow and proportionate seam for counting local-body entry without contacting a Windows share; it does not represent a read-capability grant.

The production admission phase contains string cleanup, raw-path checks, URL conversion, workspace joining, and the final resolved-path check. The extracted local body begins at line 346. Baseline-to-candidate reading shows its original root canonicalization, bounded read, cap/error mapping, filename extraction, explicit HTML policy, image probing, and image finalization remain in their original order and with their original logic. There is no duplicated body or second reading path. Its call to `media_reference::read_outbound_media_file_bounded` still owns metadata/open/bounded buffering after admission.

The final Windows prefix match distinguishes rooted `VerbatimDisk` workspaces from `UNC`, `VerbatimUNC`, device and other prefixes. The existing raw-string guard still runs first; the private workspace allowance therefore does not broaden the existing public raw verbatim-path contract. The new Windows positive records that an ordinary relative reference under a rooted verbatim-drive workspace enters once, while mixed-separator, UNC and device workspace negatives stop before the body.

Adjacent exact source was read for `AttachmentsService::dispatch_gateway("loadWebMedia", ...)`, `media_reference`, `media_file_context::resolve_local_media_path`, `connectors::channel_attachment`, the Runner file-URL metadata helper, module/build declarations, Gateway protocol and the finite Host registry. Unchanged source was read with `git show 7dd8664f5f5a48864a71accac9d587c419fde5d7`, as authorized by the handoff; the current HHP service/body and exact-H workflow material were read from the supplied immutable 947 snapshots.

The channel resolver is not a drop-in shared converter: it returns `Option`, supports managed inbound references and home-directory behavior, and its consumer reanchors paths, applies its upload contract and converts errors to absence. Substituting that route would alter this loader's typed `WebMediaError` contract and consumer responsibilities. FUA changes the existing converter rather than adding a parallel implementation, and leaves the separate channel consumer intact. The bounded media reader is reused rather than reimplemented. The real finite registry still excludes `loadWebMedia`, consistent with the candidate's direct-service-only claim.

## Capture, preservation, and cost assessment

The complete capture change adds one focused lib-test selection and extends the existing integration selection. Existing cleanup, HHP-v2/IPB-v1 commands, test lists, exact-name/summary/exit oracle, input identity checks and thirteen synthetic qualification cases remain unchanged. The five owner names, Windows-only sixth owner name, three cross-platform integration names, and Unix-only alias name match the actual definitions and `cfg` declarations. No expected-name duplication or orphan selection was found by source reading.

The new selection is exactly:

`cargo test --locked --manifest-path source/host/Cargo.toml --lib extensions::attachments::web_media::file_url_admission_tests:: -- --test-threads=1`

The integration command remains the existing complete `openclaw_web_media_contract` target. Summing the explicitly selected names gives 79 for Linux/macOS and 76 for Windows, matching the declaration. These are selected source counts, not observed execution results. The supplied exact-H workflow continues to call the shared capture on the native Linux, macOS and Windows jobs and upload `.evidence/media-store-cleanup/` under subject/run/attempt identities.

Production work remains linear in reference/path length plus the existing bounded local read. The new checks do not add filesystem operations, payload copies, network resolution, caches, threads, or background activity. The private generic seam adds no required heap allocation or dynamic-dispatch architecture. No material runtime or build-reach regression is established; an additional focused Cargo invocation uses the existing Host test target rather than introducing another dependency or platform job.

The changed ledger/RTM declarations retain historical reading, `verification=not-run`, `wiring=not-wired`, `acceptance=pending`, and the existing completion boundaries. They identify the new behavior without promoting this source repair to full fixed-U reading, safe-reader authority, public Gateway exposure or complete media acceptance.

## Provenance inspected and limits

The supplied fixed-U `web-media.ts` and Windows test independently hash to the declared `f48bca504f6fb77c3d094864eb656cb2b1bf25e2` and `bdaedcde111ea2880b96d9afa547a9c1d2a36b42` blobs. The actual caller imports `safeFileURLToPath` from `@openclaw/fs-safe/advanced`, wraps its errors as `invalid-file-url`, and checks resolved local paths before local reads. The archive SHA-256 independently matches `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`. Its package export and `advanced.js` lead to the read `local-file-access.js` contract; the supplied member hash matches `7530a957ca9d56f56deba8da1dd06be03c894c6f6472bc8571080937b57bfda8`. The evidence correctly keeps complete Windows filesystem-alias and dependency closure outside this bounded repair.

The repository root review rules, OCRM execution amendment §17, project authority entry, and the read-scope spec were consulted. No applicable nested `AGENTS.md` was found for these paths in the supplied exact Git source.

No compilation, test execution, parser experiment, Windows native-path experiment, syscall interception, or benchmark was performed. In particular, zero injected reader entries plus source-established admission purity are a meaningful boundary oracle, but no observed candidate results exist in this review. Exact-H Linux/macOS/Windows Actions evidence, other independent review lenses, and the broader scope already listed as open remain required. There is no finding requiring a fix round from this reviewer.
