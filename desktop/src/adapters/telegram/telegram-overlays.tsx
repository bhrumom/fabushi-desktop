import React, { useEffect, useState } from 'react';
import {
  Check,
  Copy,
  Edit3,
  FileText,
  Forward,
  Image,
  MapPin,
  Pin,
  Plus,
  Reply,
  Smile,
  SquarePen,
  Trash2,
  UserPlus,
  WalletCards,
  X,
} from 'lucide-react';
import type { BotSummary } from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type {
  MessagingActor,
  MessagingCommunityMember,
  MessagingCommunityState,
  MessagingForumTopic,
  MessagingMediaRef,
  MessagingStory,
} from '../../selfhosted-messaging-client-v2';
import type {
  DisplayMessage,
  MessageMenu,
  NewDialog,
} from '../compatibility/compatibility-model';
import type { CompatibilityPeerItem as PeerItem } from '../compatibility/messenger-compatibility-adapter';
import { FabAvatar } from '../../ui/primitives/fab-primitives';
import styles from '../../messaging-shell.module.css';
import extra from '../../agent-workspace/agent-root-shell.module.css';

function blobMediaUrl(media?: MessagingMediaRef): string | undefined {
  if (!media?.id) return undefined;
  return `fabushi-blob://${encodeURIComponent(media.id)}`;
}

export function AttachmentMenu({ onMedia, onFile, onPoll, onLocation, onSchedule }: { onMedia: () => void; onFile: () => void; onPoll: () => void; onLocation: () => void; onSchedule: () => void }) {
  return <div className={extra.popover} onClick={(event) => event.stopPropagation()}>
    <button type="button" onClick={onMedia}><Image size={17} />图片或视频</button>
    <button type="button" onClick={onFile}><FileText size={17} />文件</button>
    <button type="button" onClick={onPoll}><span>📊</span>投票</button>
    <button type="button" onClick={onLocation}><MapPin size={17} />位置</button>
    <button type="button" onClick={onSchedule}><span>⏱</span>定时发送</button>
  </div>;
}

export function MessageContextMenu({ menu, onAction }: { menu: NonNullable<MessageMenu>; onAction: (action: 'copy' | 'reply' | 'forward' | 'checkout' | 'edit' | 'delete' | 'react' | 'pin') => void }) {
  return <div className={extra.contextMenu} data-testid="message-context-menu" style={{ left: menu.x, top: menu.y }} onClick={(event) => event.stopPropagation()}>
    <button type="button" data-testid="message-action-reply" onClick={() => onAction('reply')}><Reply size={16} />回复</button>
    <button type="button" data-testid="message-action-copy" onClick={() => onAction('copy')}><Copy size={16} />复制</button>
    <button type="button" data-testid="message-action-react" onClick={() => onAction('react')}><Smile size={16} />👍 反应</button>
    {menu.message.role === 'me' ? <button type="button" data-testid="message-action-edit" onClick={() => onAction('edit')}><Edit3 size={16} />编辑</button> : null}
    <button type="button" data-testid="message-action-pin" onClick={() => onAction('pin')}><Pin size={16} />{menu.message.pinned ? '取消置顶' : '置顶'}</button>
    <button type="button" data-testid="message-action-forward" onClick={() => onAction('forward')}><Forward size={16} />转发</button>
    {menu.message.invoiceId ? <button type="button" data-testid="message-action-checkout" onClick={() => onAction('checkout')}><WalletCards size={16} />支付账单</button> : null}
    <button type="button" data-testid="message-action-delete" onClick={() => onAction('delete')}><Trash2 size={16} />删除</button>
  </div>;
}

