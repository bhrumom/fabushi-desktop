'use strict';

const fs=require('node:fs/promises');
const os=require('node:os');
const path=require('node:path');
const crypto=require('node:crypto');
const {execFile,spawn}=require('node:child_process');

const MAX_TEXT=50000;
const backgroundProcesses=new Map();

function clamp(text){
  const value=String(text??'');
  return value.length>MAX_TEXT?value.slice(0,MAX_TEXT)+'\n[truncated]':value;
}
function resolvePath(value){
  const raw=String(value||'').trim();
  if(!raw)throw Error('A path is required.');
  return path.resolve(raw.replace(/^~(?=\/|$)/,process.env.HOME||''));
}
function runExec(file,args,options={}){
  return new Promise((resolve,reject)=>{
    const child=execFile(file,args,{...options,maxBuffer:2*1024*1024},(error,stdout,stderr)=>{
      if(error){error.stdout=stdout;error.stderr=stderr;reject(error);return;}
      resolve({stdout:stdout||'',stderr:stderr||''});
    });
    const signal=options.signal;
    if(signal){
      const stop=()=>{try{child.kill('SIGTERM')}catch{}};
      if(signal.aborted)stop(); else signal.addEventListener('abort',stop,{once:true});
      child.once('exit',()=>signal.removeEventListener('abort',stop));
    }
  });
}
function descriptor(name,args={}){
  const table={
    list_directory:{mutation:false,summary:`List directory ${args.path||''}`},
    read_file:{mutation:false,summary:`Read file ${args.path||''}`},
    write_file:{mutation:true,summary:`Write file ${args.path||''}`},
    create_directory:{mutation:true,summary:`Create directory ${args.path||''}`},
    move_path:{mutation:true,summary:`Move ${args.from||''} to ${args.to||''}`},
    run_terminal:{mutation:true,summary:`Run terminal command: ${String(args.command||'').slice(0,160)}`},
    start_background_terminal:{mutation:true,summary:`Start background command: ${String(args.command||'').slice(0,160)}`},
    write_background_terminal:{mutation:true,summary:`Write to background process ${args.processId||''}`},
    stop_background_terminal:{mutation:true,summary:`Stop background process ${args.processId||''}`},
    get_background_terminal:{mutation:false,summary:`Inspect background process ${args.processId||''}`},
    open_url:{mutation:true,summary:`Open URL ${args.url||''}`},
    computer_screenshot:{mutation:false,summary:'Capture the current Mac screen'},
    computer_click:{mutation:true,summary:`Click at (${args.x}, ${args.y}) on this Mac`},
    computer_type:{mutation:true,summary:`Type text on this Mac: ${String(args.text||'').slice(0,120)}`},
    computer_key:{mutation:true,summary:`Press ${args.key||''} on this Mac`}
  };
  return table[name]||{mutation:true,summary:`Run tool ${name}`};
}

function toolDefinitions(enabled){
  const tools=[];
  const fn=(name,description,properties,required=[])=>({type:'function',function:{name,description,parameters:{type:'object',properties,required,additionalProperties:false}}});
  if(enabled.has('filesystem')){
    tools.push(fn('list_directory','List files and folders on the installed Mac.',{path:{type:'string'}},['path']));
    tools.push(fn('read_file','Read a UTF-8 text file on the installed Mac.',{path:{type:'string'}},['path']));
    tools.push(fn('write_file','Write UTF-8 text to a file on the installed Mac.',{path:{type:'string'},content:{type:'string'}},['path','content']));
    tools.push(fn('create_directory','Create a directory on the installed Mac.',{path:{type:'string'}},['path']));
    tools.push(fn('move_path','Move or rename a file or directory on the installed Mac.',{from:{type:'string'},to:{type:'string'}},['from','to']));
  }
  if(enabled.has('shell')){
    tools.push(fn('run_terminal','Run a foreground terminal command on the installed Mac.',{command:{type:'string'},cwd:{type:'string'}},['command']));
    tools.push(fn('start_background_terminal','Start a background terminal command and return a process id.',{command:{type:'string'},cwd:{type:'string'}},['command']));
    tools.push(fn('get_background_terminal','Read status and buffered output for a background terminal process.',{processId:{type:'string'}},['processId']));
    tools.push(fn('write_background_terminal','Write text to a running background terminal process.',{processId:{type:'string'},text:{type:'string'}},['processId','text']));
    tools.push(fn('stop_background_terminal','Stop a running background terminal process.',{processId:{type:'string'}},['processId']));
  }
  if(enabled.has('browser'))tools.push(fn('open_url','Open an HTTPS URL in the default browser on the installed Mac.',{url:{type:'string'}},['url']));
  if(enabled.has('computer')){
    tools.push(fn('computer_screenshot','Capture the current screen of the installed Mac.',{}));
    tools.push(fn('computer_click','Click a screen coordinate on the installed Mac. Requires macOS Accessibility permission.',{x:{type:'integer'},y:{type:'integer'},purpose:{type:'string'}},['x','y','purpose']));
    tools.push(fn('computer_type','Type text into the focused application on the installed Mac. Requires macOS Accessibility permission.',{text:{type:'string'}},['text']));
    tools.push(fn('computer_key','Press a supported key in the focused application on the installed Mac. Requires macOS Accessibility permission.',{key:{type:'string'}},['key']));
  }
  return tools;
}

