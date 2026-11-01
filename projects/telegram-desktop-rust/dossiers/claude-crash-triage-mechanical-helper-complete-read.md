# Fixed-U complete read — crash-triage mechanical helper

Source: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4:.claude/scripts/crash_triage.py
Blob: 2db6cc14d8e529c4e61c5dcf169aade31fd35c58
Tree: 91f2b083ab2877c0a17d07868eabec362a5cfca6; index 100 zero-based; mode 100755; 38,738 Git bytes; 38,738 decoded ASCII characters; 1,030 newlines; trailing newline; 1,031 split lines.
Direct caller/consumer: .claude/commands/crash-reports-process.md@5ac555406670721694e78c4e1e6b2a96b3e98fb5.
Existing caller owner: TDRP-R9-CRASH-REPORT-PROCESS-CLAUDE-COMMAND-ADAPTER-001 (spec-first commit b996f372412e78272a180c8ed8dc878e73565aa7).
Stage: A; validation not run; accepted false.
Desktop parent: c7b420e19fa091cadc149b236bab90db45b6177f
Platform-core: e92b51e23b39dae5719f033fa1f7a20d6579dc8c

## Acquisition and run rotation

The default corpus is ~/Telegram/Crashes/all. A crash id is claimable only when log_<id>.txt, err_<id>.txt and dmp_<id>.dmp all exist. complete_triples sorts ids and leaves partial triples untouched. claim expands the inbox/backups paths, creates the backup root, resumes the first existing .processing-* folder if present, otherwise creates a timestamped processing folder, moves each still-present member of every complete triple, and counts one claimed id per ready triple.

After movement, a zero-report processing folder is removed and the latest prior run is reused unless --new was requested. With no prior run, claim returns run_dir=None and report_claim returns failure. Otherwise the processing folder is renamed to its final <stamp>-crashes name, incomplete inbox logs are counted as left behind, and the result exposes run_dir, claimed, reports, reused and left. This is the source's interruption/recovery boundary.

## Version resolution and parsing

resolve_target accepts x.y.z, numeric release encoding, or no version. Explicit semantic versions become v<x.y.z>; explicit numeric values above 100000000 are divided by 1000 to normalize closed-alpha numbering. Both base and base*1000 are accepted. Default resolution asks git for version-sorted v*.*.* tags, exits when none exist, prefers AppVersion from Telegram/build/version at the selected tag, and otherwise computes major*1000000 + minor*1000 + patch. The git helper returns empty text on command failure or missing git.

parse_log reads text with errors=replace. It extracts selected headers, Crash reason/address, Process uptime, OS, CPU, loaded module path fields, crashed-thread frames, optional file/line and Found by metadata. OSError returns None. Version is the leading numeric prefix of the Version header and beta is derived from the raw header.

## Classification, signatures and routing hints

classify is exact: version outside accepted -> old-version; <no frames> or empty frames -> no-frames; no symbolic frame -> unsymbolized; otherwise usable.

signature first prefers frames not marked Found by: stack scanning. It trims leading assertion/crash/allocation machinery but restores the original frame list if trimming would remove everything. canonical normalizes MSVC/GCC lambda/scope spelling and mines non-standard qualified application names out of forwarding frames. Consecutive duplicate keys collapse. A non-generic Assertion is the primary cross-platform key; generic assertions are combined with the loose stack; otherwise the loose canonical stack is the key. stable_key strips source line numbers and uses the first 12 hex characters of SHA-1.

hints emits routing metadata for third-party interceptor modules, clone binaries, GPU-driver frames, OOM, deadlock detector, Qt fatal, pure virtual, Win32 and assertion. The helper itself does not assign fixed/not-actionable/needs-discussion verdicts; that remains with the caller process.

## Repository cross-reference and scan outputs

RepoIndex is read-only against the repository. It prefers git ls-files --recurse-submodules, falls back to ls-files, indexes tracked paths by basename, and records git diff --name-only <tag>..<against>. lookup prefers Telegram/SourceFiles then Telegram/lib_* over other same-name paths; ambiguous matches are joined from up to three candidates. source_map emits distinct resolved files, source lines, tracked paths and touched-since-tag flags.

