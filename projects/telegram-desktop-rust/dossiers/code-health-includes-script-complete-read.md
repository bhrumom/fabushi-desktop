# Code-health includes.py complete read

## Source authority

- Repository: `telegramdesktop/tdesktop`
- Commit: `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`
- Path: `.agents/skills/code-health/scripts/includes.py`
- Blob: `2c02b4e7eb821f123588e0a80056edef86fc084c`
- Recursive authority position: array index 30, immediately after `history.py` at index 29.
- File history at the accepted authority contains one introducing commit: `0d6570aeb5794b82b071a004cd4745142a21a048` ("[ai] Add a code-health skill measuring and checking code").

## Symbols and behavior

`IncludeGraph.__init__(tree, model)` builds a read-only source include/style-import graph from one exact scanned source tree and the source-derived build model. C/C++ edges come from quoted includes recognized by `cpp_text.includes`; `.style` edges come from `style_imports`. Translation units are restricted to files accepted by the build model, unless the model has no file-target mapping.

`resolve(source, include)` resolves an include first relative to the source directory, then through the source's own root followed by the other declared roots, then maps generated-style include names of the form `styles/style_<name>.h` back to the matching `<name>.style` source. Unresolved include strings are counted separately and are not fabricated into graph edges.

`closure(start)` computes and caches a cycle-safe transitive include closure. `pch_closure(unit)` resolves the source-derived target PCH and returns an empty set when the declared PCH is not in the exact scanned source tree.

`costs()` derives source-level build-work proxies: per-translation-unit header-line weight excluding the target PCH closure, per-header reach, per-header rebuild work, and total work including each distinct PCH closure. These are source-line proxies, not compiler timings or configured-build truth.

`layering()` compares resolved include edges with the source-derived target dependency closure and reports path/line/include/from/to records when a source target includes a target it does not link.

`style_imports(text, path, tree)` recognizes bounded `using "<path>";` style imports, searches declared roots, and preserves source line numbers.

## Real callers and responsibility boundary

- `.agents/skills/code-health/scripts/report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` builds `IncludeGraph`, consumes `costs()`, PCH closure and `layering()`, then projects weight/reach/rebuild/layering into repository-quality reports.
- `.agents/skills/code-health/scripts/review.py@070b55553052267e8fbc12d70985b447b24bbb02` builds the same graph for changed-source checks, resolves new includes, derives header build-weight warnings and rejects/warns cross-target include edges according to caller-owned review policy.
- `.agents/skills/code-health/scripts/health_test.py@2176aea672fd9ed16c01342939fbffb41490ae73` GraphTests exercise target/PCH composition, upward-layering detection and reach/weight accounting.
- `.agents/skills/code-health/scripts/health.py` imports the module as part of the integrated quality command, but result/finding/publication semantics stay with report/review/health.
- The graph does not own CMake parsing, configured build truth, compiler preprocessing, warnings/errors, durable findings, source edits, task routing, product runtime or release acceptance.

## Existing-owner decision

Applicable responsibility: source-neutral repository include/dependency graph evidence for path resolution, transitive closure, PCH-excluded build-work proxies, header reach/rebuild estimates, style-import resolution and cross-target layering investigation.

Current Fabushi already places include resolution/layering/build-cost interpretation under repository quality governance, with source-derived build-model evidence explicitly supplying approximate target/link/PCH relationships and configured-build graph provenance remaining the stronger authority. The correct migration is therefore to make the include-graph governance/qualification/evidence contract explicit under those existing owners. No Telegram-specific include graph, build owner, findings store or second dependency graph runtime is introduced.

## Failure and lifecycle boundaries

Every graph observation is bound to the exact scanned source snapshot plus the exact source-derived build-model recognizer. An unresolved include remains unresolved evidence; missing/unsupported build-model semantics cannot be converted to a clean layering result. Local/root/style resolution order is part of the bounded recognizer contract. Cycles must terminate without losing reachable nodes. PCH exclusion and line-count build work are quality proxies, not proof of compiler input, configured target membership, wall-clock build cost or artifact reachability. Caller-owned report/review thresholds and warning/error decisions remain outside the graph.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-INCLUDE-GRAPH-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-INCLUDE-RESOLUTION-001`, `ORA-TDRP-CODE-HEALTH-INCLUDE-COST-001`, `ORA-TDRP-CODE-HEALTH-LAYERING-001`
- Invariants: `INV-TDRP-CODE-HEALTH-INCLUDE-EXACT-SOURCE-001`, `INV-TDRP-CODE-HEALTH-INCLUDE-NO-FABRICATED-EDGE-001`, `INV-TDRP-CODE-HEALTH-INCLUDE-NO-CONFIGURED-BUILD-CLAIM-001`

These identifiers bind only this source-entry responsibility and its repository-quality evidence boundary; they do not claim configured-build, product/runtime or release acceptance.
