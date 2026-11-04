# Complete read — fixed-U code-health `metrics.py`

Status: Stage A mapped/implemented as source-neutral repository quality contract; executable validation not run; accepted=false  
Task round: 110  
Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries / 6,859 blobs)  
Source: `.agents/skills/code-health/scripts/metrics.py@42f42b1100f88bd85977b66dfb6bf13e9dc88e46` (4,296 bytes)

## Exact responsibility

The complete blob was read. It owns text-level repository source metrics and dead-code candidates, not shipping behavior.

- `area_of(path)` is a fallback directory classifier: under a `SourceFiles` source root it keeps up to two inner directory levels; under a `lib_*` root it keeps the library root plus up to two inner levels; otherwise it uses the parent path.
- `file_metrics(tree)` considers only C/C++ translation units and headers from `cpp_text`. It records line count for every selected file; parses functions only for translation units; records function count and maximum parsed function length; retains parsed functions at least 150 lines as a pre-filter; parses rough classes/structs and retains those with methods+fields at least 40 as a pre-filter.
- `unused_candidates(tree)` counts lexical identifiers across the scanned tree except `Telegram/Resources/langs/lang.strings`, then considers only non-test `Telegram/SourceFiles/*.cpp|*.cc|*.cxx` function definitions. It excludes the declared exemption regex, non-alphabetic short names, names with more than two total mentions, and constructor-shaped `Type::Type` names. A candidate survives only when `only_declared` can account for all global mentions in the source plus paired `.h` and every non-definition source mention is declaration-shaped.
- `declares` accepts only a column-zero mention that is not an initializer-list entry and has neither `=` nor `(` before the name. This is deliberately conservative and lexical; it does not resolve generated code, configured build ownership, dynamic dispatch, reflection, linker reachability, or runtime calls.
- The source docstring is explicit that unused results are candidates only and deletion is proven by building without them. Fabushi therefore must not interpret this scanner as Q0 deletion authority.

## Direct callers / consumers at fixed U

- `report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12` directly calls `metrics.file_metrics`, `metrics.unused_candidates`, and fallback `metrics.area_of`. It combines file lines with `history.churn` as `round(churn * lines / 1000, 1)`, aggregates hotspot by area, and separately applies finding thresholds. Current consumer limits are 300 lines for a long-function finding, 120 methods or 60 fields for a large-class finding, and top-20 positive-hotspot areas with zero tested lines for `untested-hot-area`. Its report says scores/classes are suggested routing, not deletion proof.
- `review.py@89f15913714c54467ef40d6f42feb5c526cb0f32` directly reuses `metrics.declares` and `metrics.UNUSED_EXEMPT` when deciding whether a changed reference may have left an orphaned definition. This is diff-time lexical evidence, not a second dead-code authority.
- `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` imports metrics and directly uses fallback `metrics.area_of` for area projection when the snapshot lacks an assigned area.
- `health_test.py@6058fa64a5121c966021d8589b2010480aec8794` directly exercises unused-candidate behavior: a declared-but-never-called function remains a candidate, while a constructor initializer-list call prevents `Label` from being classified unused.
- `history.py@7c12f5b091936723c6ad005dda4bb8e2b1e549df` supplies the downstream hotspot churn operand as commits per scanned path over the requested history window (180 days by the report default), including available locked submodule history.
- `cpp_text.py@366f173248add43c34317c8fe17d9ca31d46b1d4` supplies the text-level function/class/identifier parser; `sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b` supplies the exact scanned tree and `LANG_FILE`.

No independent product/runtime consumer is introduced by this source entry. `report.py` is the next independent fixed-U tree entry and remains separately unread/unclosed in the unique ledger.

## Contract boundary

Hotspot is not implemented by `metrics.py` itself; it is a downstream composition of its line metric with Git history. A 150-line function and methods+fields>=40 class are scanner pre-filters, not the report's final 300-line / 120-method / 60-field finding thresholds. “Unused” is a lexical review candidate and cannot establish deletion safety, configured-build reachability, runtime non-use, product correctness, or release acceptance.

Existing-owner-first maps the portable responsibility to the current Fabushi repository quality/test-evidence governance in `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` and `quality/evidence-contract.md`. Shipping Agent/Coordinator/Host/Runner, product performance telemetry, configured-build provenance, and platform-core are rejected owners. No new ADR is required; no upstream Python scanner, second findings ledger/task queue, or runtime owner is created.

## Authored contracts — not run

- `CONTRACT-TDRP-CODE-HEALTH-METRICS-AREA-001`
- `CONTRACT-TDRP-CODE-HEALTH-METRICS-FILE-SHAPE-001`
- `CONTRACT-TDRP-CODE-HEALTH-METRICS-UNUSED-CANDIDATE-001`
- `CONTRACT-TDRP-CODE-HEALTH-METRICS-UNUSED-NEGATIVE-001`
- `CONTRACT-TDRP-CODE-HEALTH-METRICS-HOTSPOT-COMPOSITION-001`
- `CONTRACT-TDRP-CODE-HEALTH-METRICS-FINDING-THRESHOLDS-001`
- `INT-TDRP-CODE-HEALTH-METRICS-REPORT-001`

Stage B must execute these against the frozen exact-head candidate in GitHub Actions. Source review in Stage A is not passing execution evidence.
