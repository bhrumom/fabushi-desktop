# Ninja target qualification suite complete read

Status: mapped / exact-upstream read complete / exact-head verification pending  
Project: TDRP-001 Revision 9  
Fabushi write lease: bhrumom/fabushi-desktop@c4226fc2017853f20492d730dfbe6f25b4eb9fb9  
Upstream: telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a  
Tree: 5151db79560ed40bf3802482fcfb74b950be6c40  
Path: .agents/shared/evidence/ninja_target_test.py  
Blob: a9c635eeb46a6ddf4295a815ef29bcae15ba2208  
Production companion: .agents/shared/evidence/ninja_target.py@f638b0e9c039784832155269b5931ab0cabdb16b

## Live authority reconciliation

Immediately before the write, Fabushi main was re-read at `c4226fc2017853f20492d730dfbe6f25b4eb9fb9` with root tree `b55a71bba37fdba271dc1bc353bcfc2de602ace0`; upstream `dev` was re-read at `351204f3695b4a39bc3f4d2f230c061b9a01fe2a` with tree `5151db79560ed40bf3802482fcfb74b950be6c40`. The target test blob remains exactly `a9c635eeb46a6ddf4295a815ef29bcae15ba2208`, so the completed 1,315-line review remains applicable without source drift.

This is the tenth ordered recursive source entry in the current rebaseline. Closing only this independently owned source entry changes recursive counters from unknown=16,193/unread=16,166 to unknown=16,192/unread=16,165; omitted remains 0. Baseline readiness, source closure and final release acceptance remain false.

## Source role and ownership boundary

The file is the independent qualification/control suite for the configured-build graph provenance helper. It imports `ninja_target` and tests that the production oracle is both useful on accepted inputs and fail-closed at its declared grammar, identity and stability boundaries. It is build/release evidence tooling, not a Telegram product runtime, shipping build owner, package owner, or runtime reachability owner.

Existing-owner-first disposition is source-neutral: the applicable responsibility is owned by `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` under **Configured-build graph qualification**, with the machine-evidence boundary anchored by `projects/fabushi-communication-platform/quality/evidence-contract.md` under **Configured-build graph provenance evidence**. No Ninja-specific or Telegram-specific Fabushi test owner is introduced, and the upstream Python suite is not copied into shipping code.

Archive-member reachability, symbol evidence, executable/runtime reachability, package identity and release acceptance remain independent responsibilities. Passing this qualification cannot promote any of those stronger claims.

## Independent HandTree fixture

`HandTree` constructs a handwritten CMake-shaped Ninja Multi-Config tree rather than asking the production parser to emit its own expected input. It writes literal `CMakeCache.txt` generator/config/cache-directory bindings plus a `build-Debug.ninja` manifest containing compile, PCH, link, phony, explicit, implicit and order-only edges. The handwritten tree itself is never built.

The fixture deliberately includes:
- app, object-library and sibling-tool objects that share source names but have different rules/consumers;
- an external prebuilt object and repeated archive operands;
- a phony `app` alias and exact `Debug/app` link edge;
- Clang and GCC compiler/linker dialects, optional ccache launcher handling, C and C++ producers;
- Clang `.pch` and GCC `.gch` producer/consumer forms;
- ordered `-D`/`-U` definition state and CMake option-to-macro contracts;
- configured-path versus host-path identity;
- normal link argv and Ninja response-file forms.

This fixture is the independent basis for accepted controls and one-fault negative mutations.

## Parser, tool and binding controls

The suite separately qualifies bounded shell splitting, link grammar, compiler grammar, Ninja query parsing, compdb parsing, tool allowlisting, CMake cache identity, boolean option contracts, configuration selection and exact target/source/object selectors.

Accepted controls include safe `&&` command segments, launcher/compiler recognition, host-namespace `cd`, object/archive/shared-library operand classification, Clang/GCC PCH use and production, ordered macro definitions, well-formed query sections, compdb records, Ninja/Ninja Multi-Config cache identity and exact configured namespace mapping.

