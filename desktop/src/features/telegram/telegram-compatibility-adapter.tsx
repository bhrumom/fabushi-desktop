import React, { useEffect, useState } from 'react';
import { Check, Copy, Edit3, FileText, Forward, Image, MapPin, Pin, Plus, Reply, Smile, SquarePen, Trash2, UserPlus, WalletCards, X } from 'lucide-react';
import type { BotSummary } from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { MessagingActor, MessagingCommunityMember, MessagingCommunityState, MessagingForumTopic, MessagingStory } from '../../selfhosted-messaging-client-v2';
import type { DisplayMessage, MessageMenu, NewDialog } from '../../adapters/compatibility/compatibility-model';
import type { CompatibilityPeerItem as PeerItem } from '../../adapters/compatibility/messaging-compatibility-adapter';
import FabAvatar from '../../ui/avatar/fab-avatar';
import styles from '../../messaging-shell.module.css';
import extra from '../../adapters/compatibility/compatibility.module.css';
import { blobMediaUrl } from '../../adapters/compatibility/compatibility-runtime';

export function TelegramAttachmentMenu({ onMedia, onFile, onPoll, onLocation, onSchedule }: { onMedia: () => void; onFile: () => void; onPoll: () => void; onLocation: () => void; onSchedule: () => void }) {
  return <div className={extra.popover} onClick={(event) => event.stopPropagation()}>
    <button type="button" onClick={onMedia}><Image size={17} />图片或视频</button>
    <button type="button" onClick={onFile}><FileText size={17} />文件</button>
    <button type="button" onClick={onPoll}><span>📊</span>投票</button>
    <button type="button" onClick={onLocation}><MapPin size={17} />位置</button>
    <button type="button" onClick={onSchedule}><span>⏱</span>定时发送</button>
  </div>;
}

