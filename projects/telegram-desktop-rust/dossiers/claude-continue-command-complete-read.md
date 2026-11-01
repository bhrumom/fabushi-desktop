# Fixed-U complete read — Claude continue command adapter

- Source repository: `telegramdesktop/tdesktop`
- Fixed source commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Source path: `.claude/commands/continue.md`
- Blob: `e81c86031f2e501f30d2b6ed908fb03d648324c8`
- Recursive fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`
- Tree position: index 89 zero-based
- Identity: 3,700 ASCII/UTF-8 bytes, 3,700 decoded characters, 67 newline characters, trailing newline, 68 split lines
- Stage: A; executable validation deferred
- Desktop authority parent: `2c24fdb00934402bc6ac204d6856fe1e1a41a5c7`
- Platform-core authority: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`

## Complete source semantics

The front matter describes a Claude Continue entry and exposes the Claude host capability envelope. The body does not define a new durable scheduler. It requires complete reads of the generic Claude workflow adapter and shared continue skill, then applies Claude-specific delegation/text-host substitutions while preserving the shared skill as the task/process authority.

Without a batch specification, the wrapper stops after the shared read-only queue call, reports the queue, and asks for a specification. With a specification, it chooses exactly one shared-skill mode, freezes task ids with any active task first, and never admits unrelated tasks observed later. Only canonical split replacements and follow-ups routed from results produced by this invocation, including transitive discovered follow-ups, may extend the effective batch.

The wrapper explicitly composes shared split-required, source-lineage, consolidation, performer, discovered-routing, and review-provider behavior. Source-lineage decisions at startup and safe pre-Phase-1 switching stay in the parent scheduler; post-Phase-1 mismatch remains task-local. It forbids branch-integration work from becoming queue tasks. One stateful performer runs at a time. Split routing is fresh/foreground. Discovered routing and consolidation use the shared routing worktree lane with at most one worker of each type in flight. Publication-owning workers must first read the generic Claude adapter completely.

Review provider is selected once at startup: explicit Codex opt-out => `native`; otherwise `codex`. The choice is forwarded to every performer and cannot change canonical task/process semantics.

## Direct consumers and peers

Direct host consumer: Claude Code command discovery/user slash-command invocation for `/continue`. Repository code search finds no fixed-U file-level import/reference to `.claude/commands/continue.md` by path.

Direct dependencies:
- `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df`
- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5`

Peer host adapters, not callers:
- `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61`
- `.grok/commands/continue.md@1365e5a7dc4c167468f7b6f57ec92ff89389b32e`

Root `CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155` only points to canonical `AGENTS.md`.

## Existing-owner-first mapping

New source responsibility: `TDRP-R9-CONTINUE-CLAUDE-COMMAND-ADAPTER-001`.

Existing owners:
- repository continue task/process scheduler authority (`TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`);
- repository AI workflow host-adapter authority (`TDRP-R9-REPOSITORY-AI-WORKFLOW-HOST-ADAPTER-001`);
- repository quality/test-evidence governance.

Rejected owners:
- shipping Product Tasks / Automations;
- Agent / Coordinator / Host / Runner product runtime;
- platform-core runtime;
- Claude-specific task queue/scheduler/task-state store;
- Claude-specific routing, review, evidence, or publication store;
- any second repository workflow authority.

No new durable state owner, ADR, service, shipping entrypoint, or platform-core production delta is introduced.

## Authored contracts — not run

- `CONTRACT-TDRP-CONTINUE-CLAUDE-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-CONTINUE-CLAUDE-BATCH-FREEZE-001`
- `CONTRACT-TDRP-CONTINUE-CLAUDE-SOURCE-LINEAGE-001`
- `CONTRACT-TDRP-CONTINUE-CLAUDE-DELEGATION-001`
- `CONTRACT-TDRP-CONTINUE-CLAUDE-REVIEW-PROVIDER-001`
- `CONTRACT-TDRP-CONTINUE-CLAUDE-ROUTING-LANE-001`
- `INT-TDRP-CONTINUE-CLAUDE-COMPOSITION-001`
- `FAULT-TDRP-CONTINUE-CLAUDE-DUPLICATE-SCHEDULER-001`

All remain `validation_status=not-run`, `accepted=false`.

## Failure and recovery boundaries

Fail closed when either required dependency is not read completely; when no specification mutates anything beyond the read-only queue path; when later queue refresh widens the batch; when another checkout's task is stolen; when more than one performer is active; when source-lineage is bypassed or converted into branch-integration queue work; when split/routing/consolidation publication is partially applied or left unlanded; when provider choice changes shared artifacts/verdict semantics; or when Stage-A source review is promoted to executable evidence.

Recovery re-enters the existing shared continue/process owners and canonical helper/publication boundaries. Never create a second scheduler/state truth.

## Source-level disposition

Repository-development process governance only. Source-level responsibility is implemented by existing Fabushi process/quality owners plus this Claude host-adapter qualification. Executable proof remains deferred until Stage B.
