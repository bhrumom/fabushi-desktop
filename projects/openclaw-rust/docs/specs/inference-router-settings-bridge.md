# Inference router settings acknowledgement
Contract: `OCRM-ROUTER-SETTINGS-ACK-v1`; status: implementation contract frozen; executable evidence pending.
Implementation base H: `e01f51f33134cbd9dd0b6abd0d471dfb5d71d8de`.
Base tree: `0de92aba674d01580319e21cd2840184f78f9a30`.
Fixed U: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`.
U tree: `90c8aab39972da11cfad220361781d49e1bc6700`.
The diagnosis at `6034da1fcc1ecfe95db4fe5532c61cf88872fb5b` carries forward: the bridge, resync, mirror, Settings and Inference paths are unchanged at this base.
Task: `d56b3d1d-6125-4f6f-a8d8-e7e36728a1ba`, round 1.

## Problem and scope

Existing `main-edge.ts:136-137` saves local provider before Host acknowledgement.
It converts Host failure/null into success and ignores Host provider on reads.
Coordinator serializes writes but reads bypass that chain; reconnect does not replay provider.
Thus the existing served desktop RPC can report a preference that Host never acknowledged.
This batch repairs that bridge; it does not establish a complete provider-picker UI.
Existing Rust Settings owns the committed configuration; Electron owns only the display mirror.

## Required contract

| ID | Requirement |
| --- | --- |
| ACK-01 | `provider` means the acknowledged Host configuration value, not the effective runtime route. |
| ACK-02 | Keep the existing five-provider request validator. Invalid requests fail before Host dispatch or mirror mutation. |
| ACK-03 | Set awaits Host before touching the mirror; require a non-null record with a valid `inferenceProvider`. |
| ACK-04 | A different valid Host provider is canonical: persist and return that value, never the requested substitute. |
| ACK-05 | Get reads Host, validates its provider, then mirrors and returns it. No local-provider or cursor success fallback. |
| ACK-06 | Null, malformed/missing/invalid provider acknowledgement, read failure or write failure fails the existing RPC and preserves the previous mirror before acknowledgement. |
| ACK-07 | Each served MainEdge instance orders only router get/set through one private Promise tail, including Host operation, validation, mirror persistence and reply; rejection releases the tail. |
| ACK-08 | Mirror persistence failure is observable RPC failure. Do not roll back or retry Host; later successful get reconciles the mirror. |
| ACK-09 | Preserve usage precedence, local CLI diagnostic shape, appWindow trust, default-model and Computer Use behavior. |
| ACK-10 | No persistent queue, automatic retry, business scheduler, model catalog, routing policy or new storage authority is added. |

A missing stored Host provider is normalized by Rust to cursor in its public projection.
A missing provider in a wire response is malformed acknowledgement, not that stored-default case.
`FABUSHI_PRODUCT_MODE=1` or `SAND_PACKAGED=1` still forces Fabushi in ProductionInferenceSettings.
The bridge must not copy that override or claim a configuration write changes the active route.
The local Promise tail orders this adapter only; external Host writers remain outside its scope.

## Failure, cancellation and restart

Coordinator transport failure/null can mean the Host committed but its reply was lost.
It is never proof of Host no-write and never authorizes a blind retry or compensation.
Explicit controlled Host refusal and transport-unknown outcome need separate oracles.
Local persistence failure after acknowledgement also leaves Host commit intact.
An abandoned IPC or process exit has no new rollback/cancellation guarantee.
After restart or unknown outcome, explicitly get Host settings and reconcile the mirror.
Reopening local storage alone proves only its last locally committed image.

## Architecture and allowed paths

Production change: `source/electron-main/main-edge.ts`.
Tests: `source/electron-main/main-edge-model.contract.test.ts`.
Use existing preload/generated RPC, Coordinator ports, Rust Settings and inference consumers.
Do not reuse BoxSettingsField.apply unchanged: it writes the mirror before Host acknowledgement.
The integrator owns this Spec, the existing Config task, shared workflow receipts and ledger integration.

## GitHub Actions oracles

Use real `serveMainEdge`, generated dispatcher and file-backed `SandSettingsStore`.
A held Host write must not change bytes/reopened provider; its valid acknowledgement must.
Cover five providers, invalid inputs, canonical reply differing from request and trust refusal.
Cover null/rejected/malformed acknowledgements and failed reads, then a valid recovery operation.
Hold get→set, set→get and two sets; assert dispatch order, reply values and final reopened mirror.
Fail local persistence after Host success; assert one Host write, no rollback, and get repair.
Model a lost reply after Host commit separately; assert failure and explicit read reconciliation.
Retain existing model/Computer Use cases and unchanged Rust provider/packaged-route evidence.
Use the existing Actions renderer bundling/contract target for this changed surface; record exact H, command and outcomes.
Keep still-valid Rust evidence separate and require all applicable scoped authority/coverage before a unit-verified claim.
The existing push-triggered OCRM workflow runs the shipping Electron bundle and
its five Node targets; require all eleven named ACK positives and retain the TAP
completion counts and digest in its receipt. Keep existing native/full/config
job predicates and assertions unchanged. Their receipts remain separately bound;
this desktop receipt cannot assert a Rust job result. Reuse existing caches and
keep the preceding Rust batch's actual outcomes with its own subject.
No local/htch validation or old-H rerun; source inspection is not passing evidence.

## Provenance and acceptance

Read Config atomic-commit Spec completely; model Spec MB-01–04 and open-parameter section partially.
Read Config dossier 1–130 and relevant task/module-map sections; earlier full reads remain attributed.
Directly read U `src/gateway/server-methods/config-write-flow.ts` 1–356, blob `f76ff11ab4cb82694d2ee0df957d882011157010`.
Its 228–290 commit receipt follows publication; this anchors semantics, not new upstream coverage.
Target deep reads cover MainEdge, resync, local store, wiring and cited Rust projection/consumer paths.
Acceptance requires focused independent review and Actions evidence for ACK-01–10 on the frozen tree.
Live service, complete picker UI and overall OCRM acceptance remain required elsewhere and are not claimed by this bounded repair.
Rollback is the ordinary code revert; no stored format or data migration is introduced.
RPC failure is the existing diagnostic surface; no provider credentials or response bodies are added to errors.


