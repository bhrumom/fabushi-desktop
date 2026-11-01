# Round 197 — fixed-U Docker build/publish workflow source closure (2026-11-01)

- Task round: 166
- Desktop checkpoint parent: `cba7dfef90aaa892c9445b60ce299f24edd24b24`
- Spec-first CAS: `7def0bb02af65cd7437e0ac5820dbaa490f52fe8`
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries
- Source: `.github/workflows/docker.yml@cba00688d935d50454bfb891fc37e001626cf9a0`
- Tree index: 122 zero-based; mode 100644; 1,121 ASCII bytes/code points; 42 LF; 0 CR; trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-DOCKER-BUILD-WORKFLOW-001`
- Validation: `not-run`; accepted: false

Source closure confirms path-scoped push admission plus default-branch job gating, implicit permissions, mutable ubuntu-latest, mutable checkout v7 with recursive submodules, unpinned apt/Poetry bootstrap, GITHUB_TOKEN GHCR login, pinned free-disk-space cleanup, exact fixed-U Poetry/Jinja generator inputs, empty DEBUG/LTO rendering false, generated stdin Dockerfile BuildKit build, mutable `:latest` push and no digest/SBOM/attestation/signature handoff.

Observed checkout v7 resolves to `3d3c42e5aac5ba805825da76410c181273ba90b1`; this remains an observation rather than a source pin. Existing `build-time-acquisitions.json` remains the single dependency authority for the Dockerfile graph. Exact fixed-U `.oss-scanner/Dockerfile@0ba260f72ad88b10a0d613ff5027b7a7640be7f2` is the direct published-image consumer and uses `ghcr.io/telegramdesktop/tdesktop/centos_env:latest`, making mutable-tag overwrite a fail-closed provenance boundary.

Existing-owner-first disposition is repository container/build/toolchain + release/process + security/provenance + shared quality/evidence. No target Telegram Docker workflow, second container runtime/build/release truth, product runtime, or platform-core delta. Authored only: 16 contract, 1 integration, 9 fault/recovery and 9 security/provenance cases. None was executed.

Ledger delta: source_entries_total=16202; unknown 16112->16111; unread 16085->16084; omitted=0; responsibilities_total 166->167; unmapped_responsibilities=0. Next independent entry is `.github/workflows/issue_closer.yml@355405165590712b1826b2f92a8f867c8d568dd9`, tree index 123 zero-based.

No Actions/check/status/workflow-run query or trigger; no push/dispatch, action/tool execution, submodule fetch, apt/curl/Poetry, Docker/BuildKit, registry mutation, build, validator, test/E2E, package, signing, release, deploy, installation or runtime acceptance occurred.
