'use strict';
const fs=require('node:fs/promises');
const path=require('node:path');
const {execFile}=require('node:child_process');
const {promisify}=require('node:util');
const execFileAsync=promisify(execFile);
const MAX_TEXT=50000;

function clamp(text){const value=String(text??'');return value.length>MAX_TEXT?value.slice(0,MAX_TEXT)+'\n[truncated]':value;}
function resolvePath(value){const raw=String(value||'').trim();if(!raw)throw Error('A path is required.');return path.resolve(raw.replace(/^~(?=\/|$)/,process.env.HOME||''));}

function toolDefinitions(enabled){
 const tools=[];
 if(enabled.has('filesystem')){
  tools.push({type:'function',function:{name:'list_directory',description:'List files and folders on the installed Mac.',parameters:{type:'object',properties:{path:{type:'string'}},required:['path'],additionalProperties:false}}});
  tools.push({type:'function',function:{name:'read_file',description:'Read a UTF-8 text file on the installed Mac.',parameters:{type:'object',properties:{path:{type:'string'}},required:['path'],additionalProperties:false}}});
 }
 if(enabled.has('shell')) tools.push({type:'function',function:{name:'run_terminal',description:'Run a terminal command on the installed Mac. Prefer inspection and reversible actions. Avoid destructive commands unless the user explicitly requested them.',parameters:{type:'object',properties:{command:{type:'string'}},required:['command'],additionalProperties:false}}});
 if(enabled.has('browser')) tools.push({type:'function',function:{name:'open_url',description:'Open an HTTPS URL in the default browser on the installed Mac.',parameters:{type:'object',properties:{url:{type:'string'}},required:['url'],additionalProperties:false}}});
 return tools;
}

async function executeTool(name,args,{enabled,shell}){
 if(name==='list_directory'){
  if(!enabled.has('filesystem'))throw Error('Files plugin is disabled.');
  const target=resolvePath(args.path);const entries=await fs.readdir(target,{withFileTypes:true});
  return clamp(entries.slice(0,500).map(e=>(e.isDirectory()?'[dir] ':'[file] ')+e.name).join('\n'));
 }
 if(name==='read_file'){
  if(!enabled.has('filesystem'))throw Error('Files plugin is disabled.');
  const target=resolvePath(args.path);const stat=await fs.stat(target);if(!stat.isFile())throw Error('Path is not a file.');if(stat.size>2_000_000)throw Error('File is larger than 2 MB.');
  return clamp(await fs.readFile(target,'utf8'));
 }
 if(name==='run_terminal'){
  if(!enabled.has('shell'))throw Error('Terminal plugin is disabled.');
  const command=String(args.command||'').trim();if(!command)throw Error('Command is required.');if(command.length>12000)throw Error('Command is too long.');
  const result=await execFileAsync('/bin/zsh',['-lc',command],{timeout:120000,maxBuffer:1024*1024,env:process.env});
  return clamp((result.stdout||'')+(result.stderr?'\n[stderr]\n'+result.stderr:''));
 }
 if(name==='open_url'){
  if(!enabled.has('browser'))throw Error('Browser plugin is disabled.');
  const url=new URL(String(args.url||''));if(url.protocol!=='https:')throw Error('Only HTTPS URLs are allowed.');
  await shell.openExternal(url.toString());return 'Opened '+url.toString();
 }
 throw Error('Unknown tool: '+name);
}
module.exports={toolDefinitions,executeTool};