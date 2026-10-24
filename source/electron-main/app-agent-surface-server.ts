import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import { chmod, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { randomBytes } from "node:crypto";
import { join } from "node:path";
import type { MainBrowserWindow } from "./main.js";

const MAX_REQUEST_BYTES = 128 * 1024;
const MAX_RESPONSE_BYTES = 4 * 1024 * 1024;
const OP_TIMEOUT_MS = 35_000;

type Operation = "status" | "snapshot" | "find" | "action" | "wait" | "assert";

function json(response: ServerResponse, status: number, value: unknown): void {
  const body = JSON.stringify(value);
  if (Buffer.byteLength(body) > MAX_RESPONSE_BYTES) { response.writeHead(413); response.end(JSON.stringify({ error: "response_too_large" })); return; }
  response.writeHead(status, { "content-type": "application/json", "cache-control": "no-store" });
  response.end(body);
}

async function readJson(request: IncomingMessage): Promise<Record<string, unknown>> {
  const chunks: Buffer[] = []; let size = 0;
  for await (const chunk of request) { const value = Buffer.from(chunk as Uint8Array); size += value.byteLength; if (size > MAX_REQUEST_BYTES) throw new Error("request_too_large"); chunks.push(value); }
  if (size === 0) return {};
  const parsed = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  if (parsed == null || typeof parsed !== "object" || Array.isArray(parsed)) throw new Error("input_must_be_object");
  return parsed as Record<string, unknown>;
}

function rendererScript(operation: Operation, input: Record<string, unknown>): string {
  const encoded = JSON.stringify(input).replace(/</g, "\\u003c");
  return `(async()=>{const input=${encoded};const w=window;w.__fabushiAgentGeneration=w.__fabushiAgentGeneration||1;const generation=w.__fabushiAgentGeneration;const nodes=[...document.querySelectorAll('[data-agent-id],[data-testid],button,a,input,textarea,select,[role]')];const sensitive=(e)=>/password|token|secret|api[-_ ]?key|credential/i.test((e.getAttribute('type')||'')+' '+(e.getAttribute('name')||'')+' '+(e.getAttribute('aria-label')||'')+' '+(e.getAttribute('data-agent-id')||''));const item=(e,i)=>({ref:'g'+generation+':'+i,agentId:e.getAttribute('data-agent-id')||e.getAttribute('data-testid')||e.id||undefined,role:e.getAttribute('role')||e.tagName.toLowerCase(),name:e.getAttribute('aria-label')||e.getAttribute('name')||e.textContent?.trim().slice(0,160)||'',text:e.textContent?.trim().slice(0,240)||'',enabled:!e.hasAttribute('disabled'),visible:!!(e.offsetWidth||e.offsetHeight||e.getClientRects().length),sensitive:sensitive(e),value:sensitive(e)?undefined:((e instanceof HTMLInputElement||e instanceof HTMLTextAreaElement||e instanceof HTMLSelectElement)?e.value:undefined)});const all=()=>nodes.map(item);const matches=(x)=>all().filter(e=>(input.agentId==null||e.agentId===input.agentId)&&(input.ref==null||e.ref===input.ref)&&(input.role==null||e.role===input.role)&&(input.name==null||e.name.toLowerCase().includes(String(input.name).toLowerCase()))&&(input.text==null||e.text.toLowerCase().includes(String(input.text).toLowerCase())));if(${JSON.stringify(operation)}==='status')return {version:1,appId:'fabushi.desktop',available:true,platform:'electron',generation};if(${JSON.stringify(operation)}==='snapshot'){const limit=Math.min(Math.max(Number(input.maxElements||500),1),500);const elements=all().slice(0,limit);return {version:1,appId:'fabushi.desktop',platform:'electron',generation,elements,truncated:all().length>limit};}if(${JSON.stringify(operation)}==='find'){const found=matches(input).slice(0,Math.min(Math.max(Number(input.limit||100),1),100));return {version:1,appId:'fabushi.desktop',platform:'electron',generation,count:found.length,matches:found};}if(${JSON.stringify(operation)}==='assert'){const found=matches(input);const state=input.state||'present';const passed=state==='present'?found.length>0:state==='absent'?found.length===0:found.some(e=>state==='enabled'?e.enabled:state==='disabled'?!e.enabled:state==='visible'?e.visible:state==='hidden'?!e.visible:true);return {version:1,appId:'fabushi.desktop',platform:'electron',generation,passed,observed:found.slice(0,10)};}if(${JSON.stringify(operation)}==='wait'){const deadline=Date.now()+Math.min(Math.max(Number(input.timeoutMs||5000),100),30000);while(Date.now()<deadline){const found=matches(input);const state=input.state||'present';const ok=state==='present'?found.length>0:state==='absent'?found.length===0:found.some(e=>state==='enabled'?e.enabled:state==='disabled'?!e.enabled:state==='visible'?e.visible:state==='hidden'?!e.visible:true);if(ok)return {version:1,appId:'fabushi.desktop',platform:'electron',generation:w.__fabushiAgentGeneration,matched:true,observed:found.slice(0,10)};await new Promise(r=>setTimeout(r,100));}return {version:1,appId:'fabushi.desktop',platform:'electron',generation:w.__fabushiAgentGeneration,matched:false};}if(${JSON.stringify(operation)}==='action'){if(Number(input.generation)!==generation)throw new Error('stale_app_surface_generation');if(sensitive(nodes.find(e=>(input.agentId&&((e.getAttribute('data-agent-id')||e.getAttribute('data-testid')||e.id)===input.agentId))||(input.ref&&item(e, nodes.indexOf(e)).ref===input.ref))))throw new Error('sensitive_target');const e=nodes.find(e=>(input.agentId&&((e.getAttribute('data-agent-id')||e.getAttribute('data-testid')||e.id)===input.agentId))||(input.ref&&item(e,nodes.indexOf(e)).ref===input.ref));if(!e)throw new Error('target_not_found');const action=String(input.action||'invoke');if(action==='invoke'){(e instanceof HTMLElement)&&e.click();}else if(action==='focus'){(e instanceof HTMLElement)&&e.focus();}else if(action==='toggle'){(e instanceof HTMLInputElement)?(e.checked=!e.checked):(e instanceof HTMLElement)&&e.click();}else if(action==='setValue'){if(sensitive(e))throw new Error('sensitive_target');if(!(e instanceof HTMLInputElement||e instanceof HTMLTextAreaElement||e instanceof HTMLSelectElement))throw new Error('target_not_writable');e.value=String(input.value??'');e.dispatchEvent(new Event('input',{bubbles:true}));e.dispatchEvent(new Event('change',{bubbles:true}));}else if(action==='pressKey'){(e instanceof HTMLElement)&&e.dispatchEvent(new KeyboardEvent('keydown',{key:String(input.value||'Enter'),bubbles:true}));}else throw new Error('unsupported_action');w.__fabushiAgentGeneration++;return {version:1,appId:'fabushi.desktop',platform:'electron',status:'completed',generation:w.__fabushiAgentGeneration,target:item(e,nodes.indexOf(e))};}throw new Error('unsupported_operation');})()`;
}

export interface AppAgentSurfaceServer {
  start(): Promise<void>;
  close(): Promise<void>;
}

export function createAppAgentSurfaceServer(options: { app: { getPath(name: "userData"): string }; getWindow: () => MainBrowserWindow | null; appVersion?: string; env?: NodeJS.ProcessEnv }): AppAgentSurfaceServer {
  const discoveryPath = options.env?.FABUSHI_APP_AGENT_DISCOVERY_FILE?.trim() || join(options.app.getPath("userData"), "agent-surface", "bridge.json");
  const token = randomBytes(48).toString("base64url");
  let server: ReturnType<typeof createServer> | null = null;
  let endpoint = "";
  return {
    async start() {
      if (server != null) return;
      await mkdir(join(discoveryPath, ".."), { recursive: true, mode: 0o700 });
      server = createServer(async (request, response) => {
        try {
          if (request.method !== "POST" || !request.url?.startsWith("/v1/")) return json(response, 404, { error: "not_found" });
          if (request.headers.authorization !== `Bearer ${token}`) return json(response, 401, { error: "unauthorized" });
          const operation = request.url.slice(4).split("?")[0] as Operation;
          if (!["status", "snapshot", "find", "action", "wait", "assert"].includes(operation)) return json(response, 404, { error: "unsupported_operation" });
          const envelope = await readJson(request);
          const input = envelope.input && typeof envelope.input === "object" && !Array.isArray(envelope.input)
            ? envelope.input as Record<string, unknown>
            : envelope;
          const win = options.getWindow();
          if (win == null || win.isDestroyed() || win.webContents.isDestroyed()) return json(response, 503, { error: "app_surface_renderer_unavailable" });
          const result = await Promise.race([win.webContents.executeJavaScript(rendererScript(operation, input), true), new Promise((_, reject) => setTimeout(() => reject(new Error("operation_timeout")), OP_TIMEOUT_MS))]);
          return json(response, 200, result);
        } catch (error) { return json(response, 400, { error: error instanceof Error ? error.message : "app_surface_request_failed" }); }
      });
      await new Promise<void>((resolve, reject) => { server!.once("error", reject); server!.listen(0, "127.0.0.1", () => resolve()); });
      const address = server.address(); if (address == null || typeof address === "string") throw new Error("app_surface_bind_failed");
      endpoint = `http://127.0.0.1:${address.port}`;
      const origin = `${endpoint}/`;
      const discovery = { version: 1, appId: "fabushi.desktop", platform: "electron", pid: process.pid, port: address.port, token, endpoint, origin, ...(options.appVersion ? { appVersion: options.appVersion } : {}) };
      await writeFile(discoveryPath, JSON.stringify(discovery), { mode: 0o600 }); await chmod(discoveryPath, 0o600);
    },
    async close() { if (server != null) await new Promise<void>((resolve) => server!.close(() => resolve())); server = null; endpoint = ""; await rm(discoveryPath, { force: true }); },
  };
}


