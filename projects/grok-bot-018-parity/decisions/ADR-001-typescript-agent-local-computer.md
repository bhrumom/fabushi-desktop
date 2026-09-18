# ADR-001 — TypeScript agent runtime with local-computer adaptation

Status: accepted
Date: 2026-09-18

## Decision

Use a TypeScript agent coordinator/host/local-exec architecture matching the reference reconstruction instead of designing the Fabushi agent as a Mahayana CLI wrapper.

Map reference remote Box/Computer operations to the installed computer. Preserve explicit permission boundaries for local tools.

## Consequences

- Renderer communicates with a typed coordinator/host bridge.
- CLI compatibility may exist separately but is not the agent core.
- Cloud-computer provisioning/status UI is excluded from the parity surface.
- Rust may still be used for low-level native capabilities where it is an implementation detail behind the local-exec boundary.
