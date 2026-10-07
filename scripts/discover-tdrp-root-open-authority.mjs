#!/usr/bin/env node
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const fail=(ok,msg)=>{if(!ok) throw new Error(msg);};
const run=(cmd,args,opts={})=>{
  const r=spawnSync(cmd,args,{encoding:'utf8',maxBuffer:64*1024*1024,...opts});
  if(r.status!==0) throw new Error(cmd+' '+args.join(' ')+' failed '+r.status+'\n'+(r.stderr||''));
  return r.stdout;
};
const tryRun=(cmd,args,opts={})=>spawnSync(cmd,args,{encoding:'utf8',maxBuffer:64*1024*1024,...opts});
const inventory=JSON.parse(await fs.readFile('projects/telegram-desktop-rust/inventory/build-time-acquisitions.json','utf8'));
const lock=JSON.parse(await fs.readFile('projects/telegram-desktop-rust/upstream.lock.json','utf8'));
const openDispositions=new Set(['mutable-tag-or-branch-input','tool-bootstrap','versioned-download-input']);
const selectedGroups=(inventory.candidate_disposition_groups||[]).filter(g=>openDispositions.has(g.disposition));
const selectedKeys=new Map();
for(const group of selectedGroups){
  for(const line of group.source_lines||[]){
    const key=group.source_path+':'+line;
    fail(!selectedKeys.has(key),'duplicate root authority candidate '+key);
    selectedKeys.set(key,{source_path:group.source_path,source_line:line,disposition:group.disposition,authority_scope:group.authority_scope});
  }
}

const temp=await fs.mkdtemp(path.join(process.env.RUNNER_TEMP||os.tmpdir(),'tdrp-root-authority-'));
const upstream=path.join(temp,'tdesktop');
run('git',['clone','--filter=blob:none','--no-checkout','--quiet','https://github.com/'+lock.upstream.repository+'.git',upstream]);
run('git',['-C',upstream,'checkout','--detach','--quiet',lock.upstream.commit]);
fail(run('git',['-C',upstream,'rev-parse','HEAD']).trim()===lock.upstream.commit,'upstream checkout drift');

const byPath=new Map();
for(const item of selectedKeys.values()){
  if(!byPath.has(item.source_path)){
    const raw=await fs.readFile(path.join(upstream,item.source_path),'utf8');
    byPath.set(item.source_path,{raw,lines:raw.split(/\r?\n/)});
  }
}
const candidates=[...selectedKeys.values()].sort((a,b)=>a.source_path.localeCompare(b.source_path)||a.source_line-b.source_line).map(item=>{
  const lines=byPath.get(item.source_path).lines;
  fail(item.source_line>=1 && item.source_line<=lines.length,'candidate line outside file '+item.source_path+':'+item.source_line);
  return {...item,text:lines[item.source_line-1],context:lines.slice(Math.max(0,item.source_line-3),Math.min(lines.length,item.source_line+2)).join('\n')};
});

