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
const ledger=await readJson('projects/telegram-desktop-rust/parity-ledger.json');
const acquisitionInventory=await readJson('projects/telegram-desktop-rust/inventory/build-time-acquisitions.json');

fail(lock.project_id==='TDRP-001' && lock.spec_revision===9,'lock is not TDRP Revision 9');
fail(schema.properties?.spec_revision?.const===9,'ledger schema is not Revision 9');
fail(ledger.project_id==='TDRP-001' && ledger.spec_revision===9 && ledger.format_version===3,'ledger instance is not Revision 9');
fail(ledger.upstream_commit===lock.upstream.commit,'ledger upstream commit does not match accepted baseline identity');
fail(ledger.coverage?.source_entries_total===lock.coverage.recursive_source_entries_total,'ledger recursive source count does not match lock');
fail(ledger.coverage?.unknown>=lock.coverage.unknown_minimum,'ledger unknown count understates lock minimum');
fail(ledger.coverage?.unread>=lock.coverage.unread_minimum,'ledger unread count understates lock minimum');
fail(ledger.coverage?.omitted===lock.coverage.omitted_known,'ledger omitted count disagrees with lock');
fail(Array.isArray(ledger.rows),'ledger rows must be an array');
fail(acquisitionInventory.project_id==='TDRP-001' && acquisitionInventory.spec_revision===9,'build-time acquisition inventory is not Revision 9');
fail(acquisitionInventory.upstream_commit===lock.upstream.commit,'build-time acquisition inventory upstream commit drift');
fail(lock.build_time_acquisition_inventory?.path==='projects/telegram-desktop-rust/inventory/build-time-acquisitions.json','lock missing build-time acquisition inventory authority');
fail(acquisitionInventory.discovery?.candidate_lines_total===lock.build_time_acquisition_inventory.candidate_lines_total,'build-time candidate count drift');
fail(acquisitionInventory.discovery?.immutable_commit_pin_occurrences_classified===lock.build_time_acquisition_inventory.immutable_commit_pin_occurrences_classified,'classified immutable acquisition count drift');
fail(acquisitionInventory.discovery?.candidate_lines_pending_classification===lock.build_time_acquisition_inventory.candidate_lines_pending_classification,'pending acquisition count drift');
fail(Number.isInteger(acquisitionInventory.discovery.candidate_lines_classified),'build-time acquisition classified count missing');
fail(acquisitionInventory.discovery.candidate_lines_total===acquisitionInventory.discovery.candidate_lines_classified+acquisitionInventory.discovery.candidate_lines_pending_classification,'build-time acquisition candidate accounting is not closed arithmetically');
fail(acquisitionInventory.discovery.candidate_lines_classified===lock.build_time_acquisition_inventory.candidate_lines_classified,'classified acquisition count drift');
const dispositionGroups=acquisitionInventory.candidate_disposition_groups || [];
const dispositionKeys=new Set();
let dispositionedCandidateLines=0;
for (const group of dispositionGroups) {
  fail(typeof group.disposition==='string' && group.disposition.length>0,'candidate disposition group missing disposition');
  fail(typeof group.source_path==='string' && group.source_path.length>0,'candidate disposition group missing source path');
  fail(Array.isArray(group.source_lines) && group.source_lines.length>0,'candidate disposition group missing source lines');
  for (const line of group.source_lines) {
    fail(Number.isInteger(line) && line>0,'candidate disposition line must be a positive integer');
    const key=group.source_path+':'+line;
    fail(!dispositionKeys.has(key),'candidate disposition duplicate: '+key);
    dispositionKeys.add(key);
    dispositionedCandidateLines += 1;
  }
}
fail(dispositionedCandidateLines+acquisitionInventory.discovery.immutable_commit_pin_occurrences_classified===acquisitionInventory.discovery.candidate_lines_classified,'explicit candidate dispositions do not match classified count');
const immutableKeys=new Set((acquisitionInventory.immutable_commit_pin_occurrences || []).map(item=>item.source_path+':'+item.source_line));
for (const key of dispositionKeys) fail(!immutableKeys.has(key),'candidate disposition overlaps immutable occurrence: '+key);
fail(lock.build_time_acquisition_inventory.closure_status==='open','build-time acquisition closure must remain open while candidate lines are pending');
for (const acquisition of acquisitionInventory.immutable_commit_pin_occurrences || []) {
  fail(/^[0-9a-f]{40}$/.test(acquisition.commit),'immutable acquisition lacks full commit pin: '+acquisition.id);
  const raw=await get(`https://raw.githubusercontent.com/${lock.upstream.repository}/${lock.upstream.commit}/${acquisition.source_path}`,false);
  fail(raw.includes(acquisition.repository),'acquisition repository missing at accepted upstream: '+acquisition.id);
  fail(raw.includes(acquisition.commit),'acquisition commit missing at accepted upstream: '+acquisition.id);
}
const recursiveChildren=acquisitionInventory.recursive_child_inputs || [];
fail(recursiveChildren.length===lock.build_time_acquisition_inventory.recursive_child_input_occurrences_recorded,'recursive child occurrence count drift');
const externalScans=acquisitionInventory.external_nested_scans || [];
fail(externalScans.length===lock.build_time_acquisition_inventory.recursive_child_unique_external_nested_scans_complete,'external nested scan complete-count drift');
fail(lock.build_time_acquisition_inventory.recursive_child_unique_external_nested_scans_pending===0,'external nested scans remain pending');
const externalScanIds=new Set();
for (const scan of externalScans) {
  fail(typeof scan.id==='string' && scan.id.length>0,'external nested scan missing id');
  fail(!externalScanIds.has(scan.id),'duplicate external nested scan id: '+scan.id);
  externalScanIds.add(scan.id);
  fail(/^[0-9a-f]{40}$/.test(scan.commit),'external nested scan commit is not immutable: '+scan.id);
  fail(scan.status==='complete-live-scan-dispositioned','external nested scan is not closed: '+scan.id);
  const dispositionTotal=Object.values(scan.disposition_counts||{}).reduce((sum,value)=>sum+value,0);
  fail(dispositionTotal===scan.acquisition_candidates,'external nested scan dispositions do not cover every candidate: '+scan.id);
  fail(scan.gitlinks===0,'external nested scan discovered unexpanded gitlinks: '+scan.id);
  fail(scan.lfs_attribute_detected===false && scan.lfs_pointers===0,'external nested scan requires LFS expansion: '+scan.id);
}
const recursiveChildIds=new Set();
for (const child of recursiveChildren) {
  fail(typeof child.id==='string' && child.id.length>0,'recursive child missing id');
  fail(!recursiveChildIds.has(child.id),'duplicate recursive child id: '+child.id);
  recursiveChildIds.add(child.id);
  fail(child.nested_scan_status!=='pending-external-scan','recursive child external scan is still pending: '+child.id);
  if (child.external_scan_id) fail(externalScanIds.has(child.external_scan_id),'recursive child references unknown external scan: '+child.id);
  fail(/^[0-9a-f]{40}$/.test(child.parent_commit),'recursive child parent commit is not immutable: '+child.id);
  fail(/^[0-9a-f]{40}$/.test(child.child_commit),'recursive child commit is not immutable: '+child.id);
  const tree=await ghTree(child.parent_repository,child.parent_commit);
  const entry=(tree.tree||[]).find(item=>item.path===child.child_path);
  fail(entry && entry.mode==='160000','recursive child path is not a gitlink at parent: '+child.id);
  fail(entry.sha===child.child_commit,'recursive child gitlink SHA drift: '+child.id);
}
const recursiveReachability=acquisitionInventory.recursive_build_reachability;
fail(recursiveReachability?.status==='partial-audited-disposition-open','recursive build reachability status must remain explicitly partial/open');
fail(recursiveReachability.accepted_upstream_commit===lock.upstream.commit,'recursive build reachability upstream commit drift');
const qtReachability=recursiveReachability.qt_superproject;
fail(qtReachability?.repository==='https://github.com/qt/qt5','Qt reachability authority missing');
const qtEvidenceByPath=new Map((qtReachability.accepted_upstream_evidence||[]).map(item=>[item.path,item]));
const expectedQtEvidence=[
  ['Telegram/build/prepare/prepare.py','9482a53e60386743ae797d75cecc36767cd63646'],
  ['Telegram/build/docker/centos_env/Dockerfile','5516c448c628d5928d690687ed948d67b5cdaac3'],
  ['snap/snapcraft.yaml','5cff7cf59aedc430b0a9a2d6d66ab6fe3c29bd9f']
];
for (const [sourcePath,blob] of expectedQtEvidence) {
  const evidence=qtEvidenceByPath.get(sourcePath);
  fail(evidence?.blob===blob,'Qt reachability evidence blob drift: '+sourcePath);
  const tree=await ghTree(lock.upstream.repository,lock.upstream.commit);
  const sourceEntry=(tree.tree||[]).find(item=>item.path===sourcePath);
  fail(sourceEntry?.sha===blob,'Qt reachability source blob does not match accepted upstream: '+sourcePath);
}
const prepareQt=await get(`https://raw.githubusercontent.com/${lock.upstream.repository}/${lock.upstream.commit}/Telegram/build/prepare/prepare.py`,false);
const dockerQt=await get(`https://raw.githubusercontent.com/${lock.upstream.repository}/${lock.upstream.commit}/Telegram/build/docker/centos_env/Dockerfile`,false);
const snapQt=await get(`https://raw.githubusercontent.com/${lock.upstream.repository}/${lock.upstream.commit}/snap/snapcraft.yaml`,false);
for (const exact of [
  'git submodule update --init --recursive --progress qtbase qtimageformats qtsvg',
  'git submodule update --init --recursive --progress qtbase qtimageformats qtshadertools qtsvg'
]) fail(prepareQt.includes(exact),'accepted prepare.py Qt submodule selection drift: '+exact);
fail(dockerQt.includes('qtbase qtdeclarative qtwayland qtimageformats qtsvg qtshadertools'),'accepted Docker Qt submodule selection drift');
for (const module of ['qtbase','qtdeclarative','qtimageformats','qtshadertools','qtsvg','qtwayland']) {
  fail(snapQt.includes('      - '+module),'accepted Snap Qt submodule selection missing '+module);
}
fail(!prepareQt.includes('qttools'),'prepare.py unexpectedly initializes qttools');
fail(!dockerQt.includes('qttools'),'Docker Qt build unexpectedly initializes qttools');
fail(!snapQt.includes('      - qttools'),'Snap Qt build unexpectedly initializes qttools');
const nonReachable=qtReachability.explicitly_non_reachable_dispositions||[];
const byRepo=new Map(nonReachable.map(item=>[item.repository,item]));
fail(byRepo.get('https://github.com/qt/qttools')?.disposition==='not-reachable-from-accepted-tdesktop-qt-build','qttools reachability disposition missing');
fail(byRepo.get('https://code.qt.io/playground/qlitehtml')?.disposition==='not-reachable-via-non-reachable-parent','qlitehtml reachability disposition missing');
fail(byRepo.get('https://code.qt.io/qt/qttools-litehtml')?.commit==='6ca1ab0419e770e6d35a1ef690238773a1dafcee','qttools-litehtml immutable child disposition missing');
for (const commit of byRepo.get('https://github.com/qt/qttools')?.commits||[]) {
  fail(/^[0-9a-f]{40}$/.test(commit),'qttools non-reachable commit is not immutable');
}
fail(qtReachability.root_candidate_policy_status==='complete-for-current-qt-authorities','Qt root candidate policy is not fail-closed complete');
const qtRootPolicy=qtReachability.root_candidate_disposition_policy||[];
fail(qtRootPolicy.some(rule=>rule.path_exact==='cmake/QtIRGitHelpers.cmake'&&rule.disposition==='qt-init-repository-helper-not-invoked'),'QtIRGitHelpers acquisition disposition missing');

