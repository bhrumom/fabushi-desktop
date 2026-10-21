# Perform-task skill complete read — strict recursive index 45

Status: parent exact-head verified at b8ceacbe9390d182ab61019ba9c75601e5f02426; strict index 45 source-grounded and existing-owner mapped; child exact-head verification pending.

Authority: telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844 tree 1fc479a4b448aed851181bf4a4796d3015368348.
Source: `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` (234 lines).
Next strict entry: index 46 `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`.

## Parent exact-head evidence

TDRP Revision 9 run 39518408374 is terminal success on b8ceacbe9390d182ab61019ba9c75601e5f02426. Attempt 1 exposed only two external Chromium acquisition failures: root shard 7 breakpad and root shard 1 linux-syscall-support both returned HTTP 503 from chromium.googlesource.com. After the run reached terminal failure, only failed jobs were rerun. Attempt 2 is 61/61 jobs success. Source authority artifact 12310445072 digest sha256:8120984f56ec55171d3eb721056007a14e8bc5c7829dcb4b0581528fa3e863ae binds target b8ceacbe9390d182ab61019ba9c75601e5f02426, upstream 0eca52223cceed229932add1cc80792763120844, tree 1fc479a4b448aed851181bf4a4796d3015368348, root_entries=7116, root_blobs=6783, direct_gitlinks=36 and recursive_non_directory_entries=16305. Boost artifact 12308702811 digest sha256:ee91af54aaa4bfff39c873f7144822d11c7a7ed604e1b8d28c60a5b3a683b696 proves acquisition_candidates=582, dispositioned=582, open=0 and keeps source_closure_ready=false. Root aggregate artifact 12309650974 digest sha256:cfd6116112f3887cac148e8cd36ca681ef3976f65bd7d5c218994519886d8a68 covers shards 0-11; child aggregate 12310870187 digest sha256:48e65885486c5193d57402067cfff33578d4a0f636eee1af9f0d682f68912d5b covers 0-23; next aggregate 12310646025 digest sha256:6bf36768de1cdfe8db0ce1c6ec17cc6c8c168c0486f5613e8366054e44f9e71e covers 0-15. Fixed-point artifact 12310242033 digest sha256:f3f984fb833cbfe8c9e1f547a2b2d6397193ef699bf89ce055af0d87017a2642 reaches final_scanned_authorities=203 and unresolved_gitlink_authorities=0. Reachability artifact 12309837264 digest sha256:418220709fc17168ae61061d97c9a2ab78a30a6322b93c56975ec2b09c8dd2d9 remains partial-audited-open with remaining_open_totals authorities=95, gitlinks=133, acquisition_candidates=3865, generated_inputs=4674, resource_inputs=4602, tool_inputs=4255; baseline accepted=false and source_closure_ready=false.

Rust desktop runtime run 39518408392 is terminal success on b8ceacbe9390d182ab61019ba9c75601e5f02426: Windows/macOS/Ubuntu pressure profiler, renderer, platform-worker and rust-host are 6/6 success. rust-host steps 1-17, both post steps and Complete job succeeded. Artifact 12310365286 digest sha256:283fb5a29a37310483c6ceb0139c7d4d3d96778ed76673fab5ebe87379ab764f contains messaging-responsibilities.json with head_sha=b8ceacbe9390d182ab61019ba9c75601e5f02426 and result=passed.

## Complete source read

The skill is the repository-development performer for exactly one known ai-tdesktop task. It is not a product Task implementation.

- Read the complete engine: the shared pipeline, phase prompts, test loop, build-lock recovery, optional Computer Use adapter, provider/delegation adapters and code-health mechanical lane remain authoritative; host syntax is an adapter.
- Resolve the workspace and task: accept one short slug or full id, resolve it without guessing ambiguity, use the durable workspace/task state and exact source lineage, and do not process inbox, unrelated queue work, discovered follow-ups or pending-task consolidation.
- Acquire exactly this task: start/retry/resume the selected task only when readiness/ownership/source-lineage gates allow it; blocked retry is explicit and stale/conflicting ownership is not converted to progress.
- Run and publish: use the Q0-Q2 mechanical lane where applicable, otherwise the full context/plan/assessment/implementation/review/evidence pipeline. General review plus lifetime/reuse/structure/performance/security lenses are independent; NOT_APPLICABLE requires scope proof. Accepted fixes require focused rereview. Evidence classifies TEST_FLAW versus IMPL_BUG and preserves still-valid evidence rather than blindly rerunning everything.
- If the task is intrinsically broader than one independently shippable unit, publish split-required with retained state and stop. Do not create replacement tasks here; the continue scheduler owns that transaction.
- Crash/debugger fallback, locked-macOS overlay execution, Computer Use policy and Windows build-lock recovery are bounded evidence mechanics. Unreachable/unsafe required subjects remain unverified/block/hard-stop; they never become a synthetic pass.
- Success requires canonical final publication, not just a source commit. Ordinary publication races may retry; semantic conflict, unsafe checkout or unreachable remote preserves resumable state and reports a hard stop.
- The final result is task-id based and reports status/attempts/touched files/evidence or unverified behavior; commit hashes are not the cross-repository task identity.

## Real callers and references

Exact upstream callers/references were inspected at 0eca52223cceed229932add1cc80792763120844:

- `.grok/commands/perform-task.md@313e94a3202c39aa10944b832b45ee80a762f509` and `.claude/commands/perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e` both require this skill completely, constrain execution to the named task and publish split-required without creating replacements.
- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5` is the scheduler caller: it delegates one-task execution and states that perform-task owns context, planning, implementation, review, Debug build, test-loop, evidence and final publication.
- upstream `AGENTS.md@b138b4fbfae7a8d1e92d28c154e09eaca2418e51` defines the durable ai-tdesktop task/status vocabulary and routes one known task through perform-task.
- `.grok/ai-workflow-adapter.md@db447f204316d3991f6d20db3d8f097fcc021154` and `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` preserve the shared skill as semantic authority while adapting delegation/text handling.
- `.agents/skills/perform-task/references/pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7`, `phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba`, `mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` and `computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259` supply the full phase/evidence/mechanical/UI-driving details.
- `.agents/shared/test-loop.md@83c4732bf6139f7c6557f18dd87bb35c3d68808d`, `build-lock-recovery.md@ec75573e7efe74e9c9a344f3e66f2fc43ac7680f`, `codex-delegation.md@ed89d2a14344f50034a9363bd9ffe75f7972e652` and `codex-cli-leaves.md@659da44a8083c433c4e9b59ca28f64ba525bfbf8` define source-neutral evidence execution and lifecycle recovery.
- `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` and `references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e` route Q0-Q2 through the mechanical lane and semantic classes through the full pipeline.
- `.agents/skills/process-inbox/SKILL.md@0ce70794ac743d99e6d5c80894cde970b52842db` owns inbox/routing rather than task implementation.

## Existing-owner-first resolution

Fabushi exact-head already owns the applicable source-neutral responsibility:

- `AGENTS.md@57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b` routes quality findings through the ordinary task/review/evidence lifecycle, defines repository quality-task classification, requires proof-based independent review/focused rereview, establishes source-neutral worker effort and makes FBCP/TDRP executable verification GitHub-Actions-only.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@0d3a4c993784625f9b70279602c655d4788fe9fb` owns repository quality-task qualification, AI review phase execution policy and the adaptive evidence loop/bounded recovery contract.
- `projects/fabushi-communication-platform/quality/evidence-contract.md@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70` owns independent review scope, durable review arbitration, exact-head evidence and compact task-local handoff evidence.

Shipping candidates were inspected but rejected as semantic owners: `source/host/src/agents/*`, `source/host/src/workflows/*` and `source/packages/agent/tools/task*.ts` implement Fabushi product Agent/workflow/task behavior. They do not own repository-development task acquisition, source-lineage checks, code review/evidence campaigns or Git publication. Mapping perform-task into those shipping owners would incorrectly merge repository governance with product runtime.

No new product owner, second Task store, second scheduler, Telegram-specific AI runtime or copied provider adapter is required; novel_capability=false.

## Fabushi-equivalent contract

`one explicit existing repository task -> exact durable identity/source-lineage/readiness -> single owned task runner -> mechanical or full pipeline -> independent review and focused rereview -> adaptive evidence with fail-closed recovery -> canonical task publication -> Approve / Block / split-required / already-satisfied / visible hard stop`

Invariants:
- one performer never drains unrelated queue work;
- stale/ambiguous ownership or source lineage cannot silently advance;
- no review or test-evidence failure is converted to a pass;
- provider/Computer Use/debug/build-lock mechanisms remain adapters, not semantic owners;
- split-required preserves retained state and stops before replacement creation;
- shipping Fabushi Task/Agent/Workflow owners remain separate from repository-development task truth;
- canonical final publication is required for success;
- exact-head GitHub Actions evidence is required again after this mapping commit.

## Traceability

- Requirement: TDRP-R9-PERFORM-TASK-WORKFLOW-001
- Oracles: ORA-TDRP-PERFORM-TASK-SINGLE-OWNERSHIP-001, ORA-TDRP-PERFORM-TASK-REVIEW-EVIDENCE-001, ORA-TDRP-PERFORM-TASK-PUBLICATION-001
- Invariants: INV-TDRP-PERFORM-TASK-ONE-TASK-ONLY-001, INV-TDRP-PERFORM-TASK-FAIL-CLOSED-EVIDENCE-001, INV-TDRP-PERFORM-TASK-RESUMABLE-STATE-001, INV-TDRP-PERFORM-TASK-NO-SHIPPING-TASK-COLLISION-001
- Security: SEC-TDRP-PERFORM-TASK-EVIDENCE-DOWNGRADE-001
- Fault recovery: FAULT-TDRP-PERFORM-TASK-SOURCE-DRIFT-001, FAULT-TDRP-PERFORM-TASK-TEST-FLAW-001, FAULT-TDRP-PERFORM-TASK-PUBLICATION-CONFLICT-001

## Verification disposition

This closes only recursive strict index 45. Counters become source_entries_total=16,305, unknown=16,257, unread=16,232, omitted=0, responsibilities_total=64, unmapped=0. baseline accepted=false and source_closure_ready=false because source coverage is still open and recursive remaining-open totals are nonzero.

Parent exact-head evidence is successful supporting evidence only after this write. Exact-head acceptance of index 45 requires fresh TDRP Revision 9 and Rust desktop runtime workflows on the new immutable commit. Next strict entry is index 46 `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`.
