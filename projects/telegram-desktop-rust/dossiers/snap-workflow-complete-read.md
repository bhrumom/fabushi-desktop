# Complete read — fixed-U repository Linux Snap build/artifact workflow

- Date: 2026-11-02
- Task round: 174
- Fixed upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source: `.github/workflows/snap.yml@82f85ae9066a2773512afecb0bcf80875116a8c8`
- Tree index: 130 zero-based
- Mode/size: 100644 / 2,633 ASCII bytes/code points / 92 LF / 0 CR / trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-SNAP-WORKFLOW-001`
- Disposition: existing-owner-first repository Linux build/package/release/process + security/provenance + quality/evidence; executable validation deferred.

## Trigger, permissions and runner

Push excludes `public-canary` and `private-canary`; push and pull_request use the same path-ignore set and exact exception for this workflow. There is no schedule, workflow_dispatch, repository_dispatch, workflow_call or workflow_run. There is no explicit permissions, environment, concurrency, timeout, retry or cache declaration.

The sole job chooses `depot-ubuntu-latest-16` for pull requests, tag-like refs and nightly, otherwise `ubuntu-latest`. Runner image/tool versions remain mutable Stage-B evidence.

## Checkout and host toolchain

Checkout is `actions/checkout@v7`, `fetch-depth: 0`, recursive submodules. Observed 2026-11-02 tag resolution: commit `3d3c42e5aac5ba805825da76410c181273ba90b1`, action metadata `5b0524f730db83f9513c18ab31a6c086c7239076`, Node24 bundle `06ae5d221b3dc83259d396ec60027972181e51b9`, MIT. Major tag and runner remain mutable; checkout defaults persist credentials until post cleanup.

Setup changes iptables FORWARD to ACCEPT, installs unversioned classic Snapcraft from the Snap Store, adds the runner user to LXD, initializes LXD automatically and waits ready. No tool version/digest is source-pinned.

Exact `samueldr/more-space-action@97048bd0df83fb05b5257887bdbaffc848887673` uses `action.yml@da03cfe57dd8f666f4b51c4397f076eb7354866c` and `get-more-space.sh@6a1ab22d291581113140a970d026c444a265389e`; repository metadata reports LGPL-2.1. The workflow disables only default apt-pattern removal and enables an LVM span on the LXD container-storage mount. Other defaults can prune Docker, remove filesystem/home content and `/usr/local`, alter secondary-disk/swap state, format/mount storage and build LVM. LVM creation is hard-fail; many cleanup commands are intentionally tolerant.

## Snapcraft build/package input

Build command is `sudo -u $USER snap run snapcraft --verbosity=debug`. Direct build input is fixed-U `snap/snapcraft.yaml@a181d828f25d901303f7221824ac30b0ff5e773a`; this workflow closure does not close that later independent source entry.

Manifest summary: core24, stable grade, strict confinement, lzo compression; desktop/autostart and GNOME extension; audio/camera/network/home/removable-media/U2F and related plugs plus MPRIS slot; Release CMake/Ninja build; Rust 1.96.1 local Rust aggregation; build/stage system packages; patches and external sources. Exact source commits include patches `845a383f7abe8e44f92dfdcd566d603066ec2696`, zlib `e3dc0a85b7032e98380dec011bc8f2c2ee0d8fca`, TD E2E `51743dfd01dff6179e2d8f7095729caa4e2222e9`, tlottie `92df98dc209bc39b1e567ec74a8c86a0af5239de`, wallet-engine `e59e0d89d7ee90c388bf36e3334c5b276f396697`, tg_owt `07fb35bc33b3e79125a1beab7690e830f183f5d8`. Other manifest sources are tag/branch references (Ada v3.2.9, libavif v1.3.0, FFmpeg n6.1.6, mozjpeg v4.1.5, OpenAL 1.24.3, OpenSSL 3.5.9, nv-codec-headers n12.1.14.0, Qt v6.11.2, RNNoise v0.2) and remain candidate-time provenance boundaries. Apt/Snap archive package revisions are likewise environment inputs.

The manifest contains upstream Telegram API compile configuration. It is not a Snap Store credential and is not copied into Fabushi; sensitive-looking values are deliberately omitted from this dossier.

## Cache, artifact, store publication and credentials

No GitHub cache action/key exists. External runner/Snapcraft/LXD/package caches are not source-defined cache authority.

After build, `artifact_name=$(echo telegram-desktop_*.snap)` is appended to `GITHUB_ENV`; the expansion is moved into `artifact/`. Zero matches leave a literal glob and fail move; multiple matches can produce a multi-token artifact name and multi-file move, so exact cardinality is a Stage-B oracle.

Upload uses `actions/upload-artifact@v7`, one-day retention. Observed resolution is commit `cf430e030ddbb5b0abf93d22962f4752f3646cd9`, action metadata `7cb4d1e81db55320b41217e1a78a1a46e3d2baef`, Node24 bundle `7b3f92ae681d6eeca5f5e5e4cf892ad0dfb254ea`, MIT. There is no Snap Store upload/release/login, store secret/credential, signing, attestation, installation verification or runtime test. A GitHub artifact is not store publication or release acceptance.

## Callers, consumers, failure and security

Direct callers are GitHub push/pull_request admission. Direct consumers are checkout/submodule fetch, runner host/network/firewall/LXD, more-space, Snapcraft and the manifest dependency graph, GitHub artifact storage and maintainers/manual artifact users. No reusable caller exists because the workflow declares no `workflow_call`.

Fail-closed boundaries include runner/action/tag drift; recursive submodule and persisted-checkout credential trust; PR-controlled source; classic unpinned Snapcraft acquisition; host firewall/LXD/disk mutation; mutable tag/branch and system-package inputs in the manifest; no source-defined cache/reproducibility contract; Snap build failure; glob zero/multiple matches; partial artifact staging; artifact upload failure; no concurrency/rollback; and any attempt to infer Snap Store publication from a one-day artifact.

## Target ownership and authored verification

Existing owner: Fabushi repository build/toolchain + canonical package/release/process + security/provenance + shared Test Strategy/Test Evidence Contract/FMEP-001. Rejected: copied Telegram Snap workflow, Telegram-branded Snap subsystem, second package/release truth, product Tasks/Automations, Agent/Coordinator/Host/Runner runtime, platform-core service or second quality/security truth. No product shipping code or platform-core production delta is required for this repository-host workflow provenance responsibility.

Authored only, not run: 34 contract, 1 integration, 20 fault/recovery and 18 security/provenance cases. Stage A remains `implemented / validation_status=not-run / accepted=false`. No workflow, third-party Action, Snap/Snapcraft/LXD/iptables/disk operation, build, test, package execution, artifact transfer, store publication, signing, deployment, installation, runtime acceptance or Actions/check/status/workflow-run query was performed.
