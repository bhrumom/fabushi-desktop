import test from "node:test";
import assert from "node:assert/strict";
import { OfficialMcpService } from "./official-mcp-service.js";
import { OFFICIAL_MCP_CATALOG, officialMcpCatalogPlugins } from "../../shared/node/mcp/fabushi-official-catalog.js";
import { loadPluginsDesktopSnapshot } from "../../../frontend/src/recovered/features/plugins/overlay/desktop.js";
import type { DesktopBridge } from "../../../frontend/src/recovered/contracts/desktop-bridge.js";
import { createOfficialMcpOAuthPorts, type OfficialMcpOAuthPorts } from "./official-mcp-oauth.js";

const ID = "fabushi-official-github";
function oauthFixture(): OfficialMcpOAuthPorts {
  return {
    start: async () => ({ attemptId: "attempt-a", authorizationUrl: "https://api.ombhrum.com/api/mcp/oauth/authorize?ticket=opaque" }),
    poll: async () => ({ status: "ready", credential: { token: "provider-oauth-access", refreshToken: "provider-oauth-refresh", connectionId: "grant-a", expiresAt: Date.now()+3_600_000 } }),
    acknowledge: async () => {}, cancel: async () => {},
    refresh: async c => ({ ...c, token: "renewed-provider-access", expiresAt: Date.now()+3_600_000 }),
    revoke: async () => {}, changed: () => {},
  };
}
const TOOL = { name: "read_issue", description: "Read an issue", inputSchema: { type: "object" } };
type Call = { url: string; init: RequestInit; payload: any };
function harness(handler?: (call: Call) => Response | Promise<Response>, oauth?: OfficialMcpOAuthPorts) {
  let scope = "a".repeat(64);
  const vault = new Map<string, string>(), calls: Call[] = [];
  const service = new OfficialMcpService({
    oauth,
    getScope: async () => scope, read: async key => vault.get(key) ?? null,
    write: async (key, value) => { vault.set(key, value); },
    fetch: async (url, init) => {
      const call = { url, init, payload: JSON.parse(String(init.body)) }; calls.push(call);
      if (handler) return await handler(call);
      return reply(call);
    },
  });
  return { service, vault, calls, switchAccount: () => { scope = "b".repeat(64); } };
}
function json(id: number, result: unknown, headers = {}) {
  return new Response(JSON.stringify({ jsonrpc: "2.0", id, result }),
    { headers: { "Content-Type": "application/json", ...headers } });
}
function reply(call: Call) {
  if (call.payload.method === "initialize") return json(1, { protocolVersion: "2025-03-26", capabilities: {} }, { "Mcp-Session-Id": "session-1" });
  if (call.payload.method === "notifications/initialized") return new Response(null, { status: 202 });
  if (call.payload.method === "tools/list") return json(2, { tools: [TOOL] });
  if (call.payload.method === "tools/call") return json(2, { content: [{ type: "text", text: "issue content" }], structuredContent: { number: 1 } });
  throw new Error("Unexpected method");
}

