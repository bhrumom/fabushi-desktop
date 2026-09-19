# GBR-006 — macOS Actions package and prerelease delivery

Status: PASS for automated delivery / human installation and interaction testing delegated to GBR-007

## Objective
Build the completed macOS Grok-parity application in GitHub Actions, prove source/runtime and package/release evidence on one exact product SHA, publish DMG/ZIP prerelease assets, and stop before human product acceptance.

## Current human-test candidate
- version: `2.0.0-alpha.4`
- product/package SHA: `98ac56f037cb1b0eaa66781087da986334754908`
- pinned reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`
- human-test release alias: `release/grok-parity-alpha4-final` → exact same SHA
- packaging workflow gate used the legacy branch label `release/grok-parity-alpha3-final`; that ref also resolved to the same SHA during the run
- PR: #1 remains open and unmerged for human defect closure.

## Source gate evidence
- push Run `35425432746` — SUCCESS
- PR Run `35425434873` — SUCCESS
- exact pinned renderer/source parity — PASS
- pinned reference runtime bundles — PASS
- runtime contracts — **109/109 PASS, 0 fail**
- pinned reference `source:typecheck` — PASS
- Fabushi TypeScript/Vite production build — PASS

## macOS delivery evidence
Run `35425432753` — SUCCESS on the exact product/package SHA.
- install dependencies — PASS
- typecheck/build — PASS
- unsigned arm64 DMG/ZIP package — PASS
- bounded packaged renderer diagnostic — `manualValidationRequired:true`; not counted as product acceptance
- artifact upload — PASS
- release-branch SHA gate — PASS
- prerelease publication — PASS
- published tag == build SHA — PASS

GitHub Actions artifact:
- id: `10579089056`
- name: `fabushi-grok-parity-macos`
- size: `393058647` bytes
- digest: `sha256:6c91562e2d8ede5641c6c4bb9010a1cd3ebad1c0d0744a5cb8cc48b9bd1850d1`

GitHub prerelease:
- tag: `grok-parity-mac-126`
- release: https://github.com/bhrumom/fabushi-desktop/releases/tag/grok-parity-mac-126
- tag SHA: `98ac56f037cb1b0eaa66781087da986334754908`
- DMG: `fabushi-grok-parity-2.0.0-alpha.4-macos-arm64.dmg` — 196976857 bytes — `sha256:3516c7de4f8052e0fc219f08a3c2234a4648569d63c1277dcc7164a45dfe7320`
- ZIP: `fabushi-grok-parity-2.0.0-alpha.4-macos-arm64.zip` — 197404010 bytes — `sha256:c70bd2a13c2fabf2cab3a74b959ecead488a2e4df76656fd18ef7b4d52e3a57c`

## Packaged smoke boundary
The GitHub-hosted macOS runner launched the packaged executable under the bounded diagnostic path. Final installed-product UI/function acceptance remains explicitly human-owned under GBR-007; no automated packaged-UI PASS is claimed from the runner diagnostic.

## Remaining acceptance
No automated delivery blocker remains for GBR-006. GBR-007 owns installed-product validation of UI, Agent behavior, local Computer/Browser, Marketplace/plugins, Teach/sharing, approvals, cancellation and error recovery. Any product-code defect invalidates this candidate and requires a new exact-SHA source gate and Mac release.
