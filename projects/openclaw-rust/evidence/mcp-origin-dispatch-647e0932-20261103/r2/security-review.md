**MCP-ORIGIN-DISPATCH-v1 — R2 independent general/security/lifetime review**

**Verdict: CHANGES_REQUESTED.** The focused R2 assertion correction is sound and preserves the strict oracle. One remaining **P2 test-oracle defect** in the complete frozen scope must be fixed before the required account-lifetime controls can pass. I found no additional falsifiable product security or lifetime defect in the reviewed change.

This is a completed source review, not executable verification. `unitVerified`, global acceptance, and all eight global completion flags remain **false**.

**Immutable subject and provenance**

| Item | Identity |
|---|---|
| Repository | `bhrumom/fabushi-desktop` |
| Frozen base / starting HEAD | `4e80e161c6cc539f4c120d64f91421619a57def1` |
| Prior R1 tree | `474e8c79e8f01dbd1ba0da32fdef08ee51312c49` |
| Reviewed R2 tree | `73d5fac02a8410a2214dc0e4f337314e2a41b92c` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Complete R2 patch | 245,035 bytes; SHA-256 `f062218f89c5a9596a5d0ba6531dfcdd7efca67de9ee3a178e9aeb7b7148d1d3` |
| Focused R2 patch | 987 bytes; SHA-256 `d3abc39bce63dd5688ab1e8ad8da3714c3578ff62b2f128ee583eb73bdac6649` |
| Later read-only content carrier | Commit `72b6ed8b6e91a0ac7e0588fd9492a7d30c535079`, restricted to unchanged frozen-scope blobs and necessary adjacent source |

Both patch SHA-256 values were independently obtained with `sha256sum` before the environment became unavailable. Source reads initially used `git show` and `git diff` against the immutable tree. No mutable worktree source was used as the review subject.

After the execution environment became unavailable, the parent explicitly authorized finishing reads through GitHub at the pinned publication commit. Retrieved source blob identities matched the frozen manifest, including App, relay, canonical state, projection, workflow, account test, and production adapter blobs. The R2-only control-server assertion had already been read directly from the R2 tree. I did not treat the publication commit’s unrelated incoming source or evidence as part of this review.

I read the root `AGENTS.md`, the complete applicable MCP account/listing/lifetime/origin specification, and master OCRM §17. The immutable tree’s only other `AGENTS.md` is under `source/mahayana/codex-rs/tui/src/bottom_pane`, outside this scope. I did not read prior review reports, engineering diagnosis reports, root arbitration decisions, or peer reports. The supplied original renderer failure excerpt was read as raw failure evidence.

The parent assigned **gpt-6-astra / xhigh** for the complete cross-language source, account, Attempt, and custody review. I retained that session assignment and made no model/effort changes. This child does not independently attest the provider’s effective runtime model/effort configuration; prompt wording alone is not treated as an override.

**Finding R2-SEC-01 — P2 — Correct the account integration tests’ generated rejection expectation**

Classification: **TEST_FLAW**. Status: **must fix before accepting the required controls**.

Affected source:

- `source/electron-main/coordinator/coordinator-account-runtime.contract.test.ts`, lines 906–907, `assertWireOriginRetired`.
- Frozen blob: `3605572af9d62734a6cc79d1f801ddf2a158eeab`.
- File SHA-256 from the frozen manifest: `4ba9573a2b1ab3ea7ca3876b5d89729925de4dad62b9891c9720a043e7e150d1`.

The helper still requires:

```ts
assert.deepEqual(value, {
  result: {
    case: "rejected",
    value: { reason: MCP_ORIGIN_RETIRED },
  },
});
```

The actual production result on this path includes the own property `isReadonly: false`. This is the same generated-constructor/default mismatch corrected by the R2 control-server assertion.

The causal chain is explicit in the frozen source:

1. `source/packages/proto/generated/agent/v1/mcp_exec_pb.ts`, lines 410–418, initializes every `McpRejected` instance with `this.reason = ""` and `this.isReadonly = false`, then applies partial input.
2. `source/shared/node/mcp/mcp-result-factory.ts`, lines 65–68, builds the rejected arm with `new McpRejected({ reason })`, retaining that boolean default.
3. `source/electron-main/coordinator/production-provider.ts`, the `executeRoutedMcpTool` closure at lines 474–486, returns this generated result when the exact account/source capture is retired.
4. `source/electron-main/adapters/mcp-oauth.ts`, lines 377–399, returns the same generated result for a mismatched serialized source or retired Attempt.
5. The integration fixture’s `OriginNativePort` at lines 682–689 forwards through a real `node:worker_threads` `MessagePort`. `OriginChild` at lines 707–713 resolves the received outcome without stripping fields. The generated instance crosses structured clone, preserving its enumerable boolean property.
6. The helper therefore compares the received `{ reason: MCP_ORIGIN_RETIRED, isReadonly: false }` to an expected object lacking `isReadonly`, and strict deep equality fails.

Concrete affected scenarios:

