# Media settlement R2 — independent structure, reuse and performance review

**Terminal source-review verdict: APPROVED.** No actionable structure, reuse or performance defect was found in the complete four-path R1-to-R2 focus delta or in the affected source interfaces and adjacent consumers examined below. This verdict is source-only. It does not assert that the candidate compiles, that any test ran, that compiled feature modes were observed, or that MSU-v1/full migration acceptance is complete.

## Review identity and constraints

- Task: `647e0932-bea4-4113-9a61-65ae3db58955`, implementation round 1, review round 2.
- Applied worker model/effort: **`gpt-6.1-sol` / `xhigh`**, as configured in this native review-worker assignment. No fallback or same-session model alteration was used. The scope crosses App state ownership, Session/DB notification ordering, fixture composition, Cargo feature wiring and canonical traceability; it warrants `xhigh` under root `AGENTS.md`.
- Frozen base: commit `d70f86f33b5e3c67545954b270373e1548809f4b`, tree `4c4c9cfd3f5067fef57dca530042d149a8e2fc5c`.
- Starting integration parent: commit `871b033c5b9aabbe322cc7322af92648939db9b5`, tree `f641366eea64364f4d029487d04276c4c567790a`.
- Prior review snapshot: tree `69f77b9dad3f6e42a7d7c53aa2df8f67c1f872a1`.
- Reviewed immutable R2 candidate: tree **`9892b15758904c23f3888ba8af1b5e429f29b59f`**, local source snapshot `/workspace/scratch/7d27ea239237/media-settlement-r2/`.
- Fixed upstream: commit `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Focus patch SHA-256 independently read back: `4d9d40a81134e67f5eaaf72e30ef150852bfafe6cde33beb475c30f9fcacf90a`.
- Complete frozen-scope patch SHA-256 independently read back: `d813dbc3a88e2b685c9ee5bb50747d342edee75bb1cb0214ade9f61769ff4f22`.

The complete focus patch, manifest and affected functions were read directly. The complete frozen 20-path change inventory and full-scope patch were retained as context; focused re-review does not pretend that every line of all large inherited files was re-read. The source/read-depth table below records that distinction.

Root review authority was `/workspace/scratch/7d27ea239237/media-settlement-authority/AGENTS.md`, exact blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2`, particularly the independent/focused-review contract and model/effort policy. The immutable candidate Git tree was also fetched directly and was not truncated; its only nested `AGENTS.md` is under an unrelated `source/mahayana/codex-rs/tui/src/bottom_pane/` subtree. No more-specific instruction governs these paths.

No prior/other review report, review-runner adjudication, worker report, historical evidence-report body, or mutable `media-settlement-work` source was read. Historical evidence identities were used only to prove reference existence. No product/source edit, commit, push, dispatch, worker launch or executable validation was performed. Only this report is written.

## Findings

**No findings.** There is no supported failing scenario requiring a change in this lens. The following checks explain why the narrow source verdict is positive without turning static inspection into execution evidence.

### 1. Fixture imports and focus timestamp match their real owners

`source/host/app/src/tests/local_media_file_url_tests.rs:3` now explicitly imports `serde_json::{Value, json}`. The file uses `Value` in its bridge/sink signatures and transcript helpers and uses `json!` throughout the six existing local-media App controls. The addition uses the App crate's existing `serde_json = "1"` dependency. It introduces no new dependency, helper, adapter or production owner.

The edited call at `local_media_file_url_tests.rs:200–202` passes `1_000.0` into `SessionRuntime::set_window_focused`. The actual method declares `now_ms: f64`, stores the focused timestamp as `Option<f64>`, and passes that same value into the existing viewed-state owner. Its production App caller also passes an `f64` timestamp. The corrected fixture literal therefore matches the existing API and retains the same intended timestamp value; no alternate focus API or conversion layer was added.

For this adjacent definition, a contents request using the candidate tree as a commit ref returned 404. The review then fetched the file at exact starting commit `871b033c5b9aabbe322cc7322af92648939db9b5` and obtained blob `bf7e3533d2691aaf204888fcdb64e1f548716c10`. An independent read of candidate Git tree `9892…` confirms that exact blob at `source/host/src/extensions/transcript/session_runtime.rs`. Thus this is exact candidate source context, not an unverified parent assumption or another working tree.

The full local-media fixture and its existing six controls were read. `LocalMediaAppFixture::open` continues to compose the existing `ProductionSessionWorkers`, `TranscriptManager`, shared `idle_runner_owners`, roster and private production sink. `main.rs:18073–18086` places the test modules under `#[cfg(test)]`. These R2 code edits are therefore scoped to the existing App test surface. This static consistency check is not a compilation result.

