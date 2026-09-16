import React, { type ReactNode } from 'react';
import { AppWindow, ArrowDown, Check, Copy, Edit3, RotateCcw } from 'lucide-react';
import { BotMark } from '../../frontend/apps/web/src/app/host/bot-mark';
import styles from './bot-conversation-view.module.css';

export type BotTranscriptMessage = {
  id: string;
  role: 'me' | 'peer';
  text: string;
  createdAtMs: number;
  kind?: 'message' | 'action' | 'thinking';
  operationId?: string;
  streaming?: boolean;
  optimistic?: boolean;
  queued?: boolean;
  actionTitle?: string;
  actionDetail?: string;
  actionStatus?: 'running' | 'completed' | 'failed' | 'interrupted';
  miniAppId?: string;
};

type BotConversationViewProps = {
  title: string;
  description: string;
  botId: string;
  messages: BotTranscriptMessage[];
  activeOperationId?: string | null;
  hasEarlierMessages?: boolean;
  messageAreaRef?: React.RefObject<HTMLDivElement | null>;
  showScrollToLatest?: boolean;
  onLoadEarlier?: () => void;
  onOpenMiniApp?: (id: string) => void;
  onScroll?: React.UIEventHandler<HTMLDivElement>;
  onScrollToLatest?: () => void;
  onCopyMessage?: (message: BotTranscriptMessage) => void;
  onRegenerate?: (message: BotTranscriptMessage) => void;
  onEdit?: (message: BotTranscriptMessage) => void;
  onContextMenu?: (event: React.MouseEvent<HTMLElement>, message: BotTranscriptMessage) => void;
};

function renderInline(value: string): ReactNode[] {
  const pattern = /(\*\*[^*]+\*\*|\x60[^\x60]+\x60|\[[^\]]+\]\(https?:\/\/[^)]+\))/g;
  const parts = value.split(pattern);
  return parts.map((part, index) => {
    if (part.startsWith('**') && part.endsWith('**')) {
      return <strong key={index}>{part.slice(2, -2)}</strong>;
    }
    if (part.startsWith('\x60') && part.endsWith('\x60')) {
      return <code key={index}>{part.slice(1, -1)}</code>;
    }
    const link = /^\[([^\]]+)\]\((https?:\/\/[^)]+)\)$/.exec(part);
    if (link) {
      return (
        <a key={index} href={link[2]} target="_blank" rel="noreferrer">
          {link[1]}
        </a>
      );
    }
    return <React.Fragment key={index}>{part}</React.Fragment>;
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
      // Clipboard permissions are optional; the code remains selectable.
    }
  };
  return (
    <div className={styles.codeBlock}>
      <div className={styles.codeToolbar}>
        <span>{language || '代码'}</span>
        {previewable ? <button type="button" onClick={() => setPreview(!preview)} aria-expanded={preview}>{preview ? '关闭预览' : '预览小程序'}</button> : null}
        <button type="button" onClick={() => void copy()} aria-label="复制代码">
          {copied ? <Check size={14} /> : <Copy size={14} />}
          {copied ? '已复制' : '复制'}
        </button>
      </div>
      {preview && previewable ? <iframe className={styles.generatedPreview} title="生成的小程序预览"
        sandbox="allow-scripts" referrerPolicy="no-referrer"
        srcDoc={'<!doctype html><meta http-equiv="Content-Security-Policy" content="default-src \'none\'; script-src \'unsafe-inline\'; style-src \'unsafe-inline\'; img-src data: blob:; font-src data:; connect-src \'none\'; form-action \'none\'; base-uri \'none\'">' + value} /> : null}
      <pre><code>{value}</code></pre>
    </div>
  );
}

