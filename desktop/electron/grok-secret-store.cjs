'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');

function createSecretStore({app,safeStorage=null}){
  const file=path.join(app.getPath('userData'),'grok-mcp-secrets.json');
  const memory=new Map();
  let loaded=false,records={};

  function encryptedAvailable(){
    try{return Boolean(safeStorage?.isEncryptionAvailable?.())}catch{return false}
  }
  async function load(){
    if(loaded)return;
    loaded=true;
    try{
      const parsed=JSON.parse(await fs.readFile(file,'utf8'));
      if(parsed&&typeof parsed==='object'&&!Array.isArray(parsed))records=parsed;
    }catch{records={}}
  }
  async function persist(){
    if(!encryptedAvailable())return;
    await fs.mkdir(path.dirname(file),{recursive:true});
    const tmp=file+'.tmp';
    await fs.writeFile(tmp,JSON.stringify(records,null,2)+'\n',{mode:0o600});
    await fs.rename(tmp,file);
    await fs.chmod(file,0o600).catch(()=>{});
  }
  async function get(key){
    const id=String(key);
    if(memory.has(id))return memory.get(id);
    await load();
    const encoded=records[id];
    if(typeof encoded!=='string'||!encryptedAvailable())return null;
    try{
      const clear=safeStorage.decryptString(Buffer.from(encoded,'base64'));
      return JSON.parse(clear);
    }catch{return null}
  }
  async function set(key,value){
    const id=String(key);
    memory.set(id,value);
    if(!encryptedAvailable())return;
    await load();
    const encrypted=safeStorage.encryptString(JSON.stringify(value));
    records[id]=Buffer.from(encrypted).toString('base64');
    await persist();
  }
  async function remove(key){
    const id=String(key);
    memory.delete(id);
    await load();
    if(Object.prototype.hasOwnProperty.call(records,id)){
      delete records[id];
      await persist();
    }
  }
  async function move(from,to){
    const value=await get(from);
    if(value==null){await remove(from);return}
    await set(to,value);
    await remove(from);
  }
  return{get,set,remove,move,encryptedAvailable,filePath:file};
}

module.exports={createSecretStore};
