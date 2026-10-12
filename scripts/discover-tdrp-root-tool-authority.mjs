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
const inventory=JSON.parse(await fs.readFile('projects/telegram-desktop-rust/inventory/build-time-acquisitions.json','utf8'));
const lock=JSON.parse(await fs.readFile('projects/telegram-desktop-rust/upstream.lock.json','utf8'));
const wanted=new Set(['tool-bootstrap','versioned-download-input']);
const groups=(inventory.candidate_disposition_groups||[]).filter(g=>wanted.has(g.disposition));
const candidates=[];
for(const group of groups) for(const line of group.source_lines||[]) candidates.push({
  source_path:group.source_path,source_line:line,disposition:group.disposition,authority_scope:group.authority_scope,
});
candidates.sort((a,b)=>a.source_path.localeCompare(b.source_path)||a.source_line-b.source_line);

const temp=await fs.mkdtemp(path.join(process.env.RUNNER_TEMP||os.tmpdir(),'tdrp-tool-authority-'));
const upstream=path.join(temp,'tdesktop');
run('git',['clone','--filter=blob:none','--no-checkout','--quiet','https://github.com/'+lock.upstream.repository+'.git',upstream]);
run('git',['-C',upstream,'checkout','--detach','--quiet',lock.upstream.commit]);
fail(run('git',['-C',upstream,'rev-parse','HEAD']).trim()===lock.upstream.commit,'upstream checkout drift');
const cache=new Map();
async function sourceLine(item){
  if(!cache.has(item.source_path)) cache.set(item.source_path,(await fs.readFile(path.join(upstream,item.source_path),'utf8')).split(/\r?\n/));
  const lines=cache.get(item.source_path);
  fail(item.source_line>0&&item.source_line<=lines.length,'candidate line outside file');
  return lines[item.source_line-1];
}
function hashUrl(url,id){
  const dest=path.join(temp,id.replace(/[^A-Za-z0-9_.-]/g,'_'));
  run('curl',['-fsSL','--retry','3','--connect-timeout','20','--max-time','120','-o',dest,url]);
  const sha=run('sha256sum',[dest]).trim().split(/\s+/)[0];
  fail(/^[0-9a-f]{64}$/.test(sha),'download hash invalid for '+url);
  return {url,sha256:sha};
}
function resolveRef(url,ref){
  const r=spawnSync('git',['ls-remote',url,'refs/heads/'+ref,'refs/tags/'+ref,'refs/tags/'+ref+'^{}'],{encoding:'utf8',maxBuffer:8*1024*1024});
  if(r.status!==0) return {status:'resolution-error',repository:url,ref,error:(r.stderr||'').trim()};
  const matches=(r.stdout||'').split(/\r?\n/).filter(Boolean).map(x=>{const m=x.match(/^([0-9a-f]{40})\s+(.+)$/);return m?{sha:m[1],name:m[2]}:null;}).filter(Boolean);
  const peeled=matches.find(x=>x.name.endsWith('^{}'));
  const head=matches.find(x=>x.name.startsWith('refs/heads/'));
  const tag=matches.find(x=>x.name.startsWith('refs/tags/')&&!x.name.endsWith('^{}'));
  const chosen=peeled||head||tag;
  return chosen?{status:'resolved',repository:url,ref,commit:chosen.sha,matches}:{status:'ref-not-found',repository:url,ref};
}

const covered=new Set();
const records=[];
const add=(keys,record)=>{
  for(const key of keys){fail(!covered.has(key),'duplicate tool authority coverage '+key);covered.add(key);}
  records.push({...record,candidate_keys:keys});
};
const key=(p,l)=>p+':'+l;

// Immutable content snapshots for mutable tool bootstrap URLs.
const poetry=hashUrl('https://install.python-poetry.org','poetry-installer.py');
add([key('.github/workflows/docker.yml',27),key('.github/workflows/linux.yml',72)],{
  kind:'tool-bootstrap-content-snapshot',disposition:'non-product-source-toolchain',...poetry,
  rationale:'Poetry installer bootstraps CI/build tooling and carries no Telegram product responsibility; content is snapshot-hashed for authority accounting.'
});
const ccache=hashUrl('https://github.com/ccache/ccache/releases/download/v4.13.6/ccache-4.13.6-linux-x86_64-glibc.tar.xz','ccache-4.13.6.tar.xz');
add([key('.github/workflows/linux.yml',73)],{
  kind:'tool-bootstrap-versioned-archive',disposition:'non-product-source-toolchain',version:'4.13.6',...ccache
});
const rustup=hashUrl('https://sh.rustup.rs','rustup-init.sh');
add([key('snap/snapcraft.yaml',490),key('Telegram/build/docker/centos_env/Dockerfile',998),key('Telegram/build/prepare/prepare.py',597)],{
  kind:'tool-bootstrap-content-snapshot',disposition:'non-product-source-toolchain',...rustup,
  rationale:'rustup installer is toolchain bootstrap; accepted sources separately pin the Rust toolchain version.'
});

