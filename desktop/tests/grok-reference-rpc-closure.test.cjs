'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');

const root=path.join(__dirname,'..','..');
const renderer=fs.readFileSync(path.join(__dirname,'..','src','production','ProductionRenderer.tsx'),'utf8');
const coordinator=fs.readFileSync(path.join(__dirname,'..','electron','grok-reference-coordinator.cjs'),'utf8');
const preload=fs.readFileSync(path.join(__dirname,'..','electron','preload.cjs'),'utf8');
const closure=JSON.parse(fs.readFileSync(path.join(root,'reference','grok-bot-0.18','manifests','reconstruction','renderer-closure.json'),'utf8'));

test('every coordinator RPC invoked by the exact ProductionRenderer has a local implementation',()=>{
  const calls=[...renderer.matchAll(/client\.call\(\s*["'`]([^"'`]+)["'`]/g)].map(match=>match[1]);
  const referenced=[...new Set(calls)].sort();
  const implemented=new Set([...coordinator.matchAll(/case'([^']+)'/g)].map(match=>match[1]));
  const missing=referenced.filter(method=>!implemented.has(method));
  assert.ok(referenced.length>=40,'expected the recovered ProductionRenderer RPC surface');
  assert.deepEqual(missing,[]);
});

test('all coordinator calls evidenced by the pinned official renderer closure are implemented',()=>{
  const claimed=[...new Set(closure.ipcClaims.filter(row=>row.kind==='coordinator-call').map(row=>row.value))].sort();
  const implemented=new Set([...coordinator.matchAll(/case'([^']+)'/g)].map(match=>match[1]));
  assert.equal(claimed.length,58);
  assert.deepEqual(claimed.filter(method=>!implemented.has(method)),[]);
});

test('all official coordinator subscription families have a local event path',()=>{
  const claimed=[...new Set(closure.ipcClaims.filter(row=>row.kind==='coordinator-subscribe').map(row=>row.value))].sort();
  const emitted=new Set([...preload.matchAll(/emitFamily\('([^']+)'/g)].map(match=>match[1]));
  if(preload.includes("publishTrayEvent({type:")||preload.includes("emitFamily('tray'"))emitted.add('tray');
  assert.deepEqual(claimed,[
    'agent-upserted','agents','async-tasks','automations','box-disk-pressure',
    'computer-action','forever-box','sharing','subagents','tray'
  ]);
  assert.deepEqual(claimed.filter(family=>!emitted.has(family)),[]);
});

test('reference coordinator fails closed for an unknown RPC rather than fabricating success',()=>{
  assert.match(coordinator,/Unsupported Grok coordinator method/);
});