function MarkdownContent({ value }: { value: string }) {
  const lines = value.replace(/\r\n/g, '\n').split('\n');
  const blocks: ReactNode[] = [];
  let index = 0;
  while (index < lines.length) {
    if (!lines[index]?.trim()) {
      index += 1;
      continue;
    }
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
    while (
      index < lines.length
      && lines[index]?.trim()
      && !lines[index]?.startsWith('\x60\x60\x60')
    ) {
      paragraph.push(lines[index] ?? '');
      index += 1;
    }
    const first = paragraph[0] ?? '';
    const heading = /^(#{1,3})\s+(.+)$/.exec(first);
    if (heading && paragraph.length === 1) {
      const level = Math.min(heading[1].length, 3);
      const content = renderInline(heading[2]);
      blocks.push(
        level === 1
          ? <h2 key={blocks.length}>{content}</h2>
          : level === 2
            ? <h3 key={blocks.length}>{content}</h3>
            : <h4 key={blocks.length}>{content}</h4>,
      );
      continue;
    }
    const listItems = paragraph
      .filter((line) => /^(\s*[-*]|\s*\d+\.)\s+/.test(line))
      .map((line) => line.replace(/^\s*(?:[-*]|\d+\.)\s+/, ''));
    if (listItems.length === paragraph.length) {
      const ordered = /^\s*\d+\./.test(first);
      const List = ordered ? 'ol' : 'ul';
      blocks.push(
        <List key={blocks.length}>
          {listItems.map((item, itemIndex) => <li key={itemIndex}>{renderInline(item)}</li>)}
        </List>,
      );
      continue;
    }
    blocks.push(
      <p key={blocks.length}>
        {paragraph.map((line, lineIndex) => (
          <React.Fragment key={lineIndex}>
            {lineIndex ? <br /> : null}
            {renderInline(line)}
          </React.Fragment>
        ))}
      </p>,
    );
  }
  return <>{blocks}</>;
}

function formatTime(timestamp: number): string {
  return new Intl.DateTimeFormat('zh-CN', { hour: '2-digit', minute: '2-digit' }).format(timestamp);
}

function MessageActions({
  message,
  onCopyMessage,
  onRegenerate,
  onEdit,
}: {
  message: BotTranscriptMessage;
  onCopyMessage?: (message: BotTranscriptMessage) => void;
  onRegenerate?: (message: BotTranscriptMessage) => void;
  onEdit?: (message: BotTranscriptMessage) => void;
}) {
  if (!onCopyMessage && !onRegenerate && !onEdit) return null;
  return (
    <div className={styles.messageActions} role="toolbar" aria-label="消息操作">
      {onCopyMessage ? (
        <button type="button" onClick={() => onCopyMessage(message)} aria-label="复制消息" title="复制">
          <Copy size={15} />
        </button>
      ) : null}
      {message.role === 'peer' && onRegenerate ? (
        <button type="button" onClick={() => onRegenerate(message)} aria-label="重新生成" title="重新生成">
          <RotateCcw size={15} />
        </button>
      ) : null}
      {message.role === 'me' && onEdit ? (
        <button type="button" onClick={() => onEdit(message)} aria-label="编辑消息" title="编辑并重试">
          <Edit3 size={15} />
        </button>
      ) : null}
    </div>
  );
}

function StepGroup({ steps }: { steps: BotTranscriptMessage[] }) {
  const running = steps.some((step) => step.actionStatus === 'running');
  const failed = steps.some((step) => step.actionStatus === 'failed');
  const interrupted = steps.some((step) => step.actionStatus === 'interrupted');
  const completed = steps.filter((step) => step.actionStatus === 'completed').length;
  const [expanded, setExpanded] = React.useState<boolean | null>(null);
  const open = expanded ?? running;
  return (
    <section className={styles.stepGroup} aria-label="任务步骤" data-testid="agent-step-group">
      <button type="button" className={styles.stepSummary} aria-expanded={open}
        onClick={() => setExpanded(!open)}>
        <span>{running ? '正在执行' : failed ? '执行失败' : interrupted ? '已停止' : '执行记录'}</span>
        <span>{completed} / {steps.length} 步完成 · {open ? '收起' : '展开'}</span>
      </button>
      {open ? <ol className={styles.stepList}>{steps.map((step) => (
        <li key={step.id} className={styles.actionRow} data-testid="agent-step"
          data-operation-id={step.operationId} data-status={step.actionStatus}>
          <span className={styles.actionMarker} aria-hidden="true" />
          <div><strong>{step.actionTitle || '处理任务'}</strong>
            {step.actionDetail ? <details><summary>查看详情</summary><pre>{step.actionDetail}</pre></details> : null}
          </div>
          <span className={styles.actionStatus}>{step.actionStatus === 'running' ? '进行中'
            : step.actionStatus === 'failed' ? '失败' : step.actionStatus === 'interrupted' ? '已停止' : '完成'}</span>
        </li>
      ))}</ol> : null}
    </section>
  );
}

export function BotConversationView({
  title,
  description,
  botId,
  messages,
  activeOperationId,
  hasEarlierMessages = false,
  messageAreaRef,
  showScrollToLatest = false,
  onLoadEarlier,
  onOpenMiniApp,
  onScroll,
  onScrollToLatest,
  onCopyMessage,
  onRegenerate,
  onEdit,
  onContextMenu,
}: BotConversationViewProps) {
  return (
    <div className={styles.root}>
      <div
        ref={messageAreaRef}
        className={styles.messageArea}
        data-testid="message-list"
        data-agent-operation-id={activeOperationId ?? undefined}
        onScroll={onScroll}
        aria-label={title + ' 会话'}
      >
        <div className={styles.botIntro}>
          <BotMark botId={botId} state="idle" size={44} label={title} />
          <div>
            <h2>{title}</h2>
            <p>{description || 'Bot 会在这个独立会话中回复你。'}</p>
          </div>
        </div>
        {hasEarlierMessages && onLoadEarlier ? (
          <button type="button" className={styles.loadEarlier} onClick={onLoadEarlier}>
            查看更早的消息
          </button>
        ) : null}
        {messages.length === 0 ? (
          <div className={styles.emptyState}>
            <strong>开始一个新会话</strong>
            <span>向 {title} 提问，回复会连续显示在这里。</span>
          </div>
        ) : (
          <div className={styles.transcript}>
            {messages.map((message, index) => {
              if (message.kind === 'action') {
                const previous = messages[index - 1];
                if (previous?.kind === 'action' && previous.operationId === message.operationId) return null;
                const steps: BotTranscriptMessage[] = [];
                for (let i = index; i < messages.length; i += 1) {
                  const step = messages[i];
                  if (step.kind !== 'action' || step.operationId !== message.operationId) break;
                  steps.push(step);
                }
                return <StepGroup key={message.id} steps={steps} />;
              }
              if (message.kind === 'thinking') {
                return (
                  <div key={message.id} className={styles.thinkingRow} data-testid="agent-thinking" data-operation-id={message.operationId}>
                    <BotMark botId={botId} state="thinking" size={28} label={title} />
                    <div className={styles.thinkingCopy}>
                      <strong>{message.actionTitle || '正在思考'}</strong>
                      <span>{message.actionDetail || '正在整理回复…'}</span>
                    </div>
                    <span className={styles.thinkingDots} aria-hidden="true"><i /><i /><i /></span>
                  </div>
                );
              }
              const userMessage = message.role === 'me';
              return (
                <article
                  key={message.id}
                  className={styles.messageRow + ' ' + (userMessage ? styles.userRow : styles.assistantRow)}
                  data-message-id={message.id}
                  data-agent-message-role={message.role}
                  data-operation-id={message.operationId}
                  onContextMenu={(event) => onContextMenu?.(event, message)}
                >
                  {!userMessage ? <BotMark botId={botId} state={message.streaming ? 'writing' : 'idle'} size={28} label={title} /> : null}
                  <div className={styles.messageColumn}>
                    <div className={styles.messageMeta}>
                      <span>{userMessage ? '你' : title}</span>
                      <time dateTime={new Date(message.createdAtMs).toISOString()}>{formatTime(message.createdAtMs)}</time>
                    </div>
                    <div className={styles.messageBody}>
                      <MarkdownContent value={message.text || (message.streaming ? ' ' : '暂无内容')} />
                      {message.streaming ? <span className={styles.streamingCursor} aria-label="正在生成" /> : null}
                    </div>
                    {message.miniAppId && onOpenMiniApp ? (
                      <button type="button" className={styles.artifactCard} data-testid="bot-miniapp-result"
                        onClick={() => onOpenMiniApp(message.miniAppId!)}>
                        <AppWindow size={22} /><span><strong>打开小程序</strong><small>查看并使用本次结果</small></span>
                      </button>
                    ) : null}
                    <div className={styles.messageState}>
                      {message.queued ? '排队中' : message.optimistic ? '发送中' : null}
                    </div>
                  </div>
                  <MessageActions
                    message={message}
                    onCopyMessage={onCopyMessage}
                    onRegenerate={onRegenerate}
                    onEdit={onEdit}
                  />
                </article>
              );
            })}
          </div>
        )}
        {showScrollToLatest && onScrollToLatest ? (
          <button type="button" className={styles.scrollLatest} onClick={onScrollToLatest} aria-label="回到最新消息">
            <ArrowDown size={16} />
            回到最新
          </button>
        ) : null}
      </div>
    </div>
  );
}

export default BotConversationView;
