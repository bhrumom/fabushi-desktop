'use strict';

const fs=require('node:fs/promises');
const os=require('node:os');
const path=require('node:path');
const crypto=require('node:crypto');
const {execFile,spawn}=require('node:child_process');

const MAX_TEXT=50000;
const backgroundProcesses=new Map();

function computerStateIdentity(state){
  const canonical=JSON.stringify({
    application:String(state?.application||''),
    windowTitle:String(state?.windowTitle||'')
  });
  return crypto.createHash('sha256').update(canonical).digest('hex');
}
async function currentComputerState(signal){
  if(process.platform!=='darwin')throw Error('Computer state is currently implemented for macOS only.');
  const script=[
    'tell application "System Events"',
    'set frontProcess to first process whose frontmost is true',
    'set appName to name of frontProcess as text',
    'set windowName to ""',
    'try',
    'set windowName to name of front window of frontProcess as text',
    'end try',
    'return appName & linefeed & windowName',
    'end tell'
  ].join('\n');
  const result=await runExec('/usr/bin/osascript',['-e',script],{signal});
  const rows=String(result.stdout||'').replace(/\r/g,'').split('\n');
  const state={application:rows[0]||'',windowTitle:rows.slice(1).join('\n').trim()};
  return{...state,stateId:computerStateIdentity(state)};
}
async function ensureComputerState(expectedStateId,signal){
  const expected=String(expectedStateId||'').trim();
  if(!expected)throw Error('A stateId from computer_screenshot is required for this Computer action.');
  const current=await currentComputerState(signal);
  if(current.stateId!==expected)throw Error('The visible Mac target changed after review; take a fresh computer_screenshot and retry the action.');
  return current;
}
async function computerHelperPath(){
  const candidates=[process.resourcesPath?path.join(process.resourcesPath,'bin','fabushi-computer-helper'):null,path.join(__dirname,'..','native','bin','fabushi-computer-helper')].filter(Boolean);
  for(const candidate of candidates){try{await fs.access(candidate);return candidate}catch{}}
  throw Error('Fabushi Computer helper is unavailable. Use the macOS packaged build so native mouse/scroll actions are installed.');
}
async function runComputerHelper(args,signal){const helper=await computerHelperPath();return runExec(helper,args.map(String),{signal})}
function cancellableDelay(ms,signal){return new Promise((resolve,reject)=>{if(signal?.aborted){const error=Error('Tool execution cancelled.');error.name='AbortError';reject(error);return}const timer=setTimeout(done,ms);function done(){signal?.removeEventListener('abort',abort);resolve()}function abort(){clearTimeout(timer);signal?.removeEventListener('abort',abort);const error=Error('Tool execution cancelled.');error.name='AbortError';reject(error)}signal?.addEventListener('abort',abort,{once:true})})}

