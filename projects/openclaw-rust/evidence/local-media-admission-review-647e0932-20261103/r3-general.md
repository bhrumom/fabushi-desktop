# Media admission R3 — focused GENERAL / security / correctness review

## Terminal verdict

**APPROVED for the exact three-file integration source scope below.** No blocking correctness or security finding was identified in this integration delta. Findings: **0**. This is a terminal focused source-review verdict, not an approval of the complete LFA/FUA/BR implementation, an executable-validation result, or permission to promote verification/acceptance state.

The separate complete R2 reviews must converge, and the resulting exact published H must obtain its required GitHub Actions evidence. This report may be joined with those reviews only for the source bytes they actually cover.

## Identity and review boundary

- Repository: `bhrumom/fabushi-desktop`.
- Branch: `codex/openclaw-rust-fusion-chat-20261101-writer`.
- Task: `647e0932-bea4-4113-9a61-65ae3db58955`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Frozen integration base / starting HEAD: `3438eb5bac755c399c9ec686414bbb4c0816888d`, tree `b544c1d377ca82618cdadcf5106cb3c10d82411b`.
- Prior complete source candidate R2: tree `a4f1c59eb577a45c1b05da1de0a8f13644dc1a5c`.
- Reviewed R3 candidate: tree `e9a172d00139550e8a055a61bf06e225b9d4166c`, identified by the frozen `media-admission-r3-manifest.json`.
- Immutable source directories: `/workspace/scratch/7d27ea239237/media-admission-r3`, `media-admission-r3-base`, and `media-admission-r2`.
- Complete 23-path patch: SHA-256 `e7601e75db970790c37844c9ed972a56cf04174708461ffc9deb4a1c11642adc`.
- Focused R2-to-R3 integration patch: SHA-256 `7789887f05eee4f5dc6f3a3801b60ebd716b07ca8df07b9d74a7a8cb0a47c539`.
- Effort: `xhigh`, explicitly applied as specified in the parent handoff; configured model retained.

Both patch hashes were independently read back. A recursive byte comparison of the R2 and R3 snapshots reports exactly the workflow and ledger changes plus the extracted portable script; all other 20 paths are byte-identical. The complete patch and 23-path manifest were retained as context. This reviewer did not read another reviewer's report, a prior review report, runner arbitration decisions, or mutable WIP source.

The three reviewed R3 file identities were independently checked with SHA-256 and Git blob hashing:

| Path | Git blob | SHA-256 |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `9419b5467e6882a7aeff16c91aa97317a4e001eb` | `6b341356e10d3699634461f62789e3bda98daecd37ff61904788e5f7bcb81e23` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `dce650d0425bc35db032234753cf5e4ec298896c` | `101b33df807b86e7d9cd1d3544f36c0ac3b05844b8033a8309067f7a9e45d3e4` |
| `projects/openclaw-rust/parity-ledger.json` | `fda6a051b5f15ac379bc289f00290a4c25638d98` | `fa03fe933ca7228dde76428408139d031de90a46e1438a66c9f93d0fe2a81a37` |

The corresponding base files also match the exact base Git tree: workflow `6cdef39587b810e5d5478e4d6954594fe13cd570`, portable script `1f737412c18369d6c722c4d739e6aba1b17f1ce8`, and ledger `9882c933017908bca392a5635e87fd35e1c2d3fd`. The reported 10,502-entry remote candidate-tree readback is the integrator's manifest evidence; this reviewer independently checked the local reviewed blob identities and did not claim to repeat that complete API tree enumeration.

## Governing source and method