async function executeTool(name,args,{enabled,shell,signal,onStarted}={}){
  const needed={
    list_directory:'filesystem',read_file:'filesystem',write_file:'filesystem',create_directory:'filesystem',move_path:'filesystem',
    run_terminal:'shell',start_background_terminal:'shell',get_background_terminal:'shell',write_background_terminal:'shell',stop_background_terminal:'shell',
    open_url:'browser',computer_screenshot:'computer',computer_click:'computer',computer_type:'computer',computer_key:'computer'
  }[name];
  if(needed&&!enabled.has(needed))throw Error(`${needed} capability is disabled.`);
  if(signal?.aborted)throw Object.assign(Error('Tool execution cancelled.'),{name:'AbortError'});
  onStarted?.();

  if(name==='list_directory'){
    const target=resolvePath(args.path);const entries=await fs.readdir(target,{withFileTypes:true});
    return{text:clamp(entries.slice(0,500).map(e=>(e.isDirectory()?'[dir] ':'[file] ')+e.name).join('\n'))};
  }
  if(name==='read_file'){
    const target=resolvePath(args.path);const stat=await fs.stat(target);
    if(!stat.isFile())throw Error('Path is not a file.');
    if(stat.size>2_000_000)throw Error('File is larger than 2 MB.');
    return{text:clamp(await fs.readFile(target,'utf8'))};
  }
  if(name==='write_file'){
    const target=resolvePath(args.path);await fs.mkdir(path.dirname(target),{recursive:true});await fs.writeFile(target,String(args.content??''),'utf8');
    return{text:`Wrote ${Buffer.byteLength(String(args.content??''),'utf8')} bytes to ${target}`};
  }
  if(name==='create_directory'){
    const target=resolvePath(args.path);await fs.mkdir(target,{recursive:true});return{text:`Created directory ${target}`};
  }
  if(name==='move_path'){
    const from=resolvePath(args.from),to=resolvePath(args.to);await fs.mkdir(path.dirname(to),{recursive:true});await fs.rename(from,to);return{text:`Moved ${from} to ${to}`};
  }
  if(name==='run_terminal'){
    const command=String(args.command||'').trim();if(!command)throw Error('Command is required.');if(command.length>12000)throw Error('Command is too long.');
    const cwd=args.cwd?resolvePath(args.cwd):process.env.HOME;
    const result=await runExec('/bin/zsh',['-lc',command],{cwd,env:process.env,timeout:120000,signal});
    return{text:clamp((result.stdout||'')+(result.stderr?'\n[stderr]\n'+result.stderr:''))||'(completed with no output)'};
  }
  if(name==='start_background_terminal'){
    const command=String(args.command||'').trim();if(!command)throw Error('Command is required.');
    const cwd=args.cwd?resolvePath(args.cwd):process.env.HOME;const id=crypto.randomUUID();
    const child=spawn('/bin/zsh',['-lc',command],{cwd,env:process.env,stdio:['pipe','pipe','pipe']});
    const record={id,command,cwd,pid:child.pid,status:'running',stdout:'',stderr:'',exitCode:null,startedAt:Date.now(),endedAt:null,child};
    const append=(key,value)=>{record[key]=clamp(record[key]+String(value));};
    child.stdout.on('data',x=>append('stdout',x));child.stderr.on('data',x=>append('stderr',x));
    child.on('error',error=>{record.status='error';append('stderr','\n'+error.message);record.endedAt=Date.now();});
    child.on('exit',(code,signalName)=>{record.status=signalName?'cancelled':code===0?'done':'error';record.exitCode=code;record.endedAt=Date.now();});
    backgroundProcesses.set(id,record);
    return{text:`Started background process ${id} (pid ${child.pid})`};
  }
  if(name==='get_background_terminal'){
    const record=backgroundProcesses.get(String(args.processId||''));if(!record)throw Error('Background process not found.');
    return{text:clamp(JSON.stringify({id:record.id,pid:record.pid,status:record.status,exitCode:record.exitCode,stdout:record.stdout,stderr:record.stderr},null,2))};
  }
  if(name==='write_background_terminal'){
    const record=backgroundProcesses.get(String(args.processId||''));if(!record||record.status!=='running')throw Error('Background process is not running.');
    record.child.stdin.write(String(args.text??''));return{text:`Wrote input to ${record.id}`};
  }
  if(name==='stop_background_terminal'){
    const record=backgroundProcesses.get(String(args.processId||''));if(!record)throw Error('Background process not found.');
    if(record.status==='running')record.child.kill('SIGTERM');return{text:`Stop requested for ${record.id}`};
  }
  if(name==='open_url'){
    const url=new URL(String(args.url||''));if(url.protocol!=='https:')throw Error('Only HTTPS URLs are allowed.');
    await shell.openExternal(url.toString());return{text:'Opened '+url.toString()};
  }
  if(name==='computer_screenshot'){
    if(process.platform!=='darwin')throw Error('Computer screenshot is currently implemented for macOS only.');
    const target=path.join(os.tmpdir(),`fabushi-screen-${crypto.randomUUID()}.png`);
    try{
      await runExec('/usr/sbin/screencapture',['-x',target],{signal});
      const data=await fs.readFile(target);return{text:`Captured screen (${data.length} bytes).`,display:{kind:'image',dataUrl:'data:image/png;base64,'+data.toString('base64')}};
    }finally{await fs.unlink(target).catch(()=>{});}
  }
  if(name==='computer_click'){
    if(process.platform!=='darwin')throw Error('Computer click is currently implemented for macOS only.');
    const x=Number(args.x),y=Number(args.y);if(!Number.isInteger(x)||!Number.isInteger(y))throw Error('Integer x/y coordinates are required.');
    await runExec('/usr/bin/osascript',['-e',`tell application "System Events" to click at {${x}, ${y}}`],{signal});
    return{text:`Clicked (${x}, ${y}).`};
  }
  if(name==='computer_type'){
    if(process.platform!=='darwin')throw Error('Computer type is currently implemented for macOS only.');
    const text=String(args.text??'');if(text.length>8000)throw Error('Text is too long.');
    await runExec('/usr/bin/osascript',['-e','on run argv','-e','tell application "System Events" to keystroke item 1 of argv','-e','end run',text],{signal});
    return{text:`Typed ${text.length} characters.`};
  }
  if(name==='computer_key'){
    if(process.platform!=='darwin')throw Error('Computer key is currently implemented for macOS only.');
    const key=String(args.key||'').toLowerCase();
    const codes={enter:36,return:36,tab:48,escape:53,esc:53,space:49,delete:51,backspace:51,left:123,right:124,down:125,up:126};
    const code=codes[key];if(code==null)throw Error('Unsupported key. Supported: enter, tab, escape, space, delete, arrows.');
    await runExec('/usr/bin/osascript',['-e',`tell application "System Events" to key code ${code}`],{signal});
    return{text:`Pressed ${key}.`};
  }
  throw Error('Unknown tool: '+name);
}

module.exports={toolDefinitions,executeTool,descriptor};