async function captureMacScreen(signal){
  if(process.platform!=='darwin')throw Error('Screenshot is currently implemented for macOS only.');
  const state=await currentComputerState(signal),target=path.join(os.tmpdir(),`fabushi-screen-${crypto.randomUUID()}.png`);
  try{
    await runExec('/usr/sbin/screencapture',['-x',target],{signal});
    const data=await fs.readFile(target);
    return{state,data};
  }finally{await fs.unlink(target).catch(()=>{});}
}
function finiteInt(value,label){
  const n=Number(value);if(!Number.isInteger(n))throw Error(label+' must be an integer.');return n;
}
function computerSequence(args={}){
  const first={...args};delete first.then;
  const follow=Array.isArray(args.then)?args.then.slice(0,9):[];
  return[first,...follow];
}
async function executeComputerAction(args,signal){
  const action=String(args?.action||'').trim();
  if(!['screenshot','click','move','drag','type','key','scroll','wait'].includes(action))throw Error('Unknown Computer action: '+action);
  if(action==='screenshot')return'Screenshot requested.';
  if(action==='click'){
    const x=finiteInt(args.x,'x'),y=finiteInt(args.y,'y'),button=['left','right','middle'].includes(args.button)?args.button:'left';
    const count=Math.max(1,Math.min(3,Number.isInteger(Number(args.count))?Number(args.count):1));
    await runComputerHelper(['click',x,y,button,count],signal);return`Clicked (${x}, ${y}) with ${button} button x${count}.`;
  }
  if(action==='move'){
    const x=finiteInt(args.x,'x'),y=finiteInt(args.y,'y');await runComputerHelper(['move',x,y],signal);return`Moved pointer to (${x}, ${y}).`;
  }
  if(action==='drag'){
    const points=Array.isArray(args.path)?args.path.filter(row=>Number.isInteger(Number(row?.x))&&Number.isInteger(Number(row?.y))):[];
    let x1,y1,x2,y2;
    if(points.length>=2){x1=Number(points[0].x);y1=Number(points[0].y);x2=Number(points.at(-1).x);y2=Number(points.at(-1).y)}
    else{x1=finiteInt(args.x,'x');y1=finiteInt(args.y,'y');x2=finiteInt(args.x2,'x2');y2=finiteInt(args.y2,'y2')}
    const duration=Math.max(40,Math.min(30000,Number.isFinite(Number(args.durationMs))?Number(args.durationMs):300)),button=['left','right','middle'].includes(args.button)?args.button:'left';
    await runComputerHelper(['drag',x1,y1,x2,y2,duration,button],signal);return`Dragged (${x1}, ${y1}) to (${x2}, ${y2}).`;
  }
  if(action==='type'){
    const text=String(args.text??'');if(text.length>20000)throw Error('Computer text is too long.');
    await runExec('/usr/bin/osascript',['-e','on run argv','-e','tell application "System Events" to keystroke item 1 of argv','-e','end run',text],{signal});
    return`Typed ${text.length} characters.`;
  }
  if(action==='key'){
    const raw=String(args.key||'').trim();if(!raw||raw.length>256)throw Error('Computer key is invalid.');
    const lower=raw.toLowerCase(),codes={enter:36,return:36,tab:48,escape:53,esc:53,space:49,delete:51,backspace:51,left:123,arrowleft:123,right:124,arrowright:124,down:125,arrowdown:125,up:126,arrowup:126};
    if(codes[lower]!=null)await runExec('/usr/bin/osascript',['-e',`tell application "System Events" to key code ${codes[lower]}`],{signal});
    else if(raw.length===1)await runExec('/usr/bin/osascript',['-e','on run argv','-e','tell application "System Events" to keystroke item 1 of argv','-e','end run',raw],{signal});
    else{
      const parts=lower.split('+').map(x=>x.trim()).filter(Boolean),key=parts.pop(),mods=parts.map(x=>({cmd:'command down',command:'command down',shift:'shift down',alt:'option down',option:'option down',ctrl:'control down',control:'control down'}[x])).filter(Boolean);
      if(!key||key.length!==1||mods.length!==parts.length)throw Error('Unsupported Computer key: '+raw);
      const using=mods.length?' using {'+mods.join(', ')+'}':'';
      await runExec('/usr/bin/osascript',['-e',`tell application "System Events" to keystroke "${key.replace(/["\\]/g,'')}"${using}`],{signal});
    }
    return`Pressed ${raw}.`;
  }
  if(action==='scroll'){
    const amount=Number.isFinite(Number(args.amount))?Math.trunc(Number(args.amount)):3,direction=String(args.direction||'down').toLowerCase();
    const pixels=Math.max(1,Math.abs(amount))*120;let dx=0,dy=0;
    if(direction==='up')dy=pixels;else if(direction==='left')dx=pixels;else if(direction==='right')dx=-pixels;else dy=-pixels;
    await runComputerHelper(['scroll',dx,dy],signal);return`Scrolled ${direction} by ${Math.abs(amount)}.`;
  }
  const duration=Math.max(0,Math.min(30000,Number.isFinite(Number(args.durationMs))?Number(args.durationMs):1000));
  await cancellableDelay(duration,signal);return`Waited ${duration} ms.`;
}
async function executeComputer(args,signal){
  if(process.platform!=='darwin')throw Error('Computer is currently implemented for macOS only.');
  const rows=[];for(const action of computerSequence(args)){if(signal?.aborted)throw Object.assign(Error('Tool execution cancelled.'),{name:'AbortError'});rows.push(await executeComputerAction(action,signal))}
  const shot=await captureMacScreen(signal);
  return{text:rows.join('\n'),display:{kind:'image',dataUrl:'data:image/png;base64,'+shot.data.toString('base64')},computerState:{...shot.state,bytes:shot.data.length}};
}

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
function runStreamingShell(command,{cwd,signal,onOutput,timeoutMs=120000}={}){
  return new Promise((resolve,reject)=>{
    const child=spawn('/bin/zsh',['-lc',command],{cwd,env:process.env,stdio:['ignore','pipe','pipe']});
    let stdout='',stderr='',settled=false;
    const append=(stream,chunk)=>{
      const text=String(chunk??'');
      if(stream==='stdout')stdout=clamp(stdout+text);else stderr=clamp(stderr+text);
      onOutput?.({stream,text});
    };
    const cleanup=()=>{clearTimeout(timer);signal?.removeEventListener('abort',abort)};
    const finishError=(message,name='Error')=>{
      const error=Error(message);error.name=name;error.stdout=stdout;error.stderr=stderr;reject(error);
    };
    const abort=()=>{if(settled)return;try{child.kill('SIGTERM')}catch{}};
    const timer=setTimeout(()=>{if(settled)return;try{child.kill('SIGTERM')}catch{}},timeoutMs);
    child.stdout.on('data',chunk=>append('stdout',chunk));
    child.stderr.on('data',chunk=>append('stderr',chunk));
    child.on('error',error=>{if(settled)return;settled=true;cleanup();reject(error)});
    child.on('exit',(code,signalName)=>{
      if(settled)return;settled=true;cleanup();
      if(signal?.aborted){finishError('Tool execution cancelled.','AbortError');return}
      if(signalName){finishError('Terminal process stopped by '+signalName+'.\n'+clamp(stdout+(stderr?'\n[stderr]\n'+stderr:'')));return}
      if(code!==0){finishError('Terminal command exited with code '+code+'.\n'+clamp(stdout+(stderr?'\n[stderr]\n'+stderr:'')));return}
      resolve({stdout,stderr});
    });
    if(signal){if(signal.aborted)abort();else signal.addEventListener('abort',abort,{once:true})}
  });
}