test("catalog pins all nine upstream HTTPS endpoints and labels preview and credential setup", () => {
  assert.deepEqual(OFFICIAL_MCP_CATALOG.map(e => e.url), [
    "https://api.githubcopilot.com/mcp/", "https://gmailmcp.googleapis.com/mcp/v1",
    "https://drivemcp.googleapis.com/mcp/v1", "https://docsmcp.googleapis.com/mcp/v1",
    "https://sheetsmcp.googleapis.com/mcp/v1", "https://slidesmcp.googleapis.com/mcp/v1",
    "https://calendarmcp.googleapis.com/mcp/v1", "https://chatmcp.googleapis.com/mcp/v1",
    "https://people.googleapis.com/mcp/v1",
  ]);
  assert.equal(new Set(OFFICIAL_MCP_CATALOG.map(e => e.id)).size, 9);
  for (const listing of officialMcpCatalogPlugins()) {
    assert.equal(listing.marketplace?.displayName, "Fabushi 官方插件市场");
    assert.equal(listing.variableFields[0]!.isSecret, true);
    assert.equal(listing.variableFields[0]!.defaultValue, undefined);
    assert.match(listing.homepage!, /^https:\/\/(github.com|developers.google.com)\//);
    if (listing.publisher?.name === "google") assert.match(listing.description, /Developer Preview/);
  }
});

test("install without credential remains needsAuth and does not contact provider", async () => {
  const h = harness(); await h.service.install(ID);
  assert.equal((await h.service.servers())[0]!.status, "needsAuth");
  assert.equal((await h.service.effective())[0]!.pluginId, ID);
  assert.equal((await h.service.authenticate(ID, "default")).status, "not-supported");
  assert.deepEqual(await h.service.routedTools(), []);
  assert.equal(h.calls.length, 0);
});

test("OAuth completes into the scoped native vault and refreshes tools without returning credentials to UI", async () => {
  const oauth = oauthFixture(); let acknowledged = false, changed = false;
  oauth.acknowledge = async () => { acknowledged = true; };
  oauth.changed = () => { changed = true; };
  const h = harness(undefined, oauth); await h.service.install(ID);
  const started = await h.service.authenticate(ID, "default");
  assert.equal(started.status, "started");
  assert.doesNotMatch(JSON.stringify(started), /provider-oauth/);
  await new Promise(resolve => setTimeout(resolve, 2_200));
  assert.ok(acknowledged && changed);
  assert.match([...h.vault.values()].join(""), /provider-oauth-refresh/);
  assert.equal((await h.service.servers())[0]!.status, "connected");
  assert.doesNotMatch(JSON.stringify(await h.service.servers()), /provider-oauth/);
  await h.service.disconnect(ID);
  assert.doesNotMatch([...h.vault.values()].join(""), /provider-oauth/);
  h.service.dispose();
});

test("account change during pending OAuth cancels delivery and cannot install into another account", async () => {
  const oauth=oauthFixture(); let cancelled=false;
  oauth.cancel=async()=>{cancelled=true;};
  const h=harness(undefined,oauth); await h.service.install(ID);
  await h.service.authenticate(ID,"default"); h.switchAccount();
  await new Promise(resolve=>setTimeout(resolve,2_200));
  assert.ok(cancelled); assert.doesNotMatch([...h.vault.values()].join(""), /provider-oauth/);
  assert.deepEqual(await h.service.effective(),[]); h.service.dispose();
});

test("expired OAuth refresh is single-flight, stays before provider calls, and fences account switches", async () => {
  const oauth=oauthFixture(); let refreshes=0;
  oauth.refresh=async c=>{refreshes++; await new Promise(resolve=>setTimeout(resolve,20)); return {...c,token:"refreshed-secret",expiresAt:Date.now()+3_600_000};};
  const h=harness(undefined,oauth); await h.service.install(ID);
  const key="a".repeat(64); const state=JSON.parse(h.vault.get(key)!);
  Object.assign(state[ID],{token:"expired-secret",refreshToken:"refresh-secret",expiresAt:Date.now()-1000,connectionId:"grant-a"});
  h.vault.set(key,JSON.stringify(state));
  await Promise.all([h.service.tools(ID),h.service.tools(ID)]);
  assert.equal(refreshes,1);
  assert.ok(h.calls.every(call=>new Headers(call.init.headers).get("Authorization")==="Bearer refreshed-secret"));
  state[ID].expiresAt=Date.now()-1000; h.vault.set(key,JSON.stringify(state));
  oauth.refresh=async c=>{h.switchAccount();return {...c,token:"must-not-persist"};};
  await assert.rejects(h.service.tools(ID));
  assert.doesNotMatch([...h.vault.values()].join(""), /must-not-persist/); h.service.dispose();
});

test("disconnect clears local credentials even when upstream revocation fails", async () => {
  const oauth=oauthFixture(); oauth.revoke=async()=>{throw new Error("offline");};
  const h=harness(undefined,oauth); await h.service.install(ID,{ACCESS_TOKEN:"access-secret"});
  const key="a".repeat(64); const state=JSON.parse(h.vault.get(key)!); state[ID].connectionId="grant-a";
  h.vault.set(key,JSON.stringify(state));
  await assert.rejects(h.service.remove(ID),/本机授权/);
  assert.deepEqual(await h.service.effective(),[]);
  assert.doesNotMatch([...h.vault.values()].join(""),/access-secret/); h.service.dispose();
});

test("native broker confines Fabushi authorization to its origin and rejects foreign redirect URLs", async () => {
  const calls: {url:string;init:RequestInit}[]=[];
  const oauth=createOfficialMcpOAuthPorts({getAccessToken:async origin=>{assert.equal(origin,"https://api.ombhrum.com");return "fabushi-session";},changed:()=>{},
    fetch:async(input,init)=>{calls.push({url:String(input),init:init!});return new Response(JSON.stringify({attemptId:"attempt-a",authorizationUrl:"https://evil.example/steal"}),{headers:{"Content-Type":"application/json"}});}});
  await assert.rejects(oauth.start(ID),/Untrusted/);
  assert.equal(calls[0]!.url,"https://api.ombhrum.com/api/mcp/oauth/start");
  assert.equal(calls[0]!.init.redirect,"error");
  assert.equal(new Headers(calls[0]!.init.headers).get("Authorization"),"Bearer fabushi-session");
});

test("real initialize/list/call uses only provider credential, negotiated protocol and session", async () => {
  const h = harness(); await h.service.install(ID, { ACCESS_TOKEN: "provider-token" });
  assert.equal((await h.service.servers())[0]!.status, "connected");
  const tools = await h.service.routedTools(); assert.equal(tools[0]!.providerIdentifier, ID);
  const result = await h.service.execute(ID, TOOL.name, {});
  assert.equal(result.result.case, "success");
  assert.match(JSON.stringify(result.toJson()), /issue content/);
  assert.match(JSON.stringify(result.toJson()), /number/);
  for (const call of h.calls) {
    assert.equal(call.url, OFFICIAL_MCP_CATALOG[0]!.url);
    assert.equal(call.init.redirect, "error");
    assert.equal(new Headers(call.init.headers).get("Authorization"), "Bearer provider-token");
    if (call.payload.method !== "initialize") {
      assert.equal(new Headers(call.init.headers).get("Mcp-Session-Id"), "session-1");
      assert.equal(new Headers(call.init.headers).get("MCP-Protocol-Version"), "2025-03-26");
    }
  }
  assert.equal(h.calls.filter(c => c.payload.method === "tools/call").length, 1);
  assert.doesNotMatch(JSON.stringify(await h.service.servers()), /provider-token/);
  assert.doesNotMatch(JSON.stringify(await h.service.effective()), /provider-token/);
});

test("disabled tools leave inventory and cannot execute; update preserves credential; lifecycle persists", async () => {
  const h = harness(); await h.service.install(ID, { ACCESS_TOKEN: "provider-token" });
  await h.service.toggle(ID, TOOL.name);
  assert.equal((await h.service.toolSummaries(ID))[0]!.isDisabled, true);
  assert.deepEqual(await h.service.routedTools(), []);
  assert.equal((await h.service.execute(ID, TOOL.name, {})).result.case, "error");
  assert.equal(h.calls.filter(c => c.payload.method === "tools/call").length, 0);
  await h.service.install(ID, { ACCESS_TOKEN: "" });
  assert.equal((await h.service.servers())[0]!.status, "connected");
  await h.service.disconnect(ID);
  assert.equal((await h.service.servers())[0]!.status, "needsAuth");
  assert.doesNotMatch([...h.vault.values()].join(""), /provider-token/);
  await h.service.remove(ID); assert.deepEqual(await h.service.effective(), []);
});

test("SSE notifications and paginated tool inventory are correlated and decoded", async () => {
  const h = harness(call => {
    if (call.payload.method !== "tools/list") return reply(call);
    const result = call.payload.params.cursor ? { tools: [{ ...TOOL, name: "read_repo" }] }
      : { tools: [TOOL], nextCursor: "page-2" };
    return new Response('event: message\r\ndata: {"jsonrpc":"2.0","method":"notifications/progress"}\r\n\r\n' +
      `data: ${JSON.stringify({ jsonrpc: "2.0", id: 2, result })}\r\n\r\n`, { headers: { "Content-Type": "text/event-stream" } });
  });
  await h.service.install(ID, { ACCESS_TOKEN: "provider-token" });
  assert.deepEqual((await h.service.tools(ID)).map(t => t.name), ["read_issue", "read_repo"]);
});

test("repeated pagination cursor fails explicitly", async () => {
  const h = harness(call => call.payload.method === "tools/list"
    ? json(2, { tools: [], nextCursor: "repeat" }) : reply(call));
  await h.service.install(ID, { ACCESS_TOKEN: "provider-token" });
  await assert.rejects(h.service.tools(ID), /pagination/);
});

test("authorization failures are honest and uncertain writes are never retried", async () => {
  const h = harness(call => call.payload.method === "tools/call" ? new Response("provider-token", { status: 500 }) : reply(call));
  await h.service.install(ID, { ACCESS_TOKEN: "provider-token" });
  const result = await h.service.execute(ID, TOOL.name, {});
  assert.equal(result.result.case, "error");
  assert.doesNotMatch(JSON.stringify(result.toJson()), /provider-token/);
  assert.equal(h.calls.filter(c => c.payload.method === "tools/call").length, 1);
  const denied = harness(() => new Response("provider-token", { status: 401 }));
  await denied.service.install(ID, { ACCESS_TOKEN: "provider-token" });
  assert.equal((await denied.service.servers())[0]!.status, "needsAuth");
});

test("account change fences in-flight provider output and isolates installed state", async () => {
  let finish!: (response: Response) => void;
  const pending = new Promise<Response>(resolve => { finish = resolve; });
  let started!: () => void;
  const callStarted = new Promise<void>(resolve => { started = resolve; });
  const h = harness(call => { if (call.payload.method === "tools/call") { started(); return pending; } return reply(call); });
  await h.service.install(ID, { ACCESS_TOKEN: "provider-token" });
  const execution = h.service.execute(ID, TOOL.name, {});
  await callStarted; h.switchAccount(); finish(json(2, { content: [{ type: "text", text: "old-account-private-content" }] }));
  const result = await execution;
  assert.equal(result.result.case, "error");
  assert.doesNotMatch(JSON.stringify(result.toJson()), /old-account-private-content/);
  assert.deepEqual(await h.service.effective(), []);
});

test("uninstall fences in-flight output and response strings redact provider token", async () => {
  const h = harness(call => call.payload.method === "tools/call"
    ? json(2, { content: [{ type: "text", text: "echo provider-token" }] }) : reply(call));
  await h.service.install(ID, { ACCESS_TOKEN: "provider-token" });
  const result = await h.service.execute(ID, TOOL.name, {});
  assert.equal(result.result.case, "success");
  assert.doesNotMatch(JSON.stringify(result.toJson()), /provider-token/);
  let finish!: (response: Response) => void, started!: () => void;
  const pending = new Promise<Response>(resolve => { finish = resolve; });
  const ready = new Promise<void>(resolve => { started = resolve; });
  const removing = harness(call => { if (call.payload.method === "tools/call") { started(); return pending; } return reply(call); });
  await removing.service.install(ID, { ACCESS_TOKEN: "provider-token" });
  const execution = removing.service.execute(ID, TOOL.name, {}); await ready;
  await removing.service.remove(ID); finish(json(2, { content: [{ type: "text", text: "removed-secret" }] }));
  assert.equal((await execution).result.case, "error");
});

test("legacy lookup failure leaves catalog visible with explicit partial-load warning", async () => {
  const catalog = [{ id: ID, name: ID, displayName: "GitHub", description: "GitHub", category: "Development", connectors: [], skills: [] }];
  const bridge = { mcp: { catalog: async () => catalog,
    effectivePlugins: async () => { throw new Error("foreign backend refused token"); },
    list: async () => ({ servers: [] }), } } as unknown as DesktopBridge;
  const snapshot = await loadPluginsDesktopSnapshot(bridge);
  assert.equal(snapshot.catalog[0]!.id, ID);
  assert.ok(snapshot.warnings?.some(w => w.includes("尚未确认")));
});
