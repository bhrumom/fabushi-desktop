import { useCallback, useEffect, useRef, useState } from "react";
import type { DesktopBridge } from "../recovered/contracts/desktop-bridge";
import type { ProductionCoordinatorClient } from "./coordinator-client";
import type { RendererAgent } from "./model";

type CallState = "invited" | "ringing" | "negotiating" | "connected" | "reconnecting" | "ended" | "failed";
type CallRole = "creator" | "peer";
interface HumanCall {
  readonly id: string;
  readonly scopeId: string;
  readonly creatorId: string;
  readonly state: CallState;
  readonly generation: number;
  readonly signalSeq: number;
  readonly participantIds: readonly string[];
  readonly mediaCapabilities?: Record<string, unknown>;
  readonly deviceSelection?: Record<string, unknown>;
  readonly terminalReason?: string | null;
  readonly updatedAtMs?: number;
}
interface CallLease {
  readonly userId: string;
  readonly deviceId: string;
  readonly role: CallRole;
  readonly claimedDeviceId?: string | null;
  readonly isOwner: boolean;
  readonly claimAvailable: boolean;
  readonly state: CallState;
  readonly generation: number;
  readonly eventSeq: number;
}
interface CallSignal {
  readonly callId: string;
  readonly generation: number;
  readonly seq: number;
  readonly senderDeviceId: string;
  readonly kind: "offer" | "answer" | "candidate";
  readonly payload: Record<string, unknown>;
}
interface IceReply {
  readonly iceServers: RTCIceServer[];
  readonly ttlSeconds: number;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value != null && !Array.isArray(value);
}
function isCall(value: unknown): value is HumanCall {
  return isRecord(value)
    && typeof value.id === "string"
    && typeof value.scopeId === "string"
    && typeof value.creatorId === "string"
    && typeof value.state === "string"
    && typeof value.generation === "number"
    && typeof value.signalSeq === "number"
    && Array.isArray(value.participantIds);
}
function isLease(value: unknown): value is CallLease {
  return isRecord(value)
    && typeof value.userId === "string"
    && typeof value.deviceId === "string"
    && (value.role === "creator" || value.role === "peer")
    && typeof value.isOwner === "boolean"
    && typeof value.claimAvailable === "boolean"
    && typeof value.generation === "number";
}
function isSignal(value: unknown): value is CallSignal {
  return isRecord(value)
    && typeof value.callId === "string"
    && typeof value.generation === "number"
    && typeof value.seq === "number"
    && typeof value.senderDeviceId === "string"
    && (value.kind === "offer" || value.kind === "answer" || value.kind === "candidate")
    && isRecord(value.payload);
}

export function selectActiveHumanCall(value: unknown, scopeId: string): HumanCall | null {
  if (!Array.isArray(value)) return null;
  const calls = value.filter(isCall).filter((call) => call.scopeId === scopeId && call.state !== "ended" && call.state !== "failed");
  return calls.sort((left, right) => (right.updatedAtMs ?? 0) - (left.updatedAtMs ?? 0))[0] ?? null;
}

export function requireLocalCallLease(value: unknown, allowClaim: boolean): CallLease {
  if (!isLease(value)) throw new Error("Host returned a malformed call transport lease.");
  if (!value.isOwner && !(allowClaim && value.claimAvailable)) {
    throw new Error("This call is active on another device.");
  }
  return value;
}

function desktopSourceConstraint(sourceId: string): MediaTrackConstraints {
  return {
    mandatory: {
      chromeMediaSource: "desktop",
      chromeMediaSourceId: sourceId,
      maxFrameRate: 30,
    },
  } as unknown as MediaTrackConstraints;
}

function stopStream(stream: MediaStream | null): void {
  for (const track of stream?.getTracks() ?? []) {
    if (track.readyState !== "ended") track.stop();
  }
}

class CancelledCallMedia extends Error {}

function requireCurrentMedia(isCurrent: () => boolean): void {
  if (!isCurrent()) throw new CancelledCallMedia("Call media operation is no longer current.");
}

async function awaitCurrentMedia<T>(pending: Promise<T>, isCurrent: () => boolean): Promise<T> {
  try {
    const value = await pending;
    requireCurrentMedia(isCurrent);
    return value;
  } catch (failure) {
    // A rejected old request must not reach a newer call/peer's failure handler.
    requireCurrentMedia(isCurrent);
    throw failure;
  }
}

function callMediaConstraint(deviceId: string | null, enabled: boolean): boolean | MediaTrackConstraints {
  if (!enabled) return false;
  return deviceId == null ? true : { deviceId: { exact: deviceId } };
}

async function getPreferredUserMedia(
  bridge: DesktopBridge,
  input: { audio: boolean; video: boolean },
  isCurrent: () => boolean,
): Promise<MediaStream> {
  const preferences=await bridge.callMedia.getPreferences().catch(()=>({ microphoneId:null, cameraId:null }));
  requireCurrentMedia(isCurrent);
  const preferred: MediaStreamConstraints = {
    audio: callMediaConstraint(preferences.microphoneId,input.audio),
    video: callMediaConstraint(preferences.cameraId,input.video),
  };
  let stream: MediaStream;
  try {
    stream = await navigator.mediaDevices.getUserMedia(preferred);
  } catch (error) {
    requireCurrentMedia(isCurrent);
    const stalePreference=(input.audio && preferences.microphoneId!=null) || (input.video && preferences.cameraId!=null);
    const recoverable=error instanceof DOMException && (error.name==="NotFoundError" || error.name==="OverconstrainedError");
    if (!stalePreference || !recoverable) throw error;
    stream = await navigator.mediaDevices.getUserMedia({ audio: input.audio, video: input.video });
  }
  if (!isCurrent()) {
    stopStream(stream);
    throw new CancelledCallMedia("Call media capture completed after cancellation.");
  }
  return stream;
}

