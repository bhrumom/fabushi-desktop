'use strict';
const test=require('node:test');const assert=require('node:assert/strict');
const fs=require('node:fs/promises');const os=require('node:os');const path=require('node:path');
const {parseEnvOverrides,normalizeRemoteSnapshot,createExperimentsRuntime}=require('../electron/grok-experiments.cjs');

test('experiments parses environment gates and normalizes provider snapshot',()=>{
  assert.deepEqual(parseEnvOverrides('alpha=on,beta=false,ignored=maybe'),{alpha:true,beta:false});
  assert.deepEqual(normalizeRemoteSnapshot({featureGates:{a:true,b:'x'},experiments:{exp:{arm:'b'}},dynamicConfigs:{cfg:{model:'m'}}}),{
    featureGates:{a:true},experiments:{exp:{arm:'b'}},dynamicConfigs:{cfg:{model:'m'}}
  });
});

test('experiments uses authenticated provider, cache, overrides and model config without fabricated data',async()=>{
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-experiments-'));
  let persisted={local_gate:false},authSeen=null,writes=0;
  const runtime=createExperimentsRuntime({
    app:{getPath:()=>root,isPackaged:false},
    env:{FABUSHI_EXPERIMENTS_URL:'https://experiments.example.test/snapshot',SAND_FEATURE_GATE_OVERRIDES:'env_gate=true'},
    getAccessToken:async()=>'access-token',
    readOverrides:async()=>({...persisted}),
    writeOverrides:async next=>{persisted={...next};writes+=1},
    fetchImpl:async(_url,options)=>{authSeen=options.headers.authorization;return new Response(JSON.stringify({
      featureGates:{sand_computer_use_playwright:true,remote_gate:true},
      experiments:{agent_loop:{variant:'reference'}},
      dynamicConfigs:{sand_computer_use_playwright_config:{model:'computer-v1'},sand_browser_use_model:{model:'browser-v1'}}
    }),{status:200,headers:{'content-type':'application/json'}})}
  });
  try{
    await runtime.start();await runtime.refreshNow();
    const snapshot=runtime.getSnapshot();
    assert.equal(authSeen,'Bearer access-token');
    assert.equal(snapshot.featureGates.remote_gate,true);
    assert.equal(snapshot.featureGates.env_gate,true);
    assert.equal(snapshot.featureGates.local_gate,false);
    assert.deepEqual(runtime.getComputerUseModelOverride(),{model:'computer-v1'});
    assert.equal(runtime.hasLiveStatsigBootstrap(),true);
    await runtime.applyFeatureFlagOverrideCommand({kind:'set',name:'remote_gate',value:false});
    assert.equal(runtime.checkFeatureGate('remote_gate'),false);
    assert.equal(persisted.remote_gate,false);assert.equal(writes,1);
    const cached=JSON.parse(await fs.readFile(path.join(root,'grok-experiments-cache.json'),'utf8'));
    assert.equal(cached.snapshot.featureGates.remote_gate,true);
  }finally{await runtime.dispose();await fs.rm(root,{recursive:true,force:true})}
});

test('packaged experiments refuse local feature override mutation',async()=>{
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-experiments-packaged-'));
  const runtime=createExperimentsRuntime({app:{getPath:()=>root,isPackaged:true},env:{},readOverrides:async()=>({})});
  try{await runtime.start();await assert.rejects(()=>runtime.applyFeatureFlagOverrideCommand({kind:'set',name:'a',value:true}),/development builds/)}
  finally{await runtime.dispose();await fs.rm(root,{recursive:true,force:true})}
});
