# Complete read — .agents/skills/continue/agents/openai.yaml

Status: independently understood and mapped; exact-head verification pending
Project: TDRP-001 Revision 9
Upstream: `telegramdesktop/tdesktop@78ceaa548ae0e055aa56b53fde22064d13f250ef`
Upstream tree: `76b967833b38eb7aa2cb4047008c07331390e61e`
Source: `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61`

## Inventory identity

The live upstream `dev` ref was re-read immediately before this write and still resolves to `78ceaa548ae0e055aa56b53fde22064d13f250ef` / tree `76b967833b38eb7aa2cb4047008c07331390e61e`. PR #154 and branch `tdrp-r9-entry31-includes-r27-20261020` both still resolve to parent `f605a520cb7d3dc21da795a35cb89b8479afdb8f` / tree `dfc652a92b418ca32bc64924d1b7f1b3b3380a07`.

Zero-based recursive authority index 36 is `.agents/skills/continue/agents/openai.yaml`, blob `e981285ee10cbed9e27e358d60ce596c9e0b3d61`, immediately after `.agents/skills/continue/SKILL.md` index 35. The next strict entry is index 37, `.agents/skills/continue/references/consolidate-pending-tasks.md@bd301d93f8f670ad041b3db72330e17cbb50eb5c`.

Parent exact-head evidence is historical-only for this new mapping commit: TDRP run `39486129504` completed 61/61 jobs and 506/506 steps successfully at `f605a520cb7d3dc21da795a35cb89b8479afdb8f`; Rust desktop runtime `39486129565` completed 6/6 jobs, 79 successful steps and two expected platform-conditional Linux-perf skips. The 60 TDRP artifacts plus one Rust artifact were already downloaded at the parent and their actual ZIP SHA-256 values matched GitHub digests. Main authority artifact `12292833722` is `sha256:bdb9370eabb49b7e9cecc271c6e078c1aa7dd2f67c6db5b4879cc1a42195c9fe`; Rust artifact `12294430499` is `sha256:c6644bd99fbb542e629eb3d3fecd70627477cb31443faaed08a2b3fd6999ce9f`.

## Complete source read

The entire registration YAML is four semantic fields under one `interface` object:

- `interface.display_name = "Continue AI Work"`;
- `interface.short_description = "Resume active work, then run a specified batch"`;
- `interface.default_prompt = "Use $continue to place any active task first, freeze exactly the tasks my batch specification selects behind it, and process the batch one task at a time."`.

The file owns no queue state, source checkout, task publication, agent execution, persistence, product runtime, UI, network protocol, or release decision. Its independent responsibility is host-native discovery/presentation plus a default invocation selector for the already-authoritative `$continue` workflow.

## Real invocation and adapter trace

Repository search found no in-repository parser or literal path caller for this exact YAML and no separate test that executes the YAML as code. That is expected for host-native registration metadata: the host discovers the manifest, while actual continuation behavior is owned by the shared skill and host command adapters.

The real invocation chain was re-read from the same upstream commit:

1. `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5` owns frozen-batch admission, source-lineage fencing, one stateful performer, split/discovery routing, pending consolidation, blocker recovery and archival.
2. `.grok/commands/continue.md@1365e5a7dc4c167468f7b6f57ec92ff89389b32e` accepts `/continue` / `$continue`, requires the explicit batch specification and delegates through the shared skill.
3. `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8` binds the same shared skill to Claude Agent mechanics, source-lineage handling, split/consolidation and review-provider selection.
4. `.grok/ai-workflow-adapter.md@db447f204316d3991f6d20db3d8f097fcc021154` and `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` adapt child/worker lifecycle without changing queue semantics.
5. `.agents/shared/codex-cli-leaves.md@659da44a8083c433c4e9b59ca28f64ba525bfbf8`, `.agents/shared/scripts/codex_leaf.py@313c4941c61a4ec0e3b3b65cdaa59cc213916fdd`, and `.agents/shared/scripts/codex_leaf_test.py@2831841e6b131ca2c42688fe24e00b739f825ccb` define and exercise the related cross-host out-of-process Codex review/assessment adapter; they are supporting AI-invocation evidence, not a parser for this YAML.
6. `.agents/skills/process-inbox/scripts/workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c` remains the durable repository task/publication state-machine owner, with direct lifecycle regressions in `workspace_test.py@d045c95ca4415ffc02953ff02ac10e329dc69e7b`.

