# MCP official bridge retirement — independent R2 general/security/lifetime review

Verdict: **APPROVED — SOURCE ONLY.** No blocking or non-blocking defect finding is supported by this review of the complete frozen five-path scope. The supplemental R2 case independently exercises the final pre-write lifetime admission check; the scope post-await check alone cannot satisfy its oracle. This is a source verdict, not an executed test result, an Actions approval, a real-provider claim, or global OCRM acceptance.

## Identity, independence, and actual runner configuration

- Repository verified by `git rev-parse --show-toplevel`: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Frozen base: `f82ce24474131cc5fb92b29329b77ce86f58dac5`.
- Reviewed R2 candidate tree: `33d8b4c55277806d465081db13891cbca230076e`.
- Prior candidate tree used only for the immutable source delta: `6f74362fb6f0ac0b37e66cc2c57c68614a65606b`.
- Fixed upstream U remains `1440bb1771372334bea23d7642bb52deb7db1845`. This review does not create a new upstream-read or parity claim.
- Reviewer: `/root/mcp_official_security_r2`, fresh independent GENERAL/security/lifetime lens. No MCP peer/prior review report, root adjudication, performer handoff, or scratch assessment was used. Historical status statements embedded in required project authority documents were not used as evidence of this candidate's correctness.
- Actual runner setting, confirmed by the launching parent from its successful `collaboration.spawn_agent` call: `model="gpt-6-astra"`, `reasoning_effort="xhigh"`, `fork_turns="none"`. This is runner-supplied launch configuration, not independent model-runtime introspection. No prompt-only override, CLI process, global configuration change, or fallback was used. The scope warrants xhigh because it combines asynchronous lifetime, credential acquisition, reentrant callbacks, and a scheduling-sensitive write-admission oracle.
- Current root `AGENTS.md` was read completely. Its read-only content hash is `e4682748087759b2d4bb2b3a47916b6148e8a6f2`, identical to the candidate's AGENTS blob. The only nested AGENTS found in the candidate is under the unrelated Mahayana TUI subtree.

The R1-to-R2 diff changes only the spec (eight added lines and one replaced line) and the test (one import plus the 68-line supplemental case). All three product blobs are identical to the prior tree. Against the frozen base, the complete scope is five paths, 683 insertions and 15 deletions; the test changes its imports and appends the official fixture/cases without removing prior test bodies.

During review the parent supplied an integration identity, `9cfb4ca1efd46230959698b44245ed91b01a35c9`, based on `bb5a7f6098f952df6d388a672c4fb01045bc645c`. Direct `git diff` restricted to the five paths is empty and `git ls-tree` shows the same five blobs. This carries the five-path source verdict to those identical files; it does not approve the unrelated incoming plugin/media work.

## Five-path dispositions

All line references below refer to the immutable R2 candidate unless another tree is named.

