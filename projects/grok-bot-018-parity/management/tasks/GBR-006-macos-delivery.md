# GBR-006 — macOS Actions package and prerelease delivery

Status: PASS for automated delivery / human installation and interaction testing delegated to GBR-007

## Objective
Build the completed macOS Grok-parity application in GitHub Actions, prove source/runtime and package/release evidence on one immutable candidate SHA, publish DMG/ZIP prerelease assets, and stop before human product acceptance.

## Immutable human-test candidate
- version: `2.0.0-alpha.3`
- product/package SHA: `1f27c239aa8931b3266b9b83c84ff234332785d9`
- pinned reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`
- immutable release branch: `release/grok-parity-alpha3-final`
- PR: #1 remains open and unmerged for human defect closure.

## Source gate evidence
- push Run `35419150457` — SUCCESS
- PR Run `35419154451` — SUCCESS
- exact pinned renderer/source parity — PASS
- pinned reference runtime bundles — PASS
- runtime contracts — **105/105 PASS, 0 fail**
- pinned reference `source:typecheck` — PASS
- Fabushi TypeScript/Vite production build — PASS

## macOS delivery evidence
Run `35419150459` — SUCCESS on the exact package SHA.
- install dependencies — PASS
- typecheck/build — PASS
- unsigned arm64 DMG/ZIP package — PASS
- bounded packaged renderer diagnostic — `manualValidationRequired:true`; not counted as product acceptance
- artifact upload — PASS
- immutable release branch == build SHA — PASS
- prerelease publication — PASS
- published tag == build SHA — PASS

GitHub Actions artifact:
- id: `10577580224`
- name: `fabushi-grok-parity-macos`
- size: `393011773` bytes
- digest: `sha256:1c70dad6d99bc327a27165767636e47d1c4a0131db21e3b796b07982d7ccd4e1`

GitHub prerelease:
- tag: `grok-parity-mac-101`
- release: https://github.com/bhrumom/fabushi-desktop/releases/tag/grok-parity-mac-101
- tag SHA: `1f27c239aa8931b3266b9b83c84ff234332785d9`
- DMG: `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.dmg` — 196950473 bytes — `sha256:695ffe13980e185ac8f4852635147d5ddf45eefebac6903294dbb4962960371f`
- ZIP: `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.zip` — 197394273 bytes — `sha256:8db709695da85299c10e1155df2f798dd4d9325422f2f0c927101b199fb2052a`

## Packaged smoke boundary
The GitHub-hosted macOS runner launched the packaged executable but did not emit the renderer UI report within the bounded probe. The diagnostic recorded `manualValidationRequired:true`. This is not an automated UI PASS and is consistent with the original requirement to hand final product testing to humans.

## Remaining acceptance
No automated delivery blocker remains for GBR-006. GBR-007 owns installed-product validation of UI, Agent behavior, local Computer/Browser, Marketplace/plugins, Teach/sharing, approvals, cancellation and error recovery. Any product-code defect invalidates this candidate and requires a new immutable SHA + source gate + Mac release.
