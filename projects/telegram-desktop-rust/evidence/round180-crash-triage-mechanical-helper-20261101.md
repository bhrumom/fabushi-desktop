# Round 180 — fixed-U crash-triage mechanical helper

- Task round: 149
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `c7b420e19fa091cadc149b236bab90db45b6177f`
- Spec-first desktop commit: `72f2ed79215b381d8d6f49c2ae45aeef4c0258fd`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, complete, 7,194 entries / 6,859 blobs
- Closed source: `.claude/scripts/crash_triage.py@2db6cc14d8e529c4e61c5dcf169aade31fd35c58`
- Tree index: 100 zero-based; mode 100755
- Source identity: 38,738 Git bytes / 38,738 decoded ASCII characters / 1,030 newline characters / trailing newline / 1,031 split lines
- Direct caller: `.claude/commands/crash-reports-process.md@5ac555406670721694e78c4e1e6b2a96b3e98fb5`
- Responsibility: `TDRP-R9-CRASH-TRIAGE-MECHANICAL-HELPER-001`

## Verified source and composition

The complete helper was read through EOF. It resolves an explicit or newest-tag release identity; claims only complete log/err/dmp triples; resumes an interrupted .processing run; retains incomplete triples; reuses the latest prior run when nothing new arrived unless --new is explicit; parses Breakpad stackwalk text; classifies exactly old-version/no-frames/unsymbolized/usable; canonicalizes cross-toolchain crash signatures; emits routing hints; maps dump files back to tracked source; writes index.tsv, groups.md and groups.json; maintains the local stable-key verdict ledger; renders bounded representative reports; and moves default drop buckets unless explicit deletion is authorized.

The exact fixed-U crash-reports-process command is the direct repository caller and consumes scan/claim/sweep/show/record plus groups.json. It owns wave planning, one-group investigations, adversarial verification, serial product-fix application and final build/report orchestration. Existing-owner-first therefore keeps the helper under the already-qualified crash-report process owner plus repository process/quality evidence governance.

## Failure and recovery facts

The source has no explicit inter-process lock. Processing markers provide interrupted-run recovery for serialized orchestration, not proof of concurrent-invocation safety. Git read failures return empty text; no usable default tag is terminal. Unreadable logs are skipped. Missing or malformed local verdict ledger JSON returns an empty ledger, so subsequent record can lose prior decisions; this is recorded as a lossy recovery hazard rather than successful corruption recovery. Default sweep moves old-version/no-frame/unsymbolized triples; --delete is the only destructive-delete authority; --dry-run is non-mutating.

Raw crash logs/dumps, UserTag/Username, addresses and module lists remain local repository-quality data and are not Fabushi product state or broad migration evidence.

## Deferred Stage-B qualification

Authored only, not run:
- nine contract cases covering host authority, version resolution, claim rotation, parse/classify, signature/grouping, repository cross-reference, output routing, ledger record and show/sweep;
- one integration composition case with the existing crash-report command;
- four fault-recovery cases for interrupted claim, incomplete triple, malformed ledger and no-tag resolution;
- one security/privacy case for the local crash-data boundary.

Ledger state is `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`. Coverage moves exactly one independent source entry: unknown `16129 -> 16128`, unread `16102 -> 16101`, omitted stays 0, responsibilities `149 -> 150`, unmapped remains 0.

No Actions/check/status/workflow query or trigger, crash helper execution, git subprocess, crash-file read/move/delete, local ledger write, build, compile, typecheck, lint, validator, test, packaging, deployment, installation, runtime evidence or acceptance was performed. The next fixed-U blob is `.claude/settings.json@d47520dc0e3b7e8d6067456e33062140c75f7c79`.
