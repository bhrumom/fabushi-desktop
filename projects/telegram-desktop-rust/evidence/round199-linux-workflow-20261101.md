# Round 199 — fixed-U repository Linux CI build/artifact workflow source closure (2026-11-01)

- Task round: 168
- Desktop checkpoint parent: `937cb2a0f8a642ed3370462d97f48c3feae58489`
- Spec-first CAS: `c8ecf40d70d325b9fdde6b283ce31aae3b6716f0`
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries
- Source: `.github/workflows/linux.yml@dbedd23593c5ea66ff9b8b880bda6a8bc5a4cc55`
- Tree index: 124 zero-based; mode 100644; 7,284 ASCII bytes/code points; 215 LF; 0 CR; trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-LINUX-WORKFLOW-001`
- Validation: `not-run`; accepted: false

Trigger/runner/source semantics, Action/tag lineage, mixed toolchain bootstrap, generated container inputs, Buildx/cache-dance/ccache lifecycle, exact build scripts/flags, existence-only output check, Telegram+Updater artifact staging, one-day artifact publication, direct callers/consumers, security/provenance and failure-recovery boundaries were read and recorded in the dossier and source-neutral contracts.

Observed mutable tag resolutions: checkout `3d3c42e5aac5ba805825da76410c181273ba90b1`; setup-buildx `f87e5991a6d7451dcb8d9637bfbc97413f497069`; actions/cache `55cc8345863c7cc4c66a329aec7e433d2d1c52a9`; build-push `c3c9e263c25d99ce0380d002d59b67737d91b0dc`; upload-artifact `cf430e030ddbb5b0abf93d22962f4752f3646cd9`. Exact cache-dance `5422eac04292c961a382e0f584ea0f03ad9da723` action metadata is `08ad2eef853162f5336e25928967d9a4e64c0c4e` and exposes mutable busybox helper-image provenance.

Source quality is intentionally weaker than Fabushi acceptance: Telegram regular-file existence only, size printed but not gated, no Updater precheck, no executable/runtime/package/sign/install validation. Uploaded Debug output is ephemeral and unsigned, with no source-captured digest/SBOM/attestation/release identity.

Existing-owner-first maps the responsibility to current Fabushi repository build/toolchain + package/release/process + security/provenance + shared quality/evidence governance. No Telegram workflow/runtime or platform-core delta was created.

Authored only: 28 contract, 1 integration, 16 fault/recovery and 14 security/provenance cases. None was executed.

Ledger delta: source_entries_total=16202; unknown 16110->16109; unread 16083->16082; omitted=0; responsibilities_total 168->169; unmapped_responsibilities=0. Next independent entry is `.github/workflows/lock.yml@7904e79591f79a4ea61c0dc194c189f048bf5c67`, tree index 125 zero-based.

Pre-existing authority gap remains explicit: active fixed-U ledger/closure is `f985723...`, while `upstream.lock.json` / inventory metadata still contains older `351204f...` rebaseline identity. This single-entry closure does not claim formal baseline reconciliation.

No Actions/check/status/workflow-run query or trigger; no workflow/action/bootstrap/cache/build/artifact operation, validator, test/E2E, package, signing, release, deployment, installation or runtime acceptance occurred.
