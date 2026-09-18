'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const {definitions,executeCommunicationTool,SEND_MESSAGE_TOOL_NAME,REACT_TO_MESSAGE_TOOL_NAME}=require('../electron/grok-communication-tools.cjs');

test('communication tools expose the reference tool names',()=>{
  assert.deepEqual(definitions.map(row=>row.function.name),[SEND_MESSAGE_TOOL_NAME,REACT_TO_MESSAGE_TOOL_NAME]);
});
test('SendMessage delivers text and returns its id',async()=>{
  const calls=[];
  const result=await executeCommunicationTool('SendMessage',{type:'text',content:'Working on it.',reply_to:'u1'},{sendVisibleMessage:async input=>{calls.push(input);return'a1'},reactToConversationMessage:async()=>{}});
  assert.equal(result.visibleMessage,true);assert.equal(calls[0].replyToId,'u1');assert.match(result.text,/a1/);
});
test('ReactToMessage routes the target id and emoji',async()=>{
  let call=null;
  await executeCommunicationTool('ReactToMessage',{message_address:'u1',emoji:'👍'},{sendVisibleMessage:async()=>null,reactToConversationMessage:async input=>{call=input}});
  assert.deepEqual(call,{messageAddress:'u1',emoji:'👍'});
});
