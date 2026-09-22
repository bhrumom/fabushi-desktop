export const BRIDGE_MISSING_HANDLER = 'bridge/missing-handler' as const;
export const BRIDGE_INVOKE_FAILED = 'bridge/invoke-failed' as const;
export const BRIDGE_UNTRUSTED_SENDER = 'bridge/untrusted-sender' as const;

export type EdgeFailureCode =
  | typeof BRIDGE_MISSING_HANDLER
  | typeof BRIDGE_INVOKE_FAILED
  | typeof BRIDGE_UNTRUSTED_SENDER
  | (string & {});

export interface EdgeFailure {
  readonly code: EdgeFailureCode;
  readonly detail: string;
}

export type EdgeReply<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly failure: EdgeFailure };

export type EdgeMethodSpec = { readonly args: 'none' | 'object' };

export interface EdgeDescriptor<
  TMethods extends Record<string, EdgeMethodSpec>,
  TEvents extends readonly string[],
> {
  readonly edge: string;
  readonly methods: TMethods;
  readonly events: TEvents;
}

export function defineEdge<
  const TMethods extends Record<string, EdgeMethodSpec>,
  const TEvents extends readonly string[],
>(edge: string, methods: TMethods, events: TEvents): EdgeDescriptor<TMethods, TEvents> {
  return Object.freeze({ edge, methods, events });
}

export function callChannel(edge: string, method: string): string {
  return `fabushi-edge:${edge}:call:${method}`;
}

export function pushChannel(edge: string, event: string): string {
  return `fabushi-edge:${edge}:event:${event}`;
}

export function isEdgeReply(value: unknown): value is EdgeReply<unknown> {
  return Boolean(value && typeof value === 'object' && 'ok' in value && typeof (value as { ok?: unknown }).ok === 'boolean');
}

export class EdgeInvocationError extends Error {
  readonly code: EdgeFailureCode;
  readonly detail: string;

  constructor(problem: EdgeFailure) {
    super(`${problem.code}: ${problem.detail}`);
    this.name = 'EdgeInvocationError';
    this.code = problem.code;
    this.detail = problem.detail;
  }
}
