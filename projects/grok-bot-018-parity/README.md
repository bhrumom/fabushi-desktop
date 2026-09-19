# Grok Bot 0.18 parity refactor

Project ID: GBR  
Owner: Fabushi desktop  
Source of truth: this repository project folder + GitHub branch/CI/release evidence.

## Objective

Refactor Fabushi Desktop around the pinned Grok Bot 0.18 reconstruction, reproducing its observable UI, interactions, plugin experience and Agent architecture as closely as the evidence-backed reference permits.

Reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`  
Target baseline: `bhrumom/fabushi-desktop@12d4aadb4a93a1413ece44aba987162e90b31229`  
Shipped Mac candidate code SHA: `1f27c239aa8931b3266b9b83c84ff234332785d9`  
Release: `grok-parity-mac-101` / `2.0.0-alpha.3`

## Implemented result

- Production renderer is the byte-exact pinned reference renderer.
- Pinned `SandAgentRunner` + `AnysphereAgent` own the production Agent lifecycle/orchestration.
- No Mahayana/Codex CLI wrapper and no legacy hand-written model/tool loop remain in the production Agent path.
- Files, Terminal, Browser, Computer, MCP, Subagent, private-skill and workflow capabilities execute through local host adapters.
- The intentional product difference is Computer location: Agents operate the Mac where Fabushi is installed instead of a cloud Box.
- Contacts/Messenger-specific, Telegram, payment, MiniApp and Mahayana workbench surfaces are not mounted in this parity product.
- Marketplace/MCP/OAuth/account/workflow/listener paths are executable; external-service paths fail closed when unavailable.

## Acceptance state

Automated source/runtime/package/release scope is complete. Final human installation and interaction testing remains external by the original requirement.

Final automated evidence:
- source gate Run `35419150457`: 105/105 runtime tests PASS plus exact pinned source/typecheck/production build;
- macOS Run `35419150459`: build/package, artifact upload, immutable release-branch verification, release and exact tag-SHA checks PASS; the non-blocking headless renderer probe recorded `manualValidationRequired:true`, so no automated packaged-UI PASS is claimed;
- Release `grok-parity-mac-101` resolves exactly to `1f27c239aa8931b3266b9b83c84ff234332785d9`;
- Actions artifact `10577580224`, digest `sha256:1c70dad6d99bc327a27165767636e47d1c4a0131db21e3b796b07982d7ccd4e1`.

Release anchor `release/grok-parity-alpha3-final` is immutable at the shipped product SHA. PR #1 remains open for GBR-007 human acceptance and defect closure.
