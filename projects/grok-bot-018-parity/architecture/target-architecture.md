# Target architecture

## Production path

`ProductionRenderer -> context-isolated preload -> reference coordinator adapter -> SandAgentRunner -> AnysphereAgent -> local host adapters -> installed-Mac executors`

### Renderer
- `frontend/src` from the pinned reference maps byte-for-byte to `desktop/src`.
- Fabushi-only Contacts/Messenger/Telegram/payment/MiniApp/Mahayana product surfaces are not mounted.

### Agent/runtime
- Pinned reference `SandAgentRunner` owns run/interrupt/quiesce lifecycle.
- Pinned reference `AnysphereAgent` owns model/action/tool/checkpoint/summarization orchestration.
- Reference `ConversationStateStructure` persists per Agent.
- No CLI wrapper or legacy bounded model/tool loop is in the production Agent path.
- Signed-in Fabushi account credentials are used for production inference when the configured account inference endpoint is present; provider/API-key configuration is an explicit fallback.

### Host/tool boundary
- Files/Terminal/Browser/Computer/MCP/Subagent/workflow capabilities are injected through the reference Agent tool boundary.
- Permission, approval, cancellation, retry, audit and transcript lifecycle remain local host responsibilities.
- Browser reference actions execute in per-Agent local browser windows.
- Computer actions and Teach recording target the installed Mac through the packaged native helper.

### Plugins
- The local Marketplace works without a remote catalog and installs executable Custom MCP Server and Private Skill payloads.
- Remote Marketplace/provider, MCP OAuth/accounts/tool toggles, workflow publication and listener seams are executable.
- External account/subscription services fail closed when not configured.

## Product adaptation

The only intentional architecture-level difference is Computer location. Reference cloud Box/VNC provisioning is not required; the exact reference renderer's local-computer behavior maps to the installed Mac.

## Verification boundary

CI enforces byte-exact pinned renderer/source parity, official renderer closure and runtime contracts. The pinned reconstruction's own Host activation proof additionally requires immutable carrier artifact `src/app/dist/host/host-main.cjs`, which is not stored in the reference Git repository; that carrier-only proof remains an external evidence dependency.
