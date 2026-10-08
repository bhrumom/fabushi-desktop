import { randomUUID } from "node:crypto";
import { formatMcpCustomInstructionToolNote } from "../../shared/mcp-custom-instructions.js";
import { officialMcpEntry } from "../../shared/node/mcp/fabushi-official-catalog.js";
import { McpResult, McpSuccess } from "../../packages/proto/generated/agent/v1/mcp_exec_pb.js";
import { generatedMcpResultFactory } from "../../shared/node/mcp/mcp-result-factory.js";

interface Install { token?: string; revision: string; disabledTools: string[]; instructions?: string }
type State = Record<string, Install>;
export interface OfficialMcpPorts {
  getScope(): Promise<string>;
  read(scope: string): Promise<string | null>;
  write(scope: string, value: string): Promise<void>;
  fetch(input: string, init: RequestInit): Promise<Response>;
}
interface RemoteTool { name: string; description?: string; inputSchema: unknown }
class AuthRequired extends Error {}
const MAX_BODY = 4 * 1024 * 1024;
const TIMEOUT = 60_000;
const object = (v: unknown): v is Record<string, any> => typeof v === "object" && v !== null && !Array.isArray(v);

/** Native edge reached only through the existing Coordinator/Host MCP facade. */
export class OfficialMcpService {
  private mutation = Promise.resolve();
  private disposed = false;
  private pending = new Set<AbortController>();
  constructor(private readonly ports: OfficialMcpPorts) {}
  owns(id: string): boolean { return officialMcpEntry(id) !== undefined; }
  private async scope(): Promise<string> {
    if (this.disposed) throw new Error("Official MCP service is disposed.");
    const scope = await this.ports.getScope();
    if (!/^[a-f0-9]{64}$/.test(scope)) throw new Error("Sign in to Fabushi before connecting a service.");
    return scope;
  }
  private async state(scope: string): Promise<State> {
    const text = await this.ports.read(scope);
    if (!text) return {};
    const value: unknown = JSON.parse(text);
    if (!object(value)) throw new Error("Invalid official MCP install state.");
    const state: State = {};
    for (const [id, install] of Object.entries(value)) {
      if (!this.owns(id) || !object(install) || typeof install.revision !== "string" ||
          install.token !== undefined && typeof install.token !== "string" ||
          install.instructions !== undefined && typeof install.instructions !== "string" ||
          !Array.isArray(install.disabledTools) || !install.disabledTools.every((v: unknown) => typeof v === "string"))
        throw new Error("Invalid official MCP install state.");
      state[id] = install as Install;
    }
    return state;
  }
  private async mutate(change: (state: State) => void, expected?: Awaited<ReturnType<OfficialMcpService["snapshot"]>>): Promise<void> {
    const scope = await this.scope();
    if (expected && scope !== expected.scope) throw new Error("Fabushi account changed.");
    const task = this.mutation.then(async () => {
      if (await this.scope() !== scope) throw new Error("Fabushi account changed.");
      const state = await this.state(scope);
      if (expected && state[expected.entry.id]?.revision !== expected.install.revision) throw new Error("Connector changed.");
      change(state);
      if (await this.scope() !== scope) throw new Error("Fabushi account changed.");
      await this.ports.write(scope, JSON.stringify(state));
    });
    this.mutation = task.catch(() => {});
    await task;
  }
  async install(id: string, values?: Record<string, string>): Promise<void> {
    if (!this.owns(id)) throw new Error("Unknown official MCP plugin.");
    const token = values?.ACCESS_TOKEN?.trim();
    if (token && (token.length > 32_768 || /[\r\n]/.test(token))) throw new Error("Invalid provider credential.");
    await this.mutate(state => {
      state[id] = { ...(state[id] ?? { disabledTools: [] }), ...(token ? { token } : {}), revision: randomUUID() };
    });
  }
  async remove(id: string): Promise<void> { await this.mutate(state => { delete state[id]; }); }
  async disconnect(id: string): Promise<void> {
    await this.mutate(state => { if (state[id]) { delete state[id].token; state[id].revision = randomUUID(); } });
  }
  async effective() {
    const state = await this.state(await this.scope());
    return Object.keys(state).map(id => ({ pluginId: id, name: id, displayName: officialMcpEntry(id)!.name,
      installMode: "user" as const, isEnabled: true }));
  }
  private async snapshot(id: string) {
    const scope = await this.scope(), install = (await this.state(scope))[id], entry = officialMcpEntry(id);
    if (!entry || !install) throw new Error("Official MCP connector is not installed.");
    return { scope, install, entry };
  }
  private async fence(snapshot: Awaited<ReturnType<OfficialMcpService["snapshot"]>>): Promise<void> {
    if (await this.scope() !== snapshot.scope ||
        (await this.state(snapshot.scope))[snapshot.entry.id]?.revision !== snapshot.install.revision)
      throw new Error("Connector changed while the request was running.");
  }
  private redact(value: unknown, token: string): any {
    if (typeof value === "string") return value.split(token).join("[redacted]");
    if (Array.isArray(value)) return value.map(item => this.redact(item, token));
    if (object(value)) return Object.fromEntries(Object.entries(value).map(([key, item]) =>
      [key.split(token).join("[redacted]"), this.redact(item, token)]));
    return value;
  }
  private async send(url: string, token: string, payload: unknown, signal: AbortSignal,
    session?: string, protocol?: string): Promise<Response> {
    const response = await this.ports.fetch(url, {
      method: "POST", redirect: "error", signal,
      headers: { "Content-Type": "application/json", Accept: "application/json, text/event-stream",
        Authorization: `Bearer ${token}`, ...(session ? { "Mcp-Session-Id": session } : {}),
        ...(protocol ? { "MCP-Protocol-Version": protocol } : {}) },
      body: JSON.stringify(payload),
    });
    if (response.status === 401 || response.status === 403) { await response.body?.cancel(); throw new AuthRequired("Provider authorization required."); }
    if (!response.ok) { await response.body?.cancel(); throw new Error(`Official MCP service returned HTTP ${response.status}; the call was not retried.`); }
    return response;
  }
  private async response(response: Response, expectedId: number): Promise<any> {
    const reader = response.body?.getReader();
    if (!reader) throw new Error("Empty MCP response.");
    const sse = response.headers.get("content-type")?.includes("text/event-stream");
    if (!sse && !response.headers.get("content-type")?.includes("application/json")) {
      await reader.cancel(); throw new Error("Unsupported MCP response type.");
    }
    const decoder = new TextDecoder(); let text = "", size = 0;
    try {
      while (true) {
        const chunk = await reader.read();
        if (chunk.done) break;
        size += chunk.value.byteLength;
        if (size > MAX_BODY) throw new Error("MCP response exceeded the size limit.");
        text += decoder.decode(chunk.value, { stream: true });
        if (sse) {
          text = text.replace(/\r\n/g, "\n");
          let boundary: number;
          while ((boundary = text.indexOf("\n\n")) >= 0) {
            const event = text.slice(0, boundary); text = text.slice(boundary + 2);
            const data = event.split("\n").filter(line => line.startsWith("data:"))
              .map(line => line.slice(5).trimStart()).join("\n");
            if (!data) continue;
            const message: unknown = JSON.parse(data);
            if (object(message) && message.id === expectedId) return this.unwrap(message, expectedId);
          }
        }
      }
      text += decoder.decode();
      if (sse) throw new Error("MCP stream ended without a matching response.");
      return this.unwrap(JSON.parse(text), expectedId);
    } finally { await reader.cancel().catch(() => {}); }
  }
  private unwrap(message: unknown, id: number): any {
    if (!object(message) || message.jsonrpc !== "2.0" || message.id !== id) throw new Error("Invalid MCP response correlation.");
    if (message.error) throw new Error("Official MCP server rejected the request.");
    if (!object(message.result)) throw new Error("Invalid MCP result.");
    return message.result;
  }
  private async request(id: string, method: string, params: unknown) {
    const snap = await this.snapshot(id), token = snap.install.token;
    if (!token) throw new AuthRequired("Provider authorization required.");
    const controller = new AbortController(); this.pending.add(controller);
    const timeout = setTimeout(() => controller.abort(), TIMEOUT);
    try {
      await this.fence(snap);
      const initResponse = await this.send(snap.entry.url, token, { jsonrpc: "2.0", id: 1, method: "initialize",
        params: { protocolVersion: "2025-03-26", capabilities: {}, clientInfo: { name: "Fabushi", version: "1.0.0" } } }, controller.signal);
      const session = initResponse.headers.get("mcp-session-id") ?? undefined;
      const init = await this.response(initResponse, 1);
      if (typeof init.protocolVersion !== "string" || !["2024-11-05", "2025-03-26", "2025-06-18"].includes(init.protocolVersion))
        throw new Error("Unsupported MCP protocol version.");
      const notification = await this.send(snap.entry.url, token,
        { jsonrpc: "2.0", method: "notifications/initialized" }, controller.signal, session, init.protocolVersion);
      await notification.body?.cancel();
      await this.fence(snap);
      const result = await this.response(await this.send(snap.entry.url, token,
        { jsonrpc: "2.0", id: 2, method, params }, controller.signal, session, init.protocolVersion), 2);
      await this.fence(snap);
      return this.redact(result, token);
    } finally { clearTimeout(timeout); this.pending.delete(controller); }
  }
  async tools(id: string): Promise<RemoteTool[]> {
    const snap = await this.snapshot(id);
    const tools: RemoteTool[] = []; const seen = new Set<string>(); let cursor: string | undefined;
    for (let page = 0; page < 64; page++) {
      const result = await this.request(id, "tools/list", cursor ? { cursor } : {});
      await this.fence(snap);
      if (!Array.isArray(result.tools)) throw new Error("Invalid MCP tool inventory.");
      for (const tool of result.tools) {
        if (!object(tool) || typeof tool.name !== "string" || !/^[a-zA-Z0-9_.-]{1,128}$/.test(tool.name) ||
            !object(tool.inputSchema) || tools.some(existing => existing.name === tool.name))
          throw new Error("Invalid or duplicate MCP tool.");
        tools.push({ name: tool.name, description: typeof tool.description === "string" ? tool.description : undefined, inputSchema: tool.inputSchema });
      }
      if (result.nextCursor == null) return tools;
      if (typeof result.nextCursor !== "string" || seen.has(result.nextCursor)) throw new Error("Invalid MCP pagination.");
      cursor = result.nextCursor; seen.add(cursor);
    }
    throw new Error("MCP inventory exceeded the pagination limit.");
  }
  async servers() {
    const scope = await this.scope(), state = await this.state(scope);
    return await Promise.all(Object.entries(state).map(async ([id, install]) => {
      let status = "needsAuth", statusDetail = "需要连接服务账号。一键登录尚需 Fabushi OAuth 配置。", toolCount = 0;
      if (install.token) try {
        const tools = await this.tools(id); toolCount = tools.filter(tool => !install.disabledTools.includes(tool.name)).length;
        status = "connected"; statusDetail = "";
      } catch (error) {
        status = error instanceof AuthRequired ? "needsAuth" : "error";
        statusDetail = error instanceof AuthRequired ? "授权已过期或无权访问，请更新授权。" : "官方 MCP 暂时不可用；未自动重试操作。";
      }
      if (await this.scope() !== scope) throw new Error("Fabushi account changed.");
      if ((await this.state(scope))[id]?.revision !== install.revision) throw new Error("Connector changed.");
      const entry = officialMcpEntry(id)!;
      return { id, pluginId: id, name: entry.name, serverIdentifier: id, rowServerIdentifier: id, accountKey: "default",
        transport: "http", url: entry.url, toolCount, customInstructions: install.instructions ?? "", isTeamServer: false, status, statusDetail };
    }));
  }
  async routedTools() {
    const scope = await this.scope(), state = await this.state(scope);
    const groups = await Promise.all(Object.entries(state).map(async ([id, install]) => {
      if (!install.token) return [];
      try { return (await this.tools(id)).filter(tool => !install.disabledTools.includes(tool.name)).map(tool => ({
        name: `${id}_${tool.name}`, toolName: tool.name, providerIdentifier: id, clientKey: id,
        description: tool.description ?? "", inputSchema: tool.inputSchema,
      })); } catch { return []; }
    }));
    if (await this.scope() !== scope) throw new Error("Fabushi account changed.");
    const current = await this.state(scope);
    if (Object.entries(state).some(([id, install]) => current[id]?.revision !== install.revision)) throw new Error("Connector changed.");
    return groups.flat();
  }
  async toolSummaries(id: string) {
    const snap = await this.snapshot(id);
    const tools = await this.tools(id);
    await this.fence(snap);
    return tools.map(tool => ({ name: tool.name, description: tool.description ?? "", inputSchema: tool.inputSchema,
      isDisabled: snap.install.disabledTools.includes(tool.name) }));
  }
  async toggle(id: string, toolName: string) {
    const snap = await this.snapshot(id);
    if (!(await this.tools(id)).some(tool => tool.name === toolName)) throw new Error("Unknown connector tool.");
    await this.mutate(state => {
      const install = state[id]; if (!install) throw new Error("Connector is not installed.");
      install.disabledTools = install.disabledTools.includes(toolName)
        ? install.disabledTools.filter(name => name !== toolName) : [...install.disabledTools, toolName];
      install.revision = randomUUID();
    }, snap);
    return this.toolSummaries(id);
  }
  async setInstructions(id: string, instructions: string) {
    if (typeof instructions !== "string" || instructions.length > 32_768) throw new Error("Invalid MCP instructions.");
    const snap = await this.snapshot(id);
    await this.mutate(state => { state[id]!.instructions = instructions; state[id]!.revision = randomUUID(); }, snap);
    return { servers: await this.servers() };
  }
  async execute(id: string, name: string, args: unknown) {
    try {
      const snap = await this.snapshot(id);
      if (snap.install.disabledTools.includes(name)) throw new Error("Connector tool is disabled.");
      if (!(await this.tools(id)).some(tool => tool.name === name)) throw new Error("Unknown connector tool.");
      await this.fence(snap);
      const result = await this.request(id, "tools/call", { name, arguments: args ?? {} });
      await this.fence(snap);
      if (!Array.isArray(result.content)) throw new Error("Invalid MCP tool output.");
      const content = result.content.map((item: unknown) => generatedMcpResultFactory.textItem(
        object(item) && item.type === "text" && typeof item.text === "string" ? item.text : JSON.stringify(item)));
      if (object(result.structuredContent)) content.push(generatedMcpResultFactory.textItem(JSON.stringify(result.structuredContent)));
      if (snap.install.instructions?.trim()) content.unshift(generatedMcpResultFactory.textItem(
        formatMcpCustomInstructionToolNote(snap.entry.name, snap.install.instructions.trim())));
      return new McpResult({ result: { case: "success", value: new McpSuccess({ content, isError: result.isError === true }) } });
    } catch { return generatedMcpResultFactory.error("Official MCP call failed or authorization changed. The call was not retried; confirm its outcome before retrying a write."); }
  }
  async authenticate(id: string, accountKey: string) {
    const snap = await this.snapshot(id);
    if (accountKey !== "default") return { status: "not-supported", serverName: snap.entry.name,
      message: "官方连接器当前支持一个服务账号；多账号 OAuth 配置尚未完成。" };
    if (snap.install.token) {
      try { await this.tools(id); return { status: "already-authenticated", serverName: snap.entry.name }; } catch {}
    }
    return { status: "not-supported", serverName: snap.entry.name,
      message: "Fabushi 一键 OAuth 尚未配置。可在插件设置中提供独立的服务访问令牌；Google 服务还需要 Developer Preview 资格。" };
  }
  dispose() { this.disposed = true; for (const controller of this.pending) controller.abort(); this.pending.clear(); }
}
