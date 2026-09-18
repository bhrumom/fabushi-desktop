'use strict';

const crypto=require('node:crypto');

function abortError(reason='Operation cancelled.'){
  const error=Error(String(reason||'Operation cancelled.'));error.name='AbortError';return error;
}
function createAgentRunner({agentId,runTurn,onLifecycle=()=>{},onStateChanged=()=>{}}){
  if(typeof runTurn!=='function')throw Error('Agent runner requires runTurn.');
  let generation=0,active=null,quiescing=false,disposed=false;
  function snapshot(){
    return{
      agentId:String(agentId||''),generation,quiescing,
      active:active?{requestId:active.requestId,generation:active.generation,startedAt:active.startedAt,dispatched:active.dispatched,interrupted:active.interrupted}:null
    };
  }
  function emitState(){try{onStateChanged(snapshot())}catch{}}
  async function run(input={}){
    if(disposed)throw Error('Agent runner is disposed.');
    if(active)throw Error('Agent already running.');
    if(quiescing)return{quiescedForUpgrade:true,value:null};
    const requestId=String(input.requestId||crypto.randomUUID()),controller=new AbortController();
    const external=input.signal;
    const onExternalAbort=()=>controller.abort(external?.reason||abortError());
    if(external?.aborted)onExternalAbort();else external?.addEventListener?.('abort',onExternalAbort,{once:true});
    const current={requestId,generation,controller,startedAt:Date.now(),dispatched:false,interrupted:false};
    active=current;emitState();try{onLifecycle({type:'started',agentId,requestId,generation,startedAt:current.startedAt})}catch{}
    try{
      if(controller.signal.aborted)throw abortError(controller.signal.reason?.reason||controller.signal.reason?.message||'Operation cancelled.');
      current.dispatched=true;emitState();
      const value=await runTurn({...input,signal:controller.signal,requestId,generation});
      return{quiescedForUpgrade:false,value,requestId,generation};
    }finally{
      external?.removeEventListener?.('abort',onExternalAbort);
      const endedAt=Date.now();try{onLifecycle({type:'ended',agentId,requestId,generation,startedAt:current.startedAt,endedAt,durationMs:Math.max(0,endedAt-current.startedAt),interrupted:current.interrupted})}catch{}
      if(active===current)active=null;emitState();
    }
  }
  function interrupt(reason='Stopped by user.'){
    if(!active)return false;active.interrupted=true;active.controller.abort({intentional:true,reason:String(reason)});emitState();return true;
  }
  function requestQuiesceForUpgrade(){
    quiescing=true;if(active){active.interrupted=true;active.controller.abort({intentional:true,reason:'upgrade-quiesce'})}emitState();
  }
  function cancelQuiesceForUpgrade(){quiescing=false;emitState()}
  function reset(){interrupt('runner-reset');generation+=1;quiescing=false;emitState();return snapshot()}
  function bumpGeneration(){generation+=1;emitState();return generation}
  async function dispose(){if(disposed)return;disposed=true;interrupt('runner-dispose');emitState()}
  return{run,interrupt,requestQuiesceForUpgrade,cancelQuiesceForUpgrade,isQuiescingForUpgrade:()=>quiescing,reset,bumpGeneration,snapshot,dispose};
}
module.exports={createAgentRunner,abortError};