function descriptor(name,args={}){
  if(name==='Read'||name==='read_file'||name==='list_directory'||name==='get_background_terminal'||name==='Screenshot'||name==='computer_screenshot')return{mutation:false,summary:name==='Screenshot'||name==='computer_screenshot'?'Capture the current Mac screen':`Read ${args.path||args.processId||''}`};
  if(name==='Computer'){
    const actions=computerSequence(args),mutating=actions.some(row=>!['screenshot','wait'].includes(String(row?.action||'')));
    return{mutation:mutating,summary:`Computer ${actions.map(row=>String(row?.action||'')).filter(Boolean).join(' -> ')||'action'} on this Mac`};
  }
  const table={
    run_terminal_cmd:{mutation:true,summary:`Run terminal command: ${String(args.command||'').slice(0,160)}`},
    write_file:{mutation:true,summary:`Write file ${args.path||''}`},
    create_directory:{mutation:true,summary:`Create directory ${args.path||''}`},
    move_path:{mutation:true,summary:`Move ${args.from||''} to ${args.to||''}`},
    run_terminal:{mutation:true,summary:`Run terminal command: ${String(args.command||'').slice(0,160)}`},
    start_background_terminal:{mutation:true,summary:`Start background command: ${String(args.command||'').slice(0,160)}`},
    write_background_terminal:{mutation:true,summary:`Write to background process ${args.processId||''}`},
    stop_background_terminal:{mutation:true,summary:`Stop background process ${args.processId||''}`},
    open_url:{mutation:true,summary:`Open URL ${args.url||''}`}
  };
  return table[name]||{mutation:true,summary:`Run tool ${name}`};
}

