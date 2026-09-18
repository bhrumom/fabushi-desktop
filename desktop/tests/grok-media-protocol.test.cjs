'use strict';
const test=require('node:test');const assert=require('node:assert/strict');
const {buildSandMediaUrl,parseSandMediaUrl,parseRangeHeader,createSandMediaHandler}=require('../electron/grok-media-protocol.cjs');

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
