import type { CommandAccepted, RuntimeCommand } from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  MAHAYANA_COMMAND_EVENT_NAME,
  MAHAYANA_RUNTIME_EVENT_NAME,
  type MahayanaCommandBridgeContext,
  type MahayanaCommandBridgeDetail,
} from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';

const CLAIMS_KEY = 'fabushi.desktop.selfhosted-mahayana-invocations.v1';
const CLAIMS_VERSION = 1;
const MAX_CLAIMS = 256;
const INFLIGHT_TTL_MS = 5 * 60_000;
const RUNTIME_AGENT_ID = 'mahayana-assistant';

type ChatSendCommand = Extract<RuntimeCommand, { type: 'chat.send' }>;

type SelfHostedBotInvocation = {
  id: string;
  botId: string;
  senderId: string;
  conversationId: string;
  text: string;
  command?: string;
  createdAtMs: number;
};

type InvocationClaim = {
  state: 'inflight' | 'accepted' | 'failed';
  updatedAtMs: number;
};

type InvocationClaimJournal = {
  version: 1;
  claims: Record<string, InvocationClaim>;
};

function asRecord(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function asNonEmptyString(value: unknown): string | null {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function parseInvocation(detail: unknown): SelfHostedBotInvocation | null {
  const runtimeEvent = asRecord(detail);
  if (runtimeEvent?.type !== 'messaging.event') return null;
  const envelope = asRecord(runtimeEvent.envelope);
  const event = asRecord(envelope?.event);
  if (event?.type !== 'botInvocationRequested') return null;
  const invocation = asRecord(event.invocation);
  const formattedText = asRecord(invocation?.text);
  const id = asNonEmptyString(invocation?.id);
  const botId = asNonEmptyString(invocation?.botId);
  const senderId = asNonEmptyString(invocation?.senderId);
  const conversationId = asNonEmptyString(invocation?.conversationId);
  const text = asNonEmptyString(formattedText?.text);
  if (!id || !botId || !senderId || !conversationId || !text) return null;
  return {
    id,
    botId,
    senderId,
    conversationId,
    text,
    command: asNonEmptyString(invocation?.command) ?? undefined,
    createdAtMs: typeof invocation?.createdAtMs === 'number' && Number.isFinite(invocation.createdAtMs)
      ? invocation.createdAtMs
      : Date.now(),
  };
}

function emptyJournal(): InvocationClaimJournal {
  return { version: CLAIMS_VERSION, claims: {} };
}

function readJournal(): InvocationClaimJournal {
  if (typeof window === 'undefined') return emptyJournal();
  try {
    const parsed = JSON.parse(window.localStorage.getItem(CLAIMS_KEY) || 'null') as unknown;
    const record = asRecord(parsed);
    const rawClaims = asRecord(record?.claims);
    if (record?.version !== CLAIMS_VERSION || !rawClaims) return emptyJournal();
    const claims = Object.fromEntries(
      Object.entries(rawClaims)
        .map(([id, value]) => {
          const claim = asRecord(value);
          const state = claim?.state;
          const updatedAtMs = claim?.updatedAtMs;
          if (
            !id ||
            (state !== 'inflight' && state !== 'accepted' && state !== 'failed') ||
            typeof updatedAtMs !== 'number' ||
            !Number.isFinite(updatedAtMs)
          ) return null;
          return [id, { state, updatedAtMs } satisfies InvocationClaim] as const;
        })
        .filter((entry): entry is readonly [string, InvocationClaim] => Boolean(entry))
        .sort((left, right) => right[1].updatedAtMs - left[1].updatedAtMs)
        .slice(0, MAX_CLAIMS),
    );
    return { version: CLAIMS_VERSION, claims };
  } catch {
    return emptyJournal();
  }
}

function persistJournal(journal: InvocationClaimJournal): void {
  if (typeof window === 'undefined') return;
  try {
    const claims = Object.fromEntries(
      Object.entries(journal.claims)
        .sort((left, right) => right[1].updatedAtMs - left[1].updatedAtMs)
        .slice(0, MAX_CLAIMS),
    );
    window.localStorage.setItem(CLAIMS_KEY, JSON.stringify({ version: CLAIMS_VERSION, claims }));
  } catch {
    // This journal is only an idempotency aid. Storage pressure must never block a live task.
  }
}

function shouldSkipClaim(claim: InvocationClaim | undefined, nowMs: number): boolean {
  if (!claim) return false;
  if (claim.state === 'accepted') return true;
  return claim.state === 'inflight' && nowMs - claim.updatedAtMs < INFLIGHT_TTL_MS;
}

function dispatchBridgeDetail(detail: MahayanaCommandBridgeDetail): void {
  window.dispatchEvent(new CustomEvent<MahayanaCommandBridgeDetail>(MAHAYANA_COMMAND_EVENT_NAME, { detail }));
}

function messageForInvocation(invocation: SelfHostedBotInvocation): string {
  if (!invocation.command) return invocation.text;
  return invocation.text.startsWith('/') ? invocation.text : `/${invocation.command} ${invocation.text}`.trim();
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/**
 * Bridges the Rust-native self-hosted messaging BotInvocationRequested event into
 * the single Mahayana Agent runtime. The authenticated human message remains in
 * the Rust messaging store; this bridge never impersonates a Bot to write back
 * into messaging. Workbench projection identity is the self-hosted Bot while the
 * actual executor stays the canonical Mahayana runtime agent.
 */
export function installSelfHostedMahayanaInvocationBridge(): () => void {
  if (typeof window === 'undefined') return () => {};
  const journal = readJournal();
  let disposed = false;

  const mark = (id: string, state: InvocationClaim['state']) => {
    journal.claims[id] = { state, updatedAtMs: Date.now() };
    persistJournal(journal);
  };

  const executeInvocation = async (invocation: SelfHostedBotInvocation) => {
    const bridge = window.mahayana;
    if (!bridge?.invoke || disposed) return;

    const nowMs = Date.now();
    if (shouldSkipClaim(journal.claims[invocation.id], nowMs)) return;
    mark(invocation.id, 'inflight');

    const requestId = `selfhosted-bot:${invocation.id}`;
    const projectionCommand: ChatSendCommand = {
      type: 'chat.send',
      requestId,
      text: messageForInvocation(invocation),
      agentId: invocation.botId,
      mode: 'agent',
      modeStatement: [
        `Execute as the Fabushi self-hosted Bot ${invocation.botId}.`,
        `The authenticated sender is ${invocation.senderId}.`,
        `The source Messenger conversation is ${invocation.conversationId}.`,
        'Use Mahayana planning, tools, approvals and multiple steps when the task requires them.',
        'Return the final result to the active Messenger Agent Workbench surface.',
      ].join(' '),
    };
    const context: MahayanaCommandBridgeContext = {
      conversationKey: `selfhosted:${invocation.conversationId}`,
      conversationId: invocation.conversationId,
      agentId: invocation.botId,
    };

    dispatchBridgeDetail({ phase: 'dispatch', command: projectionCommand, context });

    // The self-hosted Bot id is a messaging identity, not permission to invent a
    // second runtime agent. Execute on Mahayana's canonical agent while keeping
    // the Bot identity in the projection context above.
    const runtimeCommand: ChatSendCommand = {
      ...projectionCommand,
      agentId: RUNTIME_AGENT_ID,
      conversationId: undefined,
    };

    try {
      const accepted = await bridge.invoke<CommandAccepted>('feature.execute', { command: runtimeCommand });
      mark(invocation.id, 'accepted');
      dispatchBridgeDetail({ phase: 'accepted', command: projectionCommand, accepted, context });
    } catch (error) {
      mark(invocation.id, 'failed');
      dispatchBridgeDetail({
        phase: 'failed',
        command: projectionCommand,
        error: errorMessage(error),
        context,
      });
    }
  };

  const onRuntimeEvent = (event: Event) => {
    const invocation = parseInvocation((event as CustomEvent<unknown>).detail);
    if (!invocation) return;
    void executeInvocation(invocation);
  };

  window.addEventListener(MAHAYANA_RUNTIME_EVENT_NAME, onRuntimeEvent);
  return () => {
    disposed = true;
    window.removeEventListener(MAHAYANA_RUNTIME_EVENT_NAME, onRuntimeEvent);
  };
}
