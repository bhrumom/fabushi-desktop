import { invokeNativeDesktop } from '../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import { ElectronMahayanaHostTransport, MAHAYANA_ACCOUNT_SESSION_RESET_EVENT } from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import { readAccountMiniApps } from './account-sync-client';

const protocol = 'fabushi.miniapp.webmcp.v1';
const executionProtocol = 'fabushi.miniapp.execution.v1';
const installMarker = Symbol.for('fabushi.desktop.miniapp-webmcp-host.v1');
const pendingToolCalls = new Set<string>();
const validBridgeNonces = new Map<string, string>();
const bridgeSources = new Map<string, WindowProxy>();
const executionCache = new Map<string, MiniAppExecutionState>();
const lifetimePurchaseKeys = new Map<string, string>();
const credentialToolName = 'fabushi_credential_request';
const prayerWheelCapability = 'local.prayer-wheel.start';

type ToolDescriptor = {
  name: string;
  description?: string;
  inputSchema?: Record<string, unknown>;
  annotations?: { readOnlyHint?: boolean; destructiveHint?: boolean; openWorldHint?: boolean };
  approval?: string;
};

type PluginUiDocument = { pluginId: string; html: string };

type CredentialFetchResult = {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  bodyBase64: string;
  url: string;
  credential: { secretRef: string; origin: string; injectionType: string };
};

export type MiniAppExecutionState = {
  protocol: typeof executionProtocol;
  pluginId: string;
  revision: number;
  phase: 'idle' | 'running' | 'blocked' | 'completed' | 'failed' | 'commerce';
  source: 'bot' | 'web-ui' | 'host';
  tool: string | null;
  surface: string;
  progress: string | null;
  entitlementAllowed: boolean | null;
  result?: unknown;
  error?: string;
  updatedAtMs: number;
};

function recordValue(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : null;
}

function safePluginId(value: unknown): string {
  const id = String(value ?? '').trim().toLowerCase();
  return /^[a-z0-9][a-z0-9-]{1,63}$/.test(id) ? id : '';
}

function executionKey(pluginId: string): string {
  return `fabushi.desktop.miniapp-execution.v1:${pluginId}`;
}

function lifetimePurchaseKey(pluginId: string): string {
  const existing = lifetimePurchaseKeys.get(pluginId);
  if (existing) return existing;
  const key = `miniapp-${pluginId}-${crypto.randomUUID()}`;
  lifetimePurchaseKeys.set(pluginId, key);
  return key;
}

function defaultExecution(pluginId: string): MiniAppExecutionState {
  return {
    protocol: executionProtocol,
    pluginId,
    revision: 0,
    phase: 'idle',
    source: 'host',
    tool: null,
    surface: 'home',
    progress: null,
    entitlementAllowed: null,
    updatedAtMs: Date.now(),
  };
}

function normalizeExecution(pluginId: string, value: unknown): MiniAppExecutionState | null {
  const row = recordValue(value);
  if (!row || row.protocol !== executionProtocol || row.pluginId !== pluginId) return null;
  const revision = Number(row.revision);
  if (!Number.isSafeInteger(revision) || revision < 0) return null;
  const phase = String(row.phase ?? 'idle') as MiniAppExecutionState['phase'];
  if (!['idle', 'running', 'blocked', 'completed', 'failed', 'commerce'].includes(phase)) return null;
  return {
    protocol: executionProtocol,
    pluginId,
    revision,
    phase,
    source: ['bot', 'web-ui', 'host'].includes(String(row.source)) ? row.source as MiniAppExecutionState['source'] : 'host',
    tool: typeof row.tool === 'string' ? row.tool : null,
    surface: typeof row.surface === 'string' && row.surface ? row.surface : 'home',
    progress: typeof row.progress === 'string' ? row.progress : null,
    entitlementAllowed: typeof row.entitlementAllowed === 'boolean' ? row.entitlementAllowed : null,
    ...(Object.prototype.hasOwnProperty.call(row, 'result') ? { result: row.result } : {}),
    ...(typeof row.error === 'string' ? { error: row.error } : {}),
    updatedAtMs: Number.isFinite(Number(row.updatedAtMs)) ? Number(row.updatedAtMs) : Date.now(),
  };
}

