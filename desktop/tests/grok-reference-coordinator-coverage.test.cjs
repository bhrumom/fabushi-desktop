'use strict';

const fs=require('node:fs');
const path=require('node:path');
const test=require('node:test');
const assert=require('node:assert/strict');
const {createReferenceCoordinator}=require('../electron/grok-reference-coordinator.cjs');

function referenceMethods(){
  const file=path.resolve(__dirname,'../../reference/grok-bot-0.18/source/shared/rpc/coordinator.ts');
  const source=fs.readFileSync(file,'utf8');
  const start=source.indexOf('export const COORDINATOR_METHOD_TABLE = {');
  const end=source.indexOf('} as const;',start);
  return[...source.slice(start,end).matchAll(/^\s{2}([A-Za-z0-9]+):\s*\{/gm)].map(match=>match[1]);
}
function adapterMethods(){
  const source=fs.readFileSync(path.resolve(__dirname,'../electron/grok-reference-coordinator.cjs'),'utf8');
  return[...new Set([...source.matchAll(/case'([^']+)'/g)].map(match=>match[1]))];
}

test('reference coordinator adapter covers every pinned Grok RPC method',()=>{
  const expected=referenceMethods().sort(),actual=adapterMethods().sort();
  assert.equal(expected.length,89);
  assert.deepEqual(actual,expected);
});

test('routed MCP RPC delegates to real runtime collect and execute',async()=>{
  const calls=[];
  const coordinator=createReferenceCoordinator({
    listRoutedMcpTools:async()=>[{type:'function',function:{name:'mcp__demo__echo',parameters:{type:'object'}}}],
    executeRoutedMcpTool:async input=>{calls.push(input);return{text:'ok'}}
  });
  const tools=await coordinator.call('listRoutedMcpTools');
  assert.equal(tools[0].function.name,'mcp__demo__echo');
  assert.deepEqual(await coordinator.call('executeRoutedMcpTool',{name:'mcp__demo__echo',args:{value:1}}),{text:'ok'});
  assert.deepEqual(calls,[{name:'mcp__demo__echo',args:{value:1}}]);
});

test('shared-room RPC methods delegate complete state transitions',async()=>{
  const state={isEnabled:true,selfAuthId:'local-user',pendingJoinRequests:[],rooms:[],typingUsers:[]};
  const runtime={
    getSharingState:async()=>state,
    createRoomFromAgent:async input=>({status:'ok',roomId:'r1',agentId:input.agentId}),
    createRoomInvite:async()=>({status:'ok',shareUrl:'fabushi://shared-room/join?roomId=r1',expiresAtMs:Date.now()+1000,roomId:'r1'}),
    joinSharedRoom:async()=>({status:'ok',roomId:'r1'}),
    respondToRoomJoinRequest:async()=>state,createSharedRoom:async()=>({status:'ok',roomId:'r1',agentId:'g1'}),
    addOwnAgentToSharedRoom:async()=>state,removeOwnAgentFromSharedRoom:async()=>state,setSharedRoomTyping:async()=>undefined,leaveSharedRoom:async()=>state
  };
  const coordinator=createReferenceCoordinator(runtime);
  assert.equal((await coordinator.call('getSharingState')).isEnabled,true);
  assert.equal((await coordinator.call('createRoomFromAgent',{agentId:'a1'})).roomId,'r1');
  assert.equal((await coordinator.call('createRoomInvite',{roomId:'r1'})).status,'ok');
  assert.equal((await coordinator.call('joinSharedRoom',{link:'fabushi://shared-room/join?roomId=r1'})).status,'ok');
  assert.equal((await coordinator.call('createSharedRoom',{agents:[{id:'a1'}]})).agentId,'g1');
  assert.equal((await coordinator.call('addOwnAgentToSharedRoom',{roomId:'r1',agentId:'a1'})).selfAuthId,'local-user');
  assert.equal((await coordinator.call('removeOwnAgentFromSharedRoom',{roomId:'r1',agentId:'a1'})).selfAuthId,'local-user');
  assert.equal((await coordinator.call('leaveSharedRoom',{roomId:'r1'})).selfAuthId,'local-user');
});