function mediaFailure(error: unknown): string {
  if (error instanceof DOMException && error.name === "NotAllowedError") return "Microphone/camera permission was denied.";
  return error instanceof Error ? error.message : String(error);
}

interface MediaLifetime {
  readonly epoch: number;
  readonly client: ProductionCoordinatorClient;
  readonly scopeId: string;
  readonly mediaBridge: DesktopBridge["callMedia"];
}

interface CaptureOperation {
  readonly lifetime: MediaLifetime;
  readonly call: HumanCall;
  stream?: MediaStream;
}

interface DisplayCapture {
  readonly operation: CaptureOperation;
  readonly stream: MediaStream;
  readonly track: MediaStreamTrack;
  readonly peer: RTCPeerConnection;
  readonly sender: RTCRtpSender;
  attachment: Promise<void>;
}

export function HumanCallControls({
  bridge,
  client,
  conversation,
}: {
  readonly bridge: DesktopBridge;
  readonly client: ProductionCoordinatorClient;
  readonly conversation: RendererAgent;
}) {
  const [call, setCall] = useState<HumanCall | null>(null);
  const [lease, setLease] = useState<CallLease | null>(null);
  const [status, setStatus] = useState("idle");
  const [error, setError] = useState<string | null>(null);
  const [muted, setMuted] = useState(false);
  const [cameraEnabled, setCameraEnabled] = useState(false);
  const [screenSharing, setScreenSharing] = useState(false);
  const contextRef = useRef({ client, scopeId: conversation.id, mediaBridge: bridge.callMedia });
  contextRef.current = { client, scopeId: conversation.id, mediaBridge: bridge.callMedia };
  const lifetimeRef = useRef(0);
  const disposedRef = useRef(false);
  const localStreamRef = useRef<MediaStream | null>(null);
  const remoteStreamRef = useRef<MediaStream | null>(null);
  const pcRef = useRef<RTCPeerConnection | null>(null);
  const peerOperationRef = useRef(0);
  const videoSenderRef = useRef<RTCRtpSender | null>(null);
  const localCaptureRef = useRef<CaptureOperation | null>(null);
  const displayOperationRef = useRef<CaptureOperation | null>(null);
  const displayRef = useRef<DisplayCapture | null>(null);
  const stoppedDisplayRef = useRef<CaptureOperation | null>(null);
  const startingRef = useRef<MediaLifetime | null>(null);
  const pollingRef = useRef<object | null>(null);
  const retiringCallsRef = useRef(new Map<string, object>());
  const configurePeerRef = useRef<((lifetime: MediaLifetime, nextCall: HumanCall, nextLease: CallLease, local: MediaStream) => Promise<RTCPeerConnection>) | null>(null);
  const callRef = useRef<HumanCall | null>(null);
  const leaseRef = useRef<CallLease | null>(null);
  const cameraEnabledRef = useRef(false);
  const mutedRef = useRef(false);
  const appliedSignalSeqRef = useRef(0);
  const recoveringRef = useRef<object | null>(null);
  const localVideoRef = useRef<HTMLVideoElement | null>(null);
  const remoteVideoRef = useRef<HTMLVideoElement | null>(null);

  const captureLifetime = useCallback((): MediaLifetime => ({
    epoch: lifetimeRef.current,
    ...contextRef.current,
  }), []);

  const isCurrentLifetime = useCallback((lifetime: MediaLifetime) => {
    const context = contextRef.current;
    return !disposedRef.current
      && lifetime.epoch === lifetimeRef.current
      && lifetime.client === context.client
      && lifetime.scopeId === context.scopeId
      && lifetime.mediaBridge === context.mediaBridge;
  }, []);

  const isCurrentCall = useCallback((lifetime: MediaLifetime, nextCall: HumanCall) => {
    const current = callRef.current;
    return isCurrentLifetime(lifetime)
      && nextCall.scopeId === lifetime.scopeId
      && current?.id === nextCall.id
      && current.generation === nextCall.generation
      && current.state !== "ended" && current.state !== "failed";
  }, [isCurrentLifetime]);

  const publish = useCallback((next: HumanCall | null, nextLease?: CallLease | null) => {
    callRef.current = next;
    setCall(next);
    if (nextLease !== undefined) {
      leaseRef.current = nextLease;
      setLease(nextLease);
    }
  }, []);

  const releaseDisplay = useCallback((updateState: boolean) => {
    const owned = displayRef.current;
    displayRef.current = null;
    if (owned != null) {
      // Physical release can precede a peer/generation replacement. Keep only
      // this local call's obligation until its current owner acknowledges it.
      stoppedDisplayRef.current = owned.operation;
      owned.track.onended = null;
      stopStream(owned.stream);
    }
    if (updateState) setScreenSharing(false);
    return owned;
  }, []);

  const closePeer = useCallback((updateState = true) => {
    peerOperationRef.current += 1;
    stopStream(localCaptureRef.current?.stream ?? null);
    localCaptureRef.current = null;
    displayOperationRef.current = null;
    releaseDisplay(updateState);
    const pc = pcRef.current;
    pcRef.current = null;
    videoSenderRef.current = null;
    if (pc != null) {
      pc.onicecandidate = null;
      pc.ontrack = null;
      pc.onconnectionstatechange = null;
      pc.close();
    }
    stopStream(remoteStreamRef.current);
    remoteStreamRef.current = null;
    if (remoteVideoRef.current != null) remoteVideoRef.current.srcObject = null;
  }, [releaseDisplay]);

  const disposeMedia = useCallback((updateState = true) => {
    // Invalidate first: a Host transition or browser permission reply cannot delay release.
    lifetimeRef.current += 1;
    stopStream(localCaptureRef.current?.stream ?? null);
    localCaptureRef.current = null;
    displayOperationRef.current = null;
    startingRef.current = null;
    recoveringRef.current = null;
    closePeer(updateState);
    stoppedDisplayRef.current = null;
    stopStream(localStreamRef.current);
    localStreamRef.current = null;
    if (localVideoRef.current != null) localVideoRef.current.srcObject = null;
    appliedSignalSeqRef.current = 0;
    cameraEnabledRef.current = false;
    mutedRef.current = false;
    if (updateState) {
      setCameraEnabled(false);
      setMuted(false);
    }
  }, [closePeer]);

  const readLease = useCallback(async (
    lifetime: MediaLifetime,
    nextCall: HumanCall,
    allowClaim: boolean,
    isCurrent = () => isCurrentCall(lifetime, nextCall),
  ) => {
    requireCurrentMedia(isCurrent);
    const raw = await awaitCurrentMedia(lifetime.client.call("getCallTransportLease", { callId: nextCall.id }), isCurrent);
    requireCurrentMedia(isCurrent);
    const next = requireLocalCallLease(raw, allowClaim);
    leaseRef.current = next;
    setLease(next);
    return next;
  }, [isCurrentCall]);

  const updateMediaState = useCallback(async (
    lifetime: MediaLifetime,
    nextCall: HumanCall,
    patch: Record<string, unknown>,
    isCurrent = () => isCurrentCall(lifetime, nextCall),
  ) => {
    requireCurrentMedia(isCurrent);
    const local = localStreamRef.current;
    const mediaCapabilities = {
      audio: local?.getAudioTracks().some((track) => track.readyState === "live" && track.enabled) === true,
      video: local?.getVideoTracks().some((track) => track.readyState === "live" && track.enabled) === true,
      screenShare: displayRef.current != null,
      ...patch,
    };
    const devices: Record<string, unknown> = {};
    const audio = local?.getAudioTracks()[0]?.getSettings();
    const video = local?.getVideoTracks()[0]?.getSettings();
    if (audio?.deviceId) devices.microphoneId = audio.deviceId;
    if (video?.deviceId) devices.cameraId = video.deviceId;
    const updated = await awaitCurrentMedia(lifetime.client.call("updateCallMedia", {
      callId: nextCall.id,
      generation: nextCall.generation,
      mediaCapabilities,
      deviceSelection: devices,
    }), isCurrent);
    requireCurrentMedia(isCurrent);
    if (isCall(updated)) publish(updated);
  }, [isCurrentCall, publish]);

  const transition = useCallback(async (
    lifetime: MediaLifetime,
    nextCall: HumanCall,
    action: string,
    terminalReason?: string,
    isCurrent = () => isCurrentCall(lifetime, nextCall),
  ) => {
    requireCurrentMedia(isCurrent);
    const updated = await awaitCurrentMedia(lifetime.client.call("transitionCallSession", {
      callId: nextCall.id,
      generation: nextCall.generation,
      action,
      ...(terminalReason == null ? {} : { terminalReason }),
    }), isCurrent);
    requireCurrentMedia(isCurrent);
    if (!isCall(updated)) throw new Error("Host returned a malformed call transition.");
    publish(updated);
    return updated;
  }, [isCurrentCall, publish]);

  const finishCall = useCallback(async (
    lifetime: MediaLifetime,
    action: "hangup" | "decline" | "fail",
    reason: string,
  ) => {
    if (!isCurrentLifetime(lifetime)) return;
    const current = callRef.current;
    const retirement = {};
    if (current != null) retiringCallsRef.current.set(current.id, retirement);
    disposeMedia();
    publish(null, null);
    setStatus(action === "fail" ? "failed" : "idle");
    // This terminal command was admitted by the user/current failure before disposal.
    // Its completion must never publish into a later call lifetime.
    try {
      if (current != null && current.state !== "ended" && current.state !== "failed") {
        await lifetime.client.call("transitionCallSession", {
          callId: current.id, generation: current.generation, action, terminalReason: reason,
        });
      }
    } catch {
      // Capture release is independent of an unavailable terminal transition.
    } finally {
      if (current != null && retiringCallsRef.current.get(current.id) === retirement) {
        retiringCallsRef.current.delete(current.id);
      }
    }
  }, [disposeMedia, isCurrentLifetime, publish]);

  const failCall = useCallback(async (lifetime: MediaLifetime, reason: string) => {
    if (!isCurrentLifetime(lifetime)) return;
    setError(reason);
    await finishCall(lifetime, "fail", reason);
  }, [finishCall, isCurrentLifetime]);

  const showCurrentError = useCallback((lifetime: MediaLifetime, failure: unknown) => {
    if (!(failure instanceof CancelledCallMedia) && isCurrentLifetime(lifetime)) {
      setError(mediaFailure(failure));
    }
  }, [isCurrentLifetime]);

  const ensureLocalMedia = useCallback(async (lifetime: MediaLifetime, nextCall: HumanCall, video: boolean) => {
    requireCurrentMedia(() => isCurrentCall(lifetime, nextCall));
    if (localCaptureRef.current != null) throw new CancelledCallMedia("Local capture is already pending.");
    const operation: CaptureOperation = { lifetime, call: nextCall };
    localCaptureRef.current = operation;
    const isCurrent = () => localCaptureRef.current === operation && isCurrentCall(lifetime, nextCall);
    try {
      const permission = await awaitCurrentMedia(bridge.callMedia.requestPermissions({ audio: true, video }), isCurrent);
      requireCurrentMedia(isCurrent);
      if (permission.microphone === "denied" || (video && permission.camera === "denied")) {
        throw new Error("Microphone/camera permission was denied.");
      }
      const stream = await getPreferredUserMedia(bridge, { audio: true, video }, isCurrent).catch((failure) => {
        requireCurrentMedia(isCurrent);
        throw failure;
      });
      if (!isCurrent()) {
        stopStream(stream);
        throw new CancelledCallMedia("Local capture completed after cancellation.");
      }
      if (!stream.getAudioTracks().some((track) => track.readyState === "live")
        || (video && !stream.getVideoTracks().some((track) => track.readyState === "live"))) {
        stopStream(stream);
        throw new Error("Microphone/camera did not provide the requested media tracks.");
      }
      stopStream(localStreamRef.current);
      localStreamRef.current = stream;
      mutedRef.current = false;
      cameraEnabledRef.current = video && stream.getVideoTracks().some((track) => track.readyState === "live");
      setMuted(false);
      setCameraEnabled(cameraEnabledRef.current);
      if (localVideoRef.current != null) localVideoRef.current.srcObject = stream;
      return stream;
    } finally {
      if (localCaptureRef.current === operation) localCaptureRef.current = null;
    }
  }, [bridge, isCurrentCall]);

  const stopSharing = useCallback(async (
    lifetime: MediaLifetime,
    nextCall: HumanCall,
    expected?: DisplayCapture,
  ) => {
    const owned = displayRef.current;
    if (expected != null && owned !== expected) return;
    if (!isCurrentCall(lifetime, nextCall)) return;
    if (owned == null) {
      displayOperationRef.current = null;
      setScreenSharing(false);
      return;
    }
    const operation: CaptureOperation = { lifetime, call: nextCall };
    displayOperationRef.current = operation;
    // Stop supersedes a pending camera upgrade that could otherwise race restoration.
    stopStream(localCaptureRef.current?.stream ?? null);
    localCaptureRef.current = null;
    releaseDisplay(true);
    const isCurrent = () => displayOperationRef.current === operation
      && isCurrentCall(lifetime, nextCall)
      && pcRef.current === owned.peer && videoSenderRef.current === owned.sender;
    try {
      // A previously admitted replacement can settle after Stop. Release tracks now,
      // then restore the sender only while this same peer and stop operation still own it.
      await owned.attachment.catch(() => undefined);
      requireCurrentMedia(isCurrent);
      const camera = localStreamRef.current?.getVideoTracks()
        .find((track) => track.readyState === "live") ?? null;
      await awaitCurrentMedia(owned.sender.replaceTrack(camera), isCurrent);
      requireCurrentMedia(isCurrent);
      await updateMediaState(lifetime, nextCall, { screenShare: false }, isCurrent);
      if (stoppedDisplayRef.current === owned.operation) stoppedDisplayRef.current = null;
    } finally {
      if (displayOperationRef.current === operation) displayOperationRef.current = null;
    }
  }, [isCurrentCall, releaseDisplay, updateMediaState]);

  const sendSignal = useCallback(async (
    lifetime: MediaLifetime,
    nextCall: HumanCall,
    kind: CallSignal["kind"],
    payload: Record<string, unknown>,
    isCurrent = () => isCurrentCall(lifetime, nextCall),
  ) => {
    const owner = await readLease(lifetime, nextCall, true, isCurrent);
    requireCurrentMedia(isCurrent);
    const signal = await awaitCurrentMedia(lifetime.client.call("sendCallSignal", {
      callId: nextCall.id, generation: nextCall.generation,
      seq: Math.max(1, nextCall.signalSeq + 1),
      senderDeviceId: owner.deviceId, kind, payload,
    }), isCurrent);
    requireCurrentMedia(isCurrent);
    if (!isSignal(signal)) throw new Error("Host returned a malformed call signal.");
    appliedSignalSeqRef.current = Math.max(appliedSignalSeqRef.current, signal.seq);
    return signal;
  }, [isCurrentCall, readLease]);

  const configurePeer = useCallback(async (
    lifetime: MediaLifetime,
    nextCall: HumanCall,
    nextLease: CallLease,
    local: MediaStream,
  ) => {
    requireCurrentMedia(() => isCurrentCall(lifetime, nextCall) && localStreamRef.current === local);
    closePeer();
    const stoppedDisplay = stoppedDisplayRef.current;
    const attempt = peerOperationRef.current;
    const isCurrent = () => isCurrentCall(lifetime, nextCall)
      && peerOperationRef.current === attempt && localStreamRef.current === local;
    const ice = await awaitCurrentMedia(lifetime.client.call("getCallIceServers"), isCurrent);
    requireCurrentMedia(isCurrent);
    if (!isRecord(ice) || !Array.isArray(ice.iceServers)) throw new Error("Host returned malformed ICE credentials.");
    const pc = new RTCPeerConnection({ iceServers: (ice as unknown as IceReply).iceServers });
    pcRef.current = pc;
    const isCurrentPeer = () => isCurrent() && pcRef.current === pc;
    for (const track of local.getTracks()) {
      if (track.readyState !== "live") continue;
      const sender = pc.addTrack(track, local);
      if (track.kind === "video") videoSenderRef.current = sender;
    }
    const remote = new MediaStream();
    remoteStreamRef.current = remote;
    if (remoteVideoRef.current != null) remoteVideoRef.current.srcObject = remote;
    pc.ontrack = (event) => {
      if (!isCurrentPeer()) return;
      for (const track of event.streams[0]?.getTracks() ?? [event.track]) {
        if (!remote.getTracks().some((candidate) => candidate.id === track.id)) remote.addTrack(track);
      }
      if (remoteVideoRef.current != null) remoteVideoRef.current.srcObject = remote;
    };
    pc.onicecandidate = (event) => {
      if (event.candidate == null || !isCurrentPeer()) return;
      const current = callRef.current;
      if (current == null) return;
      void sendSignal(lifetime, current, "candidate", event.candidate.toJSON() as unknown as Record<string, unknown>, isCurrentPeer)
        .catch((failure) => {
          if (!(failure instanceof CancelledCallMedia) && isCurrentPeer()) {
            void failCall(lifetime, mediaFailure(failure));
          }
        });
    };
    pc.onconnectionstatechange = () => {
      if (!isCurrentPeer()) return;
      if (pc.connectionState === "connected") {
        setStatus("connected");
        const current = callRef.current;
        if (current?.state === "negotiating" || current?.state === "reconnecting") {
          void transition(lifetime, current, "connected", undefined, isCurrentPeer).catch((failure) => {
            if (!(failure instanceof CancelledCallMedia) && isCurrentPeer()) {
              void failCall(lifetime, mediaFailure(failure));
            }
          });
        }
      }
      if ((pc.connectionState === "failed" || pc.connectionState === "disconnected") && recoveringRef.current == null) {
        const recovery = {};
        recoveringRef.current = recovery;
        void (async () => {
          let expectedCall = callRef.current;
          let expectedAttempt = peerOperationRef.current;
          let expectedPeer: RTCPeerConnection | null = pc;
          const isRecoveryCurrent = () => recoveringRef.current === recovery
            && expectedCall != null && isCurrentCall(lifetime, expectedCall)
            && peerOperationRef.current === expectedAttempt
            && (expectedPeer == null || pcRef.current === expectedPeer);
          try {
            if (expectedCall == null) return;
            requireCurrentMedia(isRecoveryCurrent);
            setStatus("reconnecting");
            const reconnecting = await transition(lifetime, expectedCall, "reconnect", undefined, isRecoveryCurrent);
            expectedCall = reconnecting;
            appliedSignalSeqRef.current = 0;
            const resumed = await transition(lifetime, reconnecting, "resume", undefined, isRecoveryCurrent);
            expectedCall = resumed;
            const owner = await readLease(lifetime, resumed, false, isRecoveryCurrent);
            const stream = localStreamRef.current ?? await awaitCurrentMedia(
              ensureLocalMedia(lifetime, resumed, cameraEnabledRef.current), isRecoveryCurrent);
            requireCurrentMedia(isRecoveryCurrent);
            const restartPeer = configurePeerRef.current;
            if (restartPeer == null) throw new Error("Call media reconnect owner is unavailable.");
            const pending = restartPeer(lifetime, resumed, owner, stream);
            expectedAttempt = peerOperationRef.current;
            expectedPeer = null;
            const restarted = await awaitCurrentMedia(pending, isRecoveryCurrent);
            expectedPeer = restarted;
            requireCurrentMedia(isRecoveryCurrent);
            if (owner.role === "creator") {
              const offer = await awaitCurrentMedia(restarted.createOffer({ iceRestart: true }), isRecoveryCurrent);
              requireCurrentMedia(isRecoveryCurrent);
              await awaitCurrentMedia(restarted.setLocalDescription(offer), isRecoveryCurrent);
              requireCurrentMedia(isRecoveryCurrent);
              await sendSignal(lifetime, resumed, "offer", { type: offer.type, sdp: offer.sdp ?? "" }, isRecoveryCurrent);
            }
          } catch (failure) {
            if (!(failure instanceof CancelledCallMedia) && isRecoveryCurrent()) {
              await failCall(lifetime, mediaFailure(failure));
            }
          } finally {
            if (recoveringRef.current === recovery) recoveringRef.current = null;
          }
        })();
      }
    };
    setStatus(nextCall.state === "reconnecting" ? "reconnecting" : "negotiating");
    if (stoppedDisplay != null && stoppedDisplayRef.current === stoppedDisplay
      && isCurrentLifetime(stoppedDisplay.lifetime) && stoppedDisplay.call.id === nextCall.id) {
      await updateMediaState(lifetime, nextCall, { screenShare: false }, isCurrentPeer);
      if (stoppedDisplayRef.current === stoppedDisplay) stoppedDisplayRef.current = null;
    }
    requireCurrentMedia(isCurrentPeer);
    if (nextLease.role === "creator") {
      const offer = await awaitCurrentMedia(pc.createOffer(), isCurrentPeer);
      requireCurrentMedia(isCurrentPeer);
      await awaitCurrentMedia(pc.setLocalDescription(offer), isCurrentPeer);
      requireCurrentMedia(isCurrentPeer);
      await sendSignal(lifetime, nextCall, "offer", { type: offer.type, sdp: offer.sdp ?? "" }, isCurrentPeer);
    }
    return pc;
  }, [closePeer, ensureLocalMedia, failCall, isCurrentCall, isCurrentLifetime, readLease, sendSignal, transition, updateMediaState]);
  configurePeerRef.current = configurePeer;

  const consumeSignals = useCallback(async (lifetime: MediaLifetime, nextCall: HumanCall, expectedPeer?: RTCPeerConnection) => {
    const pc = expectedPeer ?? pcRef.current;
    const owner = leaseRef.current;
    if (pc == null || owner == null) return;
    const isCurrent = () => isCurrentCall(lifetime, nextCall) && pcRef.current === pc;
    requireCurrentMedia(isCurrent);
    const value = await awaitCurrentMedia(lifetime.client.call("listCallSignals", {
      callId: nextCall.id, generation: nextCall.generation,
      afterSeq: appliedSignalSeqRef.current, limit: 100,
    }), isCurrent);
    requireCurrentMedia(isCurrent);
    if (!Array.isArray(value)) throw new Error("Host returned malformed call signaling.");
    for (const raw of value) {
      requireCurrentMedia(isCurrent);
      if (!isSignal(raw) || raw.generation !== nextCall.generation) continue;
      appliedSignalSeqRef.current = Math.max(appliedSignalSeqRef.current, raw.seq);
      if (raw.senderDeviceId === owner.deviceId) continue;
      if (raw.kind === "offer") {
        const sdp = typeof raw.payload.sdp === "string" ? raw.payload.sdp : "";
        await awaitCurrentMedia(pc.setRemoteDescription({ type: "offer", sdp }), isCurrent);
        requireCurrentMedia(isCurrent);
        const answer = await awaitCurrentMedia(pc.createAnswer(), isCurrent);
        requireCurrentMedia(isCurrent);
        await awaitCurrentMedia(pc.setLocalDescription(answer), isCurrent);
        requireCurrentMedia(isCurrent);
        await sendSignal(lifetime, nextCall, "answer", { type: answer.type, sdp: answer.sdp ?? "" }, isCurrent);
      } else if (raw.kind === "answer") {
        const sdp = typeof raw.payload.sdp === "string" ? raw.payload.sdp : "";
        if (pc.signalingState === "have-local-offer") {
          await awaitCurrentMedia(pc.setRemoteDescription({ type: "answer", sdp }), isCurrent);
          requireCurrentMedia(isCurrent);
        }
      } else if (raw.kind === "candidate") {
        await pc.addIceCandidate(raw.payload as unknown as RTCIceCandidateInit).catch(() => undefined);
        requireCurrentMedia(isCurrent);
      }
    }
  }, [isCurrentCall, sendSignal]);

  const refresh = useCallback(async (lifetime: MediaLifetime) => {
    requireCurrentMedia(() => isCurrentLifetime(lifetime));
    await lifetime.client.call("syncHumanCalls");
    requireCurrentMedia(() => isCurrentLifetime(lifetime));
    const value = await lifetime.client.call("listCallSessions", { scopeId: lifetime.scopeId, limit: 20 });
    requireCurrentMedia(() => isCurrentLifetime(lifetime));
    const candidates = Array.isArray(value)
      ? value.filter((item) => !isCall(item) || !retiringCallsRef.current.has(item.id)) : value;
    const next = selectActiveHumanCall(candidates, lifetime.scopeId);
    if (next == null) {
      if (callRef.current != null || localStreamRef.current != null || localCaptureRef.current != null) disposeMedia();
      publish(null, null);
      setStatus("idle");
      return null;
    }
    let nextLease: CallLease | null = null;
    try {
      const raw = await lifetime.client.call("getCallTransportLease", { callId: next.id });
      requireCurrentMedia(() => isCurrentLifetime(lifetime));
      if (isLease(raw)) nextLease = raw;
    } catch (failure) {
      requireCurrentMedia(() => isCurrentLifetime(lifetime));
      if (!(failure instanceof CancelledCallMedia)) console.warn("[human-call] lease read failed", failure);
    }
    const current = callRef.current;
    if (current != null && current.id !== next.id) {
      disposeMedia();
      setStatus("idle");
      setError(null);
    } else if (current != null && current.generation !== next.generation) {
      closePeer();
      localCaptureRef.current = null;
      appliedSignalSeqRef.current = 0;
    }
    publish(next, nextLease);
    return next;
  }, [closePeer, disposeMedia, isCurrentLifetime, publish]);

  const start = useCallback(async (video: boolean) => {
    if (startingRef.current != null || (callRef.current != null && callRef.current.state !== "ended" && callRef.current.state !== "failed")) return;
    const lifetime = captureLifetime();
    startingRef.current = lifetime;
    const initialCall = callRef.current;
    const attempt = peerOperationRef.current;
    let expectedCall: HumanCall | null = null;
    const isStartCurrent = () => startingRef.current === lifetime && isCurrentLifetime(lifetime)
      && peerOperationRef.current === attempt
      && (expectedCall == null ? callRef.current === initialCall : isCurrentCall(lifetime, expectedCall));
    setError(null);
    try {
      if (conversation.memberIds.length < 2) throw new Error("Human call requires a peer participant.");
      requireCurrentMedia(isStartCurrent);
      const created = await awaitCurrentMedia(lifetime.client.call("createCallSession", {
        scopeId: lifetime.scopeId, participantIds: conversation.memberIds,
      }), isStartCurrent);
      requireCurrentMedia(isStartCurrent);
      if (!isCall(created)) throw new Error("Host returned a malformed created call.");
      expectedCall = created;
      publish(created);
      const ringing = await transition(lifetime, created, "ring", undefined, isStartCurrent);
      expectedCall = ringing;
      const nextLease = await readLease(lifetime, ringing, true, isStartCurrent);
      await awaitCurrentMedia(ensureLocalMedia(lifetime, ringing, video), isStartCurrent);
      requireCurrentMedia(isStartCurrent);
      await updateMediaState(lifetime, ringing, { audio: true, video, screenShare: false }, isStartCurrent);
      requireCurrentMedia(isStartCurrent);
      publish(ringing, nextLease);
      setStatus("ringing");
    } catch (failure) {
      if (!(failure instanceof CancelledCallMedia) && isStartCurrent()) {
        await failCall(lifetime, mediaFailure(failure));
      }
    } finally {
      if (startingRef.current === lifetime) startingRef.current = null;
    }
  }, [captureLifetime, conversation.memberIds, ensureLocalMedia, failCall, isCurrentCall, isCurrentLifetime, publish, readLease, transition, updateMediaState]);

  const accept = useCallback(async (video: boolean) => {
    const current = callRef.current;
    if (current == null || startingRef.current != null) return;
    const lifetime = captureLifetime();
    startingRef.current = lifetime;
    let expectedCall = current;
    let expectedAttempt = peerOperationRef.current;
    const isAcceptCurrent = () => startingRef.current === lifetime
      && isCurrentCall(lifetime, expectedCall) && peerOperationRef.current === expectedAttempt;
    setError(null);
    try {
      const accepted = await transition(lifetime, current, "accept", undefined, isAcceptCurrent);
      expectedCall = accepted;
      const nextLease = await readLease(lifetime, accepted, true, isAcceptCurrent);
      const stream = await awaitCurrentMedia(ensureLocalMedia(lifetime, accepted, video), isAcceptCurrent);
      await updateMediaState(lifetime, accepted, { audio: true, video, screenShare: false }, isAcceptCurrent);
      requireCurrentMedia(isAcceptCurrent);
      publish(accepted, nextLease);
      const pending = configurePeer(lifetime, accepted, nextLease, stream);
      expectedAttempt = peerOperationRef.current;
      const peer = await awaitCurrentMedia(pending, isAcceptCurrent);
      requireCurrentMedia(() => isAcceptCurrent() && pcRef.current === peer);
      await consumeSignals(lifetime, accepted, peer);
    } catch (failure) {
      if (!(failure instanceof CancelledCallMedia) && isAcceptCurrent()) {
        await failCall(lifetime, mediaFailure(failure));
      }
    } finally {
      if (startingRef.current === lifetime) startingRef.current = null;
    }
  }, [captureLifetime, configurePeer, consumeSignals, ensureLocalMedia, failCall, isCurrentCall, publish, readLease, transition, updateMediaState]);

  const decline = useCallback(() => finishCall(captureLifetime(), "decline", "declined"), [captureLifetime, finishCall]);
  const hangup = useCallback(() => finishCall(captureLifetime(), "hangup", "user-ended"), [captureLifetime, finishCall]);

  const toggleMute = useCallback(async () => {
    const lifetime = captureLifetime();
    const stream = localStreamRef.current;
    const current = callRef.current;
    if (stream == null || current == null || !isCurrentCall(lifetime, current)) return;
    const next = !mutedRef.current;
    mutedRef.current = next;
    for (const track of stream.getAudioTracks()) track.enabled = !next;
    setMuted(next);
    try {
      await updateMediaState(lifetime, current, { audio: !next });
    } catch (failure) {
      showCurrentError(lifetime, failure);
    }
  }, [captureLifetime, isCurrentCall, showCurrentError, updateMediaState]);

  const toggleCamera = useCallback(async () => {
    const lifetime = captureLifetime();
    const stream = localStreamRef.current;
    const current = callRef.current;
    if (stream == null || current == null || !isCurrentCall(lifetime, current)) return;
    if (localCaptureRef.current != null || displayOperationRef.current != null) return;
    setError(null);
    const existing = stream.getVideoTracks().find((track) => track.readyState === "live");
    if (existing != null) {
      const enabled = !cameraEnabledRef.current;
      existing.enabled = enabled;
      cameraEnabledRef.current = enabled;
      setCameraEnabled(enabled);
      try {
        await updateMediaState(lifetime, current, { video: enabled });
      } catch (failure) {
        showCurrentError(lifetime, failure);
      }
      return;
    }
    const operation: CaptureOperation = { lifetime, call: current };
    localCaptureRef.current = operation;
    const isCurrent = () => localCaptureRef.current === operation
      && isCurrentCall(lifetime, current) && localStreamRef.current === stream;
    let camera: MediaStream | null = null;
    let adopted = false;
    try {
      const permission = await bridge.callMedia.requestPermissions({ audio: false, video: true });
      requireCurrentMedia(isCurrent);
      if (permission.camera === "denied") throw new Error("Camera permission was denied.");
      camera = await getPreferredUserMedia(bridge, { audio: false, video: true }, isCurrent);
      requireCurrentMedia(isCurrent);
      operation.stream = camera;
      const track = camera.getVideoTracks().find((candidate) => candidate.readyState === "live");
      if (track == null) throw new Error("Camera did not provide a video track.");
      const pc = pcRef.current;
      // An active display retains the sender; the newly selected camera is restored on stop.
      if (pc != null && displayRef.current == null) {
        const sender = videoSenderRef.current;
        if (sender != null) {
          await sender.replaceTrack(track);
          requireCurrentMedia(() => isCurrent() && pcRef.current === pc && videoSenderRef.current === sender);
        } else {
          videoSenderRef.current = pc.addTrack(track, stream);
        }
      }
      requireCurrentMedia(isCurrent);
      for (const prior of stream.getVideoTracks()) {
        if (prior.readyState === "ended") stream.removeTrack(prior);
      }
      stream.addTrack(track);
      adopted = true;
      delete operation.stream;
      for (const unused of camera.getTracks()) {
        if (unused !== track && unused.readyState !== "ended") unused.stop();
      }
      cameraEnabledRef.current = true;
      setCameraEnabled(true);
      if (localVideoRef.current != null) localVideoRef.current.srcObject = stream;
      await updateMediaState(lifetime, current, { video: true }, isCurrent);
    } catch (failure) {
      if (isCurrent()) showCurrentError(lifetime, failure);
    } finally {
      if (!adopted) stopStream(camera);
      if (localCaptureRef.current === operation) localCaptureRef.current = null;
    }
  }, [bridge, captureLifetime, isCurrentCall, showCurrentError, updateMediaState]);

  const toggleScreen = useCallback(async () => {
    const lifetime = captureLifetime();
    const current = callRef.current;
    const stream = localStreamRef.current;
    const pc = pcRef.current;
    if (current == null || stream == null || pc == null || !isCurrentCall(lifetime, current)) return;
    if (displayRef.current != null) {
      setError(null);
      try {
        await stopSharing(lifetime, current);
      } catch (failure) {
        showCurrentError(lifetime, failure);
      }
      return;
    }
    if (displayOperationRef.current != null || localCaptureRef.current != null) return;
    setError(null);
    const operation: CaptureOperation = { lifetime, call: current };
    displayOperationRef.current = operation;
    const isCurrent = () => displayOperationRef.current === operation
      && isCurrentCall(lifetime, current) && localStreamRef.current === stream && pcRef.current === pc;
    let capture: MediaStream | null = null;
    let owned: DisplayCapture | null = null;
    try {
      const sources = await bridge.callMedia.listDisplaySources();
      requireCurrentMedia(isCurrent);
      const source = sources[0];
      if (source == null) throw new Error("No display source is available.");
      capture = await navigator.mediaDevices.getUserMedia({ audio: false, video: desktopSourceConstraint(source.id) });
      if (!isCurrent()) {
        stopStream(capture);
        throw new CancelledCallMedia("Display capture completed after cancellation.");
      }
      const track = capture.getVideoTracks().find((candidate) => candidate.readyState === "live");
      if (track == null) throw new Error("Display source did not provide a video track.");
      let sender = videoSenderRef.current;
      const replace = sender != null;
      if (sender == null) {
        sender = pc.addTrack(track, capture);
        videoSenderRef.current = sender;
      }
      owned = { operation, stream: capture, track, peer: pc, sender, attachment: Promise.resolve() };
      const installed = owned;
      displayRef.current = installed;
      stoppedDisplayRef.current = null;
      setScreenSharing(true);
      track.onended = () => {
        if (displayRef.current !== installed || !isCurrentCall(lifetime, current)) return;
        void stopSharing(lifetime, current, installed).catch((failure) => showCurrentError(lifetime, failure));
      };
      if (replace) installed.attachment = sender.replaceTrack(track);
      await installed.attachment;
      requireCurrentMedia(() => isCurrent() && displayRef.current === installed);
      await updateMediaState(lifetime, current, { screenShare: true, displaySourceId: source.id },
        () => isCurrent() && displayRef.current === installed);
    } catch (failure) {
      const failedWhileCurrent = isCurrent();
      if (owned != null && displayRef.current === owned) {
        await stopSharing(lifetime, current, owned).catch(() => undefined);
      }
      if (failedWhileCurrent && isCurrentCall(lifetime, current)
        && (displayOperationRef.current === operation || displayOperationRef.current == null)) {
        showCurrentError(lifetime, failure);
      }
    } finally {
      if (owned == null) stopStream(capture);
      if (displayOperationRef.current === operation) displayOperationRef.current = null;
    }
  }, [bridge.callMedia, captureLifetime, isCurrentCall, showCurrentError, stopSharing, updateMediaState]);

  useEffect(() => {
    disposedRef.current = false;
    appliedSignalSeqRef.current = 0;
    publish(null, null);
    setStatus("idle");
    setError(null);
    setMuted(false);
    setCameraEnabled(false);
    setScreenSharing(false);
    const poll = async () => {
      if (pollingRef.current != null || disposedRef.current) return;
      const pending = {};
      pollingRef.current = pending;
      const lifetime = captureLifetime();
      try {
        const next = await refresh(lifetime);
        if (next == null) return;
        // Reconciliation may intentionally replace the preceding call's media lifetime.
        const activeLifetime = captureLifetime();
        const nextLease = await readLease(activeLifetime, next, false).catch((failure) => {
          if (failure instanceof CancelledCallMedia) throw failure;
          return null;
        });
        if (nextLease == null || !nextLease.isOwner) return;
        if ((next.state === "negotiating" || next.state === "connected" || next.state === "reconnecting") && pcRef.current == null) {
          const stream = localStreamRef.current;
          if (stream != null) await configurePeer(activeLifetime, next, nextLease, stream);
        }
        await consumeSignals(activeLifetime, next);
      } catch (failure) {
        showCurrentError(lifetime, failure);
      } finally {
        if (pollingRef.current === pending) pollingRef.current = null;
      }
    };
    void poll();
    const timer = window.setInterval(() => void poll(), 1000);
    return () => {
      disposedRef.current = true;
      window.clearInterval(timer);
      pollingRef.current = null;
      disposeMedia(false);
      callRef.current = null;
      leaseRef.current = null;
      retiringCallsRef.current.clear();
    };
  }, [bridge.callMedia, captureLifetime, client, configurePeer, consumeSignals, conversation.id, disposeMedia, publish, readLease, refresh, showCurrentError]);

  const incoming = call != null && lease?.role === "peer" && (call.state === "invited" || call.state === "ringing");
  const active = call != null && call.state !== "ended" && call.state !== "failed";

  return <div className="sand-human-call" role="group" aria-label="Human call controls">
    {!active ? <>
      <button aria-label="Start voice call" onClick={() => void start(false)} type="button">Voice</button>
      <button aria-label="Start video call" onClick={() => void start(true)} type="button">Video</button>
    </> : null}
    {incoming ? <>
      <span role="status">Incoming call</span>
      <button aria-label="Accept voice call" onClick={() => void accept(false)} type="button">Accept</button>
      <button aria-label="Accept video call" onClick={() => void accept(true)} type="button">Accept video</button>
      <button aria-label="Decline call" onClick={() => void decline()} type="button">Decline</button>
    </> : null}
    {active && !incoming ? <>
      <span role="status">{status === "idle" ? call.state : status}</span>
      <button aria-pressed={muted} onClick={() => void toggleMute()} type="button">{muted ? "Unmute" : "Mute"}</button>
      <button aria-pressed={cameraEnabled} onClick={() => void toggleCamera()} type="button">{cameraEnabled ? "Camera off" : "Camera on"}</button>
      <button aria-pressed={screenSharing} onClick={() => void toggleScreen()} type="button">{screenSharing ? "Stop sharing" : "Share screen"}</button>
      <button aria-label="End call" onClick={() => void hangup()} type="button">End</button>
    </> : null}
    <video aria-label="Local call preview" autoPlay muted playsInline ref={localVideoRef} hidden={!cameraEnabled && !screenSharing} />
    <video aria-label="Remote call video" autoPlay playsInline ref={remoteVideoRef} hidden={!active} />
    {error == null ? null : <span role="alert">{error}</span>}
  </div>;
}
