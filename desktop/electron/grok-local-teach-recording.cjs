'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');
const crypto=require('node:crypto');
const {spawn}=require('node:child_process');

const SAND_TEACH_MAX_DURATION_MS=10*60*1000;

function idleStatus(){
  return{state:'idle',agentId:null,startedAtMs:null,maxDurationMs:SAND_TEACH_MAX_DURATION_MS};
}
function wait(ms){return new Promise(resolve=>setTimeout(resolve,ms))}
function createAbortError(message){const error=Error(message);error.name='AbortError';return error}

function createLocalTeachRecording({
  app,
  onSaved=async()=>{},
  spawnImpl=spawn,
  helperResolver=null,
  now=()=>Date.now(),
  startupGraceMs=450,
  stopGraceMs=10000
}={}){
  const root=path.join(app?.getPath?.('userData')||process.cwd(),'teach-recordings');
  let active=null,disposing=false;

  async function defaultHelperResolver(){
    const candidates=[
      process.resourcesPath?path.join(process.resourcesPath,'bin','fabushi-computer-helper'):null,
      path.join(__dirname,'..','native','bin','fabushi-computer-helper')
    ].filter(Boolean);
    for(const candidate of candidates){try{await fs.access(candidate);return candidate}catch{}}
    throw Error('Fabushi Computer helper is unavailable. Use the packaged macOS build so Teach recording can capture the local screen.');
  }
  const resolveHelper=helperResolver||defaultHelperResolver;

  function status(){
    const row=active;
    return row==null?idleStatus():{
      state:'recording',
      agentId:row.agentId,
      startedAtMs:row.startedAtMs,
      maxDurationMs:SAND_TEACH_MAX_DURATION_MS
    };
  }

  async function start({agentId,entryPoint}={}){
    if(disposing)throw Error('Teach recording service is shutting down.');
    if(process.platform!=='darwin'&&helperResolver==null)throw Error('Teach recording is supported on the packaged macOS build.');
    const id=String(agentId||'').trim();if(!id)throw Error('Teach recording requires an agent id.');
    if(active){
      if(active.agentId===id)return status();
      throw Error('Another Teach recording is already active.');
    }
    const helper=await resolveHelper();
    const startedAtMs=now();
    const sessionId='teach-'+new Date(startedAtMs).toISOString().replace(/[-:.]/g,'')+'-'+crypto.randomUUID();
    const sessionDir=path.join(root,sessionId),videoPath=path.join(sessionDir,'demo.mov'),logPath=path.join(sessionDir,'recording.log');
    await fs.mkdir(sessionDir,{recursive:true,mode:0o700});
    const child=spawnImpl(helper,['record',videoPath,String(Math.ceil(SAND_TEACH_MAX_DURATION_MS/1000))],{
      stdio:['ignore','pipe','pipe'],
      env:process.env
    });
    let stdout='',stderr='',exited=false,exitCode=null,exitSignal=null;
    const append=(key,chunk)=>{const value=String(chunk||'');if(key==='stdout')stdout=(stdout+value).slice(-200000);else stderr=(stderr+value).slice(-200000)};
    child.stdout?.on?.('data',chunk=>append('stdout',chunk));
    child.stderr?.on?.('data',chunk=>append('stderr',chunk));
    let exitResolve;
    const exitPromise=new Promise(resolve=>{exitResolve=resolve});
    child.once('exit',(code,signal)=>{exited=true;exitCode=code;exitSignal=signal;exitResolve({code,signal})});
    child.once('error',error=>{stderr=(stderr+'\n'+error.message).slice(-200000)});
    const row={
      agentId:id,entryPoint:String(entryPoint||'screen_hover'),startedAtMs,sessionId,sessionDir,videoPath,logPath,
      child,exitPromise,get exited(){return exited},get exitCode(){return exitCode},get exitSignal(){return exitSignal},
      readLog:()=>({stdout,stderr}),capTimer:null,stopping:null
    };
    active=row;
    try{
      await Promise.race([wait(startupGraceMs),exitPromise]);
      if(exited){
        await fs.writeFile(logPath,stdout+(stderr?'\n[stderr]\n'+stderr:''),'utf8').catch(()=>{});
        active=null;await fs.rm(sessionDir,{recursive:true,force:true}).catch(()=>{});
        throw Error('Teach recording could not start: '+(stderr.trim()||('native recorder exited '+String(exitCode??exitSignal??'early'))));
      }
      row.capTimer=setTimeout(()=>{void stop({agentId:id,save:true,reason:'max-duration'}).catch(()=>{})},SAND_TEACH_MAX_DURATION_MS);
      row.capTimer.unref?.();
      await fs.writeFile(path.join(sessionDir,'session.json'),JSON.stringify({
        agentId:id,sessionId,entryPoint:row.entryPoint,startedAt:new Date(startedAtMs).toISOString(),
        maxDurationMs:SAND_TEACH_MAX_DURATION_MS,videoPath
      },null,2)+'\n',{mode:0o600});
      return status();
    }catch(error){
      if(active===row)active=null;
      throw error;
    }
  }

  async function stop({agentId,save=true,reason='user'}={}){
    const row=active;
    if(!row)return idleStatus();
    const id=String(agentId||'').trim();
    if(id&&id!==row.agentId)throw Error('Teach recording belongs to a different agent.');
    if(row.stopping)return row.stopping;
    row.stopping=(async()=>{
      if(row.capTimer){clearTimeout(row.capTimer);row.capTimer=null}
      if(!row.exited){try{row.child.kill('SIGINT')}catch{}}
      let result=await Promise.race([row.exitPromise,wait(stopGraceMs).then(()=>null)]);
      if(result==null&&!row.exited){try{row.child.kill('SIGKILL')}catch{};result=await row.exitPromise}
      const {stdout,stderr}=row.readLog();
      await fs.writeFile(row.logPath,stdout+(stderr?'\n[stderr]\n'+stderr:''),'utf8').catch(()=>{});
      active=null;
      if(save!==true){
        await fs.rm(row.sessionDir,{recursive:true,force:true});
        return idleStatus();
      }
      let stat;
      try{stat=await fs.stat(row.videoPath)}catch{stat=null}
      if(!stat?.isFile()||stat.size<=0){
        throw Error('Teach recording did not produce a video. '+(stderr.trim()||'Screen Recording permission may be required in System Settings.'));
      }
      const endedAtMs=now();
      const metadata={
        agentId:row.agentId,sessionId:row.sessionId,entryPoint:row.entryPoint,
        startedAt:new Date(row.startedAtMs).toISOString(),endedAt:new Date(endedAtMs).toISOString(),
        endReason:String(reason||'user'),maxDurationMs:SAND_TEACH_MAX_DURATION_MS,
        videoPath:row.videoPath,videoBytes:stat.size,
        recorderExitCode:row.exitCode,recorderExitSignal:row.exitSignal
      };
      await fs.writeFile(path.join(row.sessionDir,'session.json'),JSON.stringify(metadata,null,2)+'\n',{mode:0o600});
      await onSaved({...metadata,sessionDir:row.sessionDir});
      return idleStatus();
    })();
    try{return await row.stopping}finally{row.stopping=null}
  }

  async function dispose(){
    disposing=true;
    if(active)await stop({agentId:active.agentId,save:false,reason:'dispose'}).catch(()=>{});
  }

  return{status,start,stop,dispose,root,maxDurationMs:SAND_TEACH_MAX_DURATION_MS};
}

module.exports={createLocalTeachRecording,SAND_TEACH_MAX_DURATION_MS,idleStatus};
