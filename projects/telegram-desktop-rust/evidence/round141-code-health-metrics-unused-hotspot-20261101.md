# Round 141 — fixed-U source metrics, unused candidates, and hotspot composition

Task round: 110  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/code-health/scripts/metrics.py@42f42b1100f88bd85977b66dfb6bf13e9dc88e46`

The live desktop checkpoint was reread at `324ffc521c2cddaad476e80894b49a144753c680`; platform-core checkpoint `checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16` was reread at `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. The complete fixed-U root tree was reread as `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, with 7,194 entries / 6,859 blobs. The unique ledger before closure was `source_entries_total=16202 / unknown=16168 / unread=16141 / omitted=0 / responsibilities_total=128 / unmapped_responsibilities=0`. Tree/ledger anchor comparison confirms `metrics.py` is the first unrepresented independent blob after represented `includes.py`; `report.py` remains the next independent unrepresented entry.

The exact 4,296-byte source blob was read completely. Its portable responsibility is text-level repository maintainability evidence: fallback area grouping; per-file line/function/class shape observations; a 150-line function collection pre-filter; a rough methods+fields>=40 class pre-filter; and conservative lexical unused-function candidates. The unused path deliberately excludes the language resource from identifier counts, only examines non-test `Telegram/SourceFiles/*.cpp|*.cc|*.cxx`, applies callback/operator/constructor/name-count guards, and requires remaining local mentions to be declaration-shaped. It cannot observe configured build/link/runtime reachability and explicitly does not prove deletion safety.

Direct fixed-U consumers and adjacent inputs were read. `report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12` calls `file_metrics`, `unused_candidates`, and fallback `area_of`; it composes hotspot as `round(churn * lines / 1000, 1)` and independently applies 300-line long-function plus 120-method/60-field large-class finding thresholds. `review.py@89f15913714c54467ef40d6f42feb5c526cb0f32` reuses `declares` and `UNUSED_EXEMPT` for diff-time orphan-reference evidence. `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` consumes fallback area grouping. `health_test.py@6058fa64a5121c966021d8589b2010480aec8794` checks a declared-never-called candidate and an initializer-list call negative control. `cpp_text.py@366f173248add43c34317c8fe17d9ca31d46b1d4`, `sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b`, and `history.py@7c12f5b091936723c6ad005dda4bb8e2b1e549df` provide parser/source/churn inputs.

Existing-owner-first maps this to the existing Fabushi repository quality/test-evidence governance. Spec-first commit `68e34c7c7ee1459d2ec06f54fc1f4d29583a29e6` adds the source-neutral qualification/evidence contracts and complete-read dossier. No platform-core production change, upstream Python port, second metrics/dead-code scanner/findings/task/performance/runtime owner, or ADR is required.

Authored, not run: `CONTRACT-TDRP-CODE-HEALTH-METRICS-AREA-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-FILE-SHAPE-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-UNUSED-CANDIDATE-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-UNUSED-NEGATIVE-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-HOTSPOT-COMPOSITION-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-FINDING-THRESHOLDS-001`, and `INT-TDRP-CODE-HEALTH-METRICS-REPORT-001`.

After closure: `source_entries_total=16202 / unknown=16167 / unread=16140 / omitted=0 / responsibilities_total=129 / unmapped_responsibilities=0`. Exactly one independent source entry closes.

Zero-CI attestation: no Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance.