### 2. The ordering clarification accurately describes the current source chain

`projects/openclaw-rust/docs/specs/final-assistant-media-custody.md:141–143` now distinguishes App publication from existing DB listeners. This matches the actual source order:

1. `SandAgentDb::append_transcript_entries_and_release_media` attempts COMMIT and verifies autocommit. Every unknown branch returns before its successful-publication tail.
2. On confirmed success, `agent_db.rs:5083–5091` drops the connection guard, publishes inserted entries, notifies the session-admission metadata listeners, and only then returns to Session/Runtime/App.
3. The Session media facade preserves the nested typed owner result; `ProductionTranscriptRuntime::append_generated_send_message_entry_with_media_custody` returns the generated entry only after that facade succeeds.
4. The App still holds the same pending entry guard while that call is active. `ProductionSendMessageSink::try_pending_media` uses `try_lock`; same-ID callback reentry observes a busy entry and returns typed unknown before materialization, capture or another append. The pending-map guard itself does not span the owner call.
5. `main.rs:1211–1224` validates the returned entry identity, commits local custody and clears the terminal entry's work/cache/owned-media fields, then drops the entry guard. Channel dispatch, App transcript publication, ACK fulfillment and activity update follow at `main.rs:1225–1273`.

The amended wording therefore avoids claiming that App-local terminalization precedes DB-level listeners. It also accurately states the held-entry protection during that earlier listener interval. No callback move, second publication mechanism, new lock or new state owner is introduced by R2.

This reasoning concerns the explicit media `send_message` chain. It does not generalize the ordering guarantee to unrelated App operations or prove runtime reentry behavior through execution.

### 3. Canonical references resolve without upgrading evidence status

The focused ledger edit replaces two nonexistent historical aliases with actual published evidence paths. Independent candidate-tree inspection confirms:

| Canonical path under `projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/` | Candidate blob |
| --- | --- |
| `fixed-u-contract.md` | `b143ea8d36f84c02793263bbadf39a2e138619bf` |
| `current-owner-audit.md` | `01b724729a97f33ad1ef63b937360022ee5e48b9` |

Both are regular `100644` blobs. Neither obsolete alias (`media-settlement-fixed-u-contract.md`, `media-settlement-unknown-current-audit.md`) exists in that exact tree. No evidence-report body was opened.

`parity-ledger.json:36788–36789` now points to these actual objects. The four RTM edits at `requirements-traceability-matrix.json:1396,1553,2916,3096` consistently use `fixed-u-contract.md` for `OCRM-R23`, `OCRM-R24`, `OCRM-AC23` and `OCRM-AC24`. The focus patch changes only those reference strings in the machine records.

The affected ledger row remains `implementation=implementing`, `verification=not-run`, `acceptance=pending`, with both full-equivalence and exact-H-evidence blockers. Its existing owner remains `SandAgentDb`, with Session/Transcript propagation, subordinate local cleanup and App/Runner consumers. The adjacent RTM rows remain in progress. R2 repairs reachability of the declared source references; it does not create a second ledger or convert the existence of reports into verified behavior.

### 4. Build/test wiring and performance impact remain proportionate

Both Cargo manifests were read completely. Host `test-support` remains default-off; App enables it only in its dev-dependency on the same Host package. The workflow's existing shipping check and App test build retain separate actual Host-library feature observations by manifest/source path and `profile.test == false`. The new App tests remain nested under the existing channel-host-lane test module. The focused imports and numeric literal do not alter these feature declarations or test selection.

The inherited full-scope capture deltas were read as source. They extend the existing native/portable captures and App workflow rather than introducing another test runner or acceptance store. The explicit test names, qualified prefixes and separate native/App scope descriptions remain present. Their presence demonstrates selection wiring only; actual compiler artifacts, test counts and results have not been obtained by this review.

Performance is applicable to the review lens, but the R2 focus changes no shipping algorithm, lock acquisition, allocation, I/O, notification count, cache lifetime, retry path or database command. Its executable edits are an existing test module's imports and literal type. The spec/reference edits create no runtime workload. The adjacent production code still uses the single pending-media index, brief map lookup and per-entry nonblocking acquisition; its collecting cache reuses exact source resolutions. Required compact terminal retention is explicit MSU-v1 behavior, not a basis for an unrelated eviction/refactoring blocker in this narrow fix. No benchmark, resource budget or whole-project performance acceptance is inferred.

