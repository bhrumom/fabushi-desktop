import React, { type ReactNode } from 'react';
import { AppWindow, ArrowDown, Check, Copy, Edit3, FileText, RotateCcw } from 'lucide-react';
import { BotMark } from '../../../frontend/apps/web/src/app/host/bot-mark';
import { MahayanaAssistantTurnView } from '../mahayana-assistant-turn-view';
import type { TranscriptApprovalDecision, TranscriptEntry } from './transcript-model';
import { formatAgentAttachmentSize } from './agent-attachments';
import styles from '../bot-conversation-view.module.css';

export type AgentTranscriptProps = {
  title: string;
  description: string;
  botId: string;
  entries: readonly TranscriptEntry[];
  activeOperationId?: string | null;
  hasEarlierMessages?: boolean;
  messageAreaRef?: React.RefObject<HTMLDivElement | null>;
  showScrollToLatest?: boolean;
  onLoadEarlier?: () => void;
  onOpenMiniApp?: (id: string) => void;
  onScroll?: React.UIEventHandler<HTMLDivElement>;
  onScrollToLatest?: () => void;
  onCopyMessage?: (message: TranscriptEntry) => void;
  onRegenerate?: (message: TranscriptEntry) => void;
  onEdit?: (message: TranscriptEntry) => void;
  onResolveApproval?: (approvalId: string, decision: TranscriptApprovalDecision) => void;
  onContextMenu?: (event: React.MouseEvent<HTMLElement>, message: TranscriptEntry) => void;
};

function renderInline(value: string): ReactNode[] {
  const pattern = /(\*\*[^*]+\*\*|\x60[^\x60]+\x60|\[[^\]]+\]\(https?:\/\/[^)]+\))/g;
  return value.split(pattern).map((part, index) => {
    if (part.startsWith('**') && part.endsWith('**')) return <strong key={index}>{part.slice(2, -2)}</strong>;
    if (part.startsWith('\x60') && part.endsWith('\x60')) return <code key={index}>{part.slice(1, -1)}</code>;
    const link = /^\[([^\]]+)\]\((https?:\/\/[^)]+)\)$/.exec(part);
    return link
      ? <a key={index} href={link[2]} target="_blank" rel="noreferrer">{link[1]}</a>
      : <React.Fragment key={index}>{part}</React.Fragment>;
  });
}

function CodeBlock({ value, language, complete }: { value: string; language: string; complete: boolean }) {
  const [preview, setPreview] = React.useState(false);
  const previewable = complete && /^(html|htm)$/i.test(language);
  const [copied, setCopied] = React.useState(false);
  const copy = async () => {
    try {
      if (!navigator.clipboard) return;
      await navigator.clipboard.writeText(value);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1400);
    } catch {
      // Selectable code remains available when clipboard permission is denied.
    }
  };
  return <div className={styles.codeBlock}>
    <div className={styles.codeToolbar}>
      <span>{language || '代码'}</span>
      {previewable ? <button type="button" onClick={() => setPreview(!preview)} aria-expanded={preview}>{preview ? '关闭预览' : '预览小程序'}</button> : null}
      <button type="button" onClick={() => void copy()} aria-label="复制代码">{copied ? <Check size={14} /> : <Copy size={14} />}{copied ? '已复制' : '复制'}</button>
    </div>
    {preview && previewable ? <iframe className={styles.generatedPreview} title="生成的小程序预览" sandbox="allow-scripts" referrerPolicy="no-referrer" srcDoc={'<!doctype html><meta http-equiv="Content-Security-Policy" content="default-src \'none\'; script-src \'unsafe-inline\'; style-src \'unsafe-inline\'; img-src data: blob:; font-src data:; connect-src \'none\'; form-action \'none\'; base-uri \'none\'">' + value} /> : null}
    <pre><code>{value}</code></pre>
  </div>;
}

