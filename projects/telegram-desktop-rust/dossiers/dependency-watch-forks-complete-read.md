# Dependency Watch patched-fork reference — complete read

Task round: 120  
Evidence round: 151  
Stage: A / executable validation deferred  
Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/dependency-watch/references/forks.md@29a7aa587ae26118a0611a2e4037a69141a3ffde`

## Exact source and direct consumer

The target blob was read completely: 4,581 bytes / 74 lines, fixed-U recursive tree index 60. It defines repository dependency-advisory research rules for patched forks and copied source trees. The only direct in-repository workflow consumer found is `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`, which says to read this reference when checking patched/copied upstreams and to always apply it to `tg_owt` and `tg_angle` when they occur in the snapshot. GitHub code search found no additional direct `forks.md` consumer. Exact fixed-U `watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38` and `watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea` contain no direct `forks`, `tg_owt`, or `tg_angle` reference; they own snapshot/compare mechanics, not this research policy.

## Contract read

A patched/copy dependency has three separate identities: the exact fork revision consumed by Telegram Desktop, the fork's current branch, and the original project's upstream. A simple fork-head movement can be a pin update; importing the original project can carry a large local patch set. Those states are reported separately.

For `tg_owt`, discovery starts from `desktop-app/tg_owt` versus WebRTC upstream. Import clues include source-import commits, Chromium milestone, DEPS, nested gitlinks, original revision and commit-position trailers. For `tg_angle`, discovery starts from `desktop-app/tg_angle` versus Chromium ANGLE upstream using source-import commits, milestone, version/commit metadata, DEPS and README.chromium. The reference explicitly says remembered M123/M115 subjects are discovery clues, not fixed baselines; every audit re-establishes the baseline from the exact consumed fork pin.

Comparable-history review reads fork history through the pinned revision, including source imports, submodule updates and later fixes, and separately inspects the pinned shared-patches revision for each recipe because prepare, Docker and Snap may differ. The original imported revision comes from provenance metadata, commit-position/Change-Id trailers or verified source equivalence. Original stable and development heads are compared separately. For flattened copies without shared ancestry, a verified import mapping is required; raw commit-count subtraction and a failed merge-base are not divergence proof. Approximate milestones remain explicitly approximate.

Missing-fix review considers the unimported original-upstream range, upstream/fork advisories, Chromium stable security notes, public issues and nested dependency security updates. A CVE that names WebRTC/ANGLE is only a lead. Applicability requires affected version, source presence, compiled backend/feature, build flags and reachable use. Features introduced after the import may be absent. Restricted issues or unavailable fixes remain unknown.

For each candidate, the fixing diff and prerequisites are inspected. Existing ancestry, equivalent patches/patch IDs and current semantics are considered; rewritten/cherry-picked local fixes can have different hashes and paths. Missing original ancestry does not prove that a vulnerability is unbackported, and a matching subject or an applicable patch does not prove that a fix is present or correct.

Security candidates are classified exactly as: confirmed applicable and missing; already present/backported with matching-code/patch evidence; not applicable with absent feature/backend/version evidence; or needs investigation because mapping, issue access or reachability is incomplete.

A confirmed missing fix yields an advisory route: update to an existing fork revision, import newer original upstream, or backport. A backport proposal names the original fix and prerequisites, affected fork paths, destination (fork or shared patches), likely local conflicts, platform scope and focused later regression/Debug validation. Broad-refactor prerequisites are called out instead of being labeled trivial. The monitor proposes; it does not apply.

Durable advisory evidence keeps mapped baselines, scanned ranges, stable/development heads, unresolved candidates and per-fix evidence. New upstream history is scanned daily and advisories are revisited even when fix commits predate the last scan. Unexamined old tail remains visible. Age alone belongs in Review someday; an identified missing applicable security fix belongs in Most pressing updates pending.

## Fixed-U pin and import-history evidence

Exact fixed-U consumer files were read:

- `Telegram/build/prepare/prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b` pins `tg_owt@07fb35bc33b3e79125a1beab7690e830f183f5d8` and `tg_angle@9fd174a91e6155d1006c5f427366b23006f5d899`.
- `Telegram/build/docker/centos_env/Dockerfile@3fea9e6866e0ea6351f8a13ebb825ebf49cadfc2` pins the same `tg_owt` revision and recursively initializes submodules.
- `snap/snapcraft.yaml@a181d828f25d901303f7221824ac30b0ff5e773a` pins the same `tg_owt` revision.
- The prepare file builds `tg_angle` only under the Windows condition; the Qt<6 branch explicitly links the static `tg_angle` output. Exposure therefore follows actual Qt/build selection and cannot be inferred from repository presence alone.

The consumed `tg_owt` pin is a fork commit dated 2026-10-13 whose message says it backports WebRTC `7e9559f8b08e61b4cf065ad891f132bccd580fc5`. Fork search found import anchor `5b6a5c00017c115f6a54b90b109354abc1444a01` (2024-07-16, "Update: Copy updated sources (M123+patches)."). GitHub compare establishes that the consumed pin descends from that anchor and is 102 fork commits ahead. The current pinned tree has recursive gitlinks including abseil, crc32c, libsrtp and libyuv, which confirms why nested provenance is part of the fork contract.

The consumed `tg_angle` pin is a fork commit dated 2026-10-21 ("Preserve packed uniforms after matrix uploads."). Fork search found import anchor `3e2cbd93cb60a4f677af68bac4e3849176f688fa` (2023-08-10, "Update source code to M115."). GitHub compare establishes that the consumed pin descends from that anchor and is 8 fork commits ahead. These in-fork comparisons do not identify exact original WebRTC/ANGLE lag; the M123/M115 labels remain approximate upstream mapping clues until original provenance is verified.

The existing Fabushi build-time acquisition inventory `projects/telegram-desktop-rust/inventory/build-time-acquisitions.json@e9c14ce86f8e8be5fa1231fabd47147c94ee887f` contains `tg_owt` and its exact pin/recursive children, but contains no `tg_angle`, `desktop-app/tg_angle`, or `9fd174a91e6155d1006c5f427366b23006f5d899`. This is recorded as an evidence-input gap, not as absence/non-applicability. It reinforces the existing contract that dependency candidates/inventories are navigation evidence and the advisory owner must inspect the full source union.

## Existing-owner-first resolution

No new runtime, provider, task owner, updater or fork-monitor state store is introduced. This reference specializes the already implemented `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` responsibility under existing Fabushi repository quality/test-evidence governance.

Rejected owners remain:
- `source/electron-main/update/*`: product update/install/rollback, not source dependency advisory state.
- platform-core runtime: no repository dependency-advisory lifecycle.
- build-time acquisition inventory as decision authority: evidence input only, and the current `tg_angle` omission demonstrates why it cannot be the complete audit truth.
- Product Tasks/Automations or Agent/Coordinator/Host/Runner: shipping product execution state, not repository supply-chain advisory governance.
- copied Telegram-specific Python/fork monitor: would create a second source-specific owner.

Exact fixed-U `telegramdesktop/tdesktop:AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32` separately confirms that the scheduled dependency monitor remains advisory while actual dependency backports are explicit implementation requests.

## Authored validation, not run

- `CONTRACT-TDRP-DEPENDENCY-FORK-IMPORT-HISTORY-001`
- `CONTRACT-TDRP-DEPENDENCY-FORK-FIX-APPLICABILITY-001`

Both remain `validation_status=not-run`; no build, test, validator, dependency resolution, backport, package, deployment or runtime acceptance was executed.
