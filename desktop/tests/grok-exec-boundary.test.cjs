'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const {createHostRuntime}=require('../electron/grok-host-runtime.cjs');
const {computerStateIdentity,toolDefinitions}=require('../electron/local-tool-executor.cjs');
const {createResource,ResourceRegistry}=require('../electron/grok-exec-resources.cjs');
const {rootRequestContext,childRequestContext,currentRequestContext,runWithRequestContext}=require('../electron/grok-request-context.cjs');

test('resource registry resolves child override without mutating parent',()=>{
  const resource=createResource('fixture');
  const parent=new ResourceRegistry().register(resource,{value:'parent'});
  const child=parent.child();
  assert.equal(child.get(resource).value,'parent');
  child.register(resource,{value:'child'});
  assert.equal(child.get(resource).value,'child');
  assert.equal(parent.get(resource).value,'parent');
});

test('model-visible local tool surface uses Grok protocol names and rejects legacy aliases',()=>{
  const names=toolDefinitions(new Set(['filesystem','shell','computer'])).map(row=>row.function.name).sort();
  assert.deepEqual(names,['Computer','Read','Screenshot','run_terminal_cmd']);
  for(const legacy of ['read_file','run_terminal','computer_screenshot','computer_click','computer_mouse_move','computer_drag','computer_scroll','computer_type','computer_key'])assert.equal(names.includes(legacy),false);
});

test('request context preserves parent identity across a child tool scope',async()=>{
  const root=rootRequestContext({agentId:'chief',conversationId:'conversation'});
  await runWithRequestContext(root,async()=>{
    assert.equal(currentRequestContext().requestId,root.requestId);
    const child=childRequestContext({toolCallId:'call-1'});
    await runWithRequestContext(child,async()=>{
      assert.equal(currentRequestContext().parentRequestId,root.requestId);
      assert.equal(currentRequestContext().agentId,'chief');
      assert.equal(currentRequestContext().toolCallId,'call-1');
    });
  });
});

test('host presents Grok model-facing names across local, state, agent, and subagent tools',async()=>{
  let exposed=[];
  const runtime=createHostRuntime({
    shell:{openExternal:async()=>{}},
    getLocalToolPermission:async()=> 'always',
    getAutoReviewMode:async()=> 'off',
    requestApproval:async()=>true,onToolState:async()=>{},onAgentStatus:async()=>{},getWorkflowContext:async()=> '',
    subagents:{async list(){return[]},async create(){return{}},async check(){return{}},async message(){return{}},async stop(){return{}}},
    agentManagement:{async send(){return'sent'},async create(){return{}},async update(){return{}}},
    inferenceRequest:async(_messages,tools)=>{exposed=tools.map(row=>row.function.name);return{message:{content:'ok'}}}
  });
  const transcript=[{id:'u1',role:'user',text:'inspect tools',createdAt:1,status:'done'}];
  const result=await runtime.runTurn({agent:{id:'chief',name:'Chief'},history:transcript,transcript,enabled:new Set(['filesystem','shell','computer']),signal:new AbortController().signal});
  assert.equal(result,'ok');
  for(const name of ['SendMessage','ReactToMessage','update_state','SendToAgent','CreateAgent','UpdateAgent','Read','run_terminal_cmd','Screenshot','Computer','Task','CheckSubagent','MessageSubagent','StopSubagent'])assert.equal(exposed.includes(name),true,name+' should be exposed');
  for(const legacy of ['read_file','run_terminal','computer_click','computer_screenshot','create_subagent','check_subagent','message_subagent','stop_subagent'])assert.equal(exposed.includes(legacy),false,legacy+' must stay hidden');
});

