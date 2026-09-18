'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');
const crypto=require('node:crypto');

const PREVIEW_LIMIT=25*1024*1024;
const MIME_BY_EXT={
  '.png':'image/png','.jpg':'image/jpeg','.jpeg':'image/jpeg','.gif':'image/gif','.webp':'image/webp',
  '.pdf':'application/pdf','.mp4':'video/mp4','.m4v':'video/x-m4v','.mov':'video/quicktime','.webm':'video/webm','.avi':'video/x-msvideo','.ogv':'video/ogg',
  '.txt':'text/plain','.md':'text/markdown','.json':'application/json',
  '.csv':'text/csv','.tsv':'text/tab-separated-values','.html':'text/html','.htm':'text/html',
  '.js':'text/javascript','.mjs':'text/javascript','.cjs':'text/javascript','.ts':'text/plain','.tsx':'text/plain',
  '.py':'text/plain','.rs':'text/plain','.swift':'text/plain','.kt':'text/plain','.xml':'application/xml'
};
function mimeFor(filePath){return MIME_BY_EXT[path.extname(filePath).toLowerCase()]||'application/octet-stream'}
function publicRecord(record){return{id:record.id,name:record.name,mime:record.mime,size:record.size,kind:record.kind,createdAt:record.createdAt}}
function createAttachmentGateway({app}){
  const indexFile=path.join(app.getPath('userData'),'grok-attachments.json');
  let index=null,writing=Promise.resolve();
  async function load(){
    if(index)return index;
    try{
      const parsed=JSON.parse(await fs.readFile(indexFile,'utf8'));
      index=parsed&&typeof parsed==='object'&&!Array.isArray(parsed)?parsed:{};
    }catch{index={}}
    return index;
  }
  async function save(){
    const data=JSON.stringify(await load(),null,2)+'\n';
    writing=writing.then(async()=>{await fs.mkdir(path.dirname(indexFile),{recursive:true});const tmp=indexFile+'.tmp';await fs.writeFile(tmp,data,{mode:0o600});await fs.rename(tmp,indexFile)});
    return writing;
  }
  async function register(filePath){
    const absolute=path.resolve(String(filePath||''));const stat=await fs.stat(absolute);
    if(!stat.isFile())throw Error('Attachment must be a file.');
    const mime=mimeFor(absolute),id=crypto.randomUUID(),record={id,path:absolute,name:path.basename(absolute),mime,size:stat.size,kind:mime.startsWith('image/')?'image':mime==='application/pdf'?'pdf':mime.startsWith('text/')||mime.includes('json')||mime.includes('xml')?'text':'file',createdAt:Date.now()};
    (await load())[id]=record;await save();return publicRecord(record);
  }
  async function resolve(ids){
    const store=await load(),rows=[];
    for(const raw of Array.isArray(ids)?ids:[]){
      const record=store[String(raw||'')];if(!record)continue;
      try{const stat=await fs.stat(record.path);if(stat.isFile())rows.push({...record,size:stat.size})}catch{}
    }
    return rows;
  }
  async function read(id){
    const rows=await resolve([id]),record=rows[0];if(!record)throw Error('Attachment is unavailable.');
    if(record.size>PREVIEW_LIMIT)throw Error('Attachment preview exceeds 25 MB.');
    const bytes=await fs.readFile(record.path);
    return{...publicRecord(record),dataUrl:`data:${record.mime};base64,${bytes.toString('base64')}`};
  }
  async function readChunk(id,offset=0,length=4*1024*1024){
    const rows=await resolve([id]),record=rows[0];if(!record)return null;
    const start=Math.max(0,Math.floor(Number(offset)||0)),requested=Math.max(0,Math.floor(Number(length)||0));
    if(start>=record.size||requested===0)return{data:Buffer.alloc(0),totalSize:record.size,mime:record.mime};
    const size=Math.min(requested,4*1024*1024,record.size-start),handle=await fs.open(record.path,'r');
    try{const data=Buffer.alloc(size),result=await handle.read(data,0,size,start);return{data:data.subarray(0,result.bytesRead),totalSize:record.size,mime:record.mime}}
    finally{await handle.close()}
  }
  async function dispose(){await writing.catch(()=>{})}
  return{register,resolve,read,readChunk,dispose};
}
module.exports={createAttachmentGateway,mimeFor,PREVIEW_LIMIT};
