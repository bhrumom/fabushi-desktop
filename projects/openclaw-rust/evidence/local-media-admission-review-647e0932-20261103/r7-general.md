# R7 blind focused general source review — LFA/FUA integration

## Verdict

**APPROVED** for the declared R6→R7 integration delta and the affected functions and adjacent consumers inspected below. No actionable correctness, integration, regression, or evidence-gate finding was established. This is a source-review verdict, not executable verification, publication approval, scoped runtime acceptance, or full OCRM acceptance. New exact-published-H GitHub Actions evidence, including native Windows execution, remains required.

## Review identity, authority, and method

- Reviewer: `/root/media_admission_general_r7`; general lens, fresh and blind.
- Configured model: GPT-6 Astra Pro, inherited from the parent with no model override. Applied effort: **xhigh**. The parent confirmed the actual launch used `fork_turns="none"`, `reasoning_effort="xhigh"`, and omitted the model override. The worker has no separate setting-introspection surface. There was no fallback or prompt-only effort substitution. The scope warrants xhigh because it combines native Windows URL semantics, pre-I/O permission boundaries, merged test oracles, and shared execution/evidence gates.
- Repository and branch: `bhrumom/fabushi-desktop`, `codex/openclaw-rust-fusion-chat-20261101-writer`.
- Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`; tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Exact incoming base / starting head: `2a6dfc0f7295b3b6ad3e372afa5be4d27951b11a`; tree `16977e8995befe89bfa975e2f1a4a98849fe6dbb`.
- Prior R6 candidate tree: `efd035bd168369022a1a8b2131511ef20daf1646`.
- Frozen R7 candidate tree: `71e8a1838cf8e925e88d514b95b35b7245895f63`. The supplied manifest records a nontruncated recursive API tree of 10,539 entries and matching changed/base blobs. I independently hashed the local source subjects described below; I did not independently reconstruct that whole tree.
- Manifest: `/workspace/scratch/7d27ea239237/media-admission-r7-manifest.json`, SHA-256 `9f178b177141341c87a733c3408bd401afbb2bed14053076a890bb9b3125b10e`.
- Complete 23-path patch: `media-admission-r7-complete.patch`, SHA-256 `36a538ebc6ca77e8b4427fef84d356b26666e64f356641f50504326edaecbd5b`.
- Complete focused patch, read in full: `media-admission-r7-integration.patch`, SHA-256 `ab3de4b75840481cc30d3872b810d921f1a4ab6e92814576559678fe1a20e7a7`.

Both patch hashes were independently recomputed. The 18 paths outside the focus delta were independently SHA-256 compared between the R6 and R7 snapshots: all 18 match, including both hash-only evidence files. Candidate SHA-256 values match the manifest. Git blob hashes of all five focus files, their incoming-base copies, their R6 copies, the current LFA spec, and the shared converter were independently recomputed and match the supplied identities.

Authority was read from immutable commit `370c6680b6b8c072615875d72be67ca0e45ebe45`, not working bytes: root `AGENTS.md` blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2`; OCRM Source of Truth blob `3748d734b10c9ed5b2d948467fbaa9483f1dd306` (effective authority/constraints); main migration spec blob `69c9667af2328e20a1fba09ebf80c5aab4d5c336` §17 including OCRM-EXEC-02; and the relevant module-map owner boundary, blob `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3`. The complete current LFA and FUA specifications were read. The exact incoming-base instruction inventory has no applicable nested AGENTS under this scope; its only additional AGENTS is in the unrelated Mahayana TUI subtree.

Only source text, immutable Git reads/diffs, and hash operations were used. No local or htch tests, builds, parser execution, lint, schema checks, generators, or product execution occurred. Only this report was written. No source, status, ledger, global flag, or acceptance record was changed.

Blindness was preserved: no prior/other review, runner decision/adjudication/checkpoint, publication metadata, incoming decision JSON, or incoming evidence report content was read. The two named evidence JSON files were hash-only. Their complete-patch hunks were not read. Historical result/review references inside the required specifications were treated as specification history, not as independent proof or prior verdict authority.

## Focus source identities and coverage

The following paths were reviewed through every R6→R7 hunk plus their affected functions, callers, and capture consumers. The incoming base-to-R7 source diffs were also read for the workflow, portable capture, web owner, and web contract, so the integration was checked against both source lineages.

