'use strict';

const crypto=require('node:crypto');
const {SandAgentRunner}=require('./grok-reference-sand-runner.cjs');

function abortError(reason='Operation cancelled.'){
  const error=Error(String(reason||'Operation cancelled.'));error.name='AbortError';return error;
}
function promptFromInput(input){
  const history=Array.isArray(input?.history)?input.history:[];
  for(let index=history.length-1;index>=0;index-=1){
    const row=history[index];
    if(row&&row.role==='user'&&String(row.text||'').trim())return String(row.text).trim();
  }
  return '[Fabushi local agent turn]';
}
function createAgentRunner({agentId,runTurn,onLifecycle=()=>{},onStateChanged=()=>{}}){
  if(typeof runTurn!=='function')throw Error('Agent runner requires runTurn.');
  let generation=0,active=null,quiescing=false,disposed=false;
  function snapshot(){
    return{
      agentId:String(agentId||''),generation,quiescing,engine:'grok-sand-agent-runner',
      active:active?{requestId:active.requestId,generation:active.generation,startedAt:active.startedAt,dispatched:active.dispatched,interrupted:active.interrupted}:null
    };
  }
  function emitState(){try{onStateChanged(snapshot())}catch{}}
  async function run(input={}){
    if(disposed)throw Error('Agent runner is disposed.');
    if(active)throw Error('Agent already running.');
    if(quiescing)return{quiescedForUpgrade:true,value:null,engine:'grok-sand-agent-runner'};
    const requestId=String(input.requestId||crypto.randomUUID());
    if(input.signal?.aborted)throw abortError(input.signal.reason?.reason||input.signal.reason?.message||'Operation cancelled.');
    const current={requestId,generation,startedAt:Date.now(),dispatched:false,interrupted:false,referenceRunner:null};
    const referenceRunner=new SandAgentRunner({
      getAgentId:()=>String(agentId||''),
      maxSteps:1,
      runStep:async(_step,context)=>{
        if(input.signal?.aborted)throw abortError(input.signal.reason?.reason||input.signal.reason?.message||'Operation cancelled.');
        current.dispatched=true;emitState();
        const value=await runTurn({...input,signal:context.signal,requestId:context.requestId,generation});
        if(input.signal?.aborted)throw abortError(input.signal.reason?.reason||input.signal.reason?.message||'Operation cancelled.');
        return{done:true,value};
      }
    });
    current.referenceRunner=referenceRunner;active=current;emitState();
    const onExternalAbort=()=>{current.interrupted=true;referenceRunner.interrupt(String(input.signal?.reason?.reason||input.signal?.reason?.message||'Operation cancelled.'));emitState()};
    input.signal?.addEventListener?.('abort',onExternalAbort,{once:true});
    try{
      try{onLifecycle({type:'started',agentId,requestId,generation,startedAt:current.startedAt,engine:'grok-sand-agent-runner'})}catch{}
      const value=await referenceRunner.run(promptFromInput(input),{inferenceRequestId:requestId,requestSource:'user'});
      if(input.signal?.aborted)throw abortError(input.signal.reason?.reason||input.signal.reason?.message||'Operation cancelled.');
      return{quiescedForUpgrade:false,value,requestId,generation,engine:'grok-sand-agent-runner'};
    }finally{
      input.signal?.removeEventListener?.('abort',onExternalAbort);
      const endedAt=Date.now();try{onLifecycle({type:'ended',agentId,requestId,generation,startedAt:current.startedAt,endedAt,durationMs:Math.max(0,endedAt-current.startedAt),interrupted:current.interrupted,engine:'grok-sand-agent-runner'})}catch{}
      if(active===current)active=null;emitState();
    }
  }
  function interrupt(reason='Stopped by user.'){
    if(!active)return false;
    active.interrupted=true;
    const interrupted=active.referenceRunner?.interrupt(String(reason));
    emitState();
    return interrupted!==false;
  }
  function requestQuiesceForUpgrade(){
    quiescing=true;
    if(active){active.interrupted=true;active.referenceRunner?.requestQuiesceForUpgrade()}
    emitState();
  }
  function cancelQuiesceForUpgrade(){
    quiescing=false;
    active?.referenceRunner?.cancelQuiesceForUpgrade();
    emitState();
  }
  function reset(){interrupt('runner-reset');generation+=1;quiescing=false;emitState();return snapshot()}
  function bumpGeneration(){generation+=1;emitState();return generation}
  async function dispose(){if(disposed)return;disposed=true;interrupt('runner-dispose');emitState()}
  return{run,interrupt,requestQuiesceForUpgrade,cancelQuiesceForUpgrade,isQuiescingForUpgrade:()=>quiescing,reset,bumpGeneration,snapshot,dispose};
}
module.exports={createAgentRunner,abortError};