Read the exact root policy blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2`, including immutable/fresh/blind review and Actions-only rules; the current source-of-truth authority and execution constraints from blob `3748d734b10c9ed5b2d948467fbaa9483f1dd306`; and the operative main-spec §17 from blob `69c9667af2328e20a1fba09ebf80c5aab4d5c336`. Also read the candidate LFA and FUA specs and the relevant capture, workflow, App registration, test-selection, and ledger source. Exact-base Git tree inspection found no more-specific `AGENTS.md` governing these paths.

Only source/byte/hash/Git/API reads were performed. No local or htch product execution, parser execution, capture execution, test, build, lint, schema check, or generator was run. Source-text extraction for byte comparison did not execute the extracted Python or workflow. The only authored file is this report.

## Findings and supporting assessment

### 1. The extraction preserves the original capture and the R2 additions

The original 997-line `channel_pending_recovery` Python body was independently read from exact commit `0e845ce1679e0cdf95346adc54f5d58bf06464e8`. Removing only the YAML indentation produces SHA-256 `8f0044fd5f4d4d4fe6224d1d146ec47ee5e9f7dac3e7ba9722259b4514b1582f`. It is byte-identical to the base extracted script after removal of its single self-input entry.

The R2 inline body contains the two LFA names and the 15-to-17 summary update. Independently deindenting its 999 source lines and comparing them with R3's script after removal of the self-input entry produces exact equality: SHA-256 `60c113c57f2181ec200762378ab4d533fb1157f9666524605d7861849942d9fe` for both sides.

Consequently the R3 move does not lose or duplicate a case, command, required name, oracle control, receipt field, or palette assertion. The only R2 workflow hunk replaces that inline body with the single script invocation. There is one portable invocation in the workflow and no remaining inline copy of its canonical-result oracle.

### 2. The workflow executes the tracked script from the exact checkout

The `sqlite-native-path` job checks out `github.event.pull_request.head.sha || github.sha`, compares actual HEAD with `FABUSHI_TARGET_SHA`, and enforces a supplied frozen dispatch SHA. It retains its Linux runner, toolchain, Host cache, profile environment, owned temporary fixtures, and step order.

The `channel_pending_recovery` step retains its event condition, 20-minute timeout, `TMPDIR`, palette fixture path, Bash shell, and `set -euo pipefail`. It directly executes `python3 projects/openclaw-rust/scripts/capture_portable_contracts.py` without a checkout change or working-directory override. The script repeats the actual-HEAD comparison before running its cases, and its receipt records subject commit/tree, upstream, run/attempt, job, OS, compiler identity, and input hashes. The new script path is itself in `inputs_sha256`.

The stdin-to-file move does not introduce a use of `__file__`, command-line arguments, a changed working directory, or a second runner implementation. Inspection of the exact-base scripts directory found no module that shadows the standard-library imports used by this script. The script is tracked as a regular `100644` blob and is invoked through Python, so executable permission is not required.

The fresh palette-file absence assertion, actual producer-name requirement, palette file digest, and producer receipt remain intact. The downstream Node consumer still requires successful portable-step outcome and matching subject/tree/fixture digest. Failure-fixture preservation and the existing native-path artifact upload continue to cover the same `.evidence/sqlite-native-path/` output tree.

### 3. Selected test names and result counts remain strict

The generated SendMessage case still invokes the same three complete Cargo test targets, with locked dependencies, `--no-fail-fast`, and one test thread. The source declares 17 SendMessage tests, 14 Session conversation tests, and 13 generated-occurrence tests. The required list contains the corresponding 44 names, including exactly one occurrence each of the two LFA facade tests.

At script lines 963–976, each required positive must occur exactly once; both command and capture must succeed; the sorted summary tuples must equal `(13,0,0)`, `(14,0,0)`, and `(17,0,0)`; and the total observed positive-name count must equal the required-list length. The changed count therefore reflects the two additional selected tests. No failure, ignored-test, name-uniqueness, summary-cardinality, or command-exit condition was relaxed. The independent synthetic capture-oracle controls and their distinction from product positives are byte-preserved.

The macOS recovery list outside the moved block retains both LFA names, and its existing command selects `send_message_tool_contract`. These are separate retained platform executions, not duplicated synthetic positives within the portable result.

### 4. The actual App and native media coverage remain wired

The six exact App names remain in the workflow and match the candidate's registered `tests::local_media_file_url_tests` module. The existing App capture still checks the shipping binary, obtains the real test executable from Cargo's compiler-artifact output, verifies that it belongs to the owned target directory, requires an unambiguous exact listing, and requires one pass with zero failed/ignored tests for each selected execution. Its LFA spec, test, shared admission owner, both tool owners, Session DB, and media-store inputs remain hash-bound.

The native capture file is byte-identical to R2, SHA-256 `f08df363a4ba718cb3ffa0b3fa0cb6f50d61ac6a214a4cf66f8da8306525df2e`. Its existing 16 groups, single shared channel-target execution, 46 explicit inputs, before/after input comparison, and LFA/FUA/BR selections remain intact. The source-spec expectations remain 163 selected positives on Linux/macOS and 158 on Windows; these are expectations and are not reported here as execution results. The workflow still invokes this capture in its native Linux, macOS, and Windows lanes.

### 5. The ledger adjustment preserves incoming accounting and unaccepted state

The R2-to-R3 ledger diff contains only `mapped_paths: 877 -> 884` and `unclassified_paths: 61253 -> 61246`. Both resulting values exactly match the immutable incoming base ledger, with `884 + 61246 = 62130` tracked paths. The four LFA responsibility amendments remain byte-identical to R2. No source-reference set, literal read/review state, implementation/wiring/verification/acceptance status, inventory authority, blocker, or completion flag changes in this focused delta.

All eight global flags remain false. No incoming inventory file is in the complete changed-path scope. This assessment establishes faithful integration of the incoming totals and preservation of the declaration state; it does not replace an Actions inventory/ledger/schema audit or independently establish full mapping completeness.

## Remaining gates and use of this verdict

1. Join this focused approval with converged complete R2 general/security and structure/reuse/performance reviews for the unchanged product/test/spec/capture scope, plus the separate R3 specialized review. No old report has been imported into this review's conclusions.
2. Publish only the reviewed tree through the existing branch's compare-and-swap boundary, with exact commit/tree/content readback. Any subsequent source delta must be assessed for invalidated review or execution evidence.
3. Confirm actual GitHub Actions workflow creation and the required selected jobs on that exact H. The reviewed workflow is 431,076 bytes versus the incoming base's 428,924 bytes; this source byte measurement alone does not prove GitHub workflow admission or successful execution.
4. Obtain the exact-H portable, App, and native media receipts, raw-log hashes, command/capture exits, positive-name/count evidence, and matching input identities. The native Windows branch must actually execute its platform-specific cases. Palette producer/consumer continuity and failure artifact preservation still require their real run evidence.
5. Keep scoped authority/coverage, full R2 source convergence, product verification, real-service/device/platform/package acceptance, G01–G12/AC01–AC24, and global completion state separate. No current-H runtime pass or full migration acceptance is established by this report.

**Final disposition: APPROVED — focused three-file integration source review; no findings requiring implementation changes.**
