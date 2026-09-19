# Grok Bot 0.18 parity refactor

Project ID: GBR  
Owner: Fabushi desktop  
Source of truth: this repository project folder + GitHub branch/CI/release evidence.

## Objective

Refactor Fabushi Desktop around the pinned Grok Bot 0.18 reconstruction, reproducing its observable UI, interactions, plugin experience and Agent architecture as closely as the evidence-backed reference permits.

Reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`  
Target baseline: `bhrumom/fabushi-desktop@12d4aadb4a93a1413ece44aba987162e90b31229`  
Shipped Mac candidate code SHA: `528ddc8e31c320ca472191cf29a860c2086dabc2`  
Release: `grok-parity-mac-97` / `2.0.0-alpha.3`

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
- source gate Run `35418603612`: 105/105 runtime tests PASS plus exact pinned source/typecheck/production build;
- macOS Run `35418603624`: build/package, artifact upload, release and exact tag-SHA checks PASS; the non-blocking headless renderer probe produced no UI report and recorded `manualValidationRequired:true`, so no automated packaged-UI PASS is claimed;
- Release `grok-parity-mac-97` resolves exactly to `528ddc8e31c320ca472191cf29a860c2086dabc2`;
- Actions artifact `10576404427`, digest `sha256:7899c1e47537c4793de59118d9328085038505069e438b255c62b0c0421768b2`.

PR #1 remains open for GBR-007 human acceptance and defect closure.