test('host routes browser, MCP, and subagent tools through distinct executor resources',async()=>{
  const calls=[];
  let inferenceRound=0;
  const browser={
    definitions(){return[{type:'function',function:{name:'browser_snapshot',description:'snapshot',parameters:{type:'object',properties:{},additionalProperties:false}}}]},
    isMutation(){return false},
    async execute({name}){calls.push('browser:'+name);return{text:'browser-ok'}}
  };
  const subagents={
    async check({agentId}){calls.push('subagent:'+agentId);return{agentId,status:'idle'}}
  };
  const runtime=createHostRuntime({
    shell:{openExternal:async()=>{}},
    getLocalToolPermission:async()=> 'always',
    requestApproval:async()=>true,
    onToolState:async()=>{},
    onAgentStatus:async()=>{},
    getWorkflowContext:async()=> '',
    browser,subagents,
    getExternalTools:async()=>[{type:'function',function:{name:'mcp__fixture__echo',description:'echo',parameters:{type:'object',properties:{text:{type:'string'}},additionalProperties:false}},_mcp:{serverId:'fixture',toolName:'echo'}}],
    executeExternalTool:async(name,args)=>{calls.push('external:'+name+':'+args.text);return{text:'mcp-ok'}},
    inferenceRequest:async()=>{
      inferenceRound++;
      if(inferenceRound===1)return{message:{content:null,tool_calls:[
        {id:'b1',type:'function',function:{name:'browser_snapshot',arguments:'{}'}},
        {id:'m1',type:'function',function:{name:'mcp__fixture__echo',arguments:'{"text":"hello"}'}},
        {id:'s1',type:'function',function:{name:'CheckSubagent',arguments:'{"subagent_id":"child"}'}}
      ]}};
      return{message:{content:'all routed'}};
    }
  });
  const transcript=[{id:'u1',role:'user',text:'run tools',createdAt:1,status:'done'}];
  const result=await runtime.runTurn({
    agent:{id:'chief',name:'Chief'},history:transcript,transcript,enabled:new Set(['browser']),signal:new AbortController().signal
  });
  assert.equal(result,'all routed');
  assert.deepEqual(calls,['browser:browser_snapshot','external:mcp__fixture__echo:hello','subagent:child']);
  const tools=transcript.filter(row=>row.role==='tool');
  assert.equal(tools.length,3);
  assert.deepEqual(tools.map(row=>row.status),['done','done','done']);
  assert.equal(tools.every(row=>typeof row.requestId==='string'&&row.requestId.length>0),true);
});


test('Computer display identity is stable for the same visible target and changes with window identity',()=>{
  const first=computerStateIdentity({application:'Safari',windowTitle:'Docs'});
  const same=computerStateIdentity({application:'Safari',windowTitle:'Docs'});
  const changed=computerStateIdentity({application:'Safari',windowTitle:'Checkout'});
  assert.equal(first,same);
  assert.notEqual(first,changed);
});


test('private inference deltas stay hidden while the host preserves a final-text fallback',async()=>{
  let round=0;
  const deltas=[];
  const runtime=createHostRuntime({
    shell:{openExternal:async()=>{}},
    getLocalToolPermission:async()=> 'always',
    getAutoReviewMode:async()=> 'off',
    requestApproval:async()=>true,
    onToolState:async()=>{},
    onAgentStatus:async()=>{},
    onAssistantDelta:async event=>{deltas.push(event.delta)},
    getWorkflowContext:async()=> '',
    inferenceRequest:async(_messages,_tools,_signal,onDelta)=>{
      round++;
      if(round===1)return{message:{content:null,tool_calls:[{id:'tool-1',type:'function',function:{name:'Read',arguments:'{"path":"package.json"}'}}]}};
      onDelta('private ','private ');
      onDelta('scratchpad','private scratchpad');
      return{message:{content:'final answer'}};
    }
  });
  const transcript=[{id:'u1',role:'user',text:'inspect then answer',createdAt:1,status:'done'}];
  const assistant={id:'a1',role:'assistant',text:'',createdAt:2,status:'streaming'};
  const text=await runtime.runTurn({
    agent:{id:'chief',name:'Chief'},history:[...transcript],transcript,enabled:new Set(['filesystem']),
    signal:new AbortController().signal,assistantEntry:assistant
  });
  assert.equal(text,'final answer');
  assert.deepEqual(deltas,[]);
  assert.ok(transcript.some(row=>row.role==='tool'));
  assert.equal(transcript.includes(assistant),false);
});
