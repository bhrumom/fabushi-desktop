import {
  AppWindow,
  Bot,
  CheckCircle2,
  ChevronDown,
  Circle,
  Clock3,
  Cpu,
  FileText,
  LoaderCircle,
  RotateCcw,
  ShieldAlert,
  Square,
  Terminal,
  XCircle,
} from 'lucide-react';
import React, { useEffect, useMemo, useReducer, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import type {
  ApprovalResolution,
  CommandAccepted,
  RuntimeCommand,
  RuntimeEvent,
} from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  MAHAYANA_COMMAND_EVENT_NAME,
  MAHAYANA_RUNTIME_EVENT_NAME,
  type MahayanaCommandBridgeContext,
  type MahayanaCommandBridgeDetail,
} from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import {
  agentWorkbenchReducer,
  type AgentApprovalProjection,
  type AgentCardProjection,
  type AgentRunProjection,
  type AgentStepProjection,
  type AgentToolResultProjection,
  type AgentWorkbenchSnapshot,
} from './mahayana-agent-workbench';
import styles from './mahayana-agent-inline-report.module.css';

const STORAGE_KEY = 'fabushi.desktop.mahayana-agent-workbench.v1';
const REPORT_PORTAL_ID = 'mahayana-agent-inline-report-portal';

type ReducerAction = Parameters<typeof agentWorkbenchReducer>[1];

type ActivePeerContext = {
  key: string;
  agentId?: string;
  label?: string;
};

type ReportActivity =
  | { type: 'step'; id: string; timestampMs: number; step: AgentStepProjection }
  | { type: 'tool'; id: string; timestampMs: number; tool: AgentToolResultProjection }
  | { type: 'approval'; id: string; timestampMs: number; approval: AgentApprovalProjection }
  | { type: 'artifact'; id: string; timestampMs: number; card: AgentCardProjection };

function emptySnapshot(): AgentWorkbenchSnapshot {
  return {
    version: 1,
    activeConversationKey: 'mahayana-assistant',
    knownBotIds: ['mahayana-assistant'],
    runs: [],
  };
}

function readSnapshot(): AgentWorkbenchSnapshot {
  if (typeof window === 'undefined') return emptySnapshot();
  try {
    const parsed = JSON.parse(window.localStorage.getItem(STORAGE_KEY) || 'null') as Partial<AgentWorkbenchSnapshot> | null;
    if (!parsed || parsed.version !== 1 || !Array.isArray(parsed.runs)) return emptySnapshot();
    const runs = parsed.runs
      .filter((run): run is AgentRunProjection => Boolean(run && typeof run === 'object' && run.id))
      .map((run) => ({
        ...run,
        steps: Array.isArray(run.steps) ? run.steps : [],
        messages: Array.isArray(run.messages) ? run.messages : [],
        approvals: Array.isArray(run.approvals) ? run.approvals : [],
        cards: Array.isArray(run.cards) ? run.cards : [],
        observations: Array.isArray(run.observations) ? run.observations : [],
        toolResults: Array.isArray(run.toolResults) ? run.toolResults : [],
      }));
    return {
      version: 1,
      activeConversationKey: typeof parsed.activeConversationKey === 'string' && parsed.activeConversationKey
        ? parsed.activeConversationKey
        : 'mahayana-assistant',
      knownBotIds: Array.isArray(parsed.knownBotIds)
        ? parsed.knownBotIds.filter((id): id is string => typeof id === 'string')
        : ['mahayana-assistant'],
      runs,
    };
  } catch {
    return emptySnapshot();
  }
}

function directBotMark(parent: HTMLElement): HTMLElement | null {
  return Array.from(parent.children).find((child) =>
    child instanceof HTMLElement && child.dataset.engine === 'fabushi-motion-v3',
  ) as HTMLElement | null;
}

function activePeerButton(): HTMLButtonElement | null {
  return Array.from(document.querySelectorAll<HTMLButtonElement>('button[data-testid^="peer-"]'))
    .find((button) => button.className.includes('peerActive')) || null;
}

