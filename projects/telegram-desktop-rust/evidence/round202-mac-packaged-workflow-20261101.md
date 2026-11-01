# Round 202 — fixed-U repository macOS deployed-app artifact workflow source closure (2026-11-01)

- Task round: 171
- Desktop checkpoint parent: `81d4ab04c253bd02c033a4eaea2d1ad5556afe52`
- Spec-first CAS: `65b872252bb463283bb28d6682a574063776c3e0`
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries
- Source: `.github/workflows/mac_packaged.yml@6e9f0b5bddfc3146bbed0bcd56a465bd584b89fc`
- Tree index: 127 zero-based; mode 100644; 9,952 ASCII bytes/code points; 254 LF; 0 CR; trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-MAC-PACKAGED-WORKFLOW-001`
- Validation: `not-run`; accepted: false

Complete source/dependency semantic review records push/PR/canary/path admission, implicit permission boundary, mutable conditional runner selection, recursive checkout, Homebrew/Xcode/CMake host mutation, workflow cache-key preimage, GitHub-token master-ref lookups, Rust 1.96.1 and exact patches/tlottie/wallet-engine/TDE2E source pins, mutable rnnoise/tg_owt caches/builds, local-prefix composition, Debug CMake build, `TDESKTOP_API_TEST=ON` without a test command, macdeployqt, explicit app signature removal, Telegram.app-only staging and one-day artifact upload.

Pinned source commits were API-verified: `tdlib/td@51743dfd01dff6179e2d8f7095729caa4e2222e9`, `dkaraush/tlottie@92df98dc209bc39b1e567ec74a8c86a0af5239de`, `i582/wallet-engine@e59e0d89d7ee90c388bf36e3334c5b276f396697`, `desktop-app/patches@aec474953ff7ee9b6e4cd9b8658288ea86d124f3`. Mutable master observations were rnnoise `6cbfd53eb348a8d394e0757b4025c6ded34eb2b6` and tg_owt `020feec5c040fdae2cf752c2c2be62ddde7c236d`; the workflow does not reset clones to those lookup SHAs, so lookup-to-clone drift is a recorded provenance failure mode.

Existing-owner-first maps the responsibility to Fabushi repository build/toolchain + canonical macOS package/release/process + repository security/provenance + shared quality/evidence governance. Canonical release surfaces remain `release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643`, `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84`, and `desktop/scripts/notarize-after-sign.cjs@5f140c6ff8587b4969e1611ec9030a8c5dc5a6f1`. No Telegram packaged workflow, second release truth, runtime/provider/sidecar or platform-core delta was created.

All 20 fixed-U workflow blobs were inspected for direct coupling: none invokes `mac_packaged.yml`; no workflow_call/workflow_run edge exists. `mac.yml` is an independent sibling; canary independently invokes prepare/mac.sh.

Authored only: 36 contract, 1 integration, 22 fault/recovery and 18 security/provenance cases. None was executed.

Ledger delta: source_entries_total=16202; unknown 16107->16106; unread 16080->16079; omitted=0; responsibilities_total 171->172; unmapped_responsibilities=0. Next independent entry is `.github/workflows/master_updater.yml@fedc24f8e5d781a0bbdcdf80e24baf3eb2fe199f`, tree index 128 zero-based.

Pre-existing authority gap remains explicit: active fixed-U ledger/closure is `f985723...`, while `upstream.lock.json` / inventory metadata still contains older `351204f...` identity. This single-entry closure does not claim baseline reconciliation.

No Actions/check/status/workflow-run query or trigger; no workflow/Action/cache/bootstrap/token probe/submodule/network acquisition, host mutation, dependency preparation, build/test, artifact transfer, package/sign/notarize, release/deployment/install/runtime operation occurred.
