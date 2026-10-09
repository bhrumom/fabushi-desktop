import { useCallback, useEffect, useRef, useState } from "react";
import type { DesktopBridge } from "../recovered/contracts/desktop-bridge";
import { createInFlightCommandFence } from "../recovered/ui/in-flight-command";
import { SandButton } from "../recovered/ui/sand-kit-primitives";
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
  for (const track of stream?.getTracks() ?? []) track.stop();
}

function callMediaConstraint(deviceId: string | null, enabled: boolean): boolean | MediaTrackConstraints {
  if (!enabled) return false;
  return deviceId == null ? true : { deviceId: { exact: deviceId } };
}

async function getPreferredUserMedia(bridge: DesktopBridge, input: { audio: boolean; video: boolean }): Promise<MediaStream> {
  const preferences=await bridge.callMedia.getPreferences().catch(()=>({ microphoneId:null, cameraId:null }));
  const preferred: MediaStreamConstraints = {
    audio: callMediaConstraint(preferences.microphoneId,input.audio),
    video: callMediaConstraint(preferences.cameraId,input.video),
  };
  try {
    return await navigator.mediaDevices.getUserMedia(preferred);
  } catch (error) {
    const stalePreference=(input.audio && preferences.microphoneId!=null) || (input.video && preferences.cameraId!=null);
    const recoverable=error instanceof DOMException && (error.name==="NotFoundError" || error.name==="OverconstrainedError");
    if (!stalePreference || !recoverable) throw error;
    return navigator.mediaDevices.getUserMedia({ audio: input.audio, video: input.video });
  }
}

