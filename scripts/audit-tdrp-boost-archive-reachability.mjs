#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';

const fail=(ok,msg)=>{if(!ok)throw new Error(msg);};
const reportPath=process.env.SOURCE_ARCHIVE_REPORT||'artifacts/tdrp-source-archives/root-source-archive-discovery.json';
const report=JSON.parse(await fs.readFile(reportPath,'utf8'));
const boost=(report.archives||[]).find(item=>item.id==='ROOT-ARCHIVE-BOOST-190');
fail(boost,'Boost 1.90.0 source archive report missing');
fail(boost.url==='https://archives.boost.io/release/1.90.0/source/boost_1_90_0.tar.gz','Boost archive URL drift');
fail(boost.actual_sha256===boost.expected_sha256,'Boost archive digest mismatch in source archive evidence');
fail(process.env.BOOST_BUILD_LOG_SHA256?.match(/^[0-9a-f]{64}$/),'Boost build execution log digest missing');

const classify=(line)=>{
  const match=String(line).match(/^boost_1_90_0\/(.*?):(\d+):(.*)$/);
  if(!match)return null;
  const candidatePath=match[1];
  const lower=candidatePath.toLowerCase();
  if(lower.startsWith('doc/')||lower.includes('/doc/')||lower.includes('readme')||lower.includes('contributing')||lower.endsWith('/about-nls')){
    return ['documentation-not-acquisition',candidatePath];
  }
  if(lower.includes('/test/')||lower.includes('/tests/')||lower.includes('/benchmark/')||lower.includes('/bench/')||lower.includes('/example/')||lower.includes('/examples/')||lower.includes('/fuzz')){
    return ['test-benchmark-example-fuzz-not-regex-build',candidatePath];
  }
  if(candidatePath.startsWith('libs/regex/')){
    return ['regex-non-build-documentation-or-tooling',candidatePath];
  }
  if(candidatePath.startsWith('libs/')){
    return ['non-regex-library-not-selected-by-with-libraries-regex',candidatePath];
  }
  if(candidatePath.startsWith('tools/build/azure-pipelines')){
    return ['boost-build-ci-only',candidatePath];
  }
  if(candidatePath.startsWith('tools/build/doc/')){
    return ['boost-build-documentation-not-invoked',candidatePath];
  }
  if(candidatePath.startsWith('tools/build/src/tools/doxygen.jam')){
    return ['doxygen-tool-not-invoked-by-regex-build',candidatePath];
  }
  if(candidatePath.startsWith('tools/boostdep/')||candidatePath.startsWith('tools/boostlook/')||candidatePath.startsWith('tools/cmake/')||candidatePath.startsWith('tools/quickbook/')){
    return ['uninvoked-boost-tooling',candidatePath];
  }
  return null;
};

const counts={};
const dispositions=[];
const open=[];
for(const candidate of boost.acquisition_candidates||[]){
  const classified=classify(candidate);
  if(classified==null){
    open.push(candidate);
    continue;
  }
  const [disposition,candidatePath]=classified;
  counts[disposition]=(counts[disposition]||0)+1;
  dispositions.push({candidate_path:candidatePath,disposition,candidate});
}
fail((boost.acquisition_candidates||[]).length===582,'Boost acquisition candidate count drift');
fail(open.length===0,'Boost build reachability left '+open.length+' acquisition candidates open');
fail(dispositions.length===(boost.acquisition_candidates||[]).length,'Boost acquisition disposition accounting drift');

const result={
  project_id:'TDRP-001',
  spec_revision:9,
  target_commit:process.env.GITHUB_SHA,
  accepted_upstream_commit:report.accepted_upstream_commit,
  archive_id:boost.id,
  archive_url:boost.url,
  archive_sha256:boost.actual_sha256,
  tdesktop_build_path:'Telegram/build/docker/centos_env/Dockerfile boost stage',
  executed_build_contract:[
    './bootstrap.sh --prefix=<temp>/install --with-libraries=regex',
    './b2 release link=static -j2 install'
  ],
  build_execution_log_sha256:process.env.BOOST_BUILD_LOG_SHA256,
  acquisition_candidates:(boost.acquisition_candidates||[]).length,
  dispositioned:dispositions.length,
  open:open.length,
  disposition_counts:counts,
  dispositions,
  status:'complete-for-boost-acquisition-candidates',
  source_closure_ready:false,
  note:'This closes only Boost archive acquisition candidates. Archive patches/generated/resources/tool inputs and all other authorities remain governed by their own open disposition gates.'
};
const outDir='artifacts/tdrp-boost-reachability';
await fs.mkdir(outDir,{recursive:true});
await fs.writeFile(path.join(outDir,'boost-build-reachability.json'),JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify({
  acquisition_candidates:result.acquisition_candidates,
  dispositioned:result.dispositioned,
  open:result.open,
  disposition_counts:result.disposition_counts,
  build_execution_log_sha256:result.build_execution_log_sha256,
  source_closure_ready:false
},null,2));
