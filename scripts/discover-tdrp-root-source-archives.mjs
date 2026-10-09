#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { spawnSync } from 'node:child_process';

const fail=(ok,msg)=>{if(!ok)throw new Error(msg);};
const run=(cmd,args,opts={})=>{const r=spawnSync(cmd,args,{encoding:'utf8',maxBuffer:256*1024*1024,timeout:300000,...opts});if(r.status!==0)throw new Error(cmd+' '+args.join(' ')+' failed '+r.status+'\n'+(r.stderr||''));return r.stdout;};
const tryRun=(cmd,args,opts={})=>spawnSync(cmd,args,{encoding:'utf8',maxBuffer:256*1024*1024,timeout:300000,...opts});
const authority=JSON.parse(await fs.readFile('projects/telegram-desktop-rust/inventory/root-tool-authorities.json','utf8'));
const archives=authority.immutable_source_archives||[];
fail(archives.length===2,'immutable source archive count drift '+archives.length);
const acq=['git[[:space:]]+(clone|fetch|submodule)','curl[[:space:]]','wget[[:space:]]','git\\+https?://','GIT_REPOSITORY','GIT_TAG','URL[[:space:]]+https?://','pip(3)?[[:space:]]+install','cargo[[:space:]]+install','uses:[[:space:]]','^[[:space:]]*FROM[[:space:]]'].join('|');
const temp=await fs.mkdtemp(path.join(process.env.RUNNER_TEMP||os.tmpdir(),'tdrp-source-archives-'));
const reports=[];
for(const item of archives){
  fail(/^https?:\/\//.test(item.url||''),item.id+' url invalid');
  fail(/^[0-9a-f]{64}$/.test(item.sha256||''),item.id+' sha invalid');
  const bundle=path.join(temp,item.id+'.tar.gz');
  run('curl',['-fsSL','--retry','3','--connect-timeout','20','--max-time','240','-o',bundle,item.url]);
  const actual=run('sha256sum',[bundle]).trim().split(/\s+/)[0];
  fail(actual===item.sha256,item.id+' sha256 drift '+actual);
  const listing=run('tar',['-tzf',bundle]).split(/\r?\n/).filter(Boolean);
  for(const e of listing)fail(!e.startsWith('/')&&!/(^|\/)\.\.(\/|$)/.test(e),item.id+' unsafe archive path '+e);
  const dir=path.join(temp,item.id);await fs.mkdir(dir,{recursive:true});run('tar',['-xzf',bundle,'-C',dir]);
  const files=run('find',[dir,'-type','f','-print']).split(/\r?\n/).filter(Boolean).map(p=>path.relative(dir,p)).sort();
  const grep=tryRun('grep',['-RInE','--exclude=*.patch','--exclude=*.diff',acq,dir]);
  fail(grep.status===0||grep.status===1,item.id+' acquisition scan failed');
  const lfs=tryRun('grep',['-RInF','version https://git-lfs.github.com/spec/v1',dir]);
  fail(lfs.status===0||lfs.status===1,item.id+' lfs scan failed');
  const relLine=line=>line.replaceAll(dir+path.sep,'');
  const select=re=>files.filter(p=>re.test(p));
  reports.push({
    id:item.id,url:item.url,version:item.version,expected_sha256:item.sha256,actual_sha256:actual,
    archive_entries_non_directory:listing.filter(x=>!x.endsWith('/')).length,extracted_files:files.length,
    acquisition_candidates:(grep.stdout||'').split(/\r?\n/).filter(Boolean).map(relLine).sort(),
    patches:select(/\.(patch|diff)$/i),
    lfs_pointers:(lfs.stdout||'').split(/\r?\n/).filter(Boolean).map(relLine).sort(),
    generated_input_contracts:select(/(^|\/)(generate|generator|generators|gen)(\/|_|\.)|\.(gn|gni|gyp|gypi|bzl)$|(^|\/)CMakeLists\.txt$/i),
    resource_inputs:select(/(^|\/)(resources?|assets?|locales?|translations?|icons?|testdata)(\/|$)/i),
    tool_and_package_inputs:select(/(^|\/)(tools?|scripts?|build|infra)(\/|$)|(^|\/)(Cargo\.lock|package-lock\.json|pnpm-lock\.yaml|yarn\.lock|requirements[^/]*\.txt|vcpkg\.json|conanfile[^/]*|Dockerfile|Makefile|WORKSPACE(?:\.bazel)?|MODULE\.bazel)$/i),
    recursive_disposition_status:'open'
  });
}
const totals=reports.reduce((o,r)=>{o.archives++;o.entries+=r.archive_entries_non_directory;o.files+=r.extracted_files;o.acquisitions+=r.acquisition_candidates.length;o.patches+=r.patches.length;o.lfs+=r.lfs_pointers.length;o.generated+=r.generated_input_contracts.length;o.resources+=r.resource_inputs.length;o.tools+=r.tool_and_package_inputs.length;return o;},{archives:0,entries:0,files:0,acquisitions:0,patches:0,lfs:0,generated:0,resources:0,tools:0});
const report={project_id:'TDRP-001',spec_revision:9,target_commit:process.env.GITHUB_SHA,accepted_upstream_commit:authority.accepted_upstream_commit,totals,closure_status:'archive-discovery-complete-disposition-pending',archives:reports};
await fs.mkdir('artifacts/tdrp-root-source-archives',{recursive:true});
await fs.writeFile('artifacts/tdrp-root-source-archives/root-source-archive-discovery.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({target_commit:report.target_commit,totals,closure_status:report.closure_status},null,2));
