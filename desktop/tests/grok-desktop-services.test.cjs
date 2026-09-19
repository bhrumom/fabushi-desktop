'use strict';
const test=require('node:test');const assert=require('node:assert/strict');const fs=require('node:fs/promises');const os=require('node:os');const path=require('node:path');const {EventEmitter}=require('node:events');
const {createDesktopServices,parseDeepLink,updateDisabledReason}=require('../electron/grok-desktop-services.cjs');

function fakeApp(root,{packaged=true,version='2.0.0-alpha.1'}={}){return{isPackaged:packaged,getPath:()=>root,getVersion:()=>version}}
test('deep-link parser accepts the recovered info route only',()=>{
  assert.deepEqual(parseDeepLink('sand://app/v1/info?topic=deep-links'),{version:1,source:'protocol',route:'info',topic:'deep-links',url:'sand://app/v1/info?topic=deep-links'});
  assert.equal(parseDeepLink('https://example.test/'),null);
  assert.equal(parseDeepLink('sand://app/v1/other?topic=deep-links'),null);
});
test('update gate mirrors packaged/platform/feed requirements',async t=>{
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-desktop-services-'));t.after(()=>fs.rm(root,{recursive:true,force:true}));
  assert.equal(updateDisabledReason({app:fakeApp(root,{packaged:false}),env:{},platform:'darwin'}),'not-packaged');
  assert.equal(updateDisabledReason({app:fakeApp(root),env:{FABUSHI_UPDATE_FEED_URL:'https://updates.test/{track}'},platform:'linux'}),'unsupported-platform');
  assert.equal(updateDisabledReason({app:fakeApp(root),env:{FABUSHI_DISABLE_UPDATES:'1',FABUSHI_UPDATE_FEED_URL:'https://updates.test/{track}'},platform:'darwin'}),'disabled-by-env');
});
test('update service binds Electron autoUpdater, tracks status, and persists release track',async t=>{
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-desktop-services-'));t.after(()=>fs.rm(root,{recursive:true,force:true}));
  const updater=new EventEmitter();let feed=null,checks=0,installed=false;
  updater.setFeedURL=value=>{feed=value.url};updater.checkForUpdates=async()=>{checks++;updater.emit('update-available',{version:'2.0.1'});return{updateInfo:{version:'2.0.1'}}};updater.quitAndInstall=()=>{installed=true};
  const services=createDesktopServices({app:fakeApp(root),autoUpdater:updater,platform:'darwin',env:{FABUSHI_UPDATE_FEED_URL:'https://updates.test/{track}',FABUSHI_UPDATE_TRACK:'stable'}});
  let status=await services.update.setTrack('nightly');assert.equal(status.currentTrack,'nightly');
  status=await services.update.check();assert.equal(feed,'https://updates.test/nightly');assert.equal(checks,1);assert.equal(status.state.type,'available');
  updater.emit('update-downloaded',{version:'2.0.1'});await new Promise(resolve=>setImmediate(resolve));
  assert.equal((await services.update.status()).state.type,'ready');await services.update.quitAndInstall();assert.equal(installed,true);services.dispose();
});
test('feedback provider validates payload and maps HTTP outcome without a fake success',async t=>{
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'fabushi-desktop-services-'));t.after(()=>fs.rm(root,{recursive:true,force:true}));
  const updater=new EventEmitter();updater.setFeedURL=()=>{};updater.checkForUpdates=async()=>null;updater.quitAndInstall=()=>{};
  const requests=[];
  const services=createDesktopServices({app:fakeApp(root),autoUpdater:updater,platform:'darwin',env:{FABUSHI_UPDATE_FEED_URL:'https://updates.test/stable',FABUSHI_FEEDBACK_URL:'https://feedback.test/submit'},fetchImpl:async(url,options)=>{requests.push({url:String(url),body:JSON.parse(options.body)});return{ok:true,status:200}}});
  assert.deepEqual(await services.submitFeedback({message:' '}),{ok:false,code:'invalid-feedback'});
  assert.deepEqual(await services.submitFeedback({message:'UI issue',conversationId:'agent-1'}),{ok:true});
  assert.equal(requests[0].body.conversationId,'agent-1');services.dispose();
});
