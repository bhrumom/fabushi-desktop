#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { spawnSync } from 'node:child_process';

const fail=(ok,msg)=>{if(!ok)throw new Error(msg);};
const run=(cmd,args,opts={})=>{const r=spawnSync(cmd,args,{encoding:'utf8',maxBuffer:256*1024*1024,timeout:300000,...opts});if(r.status!==0)throw new Error(cmd+' '+args.join(' ')+' failed '+r.status+'\n'+(r.stderr||''));return r.stdout;};
const tryRun=(cmd,args,opts={})=>spawnSync(cmd,args,{encoding:'utf8',maxBuffer:256*1024*1024,timeout:300000,...opts});

const targetCommit=process.env.TDRP_TARGET_SHA||process.env.GITHUB_SHA;
fail(/^[0-9a-f]{40}$/.test(targetCommit||''),'target commit is missing or invalid');
const discovery=JSON.parse(await fs.readFile('artifacts/tdrp-authority/root-tool-authority-discovery.json','utf8'));
const lock=JSON.parse(await fs.readFile('projects/telegram-desktop-rust/upstream.lock.json','utf8'));
fail(discovery.project_id==='TDRP-001'&&discovery.spec_revision===9,'root tool authority discovery schema drift');
fail(discovery.target_commit===targetCommit,'root tool authority discovery target_commit mismatch');
fail(discovery.upstream_commit===lock.upstream.commit,'root tool authority discovery upstream_commit does not match lock');
fail(/^[0-9a-f]{40}$/.test(lock.upstream.commit||''),'locked upstream commit invalid');
fail(/^[0-9a-f]{40}$/.test(lock.upstream.tree||''),'locked upstream tree invalid');
fail(lock.upstream.accepted===false&&lock.acceptance?.accepted===false&&lock.acceptance?.baseline_ready===false,'baseline unexpectedly accepted before full closure');

