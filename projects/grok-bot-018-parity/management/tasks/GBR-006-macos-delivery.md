# GBR-006 — macOS Actions package and prerelease delivery

Status: PASS for automated delivery / human installation acceptance delegated to GBR-007

## Objective
Build the completed macOS parity candidate in GitHub Actions, publish a prerelease from the exact accepted branch SHA, verify the tag resolves to that exact SHA, then stop before merge for the user-requested human test round.

## Final frozen candidate
- version: `2.0.0-alpha.2`
- code SHA: `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`
- source gate: Run `35417698506` — SUCCESS
- runtime contracts: **103 / 103 PASS**
- pinned renderer/source parity: PASS
- pinned reference source typecheck: PASS
- TypeScript/Vite production build: PASS

## Final package evidence
Run `35417698507` completed SUCCESS on the exact same SHA.
- package unsigned macOS test build — success
- packaged Grok renderer/local bridge smoke — success
- artifact upload — success
- release branch still exact build SHA — success
- prerelease publish — success
- published tag exact build SHA — success

Artifact:
- id: `10576418103`
- name: `fabushi-grok-parity-macos`
- size: `393022970` bytes
- digest: `sha256:6d3bf39199c312e695a6ab59cb7eb7cf408df6ab2ff34dee1a7076eb91406b2c`

Release:
- tag: `grok-parity-mac-92`
- release: `https://github.com/bhrumom/fabushi-desktop/releases/tag/grok-parity-mac-92`
- tag SHA: `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`
- DMG: `fabushi-grok-parity-2.0.0-alpha.2-macos-arm64.dmg` — 196953101 bytes
- ZIP: `fabushi-grok-parity-2.0.0-alpha.2-macos-arm64.zip` — 197395586 bytes

## Merge / human-test boundary
The original request explicitly delegates product testing to a human. Therefore PR #1 remains open after the exact-SHA prerelease instead of merging untested product code to main. GBR-007 is the only remaining acceptance item: install this released candidate on a real Mac and exercise visual/interaction parity, signed-in inference, macOS permissions, local Computer/Browser actions, plugins/Marketplace and Teach recording. Any discovered defect should be fixed on this PR and produce a newer exact-SHA candidate before merge.