| Path | Incoming base blob | R6 blob | R7 blob |
| --- | --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `765a757720343f49b67127caedd2530ad4344db6` | `eebbd9c203d4869e23f737b741b6e56b20cce7f2` | `5abace3386897aaca3c4a256a11f90b8e3b98119` |
| `projects/openclaw-rust/docs/specs/media-file-url-admission.md` | `5733cb70842535cde24eb2bd0465a42c864e15ac` | `5414d7193cc7cbcbcfec51dec54cda68be5130ad` | `8efd93cc604fa814b4660b5153b6b38033faa2ca` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `55b17de374467b25dc98b5d121f73116b029f85b` | `62bc1cd60cf33a2f5a6c4ce6f8a61d70bbf546ad` | `ee404c54ce24a740f6808a8c7bba9650a0f0e2d6` |
| `source/host/src/extensions/attachments/web_media.rs` | `e3af38b17fd7b91627493b89d6174636f719e109` | `6447eff570c31939d3165ac72271cc1914d84055` | `bf7a38dfa100fadb6c0cc15b6bf699884c67299b` |
| `source/host/tests/openclaw_web_media_contract.rs` | `1423e74eea806fe7656b29d14aa96dfd0c47174c` | `4d7829bd42995f9503d6c9fee390986e8c1c724e` | `315cda26d1e62b0616881857d6f9047fc883abd1` |

The complete frozen 23-path scope is accounted for by these five focus paths and the following 18 unchanged paths. “Context” means the cited behavior/interface was source-read for this focused review; “identity” means no fresh whole-file semantic verdict is asserted. No earlier review verdict is being reused as source evidence.