function logicalLines(lines){
  const out=[];
  for(let i=0;i<lines.length;i++){
    const start=i+1;
    let text=lines[i];
    let end=start;
    while(/\\\s*$/.test(text) && i+1<lines.length){
      text=text.replace(/\\\s*$/,' ') + lines[++i].trim();
      end=i+1;
    }
    out.push({start,end,text});
  }
  return out;
}
function cleanToken(s){
  return s.replace(/^['"]|['",;)]$/g,'').replace(/\\$/,'');
}
function normalizeRemote(url){
  let u=cleanToken(url);
  if(/^git@github\.com:/.test(u)) u='https://github.com/'+u.slice('git@github.com:'.length);
  return u;
}
function resolveRef(url,ref,hint){
  if(!url||!ref) return {status:'missing-repository-or-ref'};
  if(/[\$%{}+]/.test(ref)||/\$/.test(url)) return {status:'expression',repository:url,ref,hint};
  const repo=normalizeRemote(url);
  const queries=[];
  if(hint==='branch') queries.push('refs/heads/'+ref);
  else if(hint==='tag') queries.push('refs/tags/'+ref,'refs/tags/'+ref+'^{}');
  else queries.push('refs/heads/'+ref,'refs/tags/'+ref,'refs/tags/'+ref+'^{}');
  const r=tryRun('git',['ls-remote',repo,...queries]);
  if(r.status!==0) return {status:'resolution-error',repository:repo,ref,hint,error:(r.stderr||'').trim()};
  const matches=(r.stdout||'').split(/\r?\n/).filter(Boolean).map(line=>{
    const m=line.match(/^([0-9a-f]{40})\s+(.+)$/);
    return m?{sha:m[1],name:m[2]}:null;
  }).filter(Boolean);
  if(!matches.length) return {status:'ref-not-found',repository:repo,ref,hint};
  const peeled=matches.find(m=>m.name.endsWith('^{}'));
  const heads=matches.filter(m=>m.name.startsWith('refs/heads/'));
  const tags=matches.filter(m=>m.name.startsWith('refs/tags/')&&!m.name.endsWith('^{}'));
  const chosen=peeled||heads[0]||tags[0];
  return {status:'resolved',repository:repo,ref,hint,commit:chosen.sha,matches};
}

const acquisitions=[];
const covered=new Set();

// Dockerfile / prepare.py git clone -b forms, including shell line continuations.
for(const [sourcePath,data] of byPath){
  if(!['Telegram/build/docker/centos_env/Dockerfile','Telegram/build/prepare/prepare.py'].includes(sourcePath)) continue;
  for(const logical of logicalLines(data.lines)){
    if(!/\bgit clone\b/.test(logical.text)||!/(?:^|\s)-b\s+/.test(logical.text)) continue;
    const refMatch=logical.text.match(/(?:^|\s)-b\s+([^\s]+)/);
    const urls=[...logical.text.matchAll(/(?:https?:\/\/[^\s'"]+|git@github\.com:[^\s'"]+)/g)].map(m=>m[0]);
    const url=urls.at(-1)||null;
    const ref=refMatch?cleanToken(refMatch[1]):null;
    const keys=[...selectedKeys.values()].filter(x=>x.source_path===sourcePath&&x.source_line>=logical.start&&x.source_line<=logical.end);
    if(!keys.length) continue;
    const resolution=resolveRef(url,ref,null);
    acquisitions.push({kind:'git-clone-ref',source_path:sourcePath,line_start:logical.start,line_end:logical.end,repository:url,ref,resolution,candidate_lines:keys.map(x=>x.source_line)});
    for(const k of keys) covered.add(k.source_path+':'+k.source_line);
  }
}

// Snapcraft source/source-tag/source-branch pairs.
{
  const sourcePath='snap/snapcraft.yaml';
  const data=byPath.get(sourcePath);
  if(data){
    let currentPart=null,source=null,sourceLine=null;
    for(let i=0;i<data.lines.length;i++){
      const line=data.lines[i];
      const part=line.match(/^  ([A-Za-z0-9_.-]+):\s*$/);
      if(part){currentPart=part[1];source=null;sourceLine=null;continue;}
      const sm=line.match(/^    source:\s*(\S+)\s*$/);
      if(sm){source=cleanToken(sm[1]);sourceLine=i+1;continue;}
      const rm=line.match(/^    source-(tag|branch):\s*(\S+)\s*$/);
      if(rm&&source){
        const ref=cleanToken(rm[2]);
        const hint=rm[1];
        const lines=[sourceLine,i+1].filter(n=>selectedKeys.has(sourcePath+':'+n));
        if(lines.length){
          acquisitions.push({kind:'snap-source-ref',part:currentPart,source_path:sourcePath,line_start:sourceLine,line_end:i+1,repository:source,ref,ref_kind:hint,resolution:resolveRef(source,ref,hint),candidate_lines:lines});
          for(const n of lines) covered.add(sourcePath+':'+n);
        }
      }
    }
  }
}

const candidateDetails=candidates.map(item=>({
  ...item,
  covered_by_resolution:covered.has(item.source_path+':'+item.source_line),
}));
const unresolvedCandidates=candidateDetails.filter(item=>item.disposition==='mutable-tag-or-branch-input'&&!item.covered_by_resolution);
const resolutions=acquisitions.map(a=>a.resolution);
const summary={
  target_commit:process.env.GITHUB_SHA||null,
  upstream_commit:lock.upstream.commit,
  candidate_lines_total:candidateDetails.length,
  candidate_lines_by_disposition:Object.fromEntries([...openDispositions].map(d=>[d,candidateDetails.filter(x=>x.disposition===d).length])),
  mutable_candidate_lines:candidateDetails.filter(x=>x.disposition==='mutable-tag-or-branch-input').length,
  mutable_candidate_lines_covered_by_parser:candidateDetails.filter(x=>x.disposition==='mutable-tag-or-branch-input'&&x.covered_by_resolution).length,
  mutable_candidate_lines_unparsed:unresolvedCandidates.length,
  acquisitions:acquisitions.length,
  resolved_acquisitions:resolutions.filter(x=>x.status==='resolved').length,
  expression_acquisitions:resolutions.filter(x=>x.status==='expression').length,
  missing_or_error_acquisitions:resolutions.filter(x=>!['resolved','expression'].includes(x.status)).length,
};
const report={project_id:'TDRP-001',spec_revision:9,...summary,acquisitions,candidates:candidateDetails,unparsed_mutable_candidates:unresolvedCandidates};
await fs.mkdir('artifacts/tdrp-authority',{recursive:true});
await fs.writeFile('artifacts/tdrp-authority/root-open-authority-discovery.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(summary,null,2));
