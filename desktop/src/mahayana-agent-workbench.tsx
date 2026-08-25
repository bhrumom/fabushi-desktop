import {
  Bot,
  CheckCircle2,
  ChevronDown,
  Circle,
  Clock3,
  Cpu,
  FileText,
  LoaderCircle,
  PauseCircle,
  Play,
  RotateCcw,
  ShieldAlert,
  Square,
  Terminal,
  XCircle,
} from 'lucide-react';
import React, { useEffect, useMemo, useReducer, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import {
  BotMark,
  botMarkStateFromActivity,
  type BotMarkState,
} from '../../frontend/apps/web/src/app/host/bot-mark';
import type {
  ApprovalResolution,
  CommandAccepted,
  RuntimeCommand,
  RuntimeEvent,
  TranscriptCard,
} from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  MAHAYANA_COMMAND_EVENT_NAME,
  MAHAYANA_RUNTIME_EVENT_NAME,
  type MahayanaCommandBridgeContext,
  type MahayanaCommandBridgeDetail,
} from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import styles from './mahayana-agent-workbench.module.css';

const STORAGE_KEY = 'fabushi.desktop.mahayana-agent-workbench.v1';
const SNAPSHOT_VERSION = 1;
const MAX_RUNS = 64;
const MAX_STEPS_PER_RUN = 160;
const MAX_MESSAGES_PER_RUN = 80;
const MAX_CARDS_PER_RUN = 32;
const MAX_OBSERVATIONS_PER_RUN = 40;
const MAX_TOOL_RESULTS_PER_RUN = 24;

export type AgentRunStatus =
  | 'queued'
  | 'running'
  | 'waiting-for-approval'
  | 'completed'
  | 'failed'
  | 'interrupted';

export type AgentStepProjection = {
  id: string;
  kind: string;
  title: string;
  detail?: string;
  status: 'running' | 'completed' | 'failed';
  progress?: number;
  total?: number;
  startedAtMs: number;
  updatedAtMs: number;
};

export type AgentMessageProjection = {
  id: string;
  role: 'user' | 'assistant';
  text: string;
  operationId?: string;
  streaming?: boolean;
  createdAtMs: number;
};

export type AgentApprovalProjection = {
  approvalId: string;
  miniAppId: string;
  capability: string;
  reason: string;
  kind?: string;
  subject?: string;
  detail?: string;
  proposedRule?: string;
  location?: string;
  decision?: ApprovalResolution['decision'];
  requestedAtMs: number;
};

export type AgentCardProjection = {
  id: string;
  card: TranscriptCard;
  createdAtMs: number;
};

export type AgentObservationProjection = {
  id: string;
  label: string;
  status?: string;
  detail?: string;
  kind: 'subagent' | 'async-task' | 'background';
};

export type AgentToolResultProjection = {
  id: string;
  server: string;
  tool: string;
  result: unknown;
  createdAtMs: number;
};

export type AgentUsageProjection = {
  inputTokens: number;
  cachedInputTokens: number;
  outputTokens: number;
  reasoningTokens: number;
  totalTokens: number;
  contextWindow?: number;
};

export type AgentRunProjection = {
  id: string;
  requestId?: string;
  operationId?: string;
  conversationKey: string;
  conversationId?: string;
  agentId: string;
  prompt: string;
  label: string;
  status: AgentRunStatus;
  interruptible: boolean;
  visualState: BotMarkState;
  provider?: string;
  model?: string;
  mode?: string;
  steps: AgentStepProjection[];
  messages: AgentMessageProjection[];
  approvals: AgentApprovalProjection[];
  cards: AgentCardProjection[];
  observations: AgentObservationProjection[];
  toolResults: AgentToolResultProjection[];
  usage?: AgentUsageProjection;
  error?: string;
  startedAtMs: number;
  updatedAtMs: number;
  completedAtMs?: number;
};

export type AgentWorkbenchSnapshot = {
  version: 1;
  activeConversationKey: string;
  knownBotIds: string[];
  runs: AgentRunProjection[];
};

type WorkbenchAction =
  | { type: 'bridge-command'; detail: MahayanaCommandBridgeDetail }
  | { type: 'runtime-event'; event: RuntimeEvent }
  | { type: 'set-active-conversation'; conversationKey: string }
  | { type: 'clear-history' };

type PortalTargets = {
  timeline: HTMLElement | null;
  headerAvatar: HTMLElement | null;
  peerAvatar: HTMLElement | null;
  infoAvatar: HTMLElement | null;
};

const EMPTY_TARGETS: PortalTargets = {
  timeline: null,
  headerAvatar: null,
  peerAvatar: null,
  infoAvatar: null,
};

function nowFromTimestamp(timestamp?: string): number {
  if (!timestamp) return Date.now();
  const parsed = Date.parse(timestamp);
  return Number.isFinite(parsed) ? parsed : Date.now();
}

function bounded<T>(items: T[], limit: number): T[] {
  return items.length > limit ? items.slice(items.length - limit) : items;
}

function cloneRun(run: AgentRunProjection): AgentRunProjection {
  return {
    ...run,
    steps: run.steps.map((step) => ({ ...step })),
    messages: run.messages.map((message) => ({ ...message })),
    approvals: run.approvals.map((approval) => ({ ...approval })),
    cards: run.cards.map((card) => ({ ...card })),
    observations: run.observations.map((item) => ({ ...item })),
    toolResults: run.toolResults.map((item) => ({ ...item })),
    usage: run.usage ? { ...run.usage } : undefined,
  };
}

function cloneSnapshot(snapshot: AgentWorkbenchSnapshot): AgentWorkbenchSnapshot {
  return {
    ...snapshot,
    knownBotIds: [...snapshot.knownBotIds],
    runs: snapshot.runs.map(cloneRun),
  };
}

function emptySnapshot(): AgentWorkbenchSnapshot {
  return {
    version: SNAPSHOT_VERSION,
    activeConversationKey: 'mahayana-assistant',
    knownBotIds: ['mahayana-assistant'],
    runs: [],
  };
}

