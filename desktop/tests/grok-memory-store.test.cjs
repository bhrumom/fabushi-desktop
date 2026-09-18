'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs/promises');
const os=require('node:os');
const path=require('node:path');
const {createMemoryStore}=require('../electron/grok-memory-store.cjs');

test('durable memory separates agent, user, and project scopes and injects bounded context',async t=>{
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-memory-'));
  t.after(()=>fs.rm(root,{recursive:true,force:true}));
  const store=createMemoryStore({app:{getPath:()=>root}});
  await store.write({agentId:'chief',content:'Prefer concise progress updates.',tier:'profile',scope:'agent'});
  await store.write({agentId:'chief',content:'The project uses protected main.',tier:'log',scope:'project',project:'grok-parity'});
  await store.write({agentId:'chief',content:'User prefers objective evidence.',tier:'profile',scope:'user'});
  assert.equal((await store.list({agentId:'chief',scope:'agent'}))[0].content,'Prefer concise progress updates.');
  assert.equal((await store.list({agentId:'chief',scope:'project',project:'grok-parity'}))[0].content,'The project uses protected main.');
  const context=await store.context('chief');
  assert.match(context,/objective evidence/);
  assert.match(context,/concise progress updates/);
  assert.doesNotMatch(context,/protected main/);
});

test('durable memory forget requires the exact recorded fact',async t=>{
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-memory-'));
  t.after(()=>fs.rm(root,{recursive:true,force:true}));
  const store=createMemoryStore({app:{getPath:()=>root}});
  await store.write({agentId:'chief',content:'Remember this exact sentence.',scope:'agent'});
  const miss=await store.forget({agentId:'chief',content:'Remember this sentence.',scope:'agent'});
  assert.equal(miss.ok,false);
  const hit=await store.forget({agentId:'chief',content:'Remember this exact sentence.',scope:'agent'});
  assert.equal(hit.ok,true);
  assert.deepEqual(await store.list({agentId:'chief',scope:'agent'}),[]);
});
