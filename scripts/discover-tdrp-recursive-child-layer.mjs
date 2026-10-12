#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

const fail=(ok,msg)=>{if(!ok)throw new Error(msg);};
const run=(cmd,args,opts={})=>{const r=spawnSync(cmd,args,{encoding:'utf8',maxBuffer:128*1024*1024,...opts});if(r.status!==0)throw new Error(cmd+' '+args.join(' ')+' failed '+r.status+'\n'+(r.stderr||''));return r.stdout;};
const tryRun=(cmd,args,opts={})=>spawnSync(cmd,args,{encoding:'utf8',maxBuffer:128*1024*1024,...opts});
const norm=(u)=>u.replace(/\.git$/,'').replace(/\/$/,'');
const resolveUrl=(parent,url)=>{
  const p=norm(parent);
  if(url.startsWith('../')||url.startsWith('./')) return norm(new URL(url,p+'/').toString());
  if(url.startsWith('git@github.com:')) return norm('https://github.com/'+url.slice('git@github.com:'.length));
  if(/^https?:\/\//.test(url)) return norm(url);
  if(/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(url)) return 'https://github.com/'+url;
  return norm(url);
};
const cloneUrl=(repo)=>norm(repo)+(norm(repo).includes('github.com/')?'.git':'');
const keyOf=(repo,commit)=>norm(repo)+'@'+commit;
const shardFor=(key,n)=>crypto.createHash('sha256').update(key).digest().readUInt32BE(0)%n;
const readReports=async(dir)=>{
  const files=(await fs.readdir(dir,{recursive:true})).filter(f=>/recursive-(root|child)-authority-shard-\d+\.json$/.test(f));
  const reports=[];
  for(const f of files){const j=JSON.parse(await fs.readFile(path.join(dir,f),'utf8')); reports.push(...(j.authorities||[]));}
  return reports;
};
const parentDir=process.env.PARENT_REPORT_DIR||'artifacts/tdrp-parent-recursive';
const parentReports=await readReports(parentDir);
fail(parentReports.length>0,'no parent recursive reports found');
const already=new Set(parentReports.map(a=>keyOf(a.repository,a.commit)));
const children=new Map();
for(const parent of parentReports){
  for(const g of parent.gitlinks||[]){
    fail(g.url,'gitlink missing url at '+parent.repository+' '+g.path);
    const repo=resolveUrl(parent.repository,g.url), commit=g.object, k=keyOf(repo,commit);
    if(already.has(k)) continue;
    const cur=children.get(k);
    const occurrence={parent_repository:parent.repository,parent_commit:parent.commit,path:g.path,raw_url:g.url,branch:g.branch||null};
    if(cur){cur.occurrences.push(occurrence);continue;}
    children.set(k,{repository:repo,commit,occurrences:[occurrence]});
  }
}
const authorities=[...children.values()].sort((a,b)=>keyOf(a.repository,a.commit).localeCompare(keyOf(b.repository,b.commit)));
authorities.forEach((a,i)=>a.id='CHILD-LAYER-'+String(i+1).padStart(3,'0'));

const args=process.argv.slice(2);
const validate=args.indexOf('--validate-reports');
if(validate>=0){
  const dir=args[validate+1]; fail(dir,'missing report directory');
  const reports=await readReports(dir);
  const seen=new Map();
  for(const a of reports){const k=keyOf(a.repository,a.commit);fail(!seen.has(k),'duplicate child layer report '+k);seen.set(k,a);}
  fail(seen.size===authorities.length,'child layer authority coverage drift '+seen.size+' != '+authorities.length);
  for(const e of authorities){
    const k=keyOf(e.repository,e.commit), a=seen.get(k); fail(a,'missing child layer '+k); fail(a.checkout_commit===e.commit,'child checkout drift '+k);
  }
  const totals=[...seen.values()].reduce((o,a)=>{o.authorities++;o.tree_entries+=a.tree_entries_non_directory;o.gitlinks+=a.gitlinks.length;o.acquisition_candidates+=a.acquisition_candidates.length;o.patches+=a.patches.length;o.lfs_pointers+=a.lfs_pointers.length;o.generated_inputs+=a.generated_input_contracts.length;o.resource_inputs+=a.resource_inputs.length;o.tool_inputs+=a.tool_and_package_inputs.length;return o;},{authorities:0,tree_entries:0,gitlinks:0,acquisition_candidates:0,patches:0,lfs_pointers:0,generated_inputs:0,resource_inputs:0,tool_inputs:0});
  await fs.writeFile(path.join(dir,'recursive-child-layer-aggregate.json'),JSON.stringify({project_id:'TDRP-001',spec_revision:9,target_commit:process.env.GITHUB_SHA,parent_authorities:parentReports.length,expected_new_child_authorities:authorities.length,covered_new_child_authorities:seen.size,totals,closure_status:'layer-discovery-complete-further-recursion-pending'},null,2)+'\n');
  console.log(JSON.stringify({parent_authorities:parentReports.length,expected_new_child_authorities:authorities.length,covered_new_child_authorities:seen.size,totals},null,2));
  process.exit(0);
}
const shard=Number(process.env.SHARD_INDEX), count=Number(process.env.SHARD_COUNT||24);
fail(Number.isInteger(shard)&&shard>=0&&shard<count,'bad shard');
const selected=authorities.filter(a=>shardFor(keyOf(a.repository,a.commit),count)===shard);
const tmp=await fs.mkdtemp(path.join(process.env.RUNNER_TEMP||os.tmpdir(),'tdrp-child-layer-'));
const acq=['git[[:space:]]+(clone|fetch|submodule)','curl[[:space:]]','wget[[:space:]]','git\\+https?://','source-(url|commit|tag|branch):','GIT_REPOSITORY','GIT_TAG','URL[[:space:]]+https?://','pip(3)?[[:space:]]+install','cargo[[:space:]]+install','uses:[[:space:]]','^[[:space:]]*FROM[[:space:]]'].join('|');
const parseTree=raw=>raw.split(/\r?\n/).filter(Boolean).map(line=>{const m=line.match(/^([0-9]{6})\s+(\S+)\s+([0-9a-f]{40})\t(.+)$/);fail(m,'bad tree '+line);return{mode:m[1],type:m[2],object:m[3],path:m[4]};});
const parseGM=raw=>{const out=[];let c=null;for(const line of raw.split(/\r?\n/)){const h=line.match(/^\s*\[submodule\s+"(.+)"\]\s*$/);if(h){if(c)out.push(c);c={name:h[1],path:null,url:null,branch:null};continue;}if(!c)continue;const p=line.match(/^\s*path\s*=\s*(.+?)\s*$/),u=line.match(/^\s*url\s*=\s*(.+?)\s*$/),b=line.match(/^\s*branch\s*=\s*(.+?)\s*$/);if(p)c.path=p[1];if(u)c.url=u[1];if(b)c.branch=b[1];}if(c)out.push(c);return out;};
const paths=(entries,re)=>entries.filter(e=>e.type==='blob'&&re.test(e.path)).map(e=>e.path).sort();
const reports=[];
for(const authority of selected){
 const dir=path.join(tmp,authority.id);
 run('git',['clone','--filter=blob:none','--no-checkout','--quiet',cloneUrl(authority.repository),dir]);
 run('git',['-C',dir,'checkout','--detach','--quiet',authority.commit]);
 const checkout=run('git',['-C',dir,'rev-parse','HEAD']).trim();fail(checkout===authority.commit,'checkout drift '+authority.repository);
 const entries=parseTree(run('git',['-C',dir,'ls-tree','-r','HEAD']));
 const rawLinks=entries.filter(e=>e.mode==='160000'||e.type==='commit');
 const gm=tryRun('git',['-C',dir,'show','HEAD:.gitmodules']); const mods=gm.status===0?parseGM(gm.stdout):[];
 const links=rawLinks.map(g=>{const m=mods.find(x=>x.path===g.path);return{...g,url:m?.url||null,branch:m?.branch||null};});
 for(const g of links)fail(g.url,authority.repository+' gitlink lacks url '+g.path);
 const grep=tryRun('git',['-C',dir,'grep','-n','-I','-E',acq,'HEAD','--','.']);fail(grep.status===0||grep.status===1,'grep failed '+authority.repository);
 const attr=tryRun('git',['-C',dir,'show','HEAD:.gitattributes']);
 const lfs=tryRun('git',['-C',dir,'grep','-n','-I','-F','version https://git-lfs.github.com/spec/v1','HEAD','--','.']);fail(lfs.status===0||lfs.status===1,'lfs failed');
 const report={id:authority.id,repository:authority.repository,commit:authority.commit,checkout_commit:checkout,occurrences:authority.occurrences,tree_entries_non_directory:entries.length,gitlinks:links,gitmodules:mods,acquisition_candidates:(grep.stdout||'').split(/\r?\n/).filter(Boolean).sort(),patches:paths(entries,/\.(patch|diff)$/i),lfs_attribute_detected:/filter=lfs|diff=lfs|merge=lfs/.test(attr.status===0?attr.stdout:''),lfs_pointers:(lfs.stdout||'').split(/\r?\n/).filter(Boolean).sort(),generated_input_contracts:paths(entries,/(^|\/)(generate|generator|generators|gen)(\/|_|\.)|\.(gn|gni|gyp|gypi|bzl)$|(^|\/)CMakeLists\.txt$/i),resource_inputs:paths(entries,/(^|\/)(resources?|assets?|locales?|translations?|icons?|testdata)(\/|$)/i),tool_and_package_inputs:paths(entries,/(^|\/)(tools?|scripts?|build|infra)(\/|$)|(^|\/)(Cargo\.lock|package-lock\.json|pnpm-lock\.yaml|yarn\.lock|requirements[^/]*\.txt|vcpkg\.json|conanfile[^/]*|Dockerfile|Makefile|WORKSPACE(?:\.bazel)?|MODULE\.bazel)$/i),recursive_disposition_status:'open'};
 reports.push(report);console.log(JSON.stringify({id:report.id,repo:report.repository,entries:report.tree_entries_non_directory,gitlinks:links.length,acquisitions:report.acquisition_candidates.length}));
}
const out='artifacts/tdrp-recursive-child';await fs.mkdir(out,{recursive:true});await fs.writeFile(path.join(out,'recursive-child-authority-shard-'+shard+'.json'),JSON.stringify({project_id:'TDRP-001',spec_revision:9,target_commit:process.env.GITHUB_SHA,shard,shard_count:count,parent_authorities:parentReports.length,total_new_child_authorities:authorities.length,selected_authorities:selected.length,authorities:reports},null,2)+'\n');
console.log(JSON.stringify({target_commit:process.env.GITHUB_SHA,shard,parent_authorities:parentReports.length,total_new_child_authorities:authorities.length,selected:selected.length},null,2));
