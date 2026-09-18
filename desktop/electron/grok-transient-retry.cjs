'use strict';

const TRANSIENT_CODES=new Set(['ECONNRESET','ETIMEDOUT','EPIPE','ECONNABORTED','ECONNREFUSED','ENETRESET','ENETDOWN','ENETUNREACH','EHOSTUNREACH','EAI_AGAIN']);
const TOKENS=['econnreset','etimedout','epipe','socket hang up','premature close','stream closed','connection reset','connection closed','network error','unavailable','deadline_exceeded'];
function chain(error){
  const rows=[],seen=new Set(),queue=[error];
  while(queue.length){const value=queue.shift();if(value==null||seen.has(value))continue;seen.add(value);rows.push(value);if(typeof value==='object'){if(value.cause)queue.push(value.cause);if(Array.isArray(value.errors))queue.push(...value.errors)}}
  return rows;
}
function messageLooksTransient(value){const lower=String(value||'').toLowerCase();return TOKENS.some(token=>lower.includes(token))}
function isTransientError(error){
  if(typeof error==='string')return messageLooksTransient(error);
  return chain(error).some(value=>typeof value==='object'&&value!==null&&(
    (typeof value.code==='string'&&TRANSIENT_CODES.has(value.code.toUpperCase()))||
    (typeof value.message==='string'&&messageLooksTransient(value.message))||
    value.retryable===true
  ));
}
function retryAfterMs(error){
  for(const value of chain(error)){
    if(typeof value!=='object'||value===null)continue;
    const raw=value.retryAfterMs??value.metadata?.get?.('retry-after')??value.metadata?.['retry-after'];
    if(raw==null||raw==='')continue;
    const direct=Number(raw);if(!Number.isFinite(direct)||direct<0)continue;
    return Math.min(30000,value.retryAfterMs!=null?Math.round(direct):Math.round(direct*1000));
  }
  return null;
}
function backoff(attempt,{baseDelayMs=750,maxDelayMs=6000,random=Math.random}={}){
  const exponential=Math.min(maxDelayMs,baseDelayMs*2**Math.max(0,attempt-1));
  return Math.min(maxDelayMs,Math.round(exponential/2+random()*exponential/2));
}
async function runWithTransientRetry(run,{signal,maxAttempts=3,baseDelayMs=750,maxDelayMs=6000,random=Math.random,sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms)),onRetry=()=>{},canRetry=()=>true}={}){
  for(let attempt=1;;attempt++){
    try{return await run(attempt)}catch(error){
      if(signal?.aborted)throw error;
      if(attempt>=maxAttempts||!isTransientError(error)||!canRetry(error,attempt))throw error;
      const server=retryAfterMs(error);
      const delayMs=server==null?backoff(attempt,{baseDelayMs,maxDelayMs,random}):Math.min(30000,Math.round(server+random()*server/2));
      onRetry({attempt,delayMs,serverPaced:server!=null,error});
      if(delayMs>0)await sleep(delayMs);
    }
  }
}
module.exports={TRANSIENT_CODES,messageLooksTransient,isTransientError,retryAfterMs,backoff,runWithTransientRetry};