| Unchanged path | R7 blob | R7 disposition |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/local-media-file-url-admission.md` | `9249e4a93f353247e97c7ef47339382932451d72` | Complete specification read |
| `projects/openclaw-rust/evidence/local-media-file-url-647e0932-20261103.json` | `0722207d574b638c1e9984737f85cb05fa3aa3b7` | Hash-only, contents excluded |
| `projects/openclaw-rust/evidence/media-file-url-normalization-recovery-647e0932-20261103.json` | `721190ec1e165a8fbdf9e5764d779796000e71e9` | Hash-only, contents excluded |
| `projects/openclaw-rust/parity-ledger.json` | `3fb5f2b4b5cade7d15d3cbedacc01c8a5dcd1cf0` | Identity; unchanged, no promotion |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `dddcfc37286add1e5c3be86bed9eb346057b0fbb` | Identity; unchanged, no promotion |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | `dfc83dd491046de7708643ce48b0fbebd74a19bb` | Complete capture source read |
| `source/host/app/src/main.rs` | `4831ae8cf002ce64e6409f2d695b48aaa1914c45` | Context: resolver and both production sinks |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `5682e951f1eac8f107efd3a3948b0748870912bd` | Identity; selection retained |
| `source/host/src/connectors/channel_attachment.rs` | `723006c2a5b054fc9ddc47f58fdb4ebd8639351e` | Context: complete local reader route |
| `source/host/src/extensions/attachments/media_file_context.rs` | `0777b2fff28f09e5a6f276ed71c6ec776d6601e6` | Context: local-media resolution |
| `source/host/src/extensions/attachments/media_reference.rs` | `0956a0db7df81390002fde917ebc3fe782e75296` | Deep context: shared URL conversion and spelling admission |
| `source/host/src/runner/tools/sand_agent_management_tools.rs` | `22e7f260ca0d23250802af3d862cf836b21d675b` | Context: batch preflight and sink ordering |
| `source/host/src/runner/tools/send_message_tool.rs` | `9e12d7d2b457715f0fac168e5e4f262e6ef15985` | Context: file-path metadata facade |
| `source/host/tests/channel_attachment_contract.rs` | `887f2cb3ea78060be617ad1c039f5b28306c6ac0` | Identity; selection retained |
| `source/host/tests/local_media_file_url_contract.rs` | `ff2d348d55572394f547deb15c2e36b2347ea43f` | Complete shared-owner contract read |
| `source/host/tests/openclaw_media_file_context_contract.rs` | `2c2365d9f2ab380bd2fc4374dba4096a1911fd0f` | Identity; selection retained |
| `source/host/tests/sand_agent_management_tools_contract.rs` | `28372cd0046620bc88e9d84ca016692e0c89fa24` | Identity; selection retained |
| `source/host/tests/send_message_tool_contract.rs` | `8c334d36e4b507842a38509b945956fe5085beb7` | Identity; selection retained |

Additional immutable adjacent source at the incoming base was read: `attachments_service.rs` blob `75a49490ee80e7416df66df2431d2f3013f1b17c`, `gateway_protocol.rs` blob `9eba2275f97ebaa9b30ac057f76dc610e862d5e7`, and `device_token_rotation_wire_gateway_contract.rs` blob `fb038d518350deebcb22e8f48a87e2d5a8c3b96e`. Host/App lock blobs are respectively `9895ce7752768b1c8d3fe107ef95e513457ca518` and `5f929af729ac7b537e054283c5977cc269be38ad`. These paths are outside the complete changed-path manifest, hence inherited unchanged into its declared candidate.

## Source conclusions

### 1. Conversion ownership and effect ordering remain coherent

R7 changes no production code relative to R6. Its `web_media.rs` changes are seven additional input rows in existing test functions. Against the incoming base, the independent web-local parsing/normalization body is replaced by delegation to the existing `media_reference::resolve_local_file_url`; no second production converter is introduced.

The shared owner (`media_reference.rs:86–203`) records `FileWithHostAndWindowsDrive` during the initial parse, refuses remote/userinfo/port authority, then performs the Windows-only literal rooted drive-pipe rewrite. Malformed escapes, encoded slash/backslash, invalid decoded UTF-8, NUL, missing rooted drive, native conversion failure, nonabsolute paths, and prohibited native aliases continue to fail before any reader. The rewrite does not run on Unix and does not retry a failed conversion or reconstruct a native path.

The web adapter (`web_media.rs:263–324`) delegates once and retains the raw leading-separator guard, including the drive-token interior-separator exemption, localhost forms, slash/backslash mixtures, and ignored tab/CR/LF bytes. Its stricter converted-network policy and `invalid-file-url` error class remain intact. The private loader admission path (`336–386`) performs raw/resolved network and native spelling checks before invoking the callback. Canonicalization and bounded reading remain in `load_local_web_media` (`388–491`); the base comparison shows that body, including caps, roots, explicit HTML policy, and image handling, is unchanged.

The contextual consumers continue to use the same owner: App main `639–704` propagates invalid-source errors before its first `fs::read` or Box fallback; both production sinks propagate that Result. The Runner Option facade remains a metadata boundary, Channel checks its final reanchored path before metadata/read, and AgentManagement finishes the pure image-vector preflight before entering the sink resolution loop. These contextual observations are not a fresh full-App or custody-lifecycle acceptance claim.

### 2. The pipe requirement has direct dependency-source support

Both locks select `url` 2.5.8 with checksum `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`; the provided archive independently hashes to that value. The inspected extracted `parser.rs` and `lib.rs` hash to `2dd4210e1773fc5d3fdc15ce506ecff30dbf8eb358a79bbbc2835cc30d513c3a` and `51f47a93654528f3b8cb92c94fc26ad25775860d84197fff0c3124a687f8c5e3`.

The actual Rust source supports the stated distinction: `parse_file_host` returns the empty/localhost host flag; `parse_file` adds a slash before the still-leading path; first-segment drive normalization can consequently miss a literal pipe, and later leading-empty removal does not repair it (`parser.rs:512–617,1057–1111,1344–1378`). The Windows converter accepts a literal colon or encoded colon drive component and rejects the retained pipe (`lib.rs:3128–3191`). `set_path` preserves existing percent escapes and restores query/fragment positions (`lib.rs:1745–1810`).

Only the allowed raw primary Node/fs-safe files were used for the new provenance claim. `U-node-version.mjs` explicitly admits Node 24.16.0 (SHA-256 `e63629fb4064d97c26144a1a4f33263a3a6dc625be1b8cfe4509730a6bed80b8`). The fs-safe 0.27.0 source, SHA-256 `7530a957ca9d56f56deba8da1dd06be03c894c6f6472bc8571080937b57bfda8`, checks host and encoded separators, calls Node conversion, then checks aliases/network paths (`29–54`).

For supplied Node commit `c7d10158bc31036de6783d66beaaaf551e3167aa`, the actual raw `url.js` and `ada.cpp` blob hashes match `dfcd071073a8d31357b5885c03a6175acb4037c3` and `80bec6c67f313a78847dc3d88b59a5bf3a1b9f4d`. Additional inspected raw blobs are `ada.h` `8f9089d2210a746a11141b1f6587ae4908ad24b3` and `node_url.cc` `5f5241f6c6741218024452202c4a17c7d0239e6b`. The constructor calls the native parse binding; the binding uses `ada::url_aggregator`; empty file host enters PATH_START; the first drive component becomes colon in `consume_prepared_path`; `getPathFromURLWin32` then converts the admitted rooted pathname. Relevant ranges: `url.js:824–851,1472–1507,1622–1630`; `node_url.cc:384–425`; `ada.cpp:13857–14025,15630–15657,15752–15804`; `ada.h:1205–1218`.

Thus the added FUA-PIPE requirement is supported by actual source tracing. This remains source inference, not an executed Node/Rust differential result or evidence that a particular value was printed in an older unlabeled failure.

### 3. Existing assertions are preserved and incoming controls are united

The complete R6→R7 patch removes no positive or negative assertion predicate. Each web test surface gains the four-separator pipe spelling, two encoded-separator pipe spellings, two encoded-pipe roots, and nonletter/long pipe roots. The Windows-only cases remain Windows-only. The private reader counter still requires exactly zero entries under rooted/unrooted and sandbox false/true options; the new fixtures cannot contact a real share through that seam.

The incoming-base comparison preserves every incoming pipe negative and all five current rooted-drive positives. The direct unchanged-dependency refusal remains inside the same selected test (`openclaw_web_media_contract.rs:276–293`), while the product table separately accepts both single-slash and triple-slash pipe forms. Those two assertions describe different owners and do not conflict.

The actual native temp-file fixture (`296–354`) retains the pre-existing colon/native variants, adds the LFA single-slash and authority-position pipe variants to the incoming triple-slash/localhost pipe forms, and applies query/fragment variants to each. On Windows the source loop covers 13 reference spellings, each with and without URL metadata. It checks exact decoded path, bytes, Unicode/space/literal-percent filename, explicit HTML content type, direct service result, one-byte-under cap refusal, and a subsequent successful call with unchanged original bytes. R7 changes only failure diagnostics in these assertions and in the shared `assert_explicit_html_loads` helper (`545–579`); exact whole-result equality, including absent generated trust, is retained.

### 4. Shared workflow and capture gates retain their strict behavior

The added Device HTTP name appears once in both required-name lists (`rust-desktop-runtime.yml:2547–2562`, `capture_portable_contracts.py:539–554`). It is an actual test in incoming-base source (`device_token_rotation_wire_gateway_contract.rs:897–932`), alongside the other 13 named controls. Its body sends the large distinct-scope request through HTTP to the bound owner, requires an exact 403/unchanged storage and credential authority across reopen, then performs an authorized successor rotation. The review does not claim that test was executed.

Both captures derive the expected full-target count from the required list. They retain one occurrence per required positive, zero failures/ignored/measured/filtered cases, command/capture exit checks, and final all-case aggregation (`workflow:2604–2624,2654–2672`; portable capture `1036–1088`). The R7 delta changes only the added required name and descriptive scope. The complete incoming-base comparison also preserves the LFA SendMessage union and its existing 13/14/17 target summaries rather than reverting the 17 back to 15.

The unchanged media capture still selects the whole web contract plus the private admission owner controls and shared converter tests. It binds input blobs to HEAD before and after execution, retains negative qualification of its result oracle, rejects duplicate/missing/extra positives and failed/ignored summaries, checks command/capture exits, and aggregates all 16 selected groups (`capture_media_cleanup.py:208–477`). Native Windows receives its Windows-specific controls. The workflow still invokes this capture in its Linux/macOS/Windows lanes. These are preserved source gates, not fresh test results.

## Findings and limits

**Actionable findings: none.** No severity/path/symbol/trigger/fix entry is warranted because no concrete failing scenario introduced or left unresolved by the focused delta was established.

This is a focused R7 integration review, not a claim that all 23 files or the entire candidate were newly deep-reviewed. Unchanged context and hash-only dispositions are explicitly listed above. The two historical evidence contents were intentionally unaudited. Full repository source closure, Device-wide semantics, arbitrary filesystem races/capabilities, broader Node/Rust URL parity, real service/device/platform/package acceptance, and full App failure recovery remain outside this verdict.

No executable evidence was produced or upgraded. Compilation and every required native oracle still need new exact-published-H Actions evidence with matching inputs and terminal exits. Older results retain their original subjects. No source/implementation/wiring/verification/acceptance/global status is promoted by this report.

**Terminal verdict: APPROVED**
