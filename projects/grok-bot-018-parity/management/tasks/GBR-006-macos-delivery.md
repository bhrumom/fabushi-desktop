# GBR-006 — macOS Actions package and prerelease delivery

Status: PASS for automated delivery / human installation and interaction testing delegated to GBR-007

## Objective
Build the completed macOS Grok-parity application in GitHub Actions, prove the final source gate and package/release chain on one exact shipped SHA, publish DMG/ZIP prerelease assets, and stop before human product acceptance.

## Final shipped candidate
- version: `2.0.0-alpha.2`
- shipped code SHA: `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`
- pinned reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`
- PR: #1 remains open and unmerged for human defect closure.

## Source gate evidence
Push source gate Run `35417698506` completed SUCCESS on the exact shipped SHA.
- exact pinned renderer/source parity — success
- pinned reference runtime bundles — success
- runtime contracts — **103/103 PASS, 0 fail**
- pinned reference `source:typecheck` — success
- Fabushi TypeScript/Vite production build — success

The PR-triggered source gate Run `35417701558` also completed SUCCESS on the same SHA.

## Final macOS delivery evidence
Run `35417698507` completed SUCCESS on `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`.
- install dependencies — success
- typecheck and build — success
- unsigned macOS DMG/ZIP package — success
- packaged renderer/local bridge headless smoke — **manual validation required**: the GitHub-hosted Mac produced no renderer smoke report within the bounded probe, so the non-blocking step wrote `manualValidationRequired:true` and exited 1; this is not counted as product acceptance
- artifact upload — success
- exact release-branch SHA verification — success
- prerelease publication — success
- published tag exact-SHA verification — success

GitHub Actions artifact:
- id: `10576418103`
- name: `fabushi-grok-parity-macos`
- size: `393022970` bytes
- digest: `sha256:6d3bf39199c312e695a6ab59cb7eb7cf408df6ab2ff34dee1a7076eb91406b2c`

GitHub prerelease:
- tag: `grok-parity-mac-92`
- tag resolves exactly to: `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`
- release: https://github.com/bhrumom/fabushi-desktop/releases/tag/grok-parity-mac-92
- DMG: `fabushi-grok-parity-2.0.0-alpha.2-macos-arm64.dmg` — `196953101` bytes
- ZIP: `fabushi-grok-parity-2.0.0-alpha.2-macos-arm64.zip` — `197395586` bytes

## Packaged smoke evidence
The final GitHub-hosted headless runner did not emit the renderer UI probe report. The workflow recorded `manualValidationRequired:true` and continued because the original requirement explicitly delegates product testing to humans. This is not counted as packaged UI acceptance.

## Remaining acceptance
No automated delivery blocker remains for GBR-006.

The original requirement explicitly delegates product testing to humans. GBR-007 therefore remains open for installation and real interaction testing of the released DMG, including UI/Agent/Computer/Browser/Marketplace/Teach/sharing flows. PR #1 must remain open until defects found in that round are fixed and re-released.