Therefore the YAML's default prompt is a convenience selector, not an alternate scheduler.

## Existing-owner-first resolution

Current Fabushi exact-head already contains source-neutral host-native invocation governance:

- `AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b`: "Host-native skill registration metadata, command aliases, and default prompts are invocation surfaces only." It requires those surfaces to resolve to an existing source-neutral repository workflow owner.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb`: native skill-surface qualification proves discovery resolves to the existing owner and rejects a second implementation/state owner.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70`: native registration evidence records display/description/default-prompt metadata and the authoritative routing contract.
- Existing repository task/evidence orchestration from `AGENTS.md` plus the index-35 mapping owns the `$continue` batch/state semantics.

Exact-head shipping AI owners were also inspected and deliberately rejected as owners for this registration responsibility: `source/host/src/extensions/transcript/run_scheduler.rs@eb8994dc45b3722e8bfcc672e9e610c662429650` owns per-agent product run lanes/watchdog/settlement, while Runner prompt and turn composition files such as `runner_prompt_glue.rs@fed229ea2c4226eba00466bd5413ec9b85c6aee5`, `system_prompt_assembly.rs@6d592794b835994a81a214049b9c5907df0cb4ff`, and `turn_agent_composition.rs@eefde7f77856070e62e4f8dbca7cfb7f28da0a17` own shipping provider/tool/prompt composition. None owns repository skill registration, branch/source-lineage truth, or AI task publication.

No new owner, second queue, second prompt/runtime stack, or source-specific adapter is needed.

## Fabushi-equivalent contract

`host discovers registration -> shows name/description -> default prompt selects $continue -> canonical repository continuation owner freezes the requested batch -> existing task/evidence lifecycle executes one task at a time`

Fail-closed rules:

- registration/default prompt cannot mutate or replace canonical queue/task state;
- provider syntax such as `$continue` is adapter provenance, not semantic ownership;
- default invocation cannot silently broaden the frozen batch or bypass source-lineage, ownership, split/consolidation, trust or evidence rules;
- host/worker completion cannot substitute for task/product/release evidence;
- shipping Agent/Runner/Automation/Workflow stores cannot be repurposed into a repository-development queue.

## Traceability

- Requirement: `TDRP-R9-CONTINUE-NATIVE-REGISTRATION-001`
- Oracles: `ORA-TDRP-CONTINUE-NATIVE-OWNER-001`, `ORA-TDRP-CONTINUE-DEFAULT-PROMPT-001`
- Invariants: `INV-TDRP-CONTINUE-NATIVE-NO-SECOND-OWNER-001`, `INV-TDRP-CONTINUE-NATIVE-FROZEN-BATCH-001`, `INV-TDRP-CONTINUE-NATIVE-NO-SHIPPING-QUEUE-COLLISION-001`

## Verification disposition

This closes only recursive index 36. Counters become recursive=16,201, unknown=16,162, unread=16,137, omitted=0. Baseline identity remains unaccepted and `source_closure_ready=false`.

Exact-head acceptance of this mapping requires new TDRP Revision 9 source-authority and Rust desktop runtime workflows on the new commit. The authority run must continue to prove the live upstream/tree, 36 direct gitlinks plus all nested gitlinks, cppgir/expected-lite inclusion, exact recursive reconciliation, fixed-point `unresolved=0`, and Boost `582/582/open=0`. Remaining recursive acquisition/generated/resource/tool disposition is still open, so this source-entry closure cannot set full source closure.