export function ForwardMessageDialog({ message, peers, onClose, onSelect }: { message: DisplayMessage; peers: PeerItem[]; onClose: () => void; onSelect: (peer: PeerItem) => void }) {
  return <div className={styles.backdrop} data-testid="forward-message-dialog" onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>转发消息</strong><small>{message.text || '媒体消息'}</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <div className={extra.forwardList}>
      {peers.length ? peers.map((peer) => <button key={peer.key} type="button" data-agent-id={`forward-message-peer:${peer.key}`} onClick={() => onSelect(peer)}><FabAvatar identity={`peer:${peer.kind}:${peer.actorId ?? peer.id}`} state="idle" size={40} label={peer.title} /><span><strong>{peer.title}</strong><small>{peer.subtitle}</small></span><Forward size={16} /></button>) : <p>暂无可转发的自建会话，请先创建群组、频道或收藏消息。</p>}
    </div>
  </section></div>;
}

export function EditMessageDialog({ value, onChange, onClose, onSave }: { value: string; onChange: (value: string) => void; onClose: () => void; onSave: () => void }) {
  return <div className={styles.backdrop} data-testid="edit-message-dialog" onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>编辑消息</strong><small>修改后会同步到 Fabushi 自建会话</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <label><span>消息内容</span><textarea autoFocus data-testid="edit-message-input" value={value} onChange={(event) => onChange(event.target.value)} rows={4} placeholder="编辑消息内容" /></label>
    <footer><button type="button" data-testid="edit-message-cancel" onClick={onClose}>取消</button><button type="button" data-testid="edit-message-save" className={styles.primaryButton} disabled={!value.trim()} onClick={onSave}>保存</button></footer>
  </section></div>;
}

export function NewConversationDialog({ dialog, bots, onChange, onClose, onSave }: { dialog: Exclude<NewDialog, null>; bots: BotSummary[]; onChange: React.Dispatch<React.SetStateAction<NewDialog>>; onClose: () => void; onSave: () => void }) {
  return <div className={styles.backdrop} onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>{dialog.type === 'group' ? '新建群组' : '新建频道'}</strong><small>{dialog.type === 'group' ? '现有 AI 群组 Host 会执行 Bot 多轮协作' : 'Fabushi 自建广播会话'}</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <label><span>名称</span><input autoFocus value={dialog.name} onChange={(event) => onChange((current) => current ? { ...current, name: event.target.value } : current)} placeholder={dialog.type === 'group' ? '群组名称' : '频道名称'} /></label>
    {dialog.type === 'channel' ? <label><span>描述</span><textarea value={dialog.description} onChange={(event) => onChange((current) => current?.type === 'channel' ? { ...current, description: event.target.value } : current)} rows={3} placeholder="频道简介" /></label> : null}
    {dialog.type === 'group' ? <div className={styles.memberPicker}><span>选择 AI Bot</span>{bots.map((bot) => {
      const selected = dialog.selectedBotIds.has(bot.id);
      return <button key={bot.id} type="button" data-testid={`group-bot-${bot.id}`} data-selected={selected} onClick={() => onChange((current) => {
        if (!current || current.type !== 'group') return current;
        const selectedBotIds = new Set(current.selectedBotIds);
        if (selectedBotIds.has(bot.id)) selectedBotIds.delete(bot.id); else selectedBotIds.add(bot.id);
        return { ...current, selectedBotIds };
      })}><FabAvatar identity={`bot:${bot.id}`} state="idle" size={34} label={bot.name} /><div><strong>{bot.name}</strong><small>{bot.description}</small></div>{selected ? <Check size={16} /> : <Plus size={16} />}</button>;
    })}</div> : null}
    <footer><button type="button" onClick={onClose}>取消</button><button type="button" className={styles.primaryButton} disabled={!dialog.name.trim() || (dialog.type === 'group' && dialog.selectedBotIds.size === 0)} onClick={onSave}>{dialog.type === 'group' ? '创建群组' : '创建频道'}</button></footer>
  </section></div>;
}

