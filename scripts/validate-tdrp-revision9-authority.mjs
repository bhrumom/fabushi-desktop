#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';

const root = process.cwd();
const requireAccepted = process.argv.includes('--require-accepted');
const fail = (ok, msg) => { if (!ok) throw new Error(msg); };
const read = p => fs.readFile(path.join(root,p),'utf8');
const readJson = async p => JSON.parse(await read(p));

async function get(url, json=true) {
  const headers={'User-Agent':'fabushi-tdrp-r9-gate','Accept':'application/vnd.github+json'};
  if (process.env.GITHUB_TOKEN && url.startsWith('https://api.github.com/')) headers.Authorization='Bearer '+process.env.GITHUB_TOKEN;
  const r=await fetch(url,{headers});
  fail(r.ok,`GET ${url} -> ${r.status}`);
  return json ? r.json() : r.text();
}
const ghTree=(repo,sha)=>get(`https://api.github.com/repos/${repo}/git/trees/${sha}?recursive=1`);
function parseGitmodules(raw) {
  const out=new Map();
  for (const block of raw.split(/\n(?=\[submodule )/)) {
    const p=block.match(/(?:^|\n)\s*path\s*=\s*(.+)/);
    const u=block.match(/(?:^|\n)\s*url\s*=\s*(.+)/);
    if (p&&u) out.set(p[1].trim(),u[1].trim());
  }
  return out;
}
function ghRepo(url) {
  const m=url.match(/github\.com[/:]([^/]+)\/([^/]+?)(?:\.git)?$/);
  return m ? `${m[1]}/${m[2].replace(/\.git$/,'')}` : null;
}
async function gitlabTree(project, sha) {
  let page=1, all=[];
  while (true) {
    const u=`https://gitlab.com/api/v4/projects/${encodeURIComponent(project)}/repository/tree?ref=${sha}&recursive=true&per_page=100&page=${page}`;
    const a=await get(u);
    all.push(...a);
    if (a.length<100) break;
    page++;
    fail(page<30,'GitLab pagination runaway');
  }
  return all;
}
async function gitlabRaw(project, sha, file) {
  const u=`https://gitlab.com/api/v4/projects/${encodeURIComponent(project)}/repository/files/${encodeURIComponent(file)}/raw?ref=${sha}`;
  return get(u,false);
}

const lock=await readJson('projects/telegram-desktop-rust/upstream.lock.json');
const schema=await readJson('projects/telegram-desktop-rust/contracts/parity-ledger.schema.json');
const spec=await read('docs/specs/telegram-desktop-rust-equivalence-migration.md');
const rtm=await read('projects/fabushi-communication-platform/quality/requirements-traceability-matrix.md');

fail(lock.project_id==='TDRP-001' && lock.spec_revision===9,'lock is not TDRP Revision 9');
fail(schema.properties?.spec_revision?.const===9,'ledger schema is not Revision 9');
for (const field of ['source_symbols','responsibility_id','existing_owner','fabushi_target_symbols','production_entrypoints','composition_root','search_scope','design_system_version','requirement_ids','oracle_ids','invariant_ids','test_execution_evidence']) {
  fail(JSON.stringify(schema).includes('"'+field+'"'),'schema missing '+field);
}
for (const gate of ['G-INVENTORY','G-FILE','G-TRACEABILITY','G-COMPOSITION','G-SEARCH','G-DESIGN-SYSTEM','G-EVIDENCE']) fail(spec.includes(gate),'spec missing '+gate);
for (const key of ['oracle_ids','invariant_ids','evidence_ids','verdict']) fail(rtm.includes(key),'RTM missing '+key);

const u=lock.upstream;
const commit=await get(`https://api.github.com/repos/${u.repository}/git/commits/${u.commit}`);
fail(commit.sha===u.commit,'upstream commit mismatch');
fail(commit.tree?.sha===u.tree,'upstream tree mismatch');
const branch=await get(`https://api.github.com/repos/${u.repository}/branches/dev`);
fail(branch.commit?.sha===u.commit,`upstream dev advanced to ${branch.commit?.sha}; rebaseline required`);

const rootTree=await ghTree(u.repository,u.commit);
fail(rootTree.truncated===false,'root recursive tree truncated');
const tree=rootTree.tree||[];
const blobs=tree.filter(e=>e.type==='blob');
const links=tree.filter(e=>e.mode==='160000');
fail(tree.length===lock.observed_root_inventory.entry_count,'root entry count drift');
fail(blobs.length===lock.observed_root_inventory.blob_count,'root blob count drift');
fail(links.length===lock.observed_root_inventory.direct_gitlink_count,'direct gitlink count drift');
fail(blobs.filter(e=>e.path.startsWith('Telegram/SourceFiles/')).length===lock.observed_root_inventory.sourcefiles_blob_count,'SourceFiles count drift');
fail(blobs.filter(e=>e.path.startsWith('Telegram/Resources/')).length===lock.observed_root_inventory.resource_blob_count,'Resources count drift');

const gmRaw=await get(`https://raw.githubusercontent.com/${u.repository}/${u.commit}/.gitmodules`,false);
const gm=parseGitmodules(gmRaw);
const pins=new Map(lock.direct_gitlinks.map(x=>[x.path,x]));
fail(pins.size===links.length,'lock/direct gitlink cardinality mismatch');
const nested=[];
for (const g of links) {
  const p=pins.get(g.path);
  fail(p && p.commit===g.sha,'gitlink pin mismatch '+g.path);
  const repo=ghRepo(gm.get(g.path)||'');
  fail(repo===p.repository,'gitlink repository mismatch '+g.path);
  const td=await ghTree(repo,g.sha);
  fail(td.truncated===false,'truncated direct gitlink '+g.path);
  for (const e of td.tree||[]) if (e.mode==='160000') nested.push({parent:`${repo}@${g.sha}`,path:e.path,commit:e.sha});
}
for (const k of lock.known_nested_gitlinks.filter(x=>!x.repository.startsWith('gitlab.com/'))) {
  fail(nested.some(n=>n.parent===k.parent&&n.path===k.path&&n.commit===k.commit),'missing known nested gitlink '+k.parent+':'+k.path);
}
const unexpected=nested.filter(n=>!lock.known_nested_gitlinks.some(k=>k.parent===n.parent&&k.path===n.path&&k.commit===n.commit));
fail(unexpected.length===0,'unexpected nested gitlinks '+JSON.stringify(unexpected));

const cppgir=lock.known_nested_gitlinks.find(x=>x.repository==='gitlab.com/mnauw/cppgir');
fail(cppgir,'cppgir nested pin missing');
const cppTree=await gitlabTree('mnauw/cppgir',cppgir.commit);
const cppNested=cppTree.filter(e=>e.mode==='160000'||e.type==='commit');
fail(cppNested.length===1,'cppgir nested gitlink count changed');
const cppModules=parseGitmodules(await gitlabRaw('mnauw/cppgir',cppgir.commit,'.gitmodules'));
const childPath=cppNested[0].path;
const childRepo=ghRepo(cppModules.get(childPath)||'');
fail(childRepo,'cppgir nested gitlink is not a resolvable GitHub repository');
const childTree=await ghTree(childRepo,cppNested[0].id);
fail(childTree.truncated===false,'cppgir child recursive tree truncated');
fail(!(childTree.tree||[]).some(e=>e.mode==='160000'),'cppgir child gained a further nested gitlink');

const attrs=await get(`https://raw.githubusercontent.com/${u.repository}/${u.commit}/.gitattributes`,false);
fail(!/filter=lfs|diff=lfs|merge=lfs/.test(attrs),'Git LFS attributes detected; explicit LFS closure required');
for (const p of lock.build_time_pin_changes) {
  const raw=await get(`https://raw.githubusercontent.com/${u.repository}/${u.commit}/${p.path}`,false);
  fail(raw.includes(p.commit),'build-time pin missing '+p.path+':'+p.commit);
}

await fs.mkdir(path.join(root,'artifacts/tdrp-authority'),{recursive:true});
await fs.writeFile(path.join(root,'artifacts/tdrp-authority/upstream-root-inventory.json'),JSON.stringify({
  project_id:'TDRP-001',spec_revision:9,repository:u.repository,commit:u.commit,tree:u.tree,
  entries:tree.filter(e=>e.type!=='tree').map(e=>({path:e.path,mode:e.mode,type:e.type,object:e.sha,size:e.size??null}))
},null,2)+'\n');
const report={
  project_id:'TDRP-001',spec_revision:9,target_commit:process.env.GITHUB_SHA||null,
  upstream_commit:u.commit,upstream_tree:u.tree,root_tree_truncated:false,
  root_entries:tree.length,root_blobs:blobs.length,direct_gitlinks:links.length,
  github_nested_gitlinks:nested,gitlab_cppgir_entries:cppTree.length,cppgir_child:{path:childPath,repository:childRepo,commit:cppNested[0].id},
  build_time_pin_changes:lock.build_time_pin_changes,coverage:lock.coverage,
  baseline_accepted:u.accepted,source_closure_ready:lock.acceptance.baseline_ready
};
await fs.writeFile(path.join(root,'artifacts/tdrp-authority/authority-report.json'),JSON.stringify(report,null,2)+'\n');

if (requireAccepted) {
  fail(u.accepted===true,'accepted mode requires upstream.accepted=true');
  fail(lock.acceptance.accepted===true,'accepted mode requires acceptance.accepted=true');
  fail(lock.acceptance.baseline_ready===true,'accepted mode requires baseline_ready=true');
  fail(lock.coverage.unknown===0,'G-INVENTORY unknown must be 0');
  fail(lock.coverage.unread===0,'G-INVENTORY unread must be 0');
  fail(lock.coverage.omitted===0,'G-INVENTORY omitted must be 0');
}
console.log(JSON.stringify(report,null,2));
