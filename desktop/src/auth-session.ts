export function authFailureMessage(cause: unknown): string {
  if (cause instanceof Error) return cause.message;
  return String(cause ?? '');
}

const terminalAuthFailurePatterns = [
  /refresh_token_reused/i,
  /refresh_token_expired/i,
  /session[_ -]?revoked/i,
  /session[_ -]?expired/i,
  /登录会话已撤销/u,
  /登录已过期/u,
  /请重新登录/u,
  /\bHTTP\s+401\b/i,
  /\bunauthorized\b/i,
];

/**
 * Terminal authentication failures require a local session reset. Transient
 * Host/network failures must not destroy a valid local-first desktop shell.
 */
export function isTerminalAuthSessionFailure(cause: unknown): boolean {
  const message = authFailureMessage(cause).trim();
  return Boolean(message) && terminalAuthFailurePatterns.some((pattern) => pattern.test(message));
}