Refusal controls cover unsupported shell syntax/operators, unsupported drivers or launchers, malformed query sections, malformed/duplicate compdb identity, unsafe Ninja tools/operands, invalid cache/generator/config/manifest expectations, wrong option contracts, basename/phony selectors, and configured directories that resolve to a different local directory.

## Object, producer, PCH, macro and path negative matrix

`test_one_mutation_negatives` changes one graph or command fact at a time and requires a specific refusal. The matrix covers duplicate object operands; graph inputs missing from the link argv; substituted sibling objects; order-only-only substitutions; producer output mismatch; producer source mismatch; non-compile object producers; PCH command/graph disagreement; duplicate PCH inclusion; PCH producer rule/dialect/header mismatch; object-versus-PCH macro disagreement; unmet policy expectations; absence claims without a known-present control; wrong `CMAKE_INTDIR`; unsupported post-build shell syntax; literal unexpanded response files; ambiguous source selection; sibling object selection; and sources not consumed by the selected target.

Additional focused cases cover GCC PCH use without a `.gch` graph input, plain forced includes that cannot support a macro claim, missing compdb entries, and exact manifest/config/phony-alias refusals.

## Stable acquisition and no-execution proof

`test_manifest_touched_during_acquisition` mutates `build-Debug.ninja` between read-only Ninja tool calls and requires the stable-manifest check to reject the result.

`test_graph_commands_are_never_executed` places a `touch` command in the link edge's `PRE_LINK`, resolves the graph through the normal runner, requires a PASS result, and then proves the sentinel file does not exist. The qualification therefore proves the evidence helper uses metadata inspection without executing the selected graph command.

## Real CMake/Ninja fixture and independent nm cross-check

`CompiledFixtureTests` creates real source files, a prebuilt object and four CMake Ninja Multi-Config build directories. The OFF and ON variants are configured and built; response-file and duplicate-object variants are configured without being built because their purpose is graph-evidence qualification.

The real fixture verifies that the selected app object excludes a sibling tool object compiled from the same `shared/common.cpp`, that the object-library cohort owns its own PCH, that ON/OFF cache options project the expected macro contract, that a same-name option/macro mismatch fails, that response-file expansion yields the same selected objects, and that duplicate link objects are refused.

For the built OFF variant, `nm` independently reads the selected and sibling object files: the selected object contains the app marker and excludes the tool marker, while the sibling object shows the inverse. This is an external cross-check of object selection, not a widening of the configured-graph oracle into general symbol or runtime reachability evidence.

## Source-neutral qualification mapping

The existing Fabushi quality contract already captures the required behavior:
- independent parser fixtures plus a real CMake/Ninja fixture;
- Ninja/Ninja Multi-Config manifest/config binding;
- safe tool/operand construction;
- query/compdb and bounded shell/compiler/link parsing;
- object/link graph reconciliation and producer/source/output identity;
- PCH/macro/path coherence;
- stable-manifest acquisition;
- response-file handling;
- machine-readable PASS/error semantics;
- a positive proof that graph commands are not executed.

The qualification suite strengthens confidence in the existing configured-build evidence owner; it does not create a second build graph truth. Focused subsets may aid authoring but cannot replace the complete qualification matrix when this evidence class is relied upon.

## Traceability

Responsibility: `TDRP-R9-CONFIGURED-BUILD-GRAPH-QUALIFICATION-001`  
Oracle: `ORA-TDRP-CONFIGURED-BUILD-GRAPH-001`  
Invariants:
- `INV-TDRP-CONFIGURED-GRAPH-STABLE-001`
- `INV-TDRP-CONFIGURED-GRAPH-NO-EXECUTION-001`
- `INV-TDRP-ARCHIVE-RUNTIME-SEPARATION-001`

Existing owner: Fabushi Test Strategy / Test Evidence Contract.  
New owner: none.  
Current disposition: implemented as a source-neutral evidence/qualification contract; exact-head GitHub Actions verification of this mapping is still required before this source-entry closure is accepted into main.