// Package-manager bootstrap commands are environment/toolchain authority, not upstream product source responsibility.
add([
  key('Telegram/build/docker/centos_env/Dockerfile',14),
  key('Telegram/build/docker/centos_env/Dockerfile',16),
  key('Telegram/build/docker/centos_env/Dockerfile',18),
  key('Telegram/build/docker/centos_env/Dockerfile',30),
],{
  kind:'package-manager-toolchain',disposition:'non-product-source-toolchain',
  rationale:'Rocky Linux dnf and pip packages provide compilers/build tools; they are not source inputs whose product responsibilities are migrated. The base image is separately resolved to an OCI digest.'
});
add([
  key('Telegram/build/prepare/prepare.py',541),
  key('Telegram/build/prepare/prepare.py',542),
  key('Telegram/build/prepare/prepare.py',556),
],{
  kind:'package-manager-toolchain',disposition:'non-product-source-toolchain',
  rationale:'MSYS2/pip bootstrap packages are build environment tools, not product source authority.'
});

// gyp@master is tool source, but is not product source; snapshot the moving ref to an immutable commit.
const gyp=resolveRef('https://chromium.googlesource.com/external/gyp','master');
fail(gyp.status==='resolved','gyp@master resolution failed: '+JSON.stringify(gyp));
add([key('Telegram/build/prepare/prepare.py',580),key('Telegram/build/prepare/prepare.py',583)],{
  kind:'tool-source-ref-snapshot',disposition:'non-product-source-toolchain',resolution:gyp,
  rationale:'gyp executes generation/build logic and is not a product responsibility source; moving master is nevertheless resolved to an immutable commit snapshot.'
});

// Actual source archives remain source authority and are content-addressed.
const boost=hashUrl('https://archives.boost.io/release/1.90.0/source/boost_1_90_0.tar.gz','boost-1.90.0.tar.gz');
add([key('Telegram/build/docker/centos_env/Dockerfile',903)],{
  kind:'source-archive-content-authority',disposition:'immutable-source-archive',version:'1.90.0',...boost
});
const libiconv=hashUrl('https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.18.tar.gz','libiconv-1.18.tar.gz');
add([key('Telegram/build/prepare/prepare.py',804)],{
  kind:'source-archive-content-authority',disposition:'immutable-source-archive',version:'1.18',...libiconv
});

const candidateDetails=[];
for(const item of candidates) candidateDetails.push({...item,text:await sourceLine(item),covered:covered.has(key(item.source_path,item.source_line))});
const missing=candidateDetails.filter(x=>!x.covered);
fail(missing.length===0,'uncovered tool/versioned authority candidates: '+JSON.stringify(missing));
fail(covered.size===candidates.length,'tool/versioned authority coverage cardinality drift');
const report={
  project_id:'TDRP-001',spec_revision:9,target_commit:process.env.GITHUB_SHA||null,upstream_commit:lock.upstream.commit,
  candidate_lines_total:candidates.length,
  tool_bootstrap_lines:candidates.filter(x=>x.disposition==='tool-bootstrap').length,
  versioned_download_lines:candidates.filter(x=>x.disposition==='versioned-download-input').length,
  covered_candidate_lines:covered.size,
  source_archive_records:records.filter(x=>x.disposition==='immutable-source-archive').length,
  non_product_source_toolchain_records:records.filter(x=>x.disposition==='non-product-source-toolchain').length,
  records,candidates:candidateDetails
};
await fs.mkdir('artifacts/tdrp-authority',{recursive:true});
await fs.writeFile('artifacts/tdrp-authority/root-tool-authority-discovery.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({
  target_commit:report.target_commit,upstream_commit:report.upstream_commit,candidate_lines_total:report.candidate_lines_total,
  tool_bootstrap_lines:report.tool_bootstrap_lines,versioned_download_lines:report.versioned_download_lines,
  covered_candidate_lines:report.covered_candidate_lines,source_archive_records:report.source_archive_records,
  non_product_source_toolchain_records:report.non_product_source_toolchain_records,
  gyp_commit:gyp.commit,boost_sha256:boost.sha256,libiconv_sha256:libiconv.sha256,
  poetry_installer_sha256:poetry.sha256,rustup_installer_sha256:rustup.sha256,ccache_sha256:ccache.sha256
},null,2));