function mediaFailure(error: unknown): string {
  if (error instanceof DOMException && error.name === "NotAllowedError") return "Microphone/camera permission was denied.";
  return error instanceof Error ? error.message : String(error);
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
  const [mutePending, setMutePending] = useState(false);
  const [callCommandFence] = useState(() => createInFlightCommandFence());
  const localStreamRef = useRef<MediaStream | null>(null);
  const remoteStreamRef = useRef<MediaStream | null>(null);
  const pcRef = useRef<RTCPeerConnection | null>(null);
  const configurePeerRef = useRef<((nextCall: HumanCall, nextLease: CallLease, local: MediaStream) => Promise<RTCPeerConnection>) | null>(null);
  const callRef = useRef<HumanCall | null>(null);
  const leaseRef = useRef<CallLease | null>(null);
  const appliedSignalSeqRef = useRef(0);
  const recoveringRef = useRef(false);
  const disposedRef = useRef(false);
  const localVideoRef = useRef<HTMLVideoElement | null>(null);
  const remoteVideoRef = useRef<HTMLVideoElement | null>(null);

  const publish = useCallback((next: HumanCall | null, nextLease?: CallLease | null) => {
    callRef.current = next;
    setCall(next);
    if (nextLease !== undefined) {
      leaseRef.current = nextLease;
      setLease(nextLease);
    }
  }, []);

  const readLease = useCallback(async (callId: string, allowClaim: boolean) => {
    const next = requireLocalCallLease(await client.call("getCallTransportLease", { callId }), allowClaim);
    leaseRef.current = next;
    setLease(next);
    return next;
  }, [client]);

  const refresh = useCallback(async () => {
    await client.call("syncHumanCalls");
    const next = selectActiveHumanCall(await client.call("listCallSessions", { scopeId: conversation.id, limit: 20 }), conversation.id);
    if (next == null) {
      publish(null, null);
      return null;
    }
    let nextLease: CallLease | null = null;
    try {
      const rawLease = await client.call("getCallTransportLease", { callId: next.id });
      if (isLease(rawLease)) nextLease = rawLease;
    } catch (leaseError) {
      console.warn("[human-call] lease read failed", leaseError);
    }
    publish(next, nextLease);
    return next;
  }, [client, conversation.id, publish]);

  const updateMediaState = useCallback(async (nextCall: HumanCall, patch: Record<string, unknown>) => {
    const mediaCapabilities = {
      audio: localStreamRef.current?.getAudioTracks().some((track) => track.enabled) === true,
      video: localStreamRef.current?.getVideoTracks().some((track) => track.enabled) === true,
      screenShare: screenSharing,
      ...patch,
    };
    const devices: Record<string, unknown> = {};
    const audio = localStreamRef.current?.getAudioTracks()[0]?.getSettings();
    const video = localStreamRef.current?.getVideoTracks()[0]?.getSettings();
    if (audio?.deviceId) devices.microphoneId = audio.deviceId;
    if (video?.deviceId) devices.cameraId = video.deviceId;
    const updated = await client.call("updateCallMedia", {
      callId: nextCall.id,
      generation: nextCall.generation,
      mediaCapabilities,
      deviceSelection: devices,
    });
    if (isCall(updated)) publish(updated);
  }, [client, publish, screenSharing]);

  const sendSignal = useCallback(async (nextCall: HumanCall, kind: CallSignal["kind"], payload: Record<string, unknown>) => {
    const owner = await readLease(nextCall.id, true);
    const fallbackSeq = Math.max(1, nextCall.signalSeq + 1);
    const signal = await client.call("sendCallSignal", {
      callId: nextCall.id,
      generation: nextCall.generation,
      seq: fallbackSeq,
      senderDeviceId: owner.deviceId,
      kind,
      payload,
    });
    if (!isSignal(signal)) throw new Error("Host returned a malformed call signal.");
    // shipping backend event.seq is authoritative. The request seq above exists
    // only for the offline/local fallback store and is not treated as global.
    appliedSignalSeqRef.current = Math.max(appliedSignalSeqRef.current, signal.seq);
    return signal;
  }, [client, readLease]);

  const ensureLocalMedia = useCallback(async (video: boolean) => {
    const permission = await bridge.callMedia.requestPermissions({ audio: true, video });
    if (permission.microphone === "denied" || (video && permission.camera === "denied")) {
      throw new Error("Microphone/camera permission was denied.");
    }
    const stream = await getPreferredUserMedia(bridge, { audio: true, video });
    stopStream(localStreamRef.current);
    localStreamRef.current = stream;
    setMuted(false);
    setCameraEnabled(video && stream.getVideoTracks().length > 0);
    if (localVideoRef.current != null) localVideoRef.current.srcObject = stream;
    return stream;
  }, [bridge.callMedia]);

  const closePeer = useCallback(() => {
    const pc = pcRef.current;
    pcRef.current = null;
    if (pc != null) {
      pc.onicecandidate = null;
      pc.ontrack = null;
      pc.onconnectionstatechange = null;
      pc.close();
    }
    stopStream(remoteStreamRef.current);
    remoteStreamRef.current = null;
    if (remoteVideoRef.current != null) remoteVideoRef.current.srcObject = null;
  }, []);

  const transition = useCallback(async (nextCall: HumanCall, action: string, terminalReason?: string) => {
    const updated = await client.call("transitionCallSession", {
      callId: nextCall.id,
      generation: nextCall.generation,
      action,
      ...(terminalReason == null ? {} : { terminalReason }),
    });
    if (!isCall(updated)) throw new Error("Host returned a malformed call transition.");
    publish(updated);
    return updated;
  }, [client, publish]);

  const failCall = useCallback(async (reason: string) => {
    setError(reason);
    setStatus("failed");
    const current = callRef.current;
    if (current != null && current.state !== "ended" && current.state !== "failed") {
      await transition(current, "fail", reason).catch(() => undefined);
    }
    closePeer();
    stopStream(localStreamRef.current);
    localStreamRef.current = null;
  }, [closePeer, transition]);

  const configurePeer = useCallback(async (nextCall: HumanCall, nextLease: CallLease, local: MediaStream) => {
    closePeer();
    const ice = await client.call("getCallIceServers");
    if (!isRecord(ice) || !Array.isArray(ice.iceServers)) throw new Error("Host returned malformed ICE credentials.");
    const pc = new RTCPeerConnection({ iceServers: (ice as unknown as IceReply).iceServers });
    pcRef.current = pc;
    for (const track of local.getTracks()) pc.addTrack(track, local);
    const remote = new MediaStream();
    remoteStreamRef.current = remote;
    if (remoteVideoRef.current != null) remoteVideoRef.current.srcObject = remote;
    pc.ontrack = (event) => {
      for (const track of event.streams[0]?.getTracks() ?? [event.track]) {
        if (!remote.getTracks().some((candidate) => candidate.id === track.id)) remote.addTrack(track);
      }
      if (remoteVideoRef.current != null) remoteVideoRef.current.srcObject = remote;
    };
    pc.onicecandidate = (event) => {
      if (event.candidate == null || disposedRef.current) return;
      const current = callRef.current;
      if (current == null) return;
      void sendSignal(current, "candidate", event.candidate.toJSON() as unknown as Record<string, unknown>).catch((candidateError) => void failCall(mediaFailure(candidateError)));
    };
    pc.onconnectionstatechange = () => {
      if (pc.connectionState === "connected") {
        setStatus("connected");
        const current = callRef.current;
        if (current?.state === "negotiating" || current?.state === "reconnecting") {
          void transition(current, "connected").catch((transitionError) => void failCall(mediaFailure(transitionError)));
        }
      }
      if ((pc.connectionState === "failed" || pc.connectionState === "disconnected") && !recoveringRef.current) {
        recoveringRef.current = true;
        void (async () => {
          const current = callRef.current;
          if (current == null) return;
          try {
            setStatus("reconnecting");
            const reconnecting = await transition(current, "reconnect");
            appliedSignalSeqRef.current = 0;
            const resumed = await transition(reconnecting, "resume");
            const owner = await readLease(resumed.id, false);
            const stream = localStreamRef.current ?? await ensureLocalMedia(cameraEnabled);
            const restartPeer = configurePeerRef.current;
            if (restartPeer == null) throw new Error("Call media reconnect owner is unavailable.");
            await restartPeer(resumed, owner, stream);
            if (owner.role === "creator") {
              const offer = await pcRef.current!.createOffer({ iceRestart: true });
              await pcRef.current!.setLocalDescription(offer);
              await sendSignal(resumed, "offer", { type: offer.type, sdp: offer.sdp ?? "" });
            }
          } catch (reconnectError) {
            await failCall(mediaFailure(reconnectError));
          } finally {
            recoveringRef.current = false;
          }
        })();
      }
    };
    setStatus(nextCall.state === "reconnecting" ? "reconnecting" : "negotiating");
    if (nextLease.role === "creator") {
      const offer = await pc.createOffer();
      await pc.setLocalDescription(offer);
      await sendSignal(nextCall, "offer", { type: offer.type, sdp: offer.sdp ?? "" });
    }
    return pc;
  }, [cameraEnabled, client, closePeer, ensureLocalMedia, failCall, readLease, sendSignal, transition]);
  configurePeerRef.current = configurePeer;

  const consumeSignals = useCallback(async (nextCall: HumanCall) => {
    const pc = pcRef.current;
    const owner = leaseRef.current;
    if (pc == null || owner == null) return;
    const value = await client.call("listCallSignals", {
      callId: nextCall.id,
      generation: nextCall.generation,
      afterSeq: appliedSignalSeqRef.current,
      limit: 100,
    });
    if (!Array.isArray(value)) throw new Error("Host returned malformed call signaling.");
    for (const raw of value) {
      if (!isSignal(raw) || raw.generation !== nextCall.generation) continue;
      appliedSignalSeqRef.current = Math.max(appliedSignalSeqRef.current, raw.seq);
      if (raw.senderDeviceId === owner.deviceId) continue;
      if (raw.kind === "offer") {
        const sdp = typeof raw.payload.sdp === "string" ? raw.payload.sdp : "";
        await pc.setRemoteDescription({ type: "offer", sdp });
        const answer = await pc.createAnswer();
        await pc.setLocalDescription(answer);
        await sendSignal(nextCall, "answer", { type: answer.type, sdp: answer.sdp ?? "" });
      } else if (raw.kind === "answer") {
        const sdp = typeof raw.payload.sdp === "string" ? raw.payload.sdp : "";
        if (pc.signalingState === "have-local-offer") await pc.setRemoteDescription({ type: "answer", sdp });
      } else if (raw.kind === "candidate") {
        await pc.addIceCandidate(raw.payload as unknown as RTCIceCandidateInit).catch(() => undefined);
      }
    }
  }, [client, sendSignal]);

  const start = useCallback(async (video: boolean) => {
    setError(null);
    try {
      if (conversation.memberIds.length < 2) throw new Error("Human call requires a peer participant.");
      const created = await client.call("createCallSession", { scopeId: conversation.id, participantIds: conversation.memberIds });
      if (!isCall(created)) throw new Error("Host returned a malformed created call.");
      publish(created);
      const ringing = await transition(created, "ring");
      const nextLease = await readLease(ringing.id, true);
      await ensureLocalMedia(video);
      await updateMediaState(ringing, { audio: true, video, screenShare: false });
      publish(ringing, nextLease);
      setStatus("ringing");
    } catch (startError) {
      await failCall(mediaFailure(startError));
    }
  }, [client, conversation.id, conversation.memberIds, ensureLocalMedia, failCall, publish, readLease, transition, updateMediaState]);

  const accept = useCallback(async (video: boolean) => {
    const current = callRef.current;
    if (current == null) return;
    setError(null);
    try {
      const accepted = await transition(current, "accept");
      const nextLease = await readLease(accepted.id, true);
      const stream = await ensureLocalMedia(video);
      await updateMediaState(accepted, { audio: true, video, screenShare: false });
      publish(accepted, nextLease);
      await configurePeer(accepted, nextLease, stream);
      await consumeSignals(accepted);
    } catch (acceptError) {
      await failCall(mediaFailure(acceptError));
    }
  }, [configurePeer, consumeSignals, ensureLocalMedia, failCall, publish, readLease, transition, updateMediaState]);

  const decline = useCallback(async () => {
    const current = callRef.current;
    if (current != null && current.state !== "ended" && current.state !== "failed") {
      await transition(current, "decline", "declined").catch(() => undefined);
    }
    closePeer();
    stopStream(localStreamRef.current);
    localStreamRef.current = null;
    setStatus("idle");
    publish(null, null);
  }, [closePeer, publish, transition]);

  const hangup = useCallback(async () => {
    const current = callRef.current;
    if (current != null && current.state !== "ended" && current.state !== "failed") {
      await transition(current, "hangup", "user-ended").catch(() => undefined);
    }
    closePeer();
    stopStream(localStreamRef.current);
    localStreamRef.current = null;
    setStatus("idle");
    setScreenSharing(false);
    setCameraEnabled(false);
    publish(null, null);
  }, [closePeer, publish, transition]);

  const toggleMute = useCallback(async () => {
    const commandLease = callCommandFence.acquire("toggle-mute");
    if (commandLease == null) return;
    setMutePending(true);
    try {
      const stream = localStreamRef.current;
      const current = callRef.current;
      if (stream == null || current == null) return;
      const next = !muted;
      for (const track of stream.getAudioTracks()) track.enabled = !next;
      setMuted(next);
      try {
        await updateMediaState(current, { audio: !next });
      } catch (error) {
        for (const track of stream.getAudioTracks()) track.enabled = !muted;
        setMuted(muted);
        throw error;
      }
    } finally {
      commandLease.release();
      setMutePending(false);
    }
  }, [callCommandFence, muted, updateMediaState]);

  const toggleCamera = useCallback(async () => {
    const stream = localStreamRef.current;
    const current = callRef.current;
    if (stream == null || current == null) return;
    const existing = stream.getVideoTracks()[0];
    if (existing != null) {
      existing.enabled = !cameraEnabled;
      setCameraEnabled(!cameraEnabled);
      await updateMediaState(current, { video: !cameraEnabled });
      return;
    }
    const permission = await bridge.callMedia.requestPermissions({ audio: false, video: true });
    if (permission.camera === "denied") throw new Error("Camera permission was denied.");
    const camera = await getPreferredUserMedia(bridge, { audio: false, video: true });
    const track = camera.getVideoTracks()[0];
    if (track == null) throw new Error("Camera did not provide a video track.");
    stream.addTrack(track);
    pcRef.current?.addTrack(track, stream);
    setCameraEnabled(true);
    if (localVideoRef.current != null) localVideoRef.current.srcObject = stream;
    await updateMediaState(current, { video: true });
  }, [bridge.callMedia, cameraEnabled, updateMediaState]);

  const toggleScreen = useCallback(async () => {
    const current = callRef.current;
    const stream = localStreamRef.current;
    const pc = pcRef.current;
    if (current == null || stream == null || pc == null) return;
    if (screenSharing) {
      setScreenSharing(false);
      await updateMediaState(current, { screenShare: false });
      return;
    }
    const sources = await bridge.callMedia.listDisplaySources();
    const source = sources[0];
    if (source == null) throw new Error("No display source is available.");
    const capture = await navigator.mediaDevices.getUserMedia({ audio: false, video: desktopSourceConstraint(source.id) });
    const track = capture.getVideoTracks()[0];
    if (track == null) throw new Error("Display source did not provide a video track.");
    const sender = pc.getSenders().find((candidate) => candidate.track?.kind === "video");
    if (sender != null) await sender.replaceTrack(track);
    else pc.addTrack(track, capture);
    setScreenSharing(true);
    track.onended = () => {
      if (disposedRef.current) return;
      setScreenSharing(false);
      const camera = localStreamRef.current?.getVideoTracks()[0] ?? null;
      const currentSender = pcRef.current?.getSenders().find((candidate) => candidate.track?.kind === "video");
      if (currentSender != null) void currentSender.replaceTrack(camera);
      const active = callRef.current;
      if (active != null) void updateMediaState(active, { screenShare: false });
    };
    await updateMediaState(current, { screenShare: true, displaySourceId: source.id });
  }, [bridge.callMedia, screenSharing, updateMediaState]);

  useEffect(() => () => {
    callCommandFence.dispose();
  }, [callCommandFence]);

  useEffect(() => {
    disposedRef.current = false;
    appliedSignalSeqRef.current = 0;
    void refresh().catch((refreshError) => setError(mediaFailure(refreshError)));
    const timer = window.setInterval(() => {
      void (async () => {
        const next = await refresh();
        if (next == null) return;
        const nextLease = await readLease(next.id, false).catch(() => null);
        if (nextLease == null || !nextLease.isOwner) return;
        if ((next.state === "negotiating" || next.state === "connected" || next.state === "reconnecting") && pcRef.current == null) {
          const stream = localStreamRef.current;
          if (stream != null) await configurePeer(next, nextLease, stream);
        }
        await consumeSignals(next);
      })().catch((pollError) => setError(mediaFailure(pollError)));
    }, 1000);
    return () => {
      disposedRef.current = true;
      window.clearInterval(timer);
      closePeer();
      stopStream(localStreamRef.current);
      localStreamRef.current = null;
      publish(null, null);
    };
  }, [closePeer, configurePeer, consumeSignals, conversation.id, publish, readLease, refresh]);

  const incoming = call != null && lease?.role === "peer" && (call.state === "invited" || call.state === "ringing");
  const active = call != null && call.state !== "ended" && call.state !== "failed";

  return <div className="sand-human-call" role="group" aria-label="Human call controls">
    {!active ? <>
      <SandButton aria-label="Start voice call" onClick={() => void start(false)} size="sm" variant="secondary">Voice</SandButton>
      <SandButton aria-label="Start video call" onClick={() => void start(true)} size="sm" variant="secondary">Video</SandButton>
    </> : null}
    {incoming ? <>
      <span role="status">Incoming call</span>
      <SandButton aria-label="Accept voice call" onClick={() => void accept(false)} size="sm">Accept</SandButton>
      <SandButton aria-label="Accept video call" onClick={() => void accept(true)} size="sm">Accept video</SandButton>
      <SandButton aria-label="Decline call" onClick={() => void decline()} sentiment="danger" size="sm" variant="secondary">Decline</SandButton>
    </> : null}
    {active && !incoming ? <>
      <span role="status">{status === "idle" ? call.state : status}</span>
      <SandButton aria-pressed={muted} onClick={() => void toggleMute().catch((toggleError) => setError(mediaFailure(toggleError)))} pending={mutePending} size="sm" variant="secondary">{muted ? "Unmute" : "Mute"}</SandButton>
      <SandButton aria-pressed={cameraEnabled} onClick={() => void toggleCamera().catch((toggleError) => setError(mediaFailure(toggleError)))} size="sm" variant="secondary">{cameraEnabled ? "Camera off" : "Camera on"}</SandButton>
      <SandButton aria-pressed={screenSharing} onClick={() => void toggleScreen().catch((toggleError) => setError(mediaFailure(toggleError)))} size="sm" variant="secondary">{screenSharing ? "Stop sharing" : "Share screen"}</SandButton>
      <SandButton aria-label="End call" onClick={() => void hangup()} sentiment="danger" size="sm">End</SandButton>
    </> : null}
    <video aria-label="Local call preview" autoPlay muted playsInline ref={localVideoRef} hidden={!cameraEnabled && !screenSharing} />
    <video aria-label="Remote call video" autoPlay playsInline ref={remoteVideoRef} hidden={!active} />
    {error == null ? null : <span role="alert">{error}</span>}
  </div>;
}