async function readExecution(pluginId: string): Promise<MiniAppExecutionState> {
  const cached = executionCache.get(pluginId);
  if (cached) return cached;
  try {
    const local = normalizeExecution(pluginId, JSON.parse(window.localStorage.getItem(executionKey(pluginId)) || 'null'));
    if (local) {
      executionCache.set(pluginId, local);
      return local;
    }
  } catch {
    // Native persistence remains the durability fallback.
  }
  try {
    const durable = normalizeExecution(pluginId, await invokeNativeDesktop('readClientPersistence', { key: executionKey(pluginId) }));
    if (durable) {
      executionCache.set(pluginId, durable);
      try { window.localStorage.setItem(executionKey(pluginId), JSON.stringify(durable)); } catch {}
      return durable;
    }
  } catch {
    // A first-run Mini App legitimately has no durable execution state yet.
  }
  const initial = defaultExecution(pluginId);
  executionCache.set(pluginId, initial);
  return initial;
}

function pushExecution(state: MiniAppExecutionState): void {
  for (const [nonce, bridgePluginId] of validBridgeNonces) {
    if (bridgePluginId !== state.pluginId) continue;
    const source = bridgeSources.get(nonce);
    if (!source) continue;
    source.postMessage({ protocol, pluginId: state.pluginId, nonce, event: 'execution', data: state }, '*');
  }
}

async function publishExecution(
  pluginId: string,
  patch: Omit<Partial<MiniAppExecutionState>, 'protocol' | 'pluginId' | 'revision' | 'updatedAtMs'>,
): Promise<MiniAppExecutionState> {
  const current = await readExecution(pluginId);
  const next: MiniAppExecutionState = {
    ...current,
    ...patch,
    protocol: executionProtocol,
    pluginId,
    revision: current.revision + 1,
    updatedAtMs: Date.now(),
  };
  executionCache.set(pluginId, next);
  try { window.localStorage.setItem(executionKey(pluginId), JSON.stringify(next)); } catch {}
  void invokeNativeDesktop('writeClientPersistence', { key: executionKey(pluginId), value: next }).catch(() => {});
  pushExecution(next);
  return next;
}

function normalizeRuntimeTool(value: unknown): ToolDescriptor | null {
  if (typeof value === 'string' && value.trim()) {
    return { name: value.trim(), description: value.trim(), inputSchema: { type: 'object', properties: {} } };
  }
  const row = recordValue(value);
  if (!row) return null;
  const name = String(row.name ?? row.id ?? '').trim();
  if (!name) return null;
  return {
    name,
    description: typeof row.description === 'string' && row.description.trim() ? row.description : name,
    inputSchema: recordValue(row.inputSchema) ?? { type: 'object', properties: {} },
    annotations: (recordValue(row.annotations) as ToolDescriptor['annotations'] | null) ?? undefined,
  };
}

