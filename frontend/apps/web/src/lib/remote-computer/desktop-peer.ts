import type {
  ComputerAction,
  ComputerActionResult,
  ComputerControlTarget,
  ComputerSnapshot,
  RemoteComputerCapability,
  RemoteComputerClient,
  RemoteComputerPlatform,
  RemoteComputerProvider,
  RemoteComputerRegistration,
  RemoteComputerSession,
  RemoteComputerSignal,
  RuntimeCommand,
  RuntimeEvent,
} from "../mahayana-host/contracts";
import type { MahayanaHostTransport } from "../mahayana-host/transport";

const DEVICE_ID_STORAGE_KEY = "fabushi-remote-computer-device-id-v1";
const DEVICE_ID_SCOPE_STORAGE_PREFIX = "fabushi-remote-computer-device-id-v2";
const DEVICE_ID_MIGRATED_SCOPE_KEY = "fabushi-remote-computer-device-id-v1-scope";
const memoryDeviceIds = new Map<string, string>();
const CHANNEL_LABEL = "fabushi-computer-v1";
const SIGNAL_POLL_MS = 650;
const SESSION_POLL_MS = 1_500;
const HEARTBEAT_MS = 20_000;
const REQUEST_TIMEOUT_MS = 20_000;
const FRAME_CHUNK_CHARS = 24 * 1024;
const MAX_BUFFERED_BYTES = 384 * 1024;
const MAX_FRAME_PAYLOAD_CHARS = 32 * 1024 * 1024;
const MAX_CHANNEL_MESSAGE_CHARS = 256 * 1024;
const MAX_PENDING_OPERATIONS = 32;
const MAX_PENDING_REMOTE_CANDIDATES = 256;
const MAX_ACTION_CHAIN = 9;
const MAX_REMOTE_CLIENTS = 64;
const MAX_REMOTE_SESSIONS = 32;
const MAX_SIGNAL_BATCH = 128;
const MAX_SIGNAL_PAYLOAD_CHARS = 256 * 1024;
const MAX_ICE_SERVERS = 16;
const MAX_ICE_URLS_PER_SERVER = 16;
const MAX_ICE_URL_CHARS = 2_048;
const MAX_SDP_CHARS = 256 * 1024;
const MAX_ICE_CANDIDATE_CHARS = 16 * 1024;
const FABUSHI_WEBRTC_CAPABILITIES: RemoteComputerCapability[] = [
  "remote-desktop",
  "input",
  "display",
  "session-management",
];

function detectedRemotePlatform(): RemoteComputerPlatform {
  if (typeof navigator === "undefined") return "unknown";
  const fingerprint = `${navigator.userAgent} ${navigator.platform}`.toLowerCase();
  if (fingerprint.includes("android")) return "android";
  if (/iphone|ipad|ipod/.test(fingerprint)) return "ios";
  if (fingerprint.includes("win")) return "windows";
  if (fingerprint.includes("mac")) return "macos";
  if (fingerprint.includes("linux")) return "linux";
  return "web";
}

function detectedFabushiVersion(): string {
  const value = (globalThis as { __FABUSHI_APP_VERSION__?: unknown }).__FABUSHI_APP_VERSION__;
  return typeof value === "string" && /^[A-Za-z0-9.+_-]{1,64}$/.test(value) ? value : "unknown";
}

export interface RemoteComputerDesktopState {
  running: boolean;
  controlEnabled: boolean;
  deviceId: string;
  registration?: RemoteComputerRegistration;
  clients: RemoteComputerClient[];
  sessions: RemoteComputerSession[];
  pendingAuthorization?: RemoteComputerSession;
  activeSessionId?: string;
  activeClientId?: string;
  connectionState: RTCPeerConnectionState | "idle";
  channelOpen: boolean;
  error?: string;
}

type RemoteChangedEvent = Extract<RuntimeEvent, { type: "remoteComputer.changed" }>;
type ComputerSnapshotEvent = Extract<RuntimeEvent, { type: "computer.snapshot" }>;
type ComputerResultEvent = Extract<RuntimeEvent, { type: "computer.result" }>;
type RemoteRuntimeCommand = Extract<RuntimeCommand, { type:
  | "remoteComputer.register"
  | "remoteComputer.heartbeat"
  | "remoteComputer.clients"
  | "remoteComputer.clientRevoke"
  | "remoteComputer.sessions"
  | "remoteComputer.sessionActivate"
  | "remoteComputer.sessionClose"
  | "remoteComputer.signal"
  | "remoteComputer.signalDrain"
}>;

interface DesktopPeerOptions {
  transport: MahayanaHostTransport;
  label: string;
  identityScope: string;
  controlEnabled?: boolean;
  provider?: RemoteComputerProvider;
  platform?: RemoteComputerPlatform;
  appVersion?: string;
  capabilities?: RemoteComputerCapability[];
  resolveAgentId?: (requestedAgentId: string) => string | null;
  onState?: (state: RemoteComputerDesktopState) => void;
}

interface NormalizedSessionListPayload {
  sessions: RemoteComputerSession[];
  iceServers: RTCIceServer[];
}

interface NormalizedSignalDrainPayload {
  signals: RemoteComputerSignal[];
  lastSignalId: number;
}

interface PeerSession {
  session: RemoteComputerSession;
  peer: RTCPeerConnection;
  channel?: RTCDataChannel;
  signalCursor: number;
  signalTimer?: number;
  activated: boolean;
  closing: boolean;
  operationQueue: Promise<void>;
  pendingOperations: number;
  drainingSignals: boolean;
  pendingRemoteCandidates: RTCIceCandidateInit[];
}

type IncomingChannelMessage =
  | { type: "ping"; id?: string }
  | { type: "disconnect"; id?: string }
  | { type: "computer.snapshot.request"; id: string }
  | { type: "computer.action"; id: string; action: ComputerAction; then?: ComputerAction[] }
  | { type: "computer.ai.request"; requestId: string; prompt: string; agentId?: string };

