import { invokeNativeDesktop } from '../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import { ElectronMahayanaHostTransport } from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import { readAccountMiniApps } from './account-sync-client';

const protocol = 'fabushi.miniapp.webmcp.v1';
const installMarker = Symbol.for('fabushi.desktop.miniapp-webmcp-host.v1');
const pendingToolCalls = new Set<string>();
const validBridgeNonces = new Map<string, string>();
const credentialToolName = 'fabushi_credential_request';

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

function safePluginId(value: unknown): string {
  const id = String(value ?? '').trim().toLowerCase();
  return /^[a-z0-9][a-z0-9-]{1,63}$/.test(id) ? id : '';
}

function normalizeRuntimeTool(value: unknown): ToolDescriptor | null {
  if (typeof value === 'string' && value.trim()) {
    return { name: value.trim(), description: value.trim(), inputSchema: { type: 'object', properties: {} } };
  }
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
  const row = value as Record<string, unknown>;
  const name = String(row.name ?? row.id ?? '').trim();
  if (!name) return null;
  return {
    name,
    description: typeof row.description === 'string' && row.description.trim() ? row.description : name,
    inputSchema: row.inputSchema && typeof row.inputSchema === 'object' && !Array.isArray(row.inputSchema)
      ? row.inputSchema as Record<string, unknown>
      : { type: 'object', properties: {} },
    annotations: row.annotations && typeof row.annotations === 'object' && !Array.isArray(row.annotations)
      ? row.annotations as ToolDescriptor['annotations']
      : undefined,
  };
}

