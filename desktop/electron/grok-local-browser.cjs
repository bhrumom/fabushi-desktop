'use strict';

const crypto=require('node:crypto');

const MAX_TEXT=30000,MAX_ELEMENTS=300;

function requireWebUrl(value){
  let url;
  try{url=new URL(String(value||'').trim())}catch{throw Error('Browser URL is invalid.')}
  if(url.protocol!=='https:'&&url.protocol!=='http:')throw Error('Browser URL must use HTTP(S).');
  if(url.username||url.password)throw Error('Do not put credentials in the browser URL.');
  return url.toString();
}

function createLocalBrowserRuntime({BrowserWindow}){
  const sessions=new Map();

  function session(agentId){
    let current=sessions.get(agentId);
    if(current&&!current.win.isDestroyed())return current;
    const win=new BrowserWindow({
      show:false,width:1280,height:900,backgroundColor:'#ffffff',
      webPreferences:{contextIsolation:true,nodeIntegration:false,sandbox:true,spellcheck:false}
    });
    win.webContents.setWindowOpenHandler(({url})=>{
      try{void win.loadURL(requireWebUrl(url))}catch{}
      return{action:'deny'};
    });
    current={win,generation:crypto.randomUUID(),stateId:null,url:null,title:null};
    sessions.set(agentId,current);
    win.on('closed',()=>{if(sessions.get(agentId)===current)sessions.delete(agentId)});
    return current;
  }

  async function snapshot(agentId){
    const current=session(agentId),win=current.win;
    const script="(()=>{const clean=v=>String(v??'').replace(/\\\\s+/g,' ').trim();const nodes=[...document.querySelectorAll('a,button,input,textarea,select,[role=\"button\"],[role=\"link\"],[tabindex]')].slice(0,"+MAX_ELEMENTS+");const prefix='fabushi-'+Math.random().toString(36).slice(2,9)+'-';const elements=nodes.map((el,index)=>{const ref=prefix+index;el.setAttribute('data-fabushi-ref',ref);const input=el instanceof HTMLInputElement;const type=input?String(el.type||'text').toLowerCase():'';return{ref,tag:el.tagName.toLowerCase(),role:el.getAttribute('role')||'',type,text:clean(el.innerText||el.textContent||'').slice(0,240),ariaLabel:clean(el.getAttribute('aria-label')||'').slice(0,160),placeholder:clean(el.getAttribute('placeholder')||'').slice(0,160),href:el instanceof HTMLAnchorElement?el.href:'',value:input&&type!=='password'?String(el.value||'').slice(0,240):el instanceof HTMLTextAreaElement?String(el.value||'').slice(0,240):'',disabled:Boolean(el.disabled)}});return{url:location.href,title:document.title,body:clean(document.body?.innerText||'').slice(0,"+MAX_TEXT+"),elements}})()";
    const page=await win.webContents.executeJavaScript(script,true);
    const canonical=JSON.stringify({
      url:page.url,title:page.title,
      elements:page.elements.map(element=>[element.ref,element.tag,element.role,element.type,element.text,element.href,element.disabled])
    });
    const stateId=crypto.createHash('sha256').update(canonical).digest('hex');
    current.stateId=stateId;current.url=page.url;current.title=page.title;
    return{viewId:'main',stateId,url:page.url,title:page.title,body:page.body,elements:page.elements,generation:current.generation};
  }

  async function ensureState(agentId,stateId){
    if(!stateId)throw Error('A browser stateId from browser_snapshot is required for this action.');
    const latest=await snapshot(agentId);
    if(latest.stateId!==stateId)throw Error('The page changed after review; take a fresh browser_snapshot and retry the action.');
    return latest;
  }

  async function navigate({agentId,url}){
    const current=session(agentId);
    await current.win.loadURL(requireWebUrl(url));
    return await snapshot(agentId);
  }

  async function screenshot(agentId){
    const current=session(agentId),image=await current.win.webContents.capturePage();
    return{
      text:'Captured browser view.',
      display:{kind:'image',dataUrl:'data:image/png;base64,'+image.toPNG().toString('base64')},
      state:await snapshot(agentId)
    };
  }

  async function action({agentId,name,args}){
    const current=session(agentId),win=current.win;
    if(name==='browser_snapshot')return{text:JSON.stringify(await snapshot(agentId),null,2)};
    if(name==='browser_screenshot')return await screenshot(agentId);
    if(name==='browser_navigate')return{text:JSON.stringify(await navigate({agentId,url:args.url}),null,2)};
    if(name==='browser_scroll'){
      const dx=Number(args.deltaX||0),dy=Number(args.deltaY||600);
      await win.webContents.executeJavaScript('window.scrollBy('+String(Number.isFinite(dx)?dx:0)+','+String(Number.isFinite(dy)?dy:600)+')',true);
      return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }

    await ensureState(agentId,String(args.stateId||''));

    if(name==='browser_click'){
      const ref=String(args.ref||'');
      if(!ref)throw Error('browser_click requires ref from browser_snapshot.');
      const selector=JSON.stringify('[data-fabushi-ref="'+ref.replace(/["\\]/g,'')+'"]');
      const result=await win.webContents.executeJavaScript("(()=>{const el=document.querySelector("+selector+");if(!el)return{ok:false};el.scrollIntoView({block:'center',inline:'center'});el.click();return{ok:true}})()",true);
      if(!result?.ok)throw Error('Browser element no longer exists; take a fresh snapshot.');
      await new Promise(resolve=>setTimeout(resolve,250));
      return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }

    if(name==='browser_type'){
      const ref=String(args.ref||''),value=String(args.text??'');
      if(!ref)throw Error('browser_type requires ref from browser_snapshot.');
      if(value.length>8000)throw Error('Browser text is too long.');
      const selector=JSON.stringify('[data-fabushi-ref="'+ref.replace(/["\\]/g,'')+'"]');
      const payload=JSON.stringify(value),clear=args.clear===false?'false':'true',submit=args.submit===true?'true':'false';
      const script="(()=>{const el=document.querySelector("+selector+");if(!el)return{ok:false};el.focus();if("+clear+"){if('value'in el)el.value=''}const value="+payload+";if('value'in el)el.value=String(el.value||'')+value;else el.textContent=String(el.textContent||'')+value;el.dispatchEvent(new Event('input',{bubbles:true}));el.dispatchEvent(new Event('change',{bubbles:true}));if("+submit+"){const form=el.form||el.closest('form');if(form?.requestSubmit)form.requestSubmit();else form?.submit?.()}return{ok:true}})()";
      const result=await win.webContents.executeJavaScript(script,true);
      if(!result?.ok)throw Error('Browser element no longer exists; take a fresh snapshot.');
      await new Promise(resolve=>setTimeout(resolve,100));
      return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }

    if(name==='browser_key'){
      const key=String(args.key||'');
      if(!key||key.length>64)throw Error('Browser key is invalid.');
      win.webContents.focus();
      win.webContents.sendInputEvent({type:'keyDown',keyCode:key});
      win.webContents.sendInputEvent({type:'keyUp',keyCode:key});
      await new Promise(resolve=>setTimeout(resolve,100));
      return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }

    throw Error('Unknown browser action: '+name);
  }

  function definitions(enabled){
    if(!enabled.has('browser'))return[];
    const fn=(name,description,properties,required=[])=>({type:'function',function:{name,description,parameters:{type:'object',properties,required,additionalProperties:false}}});
    return[
      fn('browser_snapshot','Inspect the current local browser page and return text plus stable element refs and a stateId.',{}),
      fn('browser_screenshot','Capture the current local browser page as an image.',{}),
      fn('browser_navigate','Navigate the local browser to an HTTP(S) URL.',{url:{type:'string'}},['url']),
      fn('browser_click','Click an element ref from browser_snapshot. The stateId must still match after user review.',{ref:{type:'string'},stateId:{type:'string'},purpose:{type:'string'}},['ref','stateId','purpose']),
      fn('browser_type','Type into an element ref from browser_snapshot. The stateId must still match after user review.',{ref:{type:'string'},stateId:{type:'string'},text:{type:'string'},clear:{type:'boolean'},submit:{type:'boolean'}},['ref','stateId','text']),
      fn('browser_key','Send a key to the local browser. The stateId must still match after user review.',{stateId:{type:'string'},key:{type:'string'}},['stateId','key']),
      fn('browser_scroll','Scroll the current local browser page.',{deltaX:{type:'number'},deltaY:{type:'number'}})
    ];
  }

  const readOnly=new Set(['browser_snapshot','browser_screenshot','browser_scroll']);
  return{
    definitions,
    isTool:name=>String(name).startsWith('browser_'),
    isMutation:name=>!readOnly.has(name),
    execute:({agentId,name,args})=>action({agentId,name,args}),
    disposeAgent(agentId){
      const current=sessions.get(agentId);
      if(current&&!current.win.isDestroyed())current.win.destroy();
      sessions.delete(agentId);
    }
  };
}

module.exports={createLocalBrowserRuntime};