export function TelegramMessageContextMenu({ menu, onAction }: { menu: NonNullable<MessageMenu>; onAction: (action: 'copy' | 'reply' | 'forward' | 'checkout' | 'edit' | 'delete' | 'react' | 'pin') => void }) {
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

export function TelegramForwardMessageDialog({ message, peers, onClose, onSelect }: { message: DisplayMessage; peers: PeerItem[]; onClose: () => void; onSelect: (peer: PeerItem) => void }) {
  return <div className={styles.backdrop} data-testid="forward-message-dialog" onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>转发消息</strong><small>{message.text || '媒体消息'}</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <div className={extra.forwardList}>
      {peers.length ? peers.map((peer) => <button key={peer.key} type="button" data-agent-id={`forward-message-peer:${peer.key}`} onClick={() => onSelect(peer)}><FabAvatar identity={`peer:${peer.kind}:${peer.actorId ?? peer.id}`} state="idle" size={40} label={peer.title} /><span><strong>{peer.title}</strong><small>{peer.subtitle}</small></span><Forward size={16} /></button>) : <p>暂无可转发的自建会话，请先创建群组、频道或收藏消息。</p>}
    </div>
  </section></div>;
}

export function TelegramEditMessageDialog({ value, onChange, onClose, onSave }: { value: string; onChange: (value: string) => void; onClose: () => void; onSave: () => void }) {
  return <div className={styles.backdrop} data-testid="edit-message-dialog" onMouseDown={onClose}><section className={styles.dialog} onMouseDown={(event) => event.stopPropagation()}>
    <header><div><strong>编辑消息</strong><small>修改后会同步到 Fabushi 自建会话</small></div><button type="button" onClick={onClose}><X size={17} /></button></header>
    <label><span>消息内容</span><textarea autoFocus data-testid="edit-message-input" value={value} onChange={(event) => onChange(event.target.value)} rows={4} placeholder="编辑消息内容" /></label>
    <footer><button type="button" data-testid="edit-message-cancel" onClick={onClose}>取消</button><button type="button" data-testid="edit-message-save" className={styles.primaryButton} disabled={!value.trim()} onClick={onSave}>保存</button></footer>
  </section></div>;
}

export function TelegramNewConversationDialog({ dialog, bots, onChange, onClose, onSave }: { dialog: Exclude<NewDialog, null>; bots: BotSummary[]; onChange: React.Dispatch<React.SetStateAction<NewDialog>>; onClose: () => void; onSave: () => void }) {
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

export function TelegramCommunityAdminDialog({
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

export function TelegramStoryViewer({ story, owner, own, onClose, onReact, onDelete }: { story: MessagingStory; owner?: MessagingActor; own: boolean; onClose: () => void; onReact: (reaction: string) => void; onDelete: () => void }) {
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


type StructuredMessageBlock =
  | { type: 'paragraph'; text: string }
  | { type: 'heading'; level: number; text: string }
  | { type: 'table'; headers: string[]; rows: string[][] }
  | { type: 'sources'; files: string[] };

function markdownTableCells(line: string): string[] {
  return line.trim().replace(/^\|/, '').replace(/\|$/, '').split('|').map((cell) => cell.trim());
}

function isMarkdownTableDivider(line: string): boolean {
  const cells = markdownTableCells(line);
  return cells.length > 1 && cells.every((cell) => /^:?-{3,}:?$/.test(cell));
}

function sourceFileNames(line: string): string[] {
  if (!/^source files?\s*:/i.test(line.trim())) return [];
  const body = line.replace(/^source files?\s*:/i, '');
  const names = new Set<string>();
  for (const match of body.matchAll(/`([^`]+)`/g)) names.add(match[1].trim());
  for (const match of body.matchAll(/\b[\w@./-]+\.(?:md|csv|txt|json|pdf|docx?|xlsx?|png|jpe?g|ts|tsx|js|jsx|rs|swift|kt)\b/gi)) names.add(match[0]);
  return [...names].filter(Boolean);
}

export function parseStructuredMessage(text: string): StructuredMessageBlock[] {
  const lines = text.replace(/\r\n?/g, '\n').split('\n');
  const blocks: StructuredMessageBlock[] = [];
  for (let index = 0; index < lines.length;) {
    const line = lines[index].trim();
    if (!line) { index += 1; continue; }
    const heading = /^(#{1,3})\s+(.+)$/.exec(line);
    if (heading) {
      blocks.push({ type: 'heading', level: heading[1].length, text: heading[2].trim() });
      index += 1;
      continue;
    }
    if (index + 1 < lines.length && line.includes('|') && isMarkdownTableDivider(lines[index + 1])) {
      const headers = markdownTableCells(line);
      const rows: string[][] = [];
      index += 2;
      while (index < lines.length && lines[index].trim().includes('|') && lines[index].trim()) {
        const cells = markdownTableCells(lines[index]);
        if (cells.length !== headers.length) break;
        rows.push(cells);
        index += 1;
      }
      blocks.push({ type: 'table', headers, rows });
      continue;
    }
    const files = sourceFileNames(line);
    if (files.length) {
      blocks.push({ type: 'sources', files });
      index += 1;
      continue;
    }
    const paragraph: string[] = [line];
    index += 1;
    while (index < lines.length) {
      const candidate = lines[index].trim();
      if (!candidate) break;
      if (/^(#{1,3})\s+/.test(candidate)) break;
      if (index + 1 < lines.length && candidate.includes('|') && isMarkdownTableDivider(lines[index + 1])) break;
      if (sourceFileNames(candidate).length) break;
      paragraph.push(candidate);
      index += 1;
    }
    blocks.push({ type: 'paragraph', text: paragraph.join(' ') });
  }
  return blocks;
}

export function TelegramStructuredMessageBody({ text, peers }: { text: string; peers: PeerItem[] }) {
  const blocks = parseStructuredMessage(text);
  const rich = blocks.some((block) => block.type !== 'paragraph');
  if (!rich) return <p>{text}</p>;
  return <div className={extra.structuredMessage} data-testid="structured-message-body">
    {blocks.map((block, blockIndex) => {
      if (block.type === 'heading') {
        return <strong key={`heading-${blockIndex}`} className={extra.structuredHeading} data-level={block.level}>{block.text}</strong>;
      }
      if (block.type === 'table') {
        return <div key={`table-${blockIndex}`} className={extra.structuredTableWrap}>
          <table className={extra.structuredTable} data-testid="assistant-result-table">
            <thead><tr>{block.headers.map((cell, index) => <th key={`${index}:${cell}`}>{cell}</th>)}</tr></thead>
            <tbody>{block.rows.map((row, rowIndex) => <tr key={rowIndex}>{row.map((cell, cellIndex) => {
              const peer = peers.find((candidate) => candidate.title.trim().toLocaleLowerCase() === cell.trim().replace(/^@/, '').toLocaleLowerCase());
              return <td key={`${rowIndex}:${cellIndex}`}>{peer ? <span className={extra.structuredPeerChip} data-testid="assistant-owner-chip"><FabAvatar identity={`peer:${peer.kind}:${peer.actorId ?? peer.id}`} state="result" size={16} label={peer.title} />{cell}</span> : cell}</td>;
            })}</tr>)}</tbody>
          </table>
        </div>;
      }
      if (block.type === 'sources') {
        return <div key={`sources-${blockIndex}`} className={extra.sourceFiles} data-testid="assistant-source-files">
          <span>Source files:</span>
          {block.files.map((file) => <span key={file} className={extra.sourceFileChip}><FileText size={13} />{file}</span>)}
        </div>;
      }
      return <p key={`paragraph-${blockIndex}`}>{block.text}</p>;
    })}
  </div>;
}

