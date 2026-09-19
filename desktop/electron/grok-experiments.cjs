'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');

const BASE_POLL_INTERVAL_MS=5*60_000;
const MIN_POLL_INTERVAL_MS=30_000;
function safeObject(value){return value&&typeof value==='object'&&!Array.isArray(value)?value:{}}
function parseEnvOverrides(raw){
  const result={};
  for(const pair of String(raw||'').split(',')){
    const [name,value]=pair.split('=',2).map(part=>String(part||'').trim());
    if(!name||!value)continue;
    if(['1','true','on'].includes(value.toLowerCase()))result[name]=true;
    else if(['0','false','off'].includes(value.toLowerCase()))result[name]=false;
  }
  return result;
}
function endpoint(value){
  const raw=String(value||'').trim();if(!raw)return null;
  const url=new URL(raw);
  const loopback=['127.0.0.1','localhost','::1','[::1]'].includes(url.hostname);
  if(url.protocol!=='https:'&&!(url.protocol==='http:'&&loopback))throw Error('Experiments endpoint must use HTTPS (HTTP is allowed only for loopback development).');
  return url.toString();
}
function normalizeRemoteSnapshot(value){
  const source=safeObject(value);
  return{
    featureGates:Object.fromEntries(Object.entries(safeObject(source.featureGates)).filter(([,v])=>typeof v==='boolean')),
    experiments:Object.fromEntries(Object.entries(safeObject(source.experiments)).map(([k,v])=>[k,safeObject(v)])),
    dynamicConfigs:Object.fromEntries(Object.entries(safeObject(source.dynamicConfigs)).map(([k,v])=>[k,safeObject(v)]))
  };
}
function createExperimentsRuntime({
  app,env=process.env,fetchImpl=globalThis.fetch,getAccessToken=async()=>null,
  readOverrides=async()=>({}),writeOverrides=async()=>{},onChanged=()=>{}
}){
  const remoteUrl=endpoint(env.FABUSHI_EXPERIMENTS_URL);
  const cacheFile=path.join(app.getPath('userData'),'grok-experiments-cache.json');
  const isDevBuild=app.isPackaged!==true;
  let remote={featureGates:{},experiments:{},dynamicConfigs:{}},overrides={},flagsFetchedAtMs,live=false,started=false,disposed=false,timer=null,loading=null;
  const listeners=new Set();
  function featureGates(){return{...remote.featureGates,...parseEnvOverrides(env.SAND_FEATURE_GATE_OVERRIDES),...overrides}}
  function snapshot(){
    const gates=featureGates();
    const items=Object.keys(gates).sort().map(name=>({name,value:gates[name],override:Object.prototype.hasOwnProperty.call(overrides,name)?overrides[name]:null}));
    return{isInitialized:started,featureGates:gates,experiments:{...remote.experiments},dynamicConfigs:{...remote.dynamicConfigs},...(isDevBuild?{featureFlags:{items,isLive:live}}:{})};
  }
  function emit(){const value=snapshot();try{onChanged(value)}catch{}for(const listener of listeners){try{listener(value)}catch{}}return value}
  async function load(){
    overrides=safeObject(await readOverrides());
    try{const cached=JSON.parse(await fs.readFile(cacheFile,'utf8'));remote=normalizeRemoteSnapshot(cached.snapshot);flagsFetchedAtMs=Number(cached.fetchedAtMs)||undefined}catch{}
  }
  async function persistRemote(){
    await fs.mkdir(path.dirname(cacheFile),{recursive:true});
    const tmp=cacheFile+'.tmp';
    await fs.writeFile(tmp,JSON.stringify({snapshot:remote,fetchedAtMs:flagsFetchedAtMs},null,2)+'\n',{mode:0o600});
    await fs.rename(tmp,cacheFile);
  }
  async function refreshNow(){
    if(disposed)return snapshot();
    if(!remoteUrl){live=false;return emit()}
    if(loading)return loading;
    loading=(async()=>{
      try{
        const token=await getAccessToken().catch(()=>null);
        const headers={accept:'application/json'};if(token)headers.authorization='Bearer '+token;
        const response=await fetchImpl(remoteUrl,{headers});
        if(!response.ok)throw Error('Experiments HTTP '+response.status);
        remote=normalizeRemoteSnapshot(await response.json());flagsFetchedAtMs=Date.now();live=true;await persistRemote();return emit();
      }catch(error){live=false;emit();throw error}
      finally{loading=null}
    })();
    return loading;
  }
  function schedule(){
    if(disposed||!remoteUrl)return;
    clearTimeout(timer);timer=setTimeout(()=>{void refreshNow().catch(()=>{}).finally(schedule)},Math.max(MIN_POLL_INTERVAL_MS,BASE_POLL_INTERVAL_MS));timer.unref?.();
  }
  async function start(){
    if(started)return snapshot();started=true;await load();emit();
    if(remoteUrl)void refreshNow().catch(()=>{}).finally(schedule);
    return snapshot();
  }
  async function applyFeatureFlagOverrideCommand(command){
    if(!isDevBuild)throw Error('Feature flag overrides are available only in development builds.');
    const next={...overrides},kind=String(command?.kind||'');
    if(kind==='set'){const name=String(command?.name||'').trim();if(!name)throw Error('Feature flag name is required.');next[name]=command?.value===true}
    else if(kind==='clear')delete next[String(command?.name||'')];
    else if(kind==='clear-all')for(const key of Object.keys(next))delete next[key];
    else throw Error('Unsupported feature flag override command.');
    overrides=next;await writeOverrides({...overrides});return emit();
  }
  function subscribe(listener){listeners.add(listener);return()=>listeners.delete(listener)}
  function checkFeatureGate(name){return featureGates()[String(name||'')]===true}
  function getDynamicConfig(name){return safeObject(remote.dynamicConfigs[String(name||'')])}
  function getComputerUseModelOverride(){return checkFeatureGate('sand_computer_use_playwright')?getDynamicConfig('sand_computer_use_playwright_config'):undefined}
  function getBrowserUseModelOverride(){return checkFeatureGate('sand_browser_use_subagent')?getDynamicConfig('sand_browser_use_model'):undefined}
  async function dispose(){disposed=true;if(timer)clearTimeout(timer);timer=null;listeners.clear();if(loading)await loading.catch(()=>{})}
  return{
    start,refreshNow,subscribe,getSnapshot:snapshot,getFeatureFlagOverridesRecord:()=>({...overrides}),
    applyFeatureFlagOverrideCommand,checkFeatureGate,getDynamicConfig,hasLiveStatsigBootstrap:()=>live,
    getFlagsAgeMs:()=>flagsFetchedAtMs==null?undefined:Math.max(0,Date.now()-flagsFetchedAtMs),
    getComputerUseModelOverride,getBrowserUseModelOverride,dispose
  };
}
module.exports={BASE_POLL_INTERVAL_MS,MIN_POLL_INTERVAL_MS,parseEnvOverrides,normalizeRemoteSnapshot,createExperimentsRuntime};
