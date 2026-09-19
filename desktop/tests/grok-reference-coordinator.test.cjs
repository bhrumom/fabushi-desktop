'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const {createReferenceCoordinator}=require('../electron/grok-reference-coordinator.cjs');

function fakeRuntime(){
  const calls=[];
  const agents=[{id:'a1',name:'Chief',status:'running',pinned:true,hidden:false,unread:true,updatedAt:2},{id:'a2',name:'Builder',status:'idle',pinned:false,hidden:true,unread:false,updatedAt:1}];
  return{
    calls,agents,
    async listAgents(){return agents},
    async getThread({agentId}){assert.equal(agentId,'a1');return{agent:agents[0],messages:[
      {id:'m1',role:'user',text:'hello',createdAt:10,status:'done'},
      {id:'m2',role:'assistant',text:'world',createdAt:11,status:'streaming'},
      {id:'m3',role:'tool',toolName:'terminal',text:'pwd',createdAt:12,status:'done'}
    ],outline:[]}},
    async registerAttachment({path}){calls.push(['registerAttachment',path]);return{id:'att-'+calls.length}},
    async sendMessage(input){calls.push(['sendMessage',input]);return{messageId:'m4'}},
    async createAgent(input){calls.push(['createAgent',input]);return{id:'a3',name:input.name,status:'idle',pinned:false,hidden:false,unread:false,updatedAt:3}},
    async updateAgent(input){calls.push(['updateAgent',input]);return agents[0]},
    async deleteAgent(input){calls.push(['deleteAgent',input]);return{ok:true}},
    async duplicateAgent(input){calls.push(['duplicateAgent',input]);return agents[0]},
    async setGroupMembers(){return agents[0]},
    async setAgentUnread(input){calls.push(['setAgentUnread',input])},
    async setAgentHidden(input){calls.push(['setAgentHidden',input])},
    async setAgentNotifyOnUpdates(input){calls.push(['setAgentNotifyOnUpdates',input])},
    async setAgentAvatarBytes(){return agents[0]},
    async getAsyncTasks(){return[]},async searchMedia(){return[]},async listWorkflows(){return[]},
    async getAgentAutomations(){return[]},async getAgentChannels(){return[]},
    async respondToWidget(){return null},async dismissWidget(){return{}},async submitSecret(){},async reactToMessage(){},async resolveApproval(){},
    async getRuntimeSettings(){return{}}
  };
}

test('reference adapter projects Fabushi local agents into Grok roster fields',async()=>{
  const runtime=fakeRuntime(),bridge=createReferenceCoordinator(runtime);
  const rows=await bridge.call('listAgents');
  assert.equal(rows.length,2);
  assert.equal(rows[0].isPinned,true);
  assert.equal(rows[0].hasUnread,true);
  assert.equal(rows[0].isRunning,true);
  assert.equal(rows[1].isHiddenFromSidebar,true);
});

test('reference transcript replies satisfy Grok page and window contracts',async()=>{
  const bridge=createReferenceCoordinator(fakeRuntime());
  const tail=await bridge.call('openAgentTail',{id:'a1',limit:200});
  assert.equal(tail.entries.length,3);
  assert.equal(tail.entries[0].timestampMs,10);
  assert.equal(tail.entries[1].isStreaming,true);
  assert.equal(tail.entries[2].kind,'tool-call');
  const window=await bridge.call('getAgentTranscriptWindow',{id:'a1',limit:200});
  assert.deepEqual(window.threadCounts,{});
  assert.ok(Array.isArray(window.entries));
  const box=await bridge.call('getForeverBoxStatus',{id:'a1'});
  assert.equal(box.agentId,'a1');
  assert.equal(box.computerTarget,'local-mac');
});

test('sendPrompt stays native and registers committed local attachments',async()=>{
  const runtime=fakeRuntime(),bridge=createReferenceCoordinator(runtime);
  const result=await bridge.call('sendPrompt',{agentId:'a1',prompt:'ship it',attachmentPaths:['/tmp/a.txt'],clientNonce:'n1'});
  assert.equal(result.accepted,true);
  assert.deepEqual(runtime.calls[0],['registerAttachment','/tmp/a.txt']);
  assert.equal(runtime.calls[1][0],'sendMessage');
  assert.equal(runtime.calls[1][1].agentId,'a1');
  assert.equal(runtime.calls[1][1].text,'ship it');
  assert.deepEqual(runtime.calls[1][1].attachmentIds,['att-1']);
});
