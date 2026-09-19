'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const {createHostRuntime}=require('../electron/grok-host-runtime.cjs');

function makeHost(observations){
  return createHostRuntime({
    shell:null,
    getLocalToolPermission:async()=> 'always',
    requestApproval:async()=>true,
    onToolState:async()=>{},
    onAgentStatus:async()=>{},
    onAssistantDelta:async()=>{},
    onTurnUsage:async payload=>observations.push(payload),
    inferenceRequest:async()=>({
      message:{content:'reference-host-ok'},
      usage:{inputTokens:4,outputTokens:2}
    })
  });
}

test('production Host defaults to exact AnysphereAgent runtime',async()=>{
  const before=process.env.FABUSHI_AGENT_ENGINE;
  delete process.env.FABUSHI_AGENT_ENGINE;
  try{
    const observations=[],host=makeHost(observations);
    const agent={id:'host-ref-1',name:'Chief'};
    const user={id:'u1',role:'user',text:'hello',createdAt:1,status:'done'};
    const transcript=[user],assistant={id:'a1',role:'assistant',text:'',createdAt:2,status:'streaming'};
    const value=await host.runTurn({
      agent,
      history:[user],
      transcript,
      enabled:new Set(),
      signal:new AbortController().signal,
      assistantEntry:assistant
    });
    assert.equal(value,'reference-host-ok');
    assert.equal(observations.at(-1)?.engine,'grok-anysphere-agent');
    assert.equal(typeof agent.referenceAgentStateBase64,'string');
    assert.ok(agent.referenceAgentStateBase64.length>16);
  }finally{
    if(before===undefined)delete process.env.FABUSHI_AGENT_ENGINE;
    else process.env.FABUSHI_AGENT_ENGINE=before;
  }
});

test('legacy tool loop is only selected by explicit environment override',async()=>{
  const before=process.env.FABUSHI_AGENT_ENGINE;
  process.env.FABUSHI_AGENT_ENGINE='legacy';
  try{
    const observations=[],host=makeHost(observations);
    const agent={id:'host-legacy-1',name:'Chief'};
    const user={id:'u1',role:'user',text:'hello',createdAt:1,status:'done'};
    const value=await host.runTurn({
      agent,
      history:[user],
      transcript:[user],
      enabled:new Set(),
      signal:new AbortController().signal,
      assistantEntry:{id:'a1',role:'assistant',text:'',createdAt:2,status:'streaming'}
    });
    assert.equal(value,'reference-host-ok');
    assert.equal(observations.at(-1)?.engine,'legacy-tool-loop');
    assert.equal(agent.referenceAgentStateBase64,undefined);
  }finally{
    if(before===undefined)delete process.env.FABUSHI_AGENT_ENGINE;
    else process.env.FABUSHI_AGENT_ENGINE=before;
  }
});
