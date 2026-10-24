# Ninja target provenance guide complete read

Status: mapped / exact-upstream read complete / exact-head verification pending  
Project: TDRP-001 Revision 9  
Fabushi write lease: bhrumom/fabushi-desktop@60fabff3240cb91f16456d0fd9b40b450078a931  
Upstream: telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a  
Tree: 5151db79560ed40bf3802482fcfb74b950be6c40  
Path: .agents/shared/ninja-target-evidence.md  
Blob: 7d73d0c8acbefb01b5dd748296db931c58d59b18  
Production oracle: .agents/shared/evidence/ninja_target.py@f638b0e9c039784832155269b5931ab0cabdb16b  
Qualification suite: .agents/shared/evidence/ninja_target_test.py@a9c635eeb46a6ddf4295a815ef29bcae15ba2208  
Caller: .agents/shared/test-loop.md@83c4732bf6139f7c6557f18dd87bb35c3d68808d

## Live authority reconciliation

Immediately before the write, Fabushi main was re-read at `60fabff3240cb91f16456d0fd9b40b450078a931`; upstream `dev` was re-read at `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, and the target guide still resolves to blob `7d73d0c8acbefb01b5dd748296db931c58d59b18`. The accepted upstream tree identity remains `5151db79560ed40bf3802482fcfb74b950be6c40`. This is the eleventh ordered recursive source entry in the current rebaseline.

Closing only this independently owned source entry changes recursive counters from unknown=16,192/unread=16,165 to unknown=16,191/unread=16,164; omitted remains 0. Baseline readiness, full source closure and final release acceptance remain false.

## Source role and caller chain

This Markdown file is the operator-facing evidence contract for configured-target provenance. It does not implement the graph oracle and it does not qualify the oracle; instead it constrains how callers may form claims, invoke the helper, interpret its report, publish outputs, and hand selected artifacts to stronger downstream checks.

The direct caller is `.agents/shared/test-loop.md`, which requires object/PCH/link-membership/compiler-definition claims for Ninja-built targets to come from `ninja_target.py` resolving the exact consuming output and explicitly rejects glob/basename/compdb-style ownership inference. The guide therefore owns an independent source-entry responsibility even though it is documentation.

## Configured-target provenance usage contract

The guide requires an exact configured CMake Ninja subject: `--build-dir`, `--manifest`, `--config`, and `--target` are mandatory and exact. Multi-config and single-config trees have different accepted manifest/config bindings. A phony alias, basename, suffix match or guessed target identity is not accepted evidence.

Source/object selectors must use exact graph spellings. Container-configured roots come from `CMAKE_HOME_DIRECTORY` / `CMAKE_CACHEFILE_DIR`; configured-to-host identity is accepted only through the helper's exact mapping rules, never through realpath/basename/suffix heuristics.

The guide also defines the policy-cohort rule: a whole link closure is not one compiler-definition cohort. Callers must invoke the helper per policy target/cohort where object libraries or other producers have distinct definitions/PCHs.

## Inputs, outputs and fail-closed boundary

Exit 0 emits a JSON report with `verdict: "PASS"`; any refusal or unmet expectation emits `error: ...` and exits 1. There is intentionally no soft FAIL verdict to reinterpret.

The report may expose requested macro values. The guide therefore forbids requesting/publishing private compiler definitions such as `TDESKTOP_API_ID` and `TDESKTOP_API_HASH`; raw query/compdb captures containing private values belong only in ignored task-local storage.

The report records selected objects/sources, excluded siblings, PCHs, archives/shared libraries, policy cohort, path mapping, acquisition digest and limits. Counts are observations of one configured tree, not constants to copy into future evidence.

The guide states what is not ownership evidence: glob/basename matches, bare compdb/compdb-targets lists, and `ninja -t inputs`. Unsupported generators, shell grammar, macro channels, response-file state, ambiguous selectors, graph inconsistencies, unstable manifests or unsupported paths are refusals rather than permission for a caller to widen the grammar ad hoc.

## Build, symbol, archive and runtime separation

Configured-target provenance proves only graph facts. It does not prove command execution, object existence, archive-member reachability, symbol presence/absence, executable/runtime reachability, package identity, successful launch or release acceptance.

For a symbol or absence check, the guide permits `nm` only on helper-selected object paths and recommends an excluded sibling as an independent control. Archives and shared libraries are listed but not resolved. Archive-member and executable/runtime reachability therefore remain independently owned and cannot be promoted from a configured-target PASS.

## Existing-owner-first disposition

The applicable responsibility is already expressible by the source-neutral Fabushi quality system:
- `projects/fabushi-communication-platform/quality/evidence-contract.md` owns evidence identity, fail-closed semantics and claim boundaries;
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` owns configured-build graph qualification and explicitly keeps archive-member, symbol, executable/runtime reachability and release acceptance separate.

No Ninja-specific or Telegram-specific Fabushi owner is introduced. Shipping Host/Runner are rejected because this is evidence governance, not product runtime. Existing archive/reachability scripts remain separate because absorbing this guide into them would collapse independent evidence layers.

## Traceability

Responsibility: `TDRP-R9-CONFIGURED-BUILD-GRAPH-USAGE-001`  
Oracle: `ORA-TDRP-CONFIGURED-BUILD-GRAPH-001`  
Invariants: `INV-TDRP-CONFIGURED-GRAPH-STABLE-001`, `INV-TDRP-CONFIGURED-GRAPH-NO-EXECUTION-001`, `INV-TDRP-ARCHIVE-RUNTIME-SEPARATION-001`

Existing owner: Fabushi Test Evidence Contract / Test Strategy.  
New owner: none.  
Current disposition: implemented as a source-neutral evidence-usage contract; exact-head GitHub Actions verification is required before acceptance into main.
