# Round 183 — fixed-U repository Git attributes / source-control normalization

- Task round: 152
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `cb0c819330635228766d59817641bc95f07c8cdb`
- Spec-first desktop commit: `494f6d79ca936625b8bc5f4efe7aad2cda633028`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, complete, 7,194 entries
- Closed source: `.gitattributes@d7f1c6218bd3bd6b595385b1308c86ba9923fa14`
- Tree index: 103 zero-based; mode 100644
- Source identity: 419 Git bytes / 419 ASCII characters / 15 LF newlines / 0 CR / trailing newline / 16 split lines
- Responsibility: `TDRP-R9-REPOSITORY-GIT-ATTRIBUTES-SOURCE-CONTROL-001`

## Verified source semantics

The exact source was read through EOF. Rules are exactly: `* text=auto`; `*.diff text eol=lf`; `*.bat text eol=crlf`; `Telegram/Resources/zxcvbn/*.txt text eol=lf`; `*.lottie binary`; `*.bin binary`.

Git is the direct consumer. The source governs text/index normalization, explicit LF/CRLF working-tree endings, binary classification and built-in diff/merge treatment. It declares no named custom diff or merge driver, no filter, no working-tree encoding, no export-ignore and no export-subst rule. The built-in `binary` macro is recorded separately from custom executable drivers.

## Verified fixed-U scope

Complete recursive-tree matching found:
- `*.diff`: 0 blobs.
- `*.bat`: 6 blobs.
- zxcvbn word lists: 2 blobs, `ru_50k.txt@fa4412020b6409b90a45c1278d07e98812b544e2` and `uk_50k.txt@b5c607e7fe06c4ea1f3456a1a41f35f005b0a236`.
- `*.lottie`: 14 blobs.
- `*.bin`: 10 blobs.

The zxcvbn source comment explicitly binds LF to byte-identical upstream digest stability. Binary rules protect animation/raw-table/test-fixture bytes from text conversion.

## Existing-owner-first disposition

Current Fabushi exact checkpoint inspection found no root `.gitattributes` and no tracked `*.diff`, `*.bat`, `*.lottie`, `*.bin` or zxcvbn matching paths. The only nested `.gitattributes` is an apply-patch test fixture and is not root authority. Therefore this source maps to the existing root repository development-process/source-provenance owner plus shared quality/test-evidence governance; no Telegram-specific path registry, root-config copy, custom filter service, product Storage/Resource owner, packaging exporter or platform-core service is introduced.

## Failure/recovery and deferred qualification

Changing attributes alone does not retroactively normalize existing index entries. Stale index state, wrong explicit EOL, binary/text misclassification or zxcvbn byte drift remains non-passing until explicit controlled Stage-B correction/renormalization and fresh exact-candidate evidence.

Authored only, not run: seven contract cases, one integration case, three fault-recovery cases and one security case. Ledger state is `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`. Coverage moves exactly one independent source entry: unknown `16126 -> 16125`, unread `16099 -> 16098`, omitted stays 0, responsibilities `152 -> 153`, unmapped remains 0.

No Actions/check/status/workflow query or trigger; no checkout/add/renormalize/diff/merge/archive/digest/build/compile/typecheck/lint/validator/test/E2E/packaging/deployment/installation/runtime evidence or acceptance was performed. The next fixed-U independent blob is `.github/CONTRIBUTING.md@4265e974d0b87622af223d5b8096b67edcbd4aab`.
