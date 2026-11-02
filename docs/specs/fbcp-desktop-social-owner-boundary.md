# Desktop Social ownership and retained recovery

Status: active, batch verification pending; no independent product acceptance
Spec ID: NMR-SOCIAL-OWNER
Parent: `docs/specs/fbcp-native-message-reconciliation.md` / FBCP-NMR-001
Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Context and scope

Current source inspected at `106bb95ba97ef7722e9523285783824393039bc0`
already selects Session/Transcript for Production Electron Human conversations.
The older generic Social provider must not become another desktop message
writer, and retained public conversation data must survive ordinary Agent
recovery. The existing FeatureHost boundary suite has seven cases for standalone
messaging and automation retention but no direct Social intent cases. This batch
adds those missing behavioral controls to that suite, traces the existing Runtime
and HTTP contracts, and leaves data and current owners intact.

The named IDs below do not reuse NMR-48/49 or NMR-AC-37/39: concurrent forwarding
and reply batches own those numeric IDs. Historical source comments that used
them for Social recovery are corrected to these names. This does not renumber
or replace the existing forwarding requirements.

## Required behavior and actual owners

- **NMR-SOCIAL-OWNER-01 — Shipping selection.** Production Electron AppHost
  construction selects the existing Session host policy. Ordinary and Codex
  provider registries omit generic Social while preserving Agent, model,
  MiniApp, approval, tooling and automation ownership. Explicit Test and other
  surface configurations retain their existing selection rules.
- **NMR-SOCIAL-OWNER-02 — Earliest public command refusal.**
  `FeatureHostController::execute` rejects Production Electron
  `ConversationOpen` and `ChatSend` when the trimmed conversation ID starts
  with `mahayana:contact:`. The empty suffix is also a refused public intent.
  The exact contract error directs the caller to Session/Transcript, before
  authorization, runtime access, state locks, Agent memory/workflow I/O or event
  publication. Near-prefix and unrelated namespaces retain existing dispatch.
- **NMR-SOCIAL-OWNER-03 — Reversible retained recovery.** The existing Runtime
  recovery policy preserves records belonging to a valid nonempty Social contact
  ID before active-turn, run, WaitingUser or handoff rewrites. Ordinary Agent
  recovery continues; invalid/unknown IDs retain existing errors. This narrower
  persisted-record recognition does not weaken the public-intent refusal.
  No migration deletes or rewrites retained Social data.
- **NMR-SOCIAL-OWNER-04 — Service boundary.** The desktop product's existing
  session-owned platform client rejects generic Social message and
  message-resource paths against the actual normalized base URL/origin/path
  before credentials or transport. Its desktop policy disables redirects.
  Contacts and the ordinary non-desktop client remain available. Human message
  state, native permissions, resources and synchronization stay in the existing
  Session/native_messaging and backend owners.

## Verification and acceptance

All executable checks run only in GitHub Actions. No local build, test, lint or
schema validator is authorized for this batch. The existing
`native-service-contracts.yml` remains the execution owner. Its macOS shipping
job retains every original platform requirement; the focused Ubuntu job below
also runs these same platform-independent behavioral harnesses.

| Acceptance ID | Direct oracle and test consumer |
| --- | --- |
| NMR-SOCIAL-AC-01 | The existing `desktop_messaging_boundary_tests` suite passes all ten cases. The three added cases below invoke real public dispatch and preserve the original seven cases. |
| NMR-SOCIAL-AC-02 | All nine `retained_social_recovery_tests` pass against real SQLite stores, including ordinary Agent recovery and retained record controls. All six `session_owned_platform_` HTTP boundary cases pass. Current AppHost composition and native handoff checks remain required. |

Added FeatureHost cases:

1. `desktop_social_intents_refuse_before_auth_runtime_context_io_and_events`:
   absent-runtime Production Electron fixture; ordinary, empty-suffix and
   whitespace-wrapped Social IDs; exact refusal; unchanged events; no workspace.
2. `desktop_social_refusal_does_not_lock_state_or_touch_retained_context`:
   deliberately poisoned owner state, real retained memory/workflow bytes,
   exact refusal before the lock, unchanged events and complete fixture tree.
3. `social_boundary_keeps_explicit_test_non_desktop_and_other_namespaces`:
   real in-memory Test success; Ios/Android/Wasm and unrelated namespaces still
   reach their prior absent-runtime boundary. This is a scope control, not
   evidence of live mobile or service acceptance.

The first two cases require the exact owner refusal before absent-runtime or
poisoned-state errors and require unchanged observable events and fixture bytes;
source review confirms that the guard precedes authorization. The third prevents
overbroad rejection. Fixtures own
only UUID temporary roots, use no live accounts or HTTP, and never clean user
data. The workflow requires the exact suite count so old Feature7 evidence
cannot approve this ten-case revision.

## Delivery, rollback and evidence

