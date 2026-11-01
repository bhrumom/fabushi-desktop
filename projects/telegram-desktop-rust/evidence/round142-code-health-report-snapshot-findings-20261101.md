# Round 142 — fixed-U report snapshot, finding lifecycle, and projection

Task round: 111  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/code-health/scripts/report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12`

Live desktop checkpoint was reread at `075b9005e76023479151e1fd5659bbc718861db0`; platform-core checkpoint remained `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. The complete fixed-U tree remained `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, with 7,194 entries / 6,859 blobs. Before closure the unique ledger was `source_entries_total=16202 / unknown=16167 / unread=16140 / omitted=0 / responsibilities_total=129 / unmapped_responsibilities=0`; `report.py` was unrepresented and next after represented `metrics.py`, while `review.py` remained unrepresented.

The exact 19,769-byte blob was read completely. It composes the exact-revision snapshot, stable finding identities/classes/scores, historical open/resolved/ignored/unverified merge semantics including unavailable-evidence fencing, 180-day resolved-history retention, and deterministic Markdown totals/deltas/hotspots/top findings/build-cost/helper coverage. Direct consumers were verified in `health.py`, `health_test.py`, `harness_test.py`, and `review.py`; publication/idempotence remains owned by `health.py`.

Existing-owner-first maps this to existing Fabushi repository quality/test-evidence governance and command orchestration. Spec-first commit `1195399417cf08502eda2c5e149ffad8536d2cad` adds the source-neutral qualification/evidence contracts and complete-read dossier. No platform-core production change, upstream Python port, second scanner/findings ledger/task queue/product telemetry/publication runtime, or ADR is required.

Authored, not run: `CONTRACT-TDRP-CODE-HEALTH-REPORT-SNAPSHOT-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-FINDINGS-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-MERGE-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-MAP-REFERENCES-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-MARKDOWN-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-UNAVAILABLE-EVIDENCE-001`, and `INT-TDRP-CODE-HEALTH-REPORT-HEALTH-COMPOSITION-001`.

After closure: `source_entries_total=16202 / unknown=16166 / unread=16139 / omitted=0 / responsibilities_total=130 / unmapped_responsibilities=0`. Exactly one independent source entry closes.

Zero-CI attestation: no Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance.
