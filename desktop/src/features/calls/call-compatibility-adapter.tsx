import React from 'react';
import { Mic, Phone, PhoneCall, Radio, Video } from 'lucide-react';
import type { LocalCall } from '../../adapters/compatibility/compatibility-model';
import FabAvatar from '../../ui/avatar/fab-avatar';
import styles from '../../messaging-shell.module.css';
import extra from '../../adapters/compatibility/compatibility.module.css';

export function CallCompatibilityAdapter({
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
    <header><FabAvatar identity={`call:${call.title}`} state={call.status === "active" ? "speaking" : "listening"} size={78} label={call.title} /><strong>{call.title}</strong><small>{statusText}</small></header>
    {call.kind === 'video' ? <div className={extra.callVideoStage}><video ref={remoteVideoRef} autoPlay playsInline className={extra.remoteVideo} /><video ref={localVideoRef} autoPlay muted playsInline className={extra.localVideoPip} /></div> : <audio ref={remoteAudioRef} autoPlay />}
    {call.error ? <p>{call.error}</p> : null}
    {canAccept && call.incoming && call.status === 'ringing' ? <div className={styles.callActions}><button type="button" onClick={onDecline} className={styles.hangup}><Phone size={20} /></button><button type="button" onClick={onAccept}><PhoneCall size={20} /></button></div> : <div className={styles.callActions}>
      <button type="button" data-active={call.muted} onClick={onMute} title={call.muted ? '取消静音' : '静音'}><Mic size={20} /></button>
      {call.kind === 'video' ? <button type="button" data-active={!call.videoEnabled} onClick={onVideo} title="摄像头"><Video size={20} /></button> : null}
      {call.kind === 'video' ? <button type="button" onClick={onShare} title="共享屏幕"><Radio size={20} /></button> : null}
      <button type="button" className={styles.hangup} onClick={onEnd}><Phone size={21} /></button>
    </div>}
    <p className={styles.callNote}>Fabushi 自建信令 + WebRTC 一对一媒体；远端信令强制 TLS，NAT 穿透使用部署方配置的自建 STUN/TURN。</p>
  </section></div>;
}

