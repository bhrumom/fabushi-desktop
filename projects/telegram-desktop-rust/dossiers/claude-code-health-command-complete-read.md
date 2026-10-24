# Claude code-health command complete read — strict index 59

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Strict zero-based root index: `59`.
- Source: `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a`, 14 lines, fully read.
- Semantic owner re-read at the same upstream commit: `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097`.
- Required host adapter re-read at the same upstream commit: `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df`.
- Parent Fabushi exact head: `e2c301c1e2b032ec024e378c4fe754afa2ddc46e`, tree `053ac3214a96dc24e634c08b774aa8339ef205e2`.
- Parent TDRP source-authority run `39703486928`: terminal success at `e2c301c1e2b032ec024e378c4fe754afa2ddc46e`, 61/61 jobs success. Key artifacts: authority `12393762939` / `sha256:725d002f4221fb2b74d86920c7bf82c5a294dcdaa2a834c78fac72fae06ca70d`; recursive reachability `12393699382` / `sha256:c2a42b532bdfff1ed771bffdeceba8bd2dbd0818d9ff21ae99a4747fea946cf0`; recursive closure `12393968351` / `sha256:be6738ccc01564cc7f4a1eb7c338d7a81bc332b0c8c9b5deb8cb487f40f16163`; root source archives `12394305828` / `sha256:6c9a310196a2442c19a32d5120b0076911ed28262d608e6cfb73fbacdb5ab92c`.
- Parent Rust desktop runtime run `39703486941`: terminal success at `e2c301c1e2b032ec024e378c4fe754afa2ddc46e`, 6/6 jobs success; artifact `12393374651` / `sha256:4439d60c577b28bb695a4c9d6e89388e0fee50562e66cbb4261d3e75c55ea8a0`.
- Parent evidence becomes historical-only after this mapping commit; the child head requires fresh exact-head Actions.

## Complete source responsibility
The file is a Claude Code command alias/invocation surface, not a scanner, findings ledger, task queue, scheduler, product runtime, or independent code-health semantic owner.

Its complete behavior is:
1. declare the host-facing description and allowed Claude tools;
2. require a complete read of the Claude workflow adapter and the shared code-health skill before execution;
3. delegate all semantics to that shared skill with only the adapter's host-mechanics substitutions;
4. map no arguments or `scan` to scan mode, `survey [area]` to one area survey, and `check [base]` to a change check;
5. permit proposal routing only when the invocation explicitly includes `route`;
6. identify the scheduled form as `scan route`;
7. pass the literal `$ARGUMENTS` to the canonical workflow.

No product code, persistence, network contract, search, UI, identity/account state, messaging state or release runtime is implemented by this command file.

## Caller/dependency and owner audit
The shared code-health skill owns fresh exact-source scanning, stable findings, survey/atlas/map work, quality ratchets, proposal trust/routing, audits and report ordering. The just-closed Claude adapter owns provider-specific delegation, effort/model truth, Codex/native review fallback, source-lineage boundaries and worker lifecycle. This command only selects an already-supported mode and exposes the host invocation.

Fabushi already owns the applicable responsibility in root `AGENTS.md` under repository quality-health native invocation. To avoid assigning a second responsibility to the heading already used by the earlier OpenAI registration record, this row anchors to two exact unoccupied sentences inside that existing owner:
- `A default prompt may select an already-supported mode, but it must not weaken the canonical skill/spec, let stale or mutable observations advance durable state, bypass ordinary routing approval/trust gates, or reorder reporting so proposals hide the regressions and decisions that precede them.`
- `Scheduled and ad-hoc invocations enter the same ordinary task/review/evidence lifecycle.`

Rejected owners:
- shipping Agent/Coordinator/Host/Runner, because this is repository-development command registration, not product execution;
- shipping Task/Automation/Workflow, because code-health proposals enter the ordinary repository task lifecycle rather than the end-user task store;
- a Claude-specific scanner, findings store, scheduler or task queue, because the command explicitly delegates semantics to the shared skill;
- a second code-health workflow owner, because scan/survey/check and routing semantics already exist in the source-neutral owner.

## Fail-closed equivalent contract
`Claude /code-health command -> complete adapter + shared-skill read -> supported mode selection -> canonical code-health owner -> route only when explicitly requested -> ordinary repository review/evidence lifecycle`

The command must not:
- let provider syntax become semantic ownership;
- invent a mode outside the canonical skill;
- route proposals merely because a scan or survey ran;
- let stale/mutable observations advance durable findings;
- bypass trust, ownership-conflict, audit, approval or task-count gates;
- hide regressions/decisions behind proposals;
- treat command registration itself as product implementation or passing release evidence.

## Traceability
- Requirement/responsibility: `TDRP-R9-CLAUDE-CODE-HEALTH-COMMAND-001`
- Oracles: `ORA-TDRP-CLAUDE-CODE-HEALTH-MODE-SELECTION-001`, `ORA-TDRP-CLAUDE-CODE-HEALTH-EXPLICIT-ROUTE-001`, `ORA-TDRP-CLAUDE-CODE-HEALTH-SHARED-OWNER-001`
- Invariants: `INV-TDRP-CLAUDE-CODE-HEALTH-NO-SECOND-OWNER-001`, `INV-TDRP-CLAUDE-CODE-HEALTH-NO-IMPLICIT-ROUTE-001`, `INV-TDRP-CLAUDE-CODE-HEALTH-HOST-SYNTAX-ONLY-001`
- Contract cases: `CONTRACT-TDRP-CLAUDE-CODE-HEALTH-SCAN-001`, `CONTRACT-TDRP-CLAUDE-CODE-HEALTH-SURVEY-001`, `CONTRACT-TDRP-CLAUDE-CODE-HEALTH-CHECK-001`, `CONTRACT-TDRP-CLAUDE-CODE-HEALTH-ROUTE-001`
- Regression: `REG-TDRP-CLAUDE-CODE-HEALTH-NO-ROUTE-WITHOUT-ARG-001`

Closing only strict index 59 changes counters to unknown=16,244, unread=16,219, omitted=0, responsibilities=77. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root entry: index 60 `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8`.
