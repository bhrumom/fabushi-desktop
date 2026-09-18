'use strict';

const fs=require('node:fs/promises');
const path=require('node:path');
const crypto=require('node:crypto');

const MAX_NAME=80,MAX_DESCRIPTION=1536,MAX_BODY=100000,INJECT_LIMIT=8000,UI_LIMIT=100,MAX_PER_AGENT=100;
const WORKFLOW_FILENAME='SKILL.md';

const line=(value,max)=>String(value??'').replace(/[\r\n]+/g,' ').trim().slice(0,max);
const block=(value,max)=>String(value??'').trim().slice(0,max);
function slugify(name){return line(name,MAX_NAME).normalize('NFKD').toLowerCase().replace(/[^a-z0-9]+/g,'-').replace(/^-+|-+$/g,'').slice(0,64).replace(/-+$/g,'')||'workflow'}
function yamlString(value){return JSON.stringify(String(value??''))}
function parseScalar(raw){
  const value=raw.trim();
  if(value==='true')return true;if(value==='false')return false;if(value==='null'||value==='~')return null;
  if(/^[-+]?\d+(?:\.\d+)?$/.test(value))return Number(value);
  if(value.startsWith('"')){try{return JSON.parse(value)}catch{}}
  if(value.startsWith("'")&&value.endsWith("'"))return value.slice(1,-1).replace(/''/g,"'");
  return value;
}
function parseFrontmatter(raw){
  if(!raw.startsWith('---'))return{data:{},content:raw};
  const first=raw.indexOf('\n'),close=raw.indexOf('\n---',first);
  if(first<0||close<0)return{data:{},content:raw};
  const data={},stack=[{indent:-1,value:data}];
  for(const source of raw.slice(first+1,close).split(/\r?\n/)){
    if(!source.trim()||source.trimStart().startsWith('#'))continue;
    const match=/^(\s*)([^:#][^:]*):(?:\s*(.*))?$/.exec(source);if(!match)continue;
    const indent=match[1].length,key=match[2].trim(),tail=match[3]||'';
    while(stack[stack.length-1].indent>=indent)stack.pop();
    const parent=stack[stack.length-1].value;
    if(!tail.trim()){const nested={};parent[key]=nested;stack.push({indent,value:nested})}
    else parent[key]=parseScalar(tail);
  }
  return{data,content:raw.slice(close+4).replace(/^\r?\n/,'')};
}
function parseFile(raw,fallbackId,filePath,stat){
  const parsed=parseFrontmatter(raw),data=parsed.data,content=parsed.content,metadata=data.metadata&&typeof data.metadata==='object'?data.metadata:{};
  const trigger=data.trigger&&typeof data.trigger==='object'&&String(data.trigger.schedule||'').trim()
    ?{schedule:String(data.trigger.schedule).trim().replace(/\s+/g,' '),isEnabled:data.trigger.enabled!==false}:null;
  const name=line(data.name||path.basename(path.dirname(filePath)).replace(/[-_]+/g,' '),MAX_NAME);
  const body=block(content,MAX_BODY);if(!name&&!body)return null;
  return{
    id:line(data.id||fallbackId,120),name:name||'Workflow',description:line(data.description,MAX_DESCRIPTION),body,trigger,
    source:'workflow',sourceRef:typeof metadata.source==='string'?metadata.source:null,
    isEnabledForAgent:data.enabledForAgent!==false,disableModelInvocation:data.disableModelInvocation===true,
    createdAt:Number(data.createdAt)||stat.birthtimeMs||stat.ctimeMs||Date.now(),updatedAt:stat.mtimeMs||Date.now(),
    helperScripts:[],filePath
  };
}
function serialize(record){
  const lines=[
    '---',
    'id: '+yamlString(record.id),
    'name: '+yamlString(record.name),
    ...(record.description?['description: '+yamlString(record.description)]:[]),
    'enabledForAgent: '+String(record.isEnabledForAgent!==false),
    ...(record.disableModelInvocation?['disableModelInvocation: true']:[]),
    'createdAt: '+String(Number(record.createdAt)||Date.now())
  ];
  if(record.sourceRef){lines.push('metadata:');lines.push('  source: '+yamlString(record.sourceRef))}
  if(record.trigger){lines.push('trigger:');lines.push('  schedule: '+yamlString(record.trigger.schedule));lines.push('  enabled: '+String(record.trigger.isEnabled!==false))}
  lines.push('---');lines.push(record.body.trim());lines.push('');
  return lines.join('\n');
}
function createWorkflowManager({app}){
  const root=path.join(app.getPath('userData'),'workflows');
  async function ensureRoot(){await fs.mkdir(root,{recursive:true,mode:0o755})}
  async function list(){
    await ensureRoot();const dirs=await fs.readdir(root,{withFileTypes:true});const rows=[];
    for(const dirent of dirs){
      if(!dirent.isDirectory())continue;
      const filePath=path.join(root,dirent.name,WORKFLOW_FILENAME);
      try{const values=await Promise.all([fs.readFile(filePath,'utf8'),fs.stat(filePath)]);const parsed=parseFile(values[0],dirent.name,filePath,values[1]);if(parsed)rows.push(parsed)}catch{}
    }
    return rows.sort((a,b)=>b.updatedAt-a.updatedAt).slice(0,UI_LIMIT);
  }
  async function saveWorkflow(input){
    const name=line(input?.name,MAX_NAME),description=line(input?.description,MAX_DESCRIPTION),body=block(input?.body,MAX_BODY);
    if(!name)throw Error('Workflow name is required.');if(!body)throw Error('Workflow body is required.');
    const existing=input?.id?(await list()).find(x=>x.id===input.id):null;
    const id=existing?.id||crypto.randomUUID();
    const dirName=existing?path.basename(path.dirname(existing.filePath)):slugify(name)+'-'+id.slice(0,8);
    const dir=path.join(root,dirName),filePath=path.join(dir,WORKFLOW_FILENAME);
    const trigger=input?.trigger&&String(input.trigger.schedule||'').trim()
      ?{schedule:String(input.trigger.schedule).trim().replace(/\s+/g,' '),isEnabled:input.trigger.isEnabled!==false}:null;
    const record={id,name,description,body,trigger,source:'workflow',sourceRef:existing?.sourceRef||null,
      isEnabledForAgent:input?.isEnabledForAgent!==false,disableModelInvocation:input?.disableModelInvocation===true,
      createdAt:existing?.createdAt||Date.now(),updatedAt:Date.now(),helperScripts:[],filePath};
    await ensureRoot();await fs.mkdir(dir,{recursive:true,mode:0o755});
    const tmp=filePath+'.tmp';await fs.writeFile(tmp,serialize(record),{mode:0o644});await fs.rename(tmp,filePath);await fs.chmod(filePath,0o644).catch(()=>{});
    return record;
  }
  async function deleteWorkflow({id}){
    const record=(await list()).find(x=>x.id===id);if(!record)throw Error('Workflow not found.');
    await fs.rm(path.dirname(record.filePath),{recursive:true,force:true});return{ok:true};
  }
  async function setWorkflowEnabled({id,enabled}){
    const record=(await list()).find(x=>x.id===id);if(!record)throw Error('Workflow not found.');
    return saveWorkflow({...record,isEnabledForAgent:enabled===true});
  }
  async function buildAgentContext(prompt=''){
    const workflows=(await list()).filter(x=>x.isEnabledForAgent&&!x.disableModelInvocation).slice(0,MAX_PER_AGENT);
    if(!workflows.length)return'';
    const lower=String(prompt).toLowerCase();
    const selected=workflows.filter(workflow=>{
      const name=workflow.name.toLowerCase(),id=workflow.id.toLowerCase();
      return lower.includes('@'+name)||lower.includes('@'+name.replace(/\s+/g,''))||lower.includes('sand-workflow:'+id);
    });
    const catalog=['Available private skills/workflows:'];
    for(const workflow of workflows)catalog.push('- '+workflow.name+' ['+workflow.id+']: '+(workflow.description||'No description'));
    if(selected.length){
      catalog.push('Selected workflow instructions:');
      for(const workflow of selected)catalog.push('### '+workflow.name+'\n'+workflow.body.slice(0,INJECT_LIMIT));
    }
    return catalog.join('\n');
  }
  return{root,list,saveWorkflow,deleteWorkflow,setWorkflowEnabled,buildAgentContext};
}
module.exports={createWorkflowManager,WORKFLOW_FILENAME};
