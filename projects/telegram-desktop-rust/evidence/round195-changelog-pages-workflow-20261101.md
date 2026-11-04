# Round 195 — fixed-U changelog Pages workflow source closure (2026-11-01)

- Task round: 164
- Desktop checkpoint parent: `c45f37934de67cf82ad81dd71f216e0bfd6994cf`
- Spec-first CAS: `73955b844523111ab0a704ba56963ddfafc1a4d9`
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries
- Source: `.github/workflows/changelog.yml@445277fe4b44dc9ccef17b7b602016572c08fe7c`
- Tree index: 120 zero-based; mode 100644; 847 ASCII bytes/code points; 45 LF; 0 CR; trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-CHANGELOG-PAGES-WORKFLOW-001`
- Validation: `not-run`; accepted: false

The workflow was read through EOF. Admission is exact dev push + root changelog.txt path or manual dispatch; permissions are contents-read/pages-write/id-token-write at workflow scope; concurrency is group pages with cancellation enabled. Build uses ubuntu-latest, checkout@v7, setup-python@v7 with Python 3.12, the exact already-qualified generator, and whole-docs Pages artifact upload. Deploy requires build, targets the github-pages environment, and publishes the uniquely named github-pages artifact through deploy-pages.

Observed mutable tag resolutions were independently read: checkout v7 -> `3d3c42e5aac5ba805825da76410c181273ba90b1`; setup-python v7 -> `6d4fd22b20465ccc0f7354476588e8df7c9ca0e0`; upload-pages-artifact v5 -> `fc324d3547104276b827a68afc52ff2a11cc49c9`; deploy-pages v5 -> `368f82528645a54fb793d4d04e342629a3f51346`. Upload-pages-artifact internally pins upload-artifact `bbbca2ddaa5d8feaa63e36b76fdaad77386f024f`. These are observations, not fixed-U pins.

Fixed-U docs has 14 entries / 12 blobs / zero tracked symlinks; upload archives with dereference/hard-dereference and one-day retention. Deploy obtains OIDC, resolves exactly one artifact, creates the Pages deployment and polls bounded status; timeout/error-threshold/signals explicitly cancel a pending deployment.

Security boundaries remain fail-closed: workflow-level pages/id-token write reaches build, checkout persists credentials by default, source-controlled Python executes after checkout, runner/Python/action tags are mutable, artifact scope is whole docs, and GitHub Pages host/environment/OIDC configuration is external later evidence. Upstream Pages success does not become Fabushi release acceptance.

Existing-owner-first maps this repository-only responsibility to root release/changelog/process governance, repository security/provenance governance, shared Test Strategy/Test Evidence Contract and FQT-REL-001; current release-macos-main/package owners are context only. No target Pages workflow, product runtime or platform-core delta is introduced.

Authored only: 17 contract, 1 integration, 8 fault/recovery and 7 security/provenance cases. None was executed.

Ledger delta: source_entries_total=16202; unknown 16114->16113; unread 16087->16086; omitted=0; responsibilities_total 164->165; unmapped_responsibilities=0. Next independent entry is `.github/workflows/copyright_year_updater.yml@7ddb3d0403a77812df954d8bb2f480672831a082`, tree index 121 zero-based.

No Actions/check/status/workflow-run query or trigger; no workflow push/dispatch, action execution, generator, artifact upload, OIDC/Pages deployment, build, validator, test/E2E, package, signing, release, installation or runtime acceptance occurred.
