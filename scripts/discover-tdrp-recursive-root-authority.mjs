#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import { spawnSync } from 'node:child_process';

const fail=(ok,msg)=>{ if(!ok) throw new Error(msg); };
const run=(cmd,args,opts={})=>{
  const r=spawnSync(cmd,args,{encoding:'utf8',maxBuffer:128*1024*1024,...opts});
  if(r.status!==0) throw new Error(cmd+' '+args.join(' ')+' failed '+r.status+'\nstdout:\n'+(r.stdout||'')+'\nstderr:\n'+(r.stderr||''));
  return r.stdout;
};
const tryRun=(cmd,args,opts={})=>spawnSync(cmd,args,{encoding:'utf8',maxBuffer:128*1024*1024,...opts});
const normalizeRepo=(repo)=>{
  if(/^https?:\/\//.test(repo)) return repo.replace(/\.git$/,'');
  if(/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repo)) return 'https://github.com/'+repo;
  if(/^[A-Za-z0-9_.-]+\.[A-Za-z]+\//.test(repo)) return 'https://'+repo.replace(/\.git$/,'');
  return repo.replace(/\.git$/,'');
};
const cloneUrl=(repo)=> normalizeRepo(repo) + (normalizeRepo(repo).includes('github.com/') ? '.git' : '');
const authorityKey=(repo,commit)=>normalizeRepo(repo)+'@'+commit;
const shardFor=(key,count)=>crypto.createHash('sha256').update(key).digest().readUInt32BE(0)%count;

const root=JSON.parse(await fs.readFile('projects/telegram-desktop-rust/inventory/root-mutable-authorities.json','utf8'));
const inv=JSON.parse(await fs.readFile('projects/telegram-desktop-rust/inventory/build-time-acquisitions.json','utf8'));
const all=new Map();
const add=(source,repo,commit,metadata={})=>{
  if(!repo || !/^[0-9a-f]{40}$/.test(commit||'')) return;
  const n=normalizeRepo(repo), key=authorityKey(n,commit);
  const existing=all.get(key);
  if(existing){ existing.sources.push({source,...metadata}); return; }
  all.set(key,{id:null,repository:n,commit,sources:[{source,...metadata}]});
};
for(const a of root.authorities||[]) add('root-mutable',a.repository,a.commit,{authority_id:a.id,occurrences:a.occurrences});
for(const a of inv.immutable_commit_pin_occurrences||[]) add('root-exact-pin',a.repository,a.commit,{authority_id:a.id,source_path:a.source_path,source_line:a.source_line});
for(const a of inv.resolved_short_ref_contexts||[]) add('root-resolved-short-ref',a.repository,a.resolved_commit,{source_path:a.source_path,source_line:a.source_line,observed_ref:a.observed_short_ref});
for(const a of inv.recursive_child_inputs||[]) add('known-recursive-child',a.repository,a.commit,{authority_id:a.id,parent:a.parent});
const authorities=[...all.values()].sort((a,b)=>authorityKey(a.repository,a.commit).localeCompare(authorityKey(b.repository,b.commit)));
authorities.forEach((a,i)=>a.id='ROOT-SOURCE-'+String(i+1).padStart(3,'0'));

const args=process.argv.slice(2);
const validateIndex=args.indexOf('--validate-reports');
if(validateIndex>=0){
  const dir=args[validateIndex+1];
  fail(dir,'--validate-reports requires directory');
  const files=(await fs.readdir(dir,{recursive:true})).filter(x=>/recursive-root-authority-shard-\d+\.json$/.test(x));
  fail(files.length===Number(process.env.SHARD_COUNT||12),'recursive shard report count drift: '+files.length);
  const seen=new Map();
  for(const rel of files){
    const report=JSON.parse(await fs.readFile(path.join(dir,rel),'utf8'));
    fail(report.target_commit===process.env.GITHUB_SHA,'shard report target HEAD drift: '+rel);
    for(const a of report.authorities||[]){
      const key=authorityKey(a.repository,a.commit);
      fail(!seen.has(key),'duplicate recursive authority report: '+key);
      seen.set(key,a);
    }
  }
  fail(seen.size===authorities.length,'recursive authority coverage drift: '+seen.size+' != '+authorities.length);
  for(const expected of authorities){
    const key=authorityKey(expected.repository,expected.commit);
    const actual=seen.get(key);
    fail(actual,'missing recursive authority report: '+key);
    fail(actual.checkout_commit===expected.commit,'recursive authority checkout drift: '+key);
    fail(Number.isInteger(actual.tree_entries_non_directory),'recursive authority missing tree accounting: '+key);
    fail(Array.isArray(actual.gitlinks)&&Array.isArray(actual.acquisition_candidates),'recursive authority incomplete child/input report: '+key);
  }
  const totals=[...seen.values()].reduce((o,a)=>{
    o.authorities++;
    o.tree_entries+=a.tree_entries_non_directory;
    o.gitlinks+=a.gitlinks.length;
    o.acquisition_candidates+=a.acquisition_candidates.length;
    o.patches+=a.patches.length;
    o.lfs_pointers+=a.lfs_pointers.length;
    o.generated_inputs+=a.generated_input_contracts.length;
    o.resource_inputs+=a.resource_inputs.length;
    o.tool_inputs+=a.tool_and_package_inputs.length;
    return o;
  },{authorities:0,tree_entries:0,gitlinks:0,acquisition_candidates:0,patches:0,lfs_pointers:0,generated_inputs:0,resource_inputs:0,tool_inputs:0});
  await fs.writeFile(path.join(dir,'recursive-root-authority-aggregate.json'),JSON.stringify({
    project_id:'TDRP-001',spec_revision:9,target_commit:process.env.GITHUB_SHA,
    expected_authorities:authorities.length,covered_authorities:seen.size,totals,
    closure_status:'discovery-complete-disposition-pending'
  },null,2)+'\n');
  console.log(JSON.stringify({expected_authorities:authorities.length,covered_authorities:seen.size,totals},null,2));
  process.exit(0);
}

const shard=Number(process.env.SHARD_INDEX);
const shardCount=Number(process.env.SHARD_COUNT||12);
fail(Number.isInteger(shard)&&shard>=0&&shard<shardCount,'invalid SHARD_INDEX');
const selected=authorities.filter(a=>shardFor(authorityKey(a.repository,a.commit),shardCount)===shard);
const tempRoot=await fs.mkdtemp(path.join(process.env.RUNNER_TEMP||os.tmpdir(),'tdrp-recursive-root-'));
const acquisitionPattern=[
 'git[[:space:]]+(clone|fetch|submodule)','curl[[:space:]]','wget[[:space:]]',
 '(^|[;&|[:space:]])(iwr|Invoke-WebRequest)[[:space:]]','git\\+https?://',
 '^[[:space:]]*source[[:space:]]*:','source-(url|commit|tag|branch):','GIT_REPOSITORY','GIT_TAG',
 'URL[[:space:]]+https?://','python(3(\\.[0-9]+)*)?[[:space:]]+-m[[:space:]]+pip[[:space:]]+install',
 'pip(3)?[[:space:]]+install','pacman[[:space:]]+-S','apt(-get)?[[:space:]]+(install|update)',
 'dnf[[:space:]][^#]*install','brew[[:space:]]+install','cargo[[:space:]]+install',
 'uses:[[:space:]]','^[[:space:]]*FROM[[:space:]]','https?://[^[:space:]"\\x27]+\\.git'
].join('|');
const parseTree=(raw)=>raw.split(/\r?\n/).filter(Boolean).map(line=>{
  const m=line.match(/^([0-9]{6})\s+(\S+)\s+([0-9a-f]{40})\t(.+)$/);
  fail(m,'unparseable ls-tree line: '+line);
  return {mode:m[1],type:m[2],object:m[3],path:m[4]};
});
const parseGitmodules=(raw)=>{
  const out=[]; let current=null;
  for(const line of raw.split(/\r?\n/)){
    const h=line.match(/^\s*\[submodule\s+"(.+)"\]\s*$/);
    if(h){ if(current) out.push(current); current={name:h[1],path:null,url:null,branch:null}; continue; }
    if(!current) continue;
    const p=line.match(/^\s*path\s*=\s*(.+?)\s*$/),u=line.match(/^\s*url\s*=\s*(.+?)\s*$/),b=line.match(/^\s*branch\s*=\s*(.+?)\s*$/);
    if(p) current.path=p[1]; if(u) current.url=u[1]; if(b) current.branch=b[1];
  }
  if(current) out.push(current); return out;
};
const selectedPaths=(entries,re)=>entries.filter(e=>e.type==='blob'&&re.test(e.path)).map(e=>e.path).sort();
const reports=[];
for(const authority of selected){
  const dir=path.join(tempRoot,authority.id);
  run('git',['clone','--filter=blob:none','--no-checkout','--quiet',cloneUrl(authority.repository),dir]);
  run('git',['-C',dir,'checkout','--detach','--quiet',authority.commit]);
  const checkout=run('git',['-C',dir,'rev-parse','HEAD']).trim();
  fail(checkout===authority.commit,authority.id+' checkout drift: '+checkout);
  const entries=parseTree(run('git',['-C',dir,'ls-tree','-r','HEAD']));
  const gitlinks=entries.filter(e=>e.mode==='160000'||e.type==='commit');
  const gm=tryRun('git',['-C',dir,'show','HEAD:.gitmodules']);
  const gitmodules=gm.status===0?parseGitmodules(gm.stdout):[];
  const linked=gitlinks.map(link=>{
    const mod=gitmodules.find(m=>m.path===link.path);
    return {...link,url:mod?.url||null,branch:mod?.branch||null};
  });
  for(const link of linked) fail(link.url,authority.id+' gitlink lacks .gitmodules URL: '+link.path);
  const grep=tryRun('git',['-C',dir,'grep','-n','-I','-E',acquisitionPattern,'HEAD','--','.']);
  fail(grep.status===0||grep.status===1,authority.id+' acquisition grep failed: '+(grep.stderr||''));
  const acquisition=(grep.stdout||'').split(/\r?\n/).filter(Boolean).sort();
  const attr=tryRun('git',['-C',dir,'show','HEAD:.gitattributes']);
  const attributes=attr.status===0?attr.stdout:'';
  const lfs=tryRun('git',['-C',dir,'grep','-n','-I','-F','version https://git-lfs.github.com/spec/v1','HEAD','--','.']);
  fail(lfs.status===0||lfs.status===1,authority.id+' lfs scan failed');
  const lfsPointers=(lfs.stdout||'').split(/\r?\n/).filter(Boolean).sort();
  const report={
    id:authority.id,repository:authority.repository,commit:authority.commit,checkout_commit:checkout,sources:authority.sources,
    tree_entries_non_directory:entries.length,gitlinks:linked,gitmodules,
    acquisition_candidates:acquisition,
    patches:selectedPaths(entries,/\.(patch|diff)$/i),
    lfs_attribute_detected:/filter=lfs|diff=lfs|merge=lfs/.test(attributes),lfs_pointers:lfsPointers,
    generated_input_contracts:selectedPaths(entries,/(^|\/)(generate|generator|generators|gen)(\/|_|\.)|\.(gn|gni|gyp|gypi|bzl)$|(^|\/)CMakeLists\.txt$/i),
    resource_inputs:selectedPaths(entries,/(^|\/)(resources?|assets?|locales?|translations?|icons?|testdata)(\/|$)/i),
    tool_and_package_inputs:selectedPaths(entries,/(^|\/)(tools?|scripts?|build|infra)(\/|$)|(^|\/)(Cargo\.lock|package-lock\.json|pnpm-lock\.yaml|yarn\.lock|Pipfile\.lock|poetry\.lock|requirements[^/]*\.txt|vcpkg\.json|conanfile[^/]*|Dockerfile|Makefile|WORKSPACE(?:\.bazel)?|MODULE\.bazel)$/i),
    recursive_disposition_status:'open'
  };
  reports.push(report);
  console.log(JSON.stringify({id:report.id,repository:report.repository,entries:report.tree_entries_non_directory,gitlinks:report.gitlinks.length,acquisitions:report.acquisition_candidates.length,patches:report.patches.length,lfs:report.lfs_pointers.length,generated:report.generated_input_contracts.length,resources:report.resource_inputs.length,tools:report.tool_and_package_inputs.length}));
}
const outDir='artifacts/tdrp-recursive-root';
await fs.mkdir(outDir,{recursive:true});
await fs.writeFile(path.join(outDir,'recursive-root-authority-shard-'+shard+'.json'),JSON.stringify({
 project_id:'TDRP-001',spec_revision:9,target_commit:process.env.GITHUB_SHA,shard,shard_count:shardCount,
 selected_authorities:selected.length,authorities:reports
},null,2)+'\n');
console.log(JSON.stringify({target_commit:process.env.GITHUB_SHA,shard,selected_authorities:selected.length,total_authorities:authorities.length},null,2));
