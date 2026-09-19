# Grok Bot 0.18 parity refactor

Project ID: GBR  
Owner: Fabushi desktop  
Source of truth: this repository project folder + GitHub branch/CI/release evidence.

## Objective

Refactor Fabushi Desktop around the pinned Grok Bot 0.18 reconstruction, reproducing its observable UI, interactions, plugin experience and Agent architecture as closely as the evidence-backed reference permits.

Reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`  
Target baseline: `bhrumom/fabushi-desktop@12d4aadb4a93a1413ece44aba987162e90b31229`  
Final shipped Mac package SHA: `1f27c239aa8931b3266b9b83c84ff234332785d9`  
Immutable release branch: `release/grok-parity-alpha3-final`  
Release: `grok-parity-mac-101` / `2.0.0-alpha.3`

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

- Push source gate Run `35419150457`: **105/105 runtime tests PASS**, exact pinned source parity, reference source typecheck and production build PASS.
- PR source gate Run `35419154451`: SUCCESS on the same package SHA.
- macOS Run `35419150459`: SUCCESS for build/package, bounded diagnostic smoke, artifact upload, immutable release-branch SHA verification, prerelease publication and final tag-SHA verification.
- Artifact `10577580224`: `393011773` bytes; digest `sha256:1c70dad6d99bc327a27165767636e47d1c4a0131db21e3b796b07982d7ccd4e1`.
- Release `grok-parity-mac-101` tag resolves exactly to `1f27c239aa8931b3266b9b83c84ff234332785d9`.
- DMG: `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.dmg`, `196950473` bytes, `sha256:695ffe13980e185ac8f4852635147d5ddf45eefebac6903294dbb4962960371f`.
- ZIP: `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.zip`, `197394273` bytes, `sha256:8db709695da85299c10e1155df2f798dd4d9325422f2f0c927101b199fb2052a`.

The GitHub-hosted headless renderer probe still produced no UI report and recorded `manualValidationRequired:true`; no automated packaged-UI PASS is claimed. Automated implementation and delivery are complete. PR #1 remains open for GBR-007 human installation/visual/function acceptance and defect closure.
