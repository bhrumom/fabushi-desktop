# Round 143 — fixed-U diff review ratchet and move-only proof

Task round: 112  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/code-health/scripts/review.py@89f15913714c54467ef40d6f42feb5c526cb0f32`

Live desktop checkpoint was reread at `fb2f415ce862ec8890878814eded232d436c3b2d`; desktop main remained `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core checkpoint remained `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. The complete fixed-U root tree remained `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, with 7,194 entries / 6,859 blobs. Before closure the unique ledger was `source_entries_total=16202 / unknown=16166 / unread=16139 / omitted=0 / responsibilities_total=130 / unmapped_responsibilities=0`; `review.py` was independently unrepresented and next after represented `report.py`.

The exact 21,693-byte blob was read completely. Its portable responsibility is the explicit-base diff review ratchet plus move-only proof: tracked/untracked/staged change acquisition; warning/error strictness; new/known target-crossing include rules; shared file/class growth limits; copied-from additions; source-derived fresh-header build-cost thresholding; conservative stranded-definition and last-use orphan reference checks across platform/style/language cases; and normalized moved/lost/novel/order evidence.

Direct consumers were verified in `health.py`, `health_test.py`, and the code-health `SKILL.md`. Existing clone/CMake/text/include/metrics/report/source modules provide independently-accounted facts and are not re-claimed. Existing-owner-first maps the responsibility to current Fabushi repository quality/test-evidence governance and existing quality-health command orchestration; no platform-core production delta, upstream Python review port, second scanner/findings/task queue/configured-build/product runtime, or ADR is required.

Spec-first commit `467f75c86e3558d7c15454a1cd66f22e2ea8e6c5` added the review-ratchet qualification/evidence contracts and complete-read dossier. Authored, not run: `CONTRACT-TDRP-CODE-HEALTH-REVIEW-SCOPE-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-SEVERITY-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-ORPHAN-REFERENCE-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-CLONE-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-INCLUDE-LAYERING-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-BUILD-COST-001`, `CONTRACT-TDRP-CODE-HEALTH-REVIEW-CLASS-GROWTH-001`, `CONTRACT-TDRP-CODE-HEALTH-MOVE-ONLY-001`, and `INT-TDRP-CODE-HEALTH-REVIEW-HEALTH-COMPOSITION-001`.

After closure: `source_entries_total=16202 / unknown=16165 / unread=16138 / omitted=0 / responsibilities_total=131 / unmapped_responsibilities=0`. Exactly one independent source entry closes; `sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b` remains independently unrepresented and next in the scripts tree.

Zero-CI attestation: no Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance.