## Scope and actual read depths

All paths below are the exact immutable identities listed in `media-settlement-r2-manifest.json`; the focus files' local SHA-256 values were independently checked against that manifest. “Context” means source/hunk/interface inspection sufficient for this focused round, not a claim of a fresh full-file deep read or inherited acceptance.

| Frozen path | R2 depth and purpose |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | Context: complete changed hunks; shipping/test feature observation, input binding and App/native test selections. |
| `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md` | **Focus, complete document and complete fix delta**; revised ordering compared with owner/App source. |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | Complete applicable MSU-v1 contract, including state, typed boundaries, test support, recovery, idle assertions and exit limitations. |
| `projects/openclaw-rust/parity-ledger.json` | **Focus**, complete changed row and fix delta; adjacent inherited risk correction and exact reference identities. Unrelated rows were not deep-read. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | **Focus**, all four edits and affected row/reference/status context. Unrelated requirements were not deep-read. |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | Context: complete changed hunks, named native DB/local custody/transaction selection and evidence-scope declarations. |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | Context: complete changed hunks, retained source-oracle/SendMessage selections and scope/count declarations. |
| `source/host/Cargo.toml` | Complete manifest; default-off feature and dependency placement. |
| `source/host/app/Cargo.toml` | Complete manifest; existing JSON dependency and test-only Host feature enablement. |
| `source/host/app/src/main.rs` | Deep affected state/locking/rollback/resolve/abort/send chain, principally lines 700–910 and 1000–1295; test-module inclusion around 18073–18086. Unrelated large App sections were not deep-read. |
| `source/host/app/src/tests/channel_host_lane_tests.rs` | Complete inherited changed hunks; fixture composition around 410–460; test nesting and reusable Responses/child/wire fixture surfaces. |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | **Focus, complete file and complete fix delta**; imports, fixture construction, observer, idle proof, cleanup/reopen and all six controls. |
| `source/host/app/src/tests/media_settlement_unknown_tests.rs` | Context: imports, fixture/observation and test-symbol inventory; deep collecting/cache/multi-image/terminal case around 489–585. Other inherited cases were not represented as freshly full-read or executed. |
| `source/host/src/attachment_paths.rs` | Deep local custody state/Drop API around 1–128; inherited change context and separation from durable truth. |
| `source/host/src/extensions/session/agent_db.rs` | Deep exact fault lease/owner state around 875–1060, command claim and full media transaction around 4710–5135, command/authorizer helper around 6237–6350; other inherited private test bodies not fully re-read. |
| `source/host/src/extensions/session/production.rs` | Complete inherited changed hunks; deep typed media facade around 4060–4100 and recovery `track` adaptation. |
| `source/host/src/extensions/transcript/production_runtime.rs` | Complete inherited changed hunks; imports and full typed media append function around 1790–1840. |
| `source/host/src/runner/tools/send_message_tool.rs` | Complete inherited changed hunk and adjacent message-build/encode path around 388–432. |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | Context: media/retained oracle symbol and selection inventory; actual-owner/admission positive around 1884–1910. Entire parser/oracle suite was not executed or claimed freshly deep-read. |
| `source/host/tests/send_message_tool_contract.rs` | Context: all relevant symbols, complete prepared-custody refusal helper and two unknown/known controls around 569–691. |

Additional immutable authority/context reads: OCRM `SOURCE_OF_TRUTH.md` blob `3748d734b10c9ed5b2d948467fbaa9483f1dd306` (current authority/constraints and evidence-state portions); main migration spec blob `69c9667af2328e20a1fba09ebf80c5aab4d5c336` (relevant requirements, existing-owner architecture, nonfunctional constraints, §12 evidence contract and Revision 2 §17); module map blob `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3` (ownership/navigation context); current full-fusion entry blob `5ff7894d245588ed539ab075b09215a2f1a82716` (complete); and exact candidate `session_runtime.rs` blob `bf7e3533d2691aaf204888fcdb64e1f548716c10` (complete). These canonical documents do not substitute for current production source or independent execution.

The later live publication-parent advance reported during this review does not change the reviewed tree or the 20-path subject. No claim about that separate integration result is made here.

## Terminal disposition

**APPROVED — focused structure/reuse/performance source review converged with zero findings.** The two fixture edits, ordering clarification and six corrected metadata reference occurrences are consistent with their existing owners and exact candidate objects. Current-candidate compilation, tests, feature observations and all broader product/evidence gates remain separate and unproven by this report.