function MarkdownContent({ value }: { value: string }) {
  const lines = value.replace(/\r\n/g, '\n').split('\n');
  const blocks: ReactNode[] = [];
  let index = 0;
  while (index < lines.length) {
    if (!lines[index]?.trim()) { index += 1; continue; }
    if (lines[index]?.startsWith('\x60\x60\x60')) {
      const language = lines[index].slice(3).trim();
      index += 1;
      const code: string[] = [];
      while (index < lines.length && !lines[index]?.startsWith('\x60\x60\x60')) {
        code.push(lines[index] ?? '');
        index += 1;
      }
      const complete = index < lines.length;
      if (complete) index += 1;
      blocks.push(<CodeBlock key={blocks.length} value={code.join('\n')} language={language} complete={complete} />);
      continue;
    }
    const paragraph: string[] = [];
    while (index < lines.length && lines[index]?.trim() && !lines[index]?.startsWith('\x60\x60\x60')) {
      paragraph.push(lines[index] ?? '');
      index += 1;
    }
    const first = paragraph[0] ?? '';
    const heading = /^(#{1,3})\s+(.+)$/.exec(first);
    if (heading && paragraph.length === 1) {
      const content = renderInline(heading[2]);
      blocks.push(heading[1].length === 1
        ? <h2 key={blocks.length}>{content}</h2>
        : heading[1].length === 2
          ? <h3 key={blocks.length}>{content}</h3>
          : <h4 key={blocks.length}>{content}</h4>);
      continue;
    }
    const listItems = paragraph
      .filter((line) => /^(\s*[-*]|\s*\d+\.)\s+/.test(line))
      .map((line) => line.replace(/^\s*(?:[-*]|\d+\.)\s+/, ''));
    if (listItems.length === paragraph.length) {
      const List = /^\s*\d+\./.test(first) ? 'ol' : 'ul';
      blocks.push(<List key={blocks.length}>{listItems.map((item, itemIndex) => <li key={itemIndex}>{renderInline(item)}</li>)}</List>);
      continue;
    }
    blocks.push(<p key={blocks.length}>{paragraph.map((line, lineIndex) => <React.Fragment key={lineIndex}>{lineIndex ? <br /> : null}{renderInline(line)}</React.Fragment>)}</p>);
  }
  return <>{blocks}</>;
}

function formatTime(timestamp: number): string {
  return new Intl.DateTimeFormat('zh-CN', { hour: '2-digit', minute: '2-digit' }).format(timestamp);
}

function MessageActions({ entry, onCopyMessage, onRegenerate, onEdit }: {
  entry: TranscriptEntry;
  onCopyMessage?: (message: TranscriptEntry) => void;
  onRegenerate?: (message: TranscriptEntry) => void;
  onEdit?: (message: TranscriptEntry) => void;
}) {
  if (!onCopyMessage && !onRegenerate && !onEdit) return null;
  return <div className={styles.messageActions} role="toolbar" aria-label="消息操作">
    {onCopyMessage ? <button type="button" onClick={() => onCopyMessage(entry)} aria-label="复制消息" title="复制"><Copy size={15} /></button> : null}
    {entry.role === 'peer' && onRegenerate ? <button type="button" onClick={() => onRegenerate(entry)} aria-label="重新生成" title="重新生成"><RotateCcw size={15} /></button> : null}
    {entry.role === 'me' && onEdit ? <button type="button" onClick={() => onEdit(entry)} aria-label="编辑消息" title="编辑并重试"><Edit3 size={15} /></button> : null}
  </div>;
}

function ToolGroup({ entries }: { entries: TranscriptEntry[] }) {
  const running = entries.some((entry) => entry.status === 'running');
  const failed = entries.some((entry) => entry.status === 'failed');
  const interrupted = entries.some((entry) => entry.status === 'interrupted');
  const completed = entries.filter((entry) => entry.status === 'completed').length;
  const [expanded, setExpanded] = React.useState<boolean | null>(null);
  const open = expanded ?? running;
  return <section className={styles.stepGroup} aria-label="任务步骤" data-testid="agent-step-group">
    <button type="button" className={styles.stepSummary} aria-expanded={open} onClick={() => setExpanded(!open)}>
      <span>{running ? '正在执行' : failed ? '执行失败' : interrupted ? '已停止' : '执行记录'}</span>
      <span>{completed} / {entries.length} 步完成 · {open ? '收起' : '展开'}</span>
    </button>
    {open ? <ol className={styles.stepList}>{entries.map((entry) => <li key={entry.id} className={styles.actionRow} data-testid="agent-step" data-operation-id={entry.operationId} data-status={entry.status}>
      <span className={styles.actionMarker} aria-hidden="true" />
      <div><strong>{entry.title || '处理任务'}</strong>{entry.detail ? <details><summary>查看详情</summary><pre>{entry.detail}</pre></details> : null}</div>
      <span className={styles.actionStatus}>{entry.status === 'running' ? '进行中' : entry.status === 'failed' ? '失败' : entry.status === 'interrupted' ? '已停止' : '完成'}</span>
    </li>)}</ol> : null}
  </section>;
}

function ApprovalCard({ entry, onResolveApproval }: {
  entry: TranscriptEntry;
  onResolveApproval?: (approvalId: string, decision: TranscriptApprovalDecision) => void;
}) {
  const approval = entry.approval;
  if (!approval) return null;
  const pending = !approval.decision;
  const decisionLabel = approval.decision === 'allow-once'
    ? 'Allowed once'
    : approval.decision === 'allow-session'
      ? 'Allowed for this session'
      : approval.decision === 'deny'
        ? 'Denied'
        : null;
  return <section
    className={styles.approvalCard}
    data-testid="agent-approval"
    data-approval-id={approval.approvalId}
    data-operation-id={entry.operationId}
    data-status={pending ? 'pending' : 'resolved'}
  >
    <div className={styles.approvalHeader}>
      <div><strong>{entry.title || approval.subject || 'Permission required'}</strong><span>{approval.capability}</span></div>
      <span className={styles.approvalState}>{pending ? 'Needs attention' : decisionLabel}</span>
    </div>
    <p>{approval.detail || approval.reason}</p>
    {approval.location ? <small>Location: {approval.location}</small> : null}
    {pending && onResolveApproval ? <div className={styles.approvalActions} role="group" aria-label="Approval actions">
      <button type="button" onClick={() => onResolveApproval(approval.approvalId, 'allow-once')}>Allow once</button>
      <button type="button" onClick={() => onResolveApproval(approval.approvalId, 'allow-session')}>Allow for session</button>
      <button type="button" data-danger="true" onClick={() => onResolveApproval(approval.approvalId, 'deny')}>Deny</button>
    </div> : null}
  </section>;
}

export default function AgentTranscript({
  title, description, botId, entries, activeOperationId, hasEarlierMessages = false,
  messageAreaRef, showScrollToLatest = false, onLoadEarlier, onOpenMiniApp, onScroll,
  onScrollToLatest, onCopyMessage, onRegenerate, onEdit, onResolveApproval, onContextMenu,
}: AgentTranscriptProps) {
  return <div className={styles.root}>
    <div ref={messageAreaRef} className={styles.messageArea} data-testid="message-list" data-agent-operation-id={activeOperationId ?? undefined} onScroll={onScroll} aria-label={title + ' 会话'}>
      <div className={styles.botIntro}><BotMark botId={botId} state="idle" size={44} label={title} /><div><h2>{title}</h2><p>{description || 'Agent 会在这个独立工作区中持续工作。'}</p></div></div>
      {hasEarlierMessages && onLoadEarlier ? <button type="button" className={styles.loadEarlier} onClick={onLoadEarlier}>查看更早的消息</button> : null}
      {entries.length === 0 ? <div className={styles.emptyState}><strong>开始一个新会话</strong><span>向 {title} 提问，回复、工具和授权会按一个连续时间线显示。</span></div> : <div className={styles.transcript}>
        {entries.map((entry, index) => {
          if (entry.kind === 'approval' && entry.approval) {
            return <ApprovalCard key={entry.id} entry={entry} onResolveApproval={onResolveApproval} />;
          }
          if (entry.kind === 'assistant-turn' && entry.assistantTurn) {
            return <div key={entry.id} data-transcript-entry-id={entry.id}>
              <MahayanaAssistantTurnView turn={entry.assistantTurn} label={title} avatar={<BotMark botId={botId} state={entry.streaming ? 'writing' : 'idle'} size={28} label={title} />} />
            </div>;
          }
          if (entry.kind === 'tool-call') {
            const previous = entries[index - 1];
            if (previous?.kind === 'tool-call' && previous.operationId === entry.operationId) return null;
            const steps: TranscriptEntry[] = [];
            for (let i = index; i < entries.length; i += 1) {
              const step = entries[i];
              if (step.kind !== 'tool-call' || step.operationId !== entry.operationId) break;
              steps.push(step);
            }
            return <div key={entry.id} data-transcript-entry-id={entry.id}><ToolGroup entries={steps} /></div>;
          }
          if (entry.kind === 'thinking') {
            return <div key={entry.id} className={styles.thinkingRow} data-testid="agent-thinking" data-transcript-entry-id={entry.id} data-operation-id={entry.operationId}>
              <BotMark botId={botId} state="thinking" size={28} label={title} />
              <div className={styles.thinkingCopy}><strong>{entry.title || '正在思考'}</strong><span>{entry.detail || '正在整理回复…'}</span></div>
              <span className={styles.thinkingDots} aria-hidden="true"><i /><i /><i /></span>
            </div>;
          }
          const userMessage = entry.role === 'me';
          return <article key={entry.id} className={styles.messageRow + ' ' + (userMessage ? styles.userRow : styles.assistantRow)} data-message-id={entry.id} data-transcript-entry-id={entry.id} data-agent-message-role={entry.role} data-operation-id={entry.operationId} onContextMenu={(event) => onContextMenu?.(event, entry)}>
            {!userMessage ? <BotMark botId={botId} state={entry.streaming ? 'writing' : 'idle'} size={28} label={title} /> : null}
            <div className={styles.messageColumn}>
              <div className={styles.messageMeta}><span>{userMessage ? '你' : title}</span><time dateTime={new Date(entry.createdAtMs).toISOString()}>{formatTime(entry.createdAtMs)}</time></div>
              {entry.text || entry.streaming ? <div className={styles.messageBody}><MarkdownContent value={entry.text || ' '} />{entry.streaming ? <span className={styles.streamingCursor} aria-label="正在生成" /> : null}</div> : null}
              {entry.attachments?.length ? <div className={styles.messageAttachments} aria-label="Attachments">
                {entry.attachments.map((attachment) => <span key={attachment.id} className={styles.messageAttachment}>
                  <FileText size={15} />
                  <span><strong>{attachment.name}</strong>{attachment.sizeBytes != null ? <small>{formatAgentAttachmentSize(attachment.sizeBytes)}</small> : null}</span>
                </span>)}
              </div> : null}
              {entry.miniAppId && onOpenMiniApp ? <button type="button" className={styles.artifactCard} data-testid="bot-miniapp-result" onClick={() => onOpenMiniApp(entry.miniAppId!)}><AppWindow size={22} /><span><strong>打开小程序</strong><small>查看并使用本次结果</small></span></button> : null}
              <div className={styles.messageState}>{entry.queued ? '排队中' : entry.optimistic ? '发送中' : null}</div>
            </div>
            <MessageActions entry={entry} onCopyMessage={onCopyMessage} onRegenerate={onRegenerate} onEdit={onEdit} />
          </article>;
        })}
      </div>}
      {showScrollToLatest && onScrollToLatest ? <button type="button" className={styles.scrollLatest} onClick={onScrollToLatest} aria-label="回到最新消息"><ArrowDown size={16} />回到最新</button> : null}
    </div>
  </div>;
}
