# Sand completion acknowledgement safety

Status: active; implementation and service acceptance pending
Owner: existing Host Automations runtime
Date: 2026-10-24
Related: PR #169 / OCRM service-contract inquiry; this is a corrective existing-owner change, not OpenClaw migration acceptance.

## Context, goal and verified baseline
At main 7394f6dc, AutomationFireBackendRuntime promotes completion HTTP 404/409 errors to Reported, then includes their IDs in poll ackRunUuids. Without a versioned service contract these errors cannot establish successful completion. The user requested resolution and an email reply. Keep unresolved completion outcomes pending rather than acknowledging them.

## Requirements and target
- R1: all completion transport errors, including 401/404/409, retain Completed and propagate on direct complete; never produce an ACK solely from an error.
- R2: retry failures retain Completed. A later successful report alone advances to Reported; the next poll may ACK it. Poll redelivery while pending must not dispatch the task again.
- R3: retain existing successful completion and poll-delay behavior.
- R4: distinguish this local safety correction from unresolved deployed service ownership, response semantics and crash/restart durability.

## Scope, architecture and contracts
Extend existing Host AutomationFireBackendRuntime only. No second scheduler, service, persistent writer, schema, pairing mutation or inferred server contract. Existing transport/auth and successful response parsing stay unchanged. Completed -> Reported is allowed only on Ok from report_completion. Existing retry cadence remains. Raw status errors are available through AutomationsBackendError; this change adds no sensitive telemetry.

## Failure modes / compatibility
404 may indicate missing or wrong-account resources; 409 may be a conflicting UUID. Neither is success evidence. 401 and lost-response transport errors remain pending. The same run ID is retained to avoid duplicate execution within this runtime. Pending state is still in memory: restart recovery, account binding, unbounded retention and actual server idempotency remain unresolved service/lifecycle work. 204 currently fails JSON decoding; do not claim server support without its contract. Conservative retention can keep work pending when a service historically expects terminal 404/409; this is preferable to silently acknowledging an unknown outcome and must be surfaced before rollout.

## Plan and verification
First document this bounded correction, remove both error-to-Reported branches, replace the old conflict-as-success test, and cover direct/retried 401/404/409 and lost-response failures, redelivery suppression, recovery and eventual ACK. Run the existing automations_backend_contract suite in GitHub Actions only, on the PR exact head via the existing Rust desktop workflow. Do not add an OCRM validator or claim AUTO-RUN-CRASH-01 passed.

## Acceptance and delivery
AC1: R1/R2 have regression tests checking actual poll request bodies and dispatch counts.
AC2: R3 existing success/delay tests remain passing in exact-head Actions.
AC3: PR and reply link fixed source, evidence and remaining external requirements without claiming production completion.
Release only through normal repository integration after review/CI; no schema migration. Rollback is revert of this change, with the original unsafe ACK risk explicitly understood.

## Sources and remaining service authority
- Desktop PR #169 head b5db35fa272c83833685d70a730b5f8285fa1176.
- Current correction baseline main 7394f6dc.
- Host auth credential_renewer.rs defaults to https://api2.cursor.sh; SAND_BACKEND_URL and CURSOR_API_BASE_URL can override. This identifies the configured default, not a proven deployed owner.
- Backend main 069f69d903e2e358714796fe465a39e7e0468ca3 has no tracked automation-runs/events route matches.
- Required external evidence: deployed endpoint/revision/owner, versioned completion and ACK definitions, account-switch and restart/idempotency receipts. These remain blocked; client mocks cannot supply them.

## Compliance
R1/R2: implementation pending; Actions pending.
R3: Actions pending.
R4: documented; production service acceptance remains blocked.
AC1/AC2: pending. AC3: pending publication and reply.
