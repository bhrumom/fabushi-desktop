'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const {TrayManager,MAX_TRAYS}=require('../electron/grok-reference-trays.cjs');
const {createReferenceCoordinator}=require('../electron/grok-reference-coordinator.cjs');

test('pinned Grok TrayManager preserves dedupe cap dismiss and clear semantics',()=>{
  let id=0,now=100;
  const trays=new TrayManager(()=>String(++id),()=>++now);
  const first=trays.pushError({title:'Network',detail:'one',dedupeKey:'same'});
  const second=trays.pushError({title:'Network',detail:'two',dedupeKey:'same'});
  assert.equal(first.id,second.id);assert.equal(second.count,2);assert.equal(trays.getTrays().length,1);
  for(let i=0;i<MAX_TRAYS+5;i++)trays.pushError({title:'E'+i,detail:'D'+i});
  assert.equal(trays.getTrays().length,MAX_TRAYS);
  const target=trays.getTrays().at(-1).id;assert.equal(trays.dismiss(target),true);assert.equal(trays.getTrays().some(x=>x.id===target),false);
  trays.clearAll();assert.equal(trays.getTrays().length,0);
});

test('coordinator tray RPC surfaces real request errors and supports dismiss clear',async()=>{
  const c=createReferenceCoordinator({getListenerConnectUrl:async()=>{throw Error('listener unavailable')}});
  await assert.rejects(()=>c.call('getListenerConnectUrl',{platform:'github'}),/listener unavailable/);
  const rows=await c.call('getTrays');assert.equal(rows.length,1);assert.equal(rows[0].kind,'error');assert.match(rows[0].detail,/listener unavailable/);
  await c.call('dismissTray',{id:rows[0].id});assert.equal((await c.call('getTrays')).length,0);
  await assert.rejects(()=>c.call('getListenerConnectUrl',{platform:'github'}));
  await c.call('clearTrays');assert.equal((await c.call('getTrays')).length,0);
});
