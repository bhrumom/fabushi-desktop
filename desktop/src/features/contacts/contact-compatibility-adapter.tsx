import React from 'react';
import { Archive, BellOff, FileText, Image, Link2, PhoneCall, Pin, Search, Video, X } from 'lucide-react';
import type { CompatibilityPeerItem } from '../../adapters/compatibility/messaging-compatibility-adapter';
import type { InfoTab } from '../../adapters/compatibility/compatibility-model';
import FabAvatar from '../../ui/avatar/fab-avatar';
import styles from '../../messaging-shell.module.css';

export interface ContactCompatibilityAdapterProps {
  readonly peer: CompatibilityPeerItem;
  readonly muted: boolean;
  readonly infoTab: InfoTab;
  readonly overlay: boolean;
  onClose(): void;
  onVoiceCall(): void;
  onVideoCall(): void;
  onSearch(): void;
  onToggleMute(): void;
  onTogglePin(): void;
  onToggleArchive(): void;
  onInfoTab(tab: InfoTab): void;
}

export function ContactCompatibilityAdapter(props: ContactCompatibilityAdapterProps) {
  const { peer } = props;
  return <aside className={styles.infoPanel} data-testid="contact-compatibility-panel" data-overlay={props.overlay || undefined}>
    <header><strong>联系人资料</strong><button type="button" onClick={props.onClose}><X size={17} /></button></header>
    <div className={styles.profileCard}>
      <FabAvatar identity={`contact:${peer.actorId ?? peer.id}`} state="idle" size={92} className={styles.agentProfileMark} label={peer.title} />
      <strong>{peer.title}</strong><small>{peer.subtitle}</small>
      <div className={styles.profileQuickActions} data-columns="3">
        <button type="button" onClick={props.onVoiceCall}><PhoneCall size={18} /><span>通话</span></button>
        <button type="button" onClick={props.onVideoCall}><Video size={18} /><span>视频</span></button>
        <button type="button" onClick={props.onSearch}><Search size={18} /><span>搜索</span></button>
      </div>
    </div>
    <div className={styles.profileActions}>
      <button type="button" onClick={props.onToggleMute}><BellOff size={17} /><span>{props.muted ? '开启通知' : '静音通知'}</span></button>
      <button type="button" onClick={props.onTogglePin}><Pin size={17} /><span>{peer.pinned ? '取消置顶' : '置顶会话'}</span></button>
      <button type="button" onClick={props.onToggleArchive}><Archive size={17} /><span>{peer.archived ? '移出归档' : '归档会话'}</span></button>
    </div>
    <nav className={styles.infoTabs}>
      <button type="button" data-active={props.infoTab === 'media'} onClick={() => props.onInfoTab('media')}>媒体</button>
      <button type="button" data-active={props.infoTab === 'files'} onClick={() => props.onInfoTab('files')}>文件</button>
      <button type="button" data-active={props.infoTab === 'links'} onClick={() => props.onInfoTab('links')}>链接</button>
    </nav>
    <div className={styles.infoContent}>
      {props.infoTab === 'media' ? <><Image size={30} /><strong>共享媒体</strong><p>图片、视频和动画按消息索引展示。</p></> : null}
      {props.infoTab === 'files' ? <><FileText size={30} /><strong>共享文件</strong><p>文档、音频和附件由 Rust 媒体层管理。</p></> : null}
      {props.infoTab === 'links' ? <><Link2 size={30} /><strong>共享链接</strong><p>富文本 URL 建立可搜索索引。</p></> : null}
    </div>
  </aside>;
}
