'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs/promises');
const os=require('node:os');
const path=require('node:path');
const {TOOL_OUTPUT_THRESHOLD_BYTES,MAX_OUTPUT_FILE_SIZE,createOutputSpiller}=require('../electron/grok-output-spill.cjs');

test('large tool output spills above reference 40k threshold and caps the local artifact at 1,000,000 characters',async t=>{
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-output-spill-'));
  t.after(()=>fs.rm(root,{recursive:true,force:true}));
  const spiller=createOutputSpiller({app:{getPath(name){assert.equal(name,'userData');return root}}});
  const small='x'.repeat(TOOL_OUTPUT_THRESHOLD_BYTES-1);
  const inline=await spiller.spillText(small,{agentId:'chief',toolCallId:'small'});
  assert.equal(inline.spilled,false);
  assert.equal(inline.text,small);

  const large='line\n'.repeat(Math.ceil((MAX_OUTPUT_FILE_SIZE+5000)/5));
  const spilled=await spiller.spillText(large,{agentId:'chief',toolCallId:'large',lead:'Tool output'});
  assert.equal(spilled.spilled,true);
  assert.ok(spilled.outputLocation.filePath.startsWith(path.join(root,'agent-tools','chief')));
  assert.equal(spilled.outputLocation.toolCallId,'large');
  assert.equal(spilled.outputLocation.truncated,true);
  assert.ok(spilled.outputLocation.originalSizeBytes>TOOL_OUTPUT_THRESHOLD_BYTES);
  assert.match(spilled.text,/Tool output written to file:/);
  const materialized=await fs.readFile(spilled.outputLocation.filePath,'utf8');
  assert.equal(materialized.length,MAX_OUTPUT_FILE_SIZE);
  const stat=await fs.stat(spilled.outputLocation.filePath);
  assert.equal((stat.mode&0o077),0);
});
