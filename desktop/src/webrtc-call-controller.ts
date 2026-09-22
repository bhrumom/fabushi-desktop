export type FabushiCallKind = 'voice' | 'video';

type InviteSignal = {
  type: 'invite';
  callId: string;
  conversationId: string;
  kind: FabushiCallKind | 'groupVoice' | 'groupVideo' | 'liveStream';
  from: string;
  participants: string[];
};

type CallSignal = InviteSignal
  | { type: 'ringing'; callId: string; actorId: string }
  | { type: 'accept'; callId: string; actorId: string }
  | { type: 'decline'; callId: string; actorId: string; reason: string }
  | { type: 'sdpOffer'; callId: string; from: string; to: string; sdp: string }
  | { type: 'sdpAnswer'; callId: string; from: string; to: string; sdp: string }
  | { type: 'iceCandidate'; callId: string; from: string; to?: string | null; candidate: string; sdpMid?: string | null; sdpMlineIndex?: number | null }
  | { type: 'mediaState'; callId: string; actorId: string; muted: boolean; videoEnabled: boolean; screenSharing: boolean }
  | { type: 'speaking'; callId: string; actorId: string; active: boolean }
  | { type: 'hangup'; callId: string; actorId: string; reason: string };

export type WebRtcCallStatus = 'idle' | 'ringing' | 'connecting' | 'active' | 'ended' | 'failed';

export interface IncomingFabushiCall {
  callId: string;
  conversationId: string;
  fromActorId: string;
  kind: FabushiCallKind;
}

export interface ActiveFabushiCall {
  callId: string;
  conversationId: string;
  peerActorId: string;
  actorId: string;
  kind: FabushiCallKind;
  incoming: boolean;
}

type ControllerCallbacks = {
  onIncoming?: (call: IncomingFabushiCall) => void;
  onStatus?: (status: WebRtcCallStatus, detail?: string) => void;
  onRemoteStream?: (stream: MediaStream | null) => void;
  onIdentity?: (identity: { actorId: string; deviceId: string; sessionId: string }) => void;
};

type IdentitySource = () => { deviceId: string; sessionId: string };

type SignalingConnection = {
  actorId: string;
  deviceId: string;
  sessionId: string;
  secure: boolean;
  iceServers?: RTCIceServer[];
};

export class FabushiWebRtcController {
  private readonly callbacks: ControllerCallbacks;
  private readonly identitySource: IdentitySource;
  private unsubscribe: (() => void) | null = null;
  private signaling: SignalingConnection | null = null;
  private connectionPromise: Promise<SignalingConnection> | null = null;
  private peerConnection: RTCPeerConnection | null = null;
  private localStream: MediaStream | null = null;
  private remoteStream: MediaStream | null = null;
  private activeCall: ActiveFabushiCall | null = null;
  private pendingInvite: InviteSignal | null = null;
  private pendingIce: RTCIceCandidateInit[] = [];
  private disposed = false;

  constructor(identitySource: IdentitySource, callbacks: ControllerCallbacks = {}) {
    this.identitySource = identitySource;
    this.callbacks = callbacks;
    const bridge = typeof window !== 'undefined' ? window.fabushiNative : undefined;
    if (bridge?.subscribe) {
      this.unsubscribe = bridge.subscribe({
        'messaging-call-signal': (payload) => void this.handleSignal(payload as CallSignal),
        'messaging-call-status': (payload) => {
          const state = String((payload as { state?: string } | null)?.state || '');
          if (state === 'failed') this.callbacks.onStatus?.('failed', String((payload as { message?: string }).message || '信令连接失败'));
        },
      });
    }
  }

  currentCall(): ActiveFabushiCall | null {
    return this.activeCall;
  }

  localMedia(): MediaStream | null {
    return this.localStream;
  }

