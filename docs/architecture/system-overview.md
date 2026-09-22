# Fabushi Desktop system overview

Status: canonical architecture boundary model  
Last reviewed: 2026-09-22

Active migration Specs may temporarily describe an implementation that has not fully reached this boundary model; such divergence must be explicit and temporary.

## Runtime boundaries

```text
React Renderer (TypeScript)
        |
        | typed application protocol / projection events
        v
Electron Preload (thin bridge)
        |
        v
Electron Main (TypeScript desktop shell / OS integration)
        |
        | typed runtime protocol
        v
Mahayana Coordinator (Rust-preferred)
        |
        +----------------------+-----------------------+
        |                      |                       |
        v                      v                       v
Agent Host                Local/Computer Exec       MCP / OAuth relay
        |
        v
Agent Runner
(model/tool execution)
```

Language is not the architecture. Each layer uses the language that best fits its environment while preserving the boundary.

## Ownership

### Renderer

Owns UI layout, interaction, accessibility, and projection of canonical runtime state. It does not own authoritative Agent lifecycle, Host/Runner supervision, or privileged local execution.

### Electron main

Owns application/window lifecycle, operating-system integration, desktop packaging/update hooks, process spawning/bridging, and routing between renderer/preload and runtime boundaries. It does not become the source of truth for Agent execution state.

### Preload

Preload is intentionally thin: a narrow typed bridge, not a business/runtime orchestration layer.

### Mahayana Coordinator

The Coordinator is the authoritative coordination boundary for Agent/runtime state. Responsibilities include, as applicable: request/reply/event correlation, cancellation, reconnect/resync, session/conversation state, Gateway routing, Host supervision, local-exec routing, MCP relay, OAuth forwarding, crash settlement/recovery, and terminalization of outstanding operations.

Coordinator, Host, and Runner remain distinct responsibility boundaries even when they share a language or package workspace.

### Agent Host

The Host owns runtime hosting/isolation concerns and presents a stable execution boundary to the Coordinator. Host failure must be detectable and settle or recover outstanding work instead of leaving the UI indefinitely busy.

### Agent Runner

The Runner owns per-turn model/tool execution. It must not depend on React/Electron UI concepts. Public behavior is expressed through runtime contracts/events rather than direct UI mutation.

### Privileged capabilities

Local execution, computer control, connectors/MCP, OAuth, filesystem/network-sensitive actions, and other privileged capabilities must pass through explicit capability/security boundaries with auditable lifecycle and failure behavior.

## State ownership

```text
Coordinator/runtime authoritative state
              |
              v
typed events / snapshots
              |
              v
Renderer projection
              |
              v
UI
```

The renderer may cache/project state but must not infer canonical Agent state from DOM shape, animation, timers, or presentation heuristics.

## Request lifecycle

Every accepted runtime request must eventually settle as completed/success, failed, or cancelled. Disconnection is not a terminal success state. Reconnect must support resynchronization when the underlying operation may survive renderer/transport loss.

## Reference architecture

Grok Bot 0.18 is a reference for Coordinator/Host/Runner separation and product/runtime behavior where active Fabushi Specs say so. Fabushi does not require identical source language, filenames, or class names. Architectural equivalence is judged by ownership, protocol, lifecycle, failure isolation, state authority, supervision, and observable behavior.