- The first departed-account rejection at line 922 in **“OD-T2 production Coordinator refuses departed account and serialized origin replay before manager B.”**
- The serialized replay check at line 932 in that same test.
- The child-restart replay check at line 962 in **“OD-T2 same-account child restart retires its source while retaining the current MCP Attempt.”**

This has an acceptance consequence. The existing renderer workflow runs the complete account-runtime contract target at lines 5097–5101. Its receipt explicitly requires unique passing rows for both affected OD-T2 tests at lines 5239–5244, then requires zero failures and complete passing counts. Correct production rejection behavior therefore cannot satisfy those required oracles while the helper remains unchanged.

**Specific fix:** add `isReadonly: false` to the helper’s literal expected rejected value, preserving whole-object strict deep equality. Keep the stale-account, replay, factory-count, no-execution, same-account restart, and replacement-positive assertions intact. There is no basis for changing the generated constructor, removing the property in production, changing to partial matching, or excluding these tests.

This finding is established by source and the known generated transport shape. I did **not** execute these account tests and do **not** claim an observed Actions failure for them. The original renderer failure corroborates the constructor/default mismatch on the separate control-server/carrier path; it is not execution evidence for the account fixture.

**Focused R2 correction**

The R2 change in `source/electron-main/coordinator/coordinator-control-server.contract.test.ts`, lines 315–317, correctly changes the full expected result to:

```ts
{
  result: {
    case: "rejected",
    value: {
      reason: MCP_ORIGIN_RETIRED,
      isReadonly: false,
    },
  },
}
```

The R2 file’s blob is `19318f75ff5454a583801351ddbd6aa1a3ff067d`; SHA-256 is `90d58ed9b066a59631b20d799bfb9153be591990c4fe7823103760bc19751388`.

This remains an independent literal oracle. It continues to fail for a wrong result discriminant, missing or changed reason, missing boolean, `isReadonly: true`, and unexpected extra fields. It neither normalizes the received value nor derives the expected value from the same production constructor.

The test still uses the generated producer, actual control-server dispatcher, real native MessageChannel clone, and the shipping carrier’s JSON serialization. Its controlled echo child records and returns the exact carrier line; it does not classify MCP results. The shipping build script binds the carrier artifact to `source/electron-main/coordinator/rust-coordinator-carrier.ts`. This establishes the intended source connection without claiming a real Rust Coordinator binary or real Electron utilityProcess acceptance.

The original failure excerpt showed exactly one extra received field, `isReadonly: false`, under the expected `rejected` arm and fixed reason. It also showed the failing command’s aggregate 177 tests / 176 pass / 1 fail. Those historical observations do not establish an R2 pass.

**Complete scope and review depth**

All **77 declared changed paths** were accounted for. Review depth followed OCRM §17 rather than treating every large file as requiring an unrelated full-file audit.

| Group | Paths | Depth and disposition |
|---|---:|---|
| Core implementation and shared contracts | 19 | Deep review of changed behavior, relevant surrounding functions, callers, lifetime state, dispatch boundaries, serialization, and custody consumers |
| Behavioral-oracle files | 7 | Deep review of added/changed cases, fixture ownership, actual production connection, retained assertions, and the focused R2 correction |
| Rust constructor adaptations | 45 | Every changed hunk inspected; contract review of the explicit `origin: None` adaptations, with adjacent constructor meaning and shared type consumers |
| Specification, CI, build, evidence capture, ledger, RTM | 6 | Complete applicable spec; changed workflow/build/capture logic and relevant acceptance consumers; narrow ledger/RTM additions reviewed as references, with no completion promotion |

The 19 core paths are:

- `source/electron-main/adapters/mcp-oauth.ts`
- `source/electron-main/coordinator/coordinator-control-server.ts`
- `source/electron-main/coordinator/coordinator-executors.ts`
- `source/electron-main/coordinator/coordinator-launcher.ts`
- `source/electron-main/coordinator/production-provider.ts`
- `source/electron-main/main-production-services.ts`
- `source/electron-main/mcp/desktop-mcp-manager.ts`
- `source/electron-main/mcp/mcp-runtime.ts`
- `source/host/app/src/main.rs`
- `source/host/src/extensions/inference/provider_session.rs`
- `source/host/src/ports/mcp_state_executor.rs`
- `source/host/src/runner/coordinator_tool_relay.rs`
- `source/host/src/runner/tools/turn_toolset.rs`
- `source/host/src/runner/turn_agent_composition.rs`
- `source/shared/node/cursor-backend/backend-mcp-exec.ts`
- `source/shared/node/cursor-backend/cursor-inference.ts`
- `source/shared/node/mcp/mcp-result-factory.ts`
- `source/shared/node/mcp/tools-discovery.ts`
- `source/shared/rpc/coordinator-port.ts`

The seven behavioral-oracle files are the account-runtime, control-server, executor, MCP-runtime, backend-origin, canonical MCP-state, and production-turn-projection contract files in the frozen manifest. App tests were also reviewed within `main.rs`.

