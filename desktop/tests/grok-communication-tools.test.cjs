'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const {definitions,executeCommunicationTool,SEND_MESSAGE_TOOL_NAME,REACT_TO_MESSAGE_TOOL_NAME,normalizeSendMessage}=require('../electron/grok-communication-tools.cjs');

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


test('SendMessage recovers attachment, widget and secret-request payloads without cloud-agent placeholders',()=>{
  assert.deepEqual(definitions[0].function.parameters.properties.type.enum,['text','attachment','widget','secret-request']);
  assert.deepEqual(normalizeSendMessage({type:'attachment',url:'https://example.test/report.pdf',alt:'Report'}),{type:'attachment',url:'https://example.test/report.pdf',alt:'Report',replyToId:null,channel:null});
  const widget=normalizeSendMessage({type:'widget',widget:{prompt:'Choose one',options:[{label:'A'},{label:'B',value:'b'}],allowCustom:true}});
  assert.equal(widget.widget.options[1].value,'b');assert.equal(widget.widget.allowCustom,true);
  const secret=normalizeSendMessage({type:'secret-request',secret:{label:'API token',connector:'github',field:'token'}});
  assert.deepEqual(secret.secret,{label:'API token',connector:'github',field:'token'});
  assert.throws(()=>normalizeSendMessage({type:'attachment',url:'http://example.test/file'}),/file:\/\/ or https:\/\//);
});
test('SendMessage routes non-text variants to the visible-message transport',async()=>{
  const calls=[];
  await executeCommunicationTool('SendMessage',{type:'widget',widget:{prompt:'Proceed?',options:[{label:'Yes',value:'yes'}]}},{sendVisibleMessage:async input=>{calls.push(input);return'w1'},reactToConversationMessage:async()=>{}});
  await executeCommunicationTool('SendMessage',{type:'secret-request',secret:{label:'Token',connector:'x',field:'token'}},{sendVisibleMessage:async input=>{calls.push(input);return's1'},reactToConversationMessage:async()=>{}});
  assert.equal(calls[0].type,'widget');assert.equal(calls[1].type,'secret-request');
});

test('SendMessage channel and widget protocol matches Grok constraints',()=>{
  const channel=normalizeSendMessage({type:'text',content:'Ship update',channel:'slack:C123'});
  assert.equal(channel.channel,'slack:C123');
  assert.throws(()=>normalizeSendMessage({type:'widget',channel:'slack:C123',widget:{prompt:'Go?',options:[{label:'Yes'}]}}),/channel is only valid/);
  assert.throws(()=>normalizeSendMessage({type:'text',content:'hello',url:'https://example.test/file'}),/url is not valid with type:text/);
  assert.throws(()=>normalizeSendMessage({type:'widget',widget:{prompt:'Too many',options:Array.from({length:7},(_,index)=>({label:String(index)}))}}),/1 to 6 options/);
});
