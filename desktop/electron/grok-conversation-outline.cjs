'use strict';

const MAX_TOOL_ACTIVITY_ARGS_CHARS=20_000;

function hiddenKey(key){
  return /token|secret|password|authorization|cookie|api[-_]?key/i.test(String(key));
}
function safeArgs(args){
  if(!args||typeof args!=='object'||Array.isArray(args))return undefined;
  const copy={};
  for(const [key,value] of Object.entries(args)){
    if(hiddenKey(key)){copy[key]='[redacted]';continue}
    if(typeof value==='string')copy[key]=value.slice(0,4000);
    else if(typeof value==='number'||typeof value==='boolean'||value==null)copy[key]=value;
    else if(Array.isArray(value))copy[key]=value.slice(0,32);
  }
  let serialized;
  try{serialized=JSON.stringify(copy)}catch{return undefined}
  if(['{}','""','[]','null'].includes(serialized))return undefined;
  if(serialized.length<=MAX_TOOL_ACTIVITY_ARGS_CHARS)return serialized;
  return serialized.slice(0,MAX_TOOL_ACTIVITY_ARGS_CHARS)+'\n… (truncated)';
}
function toolStatus(status){
  if(status==='done')return'done';
  if(status==='error'||status==='cancelled')return'failed';
  return'pending';
}
function toolSummary(message){
  const args=message?.arguments||{};
  for(const key of ['purpose','description','prompt','command','path','url']){
    const value=args?.[key];
    if(typeof value==='string'&&value.trim())return value.trim().slice(0,1000);
  }
  return safeArgs(args);
}
function outlineItem(message,index){
  const id=String(message?.id||'outline-'+index);
  if(message?.role==='user')return{kind:'user',id,text:String(message.text||'')};
  if(message?.role==='assistant'){
    const text=String(message.text||'');
    return text?{kind:'assistant-text',id,text}:null;
  }
  if(message?.role==='tool'){
    return{
      kind:'tool-call',id,name:String(message.toolName||'Tool'),
      status:toolStatus(message.status),summary:toolSummary(message),
      ...(message.outputLocation?{outputLocation:message.outputLocation}:{})
    };
  }
  return null;
}
function deriveConversationOutline(messages){
  const turns=[];let current=null;
  for(let index=0;index<(messages||[]).length;index++){
    const message=messages[index];
    if(message?.role==='user'){
      current={rawUserText:String(message.text||''),userMessageId:String(message.id||''),items:[]};
      turns.push(current);
    }else if(!current){
      current={rawUserText:'',userMessageId:'',items:[]};turns.push(current);
    }
    const item=outlineItem(message,index);
    if(item)current.items.push(item);
  }
  return turns;
}
function flattenConversationOutline(messages){
  return deriveConversationOutline(messages).flatMap(turn=>turn.items);
}

module.exports={MAX_TOOL_ACTIVITY_ARGS_CHARS,safeArgs,toolStatus,toolSummary,deriveConversationOutline,flattenConversationOutline};