  async connect(): Promise<SignalingConnection> {
    if (this.signaling) return this.signaling;
    if (this.connectionPromise) return this.connectionPromise;
    const bridge = window.fabushiNative;
    if (!bridge?.invoke) throw new Error('当前桌面环境没有 Fabushi 原生信令桥。');
    const identity = this.identitySource();
    this.connectionPromise = bridge.invoke<SignalingConnection>('connectMessagingSignaling', {
      deviceId: identity.deviceId,
      sessionId: identity.sessionId,
    }).then((connection) => {
      if (!connection?.actorId || !connection.deviceId || !connection.sessionId) {
        throw new Error('Fabushi 通话身份无效。');
      }
      this.signaling = connection;
      this.callbacks.onIdentity?.({
        actorId: connection.actorId,
        deviceId: connection.deviceId,
        sessionId: connection.sessionId,
      });
      return connection;
    }).finally(() => {
      this.connectionPromise = null;
    });
    return this.connectionPromise;
  }

  async start(input: { conversationId: string; peerActorId: string; kind: FabushiCallKind }): Promise<ActiveFabushiCall> {
    if (this.activeCall) throw new Error('已有通话正在进行。');
    const signaling = await this.connect();
    const call: ActiveFabushiCall = {
      callId: `call:${crypto.randomUUID()}`,
      conversationId: input.conversationId,
      peerActorId: input.peerActorId,
      actorId: signaling.actorId,
      kind: input.kind,
      incoming: false,
    };
    this.activeCall = call;
    this.callbacks.onStatus?.('ringing');
    await this.send({
      type: 'invite',
      callId: call.callId,
      conversationId: call.conversationId,
      kind: call.kind,
      from: call.actorId,
      participants: [call.peerActorId],
    });
    return call;
  }

  async acceptIncoming(): Promise<ActiveFabushiCall> {
    const invite = this.pendingInvite;
    if (!invite) throw new Error('没有待接听的 Fabushi 通话。');
    const signaling = await this.connect();
    const kind: FabushiCallKind = invite.kind === 'video' ? 'video' : 'voice';
    const call: ActiveFabushiCall = {
      callId: invite.callId,
      conversationId: invite.conversationId,
      peerActorId: invite.from,
      actorId: signaling.actorId,
      kind,
      incoming: true,
    };
    this.activeCall = call;
    this.pendingInvite = null;
    await this.preparePeer(call);
    await this.send({ type: 'accept', callId: call.callId, actorId: call.actorId });
    this.callbacks.onStatus?.('connecting');
    return call;
  }

  async declineIncoming(reason = 'declined'): Promise<void> {
    const invite = this.pendingInvite;
    if (!invite) return;
    const signaling = await this.connect();
    await this.send({ type: 'decline', callId: invite.callId, actorId: signaling.actorId, reason });
    this.pendingInvite = null;
    this.callbacks.onStatus?.('ended');
  }

  async hangup(reason = 'hangup'): Promise<void> {
    const call = this.activeCall;
    if (call) {
      try { await this.send({ type: 'hangup', callId: call.callId, actorId: call.actorId, reason }); } catch { /* connection may already be gone */ }
    }
    this.cleanupCall();
    this.callbacks.onStatus?.('ended');
  }

  async setMuted(muted: boolean): Promise<void> {
    this.localStream?.getAudioTracks().forEach((track) => { track.enabled = !muted; });
    await this.sendMediaState({ muted });
  }

  async setVideoEnabled(enabled: boolean): Promise<void> {
    this.localStream?.getVideoTracks().forEach((track) => { track.enabled = enabled; });
    await this.sendMediaState({ videoEnabled: enabled });
  }

  async startScreenShare(): Promise<void> {
    const call = this.activeCall;
    const connection = this.peerConnection;
    if (!call || !connection) throw new Error('通话尚未建立。');
    const display = await navigator.mediaDevices.getDisplayMedia({ video: true, audio: false });
    const track = display.getVideoTracks()[0];
    if (!track) throw new Error('没有可共享的屏幕视频轨。');
    const sender = connection.getSenders().find((candidate) => candidate.track?.kind === 'video');
    if (sender) await sender.replaceTrack(track);
    else connection.addTrack(track, display);
    track.addEventListener('ended', () => { void this.restoreCameraTrack(); }, { once: true });
    await this.sendMediaState({ screenSharing: true, videoEnabled: true });
  }

