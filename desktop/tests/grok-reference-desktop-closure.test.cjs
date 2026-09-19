'use strict';

const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');

const root=path.join(__dirname,'..','..');
const preloadSource=fs.readFileSync(path.join(__dirname,'..','electron','preload.cjs'),'utf8');
const closure=JSON.parse(fs.readFileSync(path.join(root,'reference','grok-bot-0.18','manifests','reconstruction','renderer-closure.json'),'utf8'));

function loadPreload(){
  const exposed={};
  const listeners=new Map();
  const ipcRenderer={
    invoke:async()=>null,
    on(channel,listener){
      const rows=listeners.get(channel)||[];
      rows.push(listener);listeners.set(channel,rows);
      return this;
    },
    off(channel,listener){
      const rows=listeners.get(channel)||[];
      listeners.set(channel,rows.filter(row=>row!==listener));
      return this;
    }
  };
  const contextBridge={exposeInMainWorld(name,value){exposed[name]=value}};
  const webFrame={getZoomFactor:()=>1};
  const sandbox={
    process,
    Buffer,
    URL,
    AbortSignal,
    AbortController,
    DOMException,
    queueMicrotask,
    setTimeout,
    clearTimeout,
    console,
    require(id){
      if(id==='electron')return{contextBridge,ipcRenderer,webFrame};
      return require(id);
    },
    module:{exports:{}},
    exports:{},
    __filename:path.join(__dirname,'..','electron','preload.cjs'),
    __dirname:path.join(__dirname,'..','electron')
  };
  sandbox.global=sandbox;
  const wrapper=vm.runInNewContext('(function(require,module,exports,__filename,__dirname){'+preloadSource+'\n})',sandbox,{filename:'preload.cjs'});
  wrapper(sandbox.require,sandbox.module,sandbox.module.exports,sandbox.__filename,sandbox.__dirname);
  return exposed;
}

function resolvePath(rootValue,pathValue){
  return String(pathValue).split('.').reduce((value,key)=>value==null?undefined:value[key],rootValue);
}
function collectFunctionLeafNames(value,out=new Set(),seen=new Set()){
  if(value==null)return out;
  if((typeof value!=='object'&&typeof value!=='function')||seen.has(value))return out;
  seen.add(value);
  for(const key of Object.keys(value)){
    let child;
    try{child=value[key]}catch{continue}
    if(typeof child==='function')out.add(key);
    if(child&&typeof child==='object')collectFunctionLeafNames(child,out,seen);
  }
  return out;
}

test('all official desktop bridge claims resolve to executable preload functions',()=>{
  const exposed=loadPreload();
  assert.ok(exposed.desktop&&typeof exposed.desktop==='object','desktop bridge was not exposed');
  const claimed=[...new Set(closure.ipcClaims.filter(row=>row.kind==='desktop-bridge').map(row=>row.value))].sort();
  assert.equal(claimed.length,95);
  const leafNames=collectFunctionLeafNames(exposed.desktop);
  const missing=[];
  for(const claim of claimed){
    const value=claim.includes('.')?resolvePath(exposed.desktop,claim):undefined;
    if(typeof value==='function')continue;
    const leaf=claim.split('.').at(-1);
    if(leafNames.has(leaf))continue;
    missing.push(claim);
  }
  assert.deepEqual(missing,[]);
});
