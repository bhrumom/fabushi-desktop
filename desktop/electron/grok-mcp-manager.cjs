'use strict';
const {spawn}=require('node:child_process');const crypto=require('node:crypto');
const PROTOCOL_VERSION='2025-06-18',CLIENT_INFO={name:'Fabushi',version:'2.0.0-alpha.1'};
function toolRows(result){return(Array.isArray(result?.tools)?result.tools:[]).filter(x=>x&&typeof x.name==='string').map(x=>({name:x.name,description:typeof x.description==='string'?x.description:'',inputSchema:x.inputSchema&&typeof x.inputSchema==='object'?x.inputSchema:{type:'object',properties:{}}}))}
function resultText(result){const content=Array.isArray(result?.content)?result.content:[],text=content.map(x=>x?.type==='text'?x.text:JSON.stringify(x)).filter(Boolean).join('\n');if(result?.isError)throw Error(text||'MCP tool returned an error.');return{text:text||JSON.stringify(result??{})}}
class StdioMcpClient{
  constructor(config){this.config=config;this.child=null;this.buffer='';this.nextId=1;this.pending=new Map();this.initialized=false;this.stderr=''}
  async start(){if(this.child&&this.initialized)return;const child=spawn(this.config.command,this.config.args||[],{stdio:['pipe','pipe','pipe'],env:process.env});this.child=child;child.stdout.setEncoding('utf8');child.stderr.setEncoding('utf8');child.stdout.on('data',x=>this.onData(x));child.stderr.on('data',x=>{this.stderr=(this.stderr+x).slice(-20000)});child.on('error',e=>this.failAll(e));child.on('exit',(code,signal)=>{this.initialized=false;this.child=null;this.failAll(Error('MCP server exited (code='+code+', signal='+(signal||'none')+'): '+this.stderr.slice(-2000)))});
    const result=await this.request('initialize',{protocolVersion:PROTOCOL_VERSION,capabilities:{},clientInfo:CLIENT_INFO},15000);if(!result||typeof result!=='object')throw Error('MCP initialize returned an invalid result.');this.notify('notifications/initialized',{});this.initialized=true}
  onData(chunk){this.buffer+=chunk;for(;;){const newline=this.buffer.indexOf('\n');if(newline<0)break;const line=this.buffer.slice(0,newline).trim();this.buffer=this.buffer.slice(newline+1);if(!line)continue;let message;try{message=JSON.parse(line)}catch{continue}if(message&&Object.prototype.hasOwnProperty.call(message,'id')){const pending=this.pending.get(message.id);if(!pending)continue;this.pending.delete(message.id);clearTimeout(pending.timer);message.error?pending.reject(Error(message.error.message||'MCP request failed')):pending.resolve(message.result)}}}
  failAll(error){for(const[id,p]of this.pending){clearTimeout(p.timer);p.reject(error);this.pending.delete(id)}}
  send(message){if(!this.child||this.child.killed||!this.child.stdin.writable)throw Error('MCP server is not running.');this.child.stdin.write(JSON.stringify(message)+'\n')}
  notify(method,params){this.send({jsonrpc:'2.0',method,params})}
  request(method,params,timeoutMs=30000){const id=this.nextId++;return new Promise((resolve,reject)=>{const timer=setTimeout(()=>{this.pending.delete(id);reject(Error('MCP request timed out: '+method))},timeoutMs);this.pending.set(id,{resolve,reject,timer});try{this.send({jsonrpc:'2.0',id,method,params})}catch(error){clearTimeout(timer);this.pending.delete(id);reject(error)}})}
  async listTools(){await this.start();return toolRows(await this.request('tools/list',{}))}
  async callTool(name,args){await this.start();return await this.request('tools/call',{name,arguments:args||{}},120000)}
  dispose(){this.initialized=false;this.failAll(Error('MCP client disposed.'));if(this.child&&!this.child.killed)this.child.kill('SIGTERM');this.child=null}
}
class HttpMcpClient{
  constructor(config,getAuthorizationHeader=async()=>null,getExtraHeaders=async()=>({})){this.config=config;this.getAuthorizationHeader=getAuthorizationHeader;this.getExtraHeaders=getExtraHeaders;this.nextId=1;this.initialized=false;this.sessionId=null;this.protocolVersion=PROTOCOL_VERSION}
  async headers(){
    const authorization=await this.getAuthorizationHeader(this.config),raw=await this.getExtraHeaders(this.config),extra=raw&&typeof raw==='object'&&!Array.isArray(raw)?Object.fromEntries(Object.entries(raw).map(([key,value])=>[String(key),String(value)])):{};
    return{...extra,'content-type':'application/json',accept:'application/json, text/event-stream','mcp-protocol-version':this.protocolVersion,...(this.sessionId?{'mcp-session-id':this.sessionId}:{}),...(authorization?{authorization}:{})}
  }
  parseSse(text,requestId){const rows=[];for(const line of text.split(/\r?\n/)){if(!line.startsWith('data:'))continue;const raw=line.slice(5).trim();if(!raw||raw==='[DONE]')continue;try{rows.push(JSON.parse(raw))}catch{}}return rows.find(x=>x&&x.id===requestId)||rows.find(x=>x&&Object.prototype.hasOwnProperty.call(x,'result'))||rows.at(-1)||null}
  async send(message,{notification=false,timeoutMs=30000}={}){const controller=new AbortController(),timer=setTimeout(()=>controller.abort(),timeoutMs);try{const response=await fetch(this.config.url,{method:'POST',headers:await this.headers(),body:JSON.stringify(message),signal:controller.signal});const session=response.headers.get('mcp-session-id');if(session)this.sessionId=session;if(response.status===401){const error=Error('MCP server requires authentication. Connect an OAuth account for this server.');error.code='MCP_AUTH_REQUIRED';error.wwwAuthenticate=response.headers.get('www-authenticate');throw error}if(!response.ok)throw Error('MCP HTTP '+response.status+': '+(await response.text()).slice(0,2000));if(notification||response.status===202)return null;const type=response.headers.get('content-type')||'',raw=await response.text();if(!raw.trim())return null;const payload=type.includes('text/event-stream')?this.parseSse(raw,message.id):JSON.parse(raw);if(payload?.error)throw Error(payload.error.message||'MCP request failed');return payload?.result}finally{clearTimeout(timer)}}
  async start(){if(this.initialized)return;const result=await this.send({jsonrpc:'2.0',id:this.nextId++,method:'initialize',params:{protocolVersion:PROTOCOL_VERSION,capabilities:{},clientInfo:CLIENT_INFO}},{timeoutMs:15000});if(!result||typeof result!=='object')throw Error('MCP initialize returned an invalid result.');if(typeof result.protocolVersion==='string')this.protocolVersion=result.protocolVersion;await this.send({jsonrpc:'2.0',method:'notifications/initialized',params:{}},{notification:true,timeoutMs:15000});this.initialized=true}
  async request(method,params,timeoutMs=30000){await this.start();return await this.send({jsonrpc:'2.0',id:this.nextId++,method,params},{timeoutMs})}
  async listTools(){return toolRows(await this.request('tools/list',{}))}
  async callTool(name,args){return await this.request('tools/call',{name,arguments:args||{}},120000)}
  dispose(){if(this.sessionId)void this.headers().then(headers=>fetch(this.config.url,{method:'DELETE',headers})).catch(()=>{});this.initialized=false;this.sessionId=null}
}
function normalizeServer(input){
  const name=String(input?.name||'').trim().slice(0,80);if(!name)throw Error('MCP server name is required.');
  const transport=input?.transport==='http'||String(input?.url||'').trim()?'http':'stdio';
  const accountKey=String(input?.accountKey||'default').trim().slice(0,80)||'default';
  const accountKeys=[...new Set([accountKey,...(Array.isArray(input?.accountKeys)?input.accountKeys.map(value=>String(value).trim().slice(0,80)).filter(Boolean):[])])];
  const common={id:String(input?.id||crypto.randomUUID()),name,transport,enabled:input?.enabled!==false,disabledTools:Array.isArray(input?.disabledTools)?input.disabledTools.map(String):[],customInstructions:String(input?.customInstructions||'').trim().slice(0,12000),accountKey,accountKeys};
  if(transport==='http'){
    const parseOptionalUrl=(value,label)=>{const raw=String(value||'').trim();if(!raw)return'';let parsed;try{parsed=new URL(raw)}catch{throw Error(label+' is invalid.')}if(parsed.protocol!=='https:'&&parsed.protocol!=='http:')throw Error(label+' must use HTTP(S).');if(parsed.username||parsed.password)throw Error('Do not put credentials in '+label+'.');return parsed.toString()};
    const url=parseOptionalUrl(input?.url,'MCP server URL');if(!url)throw Error('MCP server URL is required.');
    return{...common,url,command:'',args:[],
      oauthClientId:String(input?.oauthClientId||'').trim().slice(0,500),
      oauthAuthorizationUrl:parseOptionalUrl(input?.oauthAuthorizationUrl,'OAuth authorization URL'),
      oauthTokenUrl:parseOptionalUrl(input?.oauthTokenUrl,'OAuth token URL'),
      oauthRegistrationUrl:parseOptionalUrl(input?.oauthRegistrationUrl,'OAuth registration URL'),
      oauthScopes:Array.isArray(input?.oauthScopes)?input.oauthScopes.map(value=>String(value).trim()).filter(Boolean).slice(0,64):[]
    }
  }
  const command=String(input?.command||'').trim(),args=Array.isArray(input?.args)?input.args.map(x=>String(x)).slice(0,32):[];if(!command)throw Error('MCP server command is required.');if(command.includes('\0')||args.some(x=>x.includes('\0')))throw Error('Invalid MCP command.');return{...common,command,args,url:''}
}
function createMcpManager({getServers,getAuthorizationHeader=async()=>null,getExtraHeaders=async()=>({})}){const clients=new Map();
  const clientFor=server=>{let client=clients.get(server.id);const signature=JSON.stringify([server.transport,server.command,server.args,server.url,server.accountKey,server.oauthClientId,server.oauthAuthorizationUrl,server.oauthTokenUrl]);if(client&&client.signature!==signature){client.instance.dispose();clients.delete(server.id);client=null}if(!client){client={signature,instance:server.transport==='http'?new HttpMcpClient(server,getAuthorizationHeader,getExtraHeaders):new StdioMcpClient(server)};clients.set(server.id,client)}return client.instance};
  const find=async id=>{const server=(await getServers()).find(x=>x.id===id);if(!server)throw Error('MCP server not found.');if(server.enabled===false)throw Error('MCP server is disabled.');return server};
  return{normalizeServer,
    async listServerTools(serverId){const server=await find(serverId),tools=await clientFor(server).listTools(),disabled=new Set(server.disabledTools||[]);return tools.map(x=>({...x,isDisabled:disabled.has(x.name)}))},
    async collectToolDefinitions(){const definitions=[];for(const server of await getServers()){if(server.enabled===false)continue;try{const tools=await clientFor(server).listTools(),disabled=new Set(server.disabledTools||[]);for(const tool of tools){if(disabled.has(tool.name))continue;definitions.push({type:'function',function:{name:'mcp__'+server.id.replace(/[^a-zA-Z0-9_-]/g,'_')+'__'+tool.name.replace(/[^a-zA-Z0-9_-]/g,'_'),description:'['+server.name+'] '+(tool.description||tool.name)+(server.customInstructions?' Guidance: '+server.customInstructions:''),parameters:tool.inputSchema||{type:'object',properties:{}}},_mcp:{serverId:server.id,toolName:tool.name}})}}catch{}}return definitions},
    async executeRoutedTool(name,args){if(!String(name).startsWith('mcp__'))return null;const definitions=await this.collectToolDefinitions(),definition=definitions.find(x=>x.function.name===name);if(!definition)throw Error('MCP tool is unavailable or disabled.');const server=await find(definition._mcp.serverId);return resultText(await clientFor(server).callTool(definition._mcp.toolName,args))},
    disposeServer(serverId){const client=clients.get(serverId);client?.instance.dispose();clients.delete(serverId)},dispose(){for(const client of clients.values())client.instance.dispose();clients.clear()}
  }}
module.exports={createMcpManager,normalizeServer,StdioMcpClient,HttpMcpClient};
