# macOS packaged CI supply-chain workflow complete read — strict index 89

## Authority
- Upstream: `telegramdesktop/tdesktop@52a47639d9aa38713098974dd2ce07e631cb3381`, tree `42998681c09cdffcf75c0f38df71f663fc7b7b28`.
- Source: `.github/workflows/mac_packaged.yml@6e9f0b5bddfc3146bbed0bcd56a465bd584b89fc`, 254 line slots / 9952 bytes.
- Parent exact head/tree: `7ec8ba94ecce7c73ce1e4e64094d48078390f6ff` / `d206081a5f2e509eade81f3082e2e832b28e8c6e`.
- Parent exact-head 7ec8ba94ecce7c73ce1e4e64094d48078390f6ff completed TDRP run 39806248818 success (authority 12437977489 sha256:0aca8417897124c7f5fa2a1b1a3347dffc1cee51c7fc73689b6ade19c769acb0; closure 12438127803 sha256:7bb5990fe69b00d5da3af769768e1c4a5cd08e5b347b2322e00a25c3a3e4f126; reachability 12438487569 sha256:d4e55dc63e2fa762e10d0e02de9e2b62d2206dd16fc3571a233369d79b779aab) and Rust desktop runtime run 39806249072 at 6/6 success (artifact 12438301671 sha256:02dbda3cd8159e572b1b5b4f49e45d751a09bea68a4198dd129b90abe48b28dc).

## Complete-read behavior
The workflow is a macOS packaged-build CI adapter. It chooses Depot or GitHub macOS runners, installs mutable Homebrew/Xcode tooling, and builds a dependency stack before building Telegram Desktop with `TDESKTOP_API_TEST=ON`. The final application is run through `macdeployqt`, its signature is explicitly removed with `codesign --remove-signature`, and the result is uploaded for one day.

Four important dependency families are exact-pinned in workflow environment variables: `TDE2E=51743dfd...`, `TLOTTIE=92df98dc...`, `WALLET_ENGINE=e59e0d89...`, and `RUST_PATCHES=aec47495...`. Rust toolchain `1.96.1` is named, but its installer is downloaded from `sh.rustup.rs` during the run.

RNNoise and WebRTC/tg_owt are materially different. The setup step queries each repository's current `master` SHA and puts that SHA into the cache key. The build steps later execute fresh shallow `git clone` commands against `master` and never reset to the previously observed SHA. Therefore the observed SHA protects only the cache key, not the source actually built on a cache miss. A master move between lookup and clone is a real TOCTOU source-identity race and must remain fail-closed provenance.

## Exact dependency and license observations
- `desktop-app/patches@aec474953ff7ee9b6e4cd9b8658288ea86d124f3`; applied `wallet-engine.patch@7d8af1c1197589cf46a0f9b752269b3c31bf062a`. Common root LICENSE/COPYING names were not found in the checked paths, so no repository-wide license is invented here.
- `dkaraush/tlottie@92df98dc209bc39b1e567ec74a8c86a0af5239de`; `LICENSE@0f4cd8dc25aacf2ef7d2137b03ce1a21cd870e62`, MIT.
- `i582/wallet-engine@e59e0d89d7ee90c388bf36e3334c5b276f396697`; `Cargo.toml@bb73d077a0b309f79a8133b68c7ea04846c19b82`, `license = "MIT OR Apache-2.0"`.
- `tdlib/td@51743dfd01dff6179e2d8f7095729caa4e2222e9`; `LICENSE_1_0.txt@36b7cd93cdfbac762f5be4c6ce276df2ea6305c2`, Boost-1.0.
- Audit-time `xiph/rnnoise:master=6cbfd53eb348a8d394e0757b4025c6ded34eb2b6`; `COPYING@01ea4b1a2ac025fd9daa73a031b06c01fc57ed81`. This is not a checkout pin.
- Audit-time `desktop-app/tg_owt:master=07fb35bc33b3e79125a1beab7690e830f183f5d8`; `LICENSE@4c41b7b251e2f7606c70fad48eb92c8a2bc8e357`. This is not a checkout pin.

## Existing-owner-first disposition
Fabushi already has source-neutral dependency/supply-chain provenance, exact-head CI evidence, packaging/signing and release acceptance owners. The workflow maps into those owners. It does not justify a Telegram package/runtime owner, a second release root, or acceptance from an unsigned upstream artifact.

The selected canonical target is the existing AGENTS release boundary: `Actual Fabushi release remains gated by the applicable project spec, FQT-REL-001, exact artifact provenance, zero blocking defects, complete RTM/evidence, and independent acceptance; preparation must never turn a merely green build or artifact into release ACCEPT.`

## Fail-closed conclusions
- A cache-key SHA is not built-source identity when a later clone re-resolves a mutable branch.
- Rust toolchain version does not pin mutable installer bytes.
- Homebrew/Xcode/runner images are mutable environment provenance.
- Cache hits do not prove source, license, correctness, security or acceptance.
- The final upstream app is explicitly signature-stripped; it cannot satisfy Fabushi signing/notarization/package acceptance.
- Cancelled, failed, stale-head, missing-output, expired-artifact or unresolved-mutable-acquisition evidence is non-passing.

## Exact traceability
- Requirement: `TDRP-R9-MACOS-PACKAGED-CI-SUPPLY-CHAIN-PROVENANCE-001`.
- Oracles: `ORA-TDRP-MACOS-PACKAGED-EXACT-PINS-001`, `ORA-TDRP-MACOS-PACKAGED-MASTER-TOCTOU-001`, `ORA-TDRP-MACOS-PACKAGED-UNSIGNED-ARTIFACT-001`.
- Invariants: `INV-TDRP-MACOS-PACKAGED-CACHE-NO-SOURCE-AUTHORITY-001`, `INV-TDRP-MACOS-PACKAGED-MASTER-LOOKUP-NOT-CHECKOUT-PIN-001`, `INV-TDRP-MACOS-PACKAGED-NO-RELEASE-AUTHORITY-001`.
- Contracts: `CONTRACT-TDRP-MACOS-PACKAGED-SUPPLY-CHAIN-001`, `CONTRACT-TDRP-MACOS-PACKAGED-FABUSHI-RELEASE-BOUNDARY-001`.
- Temporal/fault: `TEMP-TDRP-MACOS-PACKAGED-RNNOISE-MASTER-RACE-001`, `TEMP-TDRP-MACOS-PACKAGED-WEBRTC-MASTER-RACE-001`, `TEMP-TDRP-MACOS-PACKAGED-RUNNER-TOOLCHAIN-DRIFT-001`, `FAULT-TDRP-MACOS-PACKAGED-CACHE-STALENESS-001`, `FAULT-TDRP-MACOS-PACKAGED-PATCH-APPLY-001`, `FAULT-TDRP-MACOS-PACKAGED-ARTIFACT-001`.
- Security: `SEC-TDRP-MACOS-PACKAGED-SUPPLY-CHAIN-001`, `SEC-TDRP-MACOS-PACKAGED-UNSIGNED-BOUNDARY-001`.

Closing strict index 89 changes counters to unknown=16,279, unread=16,254, omitted=0, responsibilities=107. `baseline_identity_accepted=false` and `source_closure_ready=false`. Next strict root is index 90 `.github/workflows/master_updater.yml@fedc24f8e5d781a0bbdcdf80e24baf3eb2fe199f`.
