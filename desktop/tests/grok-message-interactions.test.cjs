'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const {SELF_REACTION,toggleSelfReaction,resolveReplyTarget,extractLinks,searchWorkspaceIndex}=require('../electron/grok-message-interactions.cjs');

test('reaction state toggles the local self identity without duplicating reactions',()=>{
  const message={id:'m1',role:'assistant',reactions:[{emoji:'🎉',by:'other'}]};
  toggleSelfReaction(message,'👍');
  assert.deepEqual(message.reactions,[{emoji:'🎉',by:'other'},{emoji:'👍',by:SELF_REACTION}]);
  toggleSelfReaction(message,'👍');
  assert.deepEqual(message.reactions,[{emoji:'🎉',by:'other'}]);
});

test('reply targets resolve only conversation messages',()=>{
  const rows=[{id:'u1',role:'user',text:'hello'},{id:'t1',role:'tool',text:'tool'}];
  assert.equal(resolveReplyTarget(rows,'u1'),rows[0]);
  assert.equal(resolveReplyTarget(rows,'t1'),null);
  assert.equal(resolveReplyTarget(rows,'missing'),null);
});

test('workspace search projects messages, attachments and links from persisted transcript data',()=>{
  const index=searchWorkspaceIndex({
    agents:[{id:'a1',name:'Chief'}],
    messages:{a1:[{
      id:'u1',role:'user',text:'Review https://example.test/docs today',createdAt:100,
      attachments:[{id:'f1',name:'plan.pdf',mime:'application/pdf',createdAt:99}]
    },{id:'a1m',role:'assistant',text:'The plan is ready.',createdAt:101}]},
    query:'plan',limit:50
  });
  assert.equal(index.messages.length,1);
  assert.equal(index.messages[0].entryId,'a1m');
  assert.equal(index.media.length,1);
  assert.equal(index.media[0].kind,'pdf');
  assert.equal(index.links.length,0);
  assert.deepEqual(extractLinks('See https://example.test/docs, then https://example.test/docs.'),['https://example.test/docs']);
});
