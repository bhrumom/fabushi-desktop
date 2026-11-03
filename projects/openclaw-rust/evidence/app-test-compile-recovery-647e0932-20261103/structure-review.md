# LFA App fixture compile recovery — structure, reuse and performance review

**Verdict: SOURCE-ONLY APPROVE.** No falsifiable structure, reuse or performance defect was found in the frozen two-hunk repair. This verdict approves the causal source repair within the declared one-path scope. It is not a compilation, test-execution, runtime, platform or acceptance verdict.

## Reviewer and immutable authority

- Lens: fresh, blind, focused structure/reuse/performance recovery review.
- Actual launch setting supplied by the parent: `gpt-6.1-sol`, effort `xhigh`. No model or effort override, additional worker, external AI process or fallback was used in this review. Scope reason: diagnosis of an observed App test-harness compilation failure while preserving incoming stateful media custody and owner integration.
- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop` (`bhrumom/fabushi-desktop`).
- Frozen base commit: `871b033c5b9aabbe322cc7322af92648939db9b5`; base tree: `f641366eea64364f4d029487d04276c4c567790a`.
- Candidate snapshot: **tree**, not a commit, `7dec21739a84f7a238a60ac420a72a3d8e859928`.
- Sole changed path: `source/host/app/src/tests/local_media_file_url_tests.rs`, mode `100644`, base blob `60e6be8a61d446706ac2188366be9bff1f08cb71`, candidate blob `57a0a3393f1bb9560eb036687688656b1f3fe6c8`.
- Reviewed `manifest.json` and the entire `complete-scope.patch` in this review directory. Independently read the exact tree-to-tree raw delta and full-index binary-safe Git diff; both contain only this path and the two declared hunks.

The base commit object was not present locally. An exact GitHub Git-commit API read of `871b033c5b9aabbe322cc7322af92648939db9b5` confirmed its tree is `f641366eea64364f4d029487d04276c4c567790a` and its parent is `71b039ae860c45e982dfbc8b1f401143d49d2454`. Source review used the locally available exact trees/blobs with `GIT_NO_LAZY_FETCH=1`; missing unchanged source blobs were acquired by exact blob SHA through the GitHub API. No mutable `HEAD` or branch name was used as source authority.

## Complete scope and adjacent reads

Read the complete root `AGENTS.md`, including the independent-review, model/effort, frozen-snapshot, existing-owner and Actions-only verification rules. The candidate root policy identity is `e4682748087759b2d4bb2b3a47916b6148e8a6f2`. Exact tree enumeration found no nested `AGENTS.md` governing the changed App test path. Read the complete applicable spec `projects/openclaw-rust/docs/specs/local-media-file-url-admission.md`, candidate blob `9249e4a93f353247e97c7ef47339382932451d72`, including LFA-01–06, shared-owner boundaries, actual App consumer oracles and the retained execution/acceptance limits.

Read the full candidate test blob, all 783 lines, and the full base source while establishing incoming preservation. Adjacent exact identities and inspected contracts were:

| Source | Exact unchanged blob | Inspected integration |
| --- | --- | --- |
| `source/host/app/src/main.rs` | `7ec9bd8a22fd595f63cb3f4b632673e298294dfa` | Crate imports; `#[cfg(test)] mod tests` and sibling path-module declarations at lines 18074–18105; parent test imports at 18107–18132; shared production resolver interface; `ProductionSendMessageSink`, per-ID `Arc<Mutex<PendingOutgoingMedia>>`, phase/commit/rollback and resolve/abort paths at 708–899 and 1004–1093; direct AgentManagement resolver at 1299–1319. Full blob was acquired, with these relevant portions inspected; this was not a full-main-file product review. |
| `source/host/app/Cargo.toml` | `a2c58167a82657194b79aaf3e1e0d55577b2221c` | Complete manifest, including existing ordinary `serde_json = "1"` dependency, binary path and existing test-support dependency. |
| `source/host/src/extensions/transcript/session_runtime.rs` | `bf7e3533d2691aaf204888fcdb64e1f548716c10` | Focus accessors and `set_window_focused` at 286–344, including `now_ms: f64`, state storage and forwarding to the current Session owner. |
| `source/host/app/src/tests/channel_host_lane_tests.rs` | `f6139af6bde9829f7e9147e6a2028ec77680bf42` | Existing explicit `serde_json::{Value, json}` precedent at line 46; `FOCUS_AT_MS: f64` at 53; `IdleRunnerOwners` and disposal at 150–167; complete `idle_runner_owners` body at 224–386; existing focused-state fixture consumer around 449. |

Read the original H23 `automation-app/receipt.json` identity/command/result fields, raw `test-build.stdout.log` diagnostic messages, and complete `APP-H23-19-COMPILER-DIAGNOSTICS.txt`. No other reviewer report, runner arbitration/decision or decision-before-fix material was consulted. Separate cron changes were outside this review.