function toolDefinitions(enabled){
  const tools=[];
  const fn=(name,description,properties,required=[])=>({type:'function',function:{name,description,parameters:{type:'object',properties,required,additionalProperties:false}}});
  if(enabled.has('filesystem'))tools.push(fn('Read','Read a file from the installed Mac. Omit offset and limit to read the whole text file; image files are returned visually.',{
    path:{type:'string'},offset:{type:'integer'},limit:{type:'integer'},include_line_numbers:{type:'boolean'}
  },['path']));
  if(enabled.has('shell'))tools.push(fn('run_terminal_cmd','Execute a shell command on the installed Mac. Long-running work can be started in the background.',{
    command:{type:'string'},working_directory:{type:'string'},timeout:{type:'number'},description:{type:'string'},is_background:{type:'boolean'},block_until_ms:{type:'number'}
  },['command']));
  if(enabled.has('computer')){
    tools.push(fn('Screenshot','Capture the current screen of the installed Mac.',{}));
    const actionProperties={
      action:{type:'string',enum:['screenshot','click','move','drag','type','key','scroll','wait']},
      x:{type:'integer'},y:{type:'integer'},x2:{type:'integer'},y2:{type:'integer'},
      path:{type:'array',items:{type:'object',properties:{x:{type:'integer'},y:{type:'integer'}},required:['x','y'],additionalProperties:false}},
      text:{type:'string'},key:{type:'string'},button:{type:'string',enum:['left','right','middle']},count:{type:'integer',minimum:1,maximum:3},
      direction:{type:'string',enum:['up','down','left','right']},amount:{type:'integer'},durationMs:{type:'integer',minimum:0,maximum:30000}
    };
    tools.push(fn('Computer','Control the installed Mac using the Grok Computer action protocol. A screenshot is returned after the action sequence.',{
      ...actionProperties,description:{type:'string'},then:{type:'array',minItems:1,maxItems:9,items:{type:'object',properties:actionProperties,required:['action'],additionalProperties:false}}
    },['action']));
  }
  return tools;
}

