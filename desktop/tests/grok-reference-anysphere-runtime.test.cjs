'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');

test('exact AnysphereAgent adapter owns plain-text turn lifecycle',async()=>{
  const {createLocalAnysphereRuntime}=require('../electron/grok-reference-anysphere-runtime.cjs');
  const calls=[];
  const runtime=createLocalAnysphereRuntime({
    conversationId:'a1',
    modelId:'test-model',
    systemPrompt:()=> 'You are a local Grok parity agent.',
    getTools:()=>[],
    executeTool:async()=>{throw Error('unexpected tool')},
    transport:async(messages,tools,_signal,onDelta)=>{
      calls.push({messages,tools});
      onDelta?.('hello','hello');
      return{message:{content:'hello'},usage:{inputTokens:3,outputTokens:1}};
    }
  });
  const result=await runtime.run({prompt:'hi'});
  assert.equal(result.engine,'grok-anysphere-agent');
  assert.match(result.text,/hello/);
  assert.equal(runtime.snapshot().engine,'grok-anysphere-agent');
  assert.ok(runtime.snapshot().stateBytes>0);
  assert.equal(calls.length,1);
});

test('exact AnysphereAgent adapter executes a local tool and continues model turn',async()=>{
  const {createLocalAnysphereRuntime}=require('../electron/grok-reference-anysphere-runtime.cjs');
  let round=0,executed=null;
  const runtime=createLocalAnysphereRuntime({
    conversationId:'a2',
    modelId:'test-model',
    systemPrompt:()=> 'Use local tools when needed.',
    getTools:()=>[{type:'function',function:{name:'echo_local',description:'Echo local text',parameters:{type:'object',properties:{text:{type:'string'}},required:['text'],additionalProperties:false}}}],
    executeTool:async(name,args)=>{executed={name,args};return{text:'LOCAL:'+args.text}},
    transport:async(messages)=>{
      round+=1;
      if(round===1)return{message:{content:null,tool_calls:[{id:'tc1',type:'function',function:{name:'echo_local',arguments:'{"text":"ok"}'}}]},usage:{inputTokens:4,outputTokens:2}};
      assert.ok(messages.some(message=>message.role==='tool'));
      return{message:{content:'done'},usage:{inputTokens:5,outputTokens:1}};
    }
  });
  const result=await runtime.run({prompt:'run it'});
  assert.deepEqual(executed,{name:'echo_local',args:{text:'ok'}});
  assert.equal(round,2);
  assert.match(result.text,/done/);
});


test('provider-private deltas never become committed Agent output',async()=>{
  const {createLocalAnysphereRuntime}=require('../electron/grok-reference-anysphere-runtime.cjs');
  let round=0;
  const runtime=createLocalAnysphereRuntime({
    conversationId:'privacy',
    modelId:'test-model',
    systemPrompt:()=> 'Keep private inference private.',
    getTools:()=>[{type:'function',function:{name:'echo_local',description:'Echo',parameters:{type:'object',properties:{text:{type:'string'}},required:['text']}}}],
    executeTool:async()=>({text:'ok'}),
    transport:async(_messages,_tools,_signal,onDelta)=>{
      round+=1;
      if(round===1)return{message:{content:null,tool_calls:[{id:'t1',type:'function',function:{name:'echo_local',arguments:'{"text":"ok"}'}}]}};
      onDelta?.('private ','private ');
      onDelta?.('scratchpad','private scratchpad');
      return{message:{content:'final answer'}};
    }
  });
  const result=await runtime.run({prompt:'work then answer'});
  assert.equal(result.text,'final answer');
  assert.equal(round,2);
});

test('offline reference Agent preserves the local-host ready fallback',async()=>{
  const {createLocalAnysphereRuntime}=require('../electron/grok-reference-anysphere-runtime.cjs');
  const runtime=createLocalAnysphereRuntime({
    conversationId:'offline',
    modelId:'test-model',
    systemPrompt:()=> 'Offline test.',
    getTools:()=>[],
    executeTool:async()=>null,
    transport:async()=>({offline:true})
  });
  const result=await runtime.run({prompt:'hello'});
  assert.match(result.text,/Agent host is ready on this Mac/);
});


test('private provider deltas never become visible AnysphereAgent output',async()=>{
  const {createLocalAnysphereRuntime}=require('../electron/grok-reference-anysphere-runtime.cjs');
  const runtime=createLocalAnysphereRuntime({
    conversationId:'privacy-boundary',
    modelId:'test-model',
    systemPrompt:()=> 'Return only the final answer.',
    getTools:()=>[],
    executeTool:async()=>{throw Error('unexpected tool')},
    transport:async(_messages,_tools,_signal,onDelta)=>{
      onDelta?.('private ','private ');
      onDelta?.('scratchpad','private scratchpad');
      return{message:{content:'final answer'},usage:{inputTokens:4,outputTokens:2}};
    }
  });
  const result=await runtime.run({prompt:'answer'});
  assert.equal(result.text,'final answer');
  assert.doesNotMatch(result.text,/private scratchpad/);
});

test('offline AnysphereAgent keeps the local-host readiness contract',async()=>{
  const {createLocalAnysphereRuntime}=require('../electron/grok-reference-anysphere-runtime.cjs');
  const runtime=createLocalAnysphereRuntime({
    conversationId:'offline-boundary',
    modelId:'test-model',
    systemPrompt:()=> 'Local host.',
    getTools:()=>[],
    executeTool:async()=>{throw Error('unexpected tool')},
    transport:async()=>({offline:true})
  });
  const result=await runtime.run({prompt:'status'});
  assert.match(result.text,/Agent host is ready on this Mac/);
  assert.match(result.text,/(inference is not configured|FABUSHI_AGENT_API_URL)/);
});
