'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs/promises');
const os=require('node:os');
const path=require('node:path');
const http=require('node:http');
const {createCoordinatorRuntime}=require('../electron/grok-agent-coordinator.cjs');
const {createWorkflowManager}=require('../electron/grok-workflow-manager.cjs');

async function fixture(){
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-grok-runtime-'));
  const app={getPath(name){if(name!=='userData')throw Error('unexpected app path: '+name);return root}};
  const BrowserWindow={getAllWindows(){return[]}};
  const shell={openExternal:async()=>{}};
  return{root,app,BrowserWindow,shell};
}

test('coordinator persists MCP server records and exposes only executable catalog entries',async t=>{
  const f=await fixture();t.after(()=>fs.rm(f.root,{recursive:true,force:true}));
  const first=createCoordinatorRuntime(f);
  const added=await first.addMcpServer({name:'Local test',command:'/usr/bin/printf',args:['ready']});
  assert.equal(added.name,'Local test');
  const listed=await first.listMcpServers();
  assert.equal(listed.length,1);
  assert.equal(listed[0].command,'/usr/bin/printf');
  const plugins=await first.listPlugins();
  assert.deepEqual(plugins.filter(item=>item.kind==='local').map(item=>item.id),['filesystem','shell','browser','computer']);
  assert.equal(plugins.some(item=>item.name==='GitHub'||item.name==='Memory'),false);
  assert.equal(plugins.some(item=>item.id==='mcp:'+added.id),true);

  const second=createCoordinatorRuntime(f);
  const reloaded=await second.listMcpServers();
  assert.equal(reloaded.length,1);
  assert.equal(reloaded[0].id,added.id);
});

test('workflow manager stores real SKILL.md and reloads trigger metadata',async t=>{
  const f=await fixture();t.after(()=>fs.rm(f.root,{recursive:true,force:true}));
  const manager=createWorkflowManager({app:f.app});
  const saved=await manager.saveWorkflow({
    name:'Morning review',
    description:'Review the current project each morning.',
    body:'Inspect the current project state and report material blockers.',
    trigger:{schedule:'0 8 * * 1-5',isEnabled:true},
    isEnabledForAgent:true
  });
  assert.equal(path.basename(saved.filePath),'SKILL.md');
  const stat=await fs.stat(saved.filePath);
  assert.equal(stat.isFile(),true);
  const rows=await manager.list();
  assert.equal(rows.length,1);
  assert.equal(rows[0].name,'Morning review');
  assert.equal(rows[0].trigger.schedule,'0 8 * * 1-5');
  const context=await manager.buildAgentContext('@morningreview run it');
  assert.match(context,/Selected workflow instructions/);
  assert.match(context,/Inspect the current project state/);
  await manager.deleteWorkflow({id:saved.id});
  assert.equal((await manager.list()).length,0);
});

test('local mutation permission setting persists through coordinator restart',async t=>{
  const f=await fixture();t.after(()=>fs.rm(f.root,{recursive:true,force:true}));
  const first=createCoordinatorRuntime(f);
  await first.setLocalToolPermission({permission:'never'});
  const second=createCoordinatorRuntime(f);
  assert.equal((await second.getRuntimeSettings()).localToolPermission,'never');
});


test('per-agent routine runNow executes through host and records embedded run history',async t=>{
  const f=await fixture();t.after(()=>fs.rm(f.root,{recursive:true,force:true}));
  const runtime=createCoordinatorRuntime(f);
  const agent=(await runtime.listAgents())[0];
  const created=await runtime.createAgentAutomation({
    id:agent.id,
    spec:{name:'Contract routine',prompt:'Report that the routine executed.',trigger:{type:'cron',schedule:'@daily'},isEnabled:false}
  });
  assert.equal(created.length,1);
  const routine=created[0];
  assert.equal(routine.trigger.schedule,'@daily');
  await runtime.runAgentAutomationNow({id:agent.id,automationId:routine.id});
  const rows=await runtime.getAgentAutomations({id:agent.id});
  assert.equal(rows[0].runs.length,1);
  assert.equal(rows[0].runs[0].status,'ok');
  assert.equal(rows[0].runs[0].event,'manual');
  assert.match(rows[0].runs[0].detail,/Agent host is ready on this Mac/);
});


test('remote HTTP MCP performs initialize and tools/list over a real local HTTP endpoint',async t=>{
  const requests=[];
  const server=http.createServer((req,res)=>{
    let raw='';req.setEncoding('utf8');req.on('data',chunk=>{raw+=chunk});req.on('end',()=>{
      const message=JSON.parse(raw||'{}');requests.push(message.method);
      if(message.method==='notifications/initialized'){res.statusCode=202;res.end();return}
      res.setHeader('content-type','application/json');
      if(message.method==='initialize'){res.end(JSON.stringify({jsonrpc:'2.0',id:message.id,result:{protocolVersion:'2025-06-18',capabilities:{},serverInfo:{name:'fixture',version:'1'}}}));return}
      if(message.method==='tools/list'){res.end(JSON.stringify({jsonrpc:'2.0',id:message.id,result:{tools:[{name:'echo',description:'Echo input',inputSchema:{type:'object',properties:{text:{type:'string'}}}}]}}));return}
      if(message.method==='tools/call'){res.end(JSON.stringify({jsonrpc:'2.0',id:message.id,result:{content:[{type:'text',text:String(message.params?.arguments?.text||'')}]}}));return}
      res.statusCode=400;res.end('{}');
    });
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  t.after(()=>new Promise(resolve=>server.close(resolve)));
  const address=server.address();assert.equal(typeof address,'object');
  const f=await fixture();t.after(()=>fs.rm(f.root,{recursive:true,force:true}));
  const runtime=createCoordinatorRuntime(f);
  const added=await runtime.addMcpServer({name:'Remote fixture',transport:'http',url:'http://127.0.0.1:'+address.port+'/mcp',customInstructions:'Use only for fixture echoes.'});
  assert.equal(added.transport,'http');
  const tools=await runtime.listMcpServerTools({serverId:added.id});
  assert.equal(tools.length,1);assert.equal(tools[0].name,'echo');
  assert.deepEqual(requests.slice(0,3),['initialize','notifications/initialized','tools/list']);
  const listed=await runtime.listMcpServers();
  assert.equal(listed[0].customInstructions,'Use only for fixture echoes.');
});
