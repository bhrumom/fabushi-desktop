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
  if(/[\\$%{}+]/.test(ref)||/\\$/.test(url)) return {status:'expression',repository:url,ref,hint};
  const repo=normalizeRemote(url);
  const queries=['refs/heads/'+ref,'refs/tags/'+ref,'refs/tags/'+ref+'^{}'];
  const r=tryRun('git',['ls-remote',repo,...queries]);
  if(r.status!==0) return {status:'resolution-error',repository:repo,ref,hint,error:(r.stderr||'').trim()};
  const matches=(r.stdout||'').split(/\\r?\\n/).filter(Boolean).map(line=>{
    const m=line.match(/^([0-9a-f]{40})\\s+(.+)$/);
    return m?{sha:m[1],name:m[2]}:null;
  }).filter(Boolean);
  if(!matches.length) return {status:'ref-not-found',repository:repo,ref,hint};
  const peeled=matches.find(m=>m.name.endsWith('^{}'));
  const heads=matches.filter(m=>m.name.startsWith('refs/heads/'));
  const tags=matches.filter(m=>m.name.startsWith('refs/tags/')&&!m.name.endsWith('^{}'));
  let chosen=null;
  let resolvedKind=null;
  if(hint==='branch'){
    chosen=heads[0]||peeled||tags[0];
    resolvedKind=heads[0]?'branch':'tag-fallback';
  } else if(hint==='tag'){
    chosen=peeled||tags[0]||heads[0];
    resolvedKind=(peeled||tags[0])?'tag':'branch-fallback';
  } else {
    chosen=peeled||heads[0]||tags[0];
    resolvedKind=peeled||tags[0]?'tag':'branch';
  }
  return {status:'resolved',repository:repo,ref,hint,resolved_kind:resolvedKind,commit:chosen.sha,matches};
}

function resolveHead(url){
  const repo=normalizeRemote(url);
  const r=tryRun('git',['ls-remote',repo,'HEAD']);
  if(r.status!==0) return {status:'resolution-error',repository:repo,ref:'HEAD',error:(r.stderr||'').trim()};
  const m=(r.stdout||'').match(/^([0-9a-f]{40})\\s+HEAD/m);
  return m?{status:'resolved',repository:repo,ref:'HEAD',resolved_kind:'default-head-snapshot',commit:m[1],matches:[{sha:m[1],name:'HEAD'}]}:
    {status:'ref-not-found',repository:repo,ref:'HEAD'};
}

function resolveDockerImage(image){
  const r=tryRun('docker',['buildx','imagetools','inspect',image]);
  if(r.status!==0) return {status:'resolution-error',image,error:(r.stderr||'').trim()};
  const m=(r.stdout||'').match(/^Digest:\\s+(sha256:[0-9a-f]{64})$/m);
  return m?{status:'resolved',image,digest:m[1],resolved_kind:'oci-manifest-digest'}:{status:'digest-not-found',image,output:(r.stdout||'').slice(0,2000)};
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
    let ref=refMatch?cleanToken(refMatch[1]):null;
    const keys=[...selectedKeys.values()].filter(x=>x.source_path===sourcePath&&x.source_line>=logical.start&&x.source_line<=logical.end);
    if(!keys.length) continue;
    let authorityNote=null;
    if(sourcePath==='Telegram/build/docker/centos_env/Dockerfile' && ref==='v$QT'){
      const preceding=data.lines.slice(Math.max(0,logical.start-8),logical.start-1).reverse().find(line=>/^QT=/.test(line.trim()));
      const value=preceding?.trim().match(/^QT=([^\\s]+)$/)?.[1]||null;
      fail(value,'unable to resolve Docker QT variable at line '+logical.start);
      ref='v'+value;
      authorityNote='resolved from preceding QT assignment in same Docker RUN block';
    }
    if(sourcePath==='Telegram/build/prepare/prepare.py' && logical.start===1599){
      ref='v5.15.19-lts-lgpl';
      authorityNote='qt<6 branch; accepted qt_version.py selects Qt 5.15.19 for legacy Windows path';
    }
    if(sourcePath==='Telegram/build/prepare/prepare.py' && logical.start===1668){
      ref='v6.11.2';
      authorityNote='qt>=6 branch; accepted qt_version.py selects Qt 6.11.2 and this version is not 6.2 LTS';
    }
    const resolution=resolveRef(url,ref,null);
    acquisitions.push({kind:'git-clone-ref',source_path:sourcePath,line_start:logical.start,line_end:logical.end,repository:url,ref,resolution,authority_note:authorityNote,candidate_lines:keys.map(x=>x.source_line)});
    for(const k of keys) covered.add(k.source_path+':'+k.source_line);
  }
}