function normalizeLoadedSnapshot(value: unknown): AgentWorkbenchSnapshot {
  if (!value || typeof value !== 'object') return emptySnapshot();
  const candidate = value as Partial<AgentWorkbenchSnapshot>;
  if (candidate.version !== SNAPSHOT_VERSION || !Array.isArray(candidate.runs)) {
    return emptySnapshot();
  }

  const loadedAt = Date.now();
  const runs = candidate.runs
    .filter((run): run is AgentRunProjection => Boolean(run && typeof run === 'object' && run.id))
    .map((run) => {
      const copied = cloneRun({
        ...run,
        steps: Array.isArray(run.steps) ? run.steps : [],
        messages: Array.isArray(run.messages) ? run.messages : [],
        approvals: Array.isArray(run.approvals) ? run.approvals : [],
        cards: Array.isArray(run.cards) ? run.cards : [],
        observations: Array.isArray(run.observations) ? run.observations : [],
        toolResults: Array.isArray(run.toolResults) ? run.toolResults : [],
      });
      if (['queued', 'running', 'waiting-for-approval'].includes(copied.status)) {
        copied.status = 'interrupted';
        copied.visualState = 'idle';
        copied.interruptible = false;
        copied.error = copied.error || '应用重新启动后任务已暂停，可从运行卡片继续。';
        copied.updatedAtMs = loadedAt;
        copied.steps = copied.steps.map((step) =>
          step.status === 'running'
            ? { ...step, status: 'failed', updatedAtMs: loadedAt }
            : step,
        );
      }
      return copied;
    });

  return {
    version: SNAPSHOT_VERSION,
    activeConversationKey:
      typeof candidate.activeConversationKey === 'string' && candidate.activeConversationKey
        ? candidate.activeConversationKey
        : 'mahayana-assistant',
    knownBotIds: Array.isArray(candidate.knownBotIds)
      ? Array.from(new Set(['mahayana-assistant', ...candidate.knownBotIds.filter((id): id is string => typeof id === 'string')]))
      : ['mahayana-assistant'],
    runs: bounded(runs, MAX_RUNS),
  };
}

function loadSnapshot(): AgentWorkbenchSnapshot {
  if (typeof window === 'undefined') return emptySnapshot();
  try {
    const value = window.localStorage.getItem(STORAGE_KEY);
    return value ? normalizeLoadedSnapshot(JSON.parse(value)) : emptySnapshot();
  } catch {
    return emptySnapshot();
  }
}

function conversationKeyFromContext(
  command: RuntimeCommand,
  context?: MahayanaCommandBridgeContext,
): string {
  if (context?.conversationKey) return context.conversationKey;
  if (command.type === 'chat.send') {
    return command.conversationId || command.agentId || 'mahayana-assistant';
  }
  if (command.type === 'conversation.open') return command.conversationId;
  return 'mahayana-assistant';
}

function createRunFromCommand(
  command: Extract<RuntimeCommand, { type: 'chat.send' }>,
  context?: MahayanaCommandBridgeContext,
): AgentRunProjection {
  const now = Date.now();
  const conversationKey = conversationKeyFromContext(command, context);
  return {
    id: `request:${command.requestId}`,
    requestId: command.requestId,
    conversationKey,
    conversationId: command.conversationId || context?.conversationId,
    agentId: command.agentId || context?.agentId || 'mahayana-assistant',
    prompt: command.text,
    label: 'Mahayana Agent 运行',
    status: 'queued',
    interruptible: false,
    visualState: 'waking',
    provider: undefined,
    model: command.model,
    mode: command.mode || 'agent',
    steps: [
      {
        id: `request:${command.requestId}:accepted`,
        kind: 'plan',
        title: 'Mahayana 已接管任务',
        detail: '正在建立上下文并规划执行步骤',
        status: 'running',
        startedAtMs: now,
        updatedAtMs: now,
      },
    ],
    messages: [
      {
        id: `request:${command.requestId}:user`,
        role: 'user',
        text: command.text,
        createdAtMs: now,
      },
    ],
    approvals: [],
    cards: [],
    observations: [],
    toolResults: [],
    startedAtMs: now,
    updatedAtMs: now,
  };
}

function activeStatuses(status: AgentRunStatus): boolean {
  return status === 'queued' || status === 'running' || status === 'waiting-for-approval';
}

function findRunIndex(
  snapshot: AgentWorkbenchSnapshot,
  operationId?: string,
  requestId?: string,
): number {
  if (operationId) {
    const exact = snapshot.runs.findIndex((run) => run.operationId === operationId);
    if (exact >= 0) return exact;
  }
  if (requestId) {
    const exact = snapshot.runs.findIndex((run) => run.requestId === requestId);
    if (exact >= 0) return exact;
  }
  for (let index = snapshot.runs.length - 1; index >= 0; index -= 1) {
    const run = snapshot.runs[index];
    if (run.conversationKey === snapshot.activeConversationKey && activeStatuses(run.status)) return index;
  }
  for (let index = snapshot.runs.length - 1; index >= 0; index -= 1) {
    if (activeStatuses(snapshot.runs[index].status)) return index;
  }
  return -1;
}

function ensureRunForOperation(
  snapshot: AgentWorkbenchSnapshot,
  operationId: string | undefined,
  timestampMs: number,
): number {
  const existing = findRunIndex(snapshot, operationId);
  if (existing >= 0) {
    if (operationId && !snapshot.runs[existing].operationId) {
      snapshot.runs[existing].operationId = operationId;
    }
    return existing;
  }

  const run: AgentRunProjection = {
    id: operationId ? `operation:${operationId}` : `orphan:${timestampMs}`,
    operationId,
    conversationKey: snapshot.activeConversationKey || 'mahayana-assistant',
    agentId: 'mahayana-assistant',
    prompt: '',
    label: 'Mahayana Agent 运行',
    status: 'running',
    interruptible: true,
    visualState: 'thinking',
    steps: [],
    messages: [],
    approvals: [],
    cards: [],
    observations: [],
    toolResults: [],
    startedAtMs: timestampMs,
    updatedAtMs: timestampMs,
  };
  snapshot.runs.push(run);
  return snapshot.runs.length - 1;
}

