'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');
const crypto=require('node:crypto');

const PREVIEW_LIMIT=25*1024*1024;
const MIME_BY_EXT={
  '.png':'image/png','.jpg':'image/jpeg','.jpeg':'image/jpeg','.gif':'image/gif','.webp':'image/webp',
  '.pdf':'application/pdf','.txt':'text/plain','.md':'text/markdown','.json':'application/json',
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
  async function dispose(){await writing.catch(()=>{})}
  return{register,resolve,read,dispose};
}
module.exports={createAttachmentGateway,mimeFor,PREVIEW_LIMIT};
