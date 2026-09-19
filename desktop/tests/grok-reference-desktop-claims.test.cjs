'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');

const repo=path.join(__dirname,'..','..');
const closure=JSON.parse(fs.readFileSync(path.join(repo,'reference','grok-bot-0.18','manifests','reconstruction','renderer-closure.json'),'utf8'));
const classified=JSON.parse(fs.readFileSync(path.join(repo,'projects','grok-bot-018-parity','management','desktop-bridge-claims.json'),'utf8'));
const preload=fs.readFileSync(path.join(__dirname,'..','electron','preload.cjs'),'utf8');

const key=row=>[row.cleanPath,row.line,row.value].join(':');

test('all 95 official desktop bridge claims are classified with no extras',()=>{
  const official=closure.ipcClaims.filter(row=>row.kind==='desktop-bridge').map(row=>({cleanPath:row.cleanPath,line:row.line,value:row.value}));
  assert.equal(official.length,95);
  assert.equal(classified.claimCount,95);
  assert.deepEqual([...official.map(key)].sort(),[...classified.claims.map(key)].sort());
  const allowed=new Set(['local-implemented','local-computer-adaptation','local-computer-product-difference','external-account-service']);
  assert.deepEqual([...new Set(classified.claims.map(row=>row.classification))].filter(value=>!allowed.has(value)),[]);
});

test('product differences are explicit and external services fail closed',()=>{
  assert.doesNotMatch(preload,/openCloudAgent:async\(\)=>\{\}/);
  assert.match(preload,/local-computer-only/);
  assert.doesNotMatch(preload,/invokeDashboardAction:async\(\)=>\(\{ok:true\}\)/);
  assert.match(preload,/Dashboard actions are unavailable in local Fabushi account mode/);
  assert.doesNotMatch(preload,/reportOnboardingStep[^\n]*=>\{\}/);
  assert.match(preload,/telemetry-report/);
});

test('classification exceptions are narrow and intentional',()=>{
  const external=classified.claims.filter(row=>row.classification==='external-account-service').map(row=>row.value).sort();
  assert.deepEqual(external,['cursorAccount.cancelTrial','cursorAccount.getUsageSummary','cursorAccount.invokeDashboardAction']);
  const adapted=classified.claims.filter(row=>row.classification==='local-computer-adaptation').map(row=>row.value).sort();
  assert.deepEqual(adapted,['foreverBox.forceRecreate','foreverBox.update']);
});