function actorIdFromBotMark(botId: string | undefined): string | undefined {
  if (!botId) return undefined;
  const prefixes = ['peer:bot:', 'peer:agent:'];
  const prefix = prefixes.find((candidate) => botId.startsWith(candidate));
  return prefix ? botId.slice(prefix.length) || undefined : undefined;
}

function contextFromActivePeer(): ActivePeerContext | null {
  const button = activePeerButton();
  const testId = button?.getAttribute('data-testid') || '';
  if (!button || !testId.startsWith('peer-')) return null;
  const rawKey = testId.slice('peer-'.length);
  const mark = directBotMark(button);
  const agentId = actorIdFromBotMark(mark?.dataset.botId);
  const label = mark?.getAttribute('aria-label') || button.textContent?.trim() || undefined;

  if (rawKey.startsWith('legacy:conversation:')) {
    return { key: rawKey.slice('legacy:conversation:'.length), agentId, label };
  }
  if (rawKey.startsWith('selfhosted:')) {
    return { key: rawKey, agentId, label };
  }
  if (rawKey.startsWith('legacy:bot:') || rawKey.startsWith('account:bot:')) {
    return { key: agentId || rawKey.split(':').slice(2).join(':'), agentId, label };
  }
  return { key: rawKey, agentId, label };
}

function runForPeer(runs: AgentRunProjection[], peer: ActivePeerContext | null): AgentRunProjection | undefined {
  if (!peer) return undefined;
  const reversed = [...runs].reverse();
  const exact = reversed.find((run) => run.conversationKey === peer.key);
  if (exact) return exact;
  if (peer.agentId) {
    const byAgent = reversed.find((run) => run.agentId === peer.agentId);
    if (byAgent) return byAgent;
  }
  return undefined;
}

function reportHeadline(run: AgentRunProjection): string {
  if (run.status === 'waiting-for-approval') return '等待你的批准';
  if (run.status === 'completed') return '工作完成';
  if (run.status === 'failed') return '执行失败';
  if (run.status === 'interrupted') return '任务已暂停';
  const running = [...run.steps].reverse().find((step) => step.status === 'running');
  if (running) return running.title;
  return run.status === 'queued' ? '正在规划任务' : '正在执行任务';
}

function statusLabel(run: AgentRunProjection): string {
  const copy: Record<AgentRunProjection['status'], string> = {
    queued: '规划中',
    running: '执行中',
    'waiting-for-approval': '待批准',
    completed: '已完成',
    failed: '失败',
    interrupted: '已暂停',
  };
  return copy[run.status];
}

function durationLabel(run: AgentRunProjection): string {
  const end = run.completedAtMs || Date.now();
  const seconds = Math.max(0, Math.round((end - run.startedAtMs) / 1000));
  if (seconds < 60) return `${seconds}s`;
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}

function activityForRun(run: AgentRunProjection): ReportActivity[] {
  const activity: ReportActivity[] = [
    ...run.steps.map((step): ReportActivity => ({
      type: 'step',
      id: `step:${step.id}`,
      timestampMs: step.startedAtMs,
      step,
    })),
    ...run.toolResults.map((tool): ReportActivity => ({
      type: 'tool',
      id: `tool:${tool.id}`,
      timestampMs: tool.createdAtMs,
      tool,
    })),
    ...run.approvals.map((approval): ReportActivity => ({
      type: 'approval',
      id: `approval:${approval.approvalId}`,
      timestampMs: approval.requestedAtMs,
      approval,
    })),
    ...run.cards.map((card): ReportActivity => ({
      type: 'artifact',
      id: `artifact:${card.id}`,
      timestampMs: card.createdAtMs,
      card,
    })),
  ];
  return activity.sort((left, right) => left.timestampMs - right.timestampMs);
}

function jsonPreview(value: unknown): string {
  try {
    const text = JSON.stringify(value, null, 2);
    return text.length > 1600 ? `${text.slice(0, 1600)}\n…` : text;
  } catch {
    return String(value);
  }
}

