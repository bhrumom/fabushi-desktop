# Round 140 — fixed-U include graph and build-cost proxy

Task round: 109  
Stage: A / zero executable validation  
Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/code-health/scripts/includes.py@2c02b4e7eb821f123588e0a80056edef86fc084c`

The live desktop checkpoint was reread at `bbd9c803755a14e5ac45f9fa5ae4894f3aede06c`; platform-core checkpoint `checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16` was reread at `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. The unique ledger before this closure was `source_entries_total=16202 / unknown=16169 / unread=16142 / omitted=0 / responsibilities_total=127 / unmapped_responsibilities=0`.

The complete fixed-U root tree was reread at `91f2b083ab2877c0a17d07868eabec362a5cfca6` with `truncated=false` (7,194 entries / 6,859 blobs). Comparing blob order with all independent ledger anchors confirms `.agents/skills/code-health/scripts/includes.py@2c02b4e7eb821f123588e0a80056edef86fc084c` is the first unrepresented source entry; the immediately preceding `history.py@7c12f5b091936723c6ad005dda4bb8e2b1e549df` is already represented.

The exact 5,150-byte `includes.py` blob was read completely. It resolves project quoted C/C++ include/import and style-import edges, retains unresolved counts, computes memoized cycle-safe closure and target PCH closure, then derives per-TU non-PCH header-line weight, header reach/rebuild, total build-work proxy and target-layering violations. Resolution precedence is local path -> own source root -> other roots, with generated `styles/style_*.h` mapped to matching `.style` sources. These observations are static source/build-cost proxies, not configured-build reachability or compiler-duration evidence.

Direct consumers read: `report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12` (snapshot costs/PCH/layering and heavy-unit/header findings), `review.py@89f15913714c54467ef40d6f42feb5c526cb0f32` (new include resolution, target-crossing ratchet and reach/closure cost warning), `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` + `atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43` (atlas graph projection), and `health_test.py@6058fa64a5121c966021d8589b2010480aec8794` (`GraphTests`). Adjacent inputs read are `cpp_text.py@366f173248add43c34317c8fe17d9ca31d46b1d4`, `cmake_model.py@e40ec0fc152958f96bd58d795f70d1a3a24ab1eb`, and `sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b`.

Existing-owner-first result: keep the existing Fabushi repository quality/test-evidence owner. Spec-first commit `32178dfb8a7ea859f21870eeb92df6f66a37e831` adds source-neutral qualification/evidence contracts and the complete-read dossier; no platform-core production change, second scanner/build model/findings ledger/task queue/product runtime, or ADR is required.

Authored, not run: `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-RESOLUTION-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-CLOSURE-PCH-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-BUILD-COST-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-LAYERING-001`, `CONTRACT-TDRP-CODE-HEALTH-STYLE-IMPORT-001`, and `INT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-001`.

After closure: `source_entries_total=16202 / unknown=16168 / unread=16141 / omitted=0 / responsibilities_total=128 / unmapped_responsibilities=0`. Exactly one independent source entry closes.

Zero-CI attestation: no Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance.
