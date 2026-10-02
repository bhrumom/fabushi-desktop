# FBCP-ADR-003: Minimal call-session and signaling owner

Status: proposed for P0 owner resolution; no production implementation
Date: 2026-10-02

## Context

Frozen Telegram calls show a durable/observable session state machine and signaling lifecycle that cannot be truthfully owned by the current Computer handoff, Transcript or Electron device bridge. Fabushi needs its own signaling protocol/service if calls are implemented; Telegram phone/MTProto/tgcalls semantics are research only.

## Rejected existing owners

- Computer: owns computer-use/handoff and may integrate screen share, but not peer call session truth.
- Transcript: records typed call events/history, not live negotiation/media state.
- Shared Room/member: owns participants/roles/context, not signaling lifecycle.
- Conversation workspace: presentation only.
- Electron platform: owns OS media-device bridge, not cross-device call truth.
- Coordinator/Runner: Agent execution boundary, not human realtime signaling.

## Decision

Add a **minimal CallSession/Signaling infrastructure owner** underneath existing Conversation/Shared Room/Computer product owners.

Owned state: call ID, conversation/room ID, participant/device identities, call state, offer/answer/signaling sequence, media capabilities, encryption/session references, reconnect generation, active media-device selections and terminal reason. It does not own chat history, participants, Computer session truth or UI navigation.

Commands: create/invite, accept, decline, send signaling, update media capability/device, reconnect/resume, hangup.
Events: invited/ringing, negotiating, connected, media-state-changed, reconnecting, participant-state-changed, ended/failed.

Lifecycle: account/device scoped; explicit timeouts; sequence/generation fenced; reconnect/resume after network transition; sleep/wake and device-change handling; terminal state persists enough for history/audit while ephemeral media secrets are destroyed.

Persistence/security: durable call/session metadata and terminal reason; signaling dedupe/order window; no long-term storage of unnecessary media secrets; device/session revocation immediately fences signaling.

Dependency direction: Conversation/Shared Room requests calls and projects state; Transcript receives typed `CallStateChanged`; Computer may provide screen capture/control integration; platform adapters supply devices; native network service transports signaling.

Boundary/language: Rust preferred Host/native service for state machine/signaling/security; narrow OS/WebRTC adapters may use platform/ecosystem language.

Migration/cutover: no Telegram/tgcalls runtime. First implementation starts behind existing conversation surfaces with no second calls sidebar/workspace.

Focused tests: invite/accept/hangup races, duplicate/out-of-order signaling, reconnect generation, sleep/wake, microphone/camera/screen permission denial, device hot-swap, revocation, crash/restart, group participant changes, network handoff, bandwidth/backpressure, Windows/macOS/Linux packaged media acceptance.
