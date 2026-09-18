'use strict';

const SEND_MESSAGE_TOOL_NAME='SendMessage';
const REACT_TO_MESSAGE_TOOL_NAME='ReactToMessage';

const definitions=[
  {type:'function',function:{name:SEND_MESSAGE_TOOL_NAME,description:'Say something visible to the user in the agent chat. Use this for acknowledgements, progress updates, blockers and final results. Plain assistant scratchpad text is not a substitute for delivery.',parameters:{type:'object',properties:{type:{type:'string',enum:['text']},content:{type:'string'},reply_to:{type:'string',description:'Optional earlier message id to reply to.'}},required:['type','content'],additionalProperties:false}}},
  {type:'function',function:{name:REACT_TO_MESSAGE_TOOL_NAME,description:'React sparingly to one of the user messages with a single emoji. Repeating the same emoji toggles it off.',parameters:{type:'object',properties:{message_address:{type:'string'},emoji:{type:'string',maxLength:16}},required:['message_address','emoji'],additionalProperties:false}}}
];
function executeCommunicationTool(name,args,{sendVisibleMessage,reactToConversationMessage}){
  if(name===SEND_MESSAGE_TOOL_NAME){
    if(args?.type!=='text')throw Error('SendMessage currently requires type=text.');
    const content=String(args?.content||'').trim();if(!content)throw Error('SendMessage content is required.');
    return Promise.resolve(sendVisibleMessage({content,replyToId:String(args?.reply_to||'').trim()||null})).then(messageId=>({text:messageId?'Message sent to user. (id: '+messageId+')':'Message sent to user.',visibleMessage:true}));
  }
  if(name===REACT_TO_MESSAGE_TOOL_NAME){
    const messageAddress=String(args?.message_address||'').trim(),emoji=String(args?.emoji||'').trim();
    if(!messageAddress||!emoji)throw Error('ReactToMessage requires message_address and emoji.');
    return Promise.resolve(reactToConversationMessage({messageAddress,emoji})).then(()=>({text:'Reacted '+emoji+' on '+messageAddress+'.',visibleMessage:false}));
  }
  return null;
}
module.exports={SEND_MESSAGE_TOOL_NAME,REACT_TO_MESSAGE_TOOL_NAME,definitions,executeCommunicationTool};