Extend the existing FBCP-NMR-001 ledger row, owner inventory and RTM; create no
parallel status authority. Preserve every concurrent source review, source
identity, legal record, acceptance state and unrelated requirement. Historical
H0 Windows hook timeout, subsequent exact-head Windows11 success, and the
latest reconciliation compile failure/fix retain their own SHA and run scope.
An ordinary code revert can restore a prior reviewed implementation without
data conversion; removal of a test is not rollback acceptance.

This batch does not claim authenticated production history readiness, complete
source/legal review, all Bot regressions, supported signed packages, upgrade or
rollback acceptance, or independent ACCEPT. Backend PR #137 remains read-only.
Formal publisher authorization, real package bytes and full final release gates
are mandatory and remain separate from these controlled contracts.

References: FMEP-001 §§3–4, 12–13; FBCP-001 AC-11/12/19/21/41/47/50;
`mahayana-feature-host/src/implementation.rs`,
`mahayana-feature-host/src/desktop_messaging_boundary_tests.rs`,
`mahayana-runtime/src/lib.rs`,
`mahayana-runtime/src/retained_social_recovery_tests.rs`,
`mahayana-host/src/lib.rs`, `mahayana-product/src/lib.rs`, and the existing
`mahayana-platform-client` session-owned transport policy.

### Bounded ledger-object delivery

The round23-only helper workflow and operations file may prepare an append-only
Git blob on the exact checkpoint. It permits only the existing FBCP-NMR-001 and
TDRP-UPDATER-DISCOVERY-AUTHORITY-001 identities, declared string arrays and note
suffixes. All other fields, row order, counters, source_reviews, legal and
acceptance values remain equal to the immutable input. The job's contents:write
token is broader than a blob-only permission; the reviewed source uses it only
for POST git/blobs and never creates commits, changes refs, merges or deploys.
Root must read the object and receipt, require the fresh canonical ledger blob
to equal the producer input, then integrate through force=false/expected_sha CAS.
A changed input requires regeneration. The helper is removed after integration;
its success is object transport, not schema or product qualification.


### Direct behavior execution while macOS shipping awaits a runner

At code83c9/run41524660245 and earlier106/run41523018920, macOS shipping
remained queued with zero steps. Independent source review confirms that the
three existing harnesses use explicit surface data, UUID-owned files, real
SQLite and controlled loopback HTTP, with no macOS-only test gate or actual
GUI/account requirement. Add `desktop-social-owner-behaviors` to the existing
workflow on Ubuntu24.04 using Rust1.97.1, the existing native dependencies/cache,
unchanged default production/local-only features, `--locked`, and one test thread.
Require Feature10, Runtime9 and HTTP6, each with zero failed/ignored tests.
Also run the same existing `native_desktop_account_handoff` integration target
on Linux: seven top-level cases, zero failed/ignored/measured/filtered-out cases.
Its Unix child processes construct the production AppHost library and preserve
the account, Social SQLite, credential, resource-boundary and restart oracles.
Check and record the existing openssl/mkfifo tools, bind AppHost test/library and
Cargo inputs plus the relevant Host/FeatureHost/Runtime/Product inputs, and
retain Cargo.lock unchanged. The child is the integration-test executable;
this does not build or qualify the separately packaged shipping AppHost binary.
Record the exact checkout, source/lock/spec/workflow hashes, toolchain and full
logs in an immutable artifact, including on failure. No retries or ignored cases
may turn a missing result into success. This focused execution does not replace
the macOS AppHost, native handoff, symbol/library, binary-copy or package gates;
Windows, reconciliation, direct C ABI, Electron/UI and all current jobs remain
unchanged. NMR-SOCIAL-AC-01/02 stay pending until the containing-H logs prove
their actual results. Runner queueing remains pending rather than acceptance.

The Ubuntu lane explicitly installs pkg-config, libclang-dev and libsqlite3-dev
with the existing Linux Computer libraries: bindgen/system-deps and the separate
Runtime invocation must not depend on incidental runner packages or feature
unification from another Cargo invocation. It uses two build jobs and its own
cache key under the existing cache action. Runtime sources and workspace
Cargo.toml join the existing push paths so isolated policy/feature edits cannot
miss this gate. The full exact-H logs must name all three added Feature cases.


### Observed nocapture oracle recovery

Exactf75ba3b0/run41526236521/job125554566242 executed all ten FeatureHost
cases successfully, but the separate name check failed: the expected caught
poisoning panic writes its hook between the case prefix and its final `ok`.
This is a TEST_FLAW in log interpretation. For that one case, require the exact
Rust harness case-start prefix together with the unchanged suite result of
10 passed, zero failed/ignored/measured. Keep the other two case-name/ok checks,
Cargo pipefail, serial execution, the actual poisoned Mutex and all byte/event
assertions. The complete successful suite supplies the outcome; the prefix
proves the named case participated. Do not suppress the panic, delete the test,
add retries or claim the skipped Runtime9/HTTP6 passed. Re-execute the affected
Ubuntu job at the corrected containing H and preserve this failed attempt.
