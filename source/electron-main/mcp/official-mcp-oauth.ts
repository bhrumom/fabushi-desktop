export interface OfficialMcpCredential {
  token: string;
  refreshToken?: string;
  expiresAt?: number;
  connectionId: string;
}
export interface OfficialMcpOAuthPorts {
  start(pluginId: string): Promise<{ attemptId: string; authorizationUrl: string }>;
  poll(attemptId: string): Promise<{ status: string; credential?: OfficialMcpCredential }>;
  acknowledge(attemptId: string): Promise<void>;
  cancel(attemptId: string): Promise<void>;
  refresh(credential: OfficialMcpCredential): Promise<OfficialMcpCredential>;
  revoke(credential: OfficialMcpCredential): Promise<void>;
  changed(): void;
}

/** Provider secrets remain in native memory/vault; only the Fabushi origin gets a Fabushi session. */
export function createOfficialMcpOAuthPorts(deps: {
  getAccessToken(backendUrl: string): Promise<string>;
  changed(): void;
  fetch?: typeof fetch;
}): OfficialMcpOAuthPorts {
  const origin = "https://api.ombhrum.com";
  const request = async (path: string, body?: unknown) => {
    const token = await deps.getAccessToken(origin);
    const response = await (deps.fetch ?? fetch)(origin + path, {
      method: "POST", redirect: "error", signal: AbortSignal.timeout(30_000),
      headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
      body: JSON.stringify(body ?? {}),
    });
    if (!response.ok) {
      await response.body?.cancel();
      throw new Error(response.status === 503
        ? "Fabushi 服务授权尚未完成配置，请联系市场管理员。"
        : "服务授权失败或已过期，请重新连接。未自动重试。");
    }
    const text = await response.text();
    if (text.length > 128_000) throw new Error("Invalid service authorization response.");
    const value: unknown = JSON.parse(text);
    if (typeof value !== "object" || value == null || Array.isArray(value)) throw new Error("Invalid service authorization response.");
    return value as Record<string, unknown>;
  };
  const id = (value: string) => {
    if (!/^[a-zA-Z0-9_-]{1,100}$/.test(value)) throw new Error("Invalid service authorization id.");
    return value;
  };
  const credential = (value: unknown): OfficialMcpCredential => {
    if (typeof value !== "object" || value == null) throw new Error("Invalid service credential.");
    const c = value as OfficialMcpCredential;
    if (typeof c.token !== "string" || !c.token || c.token.length > 32_768 || /[\r\n]/.test(c.token) ||
        typeof c.connectionId !== "string" ||
        c.refreshToken !== undefined && (typeof c.refreshToken !== "string" || c.refreshToken.length > 32_768 || /[\r\n]/.test(c.refreshToken)) ||
        c.expiresAt !== undefined && (!Number.isFinite(c.expiresAt) || c.expiresAt <= 0)) throw new Error("Invalid service credential.");
    id(c.connectionId);
    return { token: c.token, connectionId: c.connectionId,
      ...(c.refreshToken ? { refreshToken: c.refreshToken } : {}), ...(c.expiresAt ? { expiresAt: c.expiresAt } : {}) };
  };
  return {
    async start(pluginId) {
      const value = await request("/api/mcp/oauth/start", { pluginId });
      if (typeof value.attemptId !== "string" || typeof value.authorizationUrl !== "string") throw new Error("Invalid service authorization start.");
      const url = new URL(value.authorizationUrl);
      if (url.origin !== origin || url.pathname !== "/api/mcp/oauth/authorize" || url.username || url.password || url.hash)
        throw new Error("Untrusted service authorization URL.");
      return { attemptId: id(value.attemptId), authorizationUrl: url.href };
    },
    async poll(attemptId) {
      const value = await request(`/api/mcp/oauth/attempts/${id(attemptId)}`);
      if (!["pending", "exchanging", "ready", "failed", "expired", "cancelled", "consumed"].includes(String(value.status)))
        throw new Error("Invalid service authorization status.");
      return { status: String(value.status), ...(value.status === "ready" ? { credential: credential(value.credential) } : {}) };
    },
    acknowledge: async attemptId => { await request(`/api/mcp/oauth/attempts/${id(attemptId)}/ack`); },
    cancel: async attemptId => { await request(`/api/mcp/oauth/attempts/${id(attemptId)}/cancel`); },
    refresh: async c => credential(await request(`/api/mcp/connections/${id(c.connectionId)}/refresh`, { refreshToken: c.refreshToken })),
    revoke: async c => { await request(`/api/mcp/connections/${id(c.connectionId)}/revoke`, { token: c.token, refreshToken: c.refreshToken }); },
    changed: deps.changed,
  };
}
