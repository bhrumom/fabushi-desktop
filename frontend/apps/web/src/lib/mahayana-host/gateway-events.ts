export const MAHAYANA_GATEWAY_PROTOCOL_VERSION = 1 as const;

export const MAHAYANA_GATEWAY_EVENT_NAMES = [
  'message.start',
  'message.delta',
  'message.interim',
  'message.complete',
  'reasoning.delta',
  'thinking.delta',
  'tool.generating',
  'tool.start',
  'tool.complete',
  'approval.request',
  'clarify.request',
  'subagent.start',
  'subagent.progress',
  'subagent.complete',
] as const;

export type MahayanaGatewayEventName = typeof MAHAYANA_GATEWAY_EVENT_NAMES[number];
export type MahayanaGatewayPayload = Record<string, unknown>;

/**
 * Renderer-facing projection of the Rust Mahayana gateway envelope.
 * Rust owns sequencing/replay and agent behavior; TypeScript only projects
 * ordered events into the active product surface.
 */
export type MahayanaGatewayEventEnvelope = {
  protocolVersion: typeof MAHAYANA_GATEWAY_PROTOCOL_VERSION;
  sessionId: string;
  turnId: string;
  seq: number;
  timestamp: string;
  replayEpoch: string;
  type: MahayanaGatewayEventName;
  payload: MahayanaGatewayPayload;
};

export type MahayanaGatewayNotification = {
  jsonrpc: '2.0';
  method: 'event';
  params: MahayanaGatewayEventEnvelope;
};

export function isMahayanaGatewayEventEnvelope(value: unknown): value is MahayanaGatewayEventEnvelope {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const candidate = value as Partial<MahayanaGatewayEventEnvelope>;
  return candidate.protocolVersion === MAHAYANA_GATEWAY_PROTOCOL_VERSION
    && typeof candidate.sessionId === 'string'
    && typeof candidate.turnId === 'string'
    && Number.isSafeInteger(candidate.seq)
    && typeof candidate.timestamp === 'string'
    && typeof candidate.replayEpoch === 'string'
    && typeof candidate.type === 'string'
    && (MAHAYANA_GATEWAY_EVENT_NAMES as readonly string[]).includes(candidate.type)
    && Boolean(candidate.payload && typeof candidate.payload === 'object' && !Array.isArray(candidate.payload));
}