function credentialTool(pluginId: string): ToolDescriptor {
  return {
    name: credentialToolName,
    description: `Use a non-revealable Fabushi SecretRef owned by ${pluginId} for an HTTPS API request. The credential value is injected only by the trusted desktop host and is never returned to this MiniApp or the model.`,
    inputSchema: {
      type: 'object',
      additionalProperties: false,
      required: ['secretRef', 'url'],
      properties: {
        secretRef: {
          type: 'string',
          description: `A reference scoped to connector/${pluginId}/… or plugin/${pluginId}/…`,
        },
        url: { type: 'string', description: 'HTTPS request URL already allowed by the credential binding.' },
        method: { type: 'string', enum: ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'] },
        headers: { type: 'object', additionalProperties: { type: 'string' } },
        body: { type: 'string', description: 'Optional UTF-8 request body. Authentication headers are forbidden here.' },
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
    ? runtimeInventory.map(normalizeRuntimeTool).filter((tool): tool is ToolDescriptor => Boolean(tool))
    : [];

  const commands = Array.isArray(app.commands) ? app.commands : [];
  const commandByTool = new Map<string, Record<string, unknown>>();
  for (const raw of commands) {
    if (!raw || typeof raw !== 'object' || Array.isArray(raw)) continue;
    const command = raw as Record<string, unknown>;
    const tool = String(command.tool ?? command.name ?? '').trim();
    if (tool) commandByTool.set(tool, command);
  }

  // runtime.tools is a host-wide inventory. Never expose another MiniApp's tool
  // through this document: intersect it with this MiniApp's signed/approved
  // Tool Contract before projecting it into WebMCP.
  const scopedRuntimeRows = runtimeRows.filter((tool) => commandByTool.has(tool.name));
  const pluginTools = scopedRuntimeRows.length > 0
    ? scopedRuntimeRows.map((tool) => {
        const command = commandByTool.get(tool.name);
        const approval = String(command?.approval ?? 'none');
        return {
          ...tool,
          description: tool.description || (typeof command?.description === 'string' ? command.description : tool.name),
          approval,
          annotations: tool.annotations ?? {
            readOnlyHint: approval === 'none',
            destructiveHint: approval === 'destructive',
            openWorldHint: false,
          },
        };
      })
    : [...commandByTool.entries()].map(([name, command]) => {
        const approval = String(command.approval ?? 'none');
        return {
          name,
          description: typeof command.description === 'string' && command.description.trim() ? command.description : name,
          inputSchema: { type: 'object', properties: {} },
          approval,
          annotations: {
            readOnlyHint: approval === 'none',
            destructiveHint: approval === 'destructive',
            openWorldHint: false,
          },
        };
      });

  // The credential tool is host-owned rather than plugin-owned. It receives
  // only an opaque SecretRef and is namespace-scoped to the installed plugin.
  // A plugin can never ask for another plugin's reference.
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
  } catch {
    return null;
  }
}

async function callCredentialTool(pluginId: string, args: Record<string, unknown>): Promise<unknown> {
  const secretRef = String(args.secretRef ?? '').trim();
  if (!credentialRefAllowed(pluginId, secretRef)) {
    throw new Error(`Credential references for ${pluginId} must use connector/${pluginId}/… or plugin/${pluginId}/…`);
  }
  const url = String(args.url ?? '').trim();
  if (!url) throw new Error('Credential request URL is required');
  const method = String(args.method ?? 'GET').trim().toUpperCase();
  if (!['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'].includes(method)) {
    throw new Error(`Unsupported credential request method ${method}`);
  }
  const headers = args.headers && typeof args.headers === 'object' && !Array.isArray(args.headers)
    ? args.headers as Record<string, unknown>
    : undefined;
  const body = typeof args.body === 'string' ? args.body : undefined;
  // Write approval is intentionally enforced again inside Electron main by the
  // credential gateway. Renderer/WebMCP code cannot mint or bypass it.
  const result = await invokeNativeDesktop<CredentialFetchResult>('egressFetch', {
    secretRef,
    url,
    method,
    ...(headers ? { headers } : {}),
    ...(body !== undefined ? { body } : {}),
  });
  return {
    status: result.status,
    statusText: result.statusText,
    headers: result.headers,
    url: result.url,
    credential: result.credential,
    body: decodeResponseBody(result),
    bodyBase64: result.bodyBase64,
  };
}

async function callLocalRuntimeTool(
  pluginId: string,
  tool: ToolDescriptor,
  args: Record<string, unknown>,
): Promise<unknown> {
  if (tool.name === credentialToolName) return callCredentialTool(pluginId, args);

  const bridge = window.mahayana;
  if (!bridge?.invoke) throw new Error('Mahayana Host bridge is unavailable');
  const key = `${pluginId}:${tool.name}`;
  if (pendingToolCalls.has(key)) throw new Error(`Tool ${tool.name} already has a pending call`);
  pendingToolCalls.add(key);

  try {
    const approval = tool.approval ?? (tool.annotations?.readOnlyHint === true ? 'none' : 'required');
    if (approval !== 'none') {
      const warning = approval === 'destructive'
        ? '该操作可能产生破坏性修改。'
        : '该操作会修改小程序或后台状态。';
      if (!window.confirm(`允许 ${pluginId} 调用 ${tool.name}？\n\n${warning}`)) {
        throw new Error('用户取消了 WebMCP Tool 调用');
      }
    }

    return await bridge.invoke('runtime.call', {
      pluginId,
      name: tool.name,
      arguments: args,
    });
  } finally {
    pendingToolCalls.delete(key);
  }
}

function webMcpBootstrap(pluginId: string, nonce: string): string {
  return `<script>(function(){
    const protocol=${JSON.stringify(protocol)};
    const pluginId=${JSON.stringify(pluginId)};
    const nonce=${JSON.stringify(nonce)};
    const pending=new Map(); let sequence=0; const localTools=new Map(); const controllers=[];
    function request(action,payload){return new Promise((resolve,reject)=>{const requestId='webmcp-'+Date.now()+'-'+(++sequence);pending.set(requestId,{resolve,reject});window.parent.postMessage({protocol,pluginId,nonce,requestId,action,...(payload||{})},'*');});}
    function publicTool(tool){const copy={...tool};delete copy.execute;return copy;}
    function register(item){
      const tool={name:item.name,description:item.description||item.name,inputSchema:item.inputSchema||{type:'object',properties:{}},annotations:{readOnlyHint:item.annotations?.readOnlyHint===true},execute:(input)=>request('call',{tool:item.name,input:input||{}})};
      localTools.set(tool.name,tool);
      if(document.modelContext&&typeof document.modelContext.registerTool==='function'){
        const controller=new AbortController(); controllers.push(controller);
        Promise.resolve(document.modelContext.registerTool(tool,{signal:controller.signal})).catch(()=>{});
      }
    }
    Object.defineProperty(window,'__fabushiWebMcp',{configurable:true,value:{version:1,list:()=>Array.from(localTools.values()).map(publicTool),call:async(name,input={})=>{const tool=localTools.get(name);if(!tool)throw new Error('Unknown WebMCP tool: '+name);return tool.execute(input);}}});
    window.addEventListener('message',(event)=>{const data=event.data||{};if(data.protocol!==protocol||data.pluginId!==pluginId||data.nonce!==nonce||!data.requestId||!pending.has(data.requestId))return;const task=pending.get(data.requestId);pending.delete(data.requestId);if(data.ok)task.resolve(data.data);else task.reject(new Error(data.error||'WebMCP host request failed'));});
    async function boot(){const tools=await request('list');for(const item of (tools||[]))register(item);window.dispatchEvent(new CustomEvent('fabushi:webmcp-ready',{detail:{pluginId,tools:(tools||[]).map(t=>t.name)}}));}
    function dispose(){for(const controller of controllers)controller.abort();controllers.length=0;window.parent.postMessage({protocol,pluginId,nonce,requestId:'dispose-'+Date.now(),action:'dispose'},'*');}
    window.addEventListener('pagehide',dispose,{once:true});
    if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',()=>void boot(),{once:true});else void boot();
  })();</script>`;
}

function injectWebMcp(html: string, pluginId: string, nonce: string): string {
  if (html.includes('fabushi.miniapp.webmcp.v1')) return html;
  const bootstrap = webMcpBootstrap(pluginId, nonce);
  return html.includes('</head>') ? html.replace('</head>', `${bootstrap}</head>`) : `${bootstrap}${html}`;
}

export function installDesktopMiniAppWebMcpHost(): void {
  const globalObject = window as Window & { [installMarker]?: boolean };
  if (globalObject[installMarker]) return;
  Object.defineProperty(globalObject, installMarker, { value: true });

  const prototype = ElectronMahayanaHostTransport.prototype as ElectronMahayanaHostTransport & {
    pluginUiDocument(pluginId: string): Promise<PluginUiDocument>;
  };
  const originalPluginUiDocument = prototype.pluginUiDocument;
  prototype.pluginUiDocument = async function pluginUiDocumentWithWebMcp(pluginId: string) {
    const document = await originalPluginUiDocument.call(this, pluginId);
    const nonce = crypto.randomUUID();
    validBridgeNonces.set(nonce, pluginId);
    return { ...document, html: injectWebMcp(document.html, pluginId, nonce) };
  };

  window.addEventListener('message', (event) => {
    const data = event.data as Record<string, unknown> | null;
    if (!data || data.protocol !== protocol || !data.requestId) return;
    const pluginId = safePluginId(data.pluginId);
    const nonce = String(data.nonce ?? '');
    const requestId = String(data.requestId);
    const source = event.source;
    if (!pluginId || !nonce || validBridgeNonces.get(nonce) !== pluginId) return;
    if (!source || typeof (source as WindowProxy).postMessage !== 'function') return;

    const respond = (ok: boolean, payload: unknown) => {
      (source as WindowProxy).postMessage({
        protocol,
        pluginId,
        nonce,
        requestId,
        ok,
        ...(ok ? { data: payload } : { error: payload instanceof Error ? payload.message : String(payload) }),
      }, '*');
    };

    void (async () => {
      try {
        if (data.action === 'dispose') {
          validBridgeNonces.delete(nonce);
          return;
        }
        const tools = await allowedTools(pluginId);
        if (data.action === 'list') {
          respond(true, tools.map(({ approval: _approval, annotations, ...tool }) => ({
            ...tool,
            annotations: { readOnlyHint: annotations?.readOnlyHint === true },
          })));
          return;
        }
        if (data.action === 'call') {
          const requested = String(data.tool ?? '').trim();
          const tool = tools.find((candidate) => candidate.name === requested);
          if (!tool) throw new Error(`WebMCP tool ${requested} is not allowed for ${pluginId}`);
          const input = data.input && typeof data.input === 'object' && !Array.isArray(data.input)
            ? data.input as Record<string, unknown>
            : {};
          respond(true, await callLocalRuntimeTool(pluginId, tool, input));
          return;
        }
        throw new Error(`Unsupported WebMCP action ${String(data.action)}`);
      } catch (error) {
        respond(false, error);
      }
    })();
  });
}