| Path | Exact candidate blob | Read and disposition |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/mcp-account-listing-retirement.md` | `b5bf702509df7923163fb54ba87599bc3dfd338a` | Full 1–572 read. Retain. The R2 MO-07 addition requires an independent late-admission oracle and preserves earlier contracts, default-composition cases, admitted outcomes, and explicit acceptance limits. |
| `source/electron-main/mcp/mcp-runtime.ts` | `306336b67122cf6af3e5235dd70fe826c0ec864c` | Full 1–214 deep read. Retain. One existing attempt identity owns credential/callback admission; reset/dispose detach before cleanup; official and legacy completion paths share the reentrant fence. |
| `source/electron-main/adapters/mcp-oauth.ts` | `90dcae53b6f1d26c89ce231e9e067e80799ee1c7` | Full 1–505 deep read, including default/root/fallback composition and routed facade callers. Retain. Official OAuth now uses the runtime's captured credential and completion ports. |
| `source/electron-main/mcp/official-mcp-service.ts` | `a27e63e6abc6c3555a802ed9222e8d53ae876bf6` | Full 1–380 deep read. Retain. The irreversible disposed latch is checked after scope I/O and immediately before a new write invocation, while already-invoked writes keep their actual outcome. |
| `source/electron-main/mcp/mcp-runtime.contract.test.ts` | `2a56f1e1ffb20ca3c65e5aaf57e3013dcbf4f427` | Full 1–2296 read. Retain. All earlier MC/MR/MB/MD and uninstall bodies remain; fourteen official default-composition cases are retained, plus the new independent pre-write case. No source execution credited. |

## R2 oracle: actual causality and falsifiable sensitivity

The new case is at `mcp-runtime.contract.test.ts:2191–2257`. It uses two actual `OfficialMcpService` instances and controls only their scope/read/write/fetch I/O. It does not replace `scope`, `mutate`, `requireActive`, or `dispose`. `deferred()` at lines 122–130 constructs an ordinary native `Promise`; the held `getScope` function at lines 2202–2213 is deliberately non-async and returns that exact promise.

The old install first reads `stored` as `{}`. That read arms `holdFinalScope` at lines 2215–2218, so the next scope query is the final `await this.scope()` immediately preceding the write in `official-mcp-service.ts:69`. The earlier entry and queue-admission scope queries already completed. The same-account replacement then installs its distinct provider token and completes its actual write. The positive assertions at test lines 2242–2245 require exactly one replacement write, the expected replacement token, and capture the exact stored string before releasing the old scope query.

On release, the causal order is:

1. The test's `finalScope.promise.then(...)` was registered inside `getScope` before the promise returned to the `scope()` await. Its reaction therefore precedes the awaiting scope continuation on fulfillment.
2. That first reaction queues a separate `queueMicrotask` retirement callback. The already-queued scope continuation is ahead of that newly queued callback.
3. The scope continuation resumes at service line 38 while `disposed` is still false, executes the post-await guard, validates the scope, and fulfills the scope method's own promise. Fulfillment makes the mutation continuation at line 69 runnable, placing it behind the already-queued retirement callback.
4. The queued callback calls the real `old.dispose()` and resolves `retirementObserved` at test lines 2208–2210. The mutation continuation then resumes with the same valid scope, reaches line 72, and is rejected by the final `requireActive` before line 73 invokes the write port.

This does not depend on elapsed delays, arbitrary counts of awaited microtasks, replacement-account hash inequality, or a test-authored admission guard. The selected build configuration retains native async semantics: `desktop/scripts/build-electron-runtime.mjs:19–23` targets Node 24, and the TypeScript contract configuration targets ES2024 (`desktop/tsconfig.grok-electron.json:2–5`). No compiled output was generated or executed to make this judgment.

The counterfactual is specific and falsifiable: remove only `this.requireActive()` at service line 72 while retaining the post-await check at line 38. Under the same schedule, line 38 has already succeeded before disposal. The mutation's scope equality succeeds, and the old write port at test lines 2220–2223 records and stores the stale image. `old.install` resolves rather than rejecting. Thus the rejection assertion at 2248, zero-old-writes assertion at 2249, and exact-byte assertion at 2250 cannot all pass. The final scope guard alone cannot satisfy this case. This sensitivity conclusion is source reasoning; no mutation experiment was run.

The oracle also rules out a vacuous result: the replacement service successfully writes through the same account scope, its stored token is checked, and the complete string is retained for equality. The old write port has no retirement logic or selective refusal that could conceal a missing production guard. An unexpected network invocation rejects through the explicit fetch control, and this install path has no OAuth or poll timer.

Cleanup releases the held promise idempotently, observes the old operation via `Promise.allSettled`, and disposes both instances in `finally` (2251–2255). The old operation has a rejection observer installed before release. The retirement callback has no pending controllers or authorizations to launch external cleanup. This supplemental case introduces no process environment, global timer, or prototype mutation.

## Retained product contract and production composition

**Credential lifetime (MO-01/MO-05).** The official getter at `adapters/mcp-oauth.ts:251–255` calls the runtime option with the original backend URL and retains the official unauthenticated message. Runtime lines 106–112 enforce current-attempt checks at entry, after obtaining the auth service, and after obtaining the token. Reset first clears `current` at 194–199; permanent disposal sets the permanent latch and clears `current` at 205–211. Facade disposal invokes `official.dispose()` synchronously before the legacy manager's disposal (`desktop-mcp-manager.ts:279`). The official service's best-effort cancellation therefore encounters the same retired getter; the change does not cache a credential or retry through a replacement.

The real OAuth request consumer (`official-mcp-oauth.ts:24–30`) obtains the token before dispatching to its pinned origin. The patch preserves that origin, request shape, and existing error behavior. As MO-04 expressly states, there is still an async gap after a token has been returned and before the request function dispatches. This review does not reinterpret the credential fence as a new final HTTP-admission fence or as successful remote cancellation.

**Completion lifetime (MO-02/MO-06).** `forwardAuthCompletion` at runtime lines 75–79 checks the exact attempt before broadcast and again before refresh. The official option uses an empty argument tuple at line 114; the legacy observer uses `[completion]` at 153–155. The default factory calls the option with the exact existing official payload at adapter line 256. The fallback runtime resolver uses a rest tuple at adapter line 279, preserving the distinction between zero arguments and an explicit `undefined`; the actual root provider retains its existing direct callback at 164. No second epoch, callback registry, or account authority was added.

The actual late producers are visible in `official-mcp-service.ts`: disconnect cleanup at 95–113, poll/ack success at 352–364, and poll failure/cancel at 367–370. Their callbacks reach the captured runtime fence through the production OAuth port. A synchronous broadcast-triggered reset is handled by the second check. The production graph's existing callback wrapper at `main-production-services.ts:754–756` remains unchanged; the zero-argument contract is preserved at the existing runtime/context invocation boundary.

**Write lifetime and admitted outcomes (MO-03/MO-04/MO-07).** The service's existing disposed latch remains irreversible (375–378). Scope now checks it both before and after awaited account I/O (35–40). `mutate` retains its per-instance queue, initial and queued scope equality checks, expected revision check, state mutation, and final scope equality (61–69). The final lifetime check has no await between it and invoking the write port (72–73). Every service mutation still goes through that owner. An already-invoked `ports.write` is awaited without a later lifetime override; no rollback, cancellation synthesis, or retry was introduced.

The production vault maps one scope to `official-mcp/<scope>.json` and writes `OFFICIAL_MCP_INSTALLS` through the native store (`official-mcp-vault.ts:11–24`). The native store's `upsert` and persistence paths are unchanged. This explains the relevant risk as a retired same-account image overwrite, not proof of a remote cross-account authorization bypass. The facade may subsequently refuse its `servers()` read after an admitted install or disconnect; the spec and held-write controls preserve that distinction.

The fourteen retained production cases at test lines 1937–2189 and 2259–2296 still call `createProductionMcpOAuthPorts()` and `createProductionMcpOAuthAdapter(ports).create(context)` at 1918–1919, with no manager/runtime resolver replacement. They exercise both credential awaits, cancellation from real reset/disposal, current credential errors and null handling, poll failure and ready results, held acknowledgment, disconnect success/failure, reentrant broadcast reset, held final scope, and already-admitted write success/failure. The fixture controls auth/network/native store I/O and owned poll timers. Disposal drains finite released continuations and restores mocks and both environment variables in `finally` (1768–1784). The supplemental direct-service case complements these integration controls; it does not replace them.

## Integration and read limits

The following adjacent source was read for concrete caller/consumer contracts, without claiming full-file coverage where ranges are listed:

- `source/electron-main/mcp/official-mcp-oauth.ts`: full 1–79.
- `source/electron-main/mcp/official-mcp-vault.ts`: full 1–26.
- `source/electron-main/mcp/desktop-mcp-manager.ts`: 30–282, including actual construction, install/auth/disconnect consumers, discovery warming, and disposal; imports 1–29 were only searched for relevant identities.
- `source/electron-main/secrets/user-secrets-store.ts`: 66–192, including reveal, upsert, slot/cache ownership, and atomic file replacement; preceding utilities were not deep-read.
- `source/electron-main/coordinator/account-transition-cleanup.ts`: full 1–74.
- `source/electron-main/coordinator/production-root-provider.ts`: 48–83; account transition reset binding at 76.
- `source/electron-main/main-production-services.ts`: 298–325 and 738–785, plus symbol search results for MCP ports; not a whole-file review.
- `desktop/scripts/build-electron-runtime.mjs`: full 1–111. The complete MCP test file remains in the plugin contracts build selection at 98.
- `desktop/tsconfig.grok-electron.json`: full 1–73.
- Candidate workflow selectors were searched at exact tree identity. Integrated `.github/workflows/rust-desktop-runtime.yml`: 4738–4785 read, including the actual build command and full MCP test target at 4762–4778. The file is selected without a new name filter. This is selection evidence only, not a workflow run or a full-workflow review.
- Incoming `host-plugin-skills-loader.ts`: exact f82-to-bb5 diff (type-only boundary adjustment) and integrated ranges 90–136, 174–210, and 236–310. Its factory returns a deferred loader at 198; the MCP official fixture does not invoke `loadHostPluginSkills`. No incoming SDK/plugin acceptance is claimed.

Governance reads at the candidate: the complete applicable MCP spec (572 lines); SOURCE_OF_TRUTH lines 1–110, with the operative identity/precedence/constraints in 1–84; main OCRM spec §17 at 671–720; current P0 task lines 1–150 plus focused MCP/official identity searches; module map lines 1–58 plus the matched M03/M16/M17/M18 and ownership-boundary rows. Initial oversized tool output for some authority reads was truncated; the full MCP spec and AGENTS and the operative authority ranges were then explicitly reread in bounded chunks. No full SOURCE_OF_TRUTH/current-task/module-map read is claimed.

All product/spec/test source reasoning used immutable `git show`, `git diff`, `git ls-tree`, and bounded text reads. There was no build, test, lint, generator, schema, harness, benchmark, candidate execution, Actions dispatch, or htch execution. No product/spec/test/workflow/ledger/GitHub mutation occurred. This report is the sole authored file.

The remaining required evidence is the permitted exact-published-H Actions build/contract execution, with actual positive test outcomes and subject identity, followed by the separately applicable wider acceptance. Authored tests, static selection, identical blobs, and this source approval do not supply those results. Real provider/account behavior, durable cross-process coordination, remaining Rust policy migration, scoped G01/G02 authority, and global OCRM flags remain outside this approval.
