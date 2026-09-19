# GBR-006 — macOS Actions package and prerelease delivery

Status: PASS for automated delivery / human installation and interaction testing delegated to GBR-007

## Objective
Build the completed macOS Grok-parity application in GitHub Actions, prove the final source gate and package/release chain on one exact shipped SHA, publish DMG/ZIP prerelease assets, and stop before human product acceptance.

## Current immutable human-test candidate
- version: `2.0.0-alpha.3`
- package SHA: `1f27c239aa8931b3266b9b83c84ff234332785d9`
- pinned reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`
- PR: #1 remains open and unmerged for human defect closure.

## Source gate evidence
Push source gate Run `35419150457` completed SUCCESS on the exact package SHA.
- exact pinned renderer/source parity — success
- pinned reference runtime bundles — success
- runtime contracts — **105/105 PASS, 0 fail**
- pinned reference `source:typecheck` — success
- Fabushi TypeScript/Vite production build — success

## Final macOS delivery evidence
Run `35419150459` completed SUCCESS on `1f27c239aa8931b3266b9b83c84ff234332785d9`.
- install dependencies — success
- typecheck and build — success
- unsigned macOS DMG/ZIP package — success
- packaged renderer/local bridge headless smoke — **manual validation required**: the GitHub-hosted Mac produced no renderer smoke report within the bounded probe, so the non-blocking step wrote `manualValidationRequired:true`; this is not counted as product acceptance
- artifact upload — success
- exact release-branch SHA verification — success
- prerelease publication — success
- published tag exact-SHA verification — success

GitHub Actions artifact:
- id: `10577580224`
- name: `fabushi-grok-parity-macos`
- size: `393011773` bytes
- digest: `sha256:1c70dad6d99bc327a27165767636e47d1c4a0131db21e3b796b07982d7ccd4e1`

GitHub prerelease:
- tag: `grok-parity-mac-101`
- tag resolves exactly to: `1f27c239aa8931b3266b9b83c84ff234332785d9`
- release: https://github.com/bhrumom/fabushi-desktop/releases/tag/grok-parity-mac-101
- DMG: `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.dmg` — `196950473` bytes — `sha256:695ffe13980e185ac8f4852635147d5ddf45eefebac6903294dbb4962960371f`
- ZIP: `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.zip` — `197394273` bytes — `sha256:8db709695da85299c10e1155df2f798dd4d9325422f2f0c927101b199fb2052a`

## Packaged smoke evidence
The GitHub-hosted headless runner did not emit the renderer UI probe report. The workflow recorded `manualValidationRequired:true` and continued because the original requirement explicitly delegates product testing to humans. This is not counted as packaged UI acceptance.

## Remaining acceptance
No automated delivery blocker remains for GBR-006.

The original requirement explicitly delegates product testing to humans. GBR-007 remains open for installation and real interaction testing of this release, including UI/Agent/Computer/Browser/Marketplace/Teach/sharing flows. Any product-code defect invalidates this candidate and requires a new exact-SHA source gate and release.
