'use strict';
const test=require('node:test');const assert=require('node:assert/strict');
const {createAgentRunner}=require('../electron/grok-agent-runner.cjs');

test('runner owns lifecycle generation and turn result',async()=>{
 const events=[];const runner=createAgentRunner({agentId:'a1',runTurn:async input=>'done:'+input.generation,onLifecycle:event=>events.push(event)});
 const result=await runner.run({});assert.equal(result.value,'done:0');assert.equal(result.generation,0);assert.equal(result.engine,'grok-sand-agent-runner');
 assert.equal(events[0].type,'started');assert.equal(events[1].type,'ended');assert.equal(runner.snapshot().active,null);assert.equal(runner.snapshot().engine,'grok-sand-agent-runner');
 assert.equal(runner.bumpGeneration(),1);assert.equal(runner.snapshot().generation,1);await runner.dispose();
});
test('runner interrupt aborts the active turn and quiesce blocks dispatch',async()=>{
 let observed=false;
 const runner=createAgentRunner({agentId:'a2',runTurn:({signal})=>new Promise((resolve,reject)=>{
   if(signal.aborted){observed=true;reject(Object.assign(Error('aborted'),{name:'AbortError'}));return}
   signal.addEventListener('abort',()=>{observed=true;reject(Object.assign(Error('aborted'),{name:'AbortError'}))},{once:true});
 })});
 const pending=runner.run({});await new Promise(resolve=>setTimeout(resolve,0));assert.equal(runner.interrupt('test'),true);
 await assert.rejects(()=>pending,/aborted/);assert.equal(observed,true);
 runner.requestQuiesceForUpgrade();const blocked=await runner.run({});assert.equal(blocked.quiescedForUpgrade,true);
 runner.cancelQuiesceForUpgrade();assert.equal(runner.isQuiescingForUpgrade(),false);await runner.dispose();
});
test('runner propagates external cancellation',async()=>{
 const external=new AbortController();
 const runner=createAgentRunner({agentId:'a3',runTurn:({signal})=>new Promise((_resolve,reject)=>signal.addEventListener('abort',()=>reject(Object.assign(Error('cancelled'),{name:'AbortError'})),{once:true}))});
 const pending=runner.run({signal:external.signal});external.abort();await assert.rejects(()=>pending,/cancelled/);await runner.dispose();
});