const authorityRecords=(discovery.records||[]).filter(x=>x.disposition==='immutable-source-archive');
fail(authorityRecords.length===2,'immutable source archive count drift '+authorityRecords.length);
const archives=[];
const archiveIdByCandidateKey=Object.freeze({
  'Telegram/build/docker/centos_env/Dockerfile:905':'ROOT-ARCHIVE-BOOST-190',
  'Telegram/build/prepare/prepare.py:806':'ROOT-ARCHIVE-LIBICONV-118'
});
for(const record of authorityRecords){
  const keys=record.candidate_keys||[];
  fail(keys.length===1,'immutable source archive must map to exactly one source archive candidate');
  const archiveId=record.id||archiveIdByCandidateKey[keys[0]];
  fail(typeof archiveId==='string'&&archiveId.length>0,'unrecognized immutable source archive candidate '+keys[0]);
  const item={...record,id:archiveId};
  fail(/^https?:\/\//.test(item.url||''),item.id+' URL invalid');
  fail(/^[0-9a-f]{64}$/.test(item.sha256||''),item.id+' SHA-256 invalid');
  const candidates=(discovery.candidates||[]).filter(c=>keys.includes(c.source_path+':'+c.source_line));
  fail(candidates.length===1,item.id+' source_path/source_line is not uniquely bound to live candidate authority');
  const candidate=candidates[0];
  fail(typeof candidate.source_path==='string'&&candidate.source_path.length>0,item.id+' source_path missing');
  fail(Number.isInteger(candidate.source_line)&&candidate.source_line>0,item.id+' source_line invalid');
  fail(candidate.disposition==='versioned-download-input',item.id+' source candidate disposition drift');
  fail(typeof candidate.text==='string'&&candidate.text.replaceAll('$VERSION',item.version).includes(item.url),item.id+' live upstream source line does not contain archive URL after pinned version expansion');
  fail(candidate.covered===true,item.id+' source archive candidate is not covered by current discovery');
  archives.push({...item,source_path:candidate.source_path,source_line:candidate.source_line,source_text:candidate.text,
    target_commit:targetCommit,upstream_commit:lock.upstream.commit,upstream_tree:lock.upstream.tree,
    lock_commit:lock.upstream.commit,lock_tree:lock.upstream.tree});
}
const acq=['git[[:space:]]+(clone|fetch|submodule)','curl[[:space:]]','wget[[:space:]]','git\\+https?://','GIT_REPOSITORY','GIT_TAG','URL[[:space:]]+https?://','pip(3)?[[:space:]]+install','cargo[[:space:]]+install','uses:[[:space:]]','^[[:space:]]*FROM[[:space:]]'].join('|');
const temp=await fs.mkdtemp(path.join(process.env.RUNNER_TEMP||os.tmpdir(),'tdrp-source-archives-'));
const reports=[];
for(const item of archives){
  const bundle=path.join(temp,item.id+'.tar.gz');
  const urls=[item.url];
  if(item.url.startsWith('https://ftp.gnu.org/pub/gnu/')){
    const relative=item.url.slice('https://ftp.gnu.org/pub/gnu/'.length);
    urls.push('https://mirrors.kernel.org/gnu/'+relative);
    urls.push('https://ftpmirror.gnu.org/'+relative);
  }
  let fetchedUrl=null;
  const downloadErrors=[];
  for(const url of urls){
    await fs.rm(bundle,{force:true});
    const downloaded=tryRun('curl',['-fsSL','--retry','3','--connect-timeout','20','--max-time','240','-o',bundle,url]);
    if(downloaded.status===0){fetchedUrl=url;break;}
    downloadErrors.push(url+': '+String(downloaded.stderr||'').trim());
  }
  fail(fetchedUrl,item.id+' archive download failed across immutable transport candidates\n'+downloadErrors.join('\n'));
  const actual=run('sha256sum',[bundle]).trim().split(/\s+/)[0];
  fail(actual===item.sha256,item.id+' sha256 mismatch against same-head authority discovery '+actual);
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
    id:item.id,url:item.url,fetched_url:fetchedUrl,transport_candidates:urls,version:item.version,
    source_path:item.source_path,source_line:item.source_line,source_text:item.source_text,
    expected_sha256:item.sha256,actual_sha256:actual,target_commit:targetCommit,
    upstream_commit:lock.upstream.commit,upstream_tree:lock.upstream.tree,
    lock_commit:item.lock_commit,lock_tree:item.lock_tree,
    archive_entries_non_directory:listing.filter(x=>!x.endsWith('/')).length,extracted_files:files.length,
    acquisition_candidates:(grep.stdout||'').split(/\r?\n/).filter(Boolean).map(relLine).sort(),
    patches:select(/\.(patch|diff)$/i),
    lfs_pointers:(lfs.stdout||'').split(/\r?\n/).filter(Boolean).map(relLine).sort(),
    generated_input_contracts:select(/(^|\/)(generate|generator|generators|gen)(\/|_|\. )|\.(gn|gni|gyp|gypi|bzl)$|(^|\/)CMakeLists\.txt$/i),
    resource_inputs:select(/(^|\/)(resources?|assets?|locales?|translations?|icons?|testdata)(\/|$)/i),
    tool_and_package_inputs:select(/(^|\/)(tools?|scripts?|build|infra)(\/|$)|(^|\/)(Cargo\.lock|package-lock\.json|pnpm-lock\.yaml|yarn\.lock|requirements[^/]*\.txt|vcpkg\.json|conanfile[^/]*|Dockerfile|Makefile|WORKSPACE(?:\.bazel)?|MODULE\.bazel)$/i),
    recursive_disposition_status:'open'
  });
}
const totals=reports.reduce((o,r)=>{o.archives++;o.entries+=r.archive_entries_non_directory;o.files+=r.extracted_files;o.acquisitions+=r.acquisition_candidates.length;o.patches+=r.patches.length;o.lfs+=r.lfs_pointers.length;o.generated+=r.generated_input_contracts.length;o.resources+=r.resource_inputs.length;o.tools+=r.tool_and_package_inputs.length;return o;},{archives:0,entries:0,files:0,acquisitions:0,patches:0,lfs:0,generated:0,resources:0,tools:0});
const report={format_version:1,project_id:'TDRP-001',spec_revision:9,target_commit:targetCommit,
  upstream_commit:lock.upstream.commit,upstream_tree:lock.upstream.tree,lock_commit:lock.upstream.commit,lock_tree:lock.upstream.tree,
  baseline_identity_accepted:false,source_closure_ready:false,recursive_disposition_status:'open',
  source_authority_target_commit:discovery.target_commit,source_authority_upstream_commit:discovery.upstream_commit,
  totals,closure_status:'archive-discovery-complete-disposition-pending',archives:reports};
await fs.mkdir('artifacts/tdrp-root-source-archives',{recursive:true});
await fs.writeFile('artifacts/tdrp-root-source-archives/root-source-archive-discovery.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({target_commit:report.target_commit,upstream_commit:report.upstream_commit,upstream_tree:report.upstream_tree,lock_commit:report.lock_commit,lock_tree:report.lock_tree,baseline_identity_accepted:false,source_closure_ready:false,recursive_disposition_status:'open',totals,closure_status:report.closure_status},null,2));
