'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const {createHostRuntime}=require('../electron/grok-host-runtime.cjs');
const {canonicalAutoReviewTarget,fingerprintAutoReviewTarget,normalizeAutoReviewMode}=require('../electron/grok-auto-review.cjs');

function makeBrowser(){
  return{
    definitions(){return[{type:'function',function:{name:'browser_navigate',description:'navigate',parameters:{type:'object',properties:{url:{type:'string'}},required:['url'],additionalProperties:false}}}]},
    isMutation(){return true},
    async execute({args}){return{text:'navigated '+args.url}}
  };
}
function makeInference(){
  let round=0;
  return async()=>{
    round++;
    if(round===1)return{message:{content:null,tool_calls:[{id:'nav-1',type:'function',function:{name:'browser_navigate',arguments:'{"url":"https://example.com"}'}}]}};
    return{message:{content:'finished'}};
  };
}
function baseRuntime(overrides={}){
  return createHostRuntime({
    shell:{openExternal:async()=>{}},
    getLocalToolPermission:async()=> 'always',
    getAutoReviewMode:async()=> 'enforce',
    requestApproval:async()=>true,
    onToolState:async()=>{},
    onAgentStatus:async()=>{},
    getWorkflowContext:async()=> '',
    browser:makeBrowser(),
    inferenceRequest:makeInference(),
    ...overrides
  });
}

test('auto-review canonical target is bounded and fingerprinted deterministically',()=>{
  const target=canonicalAutoReviewTarget('computer_click',{x:10,y:20,stateId:'state',purpose:'Open settings'});
  assert.equal(target.surface,'computer');
  assert.equal(target.displayStateIdentity,'state');
  assert.equal(fingerprintAutoReviewTarget(target),fingerprintAutoReviewTarget(target));
  assert.equal(normalizeAutoReviewMode('bogus'),'enforce');
});

test('enforce-mode classifier block requires explicit approval even when local mutations are otherwise always allowed',async()=>{
  let approvals=0;
  const transcript=[{id:'u1',role:'user',text:'navigate',createdAt:1,status:'done'}];
  const runtime=baseRuntime({
    autoReviewClassifier:async()=>({kind:'block',reason:'Review navigation target.'}),
    requestApproval:async({summary})=>{approvals++;assert.match(summary,/Review navigation target/);return true}
  });
  const text=await runtime.runTurn({agent:{id:'chief',name:'Chief'},history:transcript,transcript,enabled:new Set(['browser']),signal:new AbortController().signal});
  assert.equal(text,'finished');
  assert.equal(approvals,1);
  const tool=transcript.find(row=>row.role==='tool');
  assert.equal(tool.status,'done');
  assert.equal(tool.reviewMode,'enforce');
  assert.equal(tool.reviewDecision,'block');
  assert.equal(typeof tool.reviewFingerprint,'string');
});

test('enforce-mode classifier allow runs without a manual approval when local policy is always',async()=>{
  let approvals=0;
  const transcript=[{id:'u1',role:'user',text:'navigate',createdAt:1,status:'done'}];
  const runtime=baseRuntime({
    autoReviewClassifier:async()=>({kind:'allow',reason:'Routine reversible navigation.'}),
    requestApproval:async()=>{approvals++;return true}
  });
  const text=await runtime.runTurn({agent:{id:'chief',name:'Chief'},history:transcript,transcript,enabled:new Set(['browser']),signal:new AbortController().signal});
  assert.equal(text,'finished');
  assert.equal(approvals,0);
  assert.equal(transcript.find(row=>row.role==='tool').reviewDecision,'allow');
});