function upsertStep(
  run: AgentRunProjection,
  step: Omit<AgentStepProjection, 'startedAtMs' | 'updatedAtMs'>,
  timestampMs: number,
): void {
  const existing = run.steps.findIndex((item) => item.id === step.id);
  if (existing >= 0) {
    run.steps[existing] = {
      ...run.steps[existing],
      ...step,
      updatedAtMs: timestampMs,
    };
  } else {
    run.steps.push({
      ...step,
      startedAtMs: timestampMs,
      updatedAtMs: timestampMs,
    });
  }
  run.steps = bounded(run.steps, MAX_STEPS_PER_RUN);
}

function completeBootstrapStep(run: AgentRunProjection, timestampMs: number): void {
  const bootstrap = run.steps.find((step) => step.id.endsWith(':accepted'));
  if (bootstrap?.status === 'running') {
    bootstrap.status = 'completed';
    bootstrap.updatedAtMs = timestampMs;
  }
}

function observationFromUnknown(
  value: unknown,
  index: number,
  kind: AgentObservationProjection['kind'],
): AgentObservationProjection {
  const record = value && typeof value === 'object' ? (value as Record<string, unknown>) : {};
  const id = String(record.id || record.operationId || record.taskId || `${kind}:${index}`);
  const label = String(record.name || record.label || record.title || record.agentName || id);
  const status = typeof record.status === 'string' ? record.status : undefined;
  const detailValue = record.detail || record.description || record.source;
  return {
    id,
    label,
    status,
    detail: typeof detailValue === 'string' ? detailValue : undefined,
    kind,
  };
}

function updateAssistantDelta(run: AgentRunProjection, operationId: string, delta: string, timestampMs: number): void {
  let message = [...run.messages]
    .reverse()
    .find((item) => item.role === 'assistant' && item.operationId === operationId && item.streaming);
  if (!message) {
    message = {
      id: `stream:${operationId}`,
      role: 'assistant',
      text: '',
      operationId,
      streaming: true,
      createdAtMs: timestampMs,
    };
    run.messages.push(message);
  }
  message.text += delta;
  run.messages = bounded(run.messages, MAX_MESSAGES_PER_RUN);
}

function appendChatMessage(
  run: AgentRunProjection,
  event: Extract<RuntimeEvent, { type: 'chat.message' }>,
  timestampMs: number,
): void {
  if (event.role === 'user') {
    if (run.messages.some((item) => item.role === 'user' && item.text === event.text)) return;
    run.messages.push({
      id: `message:user:${timestampMs}`,
      role: 'user',
      text: event.text,
      operationId: event.operationId,
      createdAtMs: timestampMs,
    });
  } else {
    const streaming = [...run.messages]
      .reverse()
      .find((item) => item.role === 'assistant' && item.operationId === event.operationId && item.streaming);
    if (streaming) {
      streaming.text = event.text || streaming.text;
      streaming.streaming = false;
    } else if (!run.messages.some((item) => item.role === 'assistant' && item.text === event.text)) {
      run.messages.push({
        id: `message:assistant:${event.operationId || timestampMs}`,
        role: 'assistant',
        text: event.text,
        operationId: event.operationId,
        createdAtMs: timestampMs,
      });
    }
  }
  run.messages = bounded(run.messages, MAX_MESSAGES_PER_RUN);
}

function projectCommand(
  snapshot: AgentWorkbenchSnapshot,
  detail: MahayanaCommandBridgeDetail,
): AgentWorkbenchSnapshot {
  const next = cloneSnapshot(snapshot);
  const command = detail.command;
  const conversationKey = conversationKeyFromContext(command, detail.context);
  if (command.type === 'conversation.open') {
    next.activeConversationKey = conversationKey;
    return next;
  }
  if (command.type !== 'chat.send') return next;

  next.activeConversationKey = conversationKey;
  if (detail.phase === 'dispatch') {
    const existing = findRunIndex(next, undefined, command.requestId);
    if (existing < 0) next.runs.push(createRunFromCommand(command, detail.context));
  } else if (detail.phase === 'accepted') {
    const index = findRunIndex(next, detail.accepted.operationId, command.requestId);
    const target = index >= 0 ? index : ensureRunForOperation(next, detail.accepted.operationId, Date.now());
    const run = next.runs[target];
    run.operationId = detail.accepted.operationId || run.operationId;
    run.status = detail.accepted.operationId ? 'running' : run.status;
    run.interruptible = Boolean(detail.accepted.operationId);
    run.visualState = 'thinking';
    run.updatedAtMs = Date.now();
    completeBootstrapStep(run, run.updatedAtMs);
  } else {
    const index = findRunIndex(next, undefined, command.requestId);
    if (index >= 0) {
      const run = next.runs[index];
      run.status = 'failed';
      run.visualState = 'error';
      run.error = detail.error;
      run.interruptible = false;
      run.updatedAtMs = Date.now();
      run.completedAtMs = run.updatedAtMs;
      completeBootstrapStep(run, run.updatedAtMs);
    }
  }
  next.runs = bounded(next.runs, MAX_RUNS);
  return next;
}

