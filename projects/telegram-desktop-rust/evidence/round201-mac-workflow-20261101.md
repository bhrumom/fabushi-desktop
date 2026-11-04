# Round 201 — fixed-U repository macOS CI build/artifact workflow source closure (2026-11-01)

- Task round: 170
- Desktop checkpoint parent: `c51dc541e1987092e5424d5a9e9cb7a5cab88df1`
- Spec-first CAS: `c2d07d1b30f361789f5b8fe8d7e3e95b498f53e7`
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries
- Source: `.github/workflows/mac.yml@184a94b091d2d7bab19ae67d7f3a5881b18213ba`
- Tree index: 126 zero-based; mode 100644; 4,273 ASCII bytes/code points; 138 LF; 0 CR; trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-MAC-WORKFLOW-001`
- Validation: `not-run`; accepted: false

Complete source/dependency semantic review records push/PR admission, canary/path exclusions, mutable conditional runner selection, implicit permission boundary, observed checkout/cache/upload-artifact major-tag resolutions and MIT provenance, recursive submodules, Homebrew/Xcode/CMake host mutations, outer Libraries/ThirdParty cache, prepare.py per-stage cache validation, Qt 6.11.2/Rust 1.96.1 and fixed-U acquisition graph, destructive Libraries prune, exact configure/CMake build flags, signing-disabled Debug output, Telegram.app+Updater partial-staging boundary and one-day artifact publication.

Existing-owner-first maps the responsibility to Fabushi repository build/toolchain + canonical macOS package/release/process + repository security/provenance + shared quality/evidence governance. Canonical Fabushi release surfaces remain `release-macos-main.yml`, electron-builder package configuration, notarization/signing hook and Electron updater owners. No Telegram macOS workflow, second package/release truth, runtime/provider/sidecar or platform-core delta was created.

All 20 fixed-U workflow blobs were checked for direct coupling: no reusable/workflow-run caller invokes mac.yml; canary independently invokes prepare/mac.sh and mac_packaged.yml is a sibling workflow. Authored only: 28 contract, 1 integration, 16 fault/recovery and 14 security/provenance cases. None was executed.

Ledger delta: source_entries_total=16202; unknown 16108->16107; unread 16081->16080; omitted=0; responsibilities_total 170->171; unmapped_responsibilities=0. Next independent entry is `.github/workflows/mac_packaged.yml@6e9f0b5bddfc3146bbed0bcd56a465bd584b89fc`, tree index 127 zero-based.

Pre-existing authority gap remains explicit: active fixed-U ledger/closure is `f985723...`, while `upstream.lock.json` / inventory metadata still contains older `351204f...` rebaseline identity. This single-entry closure does not claim formal baseline reconciliation.

No Actions/check/status/workflow-run query or trigger; no workflow/Action/cache/submodule/host mutation/dependency-preparation/build/test/artifact/package/signing/notarization/release/deployment/installation/runtime operation occurred.