function StepStatusIcon({ status }: { status: AgentStepProjection['status'] }) {
  if (status === 'completed') return <CheckCircle2 size={15} />;
  if (status === 'failed') return <XCircle size={15} />;
  return <LoaderCircle className={styles.spin} size={15} />;
}

function latestAssistantText(run: AgentRunProjection): string {
  return [...run.messages].reverse().find((message) => message.role === 'assistant' && message.text.trim())?.text.trim() || '';
}

function peerMessageArticles(messageArea: HTMLElement): HTMLElement[] {
  return Array.from(messageArea.children).filter((child): child is HTMLElement =>
    child instanceof HTMLElement && child.tagName === 'ARTICLE' && child.className.includes('messagePeer'),
  );
}

function matchingAssistantArticle(messageArea: HTMLElement, run: AgentRunProjection | undefined): HTMLElement | null {
  if (!run) return null;
  const targetText = latestAssistantText(run);
  if (!targetText) return null;
  const normalizedTarget = targetText.replace(/\s+/gu, ' ').trim();
  const candidates = peerMessageArticles(messageArea).reverse();
  return candidates.find((article) => {
    const text = article.querySelector('p')?.textContent?.replace(/\s+/gu, ' ').trim() || '';
    return text === normalizedTarget || (normalizedTarget.length > 80 && text.startsWith(normalizedTarget.slice(0, 80)));
  }) || null;
}

function ensurePortal(messageArea: HTMLElement | null, before: HTMLElement | null): HTMLElement | null {
  const existing = document.getElementById(REPORT_PORTAL_ID);
  if (!messageArea) {
    existing?.remove();
    return null;
  }
  const root = existing || document.createElement('div');
  root.id = REPORT_PORTAL_ID;
  if (root.className !== styles.portalRoot) root.className = styles.portalRoot;
  if (root.parentElement !== messageArea || (before && root.nextElementSibling !== before)) {
    messageArea.insertBefore(root, before);
  }
  return root;
}

function setFinalOutputArticle(article: HTMLElement | null): void {
  document.querySelectorAll<HTMLElement>('[data-agent-final-output="true"]').forEach((element) => {
    if (element !== article) delete element.dataset.agentFinalOutput;
  });
  if (article) article.dataset.agentFinalOutput = 'true';
}

function emitCommand(detail: MahayanaCommandBridgeDetail): void {
  window.dispatchEvent(new CustomEvent<MahayanaCommandBridgeDetail>(MAHAYANA_COMMAND_EVENT_NAME, { detail }));
}

function nextRequestId(prefix: string): string {
  const suffix = typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  return `${prefix}-${suffix}`;
}

async function executeAgentCommand(
  command: Extract<RuntimeCommand, { type: 'chat.send' }>,
  context: MahayanaCommandBridgeContext,
): Promise<CommandAccepted> {
  if (!window.mahayana?.invoke) throw new Error('Mahayana Electron bridge is unavailable');
  const normalized = { ...command, mode: 'agent' as const };
  emitCommand({ phase: 'dispatch', command: normalized, context });
  try {
    const accepted = await window.mahayana.invoke<CommandAccepted>('feature.execute', { command: normalized });
    emitCommand({ phase: 'accepted', command: normalized, accepted, context });
    return accepted;
  } catch (error) {
    emitCommand({
      phase: 'failed',
      command: normalized,
      error: error instanceof Error ? error.message : String(error),
      context,
    });
    throw error;
  }
}

