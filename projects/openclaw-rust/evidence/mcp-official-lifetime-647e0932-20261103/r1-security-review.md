# MCP-OFFICIAL-LIFETIME-v1 — independent general/security/lifetime review R1

**Verdict: APPROVED for the declared frozen source scope. No blocking or non-blocking defect finding. Execution: NOT RUN.** This is source approval only; it is not unit verification, real-provider/account acceptance, native cryptographic-vault verification, packaged/platform acceptance, full Rust migration, or global OCRM ACCEPT.

Task: `647e0932-bea4-4113-9a61-65ae3db58955`. Review round: 1. Reviewer: independent general/security/lifetime worker. Actual configured model/effort: `gpt-6-astra` / `xhigh`, as configured for this worker; no model/effort override or separate CLI was used. The shared credential bridge, callback reentry, service lifetime, asynchronous mutation queue, and write admission justify xhigh.

## Exact subject and method

- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Frozen base and starting HEAD: `f82ce24474131cc5fb92b29329b77ce86f58dac5`.
- Reviewed tree: `6f74362fb6f0ac0b37e66cc2c57c68614a65606b`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`; unchanged context, with no new upstream-coverage claim.
- Manifest: `reviews/mcp-official-r1/manifest.json` in the task scratch root.
- Complete binary-safe patch: `reviews/mcp-official-r1/complete.patch`, SHA-256 `e75ddf78d851dc731d244841c4bb958e337faafa930bea86248e9d7600027cd0`. Read all 735 patch lines. A fresh read-only `git diff --binary --find-renames` from the frozen base to the frozen tree produced the same SHA-256 as the saved patch. Git name/status enumerated exactly the five declared modified paths.

Source judgments use `git show` of the frozen tree and its exact diff, not mutable product files. The root instruction file was initially discovered in the checkout, then read at the exact tree. No performer handoff, assessment report, root decision, or peer/prior-review file was used as authority. Historical status text within the required governance documents was not used as evidence for this candidate.

Read-only Git/source searches and hashing were the only investigative operations. No build, test, lint, formatter, generator, schema check, benchmark, harness, local/htch execution, GitHub mutation, or product write was performed. This review file is the sole authored output.

## Five-path disposition and read limits

All ranges below refer to the frozen tree. The complete delta of every path was read.

| Path | Exact blob | Reading and disposition |
| --- | --- | --- |
| `projects/openclaw-rust/docs/specs/mcp-account-listing-retirement.md` | `f73275f0475f04702f4cbd44eeadcc4dcea8de0d` | Read new MO contract, lines 473–565, completely; read MR/MC/MB/MD context, lines 1–262. The MO contract names the correct existing owners, all three admission edges, admitted-outcome limits, actual-composition tests, allowed paths, Actions-only verification and rollback limits. Unchanged RC campaign/history sections 263–472 were not re-reviewed. |
| `source/electron-main/mcp/mcp-runtime.ts` | `306336b67122cf6af3e5235dd70fe826c0ec864c` | Deep read all 214 lines. New option at line 7, shared helper at 75–79, official callback at 114, legacy observer at 153–155, and unchanged token/reset/disposal mechanisms are coherent. |
| `source/electron-main/adapters/mcp-oauth.ts` | `90dcae53b6f1d26c89ce231e9e067e80799ee1c7` | Read all 505 lines, deeply reviewing default composition 239–295, root precedence 145–195 and 305–312, actual facade entry points 343–434, and cleanup 479–492. Official credential and notification producers now reach the existing runtime guard. Fallback refresh preserves argument count. Unchanged Mini App/team-popularity functions were read for boundaries, not independently accepted as features. |
| `source/electron-main/mcp/official-mcp-service.ts` | `a27e63e6abc6c3555a802ed9222e8d53ae876bf6` | Deep read all 380 lines. `requireActive`/`scope` at 32–40 and final write admission at 61–76 close the declared stale-write edges. Read every caller of mutation and the OAuth poll/disconnect/dispose producers. Unchanged protocol/tool-policy behavior is contextual, not broader approval. |
| `source/electron-main/mcp/mcp-runtime.contract.test.ts` | `dc8539423dfce5306e1ac96850c6bf098e308dc1` | Read all added fixture/cases at 1738–2227 and changed imports; inspected existing deferred/construction helpers at 116–181, setup/lifetime controls at 184–225 and 400–513, auth callback controls at 515–604, token controls at 606–670, and adjacent adapter construction at 673–680. The patch preserves all prior case bodies and assertions. The entire unchanged 1,737-line prior suite was not re-reviewed or executed. |

## Findings and reasoning

### Credential currentness and secret boundary — MO-01 / MO-05

The default official OAuth closure now calls `options.getAccessToken({ backendUrl })` at adapter lines 251–255. This is the actual option created by `createMcpRuntime`, whose checks at runtime lines 106–112 occur before account-service access, after its await before the token method is invoked, and after token resolution before return. No additional attempt registry or account identity is introduced. The official origin remains supplied by the existing OAuth adapter (`official-mcp-oauth.ts:23–29`), and the current unauthenticated message and current rejection identity survive the added forwarding await.

Runtime reset detaches `current` before `disposeAttempt` calls external cleanup (`mcp-runtime.ts:194–199`); permanent disposal sets its irreversible latch first (`205–211`). Therefore `OfficialMcpService.dispose` cannot use its best-effort OAuth cancel to obtain replacement credentials: its captured getter is already retired. No token is retained or retried to make cleanup succeed. The existing HTTP boundary still accepts a token after its getter resolves; the patch correctly does not claim a new final HTTP-admission fence or remote revocation guarantee.

The authored tests use actual default factory construction, not a replacement manager or a test-authored guarded callback. They pause the real auth-service and token I/O independently, reset and replace the manager, release replacement-subject material, and assert both the expected token-call count and zero old broker requests. Current origin/token, rejection identity, null-token message, and disposal-triggered cancel refusal have positive/negative controls.

### Callback ownership and synchronous reentry — MO-02 / MO-06

`forwardAuthCompletion` at runtime lines 75–79 checks the same attempt both before broadcast and immediately afterward before refresh. Both the official factory option and the legacy completion observer call this helper. The official path passes `[]`; legacy passes `[completion]`, so an explicit undefined completion remains distinct from an omitted argument at this boundary. The adapter fallback uses a rest tuple at line 279 to preserve that distinction too. Default root ports intentionally override the fallback at 305–307; the tests exercise the root path and directly qualify the fallback shape.

Actual official producers were inspected: disconnect completion at service lines 95–114; ready persistence/acknowledgment and failure cleanup at 352–373. An old callback is refused even when it runs after network completion, and a reset synchronously triggered by broadcast suppresses the later refresh while preserving the one already-admitted broadcast. The real production root's downstream optional-completion forwarding remains unchanged (`main-production-services.ts:754–756`); the zero-argument assertion concerns the runtime-to-context refresh invocation, not a claim that every downstream wrapper preserves `arguments.length`.

The new cases cover held poll rejection and ready response, already-entered acknowledgment, disconnect cleanup success and failure, exact current event shape, current zero-argument refresh, synchronous broadcast-triggered reset, and a usable replacement. Existing legacy observer assertions remain in place and are consistent with the shared helper.

### Mutation queue, scope awaits and durable-write admission — MO-03 / MO-04 / MO-07

The existing service `disposed` latch remains the only service-lifetime authority. The additional post-lookup check in `scope` prevents a suspended scope lookup from returning successfully after disposal. The separate check at line 72 is necessary because awaiting `scope()` also introduces a continuation boundary in `mutate`; there is no await between that final check and invocation of `ports.write` at 73. Its JSON serialization operates on the service's parsed/validated state and internal mutation results. The existing per-service promise queue, scope equality checks, expected revision checks, serialization format and vault API are retained.

Queued old operations enter another checked `scope()` before loading or writing; disposal cannot reopen the latch. A currently active same-account replacement can write independently, while an old final-scope continuation cannot subsequently enter the write port. Conversely, an already-invoked write remains awaited without a newly added post-write rejection or replay. A facade's subsequent `servers()` read may reject because the service is disposed, as explicitly allowed by MO-04.

The MO-07 fixture places its scope hold through the real vault read and the next real peek operation; it does not guess a microtask count. It lets B write the same account's vault image, then releases A and asserts zero additional A write invocations and exact B bytes. Separate held-write success/failure controls establish the other side of admission: the accepted write's state effect or exact error survives reset, with no replay or rollback claim.

### Test causality, isolation and existing behavior

The new fixture constructs `createProductionMcpOAuthPorts()`, the production adapter, runtime, desktop facade, official service, OAuth adapter and vault-to-native-store mapping. Auth/network/native-store I/O are controlled. The store path is observed and asserted as the actual scope-derived destination; the test does not replace vault selection with a fixture-defined path.

There are 14 authored cases when the loop variants are expanded. Promise gates establish each relevant ordering. Poll callbacks are explicitly driven through the fixture's owned timer handles; a single `setImmediate` drains the finite released continuation chain rather than using elapsed sleep or retry to select an outcome. Each case releases held promises in `finally`; the idempotent fixture disposer runs actual adapter disposal, checks pending poll handles, restores test mocks and both environment variables in `finally`, and removes its own temporary root. The test file has no added concurrency option, and the selected Actions command keeps ordinary per-file Node test isolation. These are source properties, not proof of an executed pass.

No changed product path adds logs, credential serialization to renderer/telemetry, permission grants, provider origins, network retries, persistence schema, or a second permission/state owner. The previously existing official service policy remains an unfinished Rust-migration responsibility and is not accepted by this review.

## Adjacent source and governance inspected

The following supporting identities and ranges supplied the concrete integration evidence. All are in the same frozen tree.

| Path | Blob | Relevant read |
| --- | --- | --- |
| `AGENTS.md` | `e4682748087759b2d4bb2b3a47916b6148e8a6f2` | Repository instruction, frozen independent review, actual model/effort, evidence limits and OCRM Actions-only execution. No applicable nested AGENTS exists on the five paths. |
| `projects/openclaw-rust/SOURCE_OF_TRUTH.md` | `3748d734b10c9ed5b2d948467fbaa9483f1dd306` | Current precedence, fixed identity and global non-acceptance at 1–53; constraints section. Historical material is not new authority. |
| `docs/specs/openclaw-rust-equivalence-migration.md` | `69c9667af2328e20a1fba09ebf80c5aab4d5c336` | Complete applicable OCRM-EXEC-02 §17, including risk depth, coherent batches, exact-H evidence and final gates. |
| `projects/openclaw-rust/module-map.md` | `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3` | M16–M18 rows 77–79 and responsibility separation at 95–97. |
| `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md` | `1f0ecfb239f840df5e2ce5237029a78d945b4b9e` | Current task/header and §17 precedence; no historic verdict promoted to this candidate. |
| `source/electron-main/mcp/official-mcp-oauth.ts` | `41449071ecdfe32906fa082870e3c763aa0c2042` | All 79 lines: origin/credential/request boundary and callback producers. |
| `source/electron-main/mcp/desktop-mcp-manager.ts` | `17af10771dfaea32dec8626f5a9e3ab4d5be6ad8` | All 282 lines: factory, real facade operations, follow-up reads and official-first disposal. |
| `source/electron-main/mcp/official-mcp-vault.ts` | `ca1a8beed13059998958030d399278c3c7e9a78d` | All 26 lines: account-derived store selection, dedicated key and native-store ports. |
| `source/electron-main/secrets/user-secrets-store.ts` | `ccec290256c6804de357850da6c8d97a2b18d755` | All 192 lines: scope-specific native storage, encryption/memory behavior, cached state and atomic persistence contract. No native crypto execution. |
| `source/electron-main/coordinator/account-transition-cleanup.ts` | `9a5d5701ceefbeb137b407a8891db06737577a8f` | All 74 lines: actual account-transition MCP reset at 70. |
| `source/electron-main/coordinator/production-root-provider.ts` | `4c5a4f73315994da7258d1d605ba4f5a4d438605` | 35–100: actual cleanup composition and reset binding at 76. |
| `source/electron-main/main-production-services.ts` | `d194b12a676ca93e992a3638e5d5dde286d03c53` | 730–780 and symbol search: context refresh callback and composition. |
| `source/shared/node/sand-data-root.ts` | `53310a97346d7e11c42596d339f57bb51f6802a1` | All 69 lines: the fixture's absolute data-root override reaches production vault construction. |
| `desktop/scripts/build-electron-runtime.mjs` | `8c5e5f1e35a3388a9032cbc72ef8ee6370119e1b` | 1–111: shared shipping Node24/esbuild configuration and full MCP contract bundle entry at 98. |
| `desktop/tsconfig.grok-electron.json` | `9bf6d4880371b60ce2becfc39f56cae6b333c34f` | All 73 lines: existing full contract inclusion at 62. |
| `.github/workflows/rust-desktop-runtime.yml` | `5abace3386897aaca3c4a256a11f90b8e3b98119` | 4759–4777: existing shipping build and complete MCP contract execution command. Other workflow jobs were not re-reviewed. |

The unchanged `cursor-token.ts` parser/account hash helper was also inspected at 1–52 to establish that the same-subject fixture keeps the same scope and replacement-subject control changes it. No new source/ledger reading state or acceptance flag is claimed.

## Final boundary

There is no concrete source defect requiring a fix round in this five-path candidate. The frozen implementation matches MO-01 through MO-07 at source level and preserves the applicable MR/MC/MB/MD mechanisms. The complete existing selected build/contract target still requires its own exact published-H GitHub Actions result. No old run, authored assertion, source verdict, mocked vault byte comparison, or controlled broker response establishes that execution or the remaining provider/platform/global acceptance gates.