  async dispose(): Promise<void> {
    this.disposed = true;
    this.unsubscribe?.();
    this.unsubscribe = null;
    await this.hangup('dispose');
    try { await window.fabushiNative?.invoke('disconnectMessagingSignaling'); } catch { /* app is closing */ }
    this.signaling = null;
  }

  private async handleSignal(raw: CallSignal): Promise<void> {
    if (this.disposed || !raw || typeof raw !== 'object' || typeof raw.type !== 'string') return;
    const identity = await this.connect().catch(() => null);
    if (!identity) return;
    switch (raw.type) {
      case 'invite': {
        if (raw.from === identity.actorId || this.activeCall) return;
        if (!['voice', 'video'].includes(raw.kind)) return;
        this.pendingInvite = raw;
        await this.send({ type: 'ringing', callId: raw.callId, actorId: identity.actorId });
        this.callbacks.onIncoming?.({
          callId: raw.callId,
          conversationId: raw.conversationId,
          fromActorId: raw.from,
          kind: raw.kind as FabushiCallKind,
        });
        this.callbacks.onStatus?.('ringing');
        return;
      }
      case 'ringing':
        if (raw.callId === this.activeCall?.callId) this.callbacks.onStatus?.('ringing');
        return;
      case 'accept':
        if (!this.activeCall || this.activeCall.incoming || raw.callId !== this.activeCall.callId) return;
        await this.preparePeer(this.activeCall);
        await this.createAndSendOffer();
        return;
      case 'decline':
        if (raw.callId !== this.activeCall?.callId) return;
        this.cleanupCall();
        this.callbacks.onStatus?.('ended', raw.reason);
        return;
      case 'sdpOffer':
        if (!this.activeCall || raw.callId !== this.activeCall.callId || raw.to !== identity.actorId) return;
        await this.preparePeer(this.activeCall);
        await this.peerConnection!.setRemoteDescription({ type: 'offer', sdp: raw.sdp });
        await this.flushPendingIce();
        {
          const answer = await this.peerConnection!.createAnswer();
          await this.peerConnection!.setLocalDescription(answer);
          await this.send({
            type: 'sdpAnswer',
            callId: raw.callId,
            from: identity.actorId,
            to: raw.from,
            sdp: answer.sdp ?? '',
          });
        }
        this.callbacks.onStatus?.('connecting');
        return;
      case 'sdpAnswer':
        if (!this.peerConnection || !this.activeCall || raw.callId !== this.activeCall.callId || raw.to !== identity.actorId) return;
        await this.peerConnection.setRemoteDescription({ type: 'answer', sdp: raw.sdp });
        await this.flushPendingIce();
        return;
      case 'iceCandidate': {
        if (!this.activeCall || raw.callId !== this.activeCall.callId) return;
        if (raw.to && raw.to !== identity.actorId) return;
        const candidate: RTCIceCandidateInit = {
          candidate: raw.candidate,
          sdpMid: raw.sdpMid ?? null,
          sdpMLineIndex: raw.sdpMlineIndex ?? null,
        };
        if (!this.peerConnection?.remoteDescription) this.pendingIce.push(candidate);
        else await this.peerConnection.addIceCandidate(candidate);
        return;
      }
      case 'hangup':
        if (raw.callId !== this.activeCall?.callId) return;
        this.cleanupCall();
        this.callbacks.onStatus?.('ended', raw.reason);
        return;
      case 'mediaState':
      case 'speaking':
        return;
    }
  }

