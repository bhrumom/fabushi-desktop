# Complete read — .agents/skills/code-health/agents/openai.yaml

Status: independently understood and mapped; exact-head verification pending  
Project: TDRP-001 Revision 9  
Upstream: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`  
Source: `.agents/skills/code-health/agents/openai.yaml@59bb6304445f21e2a3ee1a97a69a8e231f934d30`

## Inventory identity

The prior merged entry18 acceptance run `39339904225` produced source-authority artifact `12228278849` (digest `sha256:2bf30b877cd6da4647937870bc0fd43aaa5e4ea18ae118d3f8179c4c1ef6d7d8`) and recursive-closure artifact `12229280379` (digest `sha256:693102b5e771b0a2bc1d501042e9ab580980ce7b05124535e4d011f6e36e0cbf`). That fresh authority identifies actual 1-based recursive array entry 19 as `.agents/skills/code-health/agents/openai.yaml`, blob `59bb6304445f21e2a3ee1a97a69a8e231f934d30`. Current main before this write is `be031eae9f178ab080b076cd4bd48ceeabab7de7`; upstream `dev` still resolves to `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`; merged parity ledger has no row for this exact path/blob.

## Complete source read

The complete 266-byte YAML is:

- `interface.display_name = "Code Health"`;
- `interface.short_description = "Measure code health and propose quality tasks"`;
- `interface.default_prompt = "Use $code-health to scan freshly fetched origin/dev, update the findings ledger, and report what got worse before this run's proposals."`.

This file contains no scanner, persistence, scheduling, task routing, product runtime, UI, network, or platform implementation. Its independent responsibility is the host-native registration surface that makes the existing code-health skill discoverable and chooses a default invocation contract.

## Relationship to canonical code-health behavior

The complete `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` establishes the actual semantics: durable scans require freshly fetched exact `origin/dev`; findings use stable identities/history; the scheduled run is `$code-health scan route` / `/code-health scan route`; reports put "Got worse or needs a decision" before trends/proposals; routing obeys task caps, owner approval, active-path conflict and audit trust; scans never edit product code.

`.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a` is a provider command adapter. It reads the same shared skill and selects `scan`, `survey`, `check`, and optional `route`; the scheduled mode is `scan route`.

`.agents/skills/process-inbox/scripts/workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c` owns health-state ensure/publish mechanics, while `.agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba` and the mechanical/full pipeline references consume the canonical quality ratchet in ordinary task verification. Therefore the YAML is discovery/invocation metadata over those owners, not an independent workflow implementation.

## Existing-owner-first resolution

Current Fabushi already owns the applicable responsibility through:

1. root `AGENTS.md` repository development and quality-health governance;
2. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` qualification of exact-authority scans, proportional proof, routing controls, and now native invocation;
3. `projects/fabushi-communication-platform/quality/evidence-contract.md` durable quality-health provenance and native registration evidence;
4. the ordinary task/review/evidence lifecycle that receives routed work.

The native registration/default prompt is therefore mapped to the existing source-neutral repository skill/task invocation owner. The provider syntax `$code-health`, OpenAI agent manifest format, and Telegram-specific path are adapter provenance only.

Rejected alternatives:

- shipping Agent/Coordinator/Host/Runner are product execution owners, not repository skill registration;
- a copied OpenAI/Telegram adapter runtime would duplicate host mechanics;
- a second scanner, findings ledger, task queue, or routing owner would violate existing-owner-first and duplicate the already-mapped code-health responsibility.

No novel shipping capability or ADR is required.

## Fabushi-equivalent contract

`host native discovery -> canonical repository skill invocation -> fresh exact authority -> canonical findings update -> regressions/decisions reported before proposals -> ordinary trust/conflict/approval routing`

Fail-closed branches:

- registration/default prompt cannot allow mutable or stale observations to advance canonical findings;
- provider syntax cannot become semantic ownership;
- proposals cannot hide or reorder the regressions/decisions that must be surfaced first;
- scheduled/ad-hoc native invocation cannot bypass route caps, active ownership conflicts, owner approval or audit suspension;
- registration cannot introduce a parallel scanner/state/task runtime.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-NATIVE-REGISTRATION-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-NATIVE-OWNER-001`, `ORA-TDRP-CODE-HEALTH-DEFAULT-PROMPT-001`
- Invariants: `INV-TDRP-CODE-HEALTH-NATIVE-NO-SECOND-OWNER-001`, `INV-TDRP-CODE-HEALTH-NATIVE-FRESH-AUTHORITY-001`, `INV-TDRP-CODE-HEALTH-REPORT-BEFORE-PROPOSALS-001`

## Verification disposition

This closure changes repository governance/evidence contracts only. It adds no shipping runtime. Exact-head acceptance is the same pair of GitHub Actions required for prior source-entry closures: TDRP Revision 9 source authority plus Rust desktop runtime on one final PR HEAD. Source-authority must remain fail-closed while full baseline closure is open, reproduce 12 root / 24 child / 16 next shards, reach fixed point with zero unresolved gitlink authorities, and preserve root archive / Boost reachability evidence. Rust runtime must preserve renderer/platform-worker/rust-host and native Mahayana messaging composition.

After this one independent source entry, counters become recursive=16,202, unknown=16,183, unread=16,156, omitted=0. Full source closure remains open.
