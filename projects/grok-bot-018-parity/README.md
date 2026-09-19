# Grok Bot 0.18 parity refactor

Project ID: GBR  
Owner: Fabushi desktop  
Source of truth: this repository project folder + GitHub branch/CI/release evidence.

## Objective

Refactor Fabushi Desktop around the pinned Grok Bot 0.18 reconstruction, reproducing its observable UI, interactions, plugin experience and Agent architecture as closely as the evidence-backed reference permits.

Reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`  
Target baseline: `bhrumom/fabushi-desktop@12d4aadb4a93a1413ece44aba987162e90b31229`  
Final shipped Mac product/package SHA: `98ac56f037cb1b0eaa66781087da986334754908`  
Release branch used by the packaging gate: `release/grok-parity-alpha3-final` -> exact same product SHA at release time  
Release: `grok-parity-mac-126` / `2.0.0-alpha.4`

## Implemented result

- Production renderer is the byte-exact pinned reference renderer: 308/308 blobs exact.
- Pinned reference backend source is vendored byte-for-byte: 1,724/1,724 blobs exact.
- Pinned `SandAgentRunner` + `AnysphereAgent` own production Agent lifecycle/orchestration.
- No Mahayana/Codex CLI wrapper and no legacy hand-written model/tool loop remain in the production Agent path.
- Files, Terminal, Browser, Computer, MCP, Subagent, private-skill, workflow, Tray, sharing and Teach paths have executable reference/local adapters.
- The intentional product difference is Computer location: Agents operate the Mac where Fabushi is installed instead of a cloud Box.
- Contacts/Messenger-specific, Telegram, payment, MiniApp and Mahayana workbench surfaces are not mounted.
- Marketplace/MCP/OAuth/account/workflow/listener paths are executable; external-service paths fail closed when unavailable.

## Final automated evidence

- Push source gate Run `35425432746`: **109 pass / 0 fail runtime contracts**, exact pinned source parity, reference source typecheck and production build PASS.
- PR source gate Run `35425434873`: SUCCESS on the same product SHA.
- macOS Run `35425432753`: SUCCESS for build/package, bounded diagnostic smoke, artifact upload, release-branch SHA verification, prerelease publication and final tag-SHA verification.
- Artifact `10579089056`: `393058647` bytes; digest `sha256:6c91562e2d8ede5641c6c4bb9010a1cd3ebad1c0d0744a5cb8cc48b9bd1850d1`.
- Release `grok-parity-mac-126` tag resolves exactly to `98ac56f037cb1b0eaa66781087da986334754908`.
- DMG: `fabushi-grok-parity-2.0.0-alpha.4-macos-arm64.dmg`, `196976857` bytes, `sha256:3516c7de4f8052e0fc219f08a3c2234a4648569d63c1277dcc7164a45dfe7320`.
- ZIP: `fabushi-grok-parity-2.0.0-alpha.4-macos-arm64.zip`, `197404010` bytes, `sha256:c70bd2a13c2fabf2cab3a74b959ecead488a2e4df76656fd18ef7b4d52e3a57c`.

The GitHub-hosted packaged renderer probe still produced no UI report and recorded `manualValidationRequired:true`; no automated packaged-UI PASS is claimed. Automated implementation and delivery are complete for the current product SHA. PR #1 remains open for GBR-007 human installation/visual/function acceptance and defect closure.

Record-only commits may advance the PR head after `98ac56f037cb1b0eaa66781087da986334754908`; they do not change the released product unless product/runtime paths change and a new exact-SHA package is published.
