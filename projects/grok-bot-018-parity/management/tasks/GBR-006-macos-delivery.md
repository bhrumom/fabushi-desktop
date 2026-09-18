# GBR-006 — macOS Actions package and prerelease delivery

Status: package mechanism verified / canonical prerelease gated on A8

## Objective
Build the macOS application in GitHub Actions, and only after parity acceptance merge the exact accepted PR head to canonical main and publish DMG/ZIP prerelease assets from that canonical SHA.

## Verified package evidence
Run `35323112810` completed successfully on branch SHA `5c3480a15003c145d26313f990e3f433cf8d7fdc`.
- job: `macos-package` — success
- install dependencies — success
- typecheck and build — success
- unsigned macOS package — success
- upload artifact — success
- prerelease step — skipped, correctly, because the run was not canonical main
- artifact id: `10538065159`
- artifact name: `fabushi-grok-parity-macos`
- size: `241686922` bytes
- digest: `sha256:884a992b9cfa2cf16891ddee05939a5daa52bc2343f8ac92e6fa0c30d9d4adae`

This corrects the previous stale "run pending" record. The run proves the package mechanism, not final parity or release.

## Packaging gate
Commit `9e6134d0a4aed9ba253aaffa77b9fcc9cd9093a9` removed automatic PR packaging. The macOS package workflow now runs only by explicit workflow dispatch or canonical-main push. A separate non-packaging source check validates implementation commits.

## Remaining acceptance
1. A8 must become PASS with no recoverable reference gaps.
2. Re-run macOS packaging on the exact accepted PR head; build/typecheck/package must be green.
3. Merge PR #1 only after that exact-head package evidence.
4. Re-read canonical main and verify it contains the accepted head.
5. Let canonical-main delivery create DMG/ZIP prerelease assets and record release URL/assets/digests.
6. Stop at the prerelease for external human installation testing.

## Current blocker
A8 is NOT COMPLETE, so no exact-head package, merge, or prerelease is permitted yet.