function credentialTool(pluginId: string): ToolDescriptor {
  return {
    name: credentialToolName,
    description: `Use a non-revealable Fabushi SecretRef owned by ${pluginId} for an HTTPS API request.`,
    inputSchema: {
      type: 'object', additionalProperties: false, required: ['secretRef', 'url'],
      properties: {
        secretRef: { type: 'string' }, url: { type: 'string' },
        method: { type: 'string', enum: ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'] },
        headers: { type: 'object', additionalProperties: { type: 'string' } }, body: { type: 'string' },
      },
    },
    approval: 'credential',
    annotations: { readOnlyHint: false, destructiveHint: false, openWorldHint: true },
  };
}

async function allowedTools(pluginId: string): Promise<ToolDescriptor[]> {
  const account = await readAccountMiniApps();
  const app = account.apps.find((candidate) => candidate.id === pluginId);
  if (!app) throw new Error(`MiniApp ${pluginId} is not installed for this account`);
  const runtimeInventory = await invokeNativeDesktop<unknown>('listMcpServerTools', { server: pluginId }).catch(() => []);
  const runtimeRows = Array.isArray(runtimeInventory)
    ? runtimeInventory.map(normalizeRuntimeTool).filter((tool): tool is ToolDescriptor => Boolean(tool)) : [];
  const commandByTool = new Map<string, Record<string, unknown>>();
  for (const raw of Array.isArray(app.commands) ? app.commands : []) {
    const command = recordValue(raw);
    if (!command) continue;
    const tool = String(command.tool ?? command.name ?? '').trim();
    if (tool) commandByTool.set(tool, command);
  }
  const scopedRuntimeRows = runtimeRows.filter((tool) => commandByTool.has(tool.name));
  const pluginTools = scopedRuntimeRows.length > 0
    ? scopedRuntimeRows.map((tool) => {
        const command = commandByTool.get(tool.name);
        const approval = String(command?.approval ?? 'none');
        return {
          ...tool,
          description: tool.description || (typeof command?.description === 'string' ? command.description : tool.name),
          approval,
          annotations: tool.annotations ?? { readOnlyHint: approval === 'none', destructiveHint: approval === 'destructive', openWorldHint: false },
        };
      })
    : [...commandByTool.entries()].map(([name, command]) => {
        const approval = String(command.approval ?? 'none');
        return {
          name,
          description: typeof command.description === 'string' && command.description.trim() ? command.description : name,
          inputSchema: { type: 'object', properties: {} }, approval,
          annotations: { readOnlyHint: approval === 'none', destructiveHint: approval === 'destructive', openWorldHint: false },
        };
      });
  return [...pluginTools, credentialTool(pluginId)];
}

function credentialRefAllowed(pluginId: string, secretRef: string): boolean {
  return secretRef.startsWith(`connector/${pluginId}/`) || secretRef.startsWith(`plugin/${pluginId}/`);
}

function decodeResponseBody(result: CredentialFetchResult): string | null {
  if (!result.bodyBase64) return '';
  try {
    const binary = atob(result.bodyBase64);
    const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
    return new TextDecoder().decode(bytes);
  } catch { return null; }
}

async function callCredentialTool(pluginId: string, args: Record<string, unknown>): Promise<unknown> {
  const secretRef = String(args.secretRef ?? '').trim();
  if (!credentialRefAllowed(pluginId, secretRef)) throw new Error(`Credential reference is outside ${pluginId}`);
  const url = String(args.url ?? '').trim();
  if (!url) throw new Error('Credential request URL is required');
  const method = String(args.method ?? 'GET').trim().toUpperCase();
  if (!['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'].includes(method)) throw new Error(`Unsupported method ${method}`);
  const headers = recordValue(args.headers) ?? undefined;
  const body = typeof args.body === 'string' ? args.body : undefined;
  const result = await invokeNativeDesktop<CredentialFetchResult>('egressFetch', {
    secretRef, url, method, ...(headers ? { headers } : {}), ...(body !== undefined ? { body } : {}),
  });
  return { status: result.status, statusText: result.statusText, headers: result.headers, url: result.url, credential: result.credential, body: decodeResponseBody(result), bodyBase64: result.bodyBase64 };
}

async function callLocalRuntimeTool(pluginId: string, tool: ToolDescriptor, args: Record<string, unknown>): Promise<unknown> {
  if (tool.name === credentialToolName) return callCredentialTool(pluginId, args);
  const key = `${pluginId}:${tool.name}`;
  if (pendingToolCalls.has(key)) throw new Error(`Tool ${tool.name} already has a pending call`);
  pendingToolCalls.add(key);
  try {
    const approval = tool.approval ?? (tool.annotations?.readOnlyHint === true ? 'none' : 'required');
    if (approval !== 'none') {
      const warning = approval === 'destructive' ? '该操作可能产生破坏性修改。' : '该操作会修改小程序或后台状态。';
      if (!window.confirm(`允许 ${pluginId} 调用 ${tool.name}？\n\n${warning}`)) throw new Error('用户取消了 WebMCP Tool 调用');
    }
    return invokeNativeDesktop('callMiniAppRuntimeTool', { pluginId, name: tool.name, arguments: args });
  } finally { pendingToolCalls.delete(key); }
}

async function readEntitlement(pluginId: string): Promise<Record<string, unknown>> {
  return invokeNativeDesktop<Record<string, unknown>>('getMiniAppEntitlement', { pluginId, capability: prayerWheelCapability });
}

async function enforceHostRequestEntitlement(pluginId: string, result: unknown): Promise<void> {
  if (pluginId !== 'global-dharma') return;
  const structured = recordValue(recordValue(result)?.structuredContent);
  const hostRequest = recordValue(structured?.hostRequest);
  const capability = String(hostRequest?.capability ?? '').trim();
  if (capability !== prayerWheelCapability) return;
  const entitlement = await readEntitlement(pluginId);
  if (recordValue(entitlement.access)?.allowed !== true) {
    throw new Error(`Mini App hostRequest ${prayerWheelCapability} requires an active canonical entitlement.`);
  }
}

function routeTool(routed: Record<string, unknown>): { name: string; args: Record<string, unknown>; surface: string } | null {
  const command = recordValue(routed.command) ?? recordValue(routed.suggestedCommand);
  if (!command) return null;
  const name = String(command.tool ?? command.name ?? '').trim();
  if (!name) return null;
  return { name, args: recordValue(routed.arguments) ?? {}, surface: String(command.surfaceId ?? routed.surface ?? 'home') };
}

function isPrayerWheelStart(pluginId: string, tool: string, input: string): boolean {
  return pluginId === 'global-dharma' && tool === 'start' && /(转经轮|prayer\s*wheel)/i.test(input);
}

export async function executeDesktopMiniAppBotInput(pluginIdValue: string, routed: Record<string, unknown>, input: string): Promise<unknown> {
  const pluginId = safePluginId(pluginIdValue);
  if (!pluginId) throw new Error('Invalid Mini App id');
  const planned = routeTool(routed);
  if (!planned) {
    await publishExecution(pluginId, { phase: 'blocked', source: 'bot', tool: null, surface: 'home', progress: '需要 Mahayana 进一步规划', entitlementAllowed: null, result: routed });
    return routed;
  }
  const tools = await allowedTools(pluginId);
  const tool = tools.find((candidate) => candidate.name === planned.name);
  if (!tool) throw new Error(`WebMCP tool ${planned.name} is not allowed for ${pluginId}`);
  const prayerWheel = isPrayerWheelStart(pluginId, planned.name, input);
  let entitlementAllowed: boolean | null = null;
  if (prayerWheel) {
    const entitlement = await readEntitlement(pluginId);
    entitlementAllowed = recordValue(entitlement.access)?.allowed === true;
    if (!entitlementAllowed) {
      const blocked = {
        content: [{ type: 'text', text: '本地转经轮需要 CNY 1080 永久权限；请在打开应用后购买或恢复。' }],
        structuredContent: { purchaseRequired: true, capability: prayerWheelCapability, entitlement },
      };
      await publishExecution(pluginId, { phase: 'blocked', source: 'bot', tool: planned.name, surface: 'local-prayer-wheel', progress: '等待购买或恢复本地转经轮权限', entitlementAllowed: false, result: blocked });
      return blocked;
    }
  }
  const surface = prayerWheel ? 'local-prayer-wheel' : planned.surface;
  await publishExecution(pluginId, { phase: 'running', source: 'bot', tool: planned.name, surface, progress: `WebMCP 正在执行 ${planned.name}`, entitlementAllowed, result: undefined, error: undefined });
  try {
    const result = await callLocalRuntimeTool(pluginId, tool, planned.args);
    await enforceHostRequestEntitlement(pluginId, result);
    await publishExecution(pluginId, { phase: 'completed', source: 'bot', tool: planned.name, surface, progress: `${planned.name} 已完成`, entitlementAllowed, result, error: undefined });
    return result;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    await publishExecution(pluginId, { phase: 'failed', source: 'bot', tool: planned.name, surface, progress: `${planned.name} 执行失败`, entitlementAllowed, error: message });
    throw error;
  }
}

function webMcpBootstrap(pluginId: string, nonce: string): string {
  return `<script>(function(){
    const protocol=${JSON.stringify(protocol)},pluginId=${JSON.stringify(pluginId)},nonce=${JSON.stringify(nonce)};
    const pending=new Map(),localTools=new Map(),controllers=[];let sequence=0,currentExecution=null;
    function request(action,payload){return new Promise((resolve,reject)=>{const requestId='webmcp-'+Date.now()+'-'+(++sequence);pending.set(requestId,{resolve,reject});parent.postMessage({protocol,pluginId,nonce,requestId,action,...(payload||{})},'*');});}
    function publicTool(tool){const copy={...tool};delete copy.execute;return copy;}
    function ensureStateBar(){let bar=document.getElementById('fabushi-host-state');if(bar)return bar;bar=document.createElement('div');bar.id='fabushi-host-state';bar.setAttribute('data-testid','fabushi-miniapp-host-state');bar.style.cssText='position:fixed;left:8px;right:8px;bottom:8px;z-index:2147483647;padding:8px 10px;border-radius:10px;background:rgba(10,18,30,.92);color:#eef6ff;font:12px/1.35 system-ui;box-shadow:0 4px 24px rgba(0,0,0,.3);pointer-events:auto';const summary=document.createElement('div');summary.setAttribute('data-testid','fabushi-miniapp-host-summary');bar.appendChild(summary);if(pluginId==='global-dharma'){const actions=document.createElement('div');actions.style.cssText='display:flex;gap:8px;margin-top:8px;flex-wrap:wrap';const purchase=document.createElement('button');purchase.type='button';purchase.setAttribute('data-testid','fabushi-miniapp-purchase-lifetime');purchase.textContent='购买 ¥1080 永久权限';purchase.style.cssText='border:0;border-radius:8px;padding:6px 10px;cursor:pointer';purchase.onclick=async()=>{purchase.disabled=true;try{await request('purchaseLifetime',{capability:${JSON.stringify(prayerWheelCapability)}});render(await request('state'));}finally{purchase.disabled=false;}};const restore=document.createElement('button');restore.type='button';restore.setAttribute('data-testid','fabushi-miniapp-restore-purchases');restore.textContent='恢复购买';restore.style.cssText='border:0;border-radius:8px;padding:6px 10px;cursor:pointer';restore.onclick=async()=>{restore.disabled=true;try{await request('restorePurchases',{capability:${JSON.stringify(prayerWheelCapability)}});render(await request('state'));}finally{restore.disabled=false;}};actions.appendChild(purchase);actions.appendChild(restore);bar.appendChild(actions);}document.body.appendChild(bar);return bar;}
    function render(state){if(!state||!document.body)return;currentExecution=state;const bar=ensureStateBar();bar.dataset.revision=String(state.revision??0);bar.dataset.phase=String(state.phase||'idle');bar.dataset.tool=String(state.tool||'');bar.dataset.surface=String(state.surface||'home');bar.dataset.entitlementAllowed=String(state.entitlementAllowed);const summary=bar.querySelector('[data-testid=\"fabushi-miniapp-host-summary\"]');if(summary)summary.textContent='Fabushi · r'+bar.dataset.revision+' · '+bar.dataset.phase+' · '+bar.dataset.surface+(bar.dataset.tool?' · '+bar.dataset.tool:'')+(state.progress?' · '+state.progress:'');const purchase=bar.querySelector('[data-testid=\"fabushi-miniapp-purchase-lifetime\"]');if(purchase)purchase.hidden=state.entitlementAllowed===true;window.dispatchEvent(new CustomEvent('fabushi:execution-state',{detail:state}));}
    function register(item){const tool={name:item.name,description:item.description||item.name,inputSchema:item.inputSchema||{type:'object',properties:{}},annotations:{readOnlyHint:item.annotations?.readOnlyHint===true},execute:(input)=>request('call',{tool:item.name,input:input||{}})};localTools.set(tool.name,tool);if(document.modelContext&&typeof document.modelContext.registerTool==='function'){const controller=new AbortController();controllers.push(controller);Promise.resolve(document.modelContext.registerTool(tool,{signal:controller.signal})).catch(()=>{});}}
    Object.defineProperty(window,'__fabushiWebMcp',{configurable:true,value:{version:1,list:()=>Array.from(localTools.values()).map(publicTool),call:async(name,input={})=>{const tool=localTools.get(name);if(!tool)throw new Error('Unknown WebMCP tool: '+name);return tool.execute(input);}}});
    Object.defineProperty(window,'__fabushiMiniAppHost',{configurable:true,value:{version:1,session:()=>request('session'),execution:()=>request('state'),snapshot:()=>currentExecution,entitlement:()=>request('entitlement',{capability:${JSON.stringify(prayerWheelCapability)}}),purchaseLifetime:()=>request('purchaseLifetime',{capability:${JSON.stringify(prayerWheelCapability)}}),restorePurchases:()=>request('restorePurchases',{capability:${JSON.stringify(prayerWheelCapability)}})}});
    addEventListener('message',(event)=>{const data=event.data||{};if(data.protocol!==protocol||data.pluginId!==pluginId||data.nonce!==nonce)return;if(data.event==='execution'){render(data.data);return;}if(!data.requestId||!pending.has(data.requestId))return;const task=pending.get(data.requestId);pending.delete(data.requestId);if(data.ok)task.resolve(data.data);else task.reject(new Error(data.error||'WebMCP host request failed'));});
    async function boot(){const [tools,state,session]=await Promise.all([request('list'),request('state'),request('session')]);for(const item of (tools||[]))register(item);render(state);window.dispatchEvent(new CustomEvent('fabushi:webmcp-ready',{detail:{pluginId,tools:(tools||[]).map(t=>t.name),session}}));}
    function dispose(){for(const controller of controllers)controller.abort();controllers.length=0;parent.postMessage({protocol,pluginId,nonce,requestId:'dispose-'+Date.now(),action:'dispose'},'*');}
    addEventListener('pagehide',dispose,{once:true});if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',()=>void boot(),{once:true});else void boot();
  })();</script>`;
}

function injectWebMcp(html: string, pluginId: string, nonce: string): string {
  if (html.includes('fabushi.miniapp.webmcp.v1')) return html;
  const bootstrap = webMcpBootstrap(pluginId, nonce);
  return html.includes('</head>') ? html.replace('</head>', `${bootstrap}</head>`) : `${bootstrap}${html}`;
}

export function prepareDesktopMiniAppWebMcpDocument(pluginId: string, html: string): string {
  const normalizedPluginId = safePluginId(pluginId);
  if (!normalizedPluginId) throw new Error(`Invalid MiniApp plugin id ${pluginId}`);
  if (html.includes(protocol)) return html;
  const nonce = crypto.randomUUID();
  validBridgeNonces.set(nonce, normalizedPluginId);
  return injectWebMcp(html, normalizedPluginId, nonce);
}

export function installDesktopMiniAppWebMcpHost(): void {
  const globalObject = window as Window & { [installMarker]?: boolean };
  if (globalObject[installMarker]) return;
  Object.defineProperty(globalObject, installMarker, { value: true });

  window.addEventListener(MAHAYANA_ACCOUNT_SESSION_RESET_EVENT, () => {
    executionCache.clear();
    lifetimePurchaseKeys.clear();
    validBridgeNonces.clear();
    bridgeSources.clear();
    const prefix = 'fabushi.desktop.miniapp-execution.v1:';
    try {
      for (const key of Object.keys(window.localStorage)) {
        if (key.startsWith(prefix)) window.localStorage.removeItem(key);
      }
    } catch {}
    void invokeNativeDesktop<string[]>('listClientPersistenceKeys', { prefix })
      .then((keys) => Promise.all(keys.map((key) => invokeNativeDesktop('removeClientPersistence', { key }))))
      .catch(() => undefined);
  });

  const prototype = ElectronMahayanaHostTransport.prototype as ElectronMahayanaHostTransport & {
    pluginUiDocument(pluginId: string): Promise<PluginUiDocument>;
  };
  const originalPluginUiDocument = prototype.pluginUiDocument;
  prototype.pluginUiDocument = async function pluginUiDocumentWithWebMcp(pluginId: string) {
    const document = await originalPluginUiDocument.call(this, pluginId);
    return { ...document, html: prepareDesktopMiniAppWebMcpDocument(pluginId, document.html) };
  };

  window.addEventListener('message', (event) => {
    const data = recordValue(event.data);
    if (!data || data.protocol !== protocol || !data.requestId) return;
    const pluginId = safePluginId(data.pluginId);
    const nonce = String(data.nonce ?? '');
    const requestId = String(data.requestId);
    const source = event.source;
    if (!pluginId || !nonce || validBridgeNonces.get(nonce) !== pluginId) return;
    if (!source || typeof (source as WindowProxy).postMessage !== 'function') return;
    bridgeSources.set(nonce, source as WindowProxy);
    const respond = (ok: boolean, payload: unknown) => {
      (source as WindowProxy).postMessage({ protocol, pluginId, nonce, requestId, ok, ...(ok ? { data: payload } : { error: payload instanceof Error ? payload.message : String(payload) }) }, '*');
    };
    void (async () => {
      try {
        if (data.action === 'dispose') { validBridgeNonces.delete(nonce); bridgeSources.delete(nonce); return; }
        if (data.action === 'state') { respond(true, await readExecution(pluginId)); return; }
        if (data.action === 'session') { respond(true, await invokeNativeDesktop('getMiniAppSessionProjection', { pluginId })); return; }
        if (data.action === 'entitlement') { respond(true, await readEntitlement(pluginId)); return; }
        if (data.action === 'purchaseLifetime') {
          const idempotencyKey = lifetimePurchaseKey(pluginId);
          await publishExecution(pluginId, { phase: 'commerce', source: 'web-ui', tool: 'purchaseLifetime', surface: 'local-prayer-wheel', progress: '正在创建 CNY 1080 永久权限订单', entitlementAllowed: false });
          const result = await invokeNativeDesktop<Record<string, unknown>>('purchaseMiniAppLifetime', { pluginId, capability: prayerWheelCapability, idempotencyKey });
          const entitlement = recordValue(result.entitlement);
          const allowed = recordValue(entitlement?.access)?.allowed === true;
          await publishExecution(pluginId, { phase: allowed ? 'completed' : 'commerce', source: 'web-ui', tool: 'purchaseLifetime', surface: 'local-prayer-wheel', progress: allowed ? 'CNY 1080 永久权限已生效' : '等待支付完成后恢复权限', entitlementAllowed: allowed, result });
          if (allowed) lifetimePurchaseKeys.delete(pluginId);
          respond(true, result); return;
        }
        if (data.action === 'restorePurchases') {
          const result = await invokeNativeDesktop<Record<string, unknown>>('restoreMiniAppPurchases', { pluginId, capability: prayerWheelCapability });
          const entitlement = recordValue(result.entitlement);
          const allowed = recordValue(entitlement?.access)?.allowed === true;
          await publishExecution(pluginId, { phase: 'completed', source: 'web-ui', tool: 'restorePurchases', surface: 'local-prayer-wheel', progress: allowed ? '购买已恢复，本地转经轮可用' : '未恢复到有效本地转经轮权限', entitlementAllowed: allowed, result });
          respond(true, result); return;
        }
        const tools = await allowedTools(pluginId);
        if (data.action === 'list') {
          respond(true, tools.map(({ approval: _approval, annotations, ...tool }) => ({ ...tool, annotations: { readOnlyHint: annotations?.readOnlyHint === true } })));
          return;
        }
        if (data.action === 'call') {
          const requested = String(data.tool ?? '').trim();
          const tool = tools.find((candidate) => candidate.name === requested);
          if (!tool) throw new Error(`WebMCP tool ${requested} is not allowed for ${pluginId}`);
          const input = recordValue(data.input) ?? {};
          await publishExecution(pluginId, { phase: 'running', source: 'web-ui', tool: requested, surface: 'web-ui', progress: `WebMCP 正在执行 ${requested}` });
          const result = await callLocalRuntimeTool(pluginId, tool, input);
          await enforceHostRequestEntitlement(pluginId, result);
          await publishExecution(pluginId, { phase: 'completed', source: 'web-ui', tool: requested, surface: 'web-ui', progress: `${requested} 已完成`, result });
          respond(true, result); return;
        }
        throw new Error(`Unsupported WebMCP action ${String(data.action)}`);
      } catch (error) { respond(false, error); }
    })();
  });
}