function projectRuntimeEvent(
  snapshot: AgentWorkbenchSnapshot,
  event: RuntimeEvent,
): AgentWorkbenchSnapshot {
  const next = cloneSnapshot(snapshot);
  const timestampMs = nowFromTimestamp(event.timestamp);

  if (event.type === 'conversation.opened') {
    next.activeConversationKey = event.conversationId;
    return next;
  }
  if (event.type === 'bot.listed') {
    next.knownBotIds = Array.from(new Set(['mahayana-assistant', ...event.bots.map((bot) => bot.id)]));
    return next;
  }
  if (event.type === 'bot.changed') {
    const ids = new Set(next.knownBotIds);
    if (event.action === 'deleted') ids.delete(event.bot.id);
    else ids.add(event.bot.id);
    next.knownBotIds = Array.from(ids);
    return next;
  }
  if (event.type === 'host.closed') {
    next.runs.forEach((run) => {
      if (!activeStatuses(run.status)) return;
      run.status = 'interrupted';
      run.visualState = 'idle';
      run.interruptible = false;
      run.error = run.error || 'Mahayana Host 已关闭，任务可以恢复。';
      run.updatedAtMs = timestampMs;
    });
    return next;
  }

  if (event.type === 'operation.started') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    run.operationId = event.operationId;
    run.label = event.label || run.label;
    run.status = 'running';
    run.interruptible = event.interruptible;
    run.visualState = 'thinking';
    run.updatedAtMs = timestampMs;
    completeBootstrapStep(run, timestampMs);
    upsertStep(run, {
      id: `${event.operationId}:runtime-start`,
      kind: 'runtime',
      title: '运行已启动',
      detail: event.label,
      status: 'running',
    }, timestampMs);
  } else if (event.type === 'model.routed') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    run.provider = event.provider;
    run.model = event.model;
    run.mode = event.mode;
    run.status = 'running';
    run.visualState = 'thinking';
    run.updatedAtMs = timestampMs;
    completeBootstrapStep(run, timestampMs);
    upsertStep(run, {
      id: `${event.operationId}:model-route`,
      kind: 'model',
      title: `模型路由 · ${event.provider}`,
      detail: `${event.model} · ${event.mode}`,
      status: 'completed',
    }, timestampMs);
  } else if (event.type === 'agent.step') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    run.status = event.status === 'failed' ? 'failed' : 'running';
    run.visualState = event.status === 'failed'
      ? 'error'
      : botMarkStateFromActivity({ kind: event.kind, title: event.title, detail: event.detail });
    run.updatedAtMs = timestampMs;
    if (event.status === 'failed') run.error = event.detail || event.title;
    completeBootstrapStep(run, timestampMs);
    const runtimeStart = run.steps.find((step) => step.id.endsWith(':runtime-start'));
    if (runtimeStart?.status === 'running') {
      runtimeStart.status = 'completed';
      runtimeStart.updatedAtMs = timestampMs;
    }
    upsertStep(run, {
      id: event.stepId,
      kind: event.kind,
      title: event.title,
      detail: event.detail,
      status: event.status,
      progress: event.progress,
      total: event.total,
    }, timestampMs);
  } else if (event.type === 'chat.delta') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    run.status = 'running';
    run.visualState = 'speaking';
    run.updatedAtMs = timestampMs;
    updateAssistantDelta(run, event.operationId, event.delta, timestampMs);
  } else if (event.type === 'chat.message') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    appendChatMessage(run, event, timestampMs);
    run.updatedAtMs = timestampMs;
    if (event.role === 'assistant') run.visualState = 'speaking';
  } else if (event.type === 'transcript.card') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    if (!run.cards.some((item) => item.id === event.entryId)) {
      run.cards.push({ id: event.entryId, card: event.card, createdAtMs: timestampMs });
      run.cards = bounded(run.cards, MAX_CARDS_PER_RUN);
    }
    run.visualState = botMarkStateFromActivity({ kind: event.card.kind, title: event.card.kind });
    run.updatedAtMs = timestampMs;
  } else if (event.type === 'mcp.toolResult') {
    const index = ensureRunForOperation(next, undefined, timestampMs);
    const run = next.runs[index];
    run.toolResults.push({
      id: `mcp:${event.server}:${event.tool}:${timestampMs}`,
      server: event.server,
      tool: event.tool,
      result: event.result,
      createdAtMs: timestampMs,
    });
    run.toolResults = bounded(run.toolResults, MAX_TOOL_RESULTS_PER_RUN);
    run.visualState = 'tool-running';
    run.updatedAtMs = timestampMs;
  } else if (event.type === 'approval.requested') {
    const index = ensureRunForOperation(next, undefined, timestampMs);
    const run = next.runs[index];
    if (!run.approvals.some((item) => item.approvalId === event.approvalId)) {
      run.approvals.push({
        approvalId: event.approvalId,
        miniAppId: event.miniAppId,
        capability: event.capability,
        reason: event.reason,
        kind: event.kind,
        subject: event.subject,
        detail: event.detail,
        proposedRule: event.proposedRule,
        location: event.location,
        requestedAtMs: timestampMs,
      });
    }
    run.status = 'waiting-for-approval';
    run.visualState = 'alerting';
    run.updatedAtMs = timestampMs;
  } else if (event.type === 'approval.resolved') {
    const run = next.runs.find((item) => item.approvals.some((approval) => approval.approvalId === event.approvalId));
    if (run) {
      const approval = run.approvals.find((item) => item.approvalId === event.approvalId);
      if (approval) approval.decision = event.decision;
      const pending = run.approvals.some((item) => !item.decision);
      run.status = pending ? 'waiting-for-approval' : 'running';
      run.visualState = pending ? 'alerting' : 'thinking';
      run.updatedAtMs = timestampMs;
    }
  } else if (event.type === 'usage.updated') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    run.usage = {
      inputTokens: event.inputTokens,
      cachedInputTokens: event.cachedInputTokens,
      outputTokens: event.outputTokens,
      reasoningTokens: event.reasoningTokens,
      totalTokens: event.totalTokens,
      contextWindow: event.contextWindow,
    };
    run.updatedAtMs = timestampMs;
    if (run.provider === 'mahayana-test' && run.messages.some((message) => message.role === 'assistant')) {
      run.status = 'completed';
      run.visualState = 'result';
      run.interruptible = false;
      run.completedAtMs = timestampMs;
      run.steps.forEach((step) => {
        if (step.status === 'running') {
          step.status = 'completed';
          step.updatedAtMs = timestampMs;
        }
      });
    }
  } else if (event.type === 'subagent.listed' || event.type === 'subagent.changed') {
    const index = ensureRunForOperation(next, undefined, timestampMs);
    const run = next.runs[index];
    const values = event.type === 'subagent.listed' ? event.subagents : [event.subagent];
    const incoming = values.map((value, itemIndex) => observationFromUnknown(value, itemIndex, 'subagent'));
    const byId = new Map(run.observations.map((item) => [item.id, item]));
    incoming.forEach((item) => byId.set(item.id, item));
    run.observations = bounded(Array.from(byId.values()), MAX_OBSERVATIONS_PER_RUN);
    run.visualState = 'spawning';
    run.updatedAtMs = timestampMs;
  } else if (event.type === 'asyncTask.listed' || event.type === 'asyncTask.changed') {
    const index = ensureRunForOperation(next, undefined, timestampMs);
    const run = next.runs[index];
    const incoming = event.tasks.map((value, itemIndex) => observationFromUnknown(value, itemIndex, 'async-task'));
    const retained = run.observations.filter((item) => item.kind !== 'async-task');
    run.observations = bounded([...retained, ...incoming], MAX_OBSERVATIONS_PER_RUN);
    run.visualState = incoming.length ? 'orbit' : run.visualState;
    run.updatedAtMs = timestampMs;
  } else if (event.type === 'agent.backgroundStarted') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    run.agentId = event.agentId;
    run.status = 'running';
    run.visualState = 'orbit';
    run.observations.push({
      id: `background:${event.operationId}`,
      label: event.agentName,
      status: 'running',
      detail: event.source,
      kind: 'background',
    });
    run.observations = bounded(run.observations, MAX_OBSERVATIONS_PER_RUN);
    run.updatedAtMs = timestampMs;
  } else if (event.type === 'agent.backgroundDelta' || event.type === 'agent.backgroundMessage') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    const text = event.type === 'agent.backgroundDelta' ? event.delta : event.text;
    updateAssistantDelta(run, event.operationId, text, timestampMs);
    run.visualState = 'receiving';
    run.updatedAtMs = timestampMs;
  } else if (event.type === 'agent.backgroundFinished') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    const observation = run.observations.find((item) => item.id === `background:${event.operationId}`);
    if (observation) observation.status = event.error ? 'failed' : 'completed';
    run.updatedAtMs = timestampMs;
    if (event.error) {
      run.status = 'failed';
      run.visualState = 'error';
      run.error = event.error;
    }
  } else if (event.type === 'operation.completed') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    run.status = 'completed';
    run.visualState = 'result';
    run.interruptible = false;
    run.completedAtMs = timestampMs;
    run.updatedAtMs = timestampMs;
    run.steps.forEach((step) => {
      if (step.status === 'running') {
        step.status = 'completed';
        step.updatedAtMs = timestampMs;
      }
    });
  } else if (event.type === 'operation.failed') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    run.status = 'failed';
    run.visualState = 'error';
    run.interruptible = false;
    run.error = `${event.code}: ${event.message}`;
    run.completedAtMs = timestampMs;
    run.updatedAtMs = timestampMs;
    run.steps.forEach((step) => {
      if (step.status === 'running') {
        step.status = 'failed';
        step.updatedAtMs = timestampMs;
      }
    });
  } else if (event.type === 'operation.interrupted') {
    const index = ensureRunForOperation(next, event.operationId, timestampMs);
    const run = next.runs[index];
    run.status = 'interrupted';
    run.visualState = 'idle';
    run.interruptible = false;
    run.completedAtMs = timestampMs;
    run.updatedAtMs = timestampMs;
    run.steps.forEach((step) => {
      if (step.status === 'running') {
        step.status = 'failed';
        step.updatedAtMs = timestampMs;
      }
    });
  }

  next.runs = bounded(next.runs, MAX_RUNS);
  return next;
}

