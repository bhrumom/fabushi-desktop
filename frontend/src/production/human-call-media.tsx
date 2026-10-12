import { useCallback, useEffect, useRef, useState } from "react";
import type { DesktopBridge } from "../recovered/contracts/desktop-bridge";
import { createInFlightCommandFence } from "../recovered/ui/in-flight-command";
import { OverlayDialog } from "../recovered/ui/overlay-primitives";
import { SandButton } from "../recovered/ui/sand-kit-primitives";
import { SandMenuItem } from "../recovered/ui/sand-menu-item";
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
interface CallMediaSession {
  readonly mediaSessionId: string;
  readonly callId: string;
  readonly userId: string | number;
  readonly deviceId: string;
  readonly generation: number;
  readonly purpose: "publish" | "subscribe";
  readonly status: "pending" | "ready" | "uncertain" | "failed" | "closed";
  readonly mutationState?: string | null;
  readonly cleanupRequired?: boolean;
}
interface CallMediaPublication {
  readonly publicationId: string;
  readonly memberUserId: string | number;
  readonly trackName: string;
  readonly mediaKind: "audio" | "video";
  readonly simulcastRids: readonly string[];
}
interface CallMediaSubscriptionResult {
  readonly subscriptionId: string;
  readonly publicationId: string;
  readonly status: "ready" | "failed";
}
interface SfuRemoteVideoSubscription {
  readonly subscriptionId: string;
  readonly publicationId: string;
  readonly simulcastRids: readonly string[];
  preferredRid: string | null;
}
interface SfuMediaRuntime {
  readonly generation: number;
  readonly publishSessionId: string;
  readonly subscribeSessionId: string;
  readonly publishedMids: Set<string>;
  readonly subscribedPublicationIds: Set<string>;
  readonly remoteVideoSubscriptions: Map<string, SfuRemoteVideoSubscription>;
}
interface CallDisplaySource {
  readonly id: string;
  readonly name: string;
  readonly displayId?: string;
}
interface CallDiagnostics {
  readonly codec: string | null;
  readonly path: string | null;
  readonly roundTripTimeMs: number | null;
  readonly sendBitrateKbps: number | null;
  readonly receiveBitrateKbps: number | null;
  readonly packetLossPercent: number | null;
  readonly jitterMs: number | null;
  readonly sampledAtMs: number;
}
interface CallStatsBaseline {
  readonly sampledAtMs: number;
  readonly sentBytes: number;
  readonly receivedBytes: number;
}

type CallToneKind = "incoming" | "outgoing";
type CallTonePlaybackState = "idle" | CallToneKind | "failed";

export const CALL_TONE_SAMPLE_RATE = 48_000;
export const CALL_TONE_LOOP_COUNT = 60;
export const CALL_TONE_INITIAL_OFFSET_SAMPLES = 0;
const CALL_TONE_PERIOD_SECONDS = 4;

interface CallToneResource {
  readonly samples: Int16Array;
  readonly sampleRate: number;
  loopCount: number;
  offsetSamples: number;
}

export function createCallToneResource(kind: CallToneKind): CallToneResource {
  const sampleRate = CALL_TONE_SAMPLE_RATE;
  const sampleCount = sampleRate * CALL_TONE_PERIOD_SECONDS;
  const samples = new Int16Array(sampleCount);
  const primaryFrequency = kind === "incoming" ? 660 : 440;
  const secondaryFrequency = kind === "incoming" ? 880 : 523.25;
  const active = (seconds: number) => kind === "incoming"
    ? (seconds < 0.34 || (seconds >= 0.5 && seconds < 0.84))
    : seconds < 0.72;
  for (let index = 0; index < sampleCount; index += 1) {
    const seconds = index / sampleRate;
    if (!active(seconds)) continue;
    const phase = seconds * Math.PI * 2;
    const envelope = Math.min(1, seconds * 30) * Math.min(1, Math.max(0, 0.9 - seconds) * 30);
    const mixed = (Math.sin(phase * primaryFrequency) + (0.35 * Math.sin(phase * secondaryFrequency))) / 1.35;
    samples[index] = Math.round(Math.max(-1, Math.min(1, mixed * envelope * 0.12)) * 0x7fff);
  }
  return {
    samples,
    sampleRate,
    loopCount: CALL_TONE_LOOP_COUNT,
    offsetSamples: CALL_TONE_INITIAL_OFFSET_SAMPLES,
  };
}

function advanceCallToneOffset(resource: CallToneResource, elapsedSeconds: number): void {
  if (resource.samples.length === 0 || resource.loopCount <= 0) {
    resource.offsetSamples = 0;
    resource.loopCount = 0;
    return;
  }
  const elapsedSamples = Math.max(0, Math.floor(elapsedSeconds * resource.sampleRate));
  const totalSamples = resource.offsetSamples + elapsedSamples;
  const completedLoops = Math.floor(totalSamples / resource.samples.length);
  if (completedLoops >= resource.loopCount) {
    resource.offsetSamples = 0;
    resource.loopCount = 0;
    return;
  }
  resource.loopCount -= completedLoops;
  resource.offsetSamples = totalSamples % resource.samples.length;
}

class HumanCallTonePlayer {
  private context: AudioContext | null = null;
  private source: AudioBufferSourceNode | null = null;
  private resource: CallToneResource | null = null;
  private startedAtContextTime = 0;

  async start(kind: CallToneKind): Promise<CallToneResource> {
    this.stop(false);
    const reusable = this.context?.state === "closed" ? null : this.context;
    this.context = reusable ?? new AudioContext({ sampleRate: CALL_TONE_SAMPLE_RATE });
    if (this.context.state === "suspended") await this.context.resume();
    if (this.context.state !== "running") {
      throw new Error("Desktop audio playback is blocked until the app receives an audio gesture.");
    }

    const resource = createCallToneResource(kind);
    const buffer = this.context.createBuffer(1, resource.samples.length, resource.sampleRate);
    const channel = new Float32Array(resource.samples.length);
    for (let index = 0; index < resource.samples.length; index += 1) {
      channel[index] = resource.samples[index] / 0x8000;
    }
    buffer.copyToChannel(channel, 0);

    const source = this.context.createBufferSource();
    source.buffer = buffer;
    source.loop = resource.loopCount > 1;
    source.loopStart = 0;
    source.loopEnd = buffer.duration;
    source.connect(this.context.destination);

    const initialOffset = resource.offsetSamples / resource.sampleRate;
    const remainingSamples = (resource.samples.length - resource.offsetSamples)
      + (Math.max(0, resource.loopCount - 1) * resource.samples.length);
    source.onended = () => {
      if (this.source !== source) return;
      this.source = null;
      this.resource = null;
      resource.offsetSamples = 0;
      resource.loopCount = 0;
    };
    source.start(0, initialOffset);
    source.stop(this.context.currentTime + (remainingSamples / resource.sampleRate));
    this.startedAtContextTime = this.context.currentTime;
    this.resource = resource;
    this.source = source;
    return resource;
  }

