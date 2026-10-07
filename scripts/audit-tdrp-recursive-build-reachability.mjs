#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';

const fail=(ok,msg)=>{if(!ok)throw new Error(msg);};
const norm=(value)=>String(value||'').replace(/\.git$/,'').replace(/\/$/,'');
const keyOf=(repo,commit)=>norm(repo)+'@'+commit;

const reportDirs=(process.env.REPORT_DIRS||
  'artifacts/tdrp-layer-root:artifacts/tdrp-layer-child:artifacts/tdrp-layer-next:artifacts/tdrp-layer-closure')
  .split(':').filter(Boolean);

const readAuthorities=async(dir)=>{
  const files=(await fs.readdir(dir,{recursive:true})).filter(file=>
    /recursive-(root|child|next)-authority-shard-\d+\.json$/.test(file)
    || file==='recursive-closure-authorities.json'
  );
  const out=[];
  for(const file of files){
    const json=JSON.parse(await fs.readFile(path.join(dir,file),'utf8'));
    out.push(...(json.authorities||[]));
  }
  return out;
};

const all=new Map();
const rootKeys=new Set();
for(const dir of reportDirs){
  const authorities=await readAuthorities(dir);
  for(const authority of authorities){
    const key=keyOf(authority.repository,authority.commit);
    fail(!all.has(key),'duplicate recursive authority report: '+key);
    all.set(key,authority);
    if(/tdrp-layer-root$/.test(dir)) rootKeys.add(key);
  }
}
fail(all.size>0,'no recursive authority reports found');

const inventory=JSON.parse(await fs.readFile('projects/telegram-desktop-rust/inventory/build-time-acquisitions.json','utf8'));
const rules=inventory.recursive_build_reachability;
fail(rules?.status==='partial-audited-disposition-open','recursive reachability rules are missing/open-state drift');

const parentsByChild=new Map();
const childrenByParent=new Map();
for(const [childKey,authority] of all){
  const occurrences=authority.occurrences||[];
  for(const occurrence of occurrences){
    if(!occurrence.parent_repository||!occurrence.parent_commit) continue;
    const parentKey=keyOf(occurrence.parent_repository,occurrence.parent_commit);
    const parents=parentsByChild.get(childKey)||[];
    parents.push({parentKey,path:occurrence.path});
    parentsByChild.set(childKey,parents);
    const children=childrenByParent.get(parentKey)||[];
    children.push({childKey,path:occurrence.path});
    childrenByParent.set(parentKey,children);
  }
}

const excluded=new Map();
const reachableDirect=new Map();

const exclude=(key,reason,evidence)=>{
  if(rootKeys.has(key)) throw new Error('cannot exclude independently acquired root authority '+key);
  const prior=excluded.get(key);
  if(prior) return;
  excluded.set(key,{reason,evidence});
};

for(const qt of rules.qt_superproject?.observed_authorities||[]){
  const parentKey=keyOf(rules.qt_superproject.repository,qt.commit);
  fail(all.has(parentKey),'Qt parent authority missing from reports: '+parentKey);
  const selected=new Set(qt.selected_submodules||qt.selected_submodules_union||[]);
  fail(selected.size>0,'Qt selected submodule set missing: '+qt.commit);
  for(const edge of childrenByParent.get(parentKey)||[]){
    if(selected.has(edge.path)){
      reachableDirect.set(edge.childKey,{reason:'selected-by-accepted-tdesktop-qt-build',parent:parentKey,path:edge.path});
    }else{
      exclude(edge.childKey,'unselected-qt-superproject-gitlink',{parent:parentKey,path:edge.path});
    }
  }
}

const openssl=rules.openssl;
const opensslParent=keyOf(openssl.repository,openssl.commit);
fail(all.has(opensslParent),'OpenSSL parent authority missing from reports');
for(const [childPath,childRepository,childCommit] of openssl.direct_gitlinks_not_fetched||[]){
  const childKey=keyOf(childRepository,childCommit);
  fail(all.has(childKey),'recorded OpenSSL child authority missing from reports: '+childKey);
  const occurrence=(parentsByChild.get(childKey)||[]).find(item=>item.parentKey===opensslParent&&item.path===childPath);
  fail(occurrence,'OpenSSL child occurrence drift: '+childPath);
  exclude(childKey,'openssl-submodule-not-fetched',{parent:opensslParent,path:childPath});
}

let changed=true;
while(changed){
  changed=false;
  for(const [childKey,parents] of parentsByChild){
    if(excluded.has(childKey)||rootKeys.has(childKey)||parents.length===0) continue;
    if(parents.every(item=>excluded.has(item.parentKey))){
      exclude(childKey,'all-known-parent-authorities-non-reachable',{parents});
      changed=true;
    }
  }
}

const totals=(authorities)=>authorities.reduce((out,authority)=>{
  out.authorities++;
  out.tree_entries+=authority.tree_entries_non_directory||0;
  out.gitlinks+=(authority.gitlinks||[]).length;
  out.acquisition_candidates+=(authority.acquisition_candidates||[]).length;
  out.patches+=(authority.patches||[]).length;
  out.lfs_pointers+=(authority.lfs_pointers||[]).length;
  out.generated_inputs+=(authority.generated_input_contracts||[]).length;
  out.resource_inputs+=(authority.resource_inputs||[]).length;
  out.tool_inputs+=(authority.tool_and_package_inputs||[]).length;
  return out;
},{authorities:0,tree_entries:0,gitlinks:0,acquisition_candidates:0,patches:0,lfs_pointers:0,generated_inputs:0,resource_inputs:0,tool_inputs:0});

