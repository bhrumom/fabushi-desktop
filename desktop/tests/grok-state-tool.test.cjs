'use strict';
const test=require('node:test');const assert=require('node:assert/strict');
const {executeStateTool,definition}=require('../electron/grok-state-tool.cjs');
test('update_state exposes durable state routes',()=>assert.equal(definition.function.name,'update_state'));
test('state tool forwards supported mutations',async()=>{let seen=null;const result=await executeStateTool({target:'memory',action:'write',fact:'Use concise updates.'},{updateState:async input=>{seen=input;return{ok:true,detail:'saved'}}});assert.equal(seen.fact,'Use concise updates.');assert.equal(result.text,'saved')});
test('state tool rejects unsupported routes',async()=>assert.rejects(()=>executeStateTool({target:'channel',action:'disconnect'},{updateState:async()=>({ok:true})}),/Unsupported state route/));
