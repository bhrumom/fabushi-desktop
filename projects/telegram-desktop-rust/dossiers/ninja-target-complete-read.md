# Ninja target configured-build provenance complete read

Status: mapped / exact-upstream read complete
Project: TDRP-001 Revision 9
Upstream: telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a
Tree: 5151db79560ed40bf3802482fcfb74b950be6c40
Primary source: .agents/shared/evidence/ninja_target.py@f638b0e9c039784832155269b5931ab0cabdb16b
Qualification source: .agents/shared/evidence/ninja_target_test.py@a9c635eeb46a6ddf4295a815ef29bcae15ba2208
Guide: .agents/shared/ninja-target-evidence.md@7d73d0c8acbefb01b5dd748296db931c58d59b18

## Responsibility closure

The upstream helper proves configured-build graph provenance for one exact CMake Ninja consuming output. It binds CMakeCache.txt to Ninja or Ninja Multi-Config, validates the exact manifest/config pair, requires Ninja >= 1.10 and the response-file-expanding `-t compdb -x` capability, and permits only `query` and `compdb` tool modes with explicit safe operands.

Acquisition is read-only. Before graph inspection it snapshots CMakeCache.txt and discovered Ninja manifests with size, mtime and SHA-256, and rejects any changed snapshot after acquisition. It parses `query` and `compdb -x` fail-closed, requires one canonical compdb directory, and records configured-to-host path mapping only through exact prefix identity plus samefile/cache binding.

Command parsing intentionally supports a bounded POSIX grammar only: whitespace-delimited `&&` simple commands, `:`, absolute `cd`, GCC/Clang drivers and optional ccache/sccache. It rejects shell expansion/operators, unsupported drivers, unexpanded response files, unknown operands, hidden macro-definition channels and unsupported include forms.

For the selected link edge it reconciles every object operand against explicit/implicit Ninja graph inputs, rejects duplicates, substitutions, order-only-only substitutions and missing link operands, and records archive/shared-library paths without resolving archive members. Object producers must be compile rules with coherent graph reverse edges, compdb output/source identity and exact source mapping.

PCH handling covers clang include-pch/emit-pch and GCC include + .gch forms. A consumed PCH must be an implicit graph input, have a compatible producer rule/dialect/header, and match the consuming object cohort. Command-line macro state is ordered (later -D/-U wins), non-PCH forced includes make macro claims fail closed, and Ninja Multi-Config cohorts validate CMAKE_INTDIR.

The JSON report includes tool versions, build/config/manifest/host/configured identities, path mapping, stable acquisition digest, target/link rule, object counts and producer rules, external objects, archives/shared libraries, PCHs, policy cohort, exact source/object selections, and explicit limits.

## Qualification coverage

The companion suite has handwritten independent Ninja-shaped fixtures and real CMake/Ninja fixtures. It covers safe shell grammar, link/compiler parsing, clang/GCC PCHs, macro ordering and hidden channels, query/compdb parser refusals, tool allowlisting, CMake cache/config/manifest binding, exact selectors/path mapping, duplicate/substituted/missing graph objects, producer/source/output mismatches, PCH incoherence, policy mismatches, ambiguous sources, stable-manifest mutation, and a sentinel proving graph commands are never executed. The real fixture cross-checks selected objects with nm and exercises ON/OFF option contracts, response-file expansion and duplicate-object refusal.

## Existing-owner-first disposition

Applicable responsibility is absorbed by existing source-neutral Fabushi quality owners:
- projects/fabushi-communication-platform/quality/evidence-contract.md: configured-build graph provenance evidence;
- projects/fabushi-communication-platform/quality/TEST_STRATEGY.md: configured-build graph qualification.

No Telegram/Ninja-specific Fabushi production, test, release or packaging owner is introduced. The upstream Python implementation is not copied.

## Explicit boundaries

This responsibility proves configured graph facts only. It does not prove command execution, object existence, archive-member reachability, runtime/executable reachability, symbol presence, package identity, successful launch, or release acceptance. Archive-member and runtime reachability remain independently owned by existing archive/reachability evidence and GitHub Actions gates.


## Traceability IDs

- TDRP-R9-CONFIGURED-BUILD-GRAPH-PROVENANCE-001
- TDRP-R9-CONFIGURED-BUILD-GRAPH-QUALIFICATION-001
- ORA-TDRP-CONFIGURED-BUILD-GRAPH-001
- INV-TDRP-CONFIGURED-GRAPH-STABLE-001
- INV-TDRP-CONFIGURED-GRAPH-NO-EXECUTION-001
- INV-TDRP-ARCHIVE-RUNTIME-SEPARATION-001
