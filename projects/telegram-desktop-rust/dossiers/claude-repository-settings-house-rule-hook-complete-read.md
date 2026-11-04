# Fixed-U complete read — Claude repository settings / house-rule PostToolUse policy

- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (complete, 7,194 entries / 6,859 blobs)
- Source: `.claude/settings.json@d47520dc0e3b7e8d6067456e33062140c75f7c79`
- Recursive-tree index: 101 zero-based
- Mode: `100644`
- Source identity: 321 Git bytes / 321 decoded ASCII characters / 16 newline characters / trailing newline / 17 split lines
- Direct host consumer: Claude Code repository settings discovery / hook dispatcher
- Direct repository helper: `tools/rules/hook.py@287b36c948566659970651226c92f7b8b056b434`
- Canonical rule dependency: `.clang-tidy@5610bd13664ca77549993def241eb3edaffeff7a`
- Source-side helper tests: `tools/rules/test_hook.py@dce99f3507e4ec9f71c8bbd5360e9dde9b62d45f`
- Responsibility: `TDRP-R9-CLAUDE-REPOSITORY-SETTINGS-HOUSE-RULE-HOOK-001`

## Complete source semantics

The complete JSON was read through EOF. It has one top-level key, `hooks`; one configured event, `PostToolUse`; one matcher, `Edit|Write|MultiEdit`; and one command hook, `python3 "$CLAUDE_PROJECT_DIR/tools/rules/hook.py"`, with status text `Checking house rules`. No other keys exist. In particular there is no explicit allow/deny permission list and no pre-use hook.

The direct host consumer is Claude Code repository-settings discovery/hook dispatch. The command invokes `tools/rules/hook.py` without `--staged`. In edit mode the helper reads JSON stdin, derives the path from host payload fields, checks only `.cpp/.h/.mm/.cxx`, reads root `.clang-tidy` CustomFunctions for banned replacements, and enforces comment rules. Policy violations return corrective nonzero output. Malformed JSON returns 0; non-source paths return 0. These are explicit source scope/fail-open facts, not permission/security guarantees.

The helper's separate `--staged` mode is reused by the fixed-U clang-tidy pre-commit recipe and the already-qualified cross-review command, but those are sibling consumers of the helper, not callers of this settings file. Source-side `tools/rules/test_hook.py` has 34 test functions covering edit/staged comment budgets, WHY blocks, moved comments, harness exemption, banned functions/replacements, false-positive exclusions, non-source paths, malformed payload, new-file license handling and MultiEdit; those tests were read as an oracle only and not run.

Existing-owner-first composes the existing Claude host adapter, canonical static-analysis policy, root repository process/security instructions, and repository quality/test-evidence governance. Rejected owners include product Settings/Permissions, Product Tasks/Automations, Agent/Coordinator/Host/Runner runtime, Computer permissions, platform-core auth/service, Telegram-specific tool-policy state, and any second quality/security findings registry.

Because `PostToolUse` is post-use, a violating edit may already exist before correction is signaled. Missing Python/script/project-dir resolution, dispatch failure, or malformed payload cannot be called a successful rules check. Recovery is correction/revert through existing repository process; no hidden retry, watcher, autonomous rollback, persistence, service, or permission escalation exists in this source.

Authored, not run: six contract cases, one integration case, two fault-recovery cases and one security case matching the IDs recorded in repository quality governance. No Claude hook invocation, edit/write execution, Python/git subprocess, clang-tidy, build, lint, validator, tests, CI, packaging, deployment, installation, runtime evidence or acceptance was performed.