## Causal repair assessment

The retained H23 subject is commit `23c2a567d874509f56353b8cae737cdf1437309d`, tree `b10485b13c15cd1cf012a5f51acaf6927aeb322a`, GitHub Actions run `41576693105`, attempt `1`, job `125721731498` (`sqlite-native-path`, Linux). Its receipt records production `cargo check --locked --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host` exit `0`, followed by test-harness `cargo test --locked --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host --no-run --message-format=json` exit `101`, `cases: []`, and result `failed`. The normalized compiler text contains 12 missing-`json` macro diagnostics, six missing-`Value` type diagnostics and one E0308 integer-versus-`f64` mismatch. These are preserved failure facts, not executed App-case results.

1. **Explicit names in the actual module scope.** Candidate line 3 adds `use serde_json::{json, Value};` directly in `tests::local_media_file_url_tests`. The module's existing `use super::super::*` imports crate-root names, while the crate root uses qualified `serde_json` names rather than supplying these bare bindings. A sibling module's private import does not supply this module's names. The new import therefore directly supplies the macro used by the unchanged JSON fixture calls and the type used by the unchanged delegate/sink/helpers. The dependency is already declared in the owning App manifest; no crate, feature, dependency owner, wrapper or replacement implementation is added. The adjacent channel fixture uses the same existing dependency and explicit imports.

2. **Exact type at the existing interface.** Candidate lines 200–202 retain the actual `session_runtime().set_window_focused(&deps.session_workers, true, ...)` call and change only `1_000` to `1_000.0`. The exact SessionRuntime method takes `now_ms: f64`, stores that value in focus state and may pass it to `mark_agent_viewed`. The new literal supplies the required type with the same intended numeric timestamp, 1000 milliseconds, which is exactly representable as `f64`. It does not change the interface, clock owner, focused-state boolean, Session owner or fixture ordering. This is a source-level repair of the compiler's specific E0308 cause, not a changed time-policy oracle.

These source facts support a causal repair of the reported diagnostic classes. They do **not** establish that the candidate test harness links or that no further diagnostic will appear after those classes are removed.

## Preservation, reuse and cost

The entire delta leaves all six App test names, their attributes, inputs, loops, assertions and real-owner calls unchanged. It adds no `ignore`, cfg exclusion, shortened selector, reduced required-positive count or alternate capture path. Since the complete tree delta contains only the import and literal edit, workflow/capture selectors, strict gates and all other production and test files are identical to the frozen base. Their execution results remain outside this source verdict.

The incoming `71b039ae860c45e982dfbc8b1f401143d49d2454` test source was also read through an exact-ref GitHub contents request and compared byte-for-byte with base blob `60e6be8a61d446706ac2188366be9bff1f08cb71`: the complete 36,289-character texts were equal. Thus the recovery starts from, and preserves, that incoming fixture contract except for the two declared repairs. In particular:

- Candidate observation lines 122–139 still call the real production resolver first, lock the real custody index, clone the per-ID `Arc`, lock its entry and observe the actual admitted bytes. No synthetic resolver substitutes for the shipping sink.
- Candidate `assert_custody_idle` lines 280–298 still requires durable work identity and paths to be released. For each retained entry it clones the `Arc`, requires a successful `try_lock`, requires `Committed` or `Aborted`, rejects pending owner settlement, and requires no owned media, no work identity and no resolved sources. It neither requires an empty map nor permits an active retained entry. This agrees with the unchanged shipping map/phase contract, which retains terminal identities to prevent same-ID recreation.
- Fixture construction still obtains the actual Session/Transcript/ForeverBox dependencies through existing `idle_runner_owners`; observation delegates sink operations; reopen/disposal still use those actual owners. The repair introduces no production media owner, state store, parser/converter, reader, scheduler, permission mechanism or duplicate test harness.

The new import is a compile-time binding to an already-present dependency. The literal edit remains inside the module excluded by `#[cfg(test)]` from the production binary and adds no loop, allocation, I/O, synchronization, worker, dependency/build reach or shipping code path. No new shipping runtime or performance mechanism is present to benchmark. This is a proportional source-cost assessment, not a measured performance result.

## Limits and required next evidence

No local or htch compilation, test, lint, formatter, parser, generator, schema check, validator, harness, benchmark or acceptance command was run. Operations were limited to source/policy/evidence reads, exact Git/API identity and diff acquisition, and writing this report. No reviewed source, index, branch, receipt, ledger or existing failure artifact was changed.

Fresh exact-published-H Actions evidence is still required to establish App test-harness build success and the selected App behavioral results. H23's `cases: []` must remain a failed compile-stage record. Neither its production-check success nor this source-only approval supplies candidate App-case passes, Windows/macOS acceptance, live provider/channel acceptance, packaged acceptance, unknown-write-outcome proof or global migration completion.