  stop(preserveOffset: boolean): void {
    const source = this.source;
    const resource = this.resource;
    const context = this.context;
    this.source = null;
    this.resource = null;
    if (source != null) {
      source.onended = null;
      try { source.stop(); } catch { /* already stopped */ }
      source.disconnect();
    }
    if (resource != null) {
      if (preserveOffset && context != null) {
        advanceCallToneOffset(resource, Math.max(0, context.currentTime - this.startedAtContextTime));
      } else {
        resource.offsetSamples = 0;
        resource.loopCount = 0;
      }
    }
  }

  async dispose(): Promise<void> {
    this.stop(false);
    const context = this.context;
    this.context = null;
    if (context != null && context.state !== "closed") await context.close().catch(() => undefined);
  }
}

function statNumber(record: Record<string, unknown> | undefined, key: string): number | null {
  const value = record?.[key];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

export function projectCallDiagnostics(report: RTCStatsReport, previous: CallStatsBaseline | null, sampledAtMs: number): { diagnostics: CallDiagnostics; baseline: CallStatsBaseline } {
  const records: Record<string, unknown>[] = [];
  const byId = new Map<string, Record<string, unknown>>();
  report.forEach((raw) => {
    if (!isRecord(raw)) return;
    records.push(raw);
    if (typeof raw.id === "string") byId.set(raw.id, raw);
  });
  const inbound = records.filter((record) => record.type === "inbound-rtp" && record.isRemote !== true);
  const outbound = records.filter((record) => record.type === "outbound-rtp" && record.isRemote !== true);
  const selectedPair = records.find((record) => record.type === "candidate-pair" && record.state === "succeeded" && (record.nominated === true || record.selected === true))
    ?? records.find((record) => record.type === "candidate-pair" && record.state === "succeeded");
  const localCandidate = typeof selectedPair?.localCandidateId === "string" ? byId.get(selectedPair.localCandidateId) : undefined;
  const remoteCandidate = typeof selectedPair?.remoteCandidateId === "string" ? byId.get(selectedPair.remoteCandidateId) : undefined;
  const codecId = [...outbound, ...inbound].map((record) => record.codecId).find((value): value is string => typeof value === "string");
  const codecRecord = codecId == null ? undefined : byId.get(codecId);
  const sentBytes = outbound.reduce((total, record) => total + (statNumber(record, "bytesSent") ?? 0), 0);
  const receivedBytes = inbound.reduce((total, record) => total + (statNumber(record, "bytesReceived") ?? 0), 0);
  const packetsLost = inbound.reduce((total, record) => total + Math.max(0, statNumber(record, "packetsLost") ?? 0), 0);
  const packetsReceived = inbound.reduce((total, record) => total + Math.max(0, statNumber(record, "packetsReceived") ?? 0), 0);
  const jitterSeconds = inbound.reduce<number | null>((max, record) => {
    const value = statNumber(record, "jitter");
    return value == null ? max : Math.max(max ?? 0, value);
  }, null);
  const elapsedMs = previous == null ? 0 : Math.max(0, sampledAtMs - previous.sampledAtMs);
  const bitrate = (bytes: number, prior: number): number | null => elapsedMs <= 0 ? null : Math.max(0, Math.round((((bytes - prior) * 8) / elapsedMs) * 10) / 10);
  const localType = typeof localCandidate?.candidateType === "string" ? localCandidate.candidateType : null;
  const remoteType = typeof remoteCandidate?.candidateType === "string" ? remoteCandidate.candidateType : null;
  const protocol = typeof localCandidate?.protocol === "string" ? localCandidate.protocol : null;
  return {
    diagnostics: {
      codec: typeof codecRecord?.mimeType === "string" ? codecRecord.mimeType : null,
      path: localType == null && remoteType == null ? null : `${localType ?? "unknown"} -> ${remoteType ?? "unknown"}${protocol == null ? "" : ` (${protocol})`}`,
      roundTripTimeMs: statNumber(selectedPair, "currentRoundTripTime") == null ? null : Math.round((statNumber(selectedPair, "currentRoundTripTime") ?? 0) * 1000),
      sendBitrateKbps: previous == null ? null : bitrate(sentBytes, previous.sentBytes),
      receiveBitrateKbps: previous == null ? null : bitrate(receivedBytes, previous.receivedBytes),
      packetLossPercent: packetsLost + packetsReceived <= 0 ? null : Math.round((packetsLost / (packetsLost + packetsReceived)) * 10000) / 100,
      jitterMs: jitterSeconds == null ? null : Math.round(jitterSeconds * 1000),
      sampledAtMs,
    },
    baseline: { sampledAtMs, sentBytes, receivedBytes },
  };
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

function isCallMediaSession(value: unknown): value is CallMediaSession {
  return isRecord(value)
    && typeof value.mediaSessionId === "string"
    && typeof value.callId === "string"
    && (typeof value.userId === "string" || typeof value.userId === "number")
    && typeof value.deviceId === "string"
    && typeof value.generation === "number"
    && (value.purpose === "publish" || value.purpose === "subscribe")
    && ["pending", "ready", "uncertain", "failed", "closed"].includes(String(value.status));
}

function isCallMediaPublication(value: unknown): value is CallMediaPublication {
  return isRecord(value)
    && typeof value.publicationId === "string"
    && (typeof value.memberUserId === "string" || typeof value.memberUserId === "number")
    && typeof value.trackName === "string"
    && (value.mediaKind === "audio" || value.mediaKind === "video")
    && Array.isArray(value.simulcastRids)
    && value.simulcastRids.every((rid) => typeof rid === "string");
}

function callMediaNonce(prefix: string): string {
  return `${prefix}_${crypto.randomUUID().replaceAll("-", "")}`;
}

async function waitForIceGatheringComplete(pc: RTCPeerConnection): Promise<void> {
  if (pc.iceGatheringState === "complete") return;
  await new Promise<void>((resolve, reject) => {
    const timeout = window.setTimeout(() => {
      pc.removeEventListener("icegatheringstatechange", changed);
      reject(new Error("Call media ICE gathering timed out."));
    }, 10_000);
    const changed = () => {
      if (pc.iceGatheringState !== "complete") return;
      window.clearTimeout(timeout);
      pc.removeEventListener("icegatheringstatechange", changed);
      resolve();
    };
    pc.addEventListener("icegatheringstatechange", changed);
  });
}

function requireSessionDescription(value: unknown, expectedType: "offer" | "answer"): RTCSessionDescriptionInit {
  if (!isRecord(value) || value.type !== expectedType || typeof value.sdp !== "string" || value.sdp.length === 0) {
    throw new Error(`Host returned a malformed SFU ${expectedType}.`);
  }
  return { type: expectedType, sdp: value.sdp };
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
  const [cameraCapturePaused, setCameraCapturePaused] = useState(false);
  const [screenSharing, setScreenSharing] = useState(false);
  const [screenCapturePaused, setScreenCapturePaused] = useState(false);
  const [displaySources, setDisplaySources] = useState<readonly CallDisplaySource[]>([]);
  const [displayPickerOpen, setDisplayPickerOpen] = useState(false);
  const [diagnostics, setDiagnostics] = useState<CallDiagnostics | null>(null);
  const [mutePending, setMutePending] = useState(false);
  const [cameraPending, setCameraPending] = useState(false);
  const [screenPending, setScreenPending] = useState(false);
  const [layerPending, setLayerPending] = useState(false);
  const [remoteVideoLayer, setRemoteVideoLayer] = useState<{ readonly current: string | null; readonly available: readonly string[] } | null>(null);
  const [callToneState, setCallToneState] = useState<CallTonePlaybackState>("idle");
  const [callCommandFence] = useState(() => createInFlightCommandFence());
  const localStreamRef = useRef<MediaStream | null>(null);
  const remoteStreamRef = useRef<MediaStream | null>(null);
  const screenStreamRef = useRef<MediaStream | null>(null);
  const cameraCaptureGenerationRef = useRef(0);
  const cameraRecoveryInFlightRef = useRef(false);
  const bindCameraTrackLifecycleRef = useRef<(stream: MediaStream, track: MediaStreamTrack) => void>(() => {});
  const pcRef = useRef<RTCPeerConnection | null>(null);
  const subscribePcRef = useRef<RTCPeerConnection | null>(null);
  const sfuRuntimeRef = useRef<SfuMediaRuntime | null>(null);
  const configurePeerRef = useRef<((nextCall: HumanCall, nextLease: CallLease, local: MediaStream) => Promise<RTCPeerConnection>) | null>(null);
  const callRef = useRef<HumanCall | null>(null);
  const leaseRef = useRef<CallLease | null>(null);
  const sfuSyncInFlightRef = useRef(false);
  const recoveringRef = useRef(false);
  const disposedRef = useRef(false);
  const displayPickerGenerationRef = useRef(0);
  const diagnosticsGenerationRef = useRef(0);
  const diagnosticsTimerRef = useRef<number | null>(null);
  const diagnosticsBaselineRef = useRef<CallStatsBaseline | null>(null);
  const localVideoRef = useRef<HTMLVideoElement | null>(null);
  const remoteVideoRef = useRef<HTMLVideoElement | null>(null);
  const callTonePlayerRef = useRef<HumanCallTonePlayer | null>(null);
  const callToneKeyRef = useRef<string | null>(null);
  const callToneFailedKeyRef = useRef<string | null>(null);

  const publish = useCallback((next: HumanCall | null, nextLease?: CallLease | null) => {
    callRef.current = next;
    setCall(next);
    if (nextLease !== undefined) {
      leaseRef.current = nextLease;
      setLease(nextLease);
    }
  }, []);

  const stopCallTone = useCallback((preserveOffset = false, resetFailure = false) => {
    callToneKeyRef.current = null;
    callTonePlayerRef.current?.stop(preserveOffset);
    if (resetFailure) callToneFailedKeyRef.current = null;
    setCallToneState("idle");
  }, []);

  const startCallToneFor = useCallback(async (nextCall: HumanCall | null, nextLease: CallLease | null, force = false) => {
    const ringing = nextCall != null
      && nextLease != null
      && (nextLease.isOwner || nextLease.claimAvailable)
      && (nextCall.state === "invited" || nextCall.state === "ringing");
    if (!ringing) {
      stopCallTone(false, true);
      return;
    }
    const kind: CallToneKind = nextLease.role === "peer" ? "incoming" : "outgoing";
    const key = `${nextCall.id}:${nextCall.generation}:${kind}`;
    if (callToneKeyRef.current === key) return;
    if (!force && callToneFailedKeyRef.current === key) return;
    stopCallTone(false, false);
    if (force) callToneFailedKeyRef.current = null;
    callToneKeyRef.current = key;
    const player = callTonePlayerRef.current ?? new HumanCallTonePlayer();
    callTonePlayerRef.current = player;
    try {
      await player.start(kind);
      if (callToneKeyRef.current === key) setCallToneState(kind);
    } catch (toneError) {
      if (callToneKeyRef.current !== key) return;
      callToneKeyRef.current = null;
      callToneFailedKeyRef.current = key;
      setCallToneState("failed");
      setError(`Call audio feedback is unavailable. You can still answer or end the call. ${mediaFailure(toneError)}`);
    }
  }, [stopCallTone]);

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
      audio: localStreamRef.current?.getAudioTracks().some((track) => track.enabled && track.readyState === "live") === true,
      video: localStreamRef.current?.getVideoTracks().some((track) => track.enabled && track.readyState === "live") === true,
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

  const updateMediaStateRef = useRef(updateMediaState);
  updateMediaStateRef.current = updateMediaState;

  const invalidateCameraCapture = useCallback(() => {
    cameraCaptureGenerationRef.current += 1;
    cameraRecoveryInFlightRef.current = false;
  }, []);

  const bindCameraTrackLifecycle = useCallback((stream: MediaStream, track: MediaStreamTrack) => {
    const generation = ++cameraCaptureGenerationRef.current;
    cameraRecoveryInFlightRef.current = false;
    const isOwned = () => !disposedRef.current
      && cameraCaptureGenerationRef.current === generation
      && localStreamRef.current === stream
      && stream.getVideoTracks()[0] === track;
    const projectActivity = (paused: boolean) => {
      if (!isOwned()) return;
      setCameraCapturePaused(paused);
      const active = callRef.current;
      if (active == null || active.state === "ended" || active.state === "failed") return;
      void updateMediaStateRef.current(active, {
        video: track.enabled,
        videoPaused: paused,
        videoActive: track.enabled && !paused && track.readyState === "live",
      }).catch((activityError) => setError(mediaFailure(activityError)));
    };
    track.onmute = () => projectActivity(true);
    track.onunmute = () => projectActivity(false);
    track.onended = () => {
      if (!track.enabled || !isOwned() || cameraRecoveryInFlightRef.current) return;
      cameraRecoveryInFlightRef.current = true;
      setCameraCapturePaused(false);
      void (async () => {
        let replacement: MediaStreamTrack | null = null;
        let sender: RTCRtpSender | null = null;
        let addedSender: RTCRtpSender | null = null;
        try {
          const active = callRef.current;
          if (active == null || active.state === "ended" || active.state === "failed") return;
          const recoveryStream = await getPreferredUserMedia(bridge, { audio: false, video: true });
          replacement = recoveryStream.getVideoTracks()[0] ?? null;
          if (replacement == null) {
            stopStream(recoveryStream);
            throw new Error("Camera recovery did not provide a video track.");
          }
          if (!isOwned()) {
            stopStream(recoveryStream);
            return;
          }
          const pc = pcRef.current;
          if (screenStreamRef.current == null && pc != null) {
            sender = pc.getSenders().find((candidate) => candidate.track?.kind === "video") ?? null;
          }
          stream.removeTrack(track);
          stream.addTrack(replacement);
          if (sender != null) await sender.replaceTrack(replacement);
          else if (screenStreamRef.current == null && pc != null) addedSender = pc.addTrack(replacement, stream);
          try {
            await updateMediaStateRef.current(active, {
              video: true,
              videoPaused: replacement.muted,
              videoActive: !replacement.muted && replacement.readyState === "live",
            });
          } catch (projectionError) {
            stream.removeTrack(replacement);
            replacement.stop();
            if (sender != null) await sender.replaceTrack(null).catch(() => undefined);
            if (addedSender != null && pc != null && pcRef.current === pc) pc.removeTrack(addedSender);
            throw projectionError;
          }
          bindCameraTrackLifecycleRef.current(stream, replacement);
          setCameraEnabled(true);
          setCameraCapturePaused(replacement.muted);
          if (localVideoRef.current != null) localVideoRef.current.srcObject = stream;
        } catch (recoveryError) {
          if (cameraCaptureGenerationRef.current !== generation || localStreamRef.current !== stream) return;
          stream.removeTrack(track);
          invalidateCameraCapture();
          setCameraEnabled(false);
          setCameraCapturePaused(false);
          const active = callRef.current;
          if (active != null && active.state !== "ended" && active.state !== "failed") {
            await updateMediaStateRef.current(active, { video: false, videoPaused: false, videoActive: false }).catch(() => undefined);
          }
          setError(`Camera capture stopped after one recovery attempt: ${mediaFailure(recoveryError)}`);
        } finally {
          cameraRecoveryInFlightRef.current = false;
        }
      })();
    };
  }, [bridge, invalidateCameraCapture]);
  bindCameraTrackLifecycleRef.current = bindCameraTrackLifecycle;

  const closeRemoteMediaSessions = useCallback(async (nextCall: HumanCall) => {
    const value = await client.call("listCallMediaSessions", { callId: nextCall.id });
    if (!Array.isArray(value)) throw new Error("Host returned malformed call media recovery state.");
    for (const raw of value) {
      if (!isCallMediaSession(raw) || raw.callId !== nextCall.id) {
        throw new Error("Host returned a malformed recovered call media session.");
      }
      if (raw.status === "closed" || raw.generation > nextCall.generation) continue;
      await client.call("closeCallMediaSession", {
        callId: nextCall.id,
        mediaSessionId: raw.mediaSessionId,
        clientTeardownId: callMediaNonce("teardown"),
        generation: raw.generation,
      });
    }
  }, [client]);

  const ensureLocalMedia = useCallback(async (video: boolean) => {
    const permission = await bridge.callMedia.requestPermissions({ audio: true, video });
    if (permission.microphone === "denied" || (video && permission.camera === "denied")) {
      throw new Error("Microphone/camera permission was denied.");
    }
    const stream = await getPreferredUserMedia(bridge, { audio: true, video });
    invalidateCameraCapture();
    stopStream(localStreamRef.current);
    localStreamRef.current = stream;
    setMuted(false);
    const cameraTrack = stream.getVideoTracks()[0] ?? null;
    setCameraEnabled(video && cameraTrack != null);
    setCameraCapturePaused(cameraTrack?.muted === true);
    if (cameraTrack != null) bindCameraTrackLifecycle(stream, cameraTrack);
    if (localVideoRef.current != null) localVideoRef.current.srcObject = stream;
    return stream;
  }, [bindCameraTrackLifecycle, bridge.callMedia, invalidateCameraCapture]);

  const stopDiagnostics = useCallback(() => {
    diagnosticsGenerationRef.current += 1;
    if (diagnosticsTimerRef.current != null) window.clearInterval(diagnosticsTimerRef.current);
    diagnosticsTimerRef.current = null;
    diagnosticsBaselineRef.current = null;
    setDiagnostics(null);
  }, []);

  const startDiagnostics = useCallback((pc: RTCPeerConnection, callGeneration: number) => {
    stopDiagnostics();
    const samplingGeneration = diagnosticsGenerationRef.current;
    const sample = async () => {
      const report = await pc.getStats();
      if (disposedRef.current || diagnosticsGenerationRef.current !== samplingGeneration || pcRef.current !== pc || callRef.current?.generation !== callGeneration) return;
      const projection = projectCallDiagnostics(report, diagnosticsBaselineRef.current, Date.now());
      diagnosticsBaselineRef.current = projection.baseline;
      setDiagnostics(projection.diagnostics);
    };
    void sample().catch((statsError) => console.warn("[human-call] diagnostics sample failed", statsError));
    diagnosticsTimerRef.current = window.setInterval(() => {
      void sample().catch((statsError) => console.warn("[human-call] diagnostics sample failed", statsError));
    }, 2000);
  }, [stopDiagnostics]);

  const closePeer = useCallback(() => {
    stopDiagnostics();
    const publishPc = pcRef.current;
    const subscribePc = subscribePcRef.current;
    pcRef.current = null;
    subscribePcRef.current = null;
    sfuRuntimeRef.current = null;
    sfuSyncInFlightRef.current = false;
    setRemoteVideoLayer(null);
    setLayerPending(false);
    for (const pc of [publishPc, subscribePc]) {
      if (pc == null) continue;
      pc.onicecandidate = null;
      pc.ontrack = null;
      pc.onconnectionstatechange = null;
      pc.close();
    }
    stopStream(remoteStreamRef.current);
    remoteStreamRef.current = null;
    if (remoteVideoRef.current != null) remoteVideoRef.current.srcObject = null;
  }, [stopDiagnostics]);

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
    stopCallTone(false, true);
    const current = callRef.current;
    if (current != null && current.state !== "ended" && current.state !== "failed") {
      await transition(current, "fail", reason).catch(() => undefined);
    }
    closePeer();
    invalidateCameraCapture();
    stopStream(localStreamRef.current);
    localStreamRef.current = null;
    stopStream(screenStreamRef.current);
    screenStreamRef.current = null;
    setScreenSharing(false);
    setScreenCapturePaused(false);
    setCameraEnabled(false);
    setCameraCapturePaused(false);
  }, [closePeer, invalidateCameraCapture, stopCallTone, transition]);

  const syncSfuSubscriptions = useCallback(async (nextCall: HumanCall) => {
    const runtime = sfuRuntimeRef.current;
    const subscribePc = subscribePcRef.current;
    const owner = leaseRef.current;
    if (runtime == null || subscribePc == null || owner == null || runtime.generation !== nextCall.generation) return;
    if (sfuSyncInFlightRef.current) return;
    sfuSyncInFlightRef.current = true;
    try {
      const discovered = await client.call("discoverCallMediaPublications", { callId: nextCall.id });
      if (!isRecord(discovered) || discovered.generation !== nextCall.generation || !Array.isArray(discovered.publications)) {
        throw new Error("Host returned malformed SFU publication discovery.");
      }
      const pending = discovered.publications
        .filter(isCallMediaPublication)
        .filter((publication) => String(publication.memberUserId) !== owner.userId)
        .filter((publication) => !runtime.subscribedPublicationIds.has(publication.publicationId));
      if (pending.length === 0) return;

      const clientOperationId = callMediaNonce("subscribe");
      const subscriptionIntents = pending.map((publication) => ({
        clientSubscriptionId: callMediaNonce("sub"),
        publicationId: publication.publicationId,
      }));
      const response = await client.call("subscribeCallMedia", {
        callId: nextCall.id,
        mediaSessionId: runtime.subscribeSessionId,
        clientOperationId,
        generation: nextCall.generation,
        subscriptions: subscriptionIntents,
      });
      if (!isRecord(response)) throw new Error("Host returned malformed SFU subscription state.");
      const expectedSubscriptions = new Map(
        subscriptionIntents.map((intent) => [intent.clientSubscriptionId, intent.publicationId] as const),
      );
      const successfulIds = new Set<string>();
      const successfulSubscriptions = new Map<string, string>();
      const seenSubscriptionIds = new Set<string>();
      let failedSubscriptions = 0;
      if (!Array.isArray(response.results)) throw new Error("Host returned incomplete SFU subscription results.");
      for (const raw of response.results) {
        if (!isRecord(raw) || typeof raw.clientSubscriptionId !== "string" || typeof raw.publicationId !== "string"
          || (raw.status !== "ready" && raw.status !== "failed")) {
          throw new Error("Host returned malformed SFU subscription result.");
        }
        const expectedPublicationId = expectedSubscriptions.get(raw.clientSubscriptionId);
        if (expectedPublicationId == null || expectedPublicationId !== raw.publicationId || seenSubscriptionIds.has(raw.clientSubscriptionId)) {
          throw new Error("Host returned mismatched SFU subscription result identity.");
        }
        seenSubscriptionIds.add(raw.clientSubscriptionId);
        if (raw.status === "ready") {
          if (typeof raw.subscriptionId !== "string") throw new Error("Host returned SFU subscription without durable identity.");
          successfulIds.add(raw.publicationId);
          successfulSubscriptions.set(raw.publicationId, raw.subscriptionId);
        } else {
          failedSubscriptions += 1;
        }
      }
      if (seenSubscriptionIds.size !== subscriptionIntents.length) throw new Error("Host returned incomplete SFU subscription results.");
      const offer = requireSessionDescription(response.sessionDescription, "offer");
      await subscribePc.setRemoteDescription(offer);
      const answer = await subscribePc.createAnswer();
      await subscribePc.setLocalDescription(answer);
      await waitForIceGatheringComplete(subscribePc);
      const localAnswer = subscribePc.localDescription;
      if (localAnswer?.type !== "answer" || typeof localAnswer.sdp !== "string") {
        throw new Error("SFU subscription answer was not available after ICE gathering.");
      }
      await client.call("answerCallMediaSubscription", {
        callId: nextCall.id,
        mediaSessionId: runtime.subscribeSessionId,
        clientOperationId,
        generation: nextCall.generation,
        sessionDescription: { type: "answer", sdp: localAnswer.sdp },
      });
      for (const publication of pending) {
        if (!successfulIds.has(publication.publicationId)) continue;
        runtime.subscribedPublicationIds.add(publication.publicationId);
        const subscriptionId = successfulSubscriptions.get(publication.publicationId);
        if (subscriptionId != null && publication.mediaKind === "video" && publication.simulcastRids.length > 0) {
          runtime.remoteVideoSubscriptions.set(publication.publicationId, {
            subscriptionId,
            publicationId: publication.publicationId,
            simulcastRids: publication.simulcastRids,
            preferredRid: null,
          });
        }
      }
      const layerCandidate = Array.from(runtime.remoteVideoSubscriptions.values())[0] ?? null;
      setRemoteVideoLayer(layerCandidate == null ? null : {
        current: layerCandidate.preferredRid,
        available: layerCandidate.simulcastRids,
      });
      if (failedSubscriptions > 0) {
        setError(`Some remote media could not be subscribed (${failedSubscriptions}/${subscriptionIntents.length}); the remaining tracks stay connected.`);
      }
    } finally {
      sfuSyncInFlightRef.current = false;
    }
  }, [client]);

  const cycleRemoteVideoLayer = useCallback(async () => {
    const commandLease = callCommandFence.acquire("remote-video-layer");
    if (commandLease == null) return;
    setLayerPending(true);
    try {
      const current = callRef.current;
      const runtime = sfuRuntimeRef.current;
      if (current == null || runtime == null || runtime.generation !== current.generation) return;
      const subscription = Array.from(runtime.remoteVideoSubscriptions.values())[0] ?? null;
      if (subscription == null || subscription.simulcastRids.length === 0) return;
      const currentIndex = subscription.preferredRid == null
        ? -1
        : subscription.simulcastRids.indexOf(subscription.preferredRid);
      const nextRid = subscription.simulcastRids[(currentIndex + 1) % subscription.simulcastRids.length];
      const response = await client.call("updateCallMediaSubscriptionLayer", {
        callId: current.id,
        mediaSessionId: runtime.subscribeSessionId,
        subscriptionId: subscription.subscriptionId,
        clientMutationId: callMediaNonce("layer"),
        generation: current.generation,
        simulcast: {
          preferredRid: nextRid,
          priorityOrdering: "none",
          ridNotAvailable: "none",
        },
      });
      if (!isRecord(response) || response.success !== true || !isRecord(response.result)
        || response.result.status !== "applied" || response.result.preferredRid !== nextRid) {
        throw new Error("Host returned an unconfirmed SFU layer mutation.");
      }
      subscription.preferredRid = nextRid;
      setRemoteVideoLayer({ current: nextRid, available: subscription.simulcastRids });
    } finally {
      commandLease.release();
      setLayerPending(false);
    }
  }, [callCommandFence, client]);

  const configurePeer = useCallback(async (nextCall: HumanCall, nextLease: CallLease, local: MediaStream) => {
    closePeer();
    await closeRemoteMediaSessions(nextCall);
    const ice = await client.call("getCallIceServers");
    if (!isRecord(ice) || !Array.isArray(ice.iceServers)) throw new Error("Host returned malformed ICE credentials.");
    const iceServers = (ice as unknown as IceReply).iceServers;
    const publishSessionValue = await client.call("createCallMediaSession", {
      callId: nextCall.id,
      clientSessionId: callMediaNonce("publish_session"),
      generation: nextCall.generation,
      purpose: "publish",
    });
    const subscribeSessionValue = await client.call("createCallMediaSession", {
      callId: nextCall.id,
      clientSessionId: callMediaNonce("subscribe_session"),
      generation: nextCall.generation,
      purpose: "subscribe",
    });
    if (!isCallMediaSession(publishSessionValue) || publishSessionValue.status !== "ready" || publishSessionValue.purpose !== "publish") {
      throw new Error("Host returned a malformed SFU publish session.");
    }
    if (!isCallMediaSession(subscribeSessionValue) || subscribeSessionValue.status !== "ready" || subscribeSessionValue.purpose !== "subscribe") {
      throw new Error("Host returned a malformed SFU subscribe session.");
    }

    const publishPc = new RTCPeerConnection({ iceServers });
    const subscribePc = new RTCPeerConnection({ iceServers });
    pcRef.current = publishPc;
    subscribePcRef.current = subscribePc;
    const runtime: SfuMediaRuntime = {
      generation: nextCall.generation,
      publishSessionId: publishSessionValue.mediaSessionId,
      subscribeSessionId: subscribeSessionValue.mediaSessionId,
      publishedMids: new Set<string>(),
      subscribedPublicationIds: new Set<string>(),
      remoteVideoSubscriptions: new Map<string, SfuRemoteVideoSubscription>(),
    };
    sfuRuntimeRef.current = runtime;

    for (const track of local.getTracks()) publishPc.addTrack(track, local);
    const remote = new MediaStream();
    remoteStreamRef.current = remote;
    if (remoteVideoRef.current != null) remoteVideoRef.current.srcObject = remote;
    subscribePc.ontrack = (event) => {
      for (const track of event.streams[0]?.getTracks() ?? [event.track]) {
        if (!remote.getTracks().some((candidate) => candidate.id === track.id)) remote.addTrack(track);
      }
      if (remoteVideoRef.current != null) remoteVideoRef.current.srcObject = remote;
    };

    const reconnect = () => {
      if (recoveringRef.current || disposedRef.current) return;
      recoveringRef.current = true;
      void (async () => {
        const current = callRef.current;
        if (current == null) return;
        try {
          setStatus("reconnecting");
          const reconnecting = await transition(current, "reconnect");
          const resumed = await transition(reconnecting, "resume");
          const owner = await readLease(resumed.id, false);
          const stream = localStreamRef.current ?? await ensureLocalMedia(cameraEnabled);
          const restartPeer = configurePeerRef.current;
          if (restartPeer == null) throw new Error("Call media reconnect owner is unavailable.");
          await restartPeer(resumed, owner, stream);
        } catch (reconnectError) {
          await failCall(mediaFailure(reconnectError));
        } finally {
          recoveringRef.current = false;
        }
      })();
    };
    publishPc.onconnectionstatechange = () => {
      if (publishPc.connectionState === "connected") {
        setStatus("connected");
        const current = callRef.current;
        if (current?.state === "negotiating" || current?.state === "reconnecting") {
          void transition(current, "connected").catch((transitionError) => void failCall(mediaFailure(transitionError)));
        }
      } else if (publishPc.connectionState === "failed" || publishPc.connectionState === "disconnected") {
        reconnect();
      }
    };
    subscribePc.onconnectionstatechange = () => {
      if (subscribePc.connectionState === "failed" || subscribePc.connectionState === "disconnected") reconnect();
    };

    startDiagnostics(publishPc, nextCall.generation);
    setStatus(nextCall.state === "reconnecting" ? "reconnecting" : "negotiating");

    const offer = await publishPc.createOffer();
    await publishPc.setLocalDescription(offer);
    await waitForIceGatheringComplete(publishPc);
    const localOffer = publishPc.localDescription;
    if (localOffer?.type !== "offer" || typeof localOffer.sdp !== "string") {
      throw new Error("SFU publication offer was not available after ICE gathering.");
    }
    const publications = publishPc.getTransceivers().flatMap((transceiver, index) => {
      const track = transceiver.sender.track;
      const mid = transceiver.mid;
      if (track == null || mid == null) return [];
      return [{
        clientPublicationId: callMediaNonce("pub"),
        trackName: `fabushi_${track.kind}_${mid || index}`,
        mid,
        mediaKind: track.kind,
      }];
    });
    if (publications.length === 0) throw new Error("No local media tracks were available for SFU publication.");
    const publishOperationId = callMediaNonce("publish");
    const published = await client.call("publishCallMedia", {
      callId: nextCall.id,
      mediaSessionId: runtime.publishSessionId,
      clientOperationId: publishOperationId,
      generation: nextCall.generation,
      sessionDescription: { type: "offer", sdp: localOffer.sdp },
      publications,
    });
    if (!isRecord(published)) throw new Error("Host returned malformed SFU publication state.");
    const expectedPublications = new Map(
      publications.map((publication) => [publication.clientPublicationId, publication] as const),
    );
    const readyPublicationIds = new Set<string>();
    const seenPublicationIds = new Set<string>();
    let failedPublications = 0;
    if (!Array.isArray(published.results)) throw new Error("Host returned incomplete SFU publication results.");
    for (const raw of published.results) {
      if (!isRecord(raw) || typeof raw.clientPublicationId !== "string"
        || (raw.status !== "ready" && raw.status !== "failed")) {
        throw new Error("Host returned malformed SFU publication result.");
      }
      const expected = expectedPublications.get(raw.clientPublicationId);
      if (expected == null || seenPublicationIds.has(raw.clientPublicationId)) {
        throw new Error("Host returned mismatched SFU publication result identity.");
      }
      seenPublicationIds.add(raw.clientPublicationId);
      if (raw.status === "ready") {
        if (typeof raw.publicationId !== "string" || raw.mid !== expected.mid) {
          throw new Error("Host returned SFU publication with mismatched durable identity.");
        }
        readyPublicationIds.add(raw.clientPublicationId);
      } else {
        failedPublications += 1;
      }
    }
    if (seenPublicationIds.size !== publications.length || readyPublicationIds.size === 0) {
      throw new Error("Host returned incomplete SFU publication results.");
    }
    await publishPc.setRemoteDescription(requireSessionDescription(published.sessionDescription, "answer"));
    await client.call("acknowledgeCallMediaPublication", {
      callId: nextCall.id,
      mediaSessionId: runtime.publishSessionId,
      clientOperationId: publishOperationId,
      generation: nextCall.generation,
    });
    for (const publication of publications) {
      if (readyPublicationIds.has(publication.clientPublicationId)) runtime.publishedMids.add(publication.mid);
    }
    if (failedPublications > 0) {
      setError(`Some local media could not be published (${failedPublications}/${publications.length}); the remaining tracks stay connected.`);
    }
    await syncSfuSubscriptions(nextCall);
    return publishPc;
  }, [cameraEnabled, client, closePeer, closeRemoteMediaSessions, ensureLocalMedia, failCall, readLease, startDiagnostics, syncSfuSubscriptions, transition]);
  configurePeerRef.current = configurePeer;

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
    } catch (acceptError) {
      await failCall(mediaFailure(acceptError));
    }
  }, [configurePeer, ensureLocalMedia, failCall, publish, readLease, transition, updateMediaState]);

  const decline = useCallback(async () => {
    stopCallTone(false, true);
    const current = callRef.current;
    if (current != null && current.state !== "ended" && current.state !== "failed") {
      await transition(current, "decline", "declined").catch(() => undefined);
    }
    closePeer();
    invalidateCameraCapture();
    stopStream(localStreamRef.current);
    localStreamRef.current = null;
    stopStream(screenStreamRef.current);
    screenStreamRef.current = null;
    setScreenSharing(false);
    setScreenCapturePaused(false);
    setCameraEnabled(false);
    setCameraCapturePaused(false);
    setStatus("idle");
    publish(null, null);
  }, [closePeer, invalidateCameraCapture, publish, stopCallTone, transition]);

  const hangup = useCallback(async () => {
    stopCallTone(false, true);
    const current = callRef.current;
    if (current != null && current.state !== "ended" && current.state !== "failed") {
      await transition(current, "hangup", "user-ended").catch(() => undefined);
    }
    closePeer();
    invalidateCameraCapture();
    stopStream(localStreamRef.current);
    localStreamRef.current = null;
    stopStream(screenStreamRef.current);
    screenStreamRef.current = null;
    setStatus("idle");
    setScreenSharing(false);
    setScreenCapturePaused(false);
    setCameraCapturePaused(false);
    setCameraEnabled(false);
    publish(null, null);
  }, [closePeer, invalidateCameraCapture, publish, stopCallTone, transition]);

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
    const commandLease = callCommandFence.acquire("video-media");
    if (commandLease == null) return;
    setCameraPending(true);
    try {
      const stream = localStreamRef.current;
      const current = callRef.current;
      if (stream == null || current == null) return;
      const existing = stream.getVideoTracks()[0];
      if (existing != null && existing.readyState === "live") {
        const previousEnabled = existing.enabled;
        const nextEnabled = !cameraEnabled;
        existing.enabled = nextEnabled;
        setCameraEnabled(nextEnabled);
        setCameraCapturePaused(existing.muted);
        try {
          await updateMediaState(current, { video: nextEnabled, videoPaused: existing.muted, videoActive: nextEnabled && !existing.muted && existing.readyState === "live" });
        } catch (error) {
          existing.enabled = previousEnabled;
          setCameraEnabled(cameraEnabled);
          throw error;
        }
        return;
      }
      if (existing != null) {
        invalidateCameraCapture();
        stream.removeTrack(existing);
      }
      const permission = await bridge.callMedia.requestPermissions({ audio: false, video: true });
      if (permission.camera === "denied") throw new Error("Camera permission was denied.");
      const camera = await getPreferredUserMedia(bridge, { audio: false, video: true });
      const track = camera.getVideoTracks()[0];
      if (track == null) {
        stopStream(camera);
        throw new Error("Camera did not provide a video track.");
      }
      stream.addTrack(track);
      bindCameraTrackLifecycle(stream, track);
      setCameraCapturePaused(track.muted);
      const pc = pcRef.current;
      const addedSender = pc?.addTrack(track, stream) ?? null;
      setCameraEnabled(true);
      if (localVideoRef.current != null) localVideoRef.current.srcObject = stream;
      try {
        await updateMediaState(current, { video: true, videoPaused: track.muted, videoActive: !track.muted && track.readyState === "live" });
      } catch (error) {
        stream.removeTrack(track);
        track.stop();
        if (addedSender != null && pcRef.current === pc) pc?.removeTrack(addedSender);
        setCameraEnabled(false);
        setCameraCapturePaused(false);
        if (localVideoRef.current != null) localVideoRef.current.srcObject = stream;
        throw error;
      }
    } finally {
      commandLease.release();
      setCameraPending(false);
    }
  }, [bindCameraTrackLifecycle, bridge.callMedia, callCommandFence, cameraEnabled, invalidateCameraCapture, updateMediaState]);

  const closeDisplayPicker = useCallback(() => {
    displayPickerGenerationRef.current += 1;
    setDisplayPickerOpen(false);
    setDisplaySources([]);
  }, []);

  const openDisplayPicker = useCallback(async () => {
    const generation = ++displayPickerGenerationRef.current;
    setScreenPending(true);
    setError(null);
    try {
      const sources = await bridge.callMedia.listDisplaySources();
      if (disposedRef.current || generation !== displayPickerGenerationRef.current) return;
      if (sources.length === 0) throw new Error("No display source is available.");
      setDisplaySources(sources);
      setDisplayPickerOpen(true);
    } finally {
      if (generation === displayPickerGenerationRef.current) setScreenPending(false);
    }
  }, [bridge.callMedia]);

  const stopScreenShare = useCallback(async () => {
    const commandLease = callCommandFence.acquire("video-media");
    if (commandLease == null) return;
    setScreenPending(true);
    try {
      const current = callRef.current;
      const pc = pcRef.current;
      if (current == null || pc == null) return;
      const capture = screenStreamRef.current;
      const screenTrack = capture?.getVideoTracks()[0] ?? null;
      const sender = pc.getSenders().find((candidate) => candidate.track?.kind === "video");
      const camera = localStreamRef.current?.getVideoTracks()[0] ?? null;
      if (sender != null) await sender.replaceTrack(camera);
      try {
        await updateMediaState(current, { screenShare: false, displaySourceId: null, screenSharePaused: false, screenShareActive: false });
      } catch (error) {
        if (sender != null && screenTrack != null) await sender.replaceTrack(screenTrack).catch(() => undefined);
        throw error;
      }
      screenStreamRef.current = null;
      setScreenSharing(false);
      setScreenCapturePaused(false);
      stopStream(capture);
    } finally {
      commandLease.release();
      setScreenPending(false);
    }
  }, [callCommandFence, updateMediaState]);

  const selectDisplaySource = useCallback(async (sourceId: string) => {
    const commandLease = callCommandFence.acquire("video-media");
    if (commandLease == null) return;
    setScreenPending(true);
    try {
      const current = callRef.current;
      const pc = pcRef.current;
      if (current == null || pc == null) return;
      const refreshed = await bridge.callMedia.listDisplaySources();
      const source = refreshed.find((candidate) => candidate.id === sourceId);
      if (source == null) {
        setDisplaySources(refreshed);
        throw new Error("That display source is no longer available. Choose another source.");
      }
      const capture = await navigator.mediaDevices.getUserMedia({ audio: false, video: desktopSourceConstraint(source.id) });
      const track = capture.getVideoTracks()[0];
      if (track == null) {
        stopStream(capture);
        throw new Error("Display source did not provide a video track.");
      }
      const sender = pc.getSenders().find((candidate) => candidate.track?.kind === "video");
      const previousCapture = screenStreamRef.current;
      const previousTrack = sender?.track ?? null;
      const addedSender = sender == null ? pc.addTrack(track, capture) : null;
      if (sender != null) await sender.replaceTrack(track);
      try {
        await updateMediaState(current, { screenShare: true, displaySourceId: source.id, screenSharePaused: false, screenShareActive: true });
      } catch (error) {
        if (sender != null) await sender.replaceTrack(previousTrack).catch(() => undefined);
        if (addedSender != null && pcRef.current === pc) pc.removeTrack(addedSender);
        stopStream(capture);
        throw error;
      }
      screenStreamRef.current = capture;
      setScreenSharing(true);
      setScreenCapturePaused(false);
      closeDisplayPicker();
      stopStream(previousCapture);
      const projectCaptureActivity = (paused: boolean) => {
        if (disposedRef.current || screenStreamRef.current !== capture) return;
        setScreenCapturePaused(paused);
        const active = callRef.current;
        if (active == null) return;
        void updateMediaState(active, {
          screenShare: true,
          displaySourceId: source.id,
          screenSharePaused: paused,
          screenShareActive: !paused,
        }).catch((activityError) => setError(mediaFailure(activityError)));
      };
      track.onmute = () => projectCaptureActivity(true);
      track.onunmute = () => projectCaptureActivity(false);
      track.onended = () => {
        if (disposedRef.current || screenStreamRef.current !== capture) return;
        const endedLease = callCommandFence.acquire("video-media");
        if (endedLease == null) return;
        void (async () => {
          try {
            screenStreamRef.current = null;
            setScreenSharing(false);
            setScreenCapturePaused(false);
            const camera = localStreamRef.current?.getVideoTracks()[0] ?? null;
            const currentSender = pcRef.current?.getSenders().find((candidate) => candidate.track?.kind === "video");
            if (currentSender != null) await currentSender.replaceTrack(camera);
            const active = callRef.current;
            if (active != null) await updateMediaState(active, { screenShare: false, displaySourceId: null, screenSharePaused: false, screenShareActive: false });
          } catch (endedError) {
            setError(mediaFailure(endedError));
          } finally {
            endedLease.release();
          }
        })();
      };
    } finally {
      commandLease.release();
      setScreenPending(false);
    }
  }, [bridge.callMedia, callCommandFence, closeDisplayPicker, updateMediaState]);

  const toggleScreen = useCallback(async () => {
    if (screenSharing) await stopScreenShare();
    else await openDisplayPicker();
  }, [openDisplayPicker, screenSharing, stopScreenShare]);

  useEffect(() => () => {
    callCommandFence.dispose();
  }, [callCommandFence]);

  useEffect(() => {
    void startCallToneFor(call, lease);
  }, [call, lease, startCallToneFor]);

  useEffect(() => () => {
    callToneKeyRef.current = null;
    callToneFailedKeyRef.current = null;
    const player = callTonePlayerRef.current;
    callTonePlayerRef.current = null;
    if (player != null) void player.dispose();
  }, []);

  useEffect(() => {
    disposedRef.current = false;
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
        await syncSfuSubscriptions(next);
      })().catch((pollError) => setError(mediaFailure(pollError)));
    }, 1000);
    return () => {
      disposedRef.current = true;
      displayPickerGenerationRef.current += 1;
      setDisplayPickerOpen(false);
      window.clearInterval(timer);
      stopCallTone(false, true);
      closePeer();
      invalidateCameraCapture();
      stopStream(localStreamRef.current);
      localStreamRef.current = null;
      stopStream(screenStreamRef.current);
      screenStreamRef.current = null;
      publish(null, null);
    };
  }, [closePeer, configurePeer, conversation.id, invalidateCameraCapture, publish, readLease, refresh, stopCallTone, syncSfuSubscriptions]);

  const incoming = call != null && lease?.role === "peer" && (call.state === "invited" || call.state === "ringing");
  const active = call != null && call.state !== "ended" && call.state !== "failed";

  return <div className="sand-human-call" role="group" aria-label="Human call controls" data-call-tone-state={callToneState} data-call-tone-sample-rate={CALL_TONE_SAMPLE_RATE} data-call-tone-loop-count={CALL_TONE_LOOP_COUNT} data-call-tone-offset-samples={CALL_TONE_INITIAL_OFFSET_SAMPLES}>
    {!active ? <>
      <SandButton aria-label="Start voice call" onClick={() => void start(false)} size="sm" variant="secondary">Voice</SandButton>
      <SandButton aria-label="Start video call" onClick={() => void start(true)} size="sm" variant="secondary">Video</SandButton>
    </> : null}
    {incoming ? <>
      <span role="status">Incoming call</span>
      {callToneState === "failed" ? <SandButton aria-label="Enable call sound" onClick={() => { setError(null); void startCallToneFor(call, lease, true); }} size="sm" variant="secondary">Enable sound</SandButton> : null}
      <SandButton aria-label="Accept voice call" onClick={() => void accept(false)} size="sm">Accept</SandButton>
      <SandButton aria-label="Accept video call" onClick={() => void accept(true)} size="sm">Accept video</SandButton>
      <SandButton aria-label="Decline call" onClick={() => void decline()} sentiment="danger" size="sm" variant="secondary">Decline</SandButton>
    </> : null}
    {active && !incoming ? <>
      <span role="status">{status === "idle" ? call.state : status}</span>
      {callToneState === "failed" && (call.state === "invited" || call.state === "ringing") ? <SandButton aria-label="Enable call sound" onClick={() => { setError(null); void startCallToneFor(call, lease, true); }} size="sm" variant="secondary">Enable sound</SandButton> : null}
      <SandButton aria-pressed={muted} onClick={() => void toggleMute().catch((toggleError) => setError(mediaFailure(toggleError)))} pending={mutePending} size="sm" variant="secondary">{muted ? "Unmute" : "Mute"}</SandButton>
      {cameraEnabled && cameraCapturePaused ? <span role="status">Camera paused</span> : null}
      <SandButton aria-pressed={cameraEnabled} onClick={() => void toggleCamera().catch((toggleError) => setError(mediaFailure(toggleError)))} pending={cameraPending} size="sm" variant="secondary">{cameraEnabled ? "Camera off" : "Camera on"}</SandButton>
      <SandButton aria-pressed={screenSharing} onClick={() => void toggleScreen().catch((toggleError) => setError(mediaFailure(toggleError)))} pending={screenPending} size="sm" variant="secondary">{screenSharing ? "Stop sharing" : "Share screen"}</SandButton>
      {remoteVideoLayer != null && remoteVideoLayer.available.length > 0 ? <SandButton aria-label="Switch remote video quality" onClick={() => void cycleRemoteVideoLayer().catch((layerError) => setError(mediaFailure(layerError)))} pending={layerPending} size="sm" variant="secondary">Video quality{remoteVideoLayer.current == null ? "" : ` · ${remoteVideoLayer.current}`}</SandButton> : null}
      {screenSharing && screenCapturePaused ? <span role="status">Screen sharing paused</span> : null}
      {screenSharing ? <SandButton aria-label="Choose another display source" onClick={() => void openDisplayPicker().catch((pickerError) => setError(mediaFailure(pickerError)))} pending={screenPending} size="sm" variant="secondary">Change screen</SandButton> : null}
      <SandButton aria-label="End call" onClick={() => void hangup()} sentiment="danger" size="sm">End</SandButton>
      {diagnostics == null ? null : <details aria-label="Call diagnostics">
        <summary>Diagnostics</summary>
        <dl>
          <dt>Codec</dt><dd>{diagnostics.codec ?? "Unknown"}</dd>
          <dt>Path</dt><dd>{diagnostics.path ?? "Unknown"}</dd>
          <dt>RTT</dt><dd>{diagnostics.roundTripTimeMs == null ? "Unknown" : `${diagnostics.roundTripTimeMs} ms`}</dd>
          <dt>Send bitrate</dt><dd>{diagnostics.sendBitrateKbps == null ? "Sampling" : `${diagnostics.sendBitrateKbps} kbps`}</dd>
          <dt>Receive bitrate</dt><dd>{diagnostics.receiveBitrateKbps == null ? "Sampling" : `${diagnostics.receiveBitrateKbps} kbps`}</dd>
          <dt>Packet loss</dt><dd>{diagnostics.packetLossPercent == null ? "Unknown" : `${diagnostics.packetLossPercent}%`}</dd>
          <dt>Jitter</dt><dd>{diagnostics.jitterMs == null ? "Unknown" : `${diagnostics.jitterMs} ms`}</dd>
        </dl>
      </details>}
    </> : null}
    <video aria-label="Local call preview" autoPlay muted playsInline ref={localVideoRef} hidden={!cameraEnabled && !screenSharing} />
    <video aria-label="Remote call video" autoPlay playsInline ref={remoteVideoRef} hidden={!active} />
    {error == null ? null : <span role="alert">{error}</span>}
    <OverlayDialog label="Choose a display source" onClose={closeDisplayPicker} open={displayPickerOpen} panelStyle={{ width: "min(520px, calc(100vw - 32px))", padding: 16, background: "var(--fabushi-surface-elevated, Canvas)", border: "1px solid var(--fabushi-border-subtle, ButtonBorder)", borderRadius: 12 }}>
      <h2>Choose what to share</h2>
      <div aria-label="Display sources" role="menu">
        {displaySources.map((source) => <SandMenuItem key={source.id} onSelect={() => void selectDisplaySource(source.id).catch((pickerError) => setError(mediaFailure(pickerError)))}>{source.name}</SandMenuItem>)}
      </div>
      <SandButton onClick={closeDisplayPicker} size="sm" variant="secondary">Cancel</SandButton>
    </OverlayDialog>
  </div>;
}
