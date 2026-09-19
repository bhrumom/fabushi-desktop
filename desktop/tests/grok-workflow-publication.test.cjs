'use strict';
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),fsp=require('node:fs/promises'),path=require('node:path'),os=require('node:os');
const {createWorkflowManager}=require('../electron/grok-workflow-manager.cjs');
const {createReferenceCoordinator}=require('../electron/grok-reference-coordinator.cjs');

test('workflow publish lifecycle is persistent and reversible',async()=>{
  const root=await fsp.mkdtemp(path.join(os.tmpdir(),'fabushi-workflow-')),manager=createWorkflowManager({app:{getPath:()=>root}});
  const created=await manager.saveWorkflow({name:'Deploy helper',description:'Deploys locally',body:'# Deploy\nRun checks.',sourceRef:'https://example.com/SKILL.md'});
  assert.equal((await manager.getPublishTargets()).teams[0].teamId,'fabushi-local');
  const published=await manager.publishSkill({id:created.id,teamId:'fabushi-local'});
  assert.equal(published.source,'plugin');assert.equal(published.publishedByCurrentUser,true);assert.equal(published.sourceRef,'https://example.com/SKILL.md');
  const restarted=createWorkflowManager({app:{getPath:()=>root}});
  assert.equal((await restarted.list()).find(row=>row.id===created.id).source,'plugin');
  assert.equal((await manager.resyncPublishedSkill({id:created.id})).source,'plugin');
  assert.equal((await manager.unpublishSkill({id:created.id})).source,'workflow');
  await fsp.rm(root,{recursive:true,force:true});
});

test('URL skill import performs bounded fetch and preserves source URL',async()=>{
  const saved=[],old=global.fetch;global.fetch=async()=>({ok:true,status:200,text:async()=> '# Imported\nUse the local Mac.'});
  try{
    const c=createReferenceCoordinator({saveWorkflow:async spec=>{saved.push(spec);return{id:'w1',...spec}}});
    const result=await c.call('importAgentWorkflowUrl',{url:'https://example.com/my-skill.md'});
    assert.equal(result.failed.length,0);assert.equal(saved[0].sourceRef,'https://example.com/my-skill.md');assert.match(saved[0].body,/local Mac/);
  }finally{global.fetch=old}
});

test('publish and listener coordinator seams are executable, not placeholders',async()=>{
  const runtime={
    getSkillPublishTargets:async()=>({teams:[{teamId:'fabushi-local',name:'Fabushi Local Workspace'}]}),
    publishSkill:async()=>({source:'plugin'}),resyncPublishedSkill:async()=>({source:'plugin'}),unpublishSkill:async()=>({source:'workflow'}),
    getListenerIntegrations:async()=>({integrations:[{platform:'github',isConnected:true},{platform:'slack',isConnected:false}]}),
    getListenerConnectUrl:async input=>({url:'https://example.com/connect/'+input.platform})
  };
  const c=createReferenceCoordinator(runtime);
  assert.equal((await c.call('getSkillPublishTargets')).teams.length,1);
  assert.equal((await c.call('publishSkill',{workflowId:'w1',teamId:'fabushi-local'})).source,'plugin');
  assert.equal((await c.call('resyncPublishedSkill',{workflowId:'w1'})).source,'plugin');
  assert.equal((await c.call('unpublishSkill',{workflowId:'w1'})).source,'workflow');
  assert.equal((await c.call('getListenerIntegrations')).integrations.length,2);
  assert.match((await c.call('getListenerConnectUrl',{platform:'github'})).url,/github$/);
  const source=fs.readFileSync(path.resolve(__dirname,'../electron/grok-reference-coordinator.cjs'),'utf8');
  assert.equal(source.includes('URL import is not configured'),false);
  assert.equal(source.includes("status:'local-only'"),false);
  assert.equal(source.includes("case'getSkillPublishTargets':return{targets:[]}"),false);
});