The 45 constructor adaptations are exactly the remaining Rust paths in that same manifest after excluding the six core Rust paths and the two dedicated Rust oracle files above. Their changes explicitly initialize absent Coordinator authority in built-ins and fixtures; these were not misrepresented as full reviews of unrelated feature implementations.

The six governance/build paths are `.github/workflows/rust-desktop-runtime.yml`, `desktop/scripts/build-electron-runtime.mjs`, the MCP specification, `capture_portable_contracts.py`, `parity-ledger.json`, and `requirements-traceability-matrix.json`.

Necessary adjacent reads included the actual generated MCP message implementation and field descriptors, package-lock contracts, account-runtime capture/retirement, shipping carrier, native launcher/ports, canonical wire encoder/decoder, relay result classifier, and production review/dispatch consumers. The locked dependencies remain `@bufbuild/protobuf` 1.10.1 and Connect/Connect Node 1.6.1. Dependency implementations were not exhaustively re-audited.

**Security and lifetime assessment**

The reviewed source preserves the intended separation among native source identity, account lifetime, manager Attempt, and Host relay custody:

- The control server creates source identity outside parsed arguments and retires its settlement state before disposal or protocol-breach callbacks can reenter.
- Production composition captures the exact active account binding for the owned runtime; its later assertion does not reacquire a replacement account.
- Attempt reservation remains single-owner and precedes factory invocation. Execution’s exact-ID lookup neither creates nor waits on a replacement Attempt.
- Listing checks the captured source/account/Attempt after manager and catalog awaits and before publication. Malformed catalogs remain available to the Rust whole-response refusal.
- Rust validates bounded wire origin and stores an in-process relay binding. The binding has no serialization implementation; canonical state encoding excludes it, and wire decoding explicitly produces `None`.
- Both turn snapshots and dynamic projection move the binding with its row. Duplicate public names do not reconstruct authority through name lookup.
- The actual bridge retains the descriptor across AutoReview and validates the original relay binding before execution publication.
- The final legacy HTTP assertion is request-scoped, synchronous, and placed after credential/privacy awaits and the request-ID observer, directly before `next`.
- Backend retirement classification requires the request-local witness set by the final assertion’s typed refusal. Matching error text, a typed credential failure, or a typed exception from entered transport does not establish definitive non-dispatch.
- Once execution is entered, result custody remains with the original operation. The reviewed catch boundaries do not recheck currentness to manufacture a rejected outcome.
- The Rust relay’s result-envelope classifier accepts a well-formed generated rejected arm with a boolean `isReadonly`. Transport-error arms, malformed outcomes, lost replies, and post-publication relay errors remain unknown.

The authored tests cover these boundaries with held promises, current-owner controls, generated clients, controlled final transport, and a default-facade loopback case. Their existence and source connection are not runtime passing evidence. Finding R2-SEC-01 prevents accepting two required account controls as currently authored.

**Selected blob evidence**

| Source | Frozen blob |
|---|---|
| R2 control-server oracle | `19318f75ff5454a583801351ddbd6aa1a3ff067d` |
| Account integration oracle | `3605572af9d62734a6cc79d1f801ddf2a158eeab` |
| Generated MCP messages | `e1a0ccf6fdbd8d3d100aa806e45514e577a3b1ff` |
| Shared result factory | `503e2960484f5fd52b6b449de9c95941764be877` |
| Production Coordinator binding | `c21a21b5803775e74716994afe44a68b4b1ece1e` |
| Production MCP adapter | `1684a9cfafd02f6cdd96091b7b85d9efb3f3dbcc` |
| MCP Attempt owner | `b1d651721db46b8752fc9c1ef95b689082e6c94a` |
| Rust relay and result custody | `64f0548dd1c93c6813039aaa5dce6b2ab3b8f0d1` |
| Actual App bridge/decoders/tests | `eafdf6a0cbae3651bb70d55c5cae719e31a89fdb` |
| Canonical MCP state owner | `82aae3d6897b758c6c74eee206015409318c5be6` |
| Shipping carrier | `375a706177e9bfa7aa7c9bf591d6688887af12fb` |
| Existing workflow | `21027a8dac6fc0b23959cf84c946f88fe24fad26` |

**Limits and terminal disposition**

No builds, tests, linters, formatters, generators, schema checks, product parsers, or local/htch executable validation were run. Repository source, staging/index, and remote state remained unchanged. No fresh upstream full-read or parity claim was made.

This review does not close real account/provider compatibility, official/Box final resource boundaries, same-manager registration changes, native credential/session epochs, canonical Rust permission migration, packaged platform acceptance, or global OCRM G01–G12 / AC01–AC24.

**Source review complete; one P2 finding requires correction; focused R2 assertion accepted.**

The requested local report could not be written because the execution environment became unavailable. Per the parent’s subsequent instruction, this terminal text is the report for root to persist verbatim. **Report SHA-256 is pending hashing of those exact persisted bytes by root; no digest is fabricated here.**