export function CommunityAdminDialog({
  peer,
  community,
  actors,
  actorId,
  onClose,
  onSave,
  onSetMember,
  onCreateInvite,
  onRevokeInvite,
  onJoinDecision,
  onUpsertTopic,
  onDeleteTopic,
}: {
  peer: PeerItem;
  community: MessagingCommunityState;
  actors: MessagingActor[];
  actorId: string;
  onClose: () => void;
  onSave: (community: MessagingCommunityState) => void;
  onSetMember: (conversationId: string, member: MessagingCommunityMember) => void;
  onCreateInvite: (community: MessagingCommunityState) => void;
  onRevokeInvite: (conversationId: string, inviteId: string) => void;
  onJoinDecision: (conversationId: string, requesterId: string, approved: boolean) => void;
  onUpsertTopic: (topic: MessagingForumTopic) => void;
  onDeleteTopic: (conversationId: string, topicId: string) => void;
}) {
  const [draft, setDraft] = useState(community);
  useEffect(() => setDraft(community), [community]);
  const actorName = (id: string) => actors.find((actor) => actor.id === id)?.displayName ?? id;
  const members = Object.values(draft.members);
  const invites = Object.values(draft.inviteLinks).filter((invite) => !invite.revoked);
  const requests = Object.values(draft.pendingJoinRequests);
  const topics = Object.values(draft.topics);
  const canManage = draft.members[actorId]?.status === 'owner' || draft.members[actorId]?.status === 'administrator';

  function changeMemberRole(member: MessagingCommunityMember, status: MessagingCommunityMember['status']) {
    const next: MessagingCommunityMember = {
      ...member,
      status,
      adminRights: status === 'administrator' || status === 'owner'
        ? {
            ...member.adminRights,
            changeInfo: true,
            deleteMessages: true,
            banMembers: true,
            inviteMembers: true,
            pinMessages: true,
            manageTopics: true,
            manageCalls: true,
          }
        : member.adminRights,
    };
    onSetMember(draft.conversationId, next);
  }

  function createTopic() {
    const title = window.prompt('Topic 名称');
    if (!title?.trim()) return;
    onUpsertTopic({
      id: `topic:${crypto.randomUUID()}`,
      conversationId: draft.conversationId,
      title: title.trim(),
      creatorId: actorId,
      createdAtMs: Date.now(),
      pinned: false,
      closed: false,
      hidden: false,
      unreadCount: 0,
    });
  }

  return <div className={styles.backdrop} onMouseDown={onClose}><section className={extra.communityDialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>{peer.title} · 管理</strong><small>所有修改都由 Rust Community 权限层校验</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <div className={extra.communityBody}>
      <section>
        <h3>群组 / 频道设置</h3>
        <label><span>公开用户名</span><input value={draft.publicUsername ?? ''} onChange={(event) => setDraft((current) => ({ ...current, publicUsername: event.target.value || undefined }))} disabled={!canManage} placeholder="例如 fabushi" /></label>
        <label><span>慢速模式（秒）</span><input type="number" min={0} max={3600} value={draft.slowModeSeconds ?? 0} onChange={(event) => setDraft((current) => ({ ...current, slowModeSeconds: Number(event.target.value) || undefined }))} disabled={!canManage} /></label>
        <label><span>敏感词</span><textarea value={draft.bannedWords.join(', ')} onChange={(event) => setDraft((current) => ({ ...current, bannedWords: event.target.value.split(',').map((value) => value.trim()).filter(Boolean) }))} disabled={!canManage} rows={2} /></label>
        <div className={extra.communityToggles}>
          <label><input type="checkbox" checked={draft.joinRequestRequired} onChange={(event) => setDraft((current) => ({ ...current, joinRequestRequired: event.target.checked }))} disabled={!canManage} />入群需审批</label>
          <label><input type="checkbox" checked={draft.joinToSend} onChange={(event) => setDraft((current) => ({ ...current, joinToSend: event.target.checked }))} disabled={!canManage} />发言前必须入群</label>
          <label><input type="checkbox" checked={draft.signaturesEnabled} onChange={(event) => setDraft((current) => ({ ...current, signaturesEnabled: event.target.checked }))} disabled={!canManage} />频道签名</label>
        </div>
        <button type="button" className={styles.primaryButton} disabled={!canManage} onClick={() => onSave(draft)}>保存设置</button>
      </section>

      <section>
        <h3>成员</h3>
        <div className={extra.communityList}>{members.length ? members.map((member) => <div className={extra.communityRow} key={member.actorId}><div><strong>{actorName(member.actorId)}</strong><small>{member.actorId}</small></div><select value={member.status} disabled={!canManage || member.actorId === actorId && member.status === 'owner'} onChange={(event) => changeMemberRole(member, event.target.value as MessagingCommunityMember['status'])}><option value="owner">Owner</option><option value="administrator">Admin</option><option value="member">Member</option><option value="restricted">Restricted</option><option value="banned">Banned</option><option value="left">Left</option></select></div>) : <small>暂无成员记录</small>}</div>
      </section>

      <section>
        <div className={extra.communitySectionTitle}><h3>邀请链接</h3><button type="button" disabled={!canManage} onClick={() => onCreateInvite(draft)}><UserPlus size={15} />创建</button></div>
        <div className={extra.communityList}>{invites.length ? invites.map((invite) => <div className={extra.communityRow} key={invite.id}><div><strong>{invite.name ?? '邀请链接'}</strong><small>{invite.token}</small></div><div><button type="button" onClick={() => void navigator.clipboard.writeText(`fabushi://join/${invite.token}`)}><Copy size={14} /></button><button type="button" disabled={!canManage} onClick={() => onRevokeInvite(draft.conversationId, invite.id)}><Trash2 size={14} /></button></div></div>) : <small>暂无有效邀请链接</small>}</div>
      </section>

      <section>
        <h3>待审批</h3>
        <div className={extra.communityList}>{requests.length ? requests.map((request) => <div className={extra.communityRow} key={request.actorId}><div><strong>{actorName(request.actorId)}</strong><small>{request.bio ?? '请求加入'}</small></div><div><button type="button" disabled={!canManage} onClick={() => onJoinDecision(draft.conversationId, request.actorId, true)}>通过</button><button type="button" disabled={!canManage} onClick={() => onJoinDecision(draft.conversationId, request.actorId, false)}>拒绝</button></div></div>) : <small>暂无待审批成员</small>}</div>
      </section>

      <section>
        <div className={extra.communitySectionTitle}><h3>Forum Topics</h3><button type="button" disabled={!canManage} onClick={createTopic}><SquarePen size={15} />新建</button></div>
        <div className={extra.communityList}>{topics.length ? topics.map((topic) => <div className={extra.communityRow} key={topic.id}><div><strong>{topic.title}</strong><small>{topic.closed ? '已关闭' : topic.pinned ? '已置顶' : '开放'}</small></div><button type="button" disabled={!canManage} onClick={() => onDeleteTopic(draft.conversationId, topic.id)}><Trash2 size={14} /></button></div>) : <small>暂无 Topic</small>}</div>
      </section>
    </div>
  </section></div>;
}

export function StoryViewer({ story, owner, own, onClose, onReact, onDelete }: { story: MessagingStory; owner?: MessagingActor; own: boolean; onClose: () => void; onReact: (reaction: string) => void; onDelete: () => void }) {
  const source = blobMediaUrl(story.media);
  const isVideo = story.media.mimeType?.startsWith('video/') === true;
  return <div className={extra.storyBackdrop} onMouseDown={onClose}>
    <section className={extra.storyViewer} onMouseDown={(event) => event.stopPropagation()}>
      <header><FabAvatar identity={`story:${story.ownerId}`} state="idle" size={40} label={owner?.displayName ?? story.ownerId} /><div><strong>{owner?.displayName ?? story.ownerId}</strong><small>{new Date(story.createdAtMs).toLocaleString()}</small></div><button type="button" onClick={onClose}><X size={18} /></button></header>
      <div className={extra.storyMedia}>{source ? isVideo ? <video src={source} autoPlay controls playsInline /> : <img src={source} alt={story.caption.text || 'Story'} /> : <span>媒体不可用</span>}</div>
      {story.caption.text ? <p>{story.caption.text}</p> : null}
      <footer>{['👍', '❤️', '🔥', '🙏'].map((reaction) => <button key={reaction} type="button" onClick={() => onReact(reaction)}>{reaction}</button>)}{own ? <button type="button" onClick={onDelete}><Trash2 size={16} />删除</button> : null}</footer>
    </section>
  </div>;
}