const tgOwtReachability=recursiveReachability.tg_owt;
fail(tgOwtReachability?.repository==='https://github.com/desktop-app/tg_owt','tg_owt reachability authority missing');
fail(tgOwtReachability.commit==='e2d0e88d1bde6cc600da5dc92581dc97e4c1e685','tg_owt reachability commit drift');
fail(tgOwtReachability.root_candidate_policy_status==='complete-for-current-tg-owt-authority','tg_owt root candidate policy is not fail-closed complete');
const tgOwtPolicy=tgOwtReachability.root_candidate_disposition_policy||[];
for (const [kind,value,disposition] of [
  ['path_prefix','src/','compiled-source-or-comment-not-build-acquisition'],
  ['path_exact','.gitmodules','gitlink-metadata-recursively-accounted'],
  ['path_exact','CMakeLists.txt','cmake-project-metadata-not-acquisition']
]) {
  fail(tgOwtPolicy.some(rule=>rule[kind]===value&&rule.disposition===disposition),'tg_owt acquisition disposition missing: '+value);
}
const prepareTgOwtStage=prepareQt.match(/stage\('tg_owt',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerTgOwtStage=dockerQt.match(/git init tg_owt[\s\S]*?rm -rf tg_owt/)?.[0]||'';
const snapTgOwtStage=snapQt.match(/\n  webrtc:\n[\s\S]*?\n  tlottie:/)?.[0]||snapQt.match(/\n  webrtc:\n[\s\S]*$/)?.[0]||'';
fail(prepareTgOwtStage.includes('git checkout e2d0e88d1bde6cc600da5dc92581dc97e4c1e685'),'accepted prepare.py tg_owt pin drift');
fail(prepareTgOwtStage.includes('git submodule update --init --recursive'),'accepted prepare.py tg_owt recursive submodule build drift');
fail(dockerTgOwtStage.includes('git fetch --depth=1 origin e2d0e88d1bde6cc600da5dc92581dc97e4c1e685'),'accepted Docker tg_owt pin drift');
fail(dockerTgOwtStage.includes('git submodule update --init --recursive --depth=1'),'accepted Docker tg_owt recursive submodule build drift');
fail(snapTgOwtStage.includes('source-commit: e2d0e88d1bde6cc600da5dc92581dc97e4c1e685'),'accepted Snap tg_owt pin drift');

const opensslReachability=recursiveReachability.openssl;
fail(opensslReachability?.repository==='https://github.com/openssl/openssl','OpenSSL reachability authority missing');
fail(opensslReachability.commit==='a7e992847de83aa36be0c399c89db3fb827b0be2','OpenSSL reachability commit drift');
fail(opensslReachability.disposition==='not-reachable-from-accepted-tdesktop-openssl-build','OpenSSL child disposition missing');
const prepareOpenSslStage=prepareQt.match(/stage\('openssl3',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerOpenSslStage=dockerQt.match(/FROM builder AS openssl[\s\S]*?\nFROM builder AS xkbcommon/)?.[0]||'';
const snapOpenSslStage=snapQt.match(/\n  openssl:\n[\s\S]*?\n  nv-codec-headers:/)?.[0]||'';
for (const [name,stage] of [['prepare.py',prepareOpenSslStage],['Dockerfile',dockerOpenSslStage],['snapcraft.yaml',snapOpenSslStage]]) {
  fail(stage.length>0,'unable to isolate accepted OpenSSL build stage in '+name);
  fail(!/git\s+submodule|source-submodules\s*:/.test(stage),'accepted OpenSSL build unexpectedly fetches submodules in '+name);
}
const opensslTree=await ghTree('openssl/openssl',opensslReachability.commit);
const expectedOpenSslChildren=opensslReachability.direct_gitlinks_not_fetched||[];
fail(expectedOpenSslChildren.length===10,'OpenSSL unfetched direct-gitlink accounting drift');
for (const item of expectedOpenSslChildren) {
  fail(Array.isArray(item)&&item.length===3,'OpenSSL unfetched gitlink record malformed');
  const [childPath,childRepository,childCommit]=item;
  fail(/^[0-9a-f]{40}$/.test(childCommit),'OpenSSL child commit is not immutable: '+childPath);
  const entry=(opensslTree.tree||[]).find(candidate=>candidate.path===childPath);
  fail(entry?.mode==='160000','OpenSSL recorded child is not a gitlink: '+childPath);
  fail(entry.sha===childCommit,'OpenSSL recorded child commit drift: '+childPath);
  fail(typeof childRepository==='string'&&childRepository.length>0,'OpenSSL child repository missing: '+childPath);
}

const rowRequired=schema.$defs?.row?.required||[];
const responsibilityIds=new Set();
const targetSymbolOwners=new Map();
for (const row of ledger.rows) {
  for (const key of rowRequired) fail(Object.prototype.hasOwnProperty.call(row,key),'ledger row '+(row.responsibility_id||'<unknown>')+' missing required field '+key);
  fail(row.source_commit===lock.upstream.commit,'row '+row.responsibility_id+' source commit differs from accepted upstream');
  fail(!responsibilityIds.has(row.responsibility_id),'duplicate responsibility id '+row.responsibility_id);
  responsibilityIds.add(row.responsibility_id);
  const sourceObject=(await ghTree(lock.upstream.repository,lock.upstream.commit)).tree.find(e=>e.path===row.source_path);
  fail(sourceObject,'row source path absent from accepted upstream tree: '+row.source_path);
  fail(sourceObject.sha===row.source_blob_sha,'row source blob mismatch: '+row.source_path);
  fail(await fs.stat(path.join(root,row.source_analysis_path)).then(()=>true).catch(()=>false),'row dossier missing: '+row.source_analysis_path);
  fail(row.existing_owner || row.novel_capability===true,'existing-owner-first unresolved for '+row.responsibility_id);
  if (row.novel_capability!==true) {
    fail(row.new_owner_proposal==null && row.new_owner_adr==null,'non-novel row must not introduce a new owner: '+row.responsibility_id);
  }
  const parallelRootText=[row.existing_owner,row.composition_root,row.search_provider,...(row.fabushi_target_paths||[])].filter(Boolean).join(' ');
  fail(!/Telegram(Core|Runtime|Provider|Messaging|Search|Contacts|Media|Workspace|ConversationRoot|MessageStore|DraftStore|ReactionStore)/i.test(parallelRootText),'source-named parallel owner/root detected in '+row.responsibility_id);
  fail(Array.isArray(row.fabushi_target_paths) && row.fabushi_target_paths.length>0,'target path missing for '+row.responsibility_id);
  fail(Array.isArray(row.fabushi_target_symbols) && row.fabushi_target_symbols.length>0,'target symbol missing for '+row.responsibility_id);
  const targetTexts=[];
  for (const targetPath of row.fabushi_target_paths) {
    fail(await fs.stat(path.join(root,targetPath)).then(()=>true).catch(()=>false),'target path missing: '+targetPath);
    targetTexts.push(await read(targetPath));
  }
  const combinedTarget=targetTexts.join('\n');
  for (const symbol of row.fabushi_target_symbols) {
    fail(combinedTarget.includes(symbol),'target symbol '+symbol+' not found for '+row.responsibility_id);
    const key=row.fabushi_target_paths.join(',')+'#'+symbol;
    const prior=targetSymbolOwners.get(key);
    if (prior && prior!==row.responsibility_id) fail(false,'target symbol has multiple responsibility owners without explicit split: '+key);
    targetSymbolOwners.set(key,row.responsibility_id);
  }
  const traceText=rtm+'\n'+await read(row.source_analysis_path);
  for (const id of [...row.requirement_ids,...row.oracle_ids,...row.invariant_ids]) fail(traceText.includes(id),'traceability id '+id+' missing from RTM/dossier for '+row.responsibility_id);
  if (['implemented','verified'].includes(row.implementation_status)) {
    fail(row.production_entrypoints.length>0,'implemented row lacks shipping entrypoint: '+row.responsibility_id);
    fail(row.production_evidence.length>0,'implemented row lacks production evidence: '+row.responsibility_id);
    fail(row.test_evidence.length>0,'implemented row lacks test evidence: '+row.responsibility_id);
    fail(row.existing_owner!=null,'implemented row lacks existing/new canonical owner: '+row.responsibility_id);
  }
  if (/search/i.test(row.responsibility_id+' '+row.capability_ids.join(' '))) {
    fail(/Search/i.test(row.search_scope) && row.search_provider.length>0,'Search row lacks canonical Search scope/provider: '+row.responsibility_id);
    fail(/ProductShell/i.test(row.composition_root),'Search row must compose under ProductShell: '+row.responsibility_id);
  }
}
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
const recursiveInventory=[
  ...tree.filter(e=>e.type!=='tree').map(e=>({
    repository:u.repository,commit:u.commit,scope:'root',path:e.path,mode:e.mode,type:e.type,object:e.sha,size:e.size??null
  }))
];
const directComponentCounts=[];
for (const g of links) {
  const p=pins.get(g.path);
  fail(p && p.commit===g.sha,'gitlink pin mismatch '+g.path);
  const repo=ghRepo(gm.get(g.path)||'');
  fail(repo===p.repository,'gitlink repository mismatch '+g.path);
  const td=await ghTree(repo,g.sha);
  fail(td.truncated===false,'truncated direct gitlink '+g.path);
  const componentEntries=(td.tree||[]).filter(e=>e.type!=='tree');
  directComponentCounts.push({mount:g.path,repository:repo,commit:g.sha,entries:componentEntries.length});
  recursiveInventory.push(...componentEntries.map(e=>({
    repository:repo,commit:g.sha,scope:'direct-gitlink',mount:g.path,path:e.path,mode:e.mode,type:e.type,object:e.sha,size:e.size??null
  })));
  for (const e of td.tree||[]) if (e.mode==='160000') nested.push({parent:`${repo}@${g.sha}`,path:e.path,commit:e.sha});
}
for (const k of lock.known_nested_gitlinks.filter(x=>!x.repository.startsWith('gitlab.com/'))) {
  fail(nested.some(n=>n.parent===k.parent&&n.path===k.path&&n.commit===k.commit),'missing known nested gitlink '+k.parent+':'+k.path);
}
const unexpected=nested.filter(n=>!lock.known_nested_gitlinks.some(k=>k.parent===n.parent&&k.path===n.path&&k.commit===n.commit));
fail(unexpected.length===0,'unexpected nested gitlinks '+JSON.stringify(unexpected));

const githubNestedCounts=[];
for (const k of lock.known_nested_gitlinks.filter(x=>!x.repository.startsWith('gitlab.com/'))) {
  const td=await ghTree(k.repository,k.commit);
  fail(td.truncated===false,'truncated nested gitlink '+k.repository+'@'+k.commit);
  const entries=(td.tree||[]).filter(e=>e.type!=='tree');
  githubNestedCounts.push({parent:k.parent,path:k.path,repository:k.repository,commit:k.commit,entries:entries.length});
  recursiveInventory.push(...entries.map(e=>({
    repository:k.repository,commit:k.commit,scope:'nested-gitlink',parent:k.parent,mount:k.path,path:e.path,mode:e.mode,type:e.type,object:e.sha,size:e.size??null
  })));
  fail(!(td.tree||[]).some(e=>e.mode==='160000'),'known GitHub nested gitlink gained an untracked deeper gitlink '+k.repository+'@'+k.commit);
}

const cppgir=lock.known_nested_gitlinks.find(x=>x.repository==='gitlab.com/mnauw/cppgir');
fail(cppgir,'cppgir nested pin missing');
const cppTree=await gitlabTree('mnauw/cppgir',cppgir.commit);
const cppNested=cppTree.filter(e=>e.mode==='160000'||e.type==='commit');
fail(cppNested.length===1,'cppgir nested gitlink count changed');
const cppModules=parseGitmodules(await gitlabRaw('mnauw/cppgir',cppgir.commit,'.gitmodules'));
const childPath=cppNested[0].path;
const childRepo=ghRepo(cppModules.get(childPath)||'');
fail(childRepo,'cppgir nested gitlink is not a resolvable GitHub repository');
recursiveInventory.push(...cppTree.filter(e=>e.type!=='tree').map(e=>({
  repository:'gitlab.com/mnauw/cppgir',commit:cppgir.commit,scope:'nested-gitlink',parent:cppgir.parent,mount:cppgir.path,
  path:e.path,mode:e.mode??null,type:e.type,object:e.id,size:null
})));
const childTree=await ghTree(childRepo,cppNested[0].id);
fail(childTree.truncated===false,'cppgir child recursive tree truncated');
fail(!(childTree.tree||[]).some(e=>e.mode==='160000'),'cppgir child gained a further nested gitlink');
const childEntries=(childTree.tree||[]).filter(e=>e.type!=='tree');
recursiveInventory.push(...childEntries.map(e=>({
  repository:childRepo,commit:cppNested[0].id,scope:'nested-gitlink-child',parent:'gitlab.com/mnauw/cppgir@'+cppgir.commit,mount:childPath,
  path:e.path,mode:e.mode,type:e.type,object:e.sha,size:e.size??null
})));

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
await fs.writeFile(path.join(root,'artifacts/tdrp-authority/upstream-recursive-inventory.json'),JSON.stringify({
  project_id:'TDRP-001',spec_revision:9,root:{repository:u.repository,commit:u.commit,tree:u.tree},
  counts:{entries:recursiveInventory.length,direct_components:directComponentCounts,github_nested_components:githubNestedCounts,
    gitlab_cppgir_entries:cppTree.filter(e=>e.type!=='tree').length,cppgir_child_entries:childEntries.length},
  entries:recursiveInventory
},null,2)+'\n');
const report={
  project_id:'TDRP-001',spec_revision:9,target_commit:process.env.GITHUB_SHA||null,
  upstream_commit:u.commit,upstream_tree:u.tree,root_tree_truncated:false,
  root_entries:tree.length,root_blobs:blobs.length,direct_gitlinks:links.length,recursive_non_directory_entries:recursiveInventory.length,
  direct_component_counts:directComponentCounts,github_nested_gitlinks:nested,github_nested_component_counts:githubNestedCounts,
  gitlab_cppgir_entries:cppTree.length,cppgir_child:{path:childPath,repository:childRepo,commit:cppNested[0].id,entries:childEntries.length},
  build_time_pin_changes:lock.build_time_pin_changes,build_time_acquisition_inventory:acquisitionInventory.discovery,coverage:lock.coverage,ledger_coverage:ledger.coverage,
  baseline_accepted:u.accepted,source_closure_ready:lock.acceptance.baseline_ready
};
await fs.writeFile(path.join(root,'artifacts/tdrp-authority/authority-report.json'),JSON.stringify(report,null,2)+'\n');

if (requireAccepted) {
  fail(u.accepted===true,'accepted mode requires upstream.accepted=true');
  fail(lock.acceptance.accepted===true,'accepted mode requires acceptance.accepted=true');
  fail(lock.acceptance.baseline_ready===true,'accepted mode requires baseline_ready=true');
  fail(ledger.target_commit===(process.env.GITHUB_SHA||ledger.target_commit),'G-EVIDENCE ledger target_commit must equal exact GitHub Actions HEAD');
  fail(ledger.coverage.unknown===0,'G-INVENTORY unknown must be 0');
  fail(ledger.coverage.unread===0,'G-INVENTORY unread must be 0');
  fail(ledger.coverage.omitted===0,'G-INVENTORY omitted must be 0');
  fail(ledger.coverage.unmapped_responsibilities===0,'G-TRACEABILITY unmapped responsibilities must be 0');
  fail(ledger.rows.length>0,'G-FILE/G-TRACEABILITY ledger cannot be empty at closure');
}
console.log(JSON.stringify(report,null,2));
