import { useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import type { Root } from "react-dom/client";
import { flushSync } from "react-dom";
import { HumanCallControls } from "./human-call-media";
import type { ProductionCoordinatorClient } from "./coordinator-client";
import type { RendererAgent } from "./model";
import type { DesktopBridge, DesktopCallMediaBridge } from "../recovered/contracts/desktop-bridge";

type RequestKind = "permission" | "preferences" | "sources" | "user" | "display" | "replace" | "coordinator";
type RequestRecord = {
  id: string;
  kind: RequestKind;
  args: unknown;
  method?: string;
  client: string;
  settled: boolean;
};
type Deferred = {
  resolve: (value: unknown) => void;
  reject: (error: unknown) => void;
};
type TrackRecord = {
  track: MediaStreamTrack;
  label: string;
  stopCalls: number;
  nativeStop: () => void;
};
type CallRecord = {
  id: string;
  scopeId: string;
  creatorId: string;
  state: string;
  generation: number;
  signalSeq: number;
  participantIds: string[];
  mediaCapabilities: Record<string, unknown>;
  deviceSelection: Record<string, unknown>;
  updatedAtMs: number;
};

const requests: RequestRecord[] = [];
const pending = new Map<string, Deferred>();
const holds = new Map<string, number>();
const tracks = new Map<string, TrackRecord>();
const streams = new Map<string, { stream: MediaStream; label: string }>();
const savedEnded = new Map<string, MediaStreamTrack["onended"]>();
const audioContexts: AudioContext[] = [];
const audioDestinations: MediaStreamAudioDestinationNode[] = [];
const canvases: HTMLCanvasElement[] = [];
const coordinatorCalls = new Map<string, CallRecord>();
const peers: FixturePeer[] = [];
const peerEvents: Array<Record<string, unknown>> = [];
const timers = new Map<number, () => void>();
let nextRequest = 0;
let nextPeer = 0;
let nextSender = 0;
let nextTimer = 10_000;
let nextCall = 0;
let nextClient = 0;
let currentClient = "client-0";
let currentScope = "scope-a";
let fixtureRoot: Root | null = null;
let changeScope: ((scope: string) => void) | null = null;
let changeClient: (() => void) | null = null;
let rerender: (() => void) | null = null;

function key(kind: RequestKind, method?: string): string {
  return method == null ? kind : `${kind}:${method}`;
}

function controlled<T>(kind: RequestKind, args: unknown, value: () => T, method?: string, client = currentClient): Promise<T> {
  const id = `request-${++nextRequest}`;
  const record: RequestRecord = { id, kind, args, method, client, settled: false };
  requests.push(record);
  const requestKey = key(kind, method);
  const remaining = holds.get(requestKey) ?? 0;
  if (remaining > 0) holds.set(requestKey, remaining - 1);
  if (remaining > 0 || kind === "user" || kind === "display") {
    return new Promise<T>((resolve, reject) => {
      pending.set(id, {
        resolve: (replacement) => {
          record.settled = true;
          resolve((replacement === undefined ? value() : replacement) as T);
        },
        reject: (error) => { record.settled = true; reject(error); },
      });
    });
  }
  record.settled = true;
  return Promise.resolve(value());
}

async function flush(): Promise<void> {
  await Promise.resolve();
  await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
  await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
}

function clone<T>(value: T): T {
  return structuredClone(value);
}

function trackRecord(track: MediaStreamTrack, label: string): TrackRecord {
  const record: TrackRecord = { track, label, stopCalls: 0, nativeStop: track.stop.bind(track) };
  track.stop = () => {
    record.stopCalls += 1;
    record.nativeStop();
  };
  tracks.set(track.id, record);
  return record;
}

function makeStream(input: { label: string; audio?: number; video?: number }): MediaStream {
  const result = new MediaStream();
  for (let index = 0; index < (input.audio ?? 0); index++) {
    const context = new AudioContext();
    audioContexts.push(context);
    const destination = context.createMediaStreamDestination();
    audioDestinations.push(destination);
    const track = destination.stream.getAudioTracks()[0];
    if (track == null) throw new Error("Synthetic browser audio track is unavailable");
    trackRecord(track, `${input.label}:audio:${index}`);
    result.addTrack(track);
  }
  for (let index = 0; index < (input.video ?? 0); index++) {
    const canvas = document.createElement("canvas");
    canvas.width = 2;
    canvas.height = 2;
    canvas.getContext("2d")!.fillRect(0, 0, 2, 2);
    canvases.push(canvas);
    const track = canvas.captureStream(0).getVideoTracks()[0];
    if (track == null) throw new Error("Synthetic browser video track is unavailable");
    trackRecord(track, `${input.label}:video:${index}`);
    result.addTrack(track);
  }
  streams.set(result.id, { stream: result, label: input.label });
  return result;
}

function trackSnapshot(track: MediaStreamTrack | null) {
  if (track == null) return null;
  const record = tracks.get(track.id);
  return {
    id: track.id, label: record?.label ?? track.label, kind: track.kind,
    enabled: track.enabled, readyState: track.readyState, stopCalls: record?.stopCalls ?? 0,
  };
}

class FixtureSender {
  readonly id = `sender-${++nextSender}`;
  track: MediaStreamTrack | null;

  constructor(readonly peer: FixturePeer, track: MediaStreamTrack) {
    this.track = track;
  }

  async replaceTrack(track: MediaStreamTrack | null): Promise<void> {
    peerEvents.push({ kind: "replace-dispatch", peerId: this.peer.id, senderId: this.id, trackId: track?.id ?? null, closed: this.peer.closed });
    await controlled("replace", { peerId: this.peer.id, senderId: this.id, trackId: track?.id ?? null }, () => undefined);
    // Completion can be held beyond close. Tests distinguish an already-dispatched
    // replacement from a fresh production attachment attempt after cancellation.
    this.track = track;
    peerEvents.push({ kind: "replace-complete", peerId: this.peer.id, senderId: this.id, trackId: track?.id ?? null, closed: this.peer.closed });
  }
}

class FixturePeer {
  readonly id = `peer-${++nextPeer}`;
  readonly senders: FixtureSender[] = [];
  closed = false;
  connectionState: RTCPeerConnectionState = "new";
  signalingState: RTCSignalingState = "stable";
  onicecandidate: ((event: RTCPeerConnectionIceEvent) => void) | null = null;
  ontrack: ((event: RTCTrackEvent) => void) | null = null;
  onconnectionstatechange: (() => void) | null = null;

  constructor(readonly configuration: RTCConfiguration) {
    peers.push(this);
    peerEvents.push({ kind: "construct", peerId: this.id });
  }

  addTrack(track: MediaStreamTrack, ..._streams: MediaStream[]): FixtureSender {
    peerEvents.push({ kind: "addTrack", peerId: this.id, trackId: track.id, closed: this.closed });
    if (this.closed) throw new DOMException("Fixture peer is closed", "InvalidStateError");
    const sender = new FixtureSender(this, track);
    this.senders.push(sender);
    return sender;
  }

  removeTrack(sender: FixtureSender): void {
    peerEvents.push({ kind: "removeTrack", peerId: this.id, senderId: sender.id, closed: this.closed });
    sender.track = null;
  }

  getSenders(): FixtureSender[] {
    return [...this.senders];
  }

  async createOffer(): Promise<RTCSessionDescriptionInit> {
    return { type: "offer", sdp: "fixture-offer" };
  }

  async createAnswer(): Promise<RTCSessionDescriptionInit> {
    return { type: "answer", sdp: "fixture-answer" };
  }

  async setLocalDescription(description: RTCSessionDescriptionInit): Promise<void> {
    this.signalingState = description.type === "offer" ? "have-local-offer" : "stable";
  }

  async setRemoteDescription(_description: RTCSessionDescriptionInit): Promise<void> {
    this.signalingState = "stable";
  }

  async addIceCandidate(_candidate: RTCIceCandidateInit): Promise<void> {}

  close(): void {
    this.closed = true;
    this.connectionState = "closed";
    this.signalingState = "closed";
    peerEvents.push({ kind: "close", peerId: this.id });
  }
}

Object.defineProperty(window, "RTCPeerConnection", { configurable: true, value: FixturePeer });
Object.defineProperty(navigator.mediaDevices, "getUserMedia", {
  configurable: true,
  value: (constraints: MediaStreamConstraints) => {
    const video = constraints.video as (MediaTrackConstraints & { mandatory?: { chromeMediaSource?: string } }) | undefined;
    const kind = typeof video === "object" && video?.mandatory?.chromeMediaSource === "desktop" ? "display" : "user";
    return controlled<MediaStream>(kind, clone(constraints), () => new MediaStream());
  },
});

// Drive the component's real polling callback explicitly; no wall-clock races.
const nativeSetInterval = window.setInterval.bind(window);
const nativeClearInterval = window.clearInterval.bind(window);
Object.defineProperty(window, "setInterval", {
  configurable: true,
  value: (handler: TimerHandler, timeout?: number, ...args: unknown[]) => {
    if (timeout === 1000 && typeof handler === "function") {
      const id = ++nextTimer;
      timers.set(id, () => handler(...args));
      return id;
    }
    return nativeSetInterval(handler, timeout, ...args);
  },
});
Object.defineProperty(window, "clearInterval", {
  configurable: true,
  value: (id?: number) => {
    if (id != null && timers.delete(id)) return;
    nativeClearInterval(id);
  },
});

function incoming(scope = currentScope): CallRecord {
  for (const call of coordinatorCalls.values()) {
    if (call.scopeId === scope) call.state = "ended";
  }
  const call: CallRecord = {
    id: `incoming-${++nextCall}`, scopeId: scope, creatorId: "peer-human", state: "ringing",
    generation: 0, signalSeq: 0, participantIds: ["local-human", "peer-human"],
    mediaCapabilities: {}, deviceSelection: {}, updatedAtMs: nextCall,
  };
  coordinatorCalls.set(call.id, call);
  return clone(call);
}

function coordinatorReply(method: string, raw: unknown): unknown {
  const args = (raw ?? {}) as Record<string, unknown>;
  if (method === "syncHumanCalls") return [...coordinatorCalls.values()].map(clone);
  if (method === "listCallSessions") {
    return [...coordinatorCalls.values()].filter((call) => call.scopeId === args.scopeId).map(clone);
  }
  if (method === "getCallIceServers") return { iceServers: [{ urls: "stun:fixture.invalid:3478" }], ttlSeconds: 3600 };
  if (method === "createCallSession") {
    const call: CallRecord = {
      id: `outgoing-${++nextCall}`, scopeId: String(args.scopeId), creatorId: "local-human", state: "invited",
      generation: 0, signalSeq: 0, participantIds: ["local-human", "peer-human"],
      mediaCapabilities: {}, deviceSelection: {}, updatedAtMs: nextCall,
    };
    coordinatorCalls.set(call.id, call);
    return clone(call);
  }
  const call = coordinatorCalls.get(String(args.callId));
  if (call == null) throw new Error(`Fixture received ${method} for an unknown call`);
  if (method === "getCallTransportLease") {
    return {
      userId: "local-human", deviceId: "local-device", role: call.creatorId === "local-human" ? "creator" : "peer",
      claimedDeviceId: "local-device", isOwner: true, claimAvailable: false,
      state: call.state, generation: call.generation, eventSeq: call.signalSeq,
    };
  }
  if (method === "transitionCallSession") {
    // Mirror only the pinned Host generation contract needed by reconnect cases.
    // Capture ownership and stale-operation decisions remain in HumanCallControls.
    if (args.action === "reconnect") {
      if (!["negotiating", "connected", "reconnecting"].includes(call.state)) {
        throw new Error("Fixture reconnect requires an active negotiated call");
      }
      call.generation += 1;
      call.signalSeq = 0;
    } else if (args.action === "resume" && call.state !== "reconnecting") {
      throw new Error("Fixture resume requires reconnecting state");
    }
    const responseStates: Record<string, string> = {
      ring: "ringing", accept: "negotiating", connected: "connected", reconnect: "reconnecting",
      resume: "negotiating", hangup: "ended", decline: "ended", fail: "failed",
    };
    const state = responseStates[String(args.action)];
    if (state == null) throw new Error(`Fixture has no response for action ${String(args.action)}`);
    call.state = state;
    return clone(call);
  }
  if (method === "updateCallMedia") {
    call.mediaCapabilities = clone(args.mediaCapabilities as Record<string, unknown>);
    call.deviceSelection = clone(args.deviceSelection as Record<string, unknown>);
    return clone(call);
  }
  if (method === "listCallSignals") return [];
  if (method === "sendCallSignal") {
    call.signalSeq += 1;
    return { callId: call.id, generation: call.generation, seq: call.signalSeq, senderDeviceId: "local-device", kind: args.kind, payload: args.payload };
  }
  throw new Error(`Unexpected call-media Coordinator method: ${method}`);
}

function clientFor(id: string): ProductionCoordinatorClient {
  // Only the existing call surface is substituted. The real component owns every
  // acquisition, attachment, cancellation and release decision under test.
  return {
    ready: Promise.resolve(),
    call: (method: string, args?: unknown) => controlled("coordinator", clone(args ?? {}), () => coordinatorReply(method, args), method, id),
  } as unknown as ProductionCoordinatorClient;
}

const callMedia: DesktopCallMediaBridge = {
  getPreferences: () => controlled("preferences", {}, () => ({ microphoneId: null, cameraId: null })),
  setPreferences: async (preferences) => preferences,
  requestPermissions: (input) => controlled("permission", clone(input), () => ({
    microphone: input.audio ? "granted" as const : "not-requested" as const,
    camera: input.video ? "granted" as const : "not-requested" as const,
  })),
  listDisplaySources: () => controlled("sources", {}, () => [{ id: "display-fixture", name: "Fixture display" }]),
};
const bridge = { callMedia } as unknown as DesktopBridge;

function Fixture() {
  const [scope, setScope] = useState(currentScope);
  const [client, setClient] = useState(() => clientFor(currentClient));
  const [revision, setRevision] = useState(0);
  changeScope = (next) => { currentScope = next; flushSync(() => setScope(next)); };
  changeClient = () => {
    currentClient = `client-${++nextClient}`;
    incoming(currentScope);
    flushSync(() => setClient(clientFor(currentClient)));
  };
  rerender = () => flushSync(() => setRevision((value) => value + 1));
  const conversation = useMemo(() => ({
    id: scope, name: "Call fixture", memberIds: ["local-human", "peer-human"],
    conversationPartnerIds: ["peer-human"], isGroup: false, isHidden: false,
    lastEntry: null, awaitingUserResponse: null, raw: {},
  } as RendererAgent), [scope]);
  return <div data-scope={scope} data-revision={revision}>
    <HumanCallControls bridge={bridge} client={client} conversation={conversation} />
  </div>;
}

function mount(): void {
  if (fixtureRoot != null) throw new Error("Fixture is already mounted");
  fixtureRoot = createRoot(document.getElementById("root")!);
  flushSync(() => fixtureRoot!.render(<Fixture />));
}

function unmount(): void {
  fixtureRoot?.unmount();
  fixtureRoot = null;
  changeScope = null;
  changeClient = null;
  rerender = null;
}

function snapshot() {
  const preview = (label: string) => {
    const node = document.querySelector<HTMLVideoElement>(`video[aria-label="${label}"]`);
    const stream = node?.srcObject;
    return stream instanceof MediaStream ? { id: stream.id, trackIds: stream.getTracks().map((track) => track.id) } : null;
  };
  return {
    scope: currentScope, client: currentClient,
    tracks: [...tracks.values()].map((record) => trackSnapshot(record.track)),
    streams: [...streams.values()].map(({ stream, label }) => ({ id: stream.id, label, trackIds: stream.getTracks().map((track) => track.id) })),
    peers: peers.map((peer) => ({ id: peer.id, closed: peer.closed, senders: peer.senders.map((sender) => ({ id: sender.id, trackId: sender.track?.id ?? null })) })),
    peerEvents: clone(peerEvents), requests: clone(requests), pending: [...pending.keys()],
    localPreview: preview("Local call preview"), remotePreview: preview("Remote call video"),
    calls: [...coordinatorCalls.values()].map(clone), pollingOwners: timers.size,
  };
}

Object.assign(window, {
  __callMediaHarness: {
    snapshot,
    hold: (kind: RequestKind, method?: string) => {
      holds.set(key(kind, method), (holds.get(key(kind, method)) ?? 0) + 1);
    },
    resolve: async (id: string, value?: unknown) => {
      const item = pending.get(id);
      if (item == null) throw new Error(`Unknown pending request: ${id}`);
      pending.delete(id);
      item.resolve(value);
      await flush();
    },
    resolveCapture: async (id: string, input: { label: string; audio?: number; video?: number }) => {
      const item = pending.get(id);
      if (item == null) throw new Error(`Unknown pending capture: ${id}`);
      const stream = makeStream(input);
      pending.delete(id);
      item.resolve(stream);
      await flush();
      return stream.id;
    },
    reject: async (id: string, message: string, name?: string) => {
      const item = pending.get(id);
      if (item == null) throw new Error(`Unknown pending request: ${id}`);
      pending.delete(id);
      item.reject(name == null ? new Error(message) : new DOMException(message, name));
      await flush();
    },
    saveEnded: (id: string) => {
      const record = tracks.get(id);
      if (record == null) throw new Error(`Unknown track: ${id}`);
      if (record.track.onended == null) throw new Error("Production did not install the expected display-ended handler");
      savedEnded.set(id, record.track.onended);
    },
    fireSavedEnded: async (id: string) => {
      const record = tracks.get(id);
      if (record == null) throw new Error(`Unknown track: ${id}`);
      savedEnded.get(id)?.call(record.track, new Event("ended"));
      await flush();
    },
    endTrack: async (id: string) => {
      const record = tracks.get(id);
      if (record == null) throw new Error(`Unknown track: ${id}`);
      record.nativeStop();
      record.track.dispatchEvent(new Event("ended"));
      await flush();
    },
    installIncoming: (scope = currentScope) => incoming(scope),
    remoteTerminal: (id: string) => {
      const call = coordinatorCalls.get(id);
      if (call == null) throw new Error(`Unknown call: ${id}`);
      call.state = "ended";
    },
    remoteNegotiating: (id: string) => {
      const call = coordinatorCalls.get(id);
      if (call == null) throw new Error(`Unknown call: ${id}`);
      call.state = "negotiating";
    },
    advanceRemoteGeneration: (id: string) => {
      const call = coordinatorCalls.get(id);
      if (call == null) throw new Error(`Unknown call: ${id}`);
      call.generation += 1;
      call.signalSeq = 0;
      call.state = "negotiating";
      return clone(call);
    },
    setPeerConnectionState: async (id: string, state: RTCPeerConnectionState) => {
      const peer = peers.find((candidate) => candidate.id === id);
      if (peer == null || peer.closed) throw new Error(`Unknown or closed peer: ${id}`);
      peer.connectionState = state;
      peerEvents.push({ kind: "connection-state", peerId: peer.id, state });
      peer.onconnectionstatechange?.();
      await flush();
    },
    poll: async () => {
      for (const callback of [...timers.values()]) callback();
      await flush();
    },
    rerender: async () => { rerender?.(); await flush(); },
    switchScope: async (scope: string) => { changeScope?.(scope); await flush(); },
    replaceClient: async () => { changeClient?.(); await flush(); },
    unmount: async () => { unmount(); await flush(); },
    mount: async () => { mount(); await flush(); },
    flush,
    dispose: async () => {
      unmount();
      for (const record of tracks.values()) record.nativeStop();
      await Promise.all(audioContexts.map((context) => context.close()));
      audioDestinations.length = 0;
      canvases.length = 0;
    },
  },
});
incoming();
mount();