// Mutable default-branch clones that are either followed by an explicit checkout or need a snapshot.
for(const sourcePath of ['Telegram/build/prepare/prepare.py']){
  const data=byPath.get(sourcePath);
  if(!data) continue;
  for(const item of [...selectedKeys.values()].filter(x=>x.source_path===sourcePath&&x.disposition==='mutable-tag-or-branch-input'&&!covered.has(x.source_path+':'+x.source_line))){
    const text=data.lines[item.source_line-1];
    const clone=text.match(/git clone\\s+(https?:\\/\\/[^\\s'"]+)/);
    if(!clone) continue;
    const url=cleanToken(clone[1]);
    if(item.source_line===1067){
      const nearby=data.lines.slice(item.source_line,Math.min(data.lines.length,item.source_line+8)).join('\\n');
      const checkout=nearby.match(/git checkout\\s+([^\\s]+)/);
      fail(checkout,'libvpx mutable clone lacks nearby checkout');
      const ref=cleanToken(checkout[1]);
      acquisitions.push({kind:'git-clone-followed-by-checkout',source_path:sourcePath,line_start:item.source_line,line_end:item.source_line+8,repository:url,ref,resolution:resolveRef(url,ref,null),candidate_lines:[item.source_line]});
    } else if(item.source_line===1417){
      const windowsCommit=(inventory.resolved_short_ref_contexts||[]).find(x=>x.source_path===sourcePath&&x.source_line===1417)?.resolved_commit;
      fail(/^[0-9a-f]{40}$/.test(windowsCommit||''),'openal Windows checkout full commit missing');
      const macRef='coreaudio_device_uid';
      acquisitions.push({kind:'git-clone-platform-checkouts',source_path:sourcePath,line_start:item.source_line,line_end:item.source_line+20,repository:url,
        platform_authorities:[
          {platform:'windows',ref:windowsCommit,resolution:{status:'resolved',repository:url,ref:windowsCommit,resolved_kind:'commit',commit:windowsCommit}},
          {platform:'macos',ref:macRef,resolution:resolveRef(url,macRef,'branch')}
        ],candidate_lines:[item.source_line]});
    } else {
      acquisitions.push({kind:'git-clone-default-head-snapshot',source_path:sourcePath,line_start:item.source_line,line_end:item.source_line,repository:url,ref:'HEAD',resolution:resolveHead(url),candidate_lines:[item.source_line]});
    }
    covered.add(item.source_path+':'+item.source_line);
  }
}

// Docker base image tag is build-toolchain authority; snapshot its immutable manifest digest.
{
  const key='Telegram/build/docker/centos_env/Dockerfile:3';
  if(selectedKeys.has(key) && !covered.has(key)){
    const line=byPath.get('Telegram/build/docker/centos_env/Dockerfile').lines[2];
    const m=line.match(/^FROM\\s+(\\S+)/);
    fail(m,'unable to parse Docker base image authority');
    const image=m[1];
    acquisitions.push({kind:'docker-base-image',source_path:'Telegram/build/docker/centos_env/Dockerfile',line_start:3,line_end:3,image,resolution:resolveDockerImage(image),candidate_lines:[3]});
    covered.add(key);
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
const resolutions=acquisitions.flatMap(a=>a.platform_authorities ? a.platform_authorities.map(x=>x.resolution) : [a.resolution]).filter(Boolean);
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
