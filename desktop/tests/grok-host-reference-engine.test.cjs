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

test('legacy environment override cannot re-enable the removed tool loop',async()=>{
  const before=process.env.FABUSHI_AGENT_ENGINE;
  process.env.FABUSHI_AGENT_ENGINE='legacy';
  try{
    const observations=[],host=makeHost(observations);
    const agent={id:'host-ref-forced-legacy',name:'Chief'};
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
    assert.equal(observations.at(-1)?.engine,'grok-anysphere-agent');
    assert.equal(typeof agent.referenceAgentStateBase64,'string');
    assert.ok(agent.referenceAgentStateBase64.length>16);
  }finally{
    if(before===undefined)delete process.env.FABUSHI_AGENT_ENGINE;
    else process.env.FABUSHI_AGENT_ENGINE=before;
  }
});

test('production host source contains no legacy agent loop selector',()=>{
  const source=require('node:fs').readFileSync(require('node:path').resolve(__dirname,'../electron/grok-host-runtime.cjs'),'utf8');
  assert.equal(source.includes('FABUSHI_AGENT_ENGINE'),false);
  assert.equal(source.includes('legacy-tool-loop'),false);
  assert.equal(source.includes('Agent exceeded the tool-call round limit'),false);
});


test('production reference Agent can authenticate inference with the signed-in Fabushi account',async()=>{
  const beforeUrl=process.env.FABUSHI_ACCOUNT_INFERENCE_URL;
  const beforeKey=process.env.FABUSHI_AGENT_API_KEY;
  const beforeStream=process.env.FABUSHI_AGENT_STREAM;
  process.env.FABUSHI_ACCOUNT_INFERENCE_URL='https://inference.example.test/v1/chat/completions';
  process.env.FABUSHI_AGENT_STREAM='false';
  delete process.env.FABUSHI_AGENT_API_KEY;
  try{
    const calls=[];
    const host=createHostRuntime({
      shell:null,
      getLocalToolPermission:async()=> 'always',
      getInferenceAccessToken:async()=> 'account-access-token',
      fetchImpl:async(url,options)=>{
        calls.push({url:String(url),authorization:options.headers.authorization,body:JSON.parse(options.body)});
        return new Response(JSON.stringify({choices:[{message:{content:'account-auth-ok'}}],usage:{prompt_tokens:2,completion_tokens:1}}),{status:200,headers:{'content-type':'application/json'}});
      },
      requestApproval:async()=>true,
      onToolState:async()=>{},
      onAgentStatus:async()=>{},
      onAssistantDelta:async()=>{}
    });
    const agent={id:'host-account-1',name:'Chief'};
    const user={id:'u1',role:'user',text:'hello',createdAt:1,status:'done'};
    const result=await host.runTurn({
      agent,history:[user],transcript:[user],enabled:new Set(),
      signal:new AbortController().signal,
      assistantEntry:{id:'a1',role:'assistant',text:'',createdAt:2,status:'streaming'}
    });
    assert.equal(result,'account-auth-ok');
    assert.equal(calls.length,1);
    assert.equal(calls[0].authorization,'Bearer account-access-token');
    assert.equal(calls[0].url,process.env.FABUSHI_ACCOUNT_INFERENCE_URL);
  }finally{
    if(beforeUrl===undefined)delete process.env.FABUSHI_ACCOUNT_INFERENCE_URL; else process.env.FABUSHI_ACCOUNT_INFERENCE_URL=beforeUrl;
    if(beforeKey===undefined)delete process.env.FABUSHI_AGENT_API_KEY; else process.env.FABUSHI_AGENT_API_KEY=beforeKey;
    if(beforeStream===undefined)delete process.env.FABUSHI_AGENT_STREAM; else process.env.FABUSHI_AGENT_STREAM=beforeStream;
  }
});