export function agentWorkbenchReducer(
  snapshot: AgentWorkbenchSnapshot,
  action: WorkbenchAction,
): AgentWorkbenchSnapshot {
  if (action.type === 'bridge-command') return projectCommand(snapshot, action.detail);
  if (action.type === 'runtime-event') return projectRuntimeEvent(snapshot, action.event);
  if (action.type === 'set-active-conversation') {
    if (!action.conversationKey || action.conversationKey === snapshot.activeConversationKey) return snapshot;
    return { ...snapshot, activeConversationKey: action.conversationKey };
  }
  return { ...emptySnapshot(), knownBotIds: snapshot.knownBotIds };
}

function conversationKeyFromPeerTestId(testId: string): string {
  const key = testId.replace(/^peer-/, '');
  if (key.startsWith('legacy:conversation:')) return key.slice('legacy:conversation:'.length);
  return key;
}

function activePeerButton(): HTMLButtonElement | null {
  return Array.from(document.querySelectorAll<HTMLButtonElement>('button[data-testid^="peer-"]'))
    .find((button) => button.className.includes('peerActive')) || null;
}

function activePeerContext(): { peerKey: string; kind: string; actorId: string } | null {
  const button = activePeerButton();
  const testId = button?.getAttribute('data-testid');
  if (!button || !testId) return null;
  const status = document.querySelector<HTMLElement>('[data-testid="conversation-status"]');
  const identity = status?.parentElement?.parentElement;
  const mark = identity?.querySelector<HTMLElement>('[data-bot-id^="peer:"]');
  const botId = mark?.dataset.botId;
  if (!botId) return null;
  const parts = botId.split(':');
  return {
    peerKey: conversationKeyFromPeerTestId(testId),
    kind: parts[1] || '',
    actorId: parts.slice(2).join(':') || 'mahayana-assistant',
  };
}

function emitCommandDetail(detail: MahayanaCommandBridgeDetail): void {
  window.dispatchEvent(new CustomEvent<MahayanaCommandBridgeDetail>(MAHAYANA_COMMAND_EVENT_NAME, { detail }));
}

async function executeAgentCommand(
  command: Extract<RuntimeCommand, { type: 'chat.send' }>,
  context: MahayanaCommandBridgeContext,
): Promise<CommandAccepted> {
  if (!window.mahayana?.invoke) throw new Error('Mahayana Electron bridge is unavailable');
  const normalized: Extract<RuntimeCommand, { type: 'chat.send' }> = {
    ...command,
    mode: 'agent',
  };
  emitCommandDetail({ phase: 'dispatch', command: normalized, context });
  try {
    const accepted = await window.mahayana.invoke<CommandAccepted>('feature.execute', { command: normalized });
    emitCommandDetail({ phase: 'accepted', command: normalized, accepted, context });
    return accepted;
  } catch (error) {
    emitCommandDetail({
      phase: 'failed',
      command: normalized,
      error: error instanceof Error ? error.message : String(error),
      context,
    });
    throw error;
  }
}

function requestId(prefix: string): string {
  const random = typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  return `${prefix}-${random}`;
}

function statusLabel(status: AgentRunStatus): string {
  const labels: Record<AgentRunStatus, string> = {
    queued: '准备中',
    running: '执行中',
    'waiting-for-approval': '等待批准',
    completed: '已完成',
    failed: '失败',
    interrupted: '已暂停',
  };
  return labels[status];
}

