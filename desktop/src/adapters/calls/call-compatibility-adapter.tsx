import { Mic, Phone, PhoneCall, Radio, Video } from 'lucide-react';
import React from 'react';
import type { CompatibilitySection } from '../../agent-workspace/messenger-compatibility-adapter';
import type { LocalCall } from '../legacy-messaging/legacy-messaging-model';
import FabAvatar from '../../ui/avatar/fab-avatar';
import { FabIconButton } from '../../ui/primitives/fab-primitives';
import styles from '../../messaging-shell.module.css';
import extra from '../legacy-messaging/legacy-messaging-shell.module.css';

export function CallDialog({
  call,
  localVideoRef,
  remoteVideoRef,
  remoteAudioRef,
  canAccept,
  onAccept,
  onDecline,
  onMute,
  onVideo,
  onShare,
  onEnd,
}: {
  call: LocalCall;
  localVideoRef: React.RefObject<HTMLVideoElement | null>;
  remoteVideoRef: React.RefObject<HTMLVideoElement | null>;
  remoteAudioRef: React.RefObject<HTMLAudioElement | null>;
  canAccept: boolean;
  onAccept: () => void;
  onDecline: () => void;
  onMute: () => void;
  onVideo: () => void;
  onShare: () => void;
  onEnd: () => void;
}) {
  const statusText = call.status === 'ringing'
    ? call.incoming ? '来电…' : '正在呼叫…'
    : call.status === 'connecting'
      ? '正在建立端到端媒体连接…'
      : call.status === 'active'
        ? `${call.kind === 'video' ? '视频' : '语音'}通话中`
        : call.status === 'failed'
          ? '通话连接失败'
          : '通话已结束';
  return <div className={styles.backdrop}><section className={styles.callDialog}>
    <header><FabAvatar identity={`call:${call.title}`} state={call.status === 'active' ? 'speaking' : 'listening'} size={78} label={call.title} /><strong>{call.title}</strong><small>{statusText}</small></header>
    {call.kind === 'video' ? <div className={extra.callVideoStage}><video ref={remoteVideoRef} autoPlay playsInline className={extra.remoteVideo} /><video ref={localVideoRef} autoPlay muted playsInline className={extra.localVideoPip} /></div> : <audio ref={remoteAudioRef} autoPlay />}
    {call.error ? <p>{call.error}</p> : null}
    {canAccept && call.incoming && call.status === 'ringing' ? <div className={styles.callActions}>
      <FabIconButton label="拒绝通话" onClick={onDecline} className={styles.hangup}><Phone size={20} /></FabIconButton>
      <FabIconButton label="接听通话" onClick={onAccept}><PhoneCall size={20} /></FabIconButton>
    </div> : <div className={styles.callActions}>
      <FabIconButton label={call.muted ? '取消静音' : '静音'} data-active={call.muted || undefined} onClick={onMute}><Mic size={20} /></FabIconButton>
      {call.kind === 'video' ? <FabIconButton label="摄像头" data-active={!call.videoEnabled || undefined} onClick={onVideo}><Video size={20} /></FabIconButton> : null}
      {call.kind === 'video' ? <FabIconButton label="共享屏幕" onClick={onShare}><Radio size={20} /></FabIconButton> : null}
      <FabIconButton label="结束通话" className={styles.hangup} onClick={onEnd}><Phone size={21} /></FabIconButton>
    </div>}
    <p className={styles.callNote}>Fabushi 自建信令 + WebRTC 一对一媒体；远端信令强制 TLS，NAT 穿透使用部署方配置的自建 STUN/TURN。</p>
  </section></div>;
}

export default function CallCompatibilityAdapter({
  section,
  children,
}: {
  readonly section: CompatibilitySection;
  readonly children: React.ReactNode;
}) {
  return section === 'calls' ? <>{children}</> : null;
}
