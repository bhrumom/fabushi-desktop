'use strict';
const test=require('node:test');const assert=require('node:assert/strict');
const {executeStateTool,definition,TARGETS}=require('../electron/grok-state-tool.cjs');
test('update_state exposes pinned Grok durable-state targets',()=>{
  assert.equal(definition.function.name,'update_state');
  assert.deepEqual(TARGETS,['memory','routine','workflow','profile','settings','channel','project','avatar']);
});
test('state tool forwards supported mutations',async()=>{
  let seen=null;const result=await executeStateTool({target:'memory',action:'write',fact:'Use concise updates.'},{updateState:async input=>{seen=input;return{ok:true,detail:'saved'}}});
  assert.equal(seen.fact,'Use concise updates.');assert.equal(result.text,'saved');
});
test('channel project and avatar routes are part of the Grok state protocol',async()=>{
  const seen=[];for(const input of [{target:'channel',action:'disconnect',platform:'slack'},{target:'project',action:'join',project:'docs'},{target:'avatar',action:'clear'}]){
    await executeStateTool(input,{updateState:async value=>{seen.push(value);return{ok:true,detail:'ok'}}});
  }
  assert.deepEqual(seen.map(row=>row.target),['channel','project','avatar']);
});
test('routine rejects ambiguous schedule plus trigger payloads',async()=>assert.rejects(
  ()=>executeStateTool({target:'routine',action:'create',name:'watch',prompt:'watch',schedule:'@daily',trigger:{type:'github'}},{updateState:async()=>({ok:true})}),
  /either schedule or trigger/
));
