# Round 25 — exact-head entrypoint and CI-access audit (2026-10-31)

This receipt is **read-only source/API observation**, not production acceptance. Source: GitHub authenticated connector, PR #178 head `801956e06194424b5b9bcda2ef54453c43fd68ca`; main `cfef45568293a22deb32a6cdef17916f3ec19648`. PR #178 is open, draft, unmerged. GitHub repository metadata reports `admin=true`, but the connector does not expose privileged Actions-management or hook delivery APIs.

## Exact-head CI evidence

At inspected PR head, `fetch_commit_workflow_runs` returned zero PR workflow runs; REST `actions/runs?head_sha=...&per_page=100` returned `total_count=0`; REST `commits/.../check-runs?per_page=100` returned `total_count=0`; commit combined statuses returned empty `statuses`. These are **absence of evidence**, not PASS. Attempts to read `actions/permissions`, `actions/workflows`, and `hooks` through the approved GitHub fetch interface returned `INVALID_ARGUMENT` (endpoint disallowed). Therefore workflow enabled-state, Actions policy, and PR synchronize webhook delivery remain unknown, not false.

## Source identities / shipping-edge observations

- `frontend/src/production/bootstrap.ts`: **not present** (404); actual `frontend/src/production/bootstrap.tsx`, blob `63e272d027a0f1f6fb5ea308a2252f121bb46b19`, exports `acquireProductionRendererRuntime` (requires desktop bridge/coordinator claim), `requireProductionRendererMount`, `mountProductionRenderer` (StrictMode).
- `frontend/src/main.tsx`, blob `ccb3e932cb9b017742c7d6a6ecb77c2faf5814d7`: imports bootstrap functions and `ProductionRenderer`, then mounts `RootShellErrorBoundary` wrapping `ProductionRenderer`. The reported `sourceComposed: true` health field is a source-side declaration, **not runtime proof**.
- `source/electron-main/entry.ts`, blob `3c877893f5f6478251587a1130e08328c9cf15f7`: constructs IPC/coordinator adapter bindings, native bindings and services; calls `startElectronMainProduction` with service factories. The `FABUSHI_E2E` special-case external URL interception is expressly a test-mode path, not production acceptance.
- `source/electron-main/main.ts`, blob `b8eeed322ec07e5c8f67fa367cd2f58ac212b4f0`: inspected initial contract/interface section only, not full startup lifecycle.
- `source/electron-main/coordinator/production-root-provider.ts`, blob `24e924f3d52d41a0900314aa7c21d98804a9dc19`: inspected port/account authorization and transition setup; this is static wiring, not an executed Coordinator test.
- `source/node-agent-coordinator/src/main.rs`, blob `c822c07dff3027a22a658d1ece6aefe919691365`: actual Coordinator binary entrypoint candidate; initial imports include gateway, renderer port, host supervisor and runner relay.
- `source/host/app/src/main.rs`, blob `8e19361b3b278d085927a84ea4c8ce8c3fe3de80`: actual Host entrypoint, explicitly documented as shipping Host; initial source imports inspected.
- `source/mahayana/mahayana-rs/mahayana-app-host-desktop/src/main.rs`, blob `c78409876f0a8104d26bbc924128ef2842d72daf`: separate compatibility/backend binary, not independently proven shipping.
- Guessed paths `source/{coordinator,runner,host}/src/main.rs` are absent (404). The exact Runner executable entrypoint, its spawn and complete Supervisor lifecycle, and corresponding test/packaging traces are **not yet verified**.
- Recursive Fabushi PR tree lookup returned `truncated=false`, 9525 entries, for tree at commit `801956e06194424b5b9bcda2ef54453c43fd68ca`. This is the **Fabushi** tree only and says nothing about recursive upstream tdesktop/cppgir completeness.

Root `AGENTS.md`, FBCP-001 Revision 7, TDRP-001 Revision 9 and project Sources of Truth were inspected at this head. Full acceptance authority and test execution have **not** been exhausted. GitLab project 7705067 raw reads for `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` were **not obtained**; no recursive cppgir source/blob closure is claimed.

## Gate

Keep `accepted=false`, `baseline_ready=false`, `status=next`; no upstream lock, inventory, ledger, dossier, production code, or workflows modified. Absent exact-head CI run/check evidence, validated upstream closure, complete production coverage, packaged temporal acceptance and independent release `ACCEPT`, no merge/release. Next: trace actual Runner/Host/Coordinator launch + tests to packaged output; obtain privileged Actions/webhook evidence and authorized GitLab raw API access; then reassess at freshly read live HEAD.
