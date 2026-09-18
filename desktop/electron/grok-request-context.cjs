'use strict';

const crypto=require('node:crypto');
const {AsyncLocalStorage}=require('node:async_hooks');

const storage=new AsyncLocalStorage();

function rootRequestContext(input={}){
  return Object.freeze({
    requestId:String(input.requestId||crypto.randomUUID()),
    parentRequestId:input.parentRequestId?String(input.parentRequestId):null,
    agentId:input.agentId?String(input.agentId):null,
    conversationId:input.conversationId?String(input.conversationId):null,
    toolCallId:input.toolCallId?String(input.toolCallId):null,
    directionEpoch:Number.isFinite(input.directionEpoch)?Number(input.directionEpoch):Date.now(),
    depth:Number.isFinite(input.depth)?Number(input.depth):0,
    signal:input.signal||null
  });
}
function currentRequestContext(){return storage.getStore()||null}
function childRequestContext(patch={}){
  const parent=currentRequestContext();
  return rootRequestContext({
    requestId:patch.requestId,
    parentRequestId:patch.parentRequestId??parent?.requestId,
    agentId:patch.agentId??parent?.agentId,
    conversationId:patch.conversationId??parent?.conversationId,
    toolCallId:patch.toolCallId??parent?.toolCallId,
    directionEpoch:patch.directionEpoch??parent?.directionEpoch,
    depth:patch.depth??((parent?.depth||0)+1),
    signal:patch.signal??parent?.signal
  });
}
function runWithRequestContext(context,fn){return storage.run(Object.freeze({...context}),fn)}
function throwIfRequestCancelled(context=currentRequestContext()){
  if(context?.signal?.aborted){
    const error=Error('Request cancelled.');error.name='AbortError';throw error;
  }
}
module.exports={rootRequestContext,childRequestContext,currentRequestContext,runWithRequestContext,throwIfRequestCancelled};
