'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs/promises');
const os=require('node:os');
const path=require('node:path');
const {createAttachmentGateway}=require('../electron/grok-attachment-gateway.cjs');

test('attachment gateway persists opaque metadata and previews without exposing registration path',async t=>{
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-attachments-'));
  t.after(()=>fs.rm(root,{recursive:true,force:true}));
  const selected=path.join(root,'note.txt');
  await fs.writeFile(selected,'hello attachment','utf8');
  const gateway=createAttachmentGateway({app:{getPath:()=>root}});
  const descriptor=await gateway.register(selected);
  assert.equal(descriptor.name,'note.txt');
  assert.equal(descriptor.kind,'text');
  assert.equal('path' in descriptor,false);
  const resolved=await gateway.resolve([descriptor.id]);
  assert.equal(resolved[0].path,selected);
  const preview=await gateway.read(descriptor.id);
  assert.match(preview.dataUrl,/^data:text\/plain;base64,/);
  await gateway.dispose();
  const reopened=createAttachmentGateway({app:{getPath:()=>root}});
  assert.equal((await reopened.resolve([descriptor.id]))[0].path,selected);
  await reopened.dispose();
});
