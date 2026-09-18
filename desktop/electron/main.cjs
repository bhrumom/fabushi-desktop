'use strict';
const {app,BrowserWindow,dialog,ipcMain,shell,safeStorage,Notification}=require('electron');
const path=require('node:path');
const {URL}=require('node:url');
const {createRuntime}=require('./grok-agent-runtime.cjs');
const {createAttachmentGateway}=require('./grok-attachment-gateway.cjs');

let mainWindow=null,runtime=null,quitAfterDispose=false;
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
    'list-agents':'listAgents','create-agent':'createAgent','rename-agent':'renameAgent','update-agent':'updateAgent','set-agent-notify':'setAgentNotifyOnUpdates','set-agent-hidden':'setAgentHidden','delete-agent':'deleteAgent',
    'get-thread':'getThread','send-message':'sendMessage','react-to-message':'reactToMessage','search-messages':'searchMessages','search-media':'searchMedia','search-links':'searchLinks','stop-agent':'stopAgent',
    'list-plugins':'listPlugins','set-plugin-installed':'setPluginInstalled','set-plugin-enabled':'setPluginEnabled',
    'get-account-status':'getAccountStatus','login-account':'loginAccount','cancel-account-login':'cancelAccountLogin','logout-account':'logoutAccount','update-account-name':'updateAccountName','get-account-avatar':'getAccountAvatar',
    'get-runtime-settings':'getRuntimeSettings','set-local-tool-permission':'setLocalToolPermission','set-auto-review-mode':'setAutoReviewMode','set-auto-review-instructions':'setAutoReviewInstructions','resolve-approval':'resolveApproval',
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
  ipcMain.handle('grok-agent:pick-file',async event=>{
    assertTrusted(event);
    const result=await dialog.showOpenDialog(mainWindow,{properties:['openFile']});
    if(result.canceled||!result.filePaths[0])return null;
    return runtime.registerAttachment({path:result.filePaths[0]});
  });
}
function createWindow(){
  const win=new BrowserWindow({
    width:1240,height:820,minWidth:900,minHeight:620,title:'Fabushi',backgroundColor:'#111216',
    titleBarStyle:process.platform==='darwin'?'hiddenInset':'default',
    trafficLightPosition:process.platform==='darwin'?{x:13,y:13}:undefined,
    webPreferences:{preload:path.join(__dirname,'preload.cjs'),contextIsolation:true,nodeIntegration:false,sandbox:false}
  });
  if(process.env.VITE_DEV_SERVER_URL)void win.loadURL(process.env.VITE_DEV_SERVER_URL);
  else void win.loadFile(path.join(__dirname,'..','dist','index.html'));
  win.on('closed',()=>{if(mainWindow===win)mainWindow=null;});mainWindow=win;return win;
}
if(!app.requestSingleInstanceLock())app.quit();
else{
  app.on('second-instance',()=>{if(!mainWindow)createWindow();if(mainWindow.isMinimized())mainWindow.restore();mainWindow.show();mainWindow.focus();});
  app.whenReady().then(()=>{const attachmentGateway=createAttachmentGateway({app});runtime=createRuntime({app,BrowserWindow,shell,safeStorage,attachmentGateway,notify:({title,body})=>{if(Notification.isSupported())new Notification({title:String(title||'Fabushi'),body:String(body||'')}).show()}});registerIpc();createWindow();app.on('activate',()=>{if(BrowserWindow.getAllWindows().length===0)createWindow();});});
  app.on('window-all-closed',()=>{if(process.platform!=='darwin')app.quit();});
  app.on('before-quit',event=>{if(!runtime||quitAfterDispose)return;event.preventDefault();quitAfterDispose=true;void Promise.resolve(runtime.dispose?.()).finally(()=>app.quit());});
}
