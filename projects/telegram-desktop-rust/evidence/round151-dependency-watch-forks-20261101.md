# Round 151 — Dependency Watch patched-fork provenance

Task round: 120  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/dependency-watch/references/forks.md@29a7aa587ae26118a0611a2e4037a69141a3ffde`

Live desktop checkpoint was reread at `5fac5bd06a06de45410f6173c0d444b9eb08602a`; spec-first CAS published `5c8958895b5596bdab43a0d635e1b69bee125c74`; desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed-U root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was reread complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `16202 / unknown=16158 / unread=16131 / omitted=0 / responsibilities=134 / unmapped=0`.

The target blob was completely read: 4,581 bytes / 74 lines, tree index 60. It requires patched/copied dependencies to preserve distinct consumed fork pin, fork current branch and original-upstream import provenance; to re-establish tg_owt/tg_angle baselines from exact consumed pins; to use verified ancestry/import mappings rather than raw commit-count subtraction for flattened copies; and to classify missing fixes only after inspecting fixing diffs, prerequisites, equivalent patch/semantic evidence, source/backend/build flags/reachability and platform scope.

Direct workflow consumer is dependency-watch `SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`. Exact fixed-U `watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38` and `watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea` contain no direct forks/tg_owt/tg_angle reference; they do not create a second fork-policy state owner.

Fixed-U consumer facts were reread: `prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b`, CentOS Docker `3fea9e6866e0ea6351f8a13ebb825ebf49cadfc2` and Snap `a181d828f25d901303f7221824ac30b0ff5e773a` consume `tg_owt@07fb35bc33b3e79125a1beab7690e830f183f5d8`; Windows prepare consumes `tg_angle@9fd174a91e6155d1006c5f427366b23006f5d899` and explicitly links it in the Qt<6 branch. Fork import anchors were verified as `tg_owt@5b6a5c00017c115f6a54b90b109354abc1444a01` ("M123+patches") and `tg_angle@3e2cbd93cb60a4f677af68bac4e3849176f688fa` ("M115"); consumed pins are 102 and 8 in-fork commits ahead respectively. Those counts do not establish original-upstream lag.

Existing `build-time-acquisitions.json@e9c14ce86f8e8be5fa1231fabd47147c94ee887f` contains tg_owt but no tg_angle URL/name/pin. This is preserved as an evidence-input gap and demonstrates why the source union, not that inventory alone, is authoritative for dependency applicability.

Existing-owner-first attaches the reference to existing `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001`; no new responsibility, owner, ADR or platform-core production delta is introduced. The advisory owner remains non-mutating. Authored but not run: `CONTRACT-TDRP-DEPENDENCY-FORK-IMPORT-HISTORY-001` and `CONTRACT-TDRP-DEPENDENCY-FORK-FIX-APPLICABILITY-001`.

After closure: `source_entries_total=16202 / unknown=16157 / unread=16130 / omitted=0 / responsibilities_total=134 / unmapped_responsibilities=0`. Next fixed-U independent blob: `.agents/skills/dependency-watch/references/release-trust.md@81a68395f9ad54ed651a84464a933ed7e57c9d32`.

Zero-CI attestation: no GitHub Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance.
