'use strict';
const {app,BrowserWindow,dialog,ipcMain,shell}=require('electron');
const path=require('node:path');
const {URL}=require('node:url');
const {createRuntime}=require('./grok-agent-runtime.cjs');

let mainWindow=null,runtime=null;
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
    'list-agents':'listAgents','create-agent':'createAgent','rename-agent':'renameAgent','delete-agent':'deleteAgent',
    'get-thread':'getThread','send-message':'sendMessage','stop-agent':'stopAgent',
    'list-plugins':'listPlugins','set-plugin-installed':'setPluginInstalled','set-plugin-enabled':'setPluginEnabled',
    'get-runtime-settings':'getRuntimeSettings','set-local-tool-permission':'setLocalToolPermission','resolve-approval':'resolveApproval',
    'list-mcp-servers':'listMcpServers','add-mcp-server':'addMcpServer','remove-mcp-server':'removeMcpServer',
    'set-mcp-server-enabled':'setMcpServerEnabled','list-mcp-server-tools':'listMcpServerTools','set-mcp-tool-enabled':'setMcpToolEnabled',
    'list-workflows':'listWorkflows','save-workflow':'saveWorkflow','delete-workflow':'deleteWorkflow','set-workflow-enabled':'setWorkflowEnabled'
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
    return{path:result.filePaths[0],name:path.basename(result.filePaths[0])};
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
  app.whenReady().then(()=>{runtime=createRuntime({app,BrowserWindow,shell});registerIpc();createWindow();app.on('activate',()=>{if(BrowserWindow.getAllWindows().length===0)createWindow();});});
  app.on('window-all-closed',()=>{if(process.platform!=='darwin')app.quit();});
}
