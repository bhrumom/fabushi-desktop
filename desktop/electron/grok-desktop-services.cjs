'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');

const TRACKS=['stable','nightly','dogfood'];
function updateDisabledReason({app,env=process.env,platform=process.platform}){
  if(env.FABUSHI_DISABLE_UPDATES==='1')return'disabled-by-env';
  if(!app.isPackaged&&!String(env.FABUSHI_UPDATE_FEED_URL||'').trim())return'not-packaged';
  if(platform!=='darwin'&&platform!=='win32')return'unsupported-platform';
  if(!String(env.FABUSHI_UPDATE_FEED_URL||'').trim())return'disabled-by-env';
  return null;
}
function cleanTrack(value,fallback='stable'){return TRACKS.includes(value)?value:fallback}
function createDesktopServices({app,autoUpdater,env=process.env,platform=process.platform,fetchImpl=globalThis.fetch,onUpdateStatus=()=>{}}){
  const stateFile=path.join(app.getPath('userData'),'desktop-preferences.json');
  const buildTrack=cleanTrack(env.FABUSHI_UPDATE_TRACK,'stable');
  let prefs=null,configuredFeed=null,updateState=null;
  const listeners=[];
  async function loadPrefs(){
    if(prefs)return prefs;
    try{const parsed=JSON.parse(await fs.readFile(stateFile,'utf8'));prefs={trackOverride:TRACKS.includes(parsed.trackOverride)?parsed.trackOverride:null,autoUpdateWhenIdleOptIn:parsed.autoUpdateWhenIdleOptIn===true,onboardingSeen:parsed.onboardingSeen===true};}
    catch{prefs={trackOverride:null,autoUpdateWhenIdleOptIn:false,onboardingSeen:false}}
    return prefs;
  }
  async function savePrefs(){
    await fs.mkdir(path.dirname(stateFile),{recursive:true,mode:0o700});
    const tmp=stateFile+'.tmp';await fs.writeFile(tmp,JSON.stringify(await loadPrefs(),null,2)+'\n',{mode:0o600});await fs.rename(tmp,stateFile);
  }
  async function currentTrack(){const value=await loadPrefs();return cleanTrack(value.trackOverride,buildTrack)}
  function disabled(reason){return{type:'disabled',reason}}
  function emit(){void status().then(value=>{onUpdateStatus(value)}).catch(()=>{})}
  async function status(){
    const preferences=await loadPrefs(),reason=updateDisabledReason({app,env,platform});
    return{
      state:reason?disabled(reason):(updateState||{type:'idle'}),
      currentVersion:String(app.getVersion()),
      currentTrack:cleanTrack(preferences.trackOverride,buildTrack),
      trackOverride:preferences.trackOverride,
      buildDefaultTrack:buildTrack,
      availableTracks:[...TRACKS],
      isTrackManagedByPolicy:false,
      isBelowMinimumVersion:false,
      autoUpdateWhenIdleOptIn:preferences.autoUpdateWhenIdleOptIn,
      autoUpdateWhenIdleGateEnabled:false
    };
  }
  async function configureFeed(){
    const reason=updateDisabledReason({app,env,platform});if(reason)return false;
    const track=await currentTrack(),template=String(env.FABUSHI_UPDATE_FEED_URL||'').trim();
    const feed=template.replaceAll('{track}',encodeURIComponent(track));
    if(feed!==configuredFeed){autoUpdater.setFeedURL({url:feed});configuredFeed=feed}
    return true;
  }
  function bind(){
    if(!autoUpdater?.on)return;
    const on=(name,handler)=>{autoUpdater.on(name,handler);listeners.push([name,handler])};
    on('checking-for-update',()=>{updateState={type:'checking'};emit()});
    on('update-available',info=>{updateState={type:'available',version:String(info?.version||info?.releaseName||'new version')};emit()});
    on('update-not-available',()=>{updateState={type:'idle',lastCheck:{at:Date.now(),result:'up-to-date'}};emit()});
    on('download-progress',progress=>{updateState={type:'downloading',version:String(progress?.version||'new version'),progress:Number.isFinite(Number(progress?.percent))?Math.max(0,Math.min(1,Number(progress.percent)/100)):undefined};emit()});
    on('update-downloaded',info=>{updateState={type:'ready',version:String(info?.version||info?.releaseName||'new version')};emit()});
    on('error',error=>{updateState={type:'idle',lastCheck:{at:Date.now(),result:'error',errorMessage:String(error?.message||error||'Update failed')}};emit()});
  }
  bind();
  async function check(){
    if(!(await configureFeed()))return status();
    updateState={type:'checking'};emit();
    try{
      const result=await autoUpdater.checkForUpdates();
      if(result?.updateInfo?.version&&updateState?.type==='checking')updateState={type:'available',version:String(result.updateInfo.version)};
    }catch(error){updateState={type:'idle',lastCheck:{at:Date.now(),result:'error',errorMessage:String(error?.message||error)}}}
    emit();return status();
  }
  async function setTrack(track){
    const value=cleanTrack(track,null);if(!value)throw Error('Unknown update track.');
    const preferences=await loadPrefs();preferences.trackOverride=value;await savePrefs();configuredFeed=null;emit();return status();
  }
  async function setAutoUpdateWhenIdleOptIn(enabled){
    const preferences=await loadPrefs();preferences.autoUpdateWhenIdleOptIn=enabled===true;await savePrefs();emit();return status();
  }
  async function quitAndInstall(){
    if(updateState?.type!=='ready')throw Error('No staged update is ready to install.');
    autoUpdater.quitAndInstall();
  }
  async function submitFeedback(payload={}){
    const message=String(payload.message||'').trim();
    if(!message||message.length>10000)return{ok:false,code:'invalid-feedback'};
    const endpoint=String(env.FABUSHI_FEEDBACK_URL||'').trim();
    if(!endpoint)return{ok:false,code:'unavailable'};
    let url;try{url=new URL(endpoint)}catch{return{ok:false,code:'unavailable'}}
    if(url.protocol!=='https:'&&!(url.protocol==='http:'&&(url.hostname==='127.0.0.1'||url.hostname==='localhost')))return{ok:false,code:'unavailable'};
    const controller=new AbortController(),timer=setTimeout(()=>controller.abort(),12000);
    try{
      const response=await fetchImpl(url,{method:'POST',headers:{'content-type':'application/json',accept:'application/json',...(env.FABUSHI_FEEDBACK_TOKEN?{authorization:'Bearer '+env.FABUSHI_FEEDBACK_TOKEN}:{})},body:JSON.stringify({message,...(payload.conversationId?{conversationId:String(payload.conversationId)}:{})}),signal:controller.signal});
      if(response.ok)return{ok:true};
      if(response.status===401)return{ok:false,code:'not-signed-in'};
      if(response.status===402)return{ok:false,code:'subscription-required'};
      if(response.status===403)return{ok:false,code:'access-denied'};
      if(response.status===429)return{ok:false,code:'rate-limited'};
      if(response.status>=400&&response.status<500)return{ok:false,code:'invalid-feedback'};
      return{ok:false,code:'unavailable'};
    }catch{return{ok:false,code:'unavailable'}}finally{clearTimeout(timer)}
  }
  async function getOnboardingSeen(){return(await loadPrefs()).onboardingSeen===true}
  async function setOnboardingSeen(seen){const preferences=await loadPrefs();preferences.onboardingSeen=seen===true;await savePrefs();return preferences.onboardingSeen}
  function dispose(){for(const[name,handler]of listeners)autoUpdater?.off?.(name,handler);listeners.length=0}
  return{getInfo:()=>({version:String(app.getVersion()),platform,isPackaged:app.isPackaged===true}),getOnboardingSeen,setOnboardingSeen,update:{status,check,setTrack,setAutoUpdateWhenIdleOptIn,quitAndInstall},submitFeedback,dispose};
}
function parseDeepLink(value){
  let url;try{url=new URL(String(value||''))}catch{return null}
  if(url.protocol!=='sand:'&&url.protocol!=='fabushi:')return null;
  if(url.hostname!=='app'||url.pathname!=='/v1/info'||url.searchParams.get('topic')!=='deep-links')return null;
  return{version:1,source:'protocol',route:'info',topic:'deep-links',url:url.toString()};
}
module.exports={TRACKS,updateDisabledReason,createDesktopServices,parseDeepLink};
