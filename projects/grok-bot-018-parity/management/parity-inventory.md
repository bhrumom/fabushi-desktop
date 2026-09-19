# Grok Bot 0.18 parity inventory

Reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`  
Target: `bhrumom/fabushi-desktop@refactor/grok-bot-018-parity-mac`

## Current acceptance authority

`parity-inventory.generated.json` is retained as the historical 2026-09-18 audit snapshot at target head `910f65e42b0dd096e9c969eba03666f46ae764c9`. Its PASS/PARTIAL/FAIL rows describe the pre-replacement target and MUST NOT be rewritten to pretend that earlier audit already passed.

Current acceptance is fail-closed on these live sources instead:

1. exact pinned source gate: `frontend/src` → `desktop/src` and pinned `source/` → `reference/grok-bot-0.18/source/`;
2. pinned official `renderer-closure.json`;
3. executable desktop/coordinator claim classifications and runtime contracts;
4. pinned reference Agent/runtime bundles and pinned reference source typecheck;
5. exact-SHA macOS package/release evidence;
6. the user-requested manual packaged UX/function test.

## Exact source state

- Renderer: **308 / 308 pinned renderer blobs exact**, 0 missing, 0 different.
- Reference source: **1,724 / 1,724 pinned source blobs exact**, 0 missing, 0 different.
- Official renderer closure: findings = 0; clean routes without evidence = 0.
- Official coordinator calls: **58 / 58** have executable local paths.
- Official coordinator subscription families: **10 / 10** have local event paths.
- Official desktop bridge claims: **95 / 95 classified**, no unknown classification:
  - 79 local-implemented
  - 11 local-computer-product-difference
  - 2 local-computer-adaptation
  - 3 external-account-service

## Runtime state

- Production renderer is the pinned reference `ProductionRenderer`; legacy Contacts/Telegram/payment/MiniApp/Mahayana renderer code is not mounted in the Mac parity product.
- Pinned `SandAgentRunner` owns per-agent run/interrupt/quiesce lifecycle.
- Pinned `AnysphereAgent` owns production model/action/tool/checkpoint/summarization orchestration.
- The former hand-written bounded tool loop has been removed from production.
- Reference `ConversationStateStructure` is persisted per Agent.
- Files/Terminal/Browser/Computer/MCP/Subagent actions execute against the Mac where Fabushi is installed.
- Fabushi account OAuth token is the production inference bearer credential when the account inference endpoint is configured.
- Marketplace installs executable MCP/private-skill payloads and supports install/uninstall, OAuth/account and publication/listener seams.
- Cross-user/shared-room paths use the pinned Sand sharing relay adapter when a backend is configured and fail closed otherwise.
- Reference TrayManager handles coordinator failures and tray lifecycle.
- Local telemetry is persisted locally rather than silently dropped or exported to an unconfigured cloud backend.
- Local Computer Teach recording uses the packaged native helper, persists recording evidence, and dispatches the result to the Learn from demonstration private workflow.

## Latest hard source evidence

GitHub Actions push Run `35425432746` at immutable product/package SHA `98ac56f037cb1b0eaa66781087da986334754908` completed successfully:

- exact pinned source parity gate: PASS
- Host carrier external-dependency classification: PASS
- pinned SandAgentRunner / AnysphereAgent / TrayManager / Sand sharing bundles: PASS
- CommonJS syntax: PASS
- runtime contracts: **109 / 109 PASS, 0 fail**
- pinned reference source typecheck: PASS
- TypeScript/Vite production build: PASS
- PR source gate Run `35425434873`: SUCCESS on the same product SHA

Final macOS package/release Run `35425432753` on the same SHA completed successfully:
- unsigned arm64 DMG/ZIP package: PASS
- packaged renderer/local bridge headless probe: no UI report; non-blocking diagnostic only (`manualValidationRequired:true`)
- artifact upload: PASS
- release-branch SHA gate: PASS
- prerelease publish: PASS
- published tag exact-SHA check: PASS
- artifact `10579089056`, 393058647 bytes, digest `sha256:6c91562e2d8ede5641c6c4bb9010a1cd3ebad1c0d0744a5cb8cc48b9bd1850d1`
- release/tag `grok-parity-mac-126` → `98ac56f037cb1b0eaa66781087da986334754908`
- DMG `fabushi-grok-parity-2.0.0-alpha.4-macos-arm64.dmg`, 196976857 bytes, `sha256:3516c7de4f8052e0fc219f08a3c2234a4648569d63c1277dcc7164a45dfe7320`
- ZIP `fabushi-grok-parity-2.0.0-alpha.4-macos-arm64.zip`, 197404010 bytes, `sha256:c70bd2a13c2fabf2cab3a74b959ecead488a2e4df76656fd18ef7b4d52e3a57c`

## Historical generated snapshot

The old generated snapshot recorded:

| Historical closure @ 910f65e… | PASS | PARTIAL | FAIL | PRODUCT_DIFFERENCE |
|---|---:|---:|---:|---:|
| renderer modules (275) | 17 | 196 | 61 | 1 |
| runner capsules (70) | 10 | 58 | 0 | 2 |
| Electron bindings (16) | 9 | 7 | 0 | 0 |

Those counts remain preserved for audit provenance only. They are not current source truth after exact source replacement and reference-runtime cutover.

## Intentional product difference

The only authorized product-level architecture difference is Computer location: Grok's cloud Box/VNC/cursor-agent infrastructure is replaced by the Mac where Fabushi is installed. Renderer semantics remain the pinned reference semantics, including its local-computer phase.

## External evidence boundary

The pinned reconstruction's own Host activation script requires immutable carrier artifact `src/app/dist/host/host-main.cjs`. That carrier is not present in the pinned Git repository. This is **EXTERNAL_DEPENDENCY**, not a fabricated PASS and not evidence that the exact vendored source differs.

The three desktop claims classified as `external-account-service` depend on subscription/dashboard services outside the pinned source and fail closed when unavailable.

## Remaining acceptance

Source/runtime parity gates are green. Remaining product acceptance is the requested human validation of the packaged Mac build: visual/interaction parity, real signed-in provider behavior, macOS permissions, and real-device Computer/Teach workflows.
