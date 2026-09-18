'use strict';

const {spawn}=require('node:child_process');
const crypto=require('node:crypto');

class StdioMcpClient{
  constructor(config){
    this.config=config;this.child=null;this.buffer='';this.nextId=1;this.pending=new Map();this.initialized=false;this.stderr='';
  }
  async start(){
    if(this.child&&this.initialized)return;
    const child=spawn(this.config.command,this.config.args||[],{stdio:['pipe','pipe','pipe'],env:process.env});
    this.child=child;
    child.stdout.setEncoding('utf8');child.stderr.setEncoding('utf8');
    child.stdout.on('data',chunk=>this.onData(chunk));
    child.stderr.on('data',chunk=>{this.stderr=(this.stderr+chunk).slice(-20000)});
    child.on('error',error=>this.failAll(error));
    child.on('exit',(code,signal)=>{
      const error=Error(`MCP server exited (code=${code}, signal=${signal||'none'}): ${this.stderr.slice(-2000)}`);
      this.initialized=false;this.child=null;this.failAll(error);
    });
    const result=await this.request('initialize',{
      protocolVersion:'2025-06-18',
      capabilities:{},
      clientInfo:{name:'Fabushi',version:'2.0.0-alpha.1'}
    },15000);
    if(!result||typeof result!=='object')throw Error('MCP initialize returned an invalid result.');
    this.notify('notifications/initialized',{});this.initialized=true;
  }
  onData(chunk){
    this.buffer+=chunk;
    for(;;){
      const newline=this.buffer.indexOf('\n');if(newline<0)break;
      const line=this.buffer.slice(0,newline).trim();this.buffer=this.buffer.slice(newline+1);
      if(!line)continue;
      let message;try{message=JSON.parse(line)}catch{continue}
      if(message&&Object.prototype.hasOwnProperty.call(message,'id')){
        const pending=this.pending.get(message.id);if(!pending)continue;
        this.pending.delete(message.id);clearTimeout(pending.timer);
        if(message.error)pending.reject(Error(message.error.message||'MCP request failed'));
        else pending.resolve(message.result);
      }
    }
  }
  failAll(error){
    for(const [id,pending] of this.pending){clearTimeout(pending.timer);pending.reject(error);this.pending.delete(id)}
  }
  send(message){
    if(!this.child||this.child.killed||!this.child.stdin.writable)throw Error('MCP server is not running.');
    this.child.stdin.write(JSON.stringify(message)+'\n');
  }
  notify(method,params){this.send({jsonrpc:'2.0',method,params})}
  request(method,params,timeoutMs=30000){
    const id=this.nextId++;
    return new Promise((resolve,reject)=>{
      const timer=setTimeout(()=>{this.pending.delete(id);reject(Error(`MCP request timed out: ${method}`))},timeoutMs);
      this.pending.set(id,{resolve,reject,timer});
      try{this.send({jsonrpc:'2.0',id,method,params})}catch(error){clearTimeout(timer);this.pending.delete(id);reject(error)}
    });
  }
  async listTools(){
    await this.start();const result=await this.request('tools/list',{});
    const tools=Array.isArray(result?.tools)?result.tools:[];
    return tools.filter(tool=>tool&&typeof tool.name==='string').map(tool=>({
      name:tool.name,description:typeof tool.description==='string'?tool.description:'',
      inputSchema:tool.inputSchema&&typeof tool.inputSchema==='object'?tool.inputSchema:{type:'object',properties:{}}
    }));
  }
  async callTool(name,args){
    await this.start();return await this.request('tools/call',{name,arguments:args||{}},120000);
  }
  dispose(){
    this.initialized=false;this.failAll(Error('MCP client disposed.'));
    if(this.child&&!this.child.killed)this.child.kill('SIGTERM');this.child=null;
  }
}

function normalizeServer(input){
  const name=String(input?.name||'').trim().slice(0,80);const command=String(input?.command||'').trim();
  const args=Array.isArray(input?.args)?input.args.map(x=>String(x)).slice(0,32):[];
  if(!name)throw Error('MCP server name is required.');
  if(!command)throw Error('MCP server command is required.');
  if(command.includes('\0')||args.some(x=>x.includes('\0')))throw Error('Invalid MCP command.');
  return{id:String(input?.id||crypto.randomUUID()),name,command,args,enabled:input?.enabled!==false,disabledTools:Array.isArray(input?.disabledTools)?input.disabledTools.map(String):[]};
}
function createMcpManager({getServers}){
  const clients=new Map();
  const clientFor=server=>{
    let client=clients.get(server.id);
    const signature=JSON.stringify([server.command,server.args]);
    if(client&&client.signature!==signature){client.instance.dispose();clients.delete(server.id);client=null}
    if(!client){client={signature,instance:new StdioMcpClient(server)};clients.set(server.id,client)}
    return client.instance;
  };
  const find=async id=>{
    const server=(await getServers()).find(x=>x.id===id);if(!server)throw Error('MCP server not found.');
    if(server.enabled===false)throw Error('MCP server is disabled.');return server;
  };
  return{
    normalizeServer,
    async listServerTools(serverId){
      const server=await find(serverId);const tools=await clientFor(server).listTools();
      const disabled=new Set(server.disabledTools||[]);
      return tools.map(tool=>({...tool,isDisabled:disabled.has(tool.name)}));
    },
    async collectToolDefinitions(){
      const definitions=[];
      for(const server of await getServers()){
        if(server.enabled===false)continue;
        try{
          const tools=await clientFor(server).listTools(),disabled=new Set(server.disabledTools||[]);
          for(const tool of tools){
            if(disabled.has(tool.name))continue;
            definitions.push({
              type:'function',
              function:{
                name:`mcp__${server.id.replace(/[^a-zA-Z0-9_-]/g,'_')}__${tool.name.replace(/[^a-zA-Z0-9_-]/g,'_')}`,
                description:`[${server.name}] ${tool.description||tool.name}`,
                parameters:tool.inputSchema||{type:'object',properties:{}}
              },
              _mcp:{serverId:server.id,toolName:tool.name}
            });
          }
        }catch{}
      }
      return definitions;
    },
    async executeRoutedTool(name,args){
      const prefix='mcp__';if(!String(name).startsWith(prefix))return null;
      const definitions=await this.collectToolDefinitions();const definition=definitions.find(x=>x.function.name===name);
      if(!definition)throw Error('MCP tool is unavailable or disabled.');
      const server=await find(definition._mcp.serverId);
      const result=await clientFor(server).callTool(definition._mcp.toolName,args);
      const content=Array.isArray(result?.content)?result.content:[];
      const text=content.map(item=>item?.type==='text'?item.text:JSON.stringify(item)).filter(Boolean).join('\n');
      if(result?.isError)throw Error(text||'MCP tool returned an error.');
      return{text:text||JSON.stringify(result??{})};
    },
    disposeServer(serverId){const client=clients.get(serverId);client?.instance.dispose();clients.delete(serverId)},
    dispose(){for(const client of clients.values())client.instance.dispose();clients.clear()}
  };
}
module.exports={createMcpManager,normalizeServer};
