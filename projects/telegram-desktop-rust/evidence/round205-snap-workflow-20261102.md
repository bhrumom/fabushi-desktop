# Round 205 evidence — fixed-U repository Linux Snap build/artifact workflow

Date: 2026-11-02
Task round: 174
Desktop input checkpoint: `0e4bf048bcb1c726fc3c6e7033682175c6ad9225`
Spec-first checkpoint: `1ee04b2304a54fb429566363846295c4938a309c`
Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
Platform-core: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, non-truncated, 7,194 entries
Source: `.github/workflows/snap.yml@82f85ae9066a2773512afecb0bcf80875116a8c8`
Tree index: 130 zero-based
Responsibility: `TDRP-R9-REPOSITORY-SNAP-WORKFLOW-001`

The 2,633-byte ASCII workflow was read through EOF: 92 LF, zero CR, trailing newline. It admits push excluding public/private canary and pull_request under its path filters, conditionally selects Depot/Ubuntu runners, performs full-history recursive checkout, mutates FORWARD policy, installs classic floating Snapcraft, initializes LXD, executes an exact-pinned privileged disk/LVM action, builds from `snap/snapcraft.yaml@a181d828f25d901303f7221824ac30b0ff5e773a`, stages `telegram-desktop_*.snap`, and uploads a one-day GitHub Actions artifact.

Observed 2026-11-02 Action provenance: checkout `v7 -> 3d3c42e5aac5ba805825da76410c181273ba90b1`, `action.yml@5b0524f730db83f9513c18ab31a6c086c7239076`, Node24 bundle `06ae5d221b3dc83259d396ec60027972181e51b9`, MIT; exact more-space `97048bd0df83fb05b5257887bdbaffc848887673`, `action.yml@da03cfe57dd8f666f4b51c4397f076eb7354866c`, executable `get-more-space.sh@6a1ab22d291581113140a970d026c444a265389e`, LGPL-2.1; upload-artifact `v7 -> cf430e030ddbb5b0abf93d22962f4752f3646cd9`, `action.yml@7cb4d1e81db55320b41217e1a78a1a46e3d2baef`, Node24 upload bundle `7b3f92ae681d6eeca5f5e5e4cf892ad0dfb254ea`, MIT. Major tags, runner and Snapcraft acquisition remain mutable candidate-time evidence.

The manifest dependency was read for this workflow contract, without marking its independent source entry closed. Relevant graph includes core24/stable/strict, CMake/Ninja Release, Rust 1.96.1, six exact external source commits, additional tag/branch sources and mutable system packages. Telegram API build literals are not Snap Store credentials and sensitive-looking values are omitted from durable evidence.

There is no explicit workflow cache, Snap Store upload/release/login, store credential/secret input, signing/attestation, installation verification or runtime acceptance. GitHub artifact publication therefore cannot satisfy Store or Fabushi release acceptance. Failure/security boundaries preserve PR-controlled source, persisted checkout credentials, recursive submodules, mutable runner/action/tool/package identities, classic Snapcraft, LXD/firewall privilege, destructive disk defaults, no concurrency/retry/rollback, and artifact-glob zero/multiple-match behavior.

Existing-owner-first uses Fabushi repository build/toolchain + canonical package/release/process + security/provenance + shared quality/evidence governance. No target Telegram Snap workflow, second packaging truth, product runtime or platform-core delta was introduced.

Authored only and not run: 34 contract, 1 integration, 20 fault/recovery, 18 security/provenance cases. Stage A remains `implemented / validation_status=not-run / accepted=false`.

Ledger closure: source entries total 16,202; unknown 16,104 -> 16,103; unread 16,077 -> 16,076; omitted 0; responsibilities 174 -> 175; unmapped responsibilities 0. Next fixed-U entry: `.github/workflows/stale.yml@0d57eb7dbd9a9f758e1e33d45afbd1aefd212614`, tree index 131 zero-based.

Pre-existing global authority gap remains: active fixed-U ledger/closure uses `f985723...`, while upstream.lock/inventory metadata still records older `351204f...`. No Actions/check/status/workflow-run query or trigger and no executable validation/build/package/artifact/store/deployment operation was performed.
