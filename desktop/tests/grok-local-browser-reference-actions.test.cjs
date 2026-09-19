'use strict';
const test=require('node:test');
const assert=require('node:assert/strict');
const {createLocalBrowserRuntime}=require('../electron/grok-local-browser.cjs');

class FakeDebugger{
  constructor(){this.attached=false;this.calls=[]}
  isAttached(){return this.attached}
  attach(){this.attached=true}
  detach(){this.attached=false}
  async sendCommand(method,params){this.calls.push({method,params});return{ok:true,method}}
}
class FakeContents{
  constructor(){this.debugger=new FakeDebugger();this.url='https://example.test/';this.title='Example';this.events=[]}
  setWindowOpenHandler(fn){this.openHandler=fn}
  async executeJavaScript(script){
    if(script.includes("document.querySelectorAll"))return{url:this.url,title:this.title,body:'Example page',elements:[{ref:'fabushi-test-0',tag:'button',role:'',type:'',text:'Go',ariaLabel:'',placeholder:'',href:'',value:'',disabled:false,bounds:{x:10,y:20,width:80,height:30}}]};
    if(script.includes("getBoundingClientRect"))return{x:10,y:20,width:80,height:30,centerX:50,centerY:35};
    return true;
  }
  async capturePage(){return{toPNG:()=>Buffer.from('png')}}
  focus(){}
  sendInputEvent(event){this.events.push(event)}
}
class FakeWindow{
  static all=[];
  constructor(){this.webContents=new FakeContents();this.destroyed=false;this.handlers={};FakeWindow.all.push(this)}
  async loadURL(url){this.webContents.url=url}
  on(name,fn){this.handlers[name]=fn}
  isDestroyed(){return this.destroyed}
  destroy(){this.destroyed=true;this.handlers.closed?.()}
}

test('local browser exposes pinned-reference action breadth on the installed machine',()=>{
  FakeWindow.all.length=0;
  const runtime=createLocalBrowserRuntime({BrowserWindow:FakeWindow});
  const names=runtime.definitions(new Set(['browser'])).map(x=>x.function.name).sort();
  assert.deepEqual(names,[
    'browser_cdp','browser_click','browser_drag','browser_get_bounding_box','browser_highlight','browser_hover','browser_key',
    'browser_mouse_click_xy','browser_navigate','browser_screenshot','browser_scroll','browser_select','browser_snapshot','browser_tabs','browser_type'
  ]);
  runtime.dispose();
});

test('local browser tabs are real per-agent windows and CDP stays scoped to the active tab',async()=>{
  FakeWindow.all.length=0;
  const runtime=createLocalBrowserRuntime({BrowserWindow:FakeWindow});
  let snap=JSON.parse((await runtime.execute({agentId:'a1',name:'browser_snapshot',args:{}})).text);
  assert.equal(snap.tabs.length,1);
  await runtime.execute({agentId:'a1',name:'browser_tabs',args:{action:'new',url:'https://two.test/'}});
  let listed=JSON.parse((await runtime.execute({agentId:'a1',name:'browser_tabs',args:{action:'list'}})).text);
  assert.equal(listed.tabs.length,2);assert.equal(listed.activeTabIndex,1);
  snap=JSON.parse((await runtime.execute({agentId:'a1',name:'browser_snapshot',args:{}})).text);
  const active=FakeWindow.all.at(-1);
  const result=JSON.parse((await runtime.execute({agentId:'a1',name:'browser_cdp',args:{method:'Runtime.evaluate',params:{expression:'1+1'},stateId:snap.stateId,purpose:'inspect page'}})).text);
  assert.equal(result.ok,true);assert.equal(active.webContents.debugger.calls[0].method,'Runtime.evaluate');
  await runtime.execute({agentId:'a1',name:'browser_tabs',args:{action:'close',tabIndex:1,stateId:snap.stateId,purpose:'close test tab'}});
  listed=JSON.parse((await runtime.execute({agentId:'a1',name:'browser_tabs',args:{action:'list'}})).text);
  assert.equal(listed.tabs.length,1);
  runtime.dispose();
});
