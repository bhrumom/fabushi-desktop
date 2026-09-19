'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');

const renderer=fs.readFileSync(path.join(__dirname,'..','src','production','ProductionRenderer.tsx'),'utf8');
const coordinator=fs.readFileSync(path.join(__dirname,'..','electron','grok-reference-coordinator.cjs'),'utf8');

test('every coordinator RPC invoked by the exact ProductionRenderer has a local implementation',()=>{
  const calls=[...renderer.matchAll(/client\.call\(\s*["'`]([^"'`]+)["'`]/g)].map(match=>match[1]);
  const referenced=[...new Set(calls)].sort();
  const implemented=new Set([...coordinator.matchAll(/case'([^']+)'/g)].map(match=>match[1]));
  const missing=referenced.filter(method=>!implemented.has(method));
  assert.ok(referenced.length>=40,'expected the recovered ProductionRenderer RPC surface');
  assert.deepEqual(missing,[]);
});

test('reference coordinator fails closed for an unknown RPC rather than fabricating success',()=>{
  assert.match(coordinator,/Unsupported Grok coordinator method/);
});
