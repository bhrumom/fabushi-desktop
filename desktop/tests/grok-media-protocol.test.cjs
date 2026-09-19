'use strict';
const test=require('node:test');const assert=require('node:assert/strict');
const fs=require('node:fs/promises');const os=require('node:os');const path=require('node:path');
const {buildSandMediaUrl,parseSandMediaUrl,parseRangeHeader,createSandMediaHandler}=require('../electron/grok-media-protocol.cjs');
const {createAttachmentGateway}=require('../electron/grok-attachment-gateway.cjs');

test('media protocol URL/range helpers mirror recovered scheme behavior',()=>{
 const url=buildSandMediaUrl('attachment id');assert.equal(url,'sand-media://attachment/attachment%20id');assert.equal(parseSandMediaUrl(url),'attachment id');
 assert.deepEqual(parseRangeHeader('bytes=10-19',100),{start:10,end:19});assert.deepEqual(parseRangeHeader('bytes=-10',100),{start:90,end:99});assert.equal(parseRangeHeader('items=1-2',100),null);
});
test('media protocol streams registered attachment chunks with range support',async()=>{
 const bytes=Buffer.from('0123456789');
 const handler=createSandMediaHandler({attachmentGateway:{readChunk:async(id,offset,length)=>id!=='a'?null:{data:length===0?Buffer.alloc(0):bytes.subarray(offset,offset+length),totalSize:bytes.length,mime:'video/mp4'}}});
 let response=await handler(new Request(buildSandMediaUrl('a'),{headers:{range:'bytes=2-5'}}));assert.equal(response.status,206);assert.equal(response.headers.get('content-range'),'bytes 2-5/10');assert.equal(await response.text(),'2345');
 response=await handler(new Request(buildSandMediaUrl('missing')));assert.equal(response.status,404);
});

test('attachment gateway supplies production media chunks',async()=>{
 const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-media-')),file=path.join(root,'clip.mp4');
 await fs.writeFile(file,Buffer.from('abcdefghij'));
 const gateway=createAttachmentGateway({app:{getPath:()=>root}});
 try{
  const record=await gateway.register(file);assert.equal(record.mime,'video/mp4');
  const meta=await gateway.readChunk(record.id,0,0);assert.equal(meta.totalSize,10);assert.equal(meta.mime,'video/mp4');
  const chunk=await gateway.readChunk(record.id,3,4);assert.equal(chunk.data.toString(),'defg');
  const handler=createSandMediaHandler({attachmentGateway:gateway});const response=await handler(new Request(buildSandMediaUrl(record.id),{headers:{range:'bytes=4-7'}}));
  assert.equal(response.status,206);assert.equal(await response.text(),'efgh');
 }finally{await gateway.dispose();await fs.rm(root,{recursive:true,force:true})}
});
