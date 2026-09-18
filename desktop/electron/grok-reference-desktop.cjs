'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');
const crypto=require('node:crypto');
const {createSecretStore}=require('./grok-secret-store.cjs');
const {mimeFor}=require('./grok-attachment-gateway.cjs');

const MAX_FILE=25*1024*1024,MAX_VIDEO=200*1024*1024;
const VIDEO_EXTS=new Set(['.m4v','.mov','.mp4','.ogv','.webm']);

function cleanName(value){
  const base=path.basename(String(value||'file')).replace(/[\x00-\x1f<>:"/\\|?*]+/g,'-').trim();
  return (base||'file').slice(0,180);
}
function createReferenceDesktop({app,BrowserWindow,shell,dialog,safeStorage,nativeTheme,getWindow}){
  const prefsFile=path.join(app.getPath('userData'),'grok-reference-desktop.json');
  const stageRoot=path.join(app.getPath('userData'),'grok-reference-staged');
  const committedRoot=path.join(app.getPath('userData'),'grok-reference-attachments');
  const secrets=createSecretStore({app,safeStorage});
  let prefs=null;
  async function loadPrefs(){
    if(prefs)return prefs;
    try{const value=JSON.parse(await fs.readFile(prefsFile,'utf8'));prefs=value&&typeof value==='object'&&!Array.isArray(value)?value:{}}
    catch{prefs={}}
    prefs.clientPersistence=prefs.clientPersistence&&typeof prefs.clientPersistence==='object'&&!Array.isArray(prefs.clientPersistence)?prefs.clientPersistence:{};
    prefs.timeZoneOverride=typeof prefs.timeZoneOverride==='string'?prefs.timeZoneOverride:null;
    prefs.theme=['system','light','dark'].includes(prefs.theme)?prefs.theme:'system';
    return prefs;
  }
  async function savePrefs(){
    await fs.mkdir(path.dirname(prefsFile),{recursive:true,mode:0o700});
    const tmp=prefsFile+'.tmp';await fs.writeFile(tmp,JSON.stringify(await loadPrefs(),null,2)+'\n',{mode:0o600});await fs.rename(tmp,prefsFile);
  }
  async function pathInfo(filePath){
    const absolute=path.resolve(String(filePath||'')),stat=await fs.stat(absolute);
    if(!stat.isFile())throw Error('Attachment must be a file.');
    return{absolute,stat,mime:mimeFor(absolute)};
  }
  async function call(method,args={}){
    switch(method){
      case'get-window-state':{const win=getWindow();return{isFullscreen:Boolean(win?.isFullScreen?.()),isMaximized:Boolean(win?.isMaximized?.())}}
      case'window-control':{
        const win=getWindow();if(!win)return null;
        if(args.action==='minimize')win.minimize();
        else if(args.action==='toggle-maximize')win.isMaximized()?win.unmaximize():win.maximize();
        else if(args.action==='close')win.close();
        else if(args.action==='resize-width'){const [w,h]=win.getSize();const width=Math.max(900,w+Number(args.deltaWidth||0));win.setSize(width,h);return width}
        else if(args.action==='overlay-tone'&&process.platform==='win32')win.setTitleBarOverlay?.({color:args.isOverlayTone?'#111216':'#000000',symbolColor:'#ffffff'});
        return null;
      }
      case'theme-get':{
        const p=await loadPrefs(),preference=p.theme||'system';
        if(nativeTheme)nativeTheme.themeSource=preference;
        return{preference,resolved:nativeTheme?.shouldUseDarkColors?'dark':'light'};
      }
      case'theme-set':{
        const p=await loadPrefs(),value=['system','light','dark'].includes(args.preference)?args.preference:'system';
        p.theme=value;if(nativeTheme)nativeTheme.themeSource=value;await savePrefs();
        return{preference:value,resolved:nativeTheme?.shouldUseDarkColors?'dark':'light'};
      }
      case'persistence-read':return String((await loadPrefs()).clientPersistence[String(args.key||'')]??'')||null;
      case'persistence-write':{const p=await loadPrefs();p.clientPersistence[String(args.key||'')]=String(args.value??'');await savePrefs();return}
      case'persistence-remove':{const p=await loadPrefs();delete p.clientPersistence[String(args.key||'')];await savePrefs();return}
      case'persistence-list':{const prefix=String(args.prefix||'');return Object.keys((await loadPrefs()).clientPersistence).filter(k=>k.startsWith(prefix))}
      case'persistence-migrate':{
        const p=await loadPrefs();let changed=false;
        for(const row of Array.isArray(args.entries)?args.entries:[]){if(row&&typeof row.key==='string'&&typeof row.value==='string'&&p.clientPersistence[row.key]===undefined){p.clientPersistence[row.key]=row.value;changed=true}}
        if(changed)await savePrefs();return changed;
      }
      case'stage-attachment':{
        const name=cleanName(args.filename),bytes=Buffer.from(args.bytes||[]);
        if(bytes.length===0)return{ok:false,reason:'empty'};
        const max=VIDEO_EXTS.has(path.extname(name).toLowerCase())?MAX_VIDEO:MAX_FILE;if(bytes.length>max)return{ok:false,reason:'too-large'};
        await fs.mkdir(stageRoot,{recursive:true,mode:0o700});const filePath=path.join(stageRoot,crypto.randomUUID()+'-'+name);
        await fs.writeFile(filePath,bytes,{mode:0o600});return{ok:true,path:filePath};
      }
      case'commit-staged':{
        const paths=Array.isArray(args.paths)?args.paths:[],names=Array.isArray(args.filenames)?args.filenames:[];
        await fs.mkdir(committedRoot,{recursive:true,mode:0o700});const out=[];
        for(let i=0;i<paths.length;i++){
          const source=path.resolve(String(paths[i]||''));if(!source.startsWith(path.resolve(stageRoot)+path.sep))throw Error('Attachment is outside the staging area.');
          const target=path.join(committedRoot,crypto.randomUUID()+'-'+cleanName(names[i]||path.basename(source)));
          await fs.rename(source,target).catch(async()=>{await fs.copyFile(source,target);await fs.unlink(source).catch(()=>{})});out.push(target);
        }
        return out;
      }
      case'discard-staged':{
        const source=path.resolve(String(args.path||''));if(source.startsWith(path.resolve(stageRoot)+path.sep))await fs.unlink(source).catch(()=>{});return;
      }
      case'resolve-media':{
        try{
          const {absolute,stat,mime}=await pathInfo(args.path);
          if(stat.size>MAX_FILE)return null;
          const bytes=await fs.readFile(absolute),dataUrl='data:'+mime+';base64,'+bytes.toString('base64');
          if(mime.startsWith('image/'))return{kind:'image',dataUrl,width:null,height:null};
          if(mime.startsWith('video/'))return{kind:'video',src:dataUrl,width:null,height:null};
          if(mime.startsWith('audio/'))return{kind:'audio',src:dataUrl};
          return null;
        }catch{return null}
      }
      case'read-text':{
        try{const {absolute,stat,mime}=await pathInfo(args.path);if(stat.size>MAX_FILE)return{kind:'binary',bytes:stat.size};const bytes=await fs.readFile(absolute);if(!mime.startsWith('text/')&&!mime.includes('json')&&!mime.includes('xml'))return{kind:'binary',bytes:stat.size};return{kind:'text',text:bytes.toString('utf8'),truncated:false,bytes:stat.size}}catch{return null}
      }
      case'read-bytes':{
        try{const {absolute,stat}=await pathInfo(args.path),max=Math.max(0,Number(args.maxBytes)||MAX_FILE);if(stat.size>max)return{kind:'too-large',size:stat.size};return{kind:'bytes',bytes:new Uint8Array(await fs.readFile(absolute))}}catch{return null}
      }
      case'download':{
        const {absolute}=await pathInfo(args.path);const selected=await dialog.showSaveDialog(getWindow(),{defaultPath:cleanName(args.suggestedName||path.basename(absolute))});
        if(selected.canceled||!selected.filePath)return false;await fs.copyFile(absolute,selected.filePath);return true;
      }
      case'link-metadata':{
        const raw=String(args.url||'');let url;try{url=new URL(raw)}catch{return null}
        if(url.protocol!=='https:')return{url:url.toString(),title:url.hostname};
        try{const response=await fetch(url,{headers:{accept:'text/html'},signal:AbortSignal.timeout(5000)});const html=(await response.text()).slice(0,250000);const title=/<title[^>]*>([^<]+)<\/title>/i.exec(html)?.[1]?.trim();return{url:url.toString(),title:title||url.hostname}}catch{return{url:url.toString(),title:url.hostname}}
      }
      case'time-zone-get':{const p=await loadPrefs();return{detectedTimeZone:Intl.DateTimeFormat().resolvedOptions().timeZone||null,overrideTimeZone:p.timeZoneOverride}}
      case'time-zone-set':{const p=await loadPrefs();p.timeZoneOverride=args.timeZone==null?null:String(args.timeZone);await savePrefs();return{detectedTimeZone:Intl.DateTimeFormat().resolvedOptions().timeZone||null,overrideTimeZone:p.timeZoneOverride}}
      case'secrets-list':{
        const p=await loadPrefs(),keys=Array.isArray(p.secretKeys)?p.secretKeys:[];return{keys,isPersistent:secrets.encryptedAvailable()};
      }
      case'secrets-reveal':{const value=await secrets.get('ref-secret:'+String(args.key||''));return typeof value==='string'?value:null}
      case'secrets-upsert':{
        const p=await loadPrefs(),keys=new Set(Array.isArray(p.secretKeys)?p.secretKeys:[]);
        for(const [key,value] of Object.entries(args.entries&&typeof args.entries==='object'?args.entries:{})){await secrets.set('ref-secret:'+key,String(value));keys.add(key)}
        p.secretKeys=[...keys].sort();await savePrefs();return{synced:true};
      }
      case'secrets-remove':{
        const p=await loadPrefs(),keys=new Set(Array.isArray(p.secretKeys)?p.secretKeys:[]);
        for(const key of Array.isArray(args.keys)?args.keys:[]){await secrets.remove('ref-secret:'+String(key));keys.delete(String(key))}
        p.secretKeys=[...keys].sort();await savePrefs();return{synced:true};
      }
      default:throw Error('Unsupported desktop bridge method: '+method);
    }
  }
  return{call};
}
module.exports={createReferenceDesktop};