function InlineActivity({
  item,
  onResolveApproval,
}: {
  item: ReportActivity;
  onResolveApproval: (approvalId: string, decision: ApprovalResolution['decision']) => void;
}) {
  if (item.type === 'step') {
    const { step } = item;
    return (
      <div className={styles.activity} data-testid="agent-inline-step" data-status={step.status} data-kind={step.kind}>
        <span className={styles.rail}><StepStatusIcon status={step.status} /></span>
        <div className={styles.activityCopy}>
          <strong>{step.title}</strong>
          {step.detail ? <small>{step.detail}</small> : null}
          {typeof step.progress === 'number' && typeof step.total === 'number' && step.total > 0 ? (
            <span className={styles.progress} aria-label={`${step.progress}/${step.total}`}>
              <i style={{ width: `${Math.min(100, Math.max(0, (step.progress / step.total) * 100))}%` }} />
            </span>
          ) : null}
        </div>
        <time>{new Date(step.updatedAtMs).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</time>
      </div>
    );
  }

  if (item.type === 'tool') {
    return (
      <div className={styles.activity} data-testid="agent-inline-tool" data-status="completed" data-kind="tool">
        <span className={styles.rail}><CheckCircle2 size={15} /></span>
        <details className={styles.detailBlock}>
          <summary><Terminal size={14} /><strong>调用工具 · {item.tool.server}</strong><span>{item.tool.tool}</span><ChevronDown size={13} /></summary>
          <pre>{jsonPreview(item.tool.result)}</pre>
        </details>
        <time>{new Date(item.tool.createdAtMs).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</time>
      </div>
    );
  }

  if (item.type === 'approval') {
    const approval = item.approval;
    return (
      <div className={styles.activity} data-testid="agent-inline-approval" data-status={approval.decision ? 'completed' : 'running'} data-kind="approval">
        <span className={styles.rail}><ShieldAlert size={15} /></span>
        <div className={styles.approvalCopy}>
          <strong>{approval.subject || approval.capability}</strong>
          <small>{approval.detail || approval.reason}</small>
          {approval.proposedRule ? <code>{approval.proposedRule}</code> : null}
          {approval.decision ? <em>已处理 · {approval.decision}</em> : (
            <span className={styles.approvalActions}>
              <button type="button" onClick={() => onResolveApproval(approval.approvalId, 'allow-once')}>仅本次允许</button>
              <button type="button" onClick={() => onResolveApproval(approval.approvalId, 'allow-session')}>本会话允许</button>
              <button type="button" onClick={() => onResolveApproval(approval.approvalId, 'deny')}>拒绝</button>
            </span>
          )}
        </div>
        <time>{new Date(approval.requestedAtMs).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</time>
      </div>
    );
  }

  if (item.card.card.kind === 'miniApp') {
    const card = item.card.card;
    return (
      <div className={styles.activity} data-testid="agent-inline-miniapp-artifact" data-status="completed" data-kind="miniApp">
        <span className={styles.rail}><AppWindow size={15} /></span>
        <div className={styles.activityCopy}>
          <strong>{card.name}</strong>
          <small>{card.description || '可直接运行的 Fabushi 小程序产物'}</small>
          <span className={styles.actions}>
            <button
              type="button"
              data-testid="agent-inline-miniapp-open"
              onClick={() => window.dispatchEvent(new CustomEvent('fabushi:open-generated-miniapp', { detail: card }))}
            >
              <AppWindow size={13} />打开小程序
            </button>
          </span>
        </div>
        <time>{new Date(item.card.createdAtMs).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</time>
      </div>
    );
  }

  return (
    <div className={styles.activity} data-testid="agent-inline-artifact" data-status="completed" data-kind="artifact">
      <span className={styles.rail}><FileText size={15} /></span>
      <details className={styles.detailBlock}>
        <summary><FileText size={14} /><strong>生成交付物</strong><span>{item.card.card.kind}</span><ChevronDown size={13} /></summary>
        <pre>{jsonPreview(item.card.card)}</pre>
      </details>
      <time>{new Date(item.card.createdAtMs).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</time>
    </div>
  );
}

function InlineReport({ run, peer }: { run: AgentRunProjection; peer: ActivePeerContext | null }) {
  const [actionError, setActionError] = useState<string | null>(null);
  const activity = useMemo(() => activityForRun(run), [run]);
  const active = run.status === 'queued' || run.status === 'running' || run.status === 'waiting-for-approval';
  const canResume = Boolean(run.prompt && (run.status === 'failed' || run.status === 'interrupted'));

  const interrupt = () => {
    if (!run.operationId || !window.mahayana?.invoke) return;
    setActionError(null);
    void window.mahayana.invoke<void>('feature.interrupt', { operationId: run.operationId })
      .catch((error) => setActionError(error instanceof Error ? error.message : String(error)));
  };

  const resolveApproval = (approvalId: string, decision: ApprovalResolution['decision']) => {
    if (!window.mahayana?.invoke) return;
    setActionError(null);
    void window.mahayana.invoke<void>('feature.approval.resolve', { resolution: { approvalId, decision } })
      .catch((error) => setActionError(error instanceof Error ? error.message : String(error)));
  };

  const resume = () => {
    setActionError(null);
    const command: Extract<RuntimeCommand, { type: 'chat.send' }> = {
      type: 'chat.send',
      requestId: nextRequestId('inline-resume-agent'),
      text: run.prompt,
      agentId: run.agentId,
      conversationId: run.conversationId,
      mode: 'agent',
    };
    void executeAgentCommand(command, {
      conversationKey: run.conversationKey,
      conversationId: run.conversationId,
      agentId: run.agentId,
    }).catch((error) => setActionError(error instanceof Error ? error.message : String(error)));
  };

  return (
    <section
      className={styles.report}
      data-testid="agent-inline-report"
      data-status={run.status}
      data-run-id={run.id}
    >
      <header className={styles.reportHeader}>
        <span className={styles.liveGlyph} data-active={active || undefined}>
          {active ? <LoaderCircle className={styles.spin} size={16} /> : run.status === 'completed' ? <CheckCircle2 size={16} /> : <XCircle size={16} />}
        </span>
        <div className={styles.headerCopy}>
          <strong>{reportHeadline(run)}</strong>
          <small>
            Mahayana 多步骤工作
            {peer?.label ? ` · ${peer.label}` : ''}
          </small>
        </div>
        <span className={styles.statusPill} data-status={run.status}>{statusLabel(run)}</span>
      </header>

      <div className={styles.runtimeMeta}>
        <span><Cpu size={12} />{run.provider || 'mahayana'}{run.model ? ` / ${run.model}` : ''}</span>
        <span><Bot size={12} />{run.mode || 'agent'}</span>
        <span><Clock3 size={12} />{durationLabel(run)}</span>
      </div>

      <div className={styles.feed} data-testid="agent-inline-feed">
        {activity.map((item) => <InlineActivity key={item.id} item={item} onResolveApproval={resolveApproval} />)}
        {!activity.length ? (
          <div className={styles.activity} data-testid="agent-inline-step" data-status="running" data-kind="planning">
            <span className={styles.rail}><LoaderCircle className={styles.spin} size={15} /></span>
            <div className={styles.activityCopy}><strong>正在建立执行计划</strong><small>等待 Mahayana runtime 返回第一个真实步骤</small></div>
          </div>
        ) : null}
      </div>

      {run.observations.length ? (
        <div className={styles.parallel} data-testid="agent-inline-parallel">
          {run.observations.map((item) => (
            <span key={`${item.kind}:${item.id}`} data-kind={item.kind}>
              {item.kind === 'subagent' ? <Bot size={12} /> : item.kind === 'async-task' ? <Terminal size={12} /> : <Circle size={10} />}
              <strong>{item.label}</strong>
              {item.status ? <small>{item.status}</small> : null}
            </span>
          ))}
        </div>
      ) : null}

      {run.status === 'completed' ? <div className={styles.resultBridge}><CheckCircle2 size={14} /><span>执行完成，最终结果如下</span></div> : null}
      {run.error ? <div className={styles.error}><XCircle size={14} /><span>{run.error}</span></div> : null}
      {actionError ? <div className={styles.error}><XCircle size={14} /><span>{actionError}</span></div> : null}

      <footer className={styles.footer}>
        <span>{run.usage ? `${run.usage.totalTokens.toLocaleString()} tokens` : '实时 RuntimeEvent'}</span>
        <span className={styles.actions}>
          {active && run.interruptible && run.operationId ? <button type="button" data-testid="agent-inline-stop" onClick={interrupt}><Square size={12} />停止</button> : null}
          {canResume ? <button type="button" data-testid="agent-inline-resume" onClick={resume}><RotateCcw size={12} />继续任务</button> : null}
        </span>
      </footer>
    </section>
  );
}

export default function MahayanaAgentInlineReport() {
  const [snapshot, dispatch] = useReducer(
    (state: AgentWorkbenchSnapshot, action: ReducerAction) => agentWorkbenchReducer(state, action),
    undefined,
    readSnapshot,
  );
  const [peer, setPeer] = useState<ActivePeerContext | null>(null);
  const [portal, setPortal] = useState<HTMLElement | null>(null);
  const snapshotRef = useRef(snapshot);
  snapshotRef.current = snapshot;

  useEffect(() => {
    const onCommand = (event: Event) => {
      const detail = (event as CustomEvent<MahayanaCommandBridgeDetail>).detail;
      if (detail) dispatch({ type: 'bridge-command', detail });
    };
    const onRuntime = (event: Event) => {
      const detail = (event as CustomEvent<RuntimeEvent>).detail;
      if (detail) dispatch({ type: 'runtime-event', event: detail });
    };
    window.addEventListener(MAHAYANA_COMMAND_EVENT_NAME, onCommand);
    window.addEventListener(MAHAYANA_RUNTIME_EVENT_NAME, onRuntime);
    return () => {
      window.removeEventListener(MAHAYANA_COMMAND_EVENT_NAME, onCommand);
      window.removeEventListener(MAHAYANA_RUNTIME_EVENT_NAME, onRuntime);
    };
  }, []);

  useEffect(() => {
    let disposed = false;
    const refresh = () => {
      if (disposed) return;
      const nextPeer = contextFromActivePeer();
      setPeer((current) => current?.key === nextPeer?.key && current?.agentId === nextPeer?.agentId && current?.label === nextPeer?.label ? current : nextPeer);

      const workspace = document.querySelector<HTMLElement>('[data-testid="messenger-workspace"]');
      const messageArea = workspace?.querySelector<HTMLElement>('[class*="messageArea"]') || null;
      const selectedRun = runForPeer(snapshotRef.current.runs, nextPeer);
      const assistantArticle = messageArea ? matchingAssistantArticle(messageArea, selectedRun) : null;
      setFinalOutputArticle(assistantArticle);
      const root = ensurePortal(messageArea, assistantArticle);
      setPortal((current) => current === root ? current : root);
    };

    const observer = new MutationObserver((records) => {
      const relevant = records.some((record) => {
        if (record.type === 'attributes') {
          return record.attributeName === 'class' && (record.target as HTMLElement).id !== REPORT_PORTAL_ID;
        }
        if (record.type !== 'childList') return false;
        const changed = [...record.addedNodes, ...record.removedNodes];
        return changed.some((node) => !(node instanceof HTMLElement) || node.id !== REPORT_PORTAL_ID);
      });
      if (relevant) refresh();
    });
    observer.observe(document.body, { childList: true, subtree: true, attributes: true, attributeFilter: ['class'] });
    const interval = window.setInterval(refresh, 750);
    refresh();
    return () => {
      disposed = true;
      observer.disconnect();
      window.clearInterval(interval);
      document.getElementById(REPORT_PORTAL_ID)?.remove();
      setFinalOutputArticle(null);
    };
  }, []);

  useEffect(() => {
    const workspace = document.querySelector<HTMLElement>('[data-testid="messenger-workspace"]');
    const messageArea = workspace?.querySelector<HTMLElement>('[class*="messageArea"]') || null;
    const selectedRun = runForPeer(snapshot.runs, peer);
    const assistantArticle = messageArea ? matchingAssistantArticle(messageArea, selectedRun) : null;
    setFinalOutputArticle(assistantArticle);
    const root = ensurePortal(messageArea, assistantArticle);
    if (root !== portal) setPortal(root);
  }, [peer, portal, snapshot.runs]);

  const run = useMemo(() => runForPeer(snapshot.runs, peer), [peer, snapshot.runs]);
  if (!portal || !run) return null;
  return createPortal(<InlineReport run={run} peer={peer} />, portal);
}
