'use strict';

const SEND_MESSAGE_TOOL_NAME='SendMessage';
const REACT_TO_MESSAGE_TOOL_NAME='ReactToMessage';
const SEND_MESSAGE_TYPES=['text','attachment','widget','secret-request'];

const definitions=[
  {type:'function',function:{name:SEND_MESSAGE_TOOL_NAME,description:'Send something visible to the user in Fabushi. Use text for chat/progress, attachment for a file or media URL, widget for an interactive choice, and secret-request for a credential that must never be pasted into chat. The reference cursor-agent card is intentionally absent because Fabushi agents run on the installed computer rather than cloud agents.',parameters:{type:'object',properties:{
    type:{type:'string',enum:SEND_MESSAGE_TYPES},
    content:{type:'string'},url:{type:'string'},alt:{type:'string'},reply_to:{type:'string'},
    images:{type:'array',items:{type:'object',properties:{url:{type:'string'},alt:{type:'string'}},required:['url'],additionalProperties:false}},
    widget:{type:'object',properties:{prompt:{type:'string'},helpText:{type:'string'},options:{type:'array',items:{type:'object',properties:{label:{type:'string'},value:{type:'string'},description:{type:'string'},style:{type:'string'}},required:['label'],additionalProperties:false}},allowCustom:{type:'boolean'},dismissOnMoveOn:{type:'boolean'}},required:['prompt','options'],additionalProperties:false},
    secret:{type:'object',properties:{label:{type:'string'},description:{type:'string'},connector:{type:'string'},field:{type:'string'}},required:['label','connector','field'],additionalProperties:false}
  },required:['type'],additionalProperties:false}}},
  {type:'function',function:{name:REACT_TO_MESSAGE_TOOL_NAME,description:'React sparingly to one of the user messages with a single emoji. Repeating the same emoji toggles it off.',parameters:{type:'object',properties:{message_address:{type:'string'},emoji:{type:'string',maxLength:16}},required:['message_address','emoji'],additionalProperties:false}}}
];

function nonEmpty(value,label,max=4000){const text=String(value||'').trim();if(!text)throw Error(label+' is required.');return text.slice(0,max)}
function validAttachmentUrl(value){
  let url;try{url=new URL(nonEmpty(value,'Attachment URL',20000))}catch{throw Error('Attachment URL must be a valid file:// or https:// URL.')}
  if(url.protocol!=='file:'&&url.protocol!=='https:')throw Error('Attachment URL must use file:// or https://.');
  return url.toString();
}
function normalizeWidget(value){
  if(!value||typeof value!=='object'||Array.isArray(value))throw Error('Widget payload is required.');
  const prompt=nonEmpty(value.prompt,'Widget prompt',1000);
  if(!Array.isArray(value.options)||value.options.length<1||value.options.length>26)throw Error('Widget requires 1 to 26 options.');
  const options=value.options.map((option,index)=>{
    if(!option||typeof option!=='object')throw Error('Widget option '+(index+1)+' is invalid.');
    const label=nonEmpty(option.label,'Widget option label',240);
    const next={label};
    if(String(option.value||'').trim())next.value=String(option.value).trim().slice(0,1000);
    if(String(option.description||'').trim())next.description=String(option.description).trim().slice(0,1000);
    if(String(option.style||'').trim())next.style=String(option.style).trim().slice(0,80);
    return next;
  });
  return{prompt,...(String(value.helpText||'').trim()?{helpText:String(value.helpText).trim().slice(0,1000)}:{}),options,allowCustom:value.allowCustom===true,dismissOnMoveOn:value.dismissOnMoveOn===true};
}
function normalizeSecret(value){
  if(!value||typeof value!=='object'||Array.isArray(value))throw Error('Secret request payload is required.');
  return{label:nonEmpty(value.label,'Secret label',120),...(String(value.description||'').trim()?{description:String(value.description).trim().slice(0,400)}:{}),connector:nonEmpty(value.connector,'Secret connector',120),field:nonEmpty(value.field,'Secret field',120)};
}
function normalizeSendMessage(args){
  const type=String(args?.type||'');
  if(!SEND_MESSAGE_TYPES.includes(type))throw Error('Unsupported SendMessage type: '+type);
  const replyToId=String(args?.reply_to||'').trim()||null;
  if(type==='text'){
    const content=nonEmpty(args?.content,'SendMessage content',100000);
    const images=Array.isArray(args?.images)?args.images.slice(0,12).map(image=>({url:validAttachmentUrl(image?.url),...(String(image?.alt||'').trim()?{alt:String(image.alt).trim().slice(0,500)}:{})})):[];
    return{type,content,replyToId,images};
  }
  if(type==='attachment')return{type,url:validAttachmentUrl(args?.url),alt:String(args?.alt||'').trim().slice(0,500),replyToId};
  if(type==='widget')return{type,widget:normalizeWidget(args?.widget),replyToId};
  return{type,secret:normalizeSecret(args?.secret),replyToId};
}
function executeCommunicationTool(name,args,{sendVisibleMessage,reactToConversationMessage}){
  if(name===SEND_MESSAGE_TOOL_NAME){
    const payload=normalizeSendMessage(args);
    return Promise.resolve(sendVisibleMessage(payload)).then(messageId=>({text:messageId?'Message sent to user. (id: '+messageId+')':'Message sent to user.',visibleMessage:true}));
  }
  if(name===REACT_TO_MESSAGE_TOOL_NAME){
    const messageAddress=String(args?.message_address||'').trim(),emoji=String(args?.emoji||'').trim();
    if(!messageAddress||!emoji)throw Error('ReactToMessage requires message_address and emoji.');
    return Promise.resolve(reactToConversationMessage({messageAddress,emoji})).then(()=>({text:'Reacted '+emoji+' on '+messageAddress+'.',visibleMessage:false}));
  }
  return null;
}
module.exports={SEND_MESSAGE_TOOL_NAME,REACT_TO_MESSAGE_TOOL_NAME,SEND_MESSAGE_TYPES,definitions,validAttachmentUrl,normalizeWidget,normalizeSecret,normalizeSendMessage,executeCommunicationTool};