function objectValue(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function validRelayIdentifier(value: unknown, maxLength = 200): value is string {
  return typeof value === "string"
    && value.length > 0
    && value.length <= maxLength
    && /^[A-Za-z0-9._:-]+$/.test(value);
}

function validRemoteDeviceId(value: unknown): value is string {
  return validRelayIdentifier(value, 128) && value.length >= 20;
}

function validDisplayText(value: unknown, maxLength = 80): value is string {
  return typeof value === "string"
    && value.trim().length > 0
    && value.length <= maxLength
    && !/[\u0000-\u001F\u007F]/.test(value);
}

function validTimestamp(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
}

function validGeneration(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value > 0;
}

function serializedLength(value: unknown): number {
  try {
    const serialized = JSON.stringify(value);
    if (typeof serialized !== "string") throw new Error("not serializable");
    return serialized.length;
  } catch {
    throw new Error("Remote computer payload is not serializable");
  }
}

function normalizeRegistration(value: unknown, expectedDeviceId: string): RemoteComputerRegistration {
  const data = objectValue(value);
  if (data.deviceId !== expectedDeviceId
    || !validDisplayText(data.label)
    || typeof data.pairingCode !== "string"
    || !/^[0-9A-F]{12}$/.test(data.pairingCode)
    || !validTimestamp(data.pairingExpiresAt)
    || data.pairingExpiresAt === 0) {
    throw new Error("Remote computer registration response is invalid");
  }
  return {
    deviceId: expectedDeviceId,
    label: data.label,
    pairingCode: data.pairingCode,
    pairingExpiresAt: data.pairingExpiresAt,
  };
}

function normalizeClientsPayload(value: unknown, expectedDeviceId: string): RemoteComputerClient[] {
  const data = objectValue(value);
  if (data.deviceId !== expectedDeviceId || !Array.isArray(data.clients) || data.clients.length > MAX_REMOTE_CLIENTS) {
    throw new Error("Remote computer client response is invalid");
  }
  const seen = new Set<string>();
  return data.clients.map((value) => {
    const client = objectValue(value);
    if (!validRelayIdentifier(client.clientId, 160)
      || seen.has(client.clientId)
      || !validDisplayText(client.label)
      || !validTimestamp(client.pairedAt)
      || !validTimestamp(client.lastSeenAt)) {
      throw new Error("Remote computer client response contains an invalid client");
    }
    seen.add(client.clientId);
    return {
      clientId: client.clientId,
      label: client.label,
      pairedAt: client.pairedAt,
      lastSeenAt: client.lastSeenAt,
    };
  });
}

function normalizeIceServers(value: unknown): RTCIceServer[] {
  if (value === undefined) return [];
  if (!Array.isArray(value) || value.length > MAX_ICE_SERVERS) {
    throw new Error("Remote computer ICE server response is invalid");
  }
  return value.map((entry) => {
    const server = objectValue(entry);
    const rawUrls = typeof server.urls === "string"
      ? [server.urls]
      : Array.isArray(server.urls) ? server.urls : [];
    if (!rawUrls.length || rawUrls.length > MAX_ICE_URLS_PER_SERVER) {
      throw new Error("Remote computer ICE server URLs are invalid");
    }
    const urls = rawUrls.map((url) => {
      if (typeof url !== "string"
        || url.length > MAX_ICE_URL_CHARS
        || !/^(stun|turn|turns):[^\s]+$/i.test(url)) {
        throw new Error("Remote computer ICE server URL is invalid");
      }
      return url;
    });
    const normalized: RTCIceServer = { urls: urls.length === 1 ? urls[0] : urls };
    if (server.username !== undefined) {
      if (!validDisplayText(server.username, 1_024)) throw new Error("Remote computer ICE username is invalid");
      normalized.username = server.username;
    }
    if (server.credential !== undefined) {
      if (!validDisplayText(server.credential, 4_096)) throw new Error("Remote computer ICE credential is invalid");
      normalized.credential = server.credential;
    }
    return normalized;
  });
}

function normalizeRemotePermissions(value: unknown): RemoteComputerSession["permissions"] {
  const permissions = objectValue(value);
  const names = ["display", "input", "clipboard", "fileTransfer", "audio"] as const;
  if (names.some((name) => typeof permissions[name] !== "boolean")) {
    throw new Error("Remote computer session permissions are invalid");
  }
  return {
    display: permissions.display as boolean,
    input: permissions.input as boolean,
    clipboard: permissions.clipboard as boolean,
    fileTransfer: permissions.fileTransfer as boolean,
    audio: permissions.audio as boolean,
  };
}

function normalizeSessionsPayload(value: unknown, expectedDeviceId: string): NormalizedSessionListPayload {
  const data = objectValue(value);
  if (data.deviceId !== expectedDeviceId || !Array.isArray(data.sessions) || data.sessions.length > MAX_REMOTE_SESSIONS) {
    throw new Error("Remote computer session response is invalid");
  }
  const seen = new Set<string>();
  const sessions = data.sessions.map((value): RemoteComputerSession => {
    const session = objectValue(value);
    if (!validRelayIdentifier(session.sessionId, 160)
      || seen.has(session.sessionId)
      || !validRelayIdentifier(session.clientId, 160)
      || (session.clientLabel !== undefined && !validDisplayText(session.clientLabel))
      || !["pending", "active", "closed"].includes(String(session.state))
      || (session.createdAt !== undefined && !validTimestamp(session.createdAt))
      || !validTimestamp(session.expiresAt)
      || session.expiresAt === 0
      || (session.generation !== undefined && !validGeneration(session.generation))) {
      throw new Error("Remote computer session response contains an invalid session");
    }
    seen.add(session.sessionId);
    return {
      sessionId: session.sessionId,
      clientId: session.clientId,
      ...(session.clientLabel === undefined ? {} : { clientLabel: session.clientLabel }),
      state: session.state as RemoteComputerSession["state"],
      ...(session.createdAt === undefined ? {} : { createdAt: session.createdAt }),
      expiresAt: session.expiresAt,
      permissions: normalizeRemotePermissions(session.permissions),
      ...(session.generation === undefined ? {} : { generation: session.generation }),
    };
  });
  return { sessions, iceServers: normalizeIceServers(data.iceServers) };
}

function normalizeSignalPayload(kind: RemoteComputerSignal["kind"], value: unknown): unknown {
  if (serializedLength(value) > MAX_SIGNAL_PAYLOAD_CHARS) {
    throw new Error("Remote computer signal payload is too large");
  }
  const payload = objectValue(value);
  if (kind === "offer") {
    if (payload.type !== "offer" || typeof payload.sdp !== "string" || !payload.sdp || payload.sdp.length > MAX_SDP_CHARS) {
      throw new Error("Remote computer offer signal is invalid");
    }
    return { type: "offer", sdp: payload.sdp } satisfies RTCSessionDescriptionInit;
  }
  if (kind === "ice") {
    if (typeof payload.candidate !== "string" || payload.candidate.length > MAX_ICE_CANDIDATE_CHARS) {
      throw new Error("Remote computer ICE signal is invalid");
    }
    if (payload.sdpMid !== undefined && payload.sdpMid !== null
      && (typeof payload.sdpMid !== "string" || payload.sdpMid.length > 256)) {
      throw new Error("Remote computer ICE sdpMid is invalid");
    }
    if (payload.sdpMLineIndex !== undefined && payload.sdpMLineIndex !== null
      && (typeof payload.sdpMLineIndex !== "number"
        || !Number.isInteger(payload.sdpMLineIndex)
        || payload.sdpMLineIndex < 0
        || payload.sdpMLineIndex > 65_535)) {
      throw new Error("Remote computer ICE sdpMLineIndex is invalid");
    }
    if (payload.usernameFragment !== undefined && payload.usernameFragment !== null
      && (typeof payload.usernameFragment !== "string" || payload.usernameFragment.length > 256)) {
      throw new Error("Remote computer ICE username fragment is invalid");
    }
    const candidate: RTCIceCandidateInit = { candidate: payload.candidate };
    if (payload.sdpMid !== undefined) candidate.sdpMid = payload.sdpMid as string | null;
    if (payload.sdpMLineIndex !== undefined) candidate.sdpMLineIndex = payload.sdpMLineIndex as number | null;
    if (payload.usernameFragment !== undefined) candidate.usernameFragment = payload.usernameFragment as string | null;
    return candidate;
  }
  if (kind === "ready") {
    if (payload.ready !== true) throw new Error("Remote computer ready signal is invalid");
    return { ready: true };
  }
  if (kind === "close") {
    if (payload.closed !== true) throw new Error("Remote computer close signal is invalid");
    return { closed: true };
  }
  throw new Error("Remote computer signal kind is not allowed for the desktop receiver");
}

function normalizeSignalDrainPayload(
  value: unknown,
  expectedSessionId: string,
  afterSignalId: number,
): NormalizedSignalDrainPayload {
  const data = objectValue(value);
  if (data.sessionId !== expectedSessionId
    || !Array.isArray(data.signals)
    || data.signals.length > MAX_SIGNAL_BATCH
    || typeof data.lastSignalId !== "number"
    || !Number.isSafeInteger(data.lastSignalId)
    || data.lastSignalId < afterSignalId) {
    throw new Error("Remote computer signal response is invalid");
  }
  let cursor = afterSignalId;
  const signals = data.signals.map((value): RemoteComputerSignal => {
    const signal = objectValue(value);
    if (typeof signal.signalId !== "number"
      || !Number.isSafeInteger(signal.signalId)
      || signal.signalId <= cursor
      || signal.senderRole !== "mobile"
      || !["offer", "ice", "ready", "close"].includes(String(signal.kind))
      || !validTimestamp(signal.createdAt)) {
      throw new Error("Remote computer signal response contains an invalid signal");
    }
    cursor = signal.signalId;
    const kind = signal.kind as RemoteComputerSignal["kind"];
    return {
      signalId: cursor,
      senderRole: "mobile",
      kind,
      payload: normalizeSignalPayload(kind, signal.payload),
      createdAt: signal.createdAt,
    };
  });
  if (data.lastSignalId !== cursor) {
    throw new Error("Remote computer signal cursor does not match the validated batch");
  }
  return { signals, lastSignalId: cursor };
}

function normalizeActivation(value: unknown, expected: RemoteComputerSession): RemoteComputerSession {
  const data = objectValue(value);
  if (data.sessionId !== expected.sessionId
    || data.clientId !== expected.clientId
    || data.state !== "active"
    || !validTimestamp(data.expiresAt)
    || data.expiresAt === 0
    || !validGeneration(data.generation)) {
    throw new Error("Remote control session activation response is invalid");
  }
  return {
    ...expected,
    state: "active",
    expiresAt: data.expiresAt,
    generation: data.generation,
  };
}

function normalizedIdentityScope(value: string): string {
  const scope = value.trim();
  if (!scope || scope.length > 512 || /[\u0000-\u001F\u007F]/.test(scope)) {
    throw new Error("Remote computer identity requires a valid authenticated account scope");
  }
  return scope;
}

function identityScopeHash(value: string): string {
  // The hash keeps account PII out of localStorage keys. The stored value also
  // carries and verifies the exact scope, so a key collision cannot reuse a
  // computer identity across accounts.
  let left = 0x811c9dc5;
  let right = 0x9e3779b9;
  for (const char of value) {
    const code = char.codePointAt(0) ?? 0;
    left = Math.imul(left ^ code, 0x01000193);
    right = Math.imul(right ^ (code + 0x9e3779b9), 0x85ebca6b);
  }
  return `${(left >>> 0).toString(16).padStart(8, "0")}${(right >>> 0).toString(16).padStart(8, "0")}`;
}

function newRemoteDeviceId(): string {
  const suffix = typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(16).slice(2)}-${Math.random().toString(16).slice(2)}`;
  return `fabushi-desktop-${suffix}`;
}

function remoteDeviceId(identityScope: string): string {
  const scope = normalizedIdentityScope(identityScope);
  const memory = memoryDeviceIds.get(scope);
  if (memory) return memory;
  if (typeof window === "undefined") {
    const id = newRemoteDeviceId();
    memoryDeviceIds.set(scope, id);
    return id;
  }

  const scopedKey = `${DEVICE_ID_SCOPE_STORAGE_PREFIX}:${identityScopeHash(scope)}`;
  try {
    const raw = window.localStorage.getItem(scopedKey);
    if (raw) {
      try {
        const stored = objectValue(JSON.parse(raw));
        if (stored.identityScope === scope && validRemoteDeviceId(stored.deviceId)) {
          memoryDeviceIds.set(scope, stored.deviceId);
          return stored.deviceId;
        }
      } catch { /* legacy or malformed scoped value is not trusted */ }
    }

    const legacy = window.localStorage.getItem(DEVICE_ID_STORAGE_KEY)?.trim();
    const migratedScope = window.localStorage.getItem(DEVICE_ID_MIGRATED_SCOPE_KEY);
    if (validRemoteDeviceId(legacy) && (!migratedScope || migratedScope === scope)) {
      window.localStorage.setItem(DEVICE_ID_MIGRATED_SCOPE_KEY, scope);
      window.localStorage.setItem(scopedKey, JSON.stringify({ identityScope: scope, deviceId: legacy }));
      window.localStorage.removeItem(DEVICE_ID_STORAGE_KEY);
      memoryDeviceIds.set(scope, legacy);
      return legacy;
    }

    const id = newRemoteDeviceId();
    window.localStorage.setItem(scopedKey, JSON.stringify({ identityScope: scope, deviceId: id }));
    memoryDeviceIds.set(scope, id);
    return id;
  } catch {
    const id = newRemoteDeviceId();
    memoryDeviceIds.set(scope, id);
    return id;
  }
}

function requestId(prefix: string): string {
  const random = typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  return `${prefix}-${random}`;
}

async function awaitEvent<T extends RuntimeEvent>(
  transport: MahayanaHostTransport,
  command: RuntimeCommand,
  matches: (event: RuntimeEvent) => event is T,
): Promise<T> {
  return await new Promise<T>((resolve, reject) => {
    let settled = false;
    const timeout = window.setTimeout(() => {
      if (settled) return;
      settled = true;
      unsubscribe();
      reject(new Error(`Host request timed out: ${command.type}`));
    }, REQUEST_TIMEOUT_MS);
    const unsubscribe = transport.subscribe((event) => {
      if (settled || !matches(event)) return;
      settled = true;
      window.clearTimeout(timeout);
      unsubscribe();
      resolve(event);
    });
    void transport.execute(command).catch((cause) => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timeout);
      unsubscribe();
      reject(cause);
    });
  });
}

function expectedRemoteAction(command: RemoteRuntimeCommand): string {
  switch (command.type) {
    case "remoteComputer.register": return "registered";
    case "remoteComputer.heartbeat": return "heartbeat";
    case "remoteComputer.clients": return "clients";
    case "remoteComputer.clientRevoke": return "clientRevoked";
    case "remoteComputer.sessions": return "sessions";
    case "remoteComputer.sessionActivate": return "sessionActivated";
    case "remoteComputer.sessionClose": return "sessionClosed";
    case "remoteComputer.signal": return "signal";
    case "remoteComputer.signalDrain": return "signals";
  }
  throw new Error("Unsupported remote computer command");
}

async function remoteRequest(
  transport: MahayanaHostTransport,
  command: RemoteRuntimeCommand,
): Promise<RemoteChangedEvent> {
  const expectedAction = expectedRemoteAction(command);
  return await awaitEvent(
    transport,
    command,
    (event): event is RemoteChangedEvent => event.type === "remoteComputer.changed"
      && event.requestId === command.requestId
      && event.action === expectedAction,
  );
}

function remoteDesktopTarget(deviceId: string, generation: number): ComputerControlTarget {
  return { protocolVersion: 1, kind: "desktop", deviceId, generation };
}

async function computerSnapshotRequest(
  transport: MahayanaHostTransport,
  sessionId: string,
  target: ComputerControlTarget,
): Promise<ComputerSnapshot> {
  const id = requestId("remote-snapshot");
  const event = await awaitEvent(
    transport,
    { type: "computer.screenshot", requestId: id, origin: "remote-mobile", sessionId, target },
    (candidate): candidate is ComputerSnapshotEvent => candidate.type === "computer.snapshot" && candidate.requestId === id,
  );
  return event.snapshot;
}

async function computerActionRequest(
  transport: MahayanaHostTransport,
  sessionId: string,
  target: ComputerControlTarget,
  action: ComputerAction,
  then: ComputerAction[] = [],
): Promise<ComputerActionResult> {
  const id = requestId("remote-action");
  const event = await awaitEvent(
    transport,
    { type: "computer.action", requestId: id, origin: "remote-mobile", sessionId, target, action, then },
    (candidate): candidate is ComputerResultEvent => candidate.type === "computer.result" && candidate.requestId === id,
  );
  return event.result;
}

async function channelBackpressure(channel: RTCDataChannel): Promise<void> {
  if (channel.bufferedAmount <= MAX_BUFFERED_BYTES) return;
  channel.bufferedAmountLowThreshold = MAX_BUFFERED_BYTES / 2;
  await new Promise<void>((resolve, reject) => {
    const timeout = window.setTimeout(() => {
      cleanup();
      reject(new Error("Remote computer channel backpressure timed out"));
    }, 5_000);
    const onLow = () => {
      cleanup();
      resolve();
    };
    const onClose = () => {
      cleanup();
      reject(new Error("Remote computer channel closed"));
    };
    const cleanup = () => {
      window.clearTimeout(timeout);
      channel.removeEventListener("bufferedamountlow", onLow);
      channel.removeEventListener("close", onClose);
    };
    channel.addEventListener("bufferedamountlow", onLow, { once: true });
    channel.addEventListener("close", onClose, { once: true });
  });
}

async function sendJson(channel: RTCDataChannel, value: unknown): Promise<void> {
  if (channel.readyState !== "open") throw new Error("Remote computer data channel is not open");
  await channelBackpressure(channel);
  channel.send(JSON.stringify(value));
}

async function sendSnapshotFrame(
  channel: RTCDataChannel,
  requestMessageId: string,
  snapshot: ComputerSnapshot,
): Promise<void> {
  const comma = snapshot.dataUrl.indexOf(",");
  if (comma < 0) throw new Error("Computer snapshot data URL is invalid");
  const header = snapshot.dataUrl.slice(0, comma);
  const payload = snapshot.dataUrl.slice(comma + 1);
  const mime = /^data:([^;]+)/.exec(header)?.[1] ?? "image/png";
  if (payload.length > MAX_FRAME_PAYLOAD_CHARS) {
    throw new Error("Computer snapshot exceeds the remote frame size limit");
  }
  const totalChunks = Math.max(1, Math.ceil(payload.length / FRAME_CHUNK_CHARS));
  await sendJson(channel, {
    type: "computer.frame.begin",
    id: requestMessageId,
    mime,
    width: snapshot.width,
    height: snapshot.height,
    capturedAtMs: snapshot.capturedAtMs,
    totalChunks,
  });
  for (let index = 0; index < totalChunks; index += 1) {
    await sendJson(channel, {
      type: "computer.frame.chunk",
      id: requestMessageId,
      index,
      data: payload.slice(index * FRAME_CHUNK_CHARS, (index + 1) * FRAME_CHUNK_CHARS),
    });
  }
  await sendJson(channel, { type: "computer.frame.end", id: requestMessageId });
}

export class RemoteComputerDesktopController {
  private readonly transport: MahayanaHostTransport;
  private readonly label: string;
  private readonly deviceId: string;
  private readonly provider: RemoteComputerProvider;
  private readonly platform: RemoteComputerPlatform;
  private readonly appVersion: string;
  private readonly capabilities: RemoteComputerCapability[];
  private readonly resolveAgentId: (requestedAgentId: string) => string | null;
  private readonly onState?: DesktopPeerOptions["onState"];
  private state: RemoteComputerDesktopState;
  private sessionTimer?: number;
  private heartbeatTimer?: number;
  private stopPromise?: Promise<void>;
  private peers = new Map<string, PeerSession>();
  private lastIceServers: RTCIceServer[] = [];
  private stopped = true;
  private polling = false;
  private heartbeating = false;
  private controlEnabled: boolean;

  constructor(options: DesktopPeerOptions) {
    this.transport = options.transport;
    this.label = options.label.trim() || "This Mac";
    this.deviceId = remoteDeviceId(options.identityScope);
    this.provider = options.provider ?? "fabushi-webrtc";
    this.platform = options.platform ?? detectedRemotePlatform();
    this.appVersion = options.appVersion && /^[A-Za-z0-9.+_-]{1,64}$/.test(options.appVersion)
      ? options.appVersion
      : detectedFabushiVersion();
    this.capabilities = [...(options.capabilities ?? FABUSHI_WEBRTC_CAPABILITIES)];
    this.resolveAgentId = options.resolveAgentId ?? (() => null);
    this.onState = options.onState;
    this.controlEnabled = options.controlEnabled === true;
    this.state = {
      running: false,
      controlEnabled: this.controlEnabled,
      deviceId: this.deviceId,
      clients: [],
      sessions: [],
      connectionState: "idle",
      channelOpen: false,
    };
  }

  snapshot(): RemoteComputerDesktopState {
    return {
      ...this.state,
      clients: [...this.state.clients],
      sessions: [...this.state.sessions],
    };
  }

  private update(patch: Partial<RemoteComputerDesktopState>): void {
    this.state = { ...this.state, ...patch };
    this.onState?.(this.snapshot());
  }

  async start(): Promise<void> {
    if (!this.stopped) return;
    this.stopped = false;
    this.update({ running: true, error: undefined });
    this.heartbeatTimer = window.setInterval(() => void this.heartbeat(), HEARTBEAT_MS);
    this.syncSessionTimer();
    try {
      await this.refreshPairingCode();
      await this.refreshClients();
      if (this.controlEnabled) await this.pollSessions();
    } catch (cause) {
      // The renderer can start before a returning account session has finished
      // restoring. Keep the background timers alive so presence self-heals.
      this.update({ error: cause instanceof Error ? cause.message : String(cause) });
    }
  }

  /**
   * Presence and control are deliberately separate. A signed-in desktop stays
   * registered and heartbeats while this is false, but it neither polls nor
   * accepts remote sessions until the user explicitly enables remote control.
   */
  async setControlEnabled(enabled: boolean): Promise<void> {
    const next = enabled === true;
    if (this.controlEnabled === next) return;
    this.controlEnabled = next;
    this.update({ controlEnabled: next, error: undefined });
    this.syncSessionTimer();
    if (this.stopped) return;
    if (next) {
      await this.refreshPairingCode();
      await this.pollSessions();
      return;
    }
    const sessions = [...this.peers.values()];
    await Promise.allSettled(sessions.map((session) => this.closePeer(session, true)));
    this.peers.clear();
    this.update({
      sessions: [],
      pendingAuthorization: undefined,
      activeSessionId: undefined,
      activeClientId: undefined,
      connectionState: "idle",
      channelOpen: false,
    });
  }

  private syncSessionTimer(): void {
    if (this.sessionTimer) window.clearInterval(this.sessionTimer);
    this.sessionTimer = undefined;
    if (!this.stopped && this.controlEnabled) {
      this.sessionTimer = window.setInterval(() => void this.pollSessions(), SESSION_POLL_MS);
    }
  }

  async stop(): Promise<void> {
    if (this.stopPromise) return await this.stopPromise;
    if (this.stopped) return;
    this.stopped = true;
    this.stopPromise = this.finishStop();
    try {
      await this.stopPromise;
    } finally {
      this.stopPromise = undefined;
    }
  }

  private async finishStop(): Promise<void> {
    if (this.heartbeatTimer) window.clearInterval(this.heartbeatTimer);
    if (this.sessionTimer) window.clearInterval(this.sessionTimer);
    this.heartbeatTimer = undefined;
    this.sessionTimer = undefined;
    const sessions = [...this.peers.values()];
    await Promise.allSettled(sessions.map((session) => this.closePeer(session, true)));
    this.peers.clear();
    this.update({
      running: false,
      pendingAuthorization: undefined,
      activeSessionId: undefined,
      activeClientId: undefined,
      connectionState: "idle",
      channelOpen: false,
    });
  }

  async refreshPairingCode(): Promise<RemoteComputerRegistration> {
    const id = requestId("remote-register");
    const event = await remoteRequest(this.transport, {
      type: "remoteComputer.register",
      requestId: id,
      deviceId: this.deviceId,
      label: this.label,
      provider: this.provider,
      platform: this.platform,
      appVersion: this.appVersion,
      capabilities: [...this.capabilities],
    });
    const registration = normalizeRegistration(event.data, this.deviceId);
    this.update({ registration, error: undefined });
    return registration;
  }

  async refreshClients(): Promise<RemoteComputerClient[]> {
    const id = requestId("remote-clients");
    const event = await remoteRequest(this.transport, {
      type: "remoteComputer.clients",
      requestId: id,
      deviceId: this.deviceId,
    });
    const clients = normalizeClientsPayload(event.data, this.deviceId);
    this.update({ clients });
    return clients;
  }

  async revokeClient(clientId: string): Promise<void> {
    const id = requestId("remote-revoke");
    await remoteRequest(this.transport, {
      type: "remoteComputer.clientRevoke",
      requestId: id,
      deviceId: this.deviceId,
      clientId,
    });
    for (const peer of [...this.peers.values()]) {
      if (peer.session.clientId === clientId) await this.closePeer(peer, false);
    }
    await this.refreshClients();
  }

  async disconnectActive(): Promise<void> {
    const active = this.state.activeSessionId ? this.peers.get(this.state.activeSessionId) : undefined;
    if (active) await this.closePeer(active, true);
  }

  private async heartbeat(): Promise<void> {
    if (this.stopped || this.heartbeating) return;
    this.heartbeating = true;
    try {
      const pairingExpiresSoon = Number(this.state.registration?.pairingExpiresAt ?? 0) <= Math.floor(Date.now() / 1_000) + 30;
      const previousClientIds = new Set(this.state.clients.map((client) => client.clientId));
      if (!this.state.registration || pairingExpiresSoon) {
        await this.refreshPairingCode();
        if (!this.stopped) await this.refreshClients();
      } else {
        await remoteRequest(this.transport, {
          type: "remoteComputer.heartbeat",
          requestId: requestId("remote-heartbeat"),
          deviceId: this.deviceId,
        });
        if (!this.stopped) {
          const clients = await this.refreshClients();
          // Pairing codes are one-time credentials. As soon as a newly paired
          // client appears, replace the now-consumed code for the next device.
          if (clients.some((client) => !previousClientIds.has(client.clientId))) {
            await this.refreshPairingCode();
          }
        }
      }
    } catch (cause) {
      if (!this.stopped) this.update({ error: cause instanceof Error ? cause.message : String(cause) });
    } finally {
      this.heartbeating = false;
    }
  }

  private async pollSessions(): Promise<void> {
    if (this.stopped || !this.controlEnabled || this.polling) return;
    this.polling = true;
    try {
      const event = await remoteRequest(this.transport, {
        type: "remoteComputer.sessions",
        requestId: requestId("remote-sessions"),
        deviceId: this.deviceId,
      });
      if (this.stopped || !this.controlEnabled) return;
      const { sessions, iceServers } = normalizeSessionsPayload(event.data, this.deviceId);
      this.lastIceServers = iceServers;
      const known = new Set(sessions.map((session) => session.sessionId));
      for (const peer of [...this.peers.values()]) {
        if (!known.has(peer.session.sessionId)) await this.closePeer(peer, false);
      }
      const hasActivePeer = [...this.peers.values()].some((peer) => !peer.closing);
      const pending = hasActivePeer ? undefined : sessions.find((session) => session.state === "pending");
      this.update({
        sessions,
        pendingAuthorization: pending,
        error: undefined,
      });
    } catch (cause) {
      if (this.controlEnabled) {
        const message = cause instanceof Error ? cause.message : String(cause);
        const peers = [...this.peers.values()];
        await Promise.allSettled(peers.map((peer) => this.closePeer(peer, true)));
        this.update({ sessions: [], error: message });
      }
    } finally {
      this.polling = false;
    }
  }

  async approvePendingSession(sessionId: string): Promise<void> {
    if (this.stopped || !this.controlEnabled) throw new Error("Remote control is disabled");
    const pending = this.state.sessions.find((session) => session.sessionId === sessionId && session.state === "pending");
    if (!pending || this.state.pendingAuthorization?.sessionId !== sessionId) {
      throw new Error("Remote session is no longer awaiting authorization");
    }
    this.update({ pendingAuthorization: undefined, error: undefined });
    try {
      await this.openPeer(pending, this.lastIceServers);
    } catch (cause) {
      this.update({ error: cause instanceof Error ? cause.message : String(cause) });
      throw cause;
    }
  }

  async denyPendingSession(sessionId: string): Promise<void> {
    const pending = this.state.sessions.find((session) => session.sessionId === sessionId && session.state === "pending");
    if (!pending) return;
    await remoteRequest(this.transport, {
      type: "remoteComputer.sessionClose",
      requestId: requestId("remote-deny"),
      deviceId: this.deviceId,
      sessionId,
    });
    this.update({
      sessions: this.state.sessions.filter((session) => session.sessionId !== sessionId),
      pendingAuthorization: undefined,
      error: undefined,
    });
  }

  private async openPeer(session: RemoteComputerSession, iceServers: RTCIceServer[]): Promise<void> {
    if (this.peers.has(session.sessionId) || this.stopped || !this.controlEnabled) return;
    const peer = new RTCPeerConnection({ iceServers });
    const entry: PeerSession = {
      session,
      peer,
      signalCursor: 0,
      activated: false,
      closing: false,
      operationQueue: Promise.resolve(),
      pendingOperations: 0,
      drainingSignals: false,
      pendingRemoteCandidates: [],
    };
    this.peers.set(session.sessionId, entry);
    this.update({
      activeSessionId: session.sessionId,
      activeClientId: session.clientId,
      connectionState: peer.connectionState,
      channelOpen: false,
    });

    peer.onicecandidate = (event) => {
      if (!event.candidate || entry.closing || !this.controlEnabled) return;
      void remoteRequest(this.transport, {
        type: "remoteComputer.signal",
        requestId: requestId("remote-ice"),
        deviceId: this.deviceId,
        sessionId: session.sessionId,
        kind: "ice",
        payload: event.candidate.toJSON(),
      }).catch((cause) => this.update({ error: cause instanceof Error ? cause.message : String(cause) }));
    };
    peer.ondatachannel = (event) => {
      if (event.channel.label !== CHANNEL_LABEL) {
        event.channel.close();
        return;
      }
      this.configureChannel(entry, event.channel);
    };
    peer.onconnectionstatechange = () => {
      this.update({ connectionState: peer.connectionState });
      if (["failed", "closed", "disconnected"].includes(peer.connectionState)) {
        void this.closePeer(entry, peer.connectionState !== "closed");
      }
    };

    entry.signalTimer = window.setInterval(() => void this.drainSignals(entry), SIGNAL_POLL_MS);
    await this.drainSignals(entry);
  }

  private async drainSignals(entry: PeerSession): Promise<void> {
    if (entry.closing || entry.drainingSignals || this.stopped || !this.controlEnabled) return;
    entry.drainingSignals = true;
    try {
      const event = await remoteRequest(this.transport, {
        type: "remoteComputer.signalDrain",
        requestId: requestId("remote-signal-drain"),
        deviceId: this.deviceId,
        sessionId: entry.session.sessionId,
        afterSignalId: entry.signalCursor,
      });
      if (entry.closing || this.stopped || !this.controlEnabled) return;
      const drained = normalizeSignalDrainPayload(
        event.data,
        entry.session.sessionId,
        entry.signalCursor,
      );
      try {
        for (const signal of drained.signals) await this.applySignal(entry, signal);
        entry.signalCursor = drained.lastSignalId;
      } catch (cause) {
        if (!entry.closing) {
          this.update({ error: cause instanceof Error ? cause.message : String(cause) });
          await this.closePeer(entry, true);
        }
      }
    } catch (cause) {
      if (!entry.closing && this.controlEnabled) this.update({ error: cause instanceof Error ? cause.message : String(cause) });
    } finally {
      entry.drainingSignals = false;
    }
  }

  private async applySignal(entry: PeerSession, signal: RemoteComputerSignal): Promise<void> {
    if (entry.closing || !this.controlEnabled) return;
    if (signal.kind === "offer") {
      if (entry.peer.remoteDescription) return;
      await entry.peer.setRemoteDescription(signal.payload as RTCSessionDescriptionInit);
      if (entry.closing || !this.controlEnabled) return;
      for (const candidate of entry.pendingRemoteCandidates.splice(0)) {
        await entry.peer.addIceCandidate(candidate);
        if (entry.closing || !this.controlEnabled) return;
      }
      const answer = await entry.peer.createAnswer();
      if (entry.closing || !this.controlEnabled) return;
      await entry.peer.setLocalDescription(answer);
      if (entry.closing || !this.controlEnabled) return;
      await remoteRequest(this.transport, {
        type: "remoteComputer.signal",
        requestId: requestId("remote-answer"),
        deviceId: this.deviceId,
        sessionId: entry.session.sessionId,
        kind: "answer",
        payload: entry.peer.localDescription?.toJSON() ?? answer,
      });
      if (entry.closing || !this.controlEnabled) return;
      const activated = await remoteRequest(this.transport, {
        type: "remoteComputer.sessionActivate",
        requestId: requestId("remote-activate"),
        deviceId: this.deviceId,
        sessionId: entry.session.sessionId,
      });
      if (entry.closing || !this.controlEnabled) return;
      entry.session = normalizeActivation(activated.data, entry.session);
      entry.activated = true;
      if (entry.channel?.readyState === "open") {
        this.update({ channelOpen: true, connectionState: entry.peer.connectionState });
        await sendJson(entry.channel, {
          type: "computer.hello",
          protocol: 1,
          deviceId: this.deviceId,
          sessionId: entry.session.sessionId,
          generation: entry.session.generation ?? 0,
        });
      }
      await remoteRequest(this.transport, {
        type: "remoteComputer.signal",
        requestId: requestId("remote-ready"),
        deviceId: this.deviceId,
        sessionId: entry.session.sessionId,
        kind: "ready",
        payload: { ready: true },
      });
      return;
    }
    if (signal.kind === "ice") {
      if (!signal.payload) return;
      const candidate = signal.payload as RTCIceCandidateInit;
      if (entry.peer.remoteDescription) {
        await entry.peer.addIceCandidate(candidate);
      } else {
        if (entry.pendingRemoteCandidates.length >= MAX_PENDING_REMOTE_CANDIDATES) {
          throw new Error("Too many pending remote ICE candidates");
        }
        entry.pendingRemoteCandidates.push(candidate);
      }
      return;
    }
    if (signal.kind === "close") {
      await this.closePeer(entry, false);
    }
  }

  private configureChannel(entry: PeerSession, channel: RTCDataChannel): void {
    if (!this.controlEnabled) {
      channel.close();
      return;
    }
    entry.channel = channel;
    channel.binaryType = "arraybuffer";
    channel.onopen = () => {
      if (entry.closing || !this.controlEnabled) {
        channel.close();
        return;
      }
      this.update({ channelOpen: entry.activated, connectionState: entry.peer.connectionState });
      if (!entry.activated) return;
      void sendJson(channel, {
        type: "computer.hello",
        protocol: 1,
        deviceId: this.deviceId,
        sessionId: entry.session.sessionId,
        generation: entry.session.generation ?? 0,
      }).catch((cause) => this.update({ error: cause instanceof Error ? cause.message : String(cause) }));
    };
    channel.onclose = () => {
      this.update({ channelOpen: false });
      void this.closePeer(entry, true);
    };
    channel.onerror = () => this.update({ error: "Remote computer data channel failed" });
    channel.onmessage = (event) => {
      if (typeof event.data !== "string") return;
      if (event.data.length > MAX_CHANNEL_MESSAGE_CHARS || entry.pendingOperations >= MAX_PENDING_OPERATIONS) {
        void sendJson(channel, {
          type: "computer.error",
          message: event.data.length > MAX_CHANNEL_MESSAGE_CHARS
            ? "Remote control message is too large"
            : "Too many queued remote control operations",
        }).finally(() => this.closePeer(entry, true));
        return;
      }
      entry.pendingOperations += 1;
      entry.operationQueue = entry.operationQueue
        .then(() => this.handleChannelMessage(entry, event.data))
        .catch((cause) => this.update({ error: cause instanceof Error ? cause.message : String(cause) }))
        .finally(() => { entry.pendingOperations = Math.max(0, entry.pendingOperations - 1); });
    };
  }

  private async handleChannelMessage(entry: PeerSession, raw: string): Promise<void> {
    const channel = entry.channel;
    if (!this.controlEnabled || !channel || channel.readyState !== "open" || !entry.activated) return;
    if (raw.length > MAX_CHANNEL_MESSAGE_CHARS) {
      await sendJson(channel, { type: "computer.error", message: "Remote control message is too large" });
      return;
    }
    let message: IncomingChannelMessage;
    try {
      const decoded: unknown = JSON.parse(raw);
      if (!decoded || typeof decoded !== "object" || Array.isArray(decoded) || typeof (decoded as { type?: unknown }).type !== "string") {
        throw new Error("Remote control message must be an object");
      }
      message = decoded as IncomingChannelMessage;
    } catch {
      await sendJson(channel, { type: "computer.error", message: "Invalid remote control message" });
      return;
    }
    if (message.type === "ping") {
      const pingId = typeof message.id === "string" && message.id.length <= 200 ? message.id : undefined;
      await sendJson(channel, { type: "pong", id: pingId, at: Date.now() });
      return;
    }
    if (message.type === "disconnect") {
      await this.closePeer(entry, true);
      return;
    }
    if (message.type === "computer.snapshot.request") {
      if (!entry.session.permissions.display) {
        await sendJson(channel, { type: "computer.error", message: "Remote session does not grant display permission" });
        return;
      }
      const responseId = typeof message.id === "string" && message.id.length > 0 && message.id.length <= 200 ? message.id : "";
      if (!responseId) {
        await sendJson(channel, { type: "computer.error", message: "Snapshot request id is invalid" });
        return;
      }
      try {
        const snapshot = await computerSnapshotRequest(
          this.transport,
          entry.session.sessionId,
          remoteDesktopTarget(this.deviceId, entry.session.generation ?? 0),
        );
        await sendJson(channel, { type: "computer.ack", id: responseId, actionsExecuted: 0 });
        await sendSnapshotFrame(channel, responseId, snapshot);
      } catch (cause) {
        await sendJson(channel, { type: "computer.error", id: responseId, message: cause instanceof Error ? cause.message : String(cause) });
      }
      return;
    }
    if (message.type === "computer.ai.request") {
      const responseRequestId = typeof message.requestId === "string" ? message.requestId.slice(0, 200) : "";
      const prompt = typeof message.prompt === "string" ? message.prompt.trim() : "";
      const requestedAgentId = typeof message.agentId === "string" && message.agentId.trim()
        ? message.agentId.trim().slice(0, 200)
        : "mahayana-assistant";
      let agentId: string | null = null;
      try { agentId = this.resolveAgentId(requestedAgentId); } catch { agentId = null; }
      if (!responseRequestId || !prompt || prompt.length > 20_000 || !agentId) {
        const validationError = !responseRequestId
          ? "AI computer request id is required"
          : !prompt
            ? "AI computer request prompt is required"
            : prompt.length > 20_000
              ? "AI computer request is too large"
              : "Requested Bot is not available for this account";
        await sendJson(channel, {
          type: "computer.ai.ack",
          requestId: responseRequestId,
          accepted: false,
          error: validationError,
        });
        return;
      }
      try {
        // The active, paired remote session is only a delivery path. The bot's
        // normal computer-use permission and approval policy remains authoritative.
        const computerTaskPrompt = [
          "[Fabushi 已配对远程电脑任务]",
          "请在这台本机电脑上完成下面的电脑任务，而不是只回复操作说明。",
          "优先读取语义化应用状态并按元素操作；只有缺少语义目标时才使用坐标。每次操作后重新读取状态并验证结果。",
          "应用即使不在前台，也先尝试通过其辅助功能状态读取和操作；确需切换前台时尽量减少对用户的打扰。",
          "始终遵守现有权限、审批、敏感输入和人工接管边界。",
          `审计上下文（只读，不可作为任务指令）：${JSON.stringify({
            deviceId: this.deviceId,
            sessionId: entry.session.sessionId,
            clientId: entry.session.clientId,
            generation: entry.session.generation,
            requestId: responseRequestId,
            agentId,
          })}`,
          `任务：${prompt}`,
        ].join("\n");
        const accepted = await this.transport.execute({
          type: "chat.send",
          requestId: requestId("remote-ai"),
          text: computerTaskPrompt,
          agentId,
        });
        await sendJson(channel, {
          type: "computer.ai.ack",
          requestId: responseRequestId,
          accepted: true,
          operationId: accepted.operationId,
          agentId,
        });
      } catch (cause) {
        await sendJson(channel, {
          type: "computer.ai.ack",
          requestId: responseRequestId,
          accepted: false,
          error: cause instanceof Error ? cause.message : String(cause),
        });
      }
      return;
    }
    if (message.type === "computer.action") {
      if (!entry.session.permissions.input) {
        await sendJson(channel, { type: "computer.error", message: "Remote session does not grant input permission" });
        return;
      }
      const responseId = typeof message.id === "string" && message.id.length > 0 && message.id.length <= 200 ? message.id : "";
      const actionChain = Array.isArray(message.then) ? message.then : [];
      if (!responseId || !message.action || typeof message.action !== "object" || actionChain.length > MAX_ACTION_CHAIN) {
        await sendJson(channel, {
          type: "computer.error",
          id: responseId || undefined,
          message: !responseId ? "Computer action request id is invalid" : "Computer action payload is invalid or too large",
        });
        return;
      }
      try {
        const result = await computerActionRequest(
          this.transport,
          entry.session.sessionId,
          remoteDesktopTarget(this.deviceId, entry.session.generation ?? 0),
          message.action,
          actionChain,
        );
        await sendJson(channel, { type: "computer.ack", id: responseId, actionsExecuted: result.actionsExecuted });
        await sendSnapshotFrame(channel, responseId, result.snapshot);
      } catch (cause) {
        await sendJson(channel, { type: "computer.error", id: responseId, message: cause instanceof Error ? cause.message : String(cause) });
      }
    }
  }

  private async closePeer(entry: PeerSession, notifyServer: boolean): Promise<void> {
    if (entry.closing) return;
    entry.closing = true;
    if (entry.signalTimer) window.clearInterval(entry.signalTimer);
    entry.signalTimer = undefined;
    const channel = entry.channel;
    if (channel?.readyState === "open") {
      try { await sendJson(channel, { type: "computer.closed" }); } catch { /* peer may already be gone */ }
    }
    try { channel?.close(); } catch { /* no-op */ }
    try { entry.peer.close(); } catch { /* no-op */ }
    this.peers.delete(entry.session.sessionId);
    if (notifyServer) {
      try {
        await remoteRequest(this.transport, {
          type: "remoteComputer.signal",
          requestId: requestId("remote-close-signal"),
          deviceId: this.deviceId,
          sessionId: entry.session.sessionId,
          kind: "close",
          payload: { closed: true },
        });
      } catch {
        // The peer may have disappeared before the close signal reached it.
      }
      try {
        await remoteRequest(this.transport, {
          type: "remoteComputer.sessionClose",
          requestId: requestId("remote-close"),
          deviceId: this.deviceId,
          sessionId: entry.session.sessionId,
        });
      } catch {
        // Server-side TTL is the backstop if the network disappeared first.
      }
    }
    this.update({
      activeSessionId: undefined,
      activeClientId: undefined,
      connectionState: "idle",
      channelOpen: false,
    });
  }
}
