# Snap packaged CI supply-chain workflow complete read — strict index 92

## Authority
- Upstream: `telegramdesktop/tdesktop@83f9f56347cc3421c1e3ef537c7bb6dd06ce7cd0`, tree `bb0159bfedb38dd96e579af4ffe6360b6beb7332`.
- Source: `.github/workflows/snap.yml@82f85ae9066a2773512afecb0bcf80875116a8c8`, 92 line slots / 2633 bytes; complete read and SHA revalidated before publication.
- Direct packaging input: `snap/snapcraft.yaml@a181d828f25d901303f7221824ac30b0ff5e773a`, 546 line slots / 16501 bytes; complete read. Existing build-time acquisition accounting remains authoritative for its exact commits and mutable tags/branches.
- Runner-space dependency: `samueldr/more-space-action@97048bd0df83fb05b5257887bdbaffc848887673`, tree `f1d999f8b4029093775fd5633a4fb58bb4a7129f`; `action.yml@da03cfe57dd8f666f4b51c4397f076eb7354866c`; `get-more-space.sh@6a1ab22d291581113140a970d026c444a265389e`; `COPYING@9c6a6cc381bf5682b57f63ec86f3b0532d56b01a` is LGPL-2.1 text.
- Existing release identity owner: `FQT-REL-001@c5eb9d0897234bb2fdad4f505549b2350c44aa72`; evidence owner: `FQT-EVID-001@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70`.
- Parent exact head/tree: `16fea2446f9fa18fb7a50872e1f070726511f2f6` / `adc56e986fb21407c42d5ebcc9c9f3e1169874d7`.
- Parent exact-head 16fea2446f9fa18fb7a50872e1f070726511f2f6 completed TDRP run 39887592363 at 61/61 success (authority 12473552243 sha256:a8eb257aaccd02523b6c9eae32f185bf1973fb551d50357950d4cae27e4f8f44; closure 12472944907 sha256:984f6d9c5209dbfd0c89b9a5ec41dd27430d7e55e866f5d5f0d559c8c258ac48; reachability 12473985264 sha256:b20ee7cfdcaf51148be6276ed13087ddfc84e40e739e7617c8f8b29a600958e0) and Rust desktop runtime run 39887592349 at 6/6 success (artifact 12473606144 sha256:74df451c6ba5f9ac07d27caeadea2660836ea0f8c72d611dd99200ac20d28042).

## Complete-read behavior
The workflow is Linux Snap packaged-CI provenance, not a Fabushi release authority. Push and pull-request filters exclude documentation and non-Linux platform surfaces while re-including the workflow itself. Pull requests, tags and nightly use `depot-ubuntu-latest-16`; other admitted refs use `ubuntu-latest`. Checkout is full-history with recursive submodules.

The setup step mutates only the disposable runner: forwarding is accepted, Snapcraft is installed with classic confinement, the runner user is added to LXD, and LXD is initialized. The exact-pinned `more-space-action` disables only default apt-pattern removal, explicitly enables an LVM span, and mounts that span at the LXD container storage pool. Provider defaults still enable Docker prune, non-package /usr cleanup, space/home-folder cleanup, /usr/local deletion, second-drive harmonization and swapfile recreation; the LVM path additionally creates sparse files, loop devices, PV/VG/LV state, an ext4 filesystem and a mount. None of this is product state or user-device acceptance evidence.

The actual build is `sudo -u $USER snap run snapcraft --verbosity=debug`; therefore `snap/snapcraft.yaml` is a direct build authority input. Its dependency source commits/tags/branches remain owned by the existing acquisition inventory rather than being duplicated in this responsibility.

After build the workflow glob-selects `telegram-desktop_*.snap`, moves it into `artifact/`, and uploads it for one day with the filename as the artifact name. It does not record raw SHA-256 and size for the produced package before/after the move, so green CI and filename-only upload do not satisfy Fabushi's artifact identity gate.

## Existing-owner-first disposition
Fabushi reuses current release-preparation/exact-head governance, build-time acquisition provenance, FQT-REL-001, FQT-EVID-001, and existing packaging/signing owners. `novel_capability=false`. No Snap product runtime, Linux release root, package-identity ledger, runner-storage product owner, or second acceptance authority is introduced.

## Fail-closed, concurrency and security conclusions
- Privileged runner deletion/storage/LVM/filesystem/swap/Docker operations are bounded to disposable CI infrastructure only.
- Snapcraft tag/branch acquisitions remain mutable until resolved by the existing source/acquisition authority.
- Filename and provider upload metadata are not byte identity; missing raw SHA-256 + size is BLOCKED for staging, promotion or release under FQT-REL-001.
- One-day artifact retention cannot replace durable FQT evidence identity.
- No workflow concurrency group exists; cancellation, late failure or partial provider state cannot advance release truth.
- Actual Fabushi release still requires exact source, applicable signing/package/device/function/UI/a11y/recovery/security evidence, zero blockers and independent ACCEPT.

## Exact traceability IDs
- Requirement: `TDRP-R9-SNAP-PACKAGED-CI-SUPPLY-CHAIN-PROVENANCE-001`
- Oracles: `ORA-TDRP-SNAP-WORKFLOW-EXACT-SOURCE-001`, `ORA-TDRP-SNAP-ARTIFACT-IDENTITY-GAP-001`, `ORA-TDRP-SNAP-RUNNER-MUTATION-BOUNDARY-001`
- Invariants: `INV-TDRP-SNAP-GREEN-NOT-RELEASE-001`, `INV-TDRP-SNAP-RUNNER-MUTATION-NOT-PRODUCT-STATE-001`, `INV-TDRP-SNAP-ARTIFACT-IDENTITY-FAIL-CLOSED-001`
- Contracts: `CONTRACT-TDRP-SNAP-PACKAGING-ADAPTER-001`, `CONTRACT-TDRP-SNAP-ARTIFACT-PROVENANCE-001`
- Temporal: `TEMP-TDRP-SNAP-BUILD-INPUT-DRIFT-001`
- Fault/recovery: `FAULT-TDRP-SNAP-LXD-SETUP-001`, `FAULT-TDRP-SNAP-MORE-SPACE-MUTATION-001`, `FAULT-TDRP-SNAP-ARTIFACT-NAME-GLOB-001`
- Security: `SEC-TDRP-SNAP-RUNNER-SUDO-001`, `SEC-TDRP-SNAP-SUPPLY-CHAIN-001`
- Regression: `REG-TDRP-FQT-REL-001-REMAINS-RELEASE-OWNER-001`, `REG-TDRP-SNAP-NO-PRODUCT-RUNTIME-001`

Closing strict index 92 changes counters to unknown=16,282, unread=16,257, omitted=0, responsibilities=110. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 93 `.github/workflows/stale.yml@0d57eb7dbd9a9f758e1e33d45afbd1aefd212614`.
