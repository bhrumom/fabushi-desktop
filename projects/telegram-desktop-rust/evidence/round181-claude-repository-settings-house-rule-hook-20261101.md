# Round 181 — fixed-U Claude repository settings / house-rule PostToolUse policy

- Task round: 150
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `2aab8971ee0c56ac998ced67a468d6905f0e519c`
- Spec-first desktop commit: `1760d9a6716dc9e23c75bd49971ca26d65506ff1`
- Qualification-binding commit: `e614a883daa1b27bbda87abad4ccc8b576a94c8c`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, complete, 7,194 entries / 6,859 blobs
- Closed source: `.claude/settings.json@d47520dc0e3b7e8d6067456e33062140c75f7c79`
- Tree index: 101 zero-based; mode 100644
- Source identity: 321 Git bytes / 321 decoded ASCII characters / 16 newline characters / trailing newline / 17 split lines
- Direct host consumer: Claude Code repository settings discovery / Hook dispatcher
- Direct repository helper: `tools/rules/hook.py@287b36c948566659970651226c92f7b8b056b434`
- Canonical rule dependency: `.clang-tidy@5610bd13664ca77549993def241eb3edaffeff7a`
- Helper oracle: `tools/rules/test_hook.py@dce99f3507e4ec9f71c8bbd5360e9dde9b62d45f`
- Responsibility: `TDRP-R9-CLAUDE-REPOSITORY-SETTINGS-HOUSE-RULE-HOOK-001`

## Verified source and composition

The complete settings file was read through EOF. It defines one `PostToolUse` hook for exact matcher `Edit|Write|MultiEdit`; the command is `python3 "$CLAUDE_PROJECT_DIR/tools/rules/hook.py"` and status text is `Checking house rules`. No other settings keys exist, so this exact source does not define an explicit allow/deny permission model or pre-use authorization gate.

The direct helper was read completely for composition. In edit mode it consumes the host JSON payload, checks only `.cpp/.h/.mm/.cxx`, derives banned CustomFunctions from canonical root `.clang-tidy`, and applies repository comment rules. A rule violation returns corrective nonzero output. Malformed JSON returns 0 and non-source paths return 0. Its separate `--staged` mode is reused by sibling quality flows, but settings does not pass that flag.

Existing-owner-first keeps the already-qualified Claude host adapter, static-analysis policy owner, root repository development-process/security instructions, and repository quality/test-evidence lifecycle authoritative. No product Settings/Permissions, Agent/Computer/platform authorization owner, Telegram-specific policy store or duplicate findings registry is introduced.

## Failure and security facts

Because this is `PostToolUse`, a violating write may already have occurred before the helper reports correction. It is not a transactional pre-write barrier. Missing interpreter/project-dir/helper dispatch and malformed payload cannot be called successful rule enforcement. The source carries no secrets, network endpoint, durable product permission state, hidden retry, watcher, autonomous rollback, or remote service.

## Deferred Stage-B qualification

Authored only, not run:
- six contract cases for host authority, matcher scope, hook command, static-policy composition, post-use boundary and permission non-expansion;
- one integration composition case;
- two fault-recovery cases for malformed payload and unavailable hook;
- one security case for tool-scope/authorization boundaries.

Ledger state is `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`. Coverage moves exactly one independent source entry: unknown `16128 -> 16127`, unread `16101 -> 16100`, omitted stays 0, responsibilities `150 -> 151`, unmapped remains 0.

No Actions/check/status/workflow query or trigger, Claude hook invocation, Edit/Write/MultiEdit execution, Python/git subprocess, clang-tidy, build, compile, typecheck, lint, validator, test, packaging, deployment, installation, runtime evidence or acceptance was performed. The next fixed-U blob is `.devcontainer.json@16a1bcc1192bed87c5211da3dc63b0b0ed99a93e`.
