# FDN-001 Desktop Host network enrollment owner

Status: implementation in progress.
Parent: `docs/specs/fabushi-same-account-device-network.md`.
Owner: shipping Desktop Host.

## Goal

After Fabushi login, the Host uses the existing first-party Fabushi account credential and Remote Computer device credential to reconcile one account-scoped network enrollment, start the embedded tsnet worker, and complete the Core network binding without exposing product, device, grant, or network secrets to renderer, Cursor/Sand renewal, tool descriptions, or logs.

## Required behavior

- Persist a stable request identity before the first Core enrollment side effect.
- Reconcile the existing Core grant and Backend enrollment result rather than mint unrelated generations during normal retry.
- Bind worker state to the exact Fabushi account + device identity and stop the previous worker on account switch/logout.
- Pass the Headscale AuthKey only to the supervised network worker and clear the one-time value after uncertain or completed start.
- A worker-start transport/process failure after the one-time key may have been consumed is `outcome_unknown`; the Host MUST NOT replay that AuthKey.
- While that enrollment remains unexpired, recovery may only try the persisted tsnet state without the one-time key.
- After the enrollment expiry is definitely in the past, the old one-time material is no longer replayable; the Host may create a new durable requestId and reconcile a fresh enrollment generation.
- Completion transport loss is `outcome_unknown` and must be reconciled by the Backend/Core binding state before declaring completion.
- Network enrollment unavailability must not turn a valid Fabushi account login into logout or crash the Host.

## Verification

Executable verification is GitHub Actions-only. The FDN tsnet network worker and Rust desktop runtime workflows must pass on the exact PR head. Contract tests must lock account-switch fencing, private-state permissions, no secret logging, stable request identity, no blind AuthKey replay, expiry-gated recovery, and completion uncertainty.

## Acceptance for this slice

This slice can be integrated when the exact head compiles in the Rust Host workflow and the FDN worker workflow passes on macOS, Windows, Linux and the Host supervisor. It does not claim G05 direct/DERP/WSS real-network acceptance or the final official MCP-to-native-executor E2E.
