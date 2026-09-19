'use strict';
const {app,BrowserWindow,dialog,ipcMain,shell,safeStorage,Notification,autoUpdater,protocol,nativeTheme,Menu}=require('electron');
const fs=require('node:fs');
const path=require('node:path');
const {URL}=require('node:url');
const {createRuntime}=require('./grok-agent-runtime.cjs');
const {createAttachmentGateway}=require('./grok-attachment-gateway.cjs');
const {registerSandMediaScheme,registerSandMediaProtocol}=require('./grok-media-protocol.cjs');
const {createDesktopServices,parseDeepLink}=require('./grok-desktop-services.cjs');
const {createReferenceCoordinator}=require('./grok-reference-coordinator.cjs');
const {createReferenceDesktop}=require('./grok-reference-desktop.cjs');

registerSandMediaScheme(protocol);
let mainWindow=null,runtime=null,desktopServices=null,referenceCoordinator=null,referenceDesktop=null,quitAfterDispose=false,pendingDeepLink=null;
function sendReferenceEvent(name,payload={}){
  if(mainWindow&&!mainWindow.isDestroyed())mainWindow.webContents.send('grok-reference:'+name,payload);
}
function currentWindowState(win=mainWindow){
  return{isFullscreen:Boolean(win?.isFullScreen?.()),isMaximized:Boolean(win?.isMaximized?.())};
}
function installReferenceApplicationMenu(){
  const openAbout=()=>sendReferenceEvent('open-about');
  const openFeedback=()=>sendReferenceEvent('open-feedback');
  const forceOnboarding=()=>sendReferenceEvent('force-onboarding');
  const appMenu=[
    {label:'About Fabushi',click:openAbout},
    {type:'separator'},
    {role:'services'},
    {type:'separator'},
    {role:'hide'},
    {role:'hideOthers'},
    {role:'unhide'},
    {type:'separator'},
    {role:'quit'}
  ];
  const template=[
    ...(process.platform==='darwin'?[{label:app.name||'Fabushi',submenu:appMenu}]:[]),
    {label:'Edit',submenu:[{role:'undo'},{role:'redo'},{type:'separator'},{role:'cut'},{role:'copy'},{role:'paste'},{role:'selectAll'}]},
    {label:'View',submenu:[{role:'reload'},{role:'forceReload'},{type:'separator'},{role:'resetZoom'},{role:'zoomIn'},{role:'zoomOut'},{type:'separator'},{role:'togglefullscreen'}]},
    {label:'Window',submenu:[{role:'minimize'},{role:'zoom'},...(process.platform==='darwin'?[{type:'separator'},{role:'front'}]:[{role:'close'}])]},
    {label:'Help',submenu:[{label:'Run Onboarding',click:forceOnboarding},{label:'Send Feedback…',click:openFeedback},...(process.platform==='darwin'?[]:[{label:'About Fabushi',click:openAbout}])]}
  ];
  Menu.setApplicationMenu(Menu.buildFromTemplate(template));
}
function emitDeepLink(link){if(!link)return;if(mainWindow&&!mainWindow.isDestroyed()&&!mainWindow.webContents.isLoading())mainWindow.webContents.send('grok-agent:deep-link',link);else pendingDeepLink=link;}
function captureDeepLink(value){const link=parseDeepLink(value);if(!link)return false;emitDeepLink(link);return true;}
function trusted(event){
  const raw=event.senderFrame?.url||event.sender.getURL();
  try{
    const url=new URL(raw);
    if(url.protocol==='file:')return true;
    const dev=String(process.env.VITE_DEV_SERVER_URL||'').trim();
    return!!dev&&url.origin===new URL(dev).origin;
  }catch{return false;}
}
function assertTrusted(event){if(!trusted(event))throw Error('Rejected IPC sender');}
function registerIpc(){
  const methods={
    'list-agents':'listAgents','create-agent':'createAgent','rename-agent':'renameAgent','update-agent':'updateAgent','set-agent-avatar-bytes':'setAgentAvatarBytes','generate-agent-avatar-image':'generateAgentAvatarImage','set-agent-notify':'setAgentNotifyOnUpdates','set-agent-pinned':'setAgentPinned','set-agent-unread':'setAgentUnread','duplicate-agent':'duplicateAgent','set-group-members':'setGroupMembers','set-agent-hidden':'setAgentHidden','delete-agent':'deleteAgent',
    'get-thread':'getThread','send-message':'sendMessage','respond-to-widget':'respondToWidget','dismiss-widget':'dismissWidget','submit-secret':'submitSecret','react-to-message':'reactToMessage','search-messages':'searchMessages','search-media':'searchMedia','search-links':'searchLinks','stop-agent':'stopAgent',
    'list-plugins':'listPlugins','set-plugin-installed':'setPluginInstalled','set-plugin-enabled':'setPluginEnabled',
    'get-account-status':'getAccountStatus','login-account':'loginAccount','cancel-account-login':'cancelAccountLogin','logout-account':'logoutAccount','update-account-name':'updateAccountName','get-account-avatar':'getAccountAvatar',
    'get-runtime-settings':'getRuntimeSettings','set-local-tool-permission':'setLocalToolPermission','set-auto-review-mode':'setAutoReviewMode','set-auto-review-instructions':'setAutoReviewInstructions','resolve-approval':'resolveApproval',
    'get-agent-channels':'getAgentChannels','connect-channel':'connectChannel','disconnect-channel':'disconnectChannel','refresh-channel':'refreshChannel','get-async-tasks':'getAsyncTasks','get-experiments-snapshot':'getExperimentsSnapshot','refresh-experiments':'refreshExperiments','apply-feature-flag-override':'applyFeatureFlagOverride',
    'list-mcp-servers':'listMcpServers','add-mcp-server':'addMcpServer','update-mcp-server':'updateMcpServer','remove-mcp-server':'removeMcpServer',
    'set-mcp-server-enabled':'setMcpServerEnabled','get-mcp-account-status':'getMcpAccountStatus','list-mcp-accounts':'listMcpAccounts','connect-mcp-account':'connectMcpAccount','disconnect-mcp-account':'disconnectMcpAccount','rename-mcp-account':'renameMcpAccount','remove-mcp-account':'removeMcpAccount','set-mcp-active-account':'setMcpActiveAccount','list-mcp-server-tools':'listMcpServerTools','set-mcp-tool-enabled':'setMcpToolEnabled',
    'list-marketplace-plugins':'listMarketplacePlugins','install-marketplace-plugin':'installMarketplacePlugin','uninstall-marketplace-plugin':'uninstallMarketplacePlugin',
    'list-workflows':'listWorkflows','save-workflow':'saveWorkflow','delete-workflow':'deleteWorkflow','set-workflow-enabled':'setWorkflowEnabled',
    'get-agent-automations':'getAgentAutomations','create-agent-automation':'createAgentAutomation','set-agent-automation-enabled':'setAgentAutomationEnabled',
    'read-attachment':'readAttachment',
    'update-agent-automation':'updateAgentAutomation','delete-agent-automation':'deleteAgentAutomation','run-agent-automation-now':'runAgentAutomationNow'
  };
  for(const[name,method]of Object.entries(methods)){
    ipcMain.handle('grok-agent:'+name,async(event,args={})=>{
      assertTrusted(event);return runtime[method](args&&typeof args==='object'?args:{});
    });
  }
  ipcMain.handle('grok-agent:pick-avatar-file',async event=>{
    assertTrusted(event);
    const result=await dialog.showOpenDialog(mainWindow,{properties:['openFile'],filters:[{name:'Images',extensions:['png','jpg','jpeg','webp','gif','bmp','tif','tiff']}]});
    if(result.canceled||!result.filePaths[0])return null;
    const record=await runtime.registerAttachment({path:result.filePaths[0]});
    if(record.kind!=='image')throw Error('Choose an image file.');
    const preview=await runtime.readAttachment({id:record.id});
    if(!preview?.dataUrl)throw Error('That image could not be loaded.');
    return{dataUrl:preview.dataUrl,fileName:record.name};
  });
  ipcMain.handle('grok-agent:pick-file',async event=>{
    assertTrusted(event);
    const result=await dialog.showOpenDialog(mainWindow,{properties:['openFile']});
    if(result.canceled||!result.filePaths[0])return null;
    return runtime.registerAttachment({path:result.filePaths[0]});
  });
  ipcMain.handle('grok-agent:get-desktop-info',async event=>{assertTrusted(event);return desktopServices.getInfo()});
  ipcMain.handle('grok-agent:get-update-status',async event=>{assertTrusted(event);return desktopServices.update.status()});
  ipcMain.handle('grok-agent:check-update',async event=>{assertTrusted(event);return desktopServices.update.check()});
  ipcMain.handle('grok-agent:set-update-track',async(event,args={})=>{assertTrusted(event);return desktopServices.update.setTrack(args.track)});
  ipcMain.handle('grok-agent:set-auto-update',async(event,args={})=>{assertTrusted(event);return desktopServices.update.setAutoUpdateWhenIdleOptIn(args.enabled===true)});
  ipcMain.handle('grok-agent:quit-and-install',async event=>{assertTrusted(event);return desktopServices.update.quitAndInstall()});
  ipcMain.handle('grok-agent:submit-feedback',async(event,args={})=>{assertTrusted(event);return desktopServices.submitFeedback(args)});
  ipcMain.handle('grok-agent:get-onboarding-seen',async event=>{assertTrusted(event);return desktopServices.getOnboardingSeen()});
  ipcMain.handle('grok-agent:set-onboarding-seen',async(event,args={})=>{assertTrusted(event);return desktopServices.setOnboardingSeen(args.seen===true)});
  ipcMain.handle('grok-agent:open-external',async(event,args={})=>{assertTrusted(event);const value=String(args.url||'').trim();let url;try{url=new URL(value)}catch{throw Error('External URL is invalid.')}if(url.protocol!=='https:')throw Error('Only HTTPS external URLs are allowed.');await shell.openExternal(url.toString());return{ok:true}});
  ipcMain.handle('grok-reference:coordinator',async(event,payload={})=>{assertTrusted(event);if(!referenceCoordinator)throw Error('Grok coordinator bridge is unavailable.');return referenceCoordinator.call(String(payload.method||''),payload.args&&typeof payload.args==='object'?payload.args:{});});
  ipcMain.handle('grok-reference:desktop',async(event,payload={})=>{assertTrusted(event);if(!referenceDesktop)throw Error('Grok desktop bridge is unavailable.');return referenceDesktop.call(String(payload.method||''),payload.args&&typeof payload.args==='object'?payload.args:{});});
}
function createWindow(){
  const win=new BrowserWindow({
    width:1240,height:820,minWidth:900,minHeight:620,title:'Fabushi',backgroundColor:'#111216',
    titleBarStyle:process.platform==='darwin'?'hiddenInset':'default',
    trafficLightPosition:process.platform==='darwin'?{x:13,y:13}:undefined,
    webPreferences:{preload:path.join(__dirname,'preload.cjs'),contextIsolation:true,nodeIntegration:false,sandbox:false}
  });
  const smokePath=String(process.env.FABUSHI_GROK_SMOKE_REPORT||'').trim();
  let smokeScheduled=false;
  const writeSmokeFailure=(stage,error)=>{if(!smokePath)return;try{fs.writeFileSync(smokePath,JSON.stringify({ok:false,stage,error:String(error?.message||error||'unknown renderer load failure')},null,2)+'\\n',{mode:0o600})}catch{}};
  const scheduleSmoke=stage=>{
    if(!smokePath||smokeScheduled)return;
    smokeScheduled=true;
    setTimeout(()=>{void win.webContents.executeJavaScript(`(async()=>{try{const d=window.desktop;const account=d?.cursorAccount?.getStatus?await d.cursorAccount.getStatus():null;const windowState=d?.getWindowState?await d.getWindowState():null;return{ok:true,stage:${JSON.stringify(stage)},hasDesktop:!!d,hasCoordinator:!!window.coordinatorPort,hasMcp:!!d?.mcp,accountKind:account?.kind||null,windowState,readyState:document.readyState,location:String(location.href),bodyText:(document.body?.innerText||'').slice(0,4000)}}catch(error){return{ok:false,stage:${JSON.stringify(stage)},error:String(error?.message||error),readyState:document.readyState,location:String(location.href),bodyText:(document.body?.innerText||'').slice(0,4000)}}})()`,true).then(report=>fs.writeFileSync(smokePath,JSON.stringify(report,null,2)+'\\n',{mode:0o600})).catch(error=>writeSmokeFailure(stage,error))},1200);
  };
  win.webContents.on('did-fail-load',(_event,errorCode,errorDescription,validatedURL,isMainFrame)=>{if(isMainFrame!==false){console.error('renderer did-fail-load',errorCode,errorDescription,validatedURL);writeSmokeFailure('did-fail-load',errorDescription||errorCode)}});
  win.webContents.on('render-process-gone',(_event,details)=>{console.error('renderer process gone',details);writeSmokeFailure('render-process-gone',details?.reason||details)});
  const emitWindowState=()=>sendReferenceEvent('window-state',currentWindowState(win));
  for(const eventName of ['maximize','unmaximize','enter-full-screen','leave-full-screen'])win.on(eventName,emitWindowState);
  win.webContents.on('zoom-changed',()=>sendReferenceEvent('zoom-factor-changed',{factor:win.webContents.getZoomFactor()}));
  win.webContents.once('dom-ready',()=>scheduleSmoke('dom-ready'));
  win.webContents.once('did-stop-loading',()=>scheduleSmoke('did-stop-loading'));
  win.webContents.on('did-finish-load',()=>{if(pendingDeepLink){const link=pendingDeepLink;pendingDeepLink=null;win.webContents.send('grok-agent:deep-link',link)}scheduleSmoke('did-finish-load')});
  if(process.env.VITE_DEV_SERVER_URL)void win.loadURL(process.env.VITE_DEV_SERVER_URL).catch(error=>writeSmokeFailure('load-url',error));
  else void win.loadFile(path.join(__dirname,'..','dist','index.html')).catch(error=>writeSmokeFailure('load-file',error));
  win.on('closed',()=>{if(mainWindow===win)mainWindow=null;});mainWindow=win;return win;
}
if(!app.requestSingleInstanceLock())app.quit();
else{
  app.on('open-url',(event,url)=>{event.preventDefault();captureDeepLink(url)});
  app.on('second-instance',(_event,argv)=>{for(const value of argv||[])if(captureDeepLink(value))break;if(!mainWindow)createWindow();if(mainWindow.isMinimized())mainWindow.restore();mainWindow.show();mainWindow.focus();});
  app.whenReady().then(()=>{try{app.setAsDefaultProtocolClient('sand');app.setAsDefaultProtocolClient('fabushi')}catch{}const attachmentGateway=createAttachmentGateway({app});registerSandMediaProtocol(protocol,attachmentGateway);runtime=createRuntime({app,BrowserWindow,shell,safeStorage,attachmentGateway,notify:({title,body})=>{if(Notification.isSupported())new Notification({title:String(title||'Fabushi'),body:String(body||'')}).show()}});desktopServices=createDesktopServices({app,autoUpdater,onUpdateStatus:status=>{if(mainWindow&&!mainWindow.isDestroyed())mainWindow.webContents.send('grok-agent:update-status',status)}});referenceCoordinator=createReferenceCoordinator(runtime);referenceDesktop=createReferenceDesktop({app,BrowserWindow,shell,dialog,safeStorage,nativeTheme,getWindow:()=>mainWindow});registerIpc();createWindow();installReferenceApplicationMenu();nativeTheme.on('updated',()=>sendReferenceEvent('theme-changed',{preference:nativeTheme.themeSource,resolved:nativeTheme.shouldUseDarkColors?'dark':'light'}));app.on('activate',()=>{if(BrowserWindow.getAllWindows().length===0)createWindow();});});
  app.on('window-all-closed',()=>{if(process.platform!=='darwin')app.quit();});
  app.on('before-quit',event=>{if(!runtime||quitAfterDispose)return;event.preventDefault();quitAfterDispose=true;desktopServices?.dispose?.();void Promise.resolve(runtime.dispose?.()).finally(()=>app.quit());});
}
