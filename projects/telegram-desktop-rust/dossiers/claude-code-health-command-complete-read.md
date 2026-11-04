# Fixed-U complete read — Claude code-health command adapter

- Source repository: `telegramdesktop/tdesktop`
- Fixed source commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Source path: `.claude/commands/code-health.md`
- Blob: `bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a`
- Recursive fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`
- Tree position: index 88 zero-based
- Identity: 598 ASCII/UTF-8 bytes, 598 decoded characters, 19 newline characters, trailing newline, 20 split lines
- Stage: A; executable validation deferred
- Desktop authority parent: `be1db0ca4d4e30a9a5e32b2b288d333d7518ffa8`
- Platform-core authority: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`

## Complete source semantics

The command front matter describes a code-health measurement/survey/check entry and exposes `Read, Write, Edit, Glob, Grep, Bash, Agent, AskUserQuestion` to the Claude host. The body does not implement quality measurement or state. It requires a complete read of the generic Claude workflow adapter and the shared code-health skill, then delegates semantic behavior to the shared skill while applying only the adapter's Claude-specific delegation substitutions.

The fixed-U mode contract is exact: no arguments or `scan` means scan; `survey [area]` means one area survey; `check [base]` means a change check. `$ARGUMENTS` is the host-provided input. Proposal routing is separately fenced: route only when the arguments say `route`; the scheduled invocation is `scan route`.

The tool allowlist is host metadata, not semantic ownership or execution authorization. Under FMEP-001 Stage A it cannot be used to run a scan, survey, check, helper, Agent worker, route/publish action, build, validator, test, or acceptance path.

## Direct consumers and dependencies

Direct dependencies:
- `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df`
- `.agents/skills/code-health/SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f`

Direct host consumer: Claude Code command discovery/user slash-command invocation for `/code-health`. Repository search finds no fixed-U file-level import/reference to `.claude/commands/code-health.md` by path. The shared code-health skill is the invocation contract source and documents `/code-health scan route` for the scheduled run plus `/code-health survey <area>` for manual survey. `.agents/skills/code-health/agents/openai.yaml@59bb6304445f21e2a3ee1a97a69a8e231f934d30` is a peer `$code-health` host registration, not a consumer. Root `CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155` only points to `AGENTS.md`.

## Existing-owner-first mapping

New source responsibility: `TDRP-R9-CODE-HEALTH-CLAUDE-COMMAND-ADAPTER-001`.

Existing owners:
- repository quality-health command orchestration (`TDRP-R9-CODE-HEALTH-COMMAND-ORCHESTRATION-001`);
- repository AI workflow host-adapter authority (`TDRP-R9-REPOSITORY-AI-WORKFLOW-HOST-ADAPTER-001`);
- repository quality/test-evidence governance.

Rejected owners:
- shipping Product Tasks or Automations;
- Agent/Coordinator/Host/Runner product runtime;
- platform-core runtime;
- a Claude-specific scanner, findings ledger, task queue, approval store, or publication store;
- a second repository AI workflow/state authority.

There is no new durable state owner, ADR, shipping entrypoint, service, or platform-core production delta. Canonical health state, findings, proposals, owner decisions, inbox routing, and publication remain with the existing source-neutral repository quality/process owners.

## Authored contracts — not run

- `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-INVOCATION-001`
- `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-DELEGATION-001`
- `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-ROUTING-001`
- `INT-TDRP-CODE-HEALTH-CLAUDE-COMPOSITION-001`
- `FAULT-TDRP-CODE-HEALTH-CLAUDE-ROUTE-FENCE-001`

All remain `validation_status=not-run`, `accepted=false`.

## Failure and recovery boundaries

Fail closed if either required source is not read completely, if the host wrapper invents semantic state or approval, if a plain scan/survey/check implicitly routes proposals, if an unsupported/ambiguous argument is broadened into publication authority, if host delegation mechanics replace the shared skill, or if Stage-A source review is promoted to runtime verification. Recovery is to re-enter through the same existing quality/process owners; never create a second command-state or findings truth.

## Source-level disposition

This is repository development/quality governance only. Source-level responsibility is implemented by the existing Fabushi quality/process owner contracts plus this host-adapter qualification. Executable proof is intentionally absent until Stage B.