cmd_scan optionally claims first, resolves the target version, creates <out>/results, parses sorted log_*.txt files, classifies every readable report, groups usable reports by signature, accumulates exact stack/reason/platform/hint/user counts, and keeps the richest trimmed stack as representative. Existing ledger keys mark groups known. Fresh groups are ordered before known groups, then by report count, and receive G01... ids.

The scan writes index.tsv for report-level routing, groups.md for human summaries and groups.json for tooling. groups.json includes accepted_versions, corpus/out paths, bucket counts, drop ids for old-version/no-frames/unsymbolized, fresh gids, group key/known/count/machines/assertion/signature/tags/representative/members/frames/sources. These are local triage artifacts, not Fabushi product state.

## Ledger, show and sweep

The ledger path defaults to ~/Telegram/Crashes/triage-ledger.json. load_ledger returns an empty dict for a missing file, read error or malformed JSON. save_ledger writes sorted indented JSON to <ledger>.tmp and os.replace swaps it into place. record requires a known gid from groups.json, carries forward first_seen/seen/runs, updates signature/verdict/last_seen, optional note/commit, adds the run folder once, then saves and prints the recorded key. Because malformed input is treated as empty, a subsequent record can overwrite prior decisions; that is source behavior to expose, not successful corruption recovery.

show loads groups.json, selects requested gids (or all), reads the representative log from the run corpus with an old-version dropped fallback, hides module lists unless requested, hides non-crashed threads unless requested, hides register and Found-by lines unless requested, enforces --lines, and exits for unknown requested groups.

sweep consumes groups.json drop_ids. By default it moves old-version,no-frames,unsymbolized report members into <out>/_dropped/<kind>; --delete removes them instead; --dry-run prints without mutation; --kinds narrows categories; missing files are skipped. The count is per listed report id, not per file successfully moved.

## Ownership and Stage-A disposition

Responsibility: TDRP-R9-CRASH-TRIAGE-MECHANICAL-HELPER-001.
Existing owners: TDRP-R9-CRASH-REPORT-PROCESS-CLAUDE-COMMAND-ADAPTER-001; Fabushi repository development process governance; repository quality/test-evidence governance.
Rejected owners: shipping crash telemetry/upload or product crash persistence; Product Tasks/Automations; Agent/Coordinator/Host/Runner; platform-core runtime/service; second crash queue/ledger/evidence store.
No platform-core production delta, service, product UI, database, ADR or shipping runtime is introduced.

Repository code search found the crash-report process command as the file-level reference to crash_triage.py, and that exact fixed-U caller was read completely. The caller invokes scan --claim, sweep, show and record and consumes groups.json; it owns wave planning, isolated investigations, adversarial verification, serial product-fix application and final build/report orchestration. The helper owns only the mechanical local-corpus state transitions and generated triage data described above.

## Authored, not run

CONTRACT-TDRP-CRASH-TRIAGE-HOST-AUTHORITY-001
CONTRACT-TDRP-CRASH-TRIAGE-VERSION-RESOLUTION-001
CONTRACT-TDRP-CRASH-TRIAGE-CLAIM-ROTATION-001
CONTRACT-TDRP-CRASH-TRIAGE-PARSE-CLASSIFY-001
CONTRACT-TDRP-CRASH-TRIAGE-SIGNATURE-GROUPING-001
CONTRACT-TDRP-CRASH-TRIAGE-REPO-CROSSREF-001
CONTRACT-TDRP-CRASH-TRIAGE-OUTPUT-ROUTING-001
CONTRACT-TDRP-CRASH-TRIAGE-LEDGER-RECORD-001
CONTRACT-TDRP-CRASH-TRIAGE-SHOW-SWEEP-001
INT-TDRP-CRASH-TRIAGE-COMMAND-COMPOSITION-001
FAULT-TDRP-CRASH-TRIAGE-INTERRUPTED-CLAIM-001
FAULT-TDRP-CRASH-TRIAGE-INCOMPLETE-TRIPLE-001
FAULT-TDRP-CRASH-TRIAGE-MALFORMED-LEDGER-001
FAULT-TDRP-CRASH-TRIAGE-NO-TAG-001
SEC-TDRP-CRASH-TRIAGE-LOCAL-DATA-BOUNDARY-001