  private async preparePeer(call: ActiveFabushiCall): Promise<void> {
    if (this.peerConnection) return;
    const signaling = await this.connect();
    this.localStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: call.kind === 'video' });
    const connection = new RTCPeerConnection({ iceServers: signaling.iceServers ?? [] });
    this.peerConnection = connection;
    for (const track of this.localStream.getTracks()) connection.addTrack(track, this.localStream);
    connection.onicecandidate = (event) => {
      if (!event.candidate || !this.activeCall) return;
      void this.send({
        type: 'iceCandidate',
        callId: this.activeCall.callId,
        from: this.activeCall.actorId,
        to: this.activeCall.peerActorId,
        candidate: event.candidate.candidate,
        sdpMid: event.candidate.sdpMid,
        sdpMlineIndex: event.candidate.sdpMLineIndex,
      });
    };
    connection.ontrack = (event) => {
      const stream = event.streams[0] ?? this.remoteStream ?? new MediaStream();
      if (!event.streams[0]) stream.addTrack(event.track);
      this.remoteStream = stream;
      this.callbacks.onRemoteStream?.(stream);
    };
    connection.onconnectionstatechange = () => {
      if (connection.connectionState === 'connected') this.callbacks.onStatus?.('active');
      else if (['failed', 'disconnected'].includes(connection.connectionState)) this.callbacks.onStatus?.('failed', connection.connectionState);
      else if (connection.connectionState === 'connecting') this.callbacks.onStatus?.('connecting');
    };
    this.callbacks.onStatus?.('connecting');
  }

  private async createAndSendOffer(): Promise<void> {
    const call = this.activeCall;
    const connection = this.peerConnection;
    if (!call || !connection) return;
    const offer = await connection.createOffer();
    await connection.setLocalDescription(offer);
    await this.send({
      type: 'sdpOffer',
      callId: call.callId,
      from: call.actorId,
      to: call.peerActorId,
      sdp: offer.sdp ?? '',
    });
  }

  private async flushPendingIce(): Promise<void> {
    const connection = this.peerConnection;
    if (!connection?.remoteDescription) return;
    const pending = this.pendingIce.splice(0);
    for (const candidate of pending) await connection.addIceCandidate(candidate);
  }

  private async sendMediaState(patch: Partial<{ muted: boolean; videoEnabled: boolean; screenSharing: boolean }>): Promise<void> {
    const call = this.activeCall;
    if (!call) return;
    const audio = this.localStream?.getAudioTracks()[0];
    const video = this.localStream?.getVideoTracks()[0];
    await this.send({
      type: 'mediaState',
      callId: call.callId,
      actorId: call.actorId,
      muted: patch.muted ?? (audio ? !audio.enabled : true),
      videoEnabled: patch.videoEnabled ?? Boolean(video?.enabled),
      screenSharing: patch.screenSharing ?? false,
    });
  }

  private async restoreCameraTrack(): Promise<void> {
    const call = this.activeCall;
    const connection = this.peerConnection;
    if (!call || !connection || call.kind !== 'video') return;
    const camera = await navigator.mediaDevices.getUserMedia({ video: true, audio: false });
    const track = camera.getVideoTracks()[0];
    if (!track) return;
    const sender = connection.getSenders().find((candidate) => candidate.track?.kind === 'video');
    if (sender) await sender.replaceTrack(track);
    this.localStream?.getVideoTracks().forEach((old) => old.stop());
    if (this.localStream) this.localStream.addTrack(track);
    await this.sendMediaState({ screenSharing: false, videoEnabled: true });
  }

  private send(signal: CallSignal): Promise<unknown> {
    const bridge = window.fabushiNative;
    if (!bridge?.invoke) return Promise.reject(new Error('Fabushi 原生信令桥不可用。'));
    return bridge.invoke('sendMessagingSignal', { signal });
  }

  private cleanupCall(): void {
    this.peerConnection?.close();
    this.peerConnection = null;
    this.localStream?.getTracks().forEach((track) => track.stop());
    this.localStream = null;
    this.remoteStream = null;
    this.pendingIce = [];
    this.activeCall = null;
    this.callbacks.onRemoteStream?.(null);
  }
}
