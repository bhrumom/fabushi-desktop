'use strict';
const test=require('node:test');const assert=require('node:assert/strict');const fs=require('node:fs/promises');const os=require('node:os');const path=require('node:path');
const {normalizeTurnUsage,mergeTurnUsage}=require('../electron/grok-turn-usage.cjs');
const {normalizeNavigationUrl,visitedSiteHost,classifyBotBlockPage,toolAuditAction,createActionAuditor}=require('../electron/grok-action-audit.cjs');

test('turn usage normalizes provider fields and merges with safe saturation',()=>{
 const a=normalizeTurnUsage({prompt_tokens:10,completion_tokens:3,prompt_tokens_details:{cached_tokens:4},completion_tokens_details:{reasoning_tokens:2}});
 const b=normalizeTurnUsage({input_tokens:5,output_tokens:7});
 assert.deepEqual(mergeTurnUsage(a,b),{inputTokens:15,outputTokens:10,cacheReadTokens:4,cacheWriteTokens:0,reasoningTokens:2});
});
test('navigation audit drops queries and identifies bot blocks',()=>{
 assert.equal(normalizeNavigationUrl('https://www.example.com/a?q=secret#x'),'https://www.example.com/a');
 assert.equal(visitedSiteHost('https://www.example.com/a'),'example.com');
 assert.equal(classifyBotBlockPage({url:'https://accounts.google.com/signin/rejected?x=1',title:''}).family,'google_signin_rejected');
 const action=toolAuditAction('browser_navigate',{url:'https://example.com/a?token=x'},'ok',12.4);
 assert.equal(action.url,'https://example.com/a');assert.equal(action.durationMs,12);
});
test('action auditor persists scrubbed action records as ndjson',async t=>{
 const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-audit-'));t.after(()=>fs.rm(root,{recursive:true,force:true}));
 const auditor=createActionAuditor({app:{getPath:()=>root}});
 auditor.record({agentId:'chief',turnId:'t1',action:{kind:'toolCall',toolName:'Read',status:'ok',durationMs:2}});
 await auditor.flush();const rows=(await fs.readFile(auditor.file,'utf8')).trim().split('\n').map(JSON.parse);
 assert.equal(rows[0].action.toolName,'Read');assert.equal(rows[0].agentId,'chief');
});