const excludedAuthorities=[...excluded.keys()].map(key=>all.get(key)).filter(Boolean);
const remaining=[...all.entries()].filter(([key])=>!excluded.has(key)).map(([,authority])=>authority);
const parseCandidatePath=(line)=>{
  const match=String(line).match(/^HEAD:(.*?):(\d+):(.*)$/);
  return match?.[1]||null;
};
const classifyByPolicy=(candidate,policy)=>{
  const candidatePath=parseCandidatePath(candidate);
  if(candidatePath==null) return null;
  for(const rule of policy||[]){
    if(rule.path_exact&&candidatePath===rule.path_exact) return {path:candidatePath,disposition:rule.disposition,basis:rule.basis||null};
    if(rule.path_prefix&&candidatePath.startsWith(rule.path_prefix)) return {path:candidatePath,disposition:rule.disposition,basis:rule.basis||null};
    if(rule.path_suffix&&candidatePath.endsWith(rule.path_suffix)) return {path:candidatePath,disposition:rule.disposition,basis:rule.basis||null};
  }
  return null;
};

const rootCandidateAudits=[];
let rootCandidateDispositionedTotal=0;
let rootCandidateRemainingOpenTotal=0;
const auditRootCandidates=(authorityKey,policy,{requireComplete=false,scope})=>{
  const authority=all.get(authorityKey);
  fail(authority,'candidate-policy authority missing: '+authorityKey);
  fail(rootKeys.has(authorityKey),'candidate-policy authority is not a root authority: '+authorityKey);
  const counts={};
  const open=[];
  for(const candidate of authority.acquisition_candidates||[]){
    const classification=classifyByPolicy(candidate,policy);
    if(classification){
      counts[classification.disposition]=(counts[classification.disposition]||0)+1;
      rootCandidateDispositionedTotal++;
    }else{
      open.push(candidate);
      rootCandidateRemainingOpenTotal++;
    }
  }
  if(requireComplete) fail(open.length===0,scope+' candidate policy left '+open.length+' candidates open');
  rootCandidateAudits.push({
    authority:authorityKey,
    scope,
    candidate_total:(authority.acquisition_candidates||[]).length,
    dispositioned:(authority.acquisition_candidates||[]).length-open.length,
    open:open.length,
    disposition_counts:counts,
    open_candidates:open
  });
};

for(const qt of rules.qt_superproject?.observed_authorities||[]){
  auditRootCandidates(
    keyOf(rules.qt_superproject.repository,qt.commit),
    rules.qt_superproject.root_candidate_disposition_policy,
    {requireComplete:false,scope:'qt-superproject-root-candidates'}
  );
}
auditRootCandidates(
  opensslParent,
  openssl.root_candidate_disposition_policy,
  {requireComplete:openssl.root_candidate_policy_status==='complete-for-current-openssl-authority',scope:'openssl-root-candidates'}
);

const result={
  project_id:'TDRP-001',
  spec_revision:9,
  target_commit:process.env.GITHUB_SHA,
  accepted_upstream_commit:inventory.upstream_commit,
  scanned_authorities:all.size,
  independently_acquired_root_authorities:rootKeys.size,
  proven_reachable_direct_authorities:reachableDirect.size,
  proven_non_reachable_authorities:excluded.size,
  proven_non_reachable_totals:totals(excludedAuthorities),
  remaining_open_totals:totals(remaining),
  reachability_disposition_status:'partial-audited-open',
  root_candidate_dispositioned_total:rootCandidateDispositionedTotal,
  root_candidate_remaining_open_total:rootCandidateRemainingOpenTotal,
  root_candidate_audits:rootCandidateAudits,
  source_closure_ready:false,
  rule_scopes:[
    'qt-superproject-explicit-submodule-selection',
    'openssl-no-submodule-fetch'
  ],
  excluded_authorities:[...excluded.entries()].sort(([a],[b])=>a.localeCompare(b)).map(([key,meta])=>({key,...meta})),
  reachable_direct_authorities:[...reachableDirect.entries()].sort(([a],[b])=>a.localeCompare(b)).map(([key,meta])=>({key,...meta}))
};
const outDir='artifacts/tdrp-recursive-reachability';
await fs.mkdir(outDir,{recursive:true});
await fs.writeFile(path.join(outDir,'recursive-build-reachability.json'),JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify({
  scanned_authorities:result.scanned_authorities,
  proven_non_reachable_authorities:result.proven_non_reachable_authorities,
  proven_non_reachable_totals:result.proven_non_reachable_totals,
  remaining_open_totals:result.remaining_open_totals,
  root_candidate_dispositioned_total:result.root_candidate_dispositioned_total,
  root_candidate_remaining_open_total:result.root_candidate_remaining_open_total,
  reachability_disposition_status:result.reachability_disposition_status,
  source_closure_ready:false
},null,2));
