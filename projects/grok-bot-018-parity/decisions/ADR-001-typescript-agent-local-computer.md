# ADR-001 — Reference TypeScript Agent runtime with local-computer adaptation

Status: accepted and implemented  
Date: 2026-09-18  
Final automated evidence: 2026-09-19

## Decision

Use the pinned Grok Bot 0.18 TypeScript Agent/runtime architecture as the Fabushi production Agent core instead of a Mahayana/Codex CLI wrapper.

The production path uses pinned `SandAgentRunner` and `AnysphereAgent`. Fabushi supplies host adapters for inference credentials and tool execution. Reference remote Box/Computer operations map to the Mac where Fabushi is installed.

## Consequences

- Exact pinned reference renderer is the product UI contract.
- Reference Agent owns model/action/tool/checkpoint/summarization lifecycle.
- Fabushi host adapters own local execution, permission/approval, credentials and external-service seams.
- CLI compatibility is not the Agent implementation and is not the renderer's primary execution path.
- Cloud-computer provisioning is excluded; Computer/Teach operate locally.
- Product-difference claims must be explicit and evidence-backed.
- Human UX/function testing remains external after the GitHub Actions release candidate.

## Final automated evidence

Shipped code SHA: `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`  
Source gate: Run `35417698506` — 103/103 runtime tests PASS plus exact source/typecheck/build.  
Mac delivery: Run `35417698507` — package/artifact/release/tag checks PASS; the GitHub-hosted headless renderer probe did not emit its UI report and recorded `manualValidationRequired:true`, so packaged UI acceptance remains human.  
Release: `grok-parity-mac-92`.
