# Node RPC remote presence contract

Status: implementation slice, not full OCRM acceptance  
Project: OCRM-001  
Date: 2026-10-29 (Asia/Shanghai)

## Context

`UnifiedGatewayApi::call_openclaw_node_list` and `call_openclaw_node_status` can observe an approved device through `DevicePairingOwner`, but the repository has no live node-connection registry for remote node-host transport. The current handlers report approved remote devices as `connected` and `active` while `node.invoke` explicitly reports that the canonical remote node transport is unavailable. That is a false presence claim.

## Goal

Make remote node presence fail closed: pairing approval proves `paired`, while `connected` and `active` are true only when an existing live transport owner supplies that fact.

## Non-goals

- Do not create a second node registry, transport, heartbeat, or connection state owner.
- Do not implement remote `system.run` or reuse the Computer-only private transport as generic node execution.
- Do not change the local `sand-local-node` presence contract.

## Requirements

- R1: approved remote devices remain visible as paired inventory entries.
- R2: without a live transport-owner fact, approved remote entries report `connected=false` and `active=false`.
- R3: `node.status` uses the same fail-closed remote presence semantics as `node.list`.
- R4: local node presence remains approved, paired, connected, and active.
- R5: no new durable state or transport owner is introduced.

## Ownership

`DevicePairingOwner` owns pairing approval and durable device identity. A future live node transport owner must own connection/activity truth. The gateway handlers may project those facts but must not infer them from pairing approval.

## Verification and acceptance

- V1: source contract proves remote list/status no longer synthesize connected/active from approval alone.
- V2: focused contract coverage proves local positive presence and approved-remote paired-but-disconnected projection.
- V3: executable checks run only in GitHub Actions against the exact changed head.
- This slice does not establish remote node transport, device acceptance, or OCRM-M13 completion.

## References

- `projects/openclaw-rust/dossiers/node-rpc-owner-integration-audit-20261029.md`
- `source/host/app/src/main.rs` (`call_openclaw_node_list`, `call_openclaw_node_status`, `call_openclaw_node_invoke`)
- `source/host/tests/openclaw_chat_cron_node_contract.rs`
