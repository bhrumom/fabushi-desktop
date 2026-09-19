# GBR-002 — Reference TypeScript Agent runtime

Status: complete for automated production-runtime scope / packaged human acceptance delegated to GBR-007

## Objective
Replace the earlier Fabushi-specific agent loop with the pinned Grok Bot 0.18 Agent lifecycle/orchestration without using Mahayana CLI. The only product adaptation is that tools operate the Mac where Fabushi is installed.

## Actual result
Production path:

`ProductionRenderer -> context-isolated preload -> reference coordinator adapter -> SandAgentRunner -> AnysphereAgent -> local tool boundary`

- `SandAgentRunner` is bundled directly from pinned reference source and owns run/interrupt/quiesce lifecycle.
- `AnysphereAgent` is the production orchestration engine.
- The previous hand-written bounded model/tool loop was removed from production and cannot be re-enabled through `FABUSHI_AGENT_ENGINE=legacy`.
- Reference conversation state is persisted per agent.
- The reference Agent owns model steps, tool-call continuation, checkpoint/state evolution and summarization behavior exposed by the pinned source.
- Fabushi injects local Files/Terminal/Browser/Computer/MCP/Subagent executors through the reference tool boundary.
- Inference uses signed-in Fabushi account credentials when the configured account inference endpoint is present; environment API-key configuration is a provider/development fallback, not a CLI wrapper.

## Verification
- Exact reference renderer/source diff is enforced in CI.
- Run `35416952328` on `ded4645501db434753de38b174412145b3fc32dc` passed 80/80 runtime tests, pinned reference source typecheck and renderer production build.
- Tests cover:
  - production Host defaults to exact AnysphereAgent;
  - plain-text turn lifecycle;
  - model tool-call -> local execution -> follow-up model turn;
  - reference state persistence;
  - Browser/MCP/Subagent routing;
  - account-backed inference;
  - private provider delta isolation.

## Acceptance
- Non-CLI Agent design: PASS.
- Reference Agent lifecycle/orchestration in production: PASS.
- Installed-Mac tool target: PASS / PRODUCT_DIFFERENCE.
- Packaged end-user behavior: delegated to GBR-007 manual acceptance.