function statusLine(run: AgentRunProjection): string {
  const running = [...run.steps].reverse().find((step) => step.status === 'running');
  if (run.status === 'waiting-for-approval') return '等待你批准下一步';
  if (run.status === 'completed') return '任务已完成';
  if (run.status === 'failed') return run.error || '任务执行失败';
  if (run.status === 'interrupted') return '任务已暂停，可继续';
  return running?.title || 'Mahayana 正在规划和执行';
}

function formatDuration(run: AgentRunProjection): string {
  const end = run.completedAtMs || Date.now();
  const seconds = Math.max(0, Math.round((end - run.startedAtMs) / 1000));
  if (seconds < 60) return `${seconds}s`;
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}

function jsonPreview(value: unknown): string {
  try {
    const text = JSON.stringify(value, null, 2);
    return text.length > 1800 ? `${text.slice(0, 1800)}\n…` : text;
  } catch {
    return String(value);
  }
}

function transcriptCardTitle(card: TranscriptCard): string {
  switch (card.kind) {
    case 'emailDraft': return '邮件草稿';
    case 'slackDraft': return 'Slack 草稿';
    case 'secretRequest': return card.label;
    case 'listenerConnect': return `连接 ${card.platform}`;
    case 'event': return card.event.title;
    case 'pdf': return card.name;
    case 'spreadsheet': return card.name;
  }
}

function StepIcon({ status }: { status: AgentStepProjection['status'] }) {
  if (status === 'completed') return <CheckCircle2 size={15} />;
  if (status === 'failed') return <XCircle size={15} />;
  return <LoaderCircle className={styles.spin} size={15} />;
}

