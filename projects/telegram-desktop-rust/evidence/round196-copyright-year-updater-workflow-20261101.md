# Round 196 — fixed-U copyright-year updater workflow source closure (2026-11-01)

- Task round: 165
- Desktop checkpoint parent: `f8a99e5ac3b98e8a7994cceeba7c716bfe03f6bd`
- Spec-first CAS: `de79b94e2b00963f2314bb6307606448f3ec77b3`
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries
- Source: `.github/workflows/copyright_year_updater.yml@7ddb3d0403a77812df954d8bb2f480672831a082`
- Tree index: 121 zero-based; mode 100644; 338 ASCII bytes/code points; 16 LF; 0 CR; trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-COPYRIGHT-YEAR-UPDATER-WORKFLOW-001`
- Validation: `not-run`; accepted: false

The workflow and its direct external Action implementation were read through the applicable license-year path. Triggers are exact repository-dispatch type plus annual January-1 03:00 UTC schedule; no fixed-U sibling workflow directly emits that dispatch or consumes license-year. YAML permissions are implicit host policy; the sole job is mutable ubuntu-slim and calls mutable desktop-app/action_code_updater@master.

Observed Action master on 2026-11-01 is `0a8d3ebb437e5c77cf5d066d4a54cc93d34b534a`, with action metadata `5a9c7dff415a31f18a614119809f2d396b0e47c7` and runtime `index.mjs@322bfa9e3e874299a09c35650611c2cd753cca3f`. Its composite runtime installs simple-git, isbinaryfile, @actions/core and @actions/github without versions/lock and passes github.token in argv. These are mutable observations, not fixed-U pins.

LicenseYear derives process-local current year, clones the then-current default branch, recursively scans non-.git working-tree paths, skips binary files, gates on the full lowercase copyright phrase, and performs one unanchored first literal numeric-range replacement per eligible file. Fixed U has zero tracked symlinks and 36 gitlinks; this mode does not recurse submodules. No-change is exit-0.

Changed files flow to year-derived branch, all-files staging, deterministic commit, push and then PR list/create. PR API exceptions are caught without rethrow after push, so branch-without-PR partial publication can look process-successful. Effective token scope, runtime npm identities, runner/timezone/default-branch identity and actual publication objects remain Stage-B evidence.

Existing-owner-first disposition is repository source-maintenance/process + security/provenance + shared quality/evidence; no target workflow/product runtime/platform-core delta. Authored only: 18 contract, 1 integration, 10 fault/recovery and 8 security/provenance cases. None was executed.

Ledger delta: source_entries_total=16202; unknown 16113->16112; unread 16086->16085; omitted=0; responsibilities_total 165->166; unmapped_responsibilities=0. Next independent entry is `.github/workflows/docker.yml@cba00688d935d50454bfb891fc37e001626cf9a0`, tree index 122 zero-based.

No Actions/check/status/workflow-run query or trigger; no schedule/dispatch, Action/npm execution, token probe, source mutation, branch/PR publication, build, validator, test/E2E, package, signing, release, installation or runtime acceptance occurred.
