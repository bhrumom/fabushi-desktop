'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');

function manager(){
  return{
    async getSharedRoomIdForAgent(){return null},
    async listRoomAgentIds(){return[]},
    async markMirrorRoomRevoked(){},
    async getAgentDisplayProfile(id){return{id,name:'Chief',description:''}},
    async getAgentAvatar(){return null},
    async restampRoomEntry(){},
    async appendSharedRoomActivityNotice(){},
    async runRemoteRequestedMemberTurn(){return[]},
    async appendMirrorRoomEntry(){return true},
    async postSharedRoomGuestMessage(){},
    async ensureMirrorRoom(){},
    async ensureHostedSharedRoom(){return null},
    async findRoomAgentId(){return null},
    async isAgentCapReached(){return false}
  };
}

test('reference cross-user sharing stays disabled without an explicit backend',async()=>{
  const {createLocalXuserSharingRuntime}=require('../electron/grok-reference-xuser-sharing.cjs');
  let fetched=false;
  const runtime=createLocalXuserSharingRuntime({
    backendUrl:null,
    getAccessToken:async()=>{throw Error('unexpected token read')},
    getSelfAuthId:async()=> 'user-1',
    manager:manager(),
    emitSharing:()=>{},
    fetchImpl:async()=>{fetched=true;throw Error('unexpected fetch')}
  });
  const state=await runtime.getState();
  assert.equal(state.isEnabled,false);
  assert.equal(fetched,false);
  const result=await runtime.createRoomFromAgent('agent-1');
  assert.equal(result.status,'error');
  assert.match(result.message,/not configured/i);
  runtime.dispose();
});

test('reference cross-user sharing uses the exact Sand relay with Fabushi account bearer token',async()=>{
  const {createLocalXuserSharingRuntime}=require('../electron/grok-reference-xuser-sharing.cjs');
  const calls=[];
  const fetchImpl=async(url,options={})=>{
    const path=new URL(String(url)).pathname;
    calls.push({path,authorization:options.headers?.authorization,body:options.body?JSON.parse(options.body):null});
    if(path==='/sand/share-state')return new Response(JSON.stringify({pendingJoinRequests:[],rooms:[]}),{status:200,headers:{'content-type':'application/json'}});
    if(path==='/sand/xuser/poll')return new Response(JSON.stringify({events:[]}),{status:200,headers:{'content-type':'application/json'}});
    if(path==='/sand/share-rooms/from-agent')return new Response(JSON.stringify({shareUrl:'https://share.example/r/1',expiresAtMs:123456,room:{roomId:'room-1'}}),{status:200,headers:{'content-type':'application/json'}});
    throw Error('unexpected sharing path '+path);
  };
  const runtime=createLocalXuserSharingRuntime({
    backendUrl:'https://sharing.example/api/',
    getAccessToken:async()=> 'fabushi-account-token',
    getSelfAuthId:async()=> 'auth-user-1',
    manager:manager(),
    emitSharing:()=>{},
    fetchImpl
  });
  const state=await runtime.getState();
  assert.equal(state.isEnabled,true);
  const result=await runtime.createRoomFromAgent('agent-1');
  assert.equal(result.status,'ok');
  assert.equal(result.roomId,'room-1');
  assert.ok(calls.some(call=>call.path==='/sand/share-state'));
  const create=calls.find(call=>call.path==='/sand/share-rooms/from-agent');
  assert.equal(create.authorization,'Bearer fabushi-account-token');
  assert.equal(create.body.agentId,'agent-1');
  assert.equal(create.body.agentName,'Chief');
  runtime.dispose();
});
