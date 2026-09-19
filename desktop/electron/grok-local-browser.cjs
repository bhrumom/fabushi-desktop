'use strict';

const crypto=require('node:crypto');

const MAX_TEXT=30000,MAX_ELEMENTS=300,MAX_CDP_PARAMS=2000;

function requireWebUrl(value){
  let url;
  try{url=new URL(String(value||'').trim())}catch{throw Error('Browser URL is invalid.')}
  if(url.protocol!=='https:'&&url.protocol!=='http:')throw Error('Browser URL must use HTTP(S).');
  if(url.username||url.password)throw Error('Do not put credentials in the browser URL.');
  return url.toString();
}
function int(value,label,{min=0,max=100000}={}){
  const n=Number(value);if(!Number.isFinite(n))throw Error(label+' must be a finite number.');
  return Math.max(min,Math.min(max,Math.round(n)));
}
function cleanRef(value){
  const ref=String(value||'').trim();
  if(!ref)throw Error('Browser element ref is required.');
  return ref.replace(/["\\]/g,'').slice(0,200);
}

function createLocalBrowserRuntime({BrowserWindow}){
  const sessions=new Map();

  function installWindowPolicy(agentId,win){
    win.webContents.setWindowOpenHandler(({url})=>{
      try{const tab=createTab(agentId,url);void tab.win.loadURL(requireWebUrl(url)).catch(()=>{})}catch{}
      return{action:'deny'};
    });
  }
  function createTab(agentId,url=null){
    const win=new BrowserWindow({
      show:false,width:1280,height:900,backgroundColor:'#ffffff',
      webPreferences:{contextIsolation:true,nodeIntegration:false,sandbox:true,spellcheck:false}
    });
    installWindowPolicy(agentId,win);
    const tab={win,generation:crypto.randomUUID(),refPrefix:'fabushi-'+crypto.randomUUID().slice(0,8)+'-',stateId:null,url:'about:blank',title:'',createdAt:Date.now()};
    const state=sessions.get(agentId)||{tabs:[],activeIndex:0};
    state.tabs.push(tab);state.activeIndex=state.tabs.length-1;sessions.set(agentId,state);
    win.on('closed',()=>{
      const live=sessions.get(agentId);if(!live)return;
      const index=live.tabs.indexOf(tab);if(index<0)return;
      live.tabs.splice(index,1);live.activeIndex=Math.max(0,Math.min(live.activeIndex,live.tabs.length-1));
      if(live.tabs.length===0)sessions.delete(agentId);
    });
    if(url)void win.loadURL(requireWebUrl(url)).catch(()=>{});
    return tab;
  }
  function state(agentId){
    let current=sessions.get(agentId);
    if(current){
      current.tabs=current.tabs.filter(tab=>!tab.win.isDestroyed());
      current.activeIndex=Math.max(0,Math.min(current.activeIndex,current.tabs.length-1));
      if(current.tabs.length)return current;
      sessions.delete(agentId);
    }
    createTab(agentId);
    return sessions.get(agentId);
  }
  function activeTab(agentId){
    const current=state(agentId);
    return current.tabs[current.activeIndex];
  }
  function tabRows(agentId){
    const current=state(agentId);
    return current.tabs.map((tab,index)=>({index,active:index===current.activeIndex,url:tab.url||'about:blank',title:tab.title||'',generation:tab.generation}));
  }

  async function snapshot(agentId){
    const current=state(agentId),tab=activeTab(agentId),win=tab.win;
    const script="(()=>{const clean=v=>String(v??'').replace(/\\s+/g,' ').trim();const nodes=[...document.querySelectorAll('a,button,input,textarea,select,[role=\"button\"],[role=\"link\"],[tabindex]')].slice(0,"+MAX_ELEMENTS+");const prefix="+JSON.stringify(tab.refPrefix)+";const elements=nodes.map((el,index)=>{const ref=prefix+index;el.setAttribute('data-fabushi-ref',ref);const input=el instanceof HTMLInputElement;const type=input?String(el.type||'text').toLowerCase():'';const rect=el.getBoundingClientRect();return{ref,tag:el.tagName.toLowerCase(),role:el.getAttribute('role')||'',type,text:clean(el.innerText||el.textContent||'').slice(0,240),ariaLabel:clean(el.getAttribute('aria-label')||'').slice(0,160),placeholder:clean(el.getAttribute('placeholder')||'').slice(0,160),href:el instanceof HTMLAnchorElement?el.href:'',value:input&&type!=='password'?String(el.value||'').slice(0,240):el instanceof HTMLTextAreaElement?String(el.value||'').slice(0,240):'',disabled:Boolean(el.disabled),bounds:{x:Math.round(rect.x),y:Math.round(rect.y),width:Math.round(rect.width),height:Math.round(rect.height)}}});return{url:location.href,title:document.title,body:clean(document.body?.innerText||'').slice(0,"+MAX_TEXT+"),elements}})()";
    const page=await win.webContents.executeJavaScript(script,true);
    const canonical=JSON.stringify({
      url:page.url,title:page.title,
      elements:page.elements.map(element=>[element.ref,element.tag,element.role,element.type,element.text,element.href,element.disabled,element.bounds])
    });
    const stateId=crypto.createHash('sha256').update(canonical).digest('hex');
    tab.stateId=stateId;tab.url=page.url;tab.title=page.title;
    return{viewId:'main',stateId,url:page.url,title:page.title,body:page.body,elements:page.elements,generation:tab.generation,tabs:tabRows(agentId),activeTabIndex:current.activeIndex};
  }

  async function ensureState(agentId,stateId){
    if(!stateId)throw Error('A browser stateId from browser_snapshot is required for this action.');
    const latest=await snapshot(agentId);
    if(latest.stateId!==stateId)throw Error('The page changed after review; take a fresh browser_snapshot and retry the action.');
    return latest;
  }
  async function navigate({agentId,url}){
    const tab=activeTab(agentId);
    await tab.win.loadURL(requireWebUrl(url));
    return await snapshot(agentId);
  }
  async function screenshot(agentId){
    const tab=activeTab(agentId),image=await tab.win.webContents.capturePage();
    return{text:'Captured browser view.',display:{kind:'image',dataUrl:'data:image/png;base64,'+image.toPNG().toString('base64')},state:await snapshot(agentId)};
  }
  async function boundingBox(agentId,ref){
    const tab=activeTab(agentId),selector=JSON.stringify('[data-fabushi-ref="'+cleanRef(ref)+'"]');
    const result=await tab.win.webContents.executeJavaScript("(()=>{const el=document.querySelector("+selector+");if(!el)return null;const r=el.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height,centerX:r.x+r.width/2,centerY:r.y+r.height/2}})()",true);
    if(!result)throw Error('Browser element no longer exists; take a fresh snapshot.');
    return{text:JSON.stringify(result,null,2)};
  }
  async function tabsAction(agentId,args){
    const current=state(agentId),action=String(args.action||args.tabsAction||'list').toLowerCase();
    if(action==='list')return{text:JSON.stringify({tabs:tabRows(agentId),activeTabIndex:current.activeIndex},null,2)};
    if(action==='new'){
      const tab=createTab(agentId,args.url?requireWebUrl(args.url):null);
      if(args.url)await tab.win.loadURL(requireWebUrl(args.url));
      return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }
    if(action==='select'){
      const index=int(args.tabIndex,'tabIndex',{min:0,max:current.tabs.length-1});
      if(!current.tabs[index])throw Error('Browser tab does not exist.');
      current.activeIndex=index;return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }
    if(action==='close'){
      if(args.stateId)await ensureState(agentId,String(args.stateId));
      const index=args.tabIndex==null?current.activeIndex:int(args.tabIndex,'tabIndex',{min:0,max:current.tabs.length-1});
      const tab=current.tabs[index];if(!tab)throw Error('Browser tab does not exist.');
      current.tabs.splice(index,1);if(!tab.win.isDestroyed())tab.win.destroy();
      current.activeIndex=Math.max(0,Math.min(current.activeIndex,current.tabs.length-1));
      if(current.tabs.length===0)createTab(agentId);
      return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }
    throw Error('Unknown browser tabs action: '+action);
  }
  async function cdp(agentId,args){
    await ensureState(agentId,String(args.stateId||''));
    const method=String(args.method||args.cdpMethod||'').trim();
    if(!method||method.length>200)throw Error('CDP method is invalid.');
    const raw=args.params??args.cdpParams??{};
    let params=raw;
    if(typeof raw==='string'){
      if(raw.length>MAX_CDP_PARAMS)throw Error('CDP params exceed 2,000 characters.');
      try{params=raw.trim()?JSON.parse(raw):{}}catch{throw Error('CDP params must be valid JSON.')}
    }
    if(!params||typeof params!=='object'||Array.isArray(params))throw Error('CDP params must be an object.');
    const dbg=activeTab(agentId).win.webContents.debugger;
    let attached=false;
    if(!dbg.isAttached()){dbg.attach('1.3');attached=true}
    try{
      const result=await dbg.sendCommand(method,params);
      return{text:JSON.stringify(result??{},null,2)};
    }finally{if(attached&&dbg.isAttached())dbg.detach()}
  }

  async function action({agentId,name,args={}}){
    const current=state(agentId),tab=activeTab(agentId),win=tab.win;
    if(name==='browser_snapshot')return{text:JSON.stringify(await snapshot(agentId),null,2)};
    if(name==='browser_screenshot')return await screenshot(agentId);
    if(name==='browser_navigate')return{text:JSON.stringify(await navigate({agentId,url:args.url}),null,2)};
    if(name==='browser_tabs')return await tabsAction(agentId,args);
    if(name==='browser_get_bounding_box')return await boundingBox(agentId,args.ref);
    if(name==='browser_highlight'){
      const selector=JSON.stringify('[data-fabushi-ref="'+cleanRef(args.ref)+'"]');
      const result=await win.webContents.executeJavaScript("(()=>{const el=document.querySelector("+selector+");if(!el)return false;const old=el.style.outline;el.style.outline='3px solid #ff4d00';setTimeout(()=>{el.style.outline=old},1500);return true})()",true);
      if(!result)throw Error('Browser element no longer exists; take a fresh snapshot.');
      return{text:'Highlighted browser element.'};
    }
    if(name==='browser_scroll'){
      const dx=Number(args.deltaX||0),dy=Number(args.deltaY||600);
      await win.webContents.executeJavaScript('window.scrollBy('+String(Number.isFinite(dx)?dx:0)+','+String(Number.isFinite(dy)?dy:600)+')',true);
      return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }
    if(name==='browser_cdp')return await cdp(agentId,args);

    await ensureState(agentId,String(args.stateId||''));

    if(name==='browser_click'){
      const selector=JSON.stringify('[data-fabushi-ref="'+cleanRef(args.ref)+'"]');
      const result=await win.webContents.executeJavaScript("(()=>{const el=document.querySelector("+selector+");if(!el)return{ok:false};el.scrollIntoView({block:'center',inline:'center'});el.click();return{ok:true}})()",true);
      if(!result?.ok)throw Error('Browser element no longer exists; take a fresh snapshot.');
      await new Promise(resolve=>setTimeout(resolve,250));return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }
    if(name==='browser_mouse_click_xy'){
      const x=int(args.x,'x',{max:100000}),y=int(args.y,'y',{max:100000});
      win.webContents.sendInputEvent({type:'mouseDown',x,y,button:String(args.button||'left'),clickCount:args.doubleClick===true?2:1});
      win.webContents.sendInputEvent({type:'mouseUp',x,y,button:String(args.button||'left'),clickCount:args.doubleClick===true?2:1});
      await new Promise(resolve=>setTimeout(resolve,150));return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }
    if(name==='browser_hover'){
      const selector=JSON.stringify('[data-fabushi-ref="'+cleanRef(args.ref)+'"]');
      const result=await win.webContents.executeJavaScript("(()=>{const el=document.querySelector("+selector+");if(!el)return false;el.scrollIntoView({block:'center',inline:'center'});el.dispatchEvent(new MouseEvent('mouseover',{bubbles:true}));el.dispatchEvent(new MouseEvent('mouseenter',{bubbles:true}));return true})()",true);
      if(!result)throw Error('Browser element no longer exists; take a fresh snapshot.');
      return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }
    if(name==='browser_type'){
      const ref=cleanRef(args.ref),value=String(args.text??'');
      if(value.length>8000)throw Error('Browser text is too long.');
      const selector=JSON.stringify('[data-fabushi-ref="'+ref+'"]');
      const payload=JSON.stringify(value),clear=args.clear===false?'false':'true',submit=args.submit===true?'true':'false';
      const script="(()=>{const el=document.querySelector("+selector+");if(!el)return{ok:false};el.focus();if("+clear+"){if('value'in el)el.value=''}const value="+payload+";if('value'in el)el.value=String(el.value||'')+value;else el.textContent=String(el.textContent||'')+value;el.dispatchEvent(new Event('input',{bubbles:true}));el.dispatchEvent(new Event('change',{bubbles:true}));if("+submit+"){const form=el.form||el.closest('form');if(form?.requestSubmit)form.requestSubmit();else form?.submit?.()}return{ok:true}})()";
      const result=await win.webContents.executeJavaScript(script,true);
      if(!result?.ok)throw Error('Browser element no longer exists; take a fresh snapshot.');
      await new Promise(resolve=>setTimeout(resolve,100));return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }
    if(name==='browser_select'){
      const selector=JSON.stringify('[data-fabushi-ref="'+cleanRef(args.ref)+'"]'),values=Array.isArray(args.values)?args.values.map(String):[String(args.value??'')];
      if(values.join(',').length>2000)throw Error('Browser select values are too long.');
      const payload=JSON.stringify(values);
      const result=await win.webContents.executeJavaScript("(()=>{const el=document.querySelector("+selector+");if(!(el instanceof HTMLSelectElement))return false;const values="+payload+";for(const option of el.options)option.selected=values.includes(option.value)||values.includes(option.text);el.dispatchEvent(new Event('input',{bubbles:true}));el.dispatchEvent(new Event('change',{bubbles:true}));return true})()",true);
      if(!result)throw Error('Browser select element no longer exists.');
      return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }
    if(name==='browser_drag'){
      const sourceSelector=JSON.stringify('[data-fabushi-ref="'+cleanRef(args.sourceRef||args.ref)+'"]');
      const targetSelector=args.targetRef?JSON.stringify('[data-fabushi-ref="'+cleanRef(args.targetRef)+'"]'):null;
      const script="(()=>{const source=document.querySelector("+sourceSelector+");const target="+(targetSelector?"document.querySelector("+targetSelector+")":"document.elementFromPoint("+String(Number(args.targetX)||0)+","+String(Number(args.targetY)||0)+")")+";if(!source||!target)return false;const dt=new DataTransfer();source.dispatchEvent(new DragEvent('dragstart',{bubbles:true,dataTransfer:dt}));target.dispatchEvent(new DragEvent('dragenter',{bubbles:true,dataTransfer:dt}));target.dispatchEvent(new DragEvent('dragover',{bubbles:true,dataTransfer:dt}));target.dispatchEvent(new DragEvent('drop',{bubbles:true,dataTransfer:dt}));source.dispatchEvent(new DragEvent('dragend',{bubbles:true,dataTransfer:dt}));return true})()";
      const result=await win.webContents.executeJavaScript(script,true);
      if(!result)throw Error('Browser drag source or target no longer exists.');
      return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }
    if(name==='browser_key'){
      const key=String(args.key||'');
      if(!key||key.length>64)throw Error('Browser key is invalid.');
      win.webContents.focus();win.webContents.sendInputEvent({type:'keyDown',keyCode:key});win.webContents.sendInputEvent({type:'keyUp',keyCode:key});
      await new Promise(resolve=>setTimeout(resolve,100));return{text:JSON.stringify(await snapshot(agentId),null,2)};
    }
    throw Error('Unknown browser action: '+name);
  }

  function definitions(enabled){
    if(!enabled.has('browser'))return[];
    const fn=(name,description,properties,required=[])=>({type:'function',function:{name,description,parameters:{type:'object',properties,required,additionalProperties:false}}});
    const state={stateId:{type:'string'},purpose:{type:'string'}};
    return[
      fn('browser_snapshot','Inspect the current local browser page and return text, tabs, stable element refs and a stateId.',{}),
      fn('browser_screenshot','Capture the current local browser page as an image.',{}),
      fn('browser_navigate','Navigate the active local browser tab to an HTTP(S) URL.',{url:{type:'string'}},['url']),
      fn('browser_tabs','List, create, select, or close local browser tabs.',{action:{type:'string',enum:['list','new','select','close']},url:{type:'string'},tabIndex:{type:'number'},...state},['action']),
      fn('browser_get_bounding_box','Read the bounding box for an element ref.',{ref:{type:'string'}},['ref']),
      fn('browser_highlight','Temporarily highlight an element ref.',{ref:{type:'string'}},['ref']),
      fn('browser_click','Click an element ref from browser_snapshot. The stateId must still match after review.',{ref:{type:'string'},...state},['ref','stateId','purpose']),
      fn('browser_mouse_click_xy','Click browser viewport coordinates after review.',{x:{type:'number'},y:{type:'number'},button:{type:'string'},doubleClick:{type:'boolean'},...state},['x','y','stateId','purpose']),
      fn('browser_hover','Hover an element ref from browser_snapshot.',{ref:{type:'string'},...state},['ref','stateId']),
      fn('browser_type','Type into an element ref from browser_snapshot.',{ref:{type:'string'},text:{type:'string'},clear:{type:'boolean'},submit:{type:'boolean'},...state},['ref','stateId','text']),
      fn('browser_select','Select one or more values in a select element.',{ref:{type:'string'},value:{type:'string'},values:{type:'array',items:{type:'string'}},...state},['ref','stateId']),
      fn('browser_drag','Drag one element ref to another ref or coordinates.',{sourceRef:{type:'string'},targetRef:{type:'string'},targetX:{type:'number'},targetY:{type:'number'},...state},['sourceRef','stateId','purpose']),
      fn('browser_key','Send a key to the active local browser.',{key:{type:'string'},...state},['stateId','key']),
      fn('browser_scroll','Scroll the current local browser page.',{deltaX:{type:'number'},deltaY:{type:'number'}}),
      fn('browser_cdp','Run a Chrome DevTools Protocol command against the active local browser after review.',{method:{type:'string'},params:{type:'object'},...state},['method','stateId','purpose'])
    ];
  }

  const readOnly=new Set(['browser_snapshot','browser_screenshot','browser_scroll','browser_get_bounding_box','browser_highlight']);
  return{
    definitions,
    isTool:name=>String(name).startsWith('browser_'),
    isMutation:name=>!readOnly.has(name)&&(name!=='browser_tabs'||true),
    execute:({agentId,name,args})=>action({agentId,name,args}),
    disposeAgent(agentId){
      const current=sessions.get(agentId);
      if(current)for(const tab of current.tabs)if(!tab.win.isDestroyed())tab.win.destroy();
      sessions.delete(agentId);
    },
    dispose(){
      for(const current of sessions.values())for(const tab of current.tabs)if(!tab.win.isDestroyed())tab.win.destroy();
      sessions.clear();
    }
  };
}

module.exports={createLocalBrowserRuntime};