async function executeTool(name,args,{enabled,shell,signal,onStarted,onOutput,ownerAgentId=null}={}){
  const needed={
    Read:'filesystem',list_directory:'filesystem',read_file:'filesystem',write_file:'filesystem',create_directory:'filesystem',move_path:'filesystem',
    run_terminal_cmd:'shell',run_terminal:'shell',start_background_terminal:'shell',get_background_terminal:'shell',write_background_terminal:'shell',stop_background_terminal:'shell',
    open_url:'browser',Screenshot:'computer',Computer:'computer',computer_screenshot:'computer',computer_click:'computer',computer_mouse_move:'computer',computer_drag:'computer',computer_scroll:'computer',computer_wait:'computer',computer_type:'computer',computer_key:'computer'
  }[name];
  if(needed&&!enabled.has(needed))throw Error(`${needed} capability is disabled.`);
  if(signal?.aborted)throw Object.assign(Error('Tool execution cancelled.'),{name:'AbortError'});
  onStarted?.();

  if(name==='Read'){
    const target=resolvePath(args.path),stat=await fs.stat(target);if(!stat.isFile())throw Error('Path is not a file.');
    const ext=path.extname(target).toLowerCase(),imageMimes={'.png':'image/png','.jpg':'image/jpeg','.jpeg':'image/jpeg','.gif':'image/gif','.webp':'image/webp'};
    if(imageMimes[ext]){if(stat.size>20_000_000)throw Error('Image is larger than 20 MB.');const data=await fs.readFile(target);return{text:`Read image file: ${target}`,display:{kind:'image',dataUrl:`data:${imageMimes[ext]};base64,${data.toString('base64')}`}}}
    if(stat.size>2_000_000)throw Error('File is larger than 2 MB; use offset and limit to read a range.');
    const raw=await fs.readFile(target,'utf8'),lines=raw.split('\n');let start=Number(args.offset);
    if(Number.isInteger(start)&&start<0)start=Math.max(1,lines.length+start+1);if(!Number.isInteger(start)||start<1)start=1;
    const limit=Number.isInteger(Number(args.limit))&&Number(args.limit)>0?Number(args.limit):lines.length;
    const slice=lines.slice(start-1,start-1+limit),text=args.include_line_numbers===true?slice.map((line,index)=>(start+index)+'|'+line).join('\n'):slice.join('\n');
    return{text:clamp(text)};
  }
  if(name==='run_terminal_cmd'){
    const command=String(args.command||'').trim();if(!command)throw Error('Command is required.');if(command.length>12000)throw Error('Command is too long.');
    const cwd=args.working_directory?resolvePath(args.working_directory):process.env.HOME;
    const immediateBackground=args.is_background===true||Number(args.block_until_ms)===0;
    if(immediateBackground)return executeTool('start_background_terminal',{command,cwd},{enabled,shell,signal,onStarted:()=>{},onOutput,ownerAgentId});
    const timeoutRaw=Number(args.timeout??args.block_until_ms),timeoutMs=Number.isFinite(timeoutRaw)&&timeoutRaw>0?Math.max(1,Math.min(600000,timeoutRaw)):120000;
    const result=await runStreamingShell(command,{cwd,signal,onOutput,timeoutMs});
    return{text:clamp((result.stdout||'')+(result.stderr?'\n[stderr]\n'+result.stderr:''))||'(completed with no output)'};
  }
  if(name==='Screenshot'){
    const shot=await captureMacScreen(signal);return{text:'Screenshot captured from the installed Mac.',display:{kind:'image',dataUrl:'data:image/png;base64,'+shot.data.toString('base64')},computerState:{...shot.state,bytes:shot.data.length}};
  }
  if(name==='Computer')return executeComputer(args,signal);

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
    const result=await runStreamingShell(command,{cwd,signal,onOutput,timeoutMs:120000});
    return{text:clamp((result.stdout||'')+(result.stderr?'\n[stderr]\n'+result.stderr:''))||'(completed with no output)'};
  }
  if(name==='start_background_terminal'){
    const command=String(args.command||'').trim();if(!command)throw Error('Command is required.');
    const cwd=args.cwd?resolvePath(args.cwd):process.env.HOME;const id=crypto.randomUUID();
    const child=spawn('/bin/zsh',['-lc',command],{cwd,env:process.env,stdio:['pipe','pipe','pipe']});
    const record={id,command,cwd,pid:child.pid,status:'running',stdout:'',stderr:'',exitCode:null,startedAt:Date.now(),endedAt:null,ownerAgentId:ownerAgentId==null?null:String(ownerAgentId),child};
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
    const state=await currentComputerState(signal);
    const target=path.join(os.tmpdir(),`fabushi-screen-${crypto.randomUUID()}.png`);
    try{
      await runExec('/usr/sbin/screencapture',['-x',target],{signal});
      const data=await fs.readFile(target);
      return{text:JSON.stringify({...state,bytes:data.length},null,2),display:{kind:'image',dataUrl:'data:image/png;base64,'+data.toString('base64')}};
    }finally{await fs.unlink(target).catch(()=>{});}
  }
  if(name==='computer_click'){
    if(process.platform!=='darwin')throw Error('Computer click is currently implemented for macOS only.');
    await ensureComputerState(args.stateId,signal);
    const purpose=String(args.purpose||'').trim();if(!purpose)throw Error('Computer click requires a concise purpose.');if(purpose.length>500)throw Error('Computer click purpose is too long.');
    const x=Number(args.x),y=Number(args.y);if(!Number.isInteger(x)||!Number.isInteger(y))throw Error('Integer x/y coordinates are required.');
    await runExec('/usr/bin/osascript',['-e',`tell application "System Events" to click at {${x}, ${y}}`],{signal});
    return{text:`Clicked (${x}, ${y}).`};
  }
  if(name==='computer_mouse_move'){
    if(process.platform!=='darwin')throw Error('Computer mouse move is currently implemented for macOS only.');await ensureComputerState(args.stateId,signal);
    const x=Number(args.x),y=Number(args.y);if(!Number.isInteger(x)||!Number.isInteger(y))throw Error('Integer x/y coordinates are required.');if(!String(args.purpose||'').trim())throw Error('Computer mouse move requires a concise purpose.');
    await runComputerHelper(['move',x,y],signal);return{text:`Moved pointer to (${x}, ${y}).`};
  }
  if(name==='computer_drag'){
    if(process.platform!=='darwin')throw Error('Computer drag is currently implemented for macOS only.');await ensureComputerState(args.stateId,signal);
    const values=['fromX','fromY','toX','toY'].map(key=>Number(args[key]));if(values.some(value=>!Number.isInteger(value)))throw Error('Integer drag coordinates are required.');if(!String(args.purpose||'').trim())throw Error('Computer drag requires a concise purpose.');
    const duration=Math.max(40,Math.min(5000,Number(args.durationMs)||300));await runComputerHelper(['drag',...values,duration],signal);return{text:`Dragged (${values[0]}, ${values[1]}) to (${values[2]}, ${values[3]}).`};
  }
  if(name==='computer_scroll'){
    if(process.platform!=='darwin')throw Error('Computer scroll is currently implemented for macOS only.');await ensureComputerState(args.stateId,signal);if(!String(args.purpose||'').trim())throw Error('Computer scroll requires a concise purpose.');
    const dx=Number(args.deltaX)||0,dy=Number(args.deltaY);if(!Number.isInteger(dx)||!Number.isInteger(dy))throw Error('Integer scroll deltas are required.');await runComputerHelper(['scroll',dx,dy],signal);return{text:`Scrolled (${dx}, ${dy}).`};
  }
  if(name==='computer_wait'){const ms=Math.max(0,Math.min(30000,Number(args.ms)||1000));await cancellableDelay(ms,signal);return{text:`Waited ${ms} ms.`};}
  if(name==='computer_type'){
    if(process.platform!=='darwin')throw Error('Computer type is currently implemented for macOS only.');
    await ensureComputerState(args.stateId,signal);
    const text=String(args.text??'');if(text.length>2000)throw Error('Text is too long.');
    await runExec('/usr/bin/osascript',['-e','on run argv','-e','tell application "System Events" to keystroke item 1 of argv','-e','end run',text],{signal});
    return{text:`Typed ${text.length} characters.`};
  }
  if(name==='computer_key'){
    if(process.platform!=='darwin')throw Error('Computer key is currently implemented for macOS only.');
    await ensureComputerState(args.stateId,signal);
    const key=String(args.key||'').toLowerCase();if(key.length>256)throw Error('Computer key is too long.');
    const codes={enter:36,return:36,tab:48,escape:53,esc:53,space:49,delete:51,backspace:51,left:123,right:124,down:125,up:126};
    const code=codes[key];if(code==null)throw Error('Unsupported key. Supported: enter, tab, escape, space, delete, arrows.');
    await runExec('/usr/bin/osascript',['-e',`tell application "System Events" to key code ${code}`],{signal});
    return{text:`Pressed ${key}.`};
  }
  throw Error('Unknown tool: '+name);
}

function listBackgroundProcesses({ownerAgentId=null,runningOnly=false}={}){
  const owner=ownerAgentId==null?null:String(ownerAgentId);
  return [...backgroundProcesses.values()]
    .filter(record=>(owner==null||record.ownerAgentId===owner)&&(!runningOnly||record.status==='running'))
    .map(record=>({
      id:record.id,command:record.command,cwd:record.cwd,pid:record.pid,status:record.status,
      startedAt:record.startedAt,endedAt:record.endedAt,ownerAgentId:record.ownerAgentId
    }))
    .sort((a,b)=>a.startedAt-b.startedAt);
}

module.exports={toolDefinitions,executeTool,descriptor,computerStateIdentity,listBackgroundProcesses};
