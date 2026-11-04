# Complete read — fixed-U perform-task Computer Use testing adapter

## Authority snapshot

- Desktop checkpoint parent: `ba5bcf2beafaaa8264ce46bda31eda36512f9af1`.
- Desktop main reread: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries.
- Source: `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259`, tree index 71.
- Exact blob read completely: 14,178 bytes / 14,166 decoded characters / 231 newline-delimited lines.

## Source responsibility

This file is a Computer Use/UI-driver adapter over the perform-task TEST phase and shared test loop. It does not own task state, source implementation, review, queue routing, publication, product Computer, or platform runtime.

Portable contracts:
1. driver policy is `auto | overlay-only | required`; deterministic in-process/overlay is default;
2. hybrid is selected only when physical interaction/native window behavior is itself the tested claim;
3. every hybrid step predeclares one bounded action, readiness marker, target, fallback/blocker and local safety envelope;
4. one stateful performer is the sole desktop owner;
5. capability requires the current surfaced driver path, callable supported runtime, already-satisfied permissions and exact isolated test-app identity;
6. focus theft, local input interference or user-stopped sessions contaminate only that driver run;
7. exact app/process/start identity is revalidated around every state/action call;
8. fresh semantic accessibility targets are preferred; ephemeral indices/coordinates come only from current state;
9. deterministic fixture/setup, application assertions, watchdog/terminal markers and tight captures remain authoritative during hybrid operation;
10. unattended credentials/secrets, permission/security approval, account/network/OS changes, server-visible content mutation solely for evidence, external confirmation flows and payments are forbidden;
11. unavailable hybrid uses an equivalent deterministic fallback when valid, otherwise records the exact missing interaction without weakening the oracle;
12. lock screens are never driven; safe complete in-process fallback is used when available;
13. durable driver metadata, accessibility state and tight captures are persisted, with app assertions/semantic observations ahead of narration;
14. causal classification distinguishes implementation defects from evidence/test flaws and precise unavailable capability.

## Direct callers and consumers

Fixed-U code search and exact-ref reads identify two direct references:
- `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` requires this reference when UI-driver selection or operation is relevant.
- `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493` defines `COMPUTER_USE_POLICY`, capability gating, overlay/hybrid interaction, final `UI-Driver` disposition and blocker/recovery mapping.

The shared `.agents/shared/test-loop.md@bb51685e417759939cadc1e2e74b1c0199fcb8f0` remains authoritative for scenarios, oracles, watchdog/crash handling and evidence assessment.

## Existing-owner resolution

Existing-owner-first extends `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001` under Fabushi repository development process authority plus repository quality/UI-functional/visual/a11y evidence governance.

Rejected owners: shipping Product Computer/Tasks/Automations; Agent/Coordinator/Host/Runner runtime; platform-core runtime; Telegram-specific test runtime; provider-specific Computer Use service; second evidence/acceptance store.

No new responsibility or ADR is required. Telegram portable-account, `-testagent`, Qt overlay/helper paths and Telegram app naming remain upstream-specific instrumentation provenance, not Fabushi product architecture.

## Fabushi contract delta

Authored, not run:
- `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-SELECTION-001`
- `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-SAFETY-001`
- `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-EVIDENCE-001`
- `FAULT-TDRP-PERFORM-TASK-UI-DRIVER-RECOVERY-001`

FMEP-001 Stage A forbids executing these contracts now. No Computer Use, desktop automation, app launch, build, test, visual/a11y execution, package, deployment, runtime evidence campaign or acceptance was run.

## Source-closure disposition

Exactly one additional independent fixed-U source entry is closed by attaching it to the existing performer responsibility; responsibility count does not increase. The next independent fixed-U blob in tree order is `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef`.