function RunCard({
  run,
  latest,
  onInterrupt,
  onResolveApproval,
  onResume,
}: {
  run: AgentRunProjection;
  latest: boolean;
  onInterrupt: (run: AgentRunProjection) => void;
  onResolveApproval: (approvalId: string, decision: ApprovalResolution['decision']) => void;
  onResume: (run: AgentRunProjection) => void;
}) {
  const assistantMessages = run.messages.filter((message) => message.role === 'assistant' && message.text.trim());
  const canResume = Boolean(run.prompt && (run.status === 'failed' || run.status === 'interrupted'));
  return (
    <details
      className={styles.runCard}
      data-testid="agent-run"
      data-run-id={run.id}
      data-status={run.status}
      open={latest || activeStatuses(run.status)}
    >
      <summary className={styles.runSummary}>
        <BotMark
          botId={`workbench:${run.agentId}`}
          state={run.visualState}
          size={38}
          className={styles.runAvatar}
          label={run.agentId}
        />
        <span className={styles.runIdentity}>
          <strong>{run.label}</strong>
          <small>{statusLine(run)}</small>
        </span>
        <span className={styles.runStatus} data-status={run.status}>{statusLabel(run.status)}</span>
        <ChevronDown className={styles.chevron} size={16} />
      </summary>

      <div className={styles.runBody}>
        <div className={styles.runMeta}>
          <span><Cpu size={13} />{run.provider || 'mahayana'}{run.model ? ` / ${run.model}` : ''}</span>
          <span><Clock3 size={13} />{formatDuration(run)}</span>
          <span><Bot size={13} />{run.mode || 'agent'}</span>
        </div>

        {run.prompt ? <div className={styles.prompt}><span>任务</span><p>{run.prompt}</p></div> : null}

        {run.steps.length ? (
          <div className={styles.timeline} data-testid="agent-step-timeline">
            {run.steps.map((step) => (
              <div className={styles.step} data-testid="agent-step" data-status={step.status} data-kind={step.kind} key={step.id}>
                <span className={styles.stepRail}><StepIcon status={step.status} /></span>
                <span className={styles.stepCopy}>
                  <strong>{step.title}</strong>
                  {step.detail ? <small>{step.detail}</small> : null}
                  {typeof step.progress === 'number' && typeof step.total === 'number' && step.total > 0 ? (
                    <span className={styles.progress} aria-label={`${step.progress}/${step.total}`}>
                      <i style={{ width: `${Math.min(100, Math.max(0, (step.progress / step.total) * 100))}%` }} />
                    </span>
                  ) : null}
                </span>
                <time>{new Date(step.updatedAtMs).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</time>
              </div>
            ))}
          </div>
        ) : null}

        {run.approvals.map((approval) => (
          <section className={styles.approval} data-testid="agent-approval" key={approval.approvalId}>
            <ShieldAlert size={20} />
            <div>
              <strong>{approval.subject || approval.capability}</strong>
              <p>{approval.detail || approval.reason}</p>
              {approval.proposedRule ? <code>{approval.proposedRule}</code> : null}
              {approval.decision ? <small>已处理：{approval.decision}</small> : (
                <span className={styles.approvalActions}>
                  <button type="button" onClick={() => onResolveApproval(approval.approvalId, 'allow-once')}>仅本次允许</button>
                  <button type="button" onClick={() => onResolveApproval(approval.approvalId, 'allow-session')}>本次会话允许</button>
                  <button type="button" onClick={() => onResolveApproval(approval.approvalId, 'deny')}>拒绝</button>
                </span>
              )}
            </div>
          </section>
        ))}

        {run.observations.length ? (
          <div className={styles.observations}>
            {run.observations.map((item) => (
              <span key={`${item.kind}:${item.id}`} data-kind={item.kind}>
                {item.kind === 'subagent' ? <Bot size={14} /> : item.kind === 'async-task' ? <Terminal size={14} /> : <Circle size={12} />}
                <b>{item.label}</b>
                {item.status ? <small>{item.status}</small> : null}
              </span>
            ))}
          </div>
        ) : null}

        {run.cards.map((item) => (
          <section className={styles.artifact} data-testid="agent-artifact" key={item.id}>
            <FileText size={18} />
            <div><strong>{transcriptCardTitle(item.card)}</strong><pre>{jsonPreview(item.card)}</pre></div>
          </section>
        ))}

        {run.toolResults.map((item) => (
          <details className={styles.toolResult} data-testid="agent-tool-result" key={item.id}>
            <summary><Terminal size={15} /><strong>{item.server}</strong><span>{item.tool}</span></summary>
            <pre>{jsonPreview(item.result)}</pre>
          </details>
        ))}

        {assistantMessages.length ? (
          <div className={styles.agentOutput} data-testid="agent-output">
            <strong>结果</strong>
            {assistantMessages.map((message) => <p key={message.id}>{message.text}</p>)}
          </div>
        ) : null}

        {run.error ? <div className={styles.runError}><XCircle size={16} /><span>{run.error}</span></div> : null}

        <footer className={styles.runFooter}>
          <span>{run.usage ? `${run.usage.totalTokens.toLocaleString()} tokens` : 'Mahayana runtime'}</span>
          <span className={styles.runActions}>
            {run.interruptible && run.operationId && activeStatuses(run.status) ? (
              <button type="button" data-testid="agent-stop" onClick={() => onInterrupt(run)}><Square size={13} />停止</button>
            ) : null}
            {canResume ? (
              <button type="button" data-testid="agent-resume" onClick={() => onResume(run)}><RotateCcw size={13} />继续任务</button>
            ) : null}
          </span>
        </footer>
      </div>
    </details>
  );
}

function ensurePortalRoot(id: string, parent: HTMLElement | null, before?: Element | null): HTMLElement | null {
  const existing = document.getElementById(id);
  const previousParent = existing?.parentElement instanceof HTMLElement ? existing.parentElement : null;
  if (!parent) {
    if (previousParent) delete previousParent.dataset.mahayanaAvatarPortal;
    existing?.remove();
    return null;
  }
  const root = existing || document.createElement('div');
  root.id = id;
  root.className = styles.portalRoot;
  if (before instanceof HTMLElement) {
    root.dataset.sourceBotId = before.dataset.botId || '';
    root.dataset.sourceLabel = before.getAttribute('aria-label') || '';
  }
  if (previousParent && previousParent !== parent) delete previousParent.dataset.mahayanaAvatarPortal;
  if (root.parentElement !== parent) {
    if (before) parent.insertBefore(root, before);
    else parent.appendChild(root);
  }
  parent.dataset.mahayanaAvatarPortal = 'true';
  return root;
}

function directChildBotMark(parent: HTMLElement): HTMLElement | null {
  return Array.from(parent.children).find((child) =>
    child instanceof HTMLElement && child.dataset.engine === 'fabushi-motion-v2',
  ) as HTMLElement | null;
}

export default function MahayanaAgentWorkbench() {
  const [snapshot, dispatch] = useReducer(agentWorkbenchReducer, undefined, loadSnapshot);
  const [targets, setTargets] = useState<PortalTargets>(EMPTY_TARGETS);
  const [actionError, setActionError] = useState<string | null>(null);
  const snapshotRef = useRef(snapshot);
  const hiddenMarksRef = useRef(new Map<HTMLElement, string>());
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
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
    } catch (error) {
      console.warn('Failed to persist Mahayana agent workbench', error);
    }
  }, [snapshot]);

  useEffect(() => {
    let disposed = false;
    const hideOriginal = (element: HTMLElement | null) => {
      if (!element || hiddenMarksRef.current.has(element)) return;
      hiddenMarksRef.current.set(element, element.style.display);
      element.style.display = 'none';
      element.dataset.mahayanaAvatarReplaced = 'true';
    };
    const restoreOriginal = (element: HTMLElement) => {
      const display = hiddenMarksRef.current.get(element);
      if (display === undefined) return;
      element.style.display = display;
      delete element.dataset.mahayanaAvatarReplaced;
      hiddenMarksRef.current.delete(element);
    };

    const refresh = () => {
      if (disposed) return;
      const workspace = document.querySelector<HTMLElement>('[data-testid="messenger-workspace"]');
      const messageArea = workspace?.querySelector<HTMLElement>('[class*="messageArea"]') || null;
      const timeline = ensurePortalRoot('mahayana-agent-workbench-portal', messageArea);

      const status = workspace?.querySelector<HTMLElement>('[data-testid="conversation-status"]') || null;
      const identity = status?.parentElement?.parentElement instanceof HTMLElement
        ? status.parentElement.parentElement
        : null;
      const identityOriginal = identity ? directChildBotMark(identity) : null;
      const headerAvatar = ensurePortalRoot('mahayana-agent-header-avatar', identity, identityOriginal);
      hideOriginal(identityOriginal);

      const activeButton = activePeerButton();
      const peerOriginal = activeButton ? directChildBotMark(activeButton) : null;
      const peerAvatar = ensurePortalRoot('mahayana-agent-peer-avatar', activeButton, peerOriginal);
      hideOriginal(peerOriginal);

      const profileCard = workspace?.querySelector<HTMLElement>('aside [class*="profileCard"]') || null;
      const infoOriginal = profileCard ? directChildBotMark(profileCard) : null;
      const infoAvatar = ensurePortalRoot('mahayana-agent-info-avatar', profileCard, infoOriginal);
      hideOriginal(infoOriginal);

      const currentlyReplaced = new Set<HTMLElement>(
        [identityOriginal, peerOriginal, infoOriginal].filter((element): element is HTMLElement => Boolean(element)),
      );
      for (const element of [...hiddenMarksRef.current.keys()]) {
        if (!element.isConnected || !currentlyReplaced.has(element)) restoreOriginal(element);
      }

      const peerTestId = activeButton?.getAttribute('data-testid');
      if (peerTestId) {
        const conversationKey = conversationKeyFromPeerTestId(peerTestId);
        if (conversationKey && snapshotRef.current.activeConversationKey !== conversationKey) {
          dispatch({ type: 'set-active-conversation', conversationKey });
        }
      }

      setTargets((current) =>
        current.timeline === timeline &&
        current.headerAvatar === headerAvatar &&
        current.peerAvatar === peerAvatar &&
        current.infoAvatar === infoAvatar
          ? current
          : { timeline, headerAvatar, peerAvatar, infoAvatar },
      );
    };

    const observer = new MutationObserver(refresh);
    observer.observe(document.body, { childList: true, subtree: true });
    const interval = window.setInterval(refresh, 500);
    refresh();
    return () => {
      disposed = true;
      observer.disconnect();
      window.clearInterval(interval);
      ['mahayana-agent-workbench-portal', 'mahayana-agent-header-avatar', 'mahayana-agent-peer-avatar', 'mahayana-agent-info-avatar']
        .forEach((id) => document.getElementById(id)?.remove());
      hiddenMarksRef.current.forEach((display, element) => {
        element.style.display = display;
        delete element.dataset.mahayanaAvatarReplaced;
      });
      hiddenMarksRef.current.clear();
    };
  }, []);

  useEffect(() => {
    const onSubmitCapture = (event: Event) => {
      const form = event.target instanceof HTMLFormElement ? event.target : null;
      const input = form?.querySelector<HTMLTextAreaElement>('[data-testid="messenger-input"]');
      if (!form || !input) return;
      const peer = activePeerContext();
      const activeButton = activePeerButton();
      const activeTestId = activeButton?.getAttribute('data-testid') || '';
      if (!peer || !activeTestId.startsWith('peer-selfhosted:')) return;
      if (!['bot', 'agent'].includes(peer.kind.toLowerCase())) return;
      const text = input.value.trim();
      if (!text) return;

      event.preventDefault();
      event.stopImmediatePropagation();
      const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value')?.set;
      setter?.call(input, '');
      input.dispatchEvent(new Event('input', { bubbles: true }));

      const knownBotIds = snapshotRef.current.knownBotIds;
      const agentId = knownBotIds.includes(peer.actorId) ? peer.actorId : 'mahayana-assistant';
      const command: Extract<RuntimeCommand, { type: 'chat.send' }> = {
        type: 'chat.send',
        requestId: requestId('selfhosted-agent'),
        text,
        agentId,
        mode: 'agent',
      };
      void executeAgentCommand(command, {
        conversationKey: peer.peerKey,
        agentId,
      }).catch((error) => setActionError(error instanceof Error ? error.message : String(error)));
    };

    document.addEventListener('submit', onSubmitCapture, true);
    return () => document.removeEventListener('submit', onSubmitCapture, true);
  }, []);

  const visibleRuns = useMemo(() => {
    const exact = snapshot.runs.filter((run) => run.conversationKey === snapshot.activeConversationKey);
    return bounded(exact, 6);
  }, [snapshot.activeConversationKey, snapshot.runs]);

  const activeRun = visibleRuns.length ? visibleRuns[visibleRuns.length - 1] : undefined;
  const avatarState = activeRun?.visualState || 'idle';
  const avatarBotId = activeRun?.agentId || activePeerContext()?.actorId || 'mahayana-assistant';
  const portalIdentity = (target: HTMLElement | null, fallback: string) => ({
    botId: target?.dataset.sourceBotId || fallback,
    label: target?.dataset.sourceLabel || avatarBotId,
  });
  const headerIdentity = portalIdentity(targets.headerAvatar, `peer:bot:${avatarBotId}`);
  const peerIdentity = portalIdentity(targets.peerAvatar, `peer:bot:${avatarBotId}`);
  const infoIdentity = portalIdentity(targets.infoAvatar, `peer:bot:${avatarBotId}`);

  useEffect(() => {
    const status = document.querySelector<HTMLElement>('[data-testid="conversation-status"]');
    if (!status) return;
    if (!status.dataset.mahayanaOriginalText) status.dataset.mahayanaOriginalText = status.textContent || '';
    if (activeRun) {
      status.textContent = statusLine(activeRun);
      status.dataset.mahayanaState = avatarState;
    } else if (status.dataset.mahayanaOriginalText) {
      status.textContent = status.dataset.mahayanaOriginalText;
      delete status.dataset.mahayanaState;
    }
  }, [activeRun, avatarState]);

  const interrupt = (run: AgentRunProjection) => {
    if (!run.operationId || !window.mahayana?.invoke) return;
    setActionError(null);
    void window.mahayana.invoke<void>('feature.interrupt', { operationId: run.operationId })
      .catch((error) => setActionError(error instanceof Error ? error.message : String(error)));
  };

  const resolveApproval = (approvalId: string, decision: ApprovalResolution['decision']) => {
    if (!window.mahayana?.invoke) return;
    setActionError(null);
    void window.mahayana.invoke<void>('feature.approval.resolve', {
      resolution: { approvalId, decision },
    }).catch((error) => setActionError(error instanceof Error ? error.message : String(error)));
  };

  const resume = (run: AgentRunProjection) => {
    setActionError(null);
    const command: Extract<RuntimeCommand, { type: 'chat.send' }> = {
      type: 'chat.send',
      requestId: requestId('resume-agent'),
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

  const panel = (
    <section className={styles.workbench} data-testid="agent-workbench" data-empty={!visibleRuns.length || undefined}>
      {actionError ? <div className={styles.bridgeError}><XCircle size={14} />{actionError}</div> : null}
      {visibleRuns.map((run, index) => (
        <RunCard
          key={run.id}
          run={run}
          latest={index === visibleRuns.length - 1}
          onInterrupt={interrupt}
          onResolveApproval={resolveApproval}
          onResume={resume}
        />
      ))}
      {visibleRuns.length > 1 ? (
        <button className={styles.clearHistory} type="button" onClick={() => dispatch({ type: 'clear-history' })}>
          <PauseCircle size={13} />清理运行历史
        </button>
      ) : null}
    </section>
  );

  return (
    <>
      {targets.timeline && visibleRuns.length ? createPortal(panel, targets.timeline) : null}
      {targets.headerAvatar ? createPortal(
        <BotMark botId={headerIdentity.botId} state={avatarState} size={40} className={styles.portalAvatar} label={headerIdentity.label} />,
        targets.headerAvatar,
      ) : null}
      {targets.peerAvatar ? createPortal(
        <BotMark botId={peerIdentity.botId} state={avatarState} size={48} className={styles.portalAvatar} label={peerIdentity.label} />,
        targets.peerAvatar,
      ) : null}
      {targets.infoAvatar ? createPortal(
        <BotMark botId={infoIdentity.botId} state={avatarState} size={92} className={styles.portalAvatar} label={infoIdentity.label} />,
        targets.infoAvatar,
      ) : null}
    </>
  );
}
