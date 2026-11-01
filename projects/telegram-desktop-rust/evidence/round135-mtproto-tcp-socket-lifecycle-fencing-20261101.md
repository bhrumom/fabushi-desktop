# Round 135 — MTProto TCP socket lifecycle fencing

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 / round 104  
Stage: A — implementation authored, executable validation deferred

## Authority and source evidence

- fixed-U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `Telegram/SourceFiles/mtproto/details/mtproto_tcp_socket.cpp@b3130fd706b7cbfa33f7c9f54ac5e87abe9c3e10`
- `Telegram/SourceFiles/mtproto/details/mtproto_abstract_socket.cpp@653033365b543e554104c99a963ee194ddb830d9`
- `Telegram/SourceFiles/mtproto/connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a`
- `Telegram/SourceFiles/mtproto/connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`

`TcpSocket` moves the QTcpSocket to the connection thread, applies `setProxy(proxy)` before connect, forwards connected/disconnected/readyRead/error through queued owner callbacks, and logs the detailed QAbstractSocket error before emitting the generic socket error stream. `TcpConnection` binds those streams to its owned lifetimes, destroys the connect-only lifetime after the fake-PQ readiness transition succeeds, and destroys both lifetimes plus the socket when the connection is finished. `SessionPrivate` removes/disconnects callbacks by exact connection identity, adopts one winning connection, discards the rest, and restarts only from its current owned connection set.

This is an ownership/fencing contract, not a reason to create a Qt-like callback runtime in Fabushi.

## Existing-owner comparison

Canonical `telegram-network` already owns `NetworkAttemptControl`, live `TcpTransport` sockets, selected-proxy carrier establishment, read/write deadlines and transport errors. Existing RuntimeClient `SessionOwner` owns request ids, request cancellation, session adoption/removal and projection-change invalidation. ProductSettingsOwner remains the only durable proxy settings owner.

Before this round, a controlled connect/auth-handshake registered the live socket on its `NetworkAttemptControl`, and successful controlled establish/resume returned an `EstablishedSession` whose transport still retained that control. Because the control owns an absolute deadline, connect-stage ownership could leak into later adopted-session I/O. The existing request path already had the desired `invoke_raw_controlled -> attach -> invoke -> clear` behavior, so no new owner was needed.

Manual review also found `TcpTransport::flush_bounded` recursively called itself and `send_payload` bypassed it with a raw stream flush. Both were inside the same socket lifecycle boundary.

## Spec-first implementation

Platform spec commit: `550c2e170f172ab7c96ba9d11c46234121b21c11`  
Spec blob: `a463412a99805ebf38e035db3dde4f29964be206`

Platform production commit: `dc16f388756d681ef4ee6683b98525edef869e1f`  
Canonical network blob: `63f9c4a26385a776fbdd42af3d0de44a37857691`

Desktop production mirror commit: `e4fe918b8e8ba5589d1ab3d2cdae888087933719`  
Desktop network blob: `63f9c4a26385a776fbdd42af3d0de44a37857691`  
Desktop spec blob: `a463412a99805ebf38e035db3dde4f29964be206`

Implemented in the existing owner:

1. Successful `resume_auth_key_controlled_with_proxy` clears the connect-attempt control before returning the adopted session.
2. Successful `establish_auth_key_controlled_with_proxy` clears the connect-attempt control after the auth handshake and before returning the adopted session.
3. Clearing ownership releases `NetworkAttemptControl.active_stream` and removes the control from `TcpTransport` without cancelling the control or shutting the adopted transport.
4. Later `EstablishedSession::invoke_raw_controlled` retains the existing request-local attach/clear lifecycle.
5. `flush_bounded` delegates to the underlying transport and `send_payload` now uses `flush_bounded`, so flush is covered by the active write/cancellation bound.
6. RuntimeClient projection-change fencing remains unchanged at runtime blob `3a3da0eb4bf004569a3989e05e55c8b86a199c41`.

No second runtime/provider/settings/session/socket owner was added.

## Authored contracts — not executed

- `network_attempt_control_contract_tests::successful_connect_handoff_releases_attempt_stream_without_cancelling_control`
- `network_attempt_control_contract_tests::request_control_can_be_reowned_after_connect_handoff_and_cleared_again`

These contracts are authored source only. They were not built or run.

## Explicit open adjacency

The same fixed-U `TcpSocket` constructor applies `QAbstractSocket::SendBufferSizeSocketOption=kFilesSendBufferSize` and `ReceiveBufferSizeSocketOption=kFilesReceiveBufferSize` when `protocolForFiles=true`. This file-transfer socket buffer tuning is not part of the lifecycle handoff implementation and remains open for the next owner/source review.

## Accounting

Unique responsibility: `TDRP-R9-MTPROTO-TCP-SOCKET-LIFECYCLE-FENCING-001`  
State: `implemented`  
Validation: `not-run`  
Accepted: `false`

Coverage after this bookkeeping:
- `source_entries_total=16202`
- `unknown=16172`
- `unread=16145`
- `omitted=0`
- `responsibilities_total=123`
- `unmapped_responsibilities=0`

No source-entry decrement is claimed because the source blobs are part of an already-read connection family; this round isolates and implements another exact responsibility.

## Stage-A execution statement

No GitHub Actions/check/status/workflow query or trigger was performed. No build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deploy, install or runtime acceptance was executed.


## Final source-level compliance

Platform-core source-level compliance commit: `1138e34377d441a97d72b23279dacdb0bc310d2e`  
Final durable spec blob: `449cfa61c2b3b20c5fd42838a627a8e5330f8bb4`

The final spec review records R1-R7 and AC-1..AC-7 as source-level passed against canonical production `dc16f388756d681ef4ee6683b98525edef869e1f` and desktop bookkeeping checkpoint `734e37f377e7fe58b7a646a6b64b29a92bf9629e`. This does not advance executable validation: `validation_status=not-run`, `accepted=false`.
