# Fixed-U .gitattributes complete read — repository source-control normalization

- Task round: 152
- Date: 2026-11-01
- Desktop checkpoint parent: `cb0c819330635228766d59817641bc95f07c8cdb`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: complete, 7,194 entries
- Source: `.gitattributes@d7f1c6218bd3bd6b595385b1308c86ba9923fa14`
- Tree index: 103 zero-based
- Mode: 100644
- Git size: 419 bytes
- Decoded form: 419 ASCII characters, 15 LF newlines, 0 CR characters, trailing newline, 16 split lines
- Responsibility: `TDRP-R9-REPOSITORY-GIT-ATTRIBUTES-SOURCE-CONTROL-001`
- Stage-A status: implementation authored; executable validation not run; accepted=false

## Exact source

The exact blob was read through EOF:

```gitattributes
# Set the default behavior, in case people don't have core.autocrlf set.
* text=auto

# Ensure diffs have LF endings
*.diff text eol=lf
*.bat text eol=crlf

# Keep the zxcvbn word lists byte-identical to upstream so their digests hold
Telegram/Resources/zxcvbn/*.txt text eol=lf

# Ensure lottie animations are treated as binary files
*.lottie binary

# Raw float tables, such as the 3D diamond model data
*.bin binary
```

No other rule is present.

## Git attribute semantics

The direct consumer is Git's attribute engine. `* text=auto` asks Git to classify text automatically and normalize text in the index. Explicit `text eol=lf` / `text eol=crlf` rules bind working-tree line endings for matching text while preserving normalized repository text semantics. Git's built-in `binary` macro disables text treatment and textual diff/merge behavior for matching paths; it is not a named external driver.

The source contains no named `diff=<driver>`, `merge=<driver>`, `filter=<driver>`, `working-tree-encoding`, `export-ignore`, or `export-subst` declaration. Therefore no clean/smudge/process filter, custom merge/diff executable, archive exclusion, archive substitution, or encoding transform may be attributed to this blob.

Attribute edits do not retroactively rewrite already-indexed content. Any required renormalization is an explicit Git operation and must be captured as later frozen-candidate evidence; Stage A performs no such operation.

## Fixed-U path scope

The complete recursive fixed-U tree was inspected against every explicit pattern:

- `*.diff`: 0 tracked blobs.
- `*.bat`: 6 tracked blobs — `Telegram/build/build.bat`, `Telegram/build/prepare/win.bat`, `Telegram/build/set_version.bat`, `Telegram/build/test_package.bat`, `Telegram/configure.bat`, `Telegram/create.bat`.
- `Telegram/Resources/zxcvbn/*.txt`: 2 tracked blobs — `ru_50k.txt@fa4412020b6409b90a45c1278d07e98812b544e2` (998,861 bytes) and `uk_50k.txt@b5c607e7fe06c4ea1f3456a1a41f35f005b0a236` (887,259 bytes).
- `*.lottie`: 14 tracked blobs under Telegram resources, including call animations, notify toggle and device icons.
- `*.bin`: 10 tracked blobs — nine premium diamond raw-data tables plus `Telegram/SourceFiles/gram/tests/fixtures/fragment-imgproxy-deskcalendar-small.bin`.

The source comment explicitly ties the zxcvbn LF rule to byte-identical upstream digest integrity. The binary rules protect compressed/animation/raw-table/test-fixture bytes from text conversion.

## Direct consumers and non-consumers

There is no source-code import/caller for the root file. Git itself consumes repository attributes during working-tree/index conversion and classification. Matching tracked files are subjects of the policy, not callers.

Product code that later reads lottie/bin resources or build scripts that execute `.bat` files is downstream of source-control byte preservation; those product/build consumers do not become owners of Git attribute policy. Likewise archive tooling is not granted an export rule: this blob declares no `export-ignore` or `export-subst`.

## Existing-owner-first mapping

Current Fabushi checkpoint tree inspection found:
- no repository-root `.gitattributes`;
- zero tracked `*.diff`, `*.bat`, `*.lottie`, `*.bin`, or zxcvbn-word-list paths matching the fixed-U special cases;
- one nested `.gitattributes` only inside an apply-patch test fixture, which is fixture data and not repository policy.

The source-neutral target owner is therefore the existing Fabushi root repository development-process/source-provenance authority plus shared quality/test-evidence governance. This round does not create a Telegram-specific Git policy registry and does not copy Telegram-only resource paths into Fabushi. It also does not silently impose `* text=auto` on the entire current Fabushi history as a side effect of a source-entry closure; any future root attribute policy is a separate repository-wide owner decision and must be source-neutral.

Rejected owners: Product Storage/Sync, Resource lifecycle, packaging/updater/export runtime, Agent/Coordinator/Host/Runner, platform-core service/authorization, Telegram-specific source-control runtime, custom filter service, and a second evidence ledger.

## Failure and recovery contract

Non-passing conditions include: stale index content after an attribute-policy change; checkout line endings inconsistent with explicit `eol`; text/binary misclassification; zxcvbn byte/digest drift; a claimed custom filter/diff/merge/export behavior not present in the source; and treating a nested fixture `.gitattributes` as root authority.

Recovery is explicit source-control policy correction and later controlled renormalization/re-check against the frozen Stage-B candidate. No background rewrite, hidden fallback, automatic acceptance, or product persistence is introduced.

## Authored Stage-B qualification

Authored only, not run:
- `CONTRACT-TDRP-GITATTRIBUTES-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-GITATTRIBUTES-TEXT-AUTO-001`
- `CONTRACT-TDRP-GITATTRIBUTES-DIFF-LF-001`
- `CONTRACT-TDRP-GITATTRIBUTES-BAT-CRLF-001`
- `CONTRACT-TDRP-GITATTRIBUTES-ZXCVBN-LF-001`
- `CONTRACT-TDRP-GITATTRIBUTES-BINARY-MACRO-001`
- `CONTRACT-TDRP-GITATTRIBUTES-NO-CUSTOM-DRIVERS-001`
- `INT-TDRP-GITATTRIBUTES-SOURCE-CONTROL-COMPOSITION-001`
- `FAULT-TDRP-GITATTRIBUTES-STALE-INDEX-001`
- `FAULT-TDRP-GITATTRIBUTES-PATTERN-MISCLASSIFICATION-001`
- `FAULT-TDRP-GITATTRIBUTES-DIGEST-DRIFT-001`
- `SEC-TDRP-GITATTRIBUTES-NO-FILTER-EXECUTION-001`

Stage A performs no checkout/add/renormalize/diff/merge/archive/digest/build/typecheck/lint/validator/test/E2E/package/deploy/install/runtime-acceptance operation.
