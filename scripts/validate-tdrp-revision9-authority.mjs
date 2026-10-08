#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';

const root = process.cwd();
const requireAccepted = process.argv.includes('--require-accepted');
const fail = (ok, msg) => { if (!ok) throw new Error(msg); };
const read = p => fs.readFile(path.join(root,p),'utf8');
const readJson = async p => JSON.parse(await read(p));

const getCache = new Map();
const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));

async function getUncached(url, json=true) {
  const headers={'User-Agent':'fabushi-tdrp-r9-gate','Accept':'application/vnd.github+json'};
  if (process.env.GITHUB_TOKEN && url.startsWith('https://api.github.com/')) headers.Authorization='Bearer '+process.env.GITHUB_TOKEN;
  for (let attempt=0; attempt<4; attempt++) {
    const r=await fetch(url,{headers});
    if (r.ok) return json ? r.json() : r.text();
    const retryable = r.status===403 || r.status===429 || r.status>=500;
    if (!retryable || attempt===3) {
      const detail=(await r.text()).slice(0,500);
      throw new Error(`GET ${url} -> ${r.status}${detail ? `: ${detail}` : ''}`);
    }
    const retryAfter=Number(r.headers.get('retry-after'));
    const delay=Number.isFinite(retryAfter) && retryAfter>0
      ? Math.min(retryAfter*1000, 30000)
      : 1000*(2**attempt);
    await sleep(delay);
  }
  throw new Error(`GET ${url} exhausted retries`);
}
function get(url, json=true) {
  const key=(json ? 'json:' : 'text:')+url;
  if (!getCache.has(key)) getCache.set(key,getUncached(url,json));
  return getCache.get(key);
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
function dockerStage(raw, stageName) {
  const stages=[...raw.matchAll(/^FROM[^\r\n]*$/gim)];
  const wanted=String(stageName).toLowerCase();
  for (let index=0; index<stages.length; index++) {
    const line=stages[index][0];
    const alias=line.match(/[ \t]+AS[ \t]+([^ \t#]+)/i)?.[1]?.toLowerCase();
    if (alias!==wanted) continue;
    const start=stages[index].index;
    const end=index+1<stages.length ? stages[index+1].index : raw.length;
    return raw.slice(start,end);
  }
  return '';
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
const fbcSourceOfTruth=await read('projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md');
const tdrpSourceOfTruth=await read('projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md');
const p0Task=await read('projects/fabushi-communication-platform/management/tasks/P0-product-domain-and-telegram-absorption.md');
const inventoryIndex=await readJson('projects/telegram-desktop-rust/inventory/index.json');
const rtm=await read('projects/fabushi-communication-platform/quality/requirements-traceability-matrix.md');
const ledger=await readJson('projects/telegram-desktop-rust/parity-ledger.json');
const acquisitionInventory=await readJson('projects/telegram-desktop-rust/inventory/build-time-acquisitions.json');

fail(lock.project_id==='TDRP-001' && lock.spec_revision===9,'lock is not TDRP Revision 9');
const authorityCommit=lock.upstream?.commit;
const authorityTree=lock.upstream?.tree;
fail(/^[0-9a-f]{40}$/.test(authorityCommit||''),'lock upstream commit is not exact');
fail(/^[0-9a-f]{40}$/.test(authorityTree||''),'lock upstream tree is not exact');
const authorityDocs=[
  ['FBCP SOURCE_OF_TRUTH',fbcSourceOfTruth],
  ['TDRP SOURCE_OF_TRUTH',tdrpSourceOfTruth],
  ['P0 task',p0Task],
  ['TDRP spec',spec],
];
for (const [label,document] of authorityDocs) {
  fail(document.includes(authorityCommit),label+' commit authority differs from lock');
  fail(document.includes(authorityTree),label+' root-tree authority differs from lock');
  const authorityPair = document
    .split(/\n/)
    .some(line => line.includes(authorityCommit) && line.includes(authorityTree));
  fail(authorityPair,label+' does not bind the live commit and root tree on one authority statement');
}
fail(inventoryIndex.upstream?.commit===authorityCommit,'inventory index upstream commit drift');
fail(inventoryIndex.upstream?.tree===authorityTree,'inventory index upstream tree drift');
fail(inventoryIndex.rebaseline?.to_commit===authorityCommit,'inventory rebaseline target differs from live authority');
fail(!lock.coverage?.note?.includes('live authority is now telegramdesktop/tdesktop@e1ed57a44e7c14e0cbb91bcf0f7ec3e408786a39'),'lock coverage note still names historical e1ed57a as live authority');
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
  ['Telegram/build/prepare/prepare.py','93ff2c8ab4f3275644145012c9231718530c1486'],
  ['Telegram/build/docker/centos_env/Dockerfile','09a05179b48b4f9185697134002bad4e486ca4fd'],
  ['snap/snapcraft.yaml','b363829e5c131fce5e80e0644c0fb6546da8ad6c']
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
fail(tgOwtReachability.commit==='d1cf250ea73de26c4c1f0a3c8173eb2648efbb04','tg_owt reachability commit drift');
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
fail(prepareTgOwtStage.includes('git checkout d1cf250ea73de26c4c1f0a3c8173eb2648efbb04'),'accepted prepare.py tg_owt pin drift');
fail(prepareTgOwtStage.includes('git submodule update --init --recursive'),'accepted prepare.py tg_owt recursive submodule build drift');
fail(dockerTgOwtStage.includes('git fetch --depth=1 origin d1cf250ea73de26c4c1f0a3c8173eb2648efbb04'),'accepted Docker tg_owt pin drift');
fail(dockerTgOwtStage.includes('git submodule update --init --recursive --depth=1'),'accepted Docker tg_owt recursive submodule build drift');
fail(snapTgOwtStage.includes('source-commit: d1cf250ea73de26c4c1f0a3c8173eb2648efbb04'),'accepted Snap tg_owt pin drift');

const libjxlReachability=recursiveReachability.libjxl;
fail(libjxlReachability?.repository==='https://github.com/libjxl/libjxl','libjxl reachability authority missing');
fail(libjxlReachability.commit==='a7a9c787341cf703dede03c2009fa460cae5e5df','libjxl reachability commit drift');
fail(libjxlReachability.root_candidate_policy_status==='complete-for-current-libjxl-authority','libjxl root candidate policy is not fail-closed complete');
const libjxlPolicy=libjxlReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_prefix','.github/'],['path_prefix','doc/'],['path_prefix','tools/'],
  ['path_exact','deps.sh'],['path_exact','.gitmodules'],['path_exact','third_party/CMakeLists.txt']
]) fail(libjxlPolicy.some(rule=>rule[kind]===value),'libjxl acquisition disposition missing: '+value);
const prepareLibjxlStage=prepareQt.match(/stage\('libjxl',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerLibjxlStage=dockerQt.match(/FROM builder AS jxl[\s\S]*?rm -rf libjxl/)?.[0]||'';
fail(prepareLibjxlStage.includes('git clone -b v0.12.0 --recursive --shallow-submodules'),'accepted prepare.py libjxl acquisition drift');
for (const option of ['-DBUILD_TESTING=OFF','-DJPEGXL_ENABLE_DEVTOOLS=OFF','-DJPEGXL_ENABLE_TOOLS=OFF','-DJPEGXL_ENABLE_PLUGINS=OFF']) {
  fail(prepareLibjxlStage.includes(option),'accepted prepare.py libjxl option drift: '+option);
}
fail(dockerLibjxlStage.includes('git clone -b v0.12.0 --depth=1 https://github.com/libjxl/libjxl.git'),'accepted Docker libjxl acquisition drift');
for (const option of ['-DBUILD_TESTING=OFF','-DJPEGXL_ENABLE_DEVTOOLS=OFF','-DJPEGXL_ENABLE_TOOLS=OFF','-DJPEGXL_ENABLE_BENCHMARK=OFF']) {
  fail(dockerLibjxlStage.includes(option),'accepted Docker libjxl option drift: '+option);
}

const libavifReachability=recursiveReachability.libavif;
fail(libavifReachability?.repository==='https://github.com/AOMediaCodec/libavif','libavif reachability authority missing');
fail(libavifReachability.root_candidate_policy_status==='complete-for-current-libavif-authorities','libavif root candidate policy is not fail-closed complete');
const expectedLibavifCommits=new Set(['c5240fc79fe5c2407e10afd35f5505ef6333ea49','1aadfad932c98c069a1204261b1856f81f3bc199']);
for (const authority of libavifReachability.observed_authorities||[]) expectedLibavifCommits.delete(authority.commit);
fail(expectedLibavifCommits.size===0,'libavif observed authority set drift');
const libavifPolicy=libavifReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_prefix','.github/'],['path_prefix','cmake/Modules/Local'],['path_prefix','ext/'],
  ['path_prefix','src/'],['path_prefix','tests/'],['path_exact','README.md']
]) fail(libavifPolicy.some(rule=>rule[kind]===value),'libavif acquisition disposition missing: '+value);
const prepareLibavifStage=prepareQt.match(/stage\('libavif',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerLibavifStage=dockerQt.match(/git clone -b v1\.4\.2 --depth=1 https:\/\/github\.com\/AOMediaCodec\/libavif\.git[\s\S]*?rm -rf libavif/)?.[0]||'';
const snapLibavifStage=snapQt.match(/\n  avif:\n[\s\S]*?\n  ffmpeg:/)?.[0]||'';
for (const [name,stage] of [['prepare.py',prepareLibavifStage],['Dockerfile',dockerLibavifStage],['snapcraft.yaml',snapLibavifStage]]) {
  fail(stage.includes('AVIF_CODEC_DAV1D=SYSTEM'),'accepted libavif build lost DAV1D=SYSTEM in '+name);
  fail(stage.includes('AVIF_LIBYUV=OFF'),'accepted libavif build lost LIBYUV=OFF in '+name);
}
fail(snapLibavifStage.includes('source-tag: v1.3.0'),'accepted Snap libavif ref drift');

const adaReachability=recursiveReachability.ada;
fail(adaReachability?.repository==='https://github.com/ada-url/ada','Ada reachability authority missing');
fail(adaReachability.commit==='6b4612162c34e7ffcee08c03b107b449e07a5683','Ada reachability commit drift');
fail(adaReachability.root_candidate_policy_status==='complete-for-current-ada-authority','Ada root candidate policy is not fail-closed complete');
const adaPolicy=adaReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_prefix','.github/'],['path_prefix','cmake/'],['path_prefix','benchmarks/'],
  ['path_prefix','tools/'],['path_prefix','tests/'],['path_exact','README.md']
]) fail(adaPolicy.some(rule=>rule[kind]===value),'Ada acquisition disposition missing: '+value);
const prepareAdaStage=prepareQt.match(/stage\('ada',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerAdaStage=dockerQt.match(/FROM builder AS ada[\s\S]*?rm -rf ada/)?.[0]||'';
const snapAdaStage=snapQt.match(/\n  ada:\n[\s\S]*?\n  avif:/)?.[0]||'';
for (const [name,stage] of [['prepare.py',prepareAdaStage],['Dockerfile',dockerAdaStage],['snapcraft.yaml',snapAdaStage]]) {
  fail(stage.includes('ADA_TESTING=OFF'),'accepted Ada build lost ADA_TESTING=OFF in '+name);
  fail(stage.includes('ADA_TOOLS=OFF'),'accepted Ada build lost ADA_TOOLS=OFF in '+name);
  fail(stage.includes('ADA_INCLUDE_URL_PATTERN=OFF'),'accepted Ada build lost ADA_INCLUDE_URL_PATTERN=OFF in '+name);
}
fail(snapAdaStage.includes('source-tag: v3.2.9'),'accepted Snap Ada ref drift');

const openalReachability=recursiveReachability.openal_soft;
fail(openalReachability?.root_candidate_policy_status==='complete-for-current-openal-authorities','OpenAL root candidate policy is not fail-closed complete');
const expectedOpenalAuthorities=new Set([
  'https://github.com/kcat/openal-soft@b2c48f7718ef3fcf67921a8b6534c4914e328970',
  'https://github.com/kcat/openal-soft@dc7d7054a5b4f3bec1dc23a42fd616a0847af948',
  'https://github.com/telegramdesktop/openal-soft@291c0fdbbdd767f00c45c7fca562614faea57647',
  'https://github.com/telegramdesktop/openal-soft@c2eab43d72890c58d4b51d5d98eafb9f011e1c89'
]);
for (const authority of openalReachability.observed_authorities||[]) expectedOpenalAuthorities.delete(authority.repository+'@'+authority.commit);
fail(expectedOpenalAuthorities.size===0,'OpenAL observed authority set drift');
const openalPolicy=openalReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_prefix','.github/'],['path_exact','.travis.yml'],['path_prefix','fmt-'],
  ['path_prefix','gsl/'],['path_prefix','tests/'],['path_exact','README.md']
]) fail(openalPolicy.some(rule=>rule[kind]===value),'OpenAL acquisition disposition missing: '+value);
const prepareOpenalStage=prepareQt.match(/stage\('openal-soft',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerOpenalStage=dockerQt.match(/FROM builder AS openal[\s\S]*?rm -rf openal-soft/)?.[0]||'';
const snapOpenalStage=snapQt.match(/\n  openal:\n[\s\S]*?\n  openssl:/)?.[0]||'';
for (const option of ['ALSOFT_EXAMPLES=OFF','ALSOFT_UTILS=OFF']) {
  fail(prepareOpenalStage.includes(option),'accepted prepare.py OpenAL option drift: '+option);
  fail(dockerOpenalStage.includes(option),'accepted Docker OpenAL option drift: '+option);
  fail(snapOpenalStage.includes(option),'accepted Snap OpenAL option drift: '+option);
}
fail(prepareOpenalStage.includes('git checkout 291c0fdbbd'),'accepted Windows OpenAL pin drift');
fail(prepareOpenalStage.includes('git checkout coreaudio_device_uid'),'accepted Mac OpenAL branch evidence drift');
fail(dockerOpenalStage.includes('git clone -b 1.25.2 --depth=1'),'accepted Docker OpenAL ref drift');
fail(snapOpenalStage.includes('source-tag: 1.24.3'),'accepted Snap OpenAL ref drift');

const ffmpegReachability=recursiveReachability.ffmpeg;
fail(ffmpegReachability?.repository==='https://github.com/FFmpeg/FFmpeg','FFmpeg reachability authority missing');
fail(ffmpegReachability.root_candidate_policy_status==='complete-for-current-ffmpeg-authorities','FFmpeg root candidate policy is not fail-closed complete');
const expectedFfmpegCommits=new Set(['1041abdc962f4cc4f394aa8de9dc5236c0c3b9e7','f1e3a2bf7a2f2cde936d1ed97f09a26853d20125']);
for (const authority of ffmpegReachability.observed_authorities||[]) expectedFfmpegCommits.delete(authority.commit);
fail(expectedFfmpegCommits.size===0,'FFmpeg observed authority set drift');
const ffmpegPolicy=ffmpegReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_prefix','.forgejo/'],['path_prefix','doc/'],['path_prefix','tests/'],['path_prefix','tools/'],
  ['path_prefix','libavcodec/'],['path_prefix','libavfilter/'],['path_prefix','libavformat/'],
  ['path_exact','Changelog'],['path_exact','RELEASE_NOTES']
]) fail(ffmpegPolicy.some(rule=>rule[kind]===value),'FFmpeg acquisition disposition missing: '+value);
const prepareFfmpegStage=prepareQt.match(/stage\('ffmpeg',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerFfmpegStage=dockerStage(dockerQt,'ffmpeg');
fail(dockerFfmpegStage.length>0,'unable to isolate accepted FFmpeg build stage in Dockerfile');
const snapFfmpegStage=snapQt.match(/\n  ffmpeg:\n[\s\S]*?\n  [a-z0-9_-]+:/)?.[0]||'';
for (const [name,stage] of [['prepare.py',prepareFfmpegStage],['Dockerfile',dockerFfmpegStage],['snapcraft.yaml',snapFfmpegStage]]) {
  fail(stage.includes('--disable-programs'),'accepted FFmpeg build lost --disable-programs in '+name);
  fail(stage.includes('--disable-doc'),'accepted FFmpeg build lost --disable-doc in '+name);
  fail(stage.includes('--disable-network'),'accepted FFmpeg build lost --disable-network in '+name);
  fail(stage.includes('--disable-everything'),'accepted FFmpeg build lost --disable-everything in '+name);
}
fail(prepareFfmpegStage.includes('git clone -b n8.1.3'),'accepted prepare.py FFmpeg ref drift');
fail(dockerFfmpegStage.includes('git clone -b n8.1.3 --depth=1'),'accepted Docker FFmpeg ref drift');
fail(snapFfmpegStage.includes('source-branch: n6.1.6'),'accepted Snap FFmpeg ref drift');

const libheifReachability=recursiveReachability.libheif;
fail(libheifReachability?.repository==='https://github.com/strukturag/libheif','libheif reachability authority missing');
fail(libheifReachability.commit==='413e2a87e6a70b3eccc3a3adc5801179dd2d9e00','libheif reachability commit drift');
fail(libheifReachability.root_candidate_policy_status==='complete-for-current-libheif-authority','libheif root candidate policy is not fail-closed complete');
const libheifPolicy=libheifReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_prefix','.github/'],['path_prefix','scripts/'],['path_prefix','third-party/'],
  ['path_exact','build-emscripten.sh'],['path_exact','README.md'],['path_exact','SECURITY.md'],
  ['path_prefix','examples/'],['path_prefix','heifio/'],['path_prefix','libheif/']
]) fail(libheifPolicy.some(rule=>rule[kind]===value),'libheif acquisition disposition missing: '+value);
const prepareLibheifStage=prepareQt.match(/stage\('libheif',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerLibheifStage=dockerStage(dockerQt,'heif');
for (const [name,stage] of [['prepare.py',prepareLibheifStage],['Dockerfile',dockerLibheifStage]]) {
  fail(stage.length>0,'unable to isolate accepted libheif build stage in '+name);
  fail(stage.includes('git clone -b v1.23.5'),'accepted libheif ref drift in '+name);
  for (const option of [
    'BUILD_SHARED_LIBS=OFF','BUILD_DOCUMENTATION=OFF','BUILD_TESTING=OFF','ENABLE_PLUGIN_LOADING=OFF',
    'WITH_LIBDE265=OFF','WITH_X265=OFF','WITH_X264=OFF','WITH_OpenH264_DECODER=OFF',
    'WITH_SvtEnc=OFF','WITH_RAV1E=OFF','WITH_FFMPEG_DECODER=ON','WITH_LIBSHARPYUV=OFF','WITH_EXAMPLES=OFF'
  ]) fail(stage.includes(option),'accepted libheif build option drift in '+name+': '+option);
  fail(!stage.includes('third-party/'),'accepted libheif build unexpectedly invokes third-party helper in '+name);
  fail(!stage.includes('scripts/'),'accepted libheif build unexpectedly invokes upstream helper scripts in '+name);
}

const breakpadReachability=recursiveReachability.breakpad;
fail(breakpadReachability?.repository==='https://chromium.googlesource.com/breakpad/breakpad','Breakpad reachability authority missing');
fail(breakpadReachability.root_candidate_policy_status==='complete-for-current-breakpad-authorities','Breakpad root candidate policy is not fail-closed complete');
const expectedBreakpadCommits=new Set(['dfcb7b6799b7c1e2c8d65e857d8afede185471d8','9aebd3d8ef5a246deb2c929b5666aaba160ebce6']);
for (const authority of breakpadReachability.observed_authorities||[]) expectedBreakpadCommits.delete(authority.commit);
fail(expectedBreakpadCommits.size===0,'Breakpad observed authority set drift');
const breakpadPolicy=breakpadReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_prefix','.github/'],['path_exact','DEPS'],['path_prefix','autotools/'],
  ['path_prefix','docs/'],['path_prefix','src/common/'],['path_prefix','src/third_party/curl/'],
  ['path_prefix','src/third_party/libdisasm/swig/']
]) fail(breakpadPolicy.some(rule=>rule[kind]===value),'Breakpad acquisition disposition missing: '+value);
const prepareBreakpadStage=prepareQt.match(/stage\('breakpad',[\s\S]*?\n""" \+ macBreakpadBuild\)/)?.[0]||'';
const prepareStackwalkStage=prepareQt.match(/stage\('stackwalk',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerBreakpadStage=dockerStage(dockerQt,'breakpad');
for (const [name,stage] of [['prepare.py breakpad',prepareBreakpadStage],['prepare.py stackwalk',prepareStackwalkStage]]) {
  fail(stage.length>0,'unable to isolate accepted Breakpad build stage in '+name);
  fail(stage.includes('https://chromium.googlesource.com/breakpad/breakpad'),'accepted Breakpad repository drift in '+name);
  fail(stage.includes('git checkout dfcb7b6799'),'accepted Breakpad prepare pin drift in '+name);
  fail(stage.includes('depends:patches/breakpad.diff')&&stage.includes('git apply ../patches/breakpad.diff'),'accepted Breakpad patch application drift in '+name);
  fail(stage.includes('git clone -b release-1.11.0 https://github.com/google/googletest src/testing'),'accepted Breakpad googletest input drift in '+name);
  fail(!/\bgclient\b|depot_tools/.test(stage),'accepted Breakpad prepare stage unexpectedly invokes upstream dependency bootstrap in '+name);
}
fail(prepareBreakpadStage.includes('git checkout e1e7b0ad8e'),'accepted Breakpad prepare LSS pin drift');
fail(prepareStackwalkStage.includes('git checkout e1e7b0ad8e'),'accepted stackwalk LSS pin drift');
fail(dockerBreakpadStage.length>0,'unable to isolate accepted Breakpad Docker stage');
fail(dockerBreakpadStage.includes('git fetch --depth=1 origin 9aebd3d8ef5a246deb2c929b5666aaba160ebce6'),'accepted Breakpad Docker pin drift');
fail(dockerBreakpadStage.includes('git init src/third_party/lss'),'accepted Breakpad Docker LSS initialization drift');
fail(dockerBreakpadStage.includes('git fetch --depth=1 origin 29164a80da4d41134950d76d55199ea33fbb9613'),'accepted Breakpad Docker LSS pin drift');
fail(dockerBreakpadStage.includes('./configure')&&dockerBreakpadStage.includes('make -j$(nproc)')&&dockerBreakpadStage.includes('make DESTDIR=/usr/src/breakpad-cache install'),'accepted Breakpad Docker build path drift');
fail(!dockerBreakpadStage.includes('src/testing'),'accepted Breakpad Docker stage unexpectedly initializes testing dependency');
fail(!/\bgclient\b|depot_tools/.test(dockerBreakpadStage),'accepted Breakpad Docker stage unexpectedly invokes upstream dependency bootstrap');

const highwayReachability=recursiveReachability.highway;
fail(highwayReachability?.repository==='https://github.com/google/highway','Highway reachability authority missing');
fail(highwayReachability.commit==='2607d3b5b0113992fe84d3848859eae13b3b52c1','Highway reachability commit drift');
fail(highwayReachability.root_candidate_policy_status==='complete-for-current-highway-authority','Highway root candidate policy is not fail-closed complete');
const highwayPolicy=highwayReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_prefix','.github/'],['path_prefix','hwy/contrib/'],['path_prefix','g3doc/'],['path_prefix','docs/'],
  ['path_exact','README.md'],['path_exact','WORKSPACE'],['path_exact','CMakeLists.txt.in'],['path_exact','hwy/abort_test.cc']
]) fail(highwayPolicy.some(rule=>rule[kind]===value),'Highway acquisition disposition missing: '+value);
const dockerHighwayStage=dockerStage(dockerQt,'highway');
fail(dockerHighwayStage.length>0,'unable to isolate accepted Highway Docker stage');
fail(dockerHighwayStage.includes('git clone -b 1.4.0 --depth=1 https://github.com/google/highway.git'),'accepted Highway Docker ref drift');
for (const option of ['BUILD_TESTING=OFF','HWY_ENABLE_CONTRIB=OFF','HWY_ENABLE_EXAMPLES=OFF']) {
  fail(dockerHighwayStage.includes(option),'accepted Highway Docker option drift: '+option);
}

const brotliReachability=recursiveReachability.brotli;
fail(brotliReachability?.repository==='https://github.com/google/brotli','Brotli reachability authority missing');
fail(brotliReachability.commit==='028fb5a23661f123017c060daa546b55cf4bde29','Brotli reachability commit drift');
fail(brotliReachability.root_candidate_policy_status==='complete-for-current-brotli-authority','Brotli root candidate policy is not fail-closed complete');
const brotliPolicy=brotliReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_prefix','.github/'],['path_exact','README.md'],['path_prefix','python/'],['path_prefix','fetch-spec/'],
  ['path_exact','csharp/transpile.sh'],['path_exact','scripts/download_testdata.sh']
]) fail(brotliPolicy.some(rule=>rule[kind]===value),'Brotli acquisition disposition missing: '+value);
const dockerBrotliStage=dockerStage(dockerQt,'brotli');
fail(dockerBrotliStage.length>0,'unable to isolate accepted Brotli Docker stage');
fail(dockerBrotliStage.includes('git clone -b v1.2.0 --depth=1 https://github.com/google/brotli.git'),'accepted Brotli Docker ref drift');
fail(dockerBrotliStage.includes('BUILD_SHARED_LIBS=OFF'),'accepted Brotli Docker shared-library option drift');
fail(dockerBrotliStage.includes('BROTLI_DISABLE_TESTS=ON'),'accepted Brotli Docker test option drift');

const opusReachability=recursiveReachability.opus;
fail(opusReachability?.repository==='https://github.com/xiph/opus','Opus reachability authority missing');
fail(opusReachability.commit==='ddbe48383984d56acd9e1ab6a090c54ca6b735a6','Opus reachability commit drift');
fail(opusReachability.root_candidate_policy_status==='complete-for-current-opus-authority','Opus root candidate policy is not fail-closed complete');
const opusPolicy=opusReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_prefix','.github/'],['path_exact','.gitlab-ci.yml'],['path_exact','README'],['path_exact','README.draft'],
  ['path_exact','dnn/download_model.sh'],['path_prefix','dnn/torch/'],['path_prefix','doc/'],['path_exact','scripts/local_build.py']
]) fail(opusPolicy.some(rule=>rule[kind]===value),'Opus acquisition disposition missing: '+value);
const prepareOpusStage=prepareQt.match(/stage\('opus',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerOpusStage=dockerStage(dockerQt,'opus');
for (const [name,stage] of [['prepare.py',prepareOpusStage],['Dockerfile',dockerOpusStage]]) {
  fail(stage.length>0,'unable to isolate accepted Opus build stage in '+name);
  fail(stage.includes('git clone -b v1.5.2'),'accepted Opus ref drift in '+name);
  fail(!stage.includes('dnn/download_model.sh')&&!stage.includes('dnn/torch/')&&!stage.includes('scripts/local_build.py'),'accepted Opus stage unexpectedly invokes upstream helper in '+name);
}
const opusCmake=await get('https://raw.githubusercontent.com/xiph/opus/'+opusReachability.commit+'/CMakeLists.txt',false);
for (const option of ['OPUS_BUILD_TESTING','OPUS_BUILD_PROGRAMS','OPUS_DRED','OPUS_OSCE']) {
  const pattern=new RegExp('option\\('+option+'[^\\n]*OFF\\)');
  fail(pattern.test(opusCmake),'Opus CMake default no longer keeps '+option+' OFF');
}
fail(!opusCmake.includes('download_model.sh')&&!opusCmake.includes('dnn/torch/'),'Opus production CMake unexpectedly invokes neural acquisition/training helper');

const tde2eReachability=recursiveReachability.tde2e;
fail(tde2eReachability?.repository==='https://github.com/tdlib/td','TDE2E reachability authority missing');
fail(tde2eReachability.commit==='51743dfd01dff6179e2d8f7095729caa4e2222e9','TDE2E reachability commit drift');
fail(tde2eReachability.root_candidate_policy_status==='complete-for-current-tde2e-authority','TDE2E root candidate policy is not fail-closed complete');
const tde2ePolicy=tde2eReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_exact','build.html'],['path_prefix','example/'],['path_prefix','benchmark/'],
  ['path_exact','CMake/GetGitRevisionDescription.cmake'],['path_prefix','tdutils/generate/'],['path_prefix','td/generate/']
]) fail(tde2ePolicy.some(rule=>rule[kind]===value),'TDE2E acquisition disposition missing: '+value);
const prepareTde2eStage=prepareQt.match(/stage\('tde2e',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerTde2eStage=dockerStage(dockerQt,'tde2e');
const snapTde2eStage=snapQt.match(/\n  tde2e:\n[\s\S]*?\n  tlottie:/)?.[0]||'';
for (const [name,stage] of [['prepare.py',prepareTde2eStage],['Dockerfile',dockerTde2eStage],['snapcraft.yaml',snapTde2eStage]]) {
  fail(stage.length>0,'unable to isolate accepted TDE2E stage in '+name);
  fail(stage.includes('TD_E2E_ONLY=ON'),'accepted TDE2E build lost TD_E2E_ONLY=ON in '+name);
}
fail(prepareTde2eStage.includes('git checkout 51743df'),'accepted prepare.py TDE2E pin drift');
fail(dockerTde2eStage.includes('git fetch --depth=1 origin 51743dfd01dff6179e2d8f7095729caa4e2222e9'),'accepted Docker TDE2E pin drift');
fail(snapTde2eStage.includes('source-commit: 51743dfd01dff6179e2d8f7095729caa4e2222e9'),'accepted Snap TDE2E pin drift');

const libsrtpReachability=recursiveReachability.libsrtp;
fail(libsrtpReachability?.repository==='https://github.com/cisco/libsrtp','libsrtp reachability authority missing');
fail(libsrtpReachability.commit==='a566a9cfcd619e8327784aa7cff4a1276dc1e895','libsrtp reachability commit drift');
fail(libsrtpReachability.root_candidate_policy_status==='complete-for-current-libsrtp-authority','libsrtp root candidate policy is not fail-closed complete');
const libsrtpPolicy=libsrtpReachability.root_candidate_disposition_policy||[];
for (const [kind,value] of [
  ['path_prefix','.github/'],['path_exact','.travis.yml'],['path_exact','README.md'],
  ['path_exact','config.guess'],['path_exact','config.sub'],['path_prefix','test/']
]) fail(libsrtpPolicy.some(rule=>rule[kind]===value),'libsrtp acquisition disposition missing: '+value);
const tgOwtTree=await ghTree('desktop-app/tg_owt','d1cf250ea73de26c4c1f0a3c8173eb2648efbb04');
const libsrtpEntry=(tgOwtTree.tree||[]).find(item=>item.path==='src/third_party/libsrtp');
fail(libsrtpEntry?.mode==='160000','accepted tg_owt libsrtp path is no longer a gitlink');
fail(libsrtpEntry.sha==='a566a9cfcd619e8327784aa7cff4a1276dc1e895','accepted tg_owt libsrtp gitlink commit drift');
fail(prepareQt.match(/stage\('tg_owt',[\s\S]*?\n"""\)/)?.[0]?.includes('git submodule update --init --recursive'),'accepted prepare.py tg_owt no longer recursively fetches libsrtp');

const boostRegexReachability=recursiveReachability.boost_regex;
fail(boostRegexReachability?.repository==='https://github.com/boostorg/regex','Boost.Regex reachability authority missing');
fail(boostRegexReachability.commit==='4cbcd3078e6ae10d05124379623a1bf03fcb9350','Boost.Regex reachability commit drift');
fail(boostRegexReachability.root_candidate_policy_status==='complete-for-current-boost-regex-authority','Boost.Regex root candidate policy is not fail-closed complete');
const boostRegexPolicy=boostRegexReachability.root_candidate_disposition_policy||[];
fail(boostRegexPolicy.some(rule=>rule.path_prefix==='.github/'),'Boost.Regex CI disposition missing');
fail(boostRegexPolicy.some(rule=>rule.path_exact==='README.md'),'Boost.Regex README disposition missing');
const prepareRegexStage=prepareQt.match(/stage\('regex',[\s\S]*?\n"""\)/)?.[0]||'';
fail(prepareRegexStage.includes('git clone -b boost-1.83.0 https://github.com/boostorg/regex.git'),'accepted Boost.Regex prepare ref drift');
fail(!prepareRegexStage.includes('boostorg/boost')&&!prepareRegexStage.includes('boostdep'),'accepted Boost.Regex stage unexpectedly invokes upstream CI dependency bootstrap');

const implibReachability=recursiveReachability.implib;
fail(implibReachability?.commit==='ecf7bb51a92a0fb16834c5b698570ab25f9f1d21','Implib authority drift');
fail(implibReachability.root_candidate_policy_status==='complete-for-current-implib-authority','Implib policy not fail-closed complete');
const dockerImplibStage=dockerQt.match(/git init Implib\.so[\s\S]*?implib \/usr\/lib64\/libOpenGL\.so/)?.[0]||'';
fail(dockerImplibStage.includes('git fetch --depth=1 origin ecf7bb51a92a0fb16834c5b698570ab25f9f1d21'),'accepted Implib pin drift');
fail(dockerImplibStage.includes('../implib-gen.py'),'accepted Implib production invocation drift');

const zlibReachability=recursiveReachability.zlib;
fail(zlibReachability?.commit==='e3dc0a85b7032e98380dec011bc8f2c2ee0d8fca','zlib authority drift');
fail(zlibReachability.root_candidate_policy_status==='complete-for-current-zlib-authority','zlib policy not fail-closed complete');
const dockerZlibStage=dockerStage(dockerQt,'zlib');
const prepareZlibStage=prepareQt.match(/stage\('zlib',[\s\S]*?\n"""\)/)?.[0]||'';
fail(dockerZlibStage.includes('git fetch --depth=1 origin e3dc0a85b7032e98380dec011bc8f2c2ee0d8fca'),'accepted Docker zlib pin drift');
fail(prepareZlibStage.includes('git checkout e3dc0a85b7032e98380dec011bc8f2c2ee0d8fca'),'accepted prepare zlib pin drift');
for (const option of ['ZLIB_BUILD_TESTING=OFF','ZLIB_MINIZIP_BUILD_TESTING=OFF']) fail(dockerZlibStage.includes(option),'accepted Docker zlib option drift: '+option);
fail(!dockerZlibStage.includes('contrib/ada')&&!prepareZlibStage.includes('contrib/ada'),'accepted zlib build unexpectedly invokes contrib/ada');

const openh264Reachability=recursiveReachability.openh264;
fail(openh264Reachability?.commit==='652bdb7719f30b52b08e506645a7322ff1b2cc6f','OpenH264 authority drift');
fail(openh264Reachability.root_candidate_policy_status==='complete-for-current-openh264-authority','OpenH264 policy not fail-closed complete');
const prepareOpenh264Stage=prepareQt.match(/stage\('openh264',[\s\S]*?\n"""\)/)?.[0]||'';
const dockerOpenh264Stage=dockerStage(dockerQt,'openh264');
for (const [name,stage] of [['prepare.py',prepareOpenh264Stage],['Dockerfile',dockerOpenh264Stage]]) {
  fail(stage.includes('git clone -b v2.6.0'),'accepted OpenH264 ref drift in '+name);
  fail(/meson (setup|build)/.test(stage),'accepted OpenH264 Meson path drift in '+name);
  fail(!/\bmake\b/.test(stage),'accepted OpenH264 stage unexpectedly invokes Makefile fetch-capable path in '+name);
}

const xkbcommonReachability=recursiveReachability.xkbcommon;
fail(xkbcommonReachability?.commit==='d2a08f761c796733e42fac4099f5c38d443e88e1','xkbcommon authority drift');
fail(xkbcommonReachability.root_candidate_policy_status==='complete-for-current-xkbcommon-authority','xkbcommon policy not fail-closed complete');
const dockerXkbcommonStage=dockerStage(dockerQt,'xkbcommon');
fail(dockerXkbcommonStage.includes('git clone -b xkbcommon-1.6.0 --depth=1'),'accepted xkbcommon ref drift');
for (const option of ['enable-docs=false','enable-wayland=false','enable-xkbregistry=false']) fail(dockerXkbcommonStage.includes(option),'accepted xkbcommon option drift: '+option);
fail(!dockerXkbcommonStage.includes('scripts/makekeys'),'accepted xkbcommon build unexpectedly regenerates key tables');

const opensslReachability=recursiveReachability.openssl;
fail(opensslReachability?.repository==='https://github.com/openssl/openssl','OpenSSL reachability authority missing');
fail(opensslReachability.commit==='45e844fa2a14ec92d146bd8f5778ac130b6625fb','OpenSSL reachability commit drift');
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
fail(expectedOpenSslChildren.length===11,'OpenSSL unfetched direct-gitlink accounting drift');
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
  const targetTexts=new Map();
  for (const targetPath of row.fabushi_target_paths) {
    fail(await fs.stat(path.join(root,targetPath)).then(()=>true).catch(()=>false),'target path missing: '+targetPath);
    targetTexts.set(targetPath,await read(targetPath));
  }
  const bindings=Array.isArray(row.fabushi_target_bindings) ? row.fabushi_target_bindings : null;
  if (bindings) {
    fail(bindings.length>0,'explicit target bindings must not be empty for '+row.responsibility_id);
    const boundPaths=new Set();
    const boundSymbols=new Set();
    for (const binding of bindings) {
      fail(row.fabushi_target_paths.includes(binding.path),'bound target path is not declared by '+row.responsibility_id+': '+binding.path);
      fail(!boundPaths.has(binding.path),'target path is bound more than once inside '+row.responsibility_id+': '+binding.path);
      boundPaths.add(binding.path);
      const targetText=targetTexts.get(binding.path);
      fail(typeof targetText==='string','bound target path missing from loaded targets: '+binding.path);
      fail(typeof binding.responsibility_scope==='string' && binding.responsibility_scope.trim().length>0,'bound target scope missing for '+row.responsibility_id+': '+binding.path);
      fail(Array.isArray(binding.symbols) && binding.symbols.length>0,'bound target symbols missing for '+row.responsibility_id+': '+binding.path);
      for (const symbol of binding.symbols) {
        fail(row.fabushi_target_symbols.includes(symbol),'bound target symbol is not declared by '+row.responsibility_id+': '+symbol);
        fail(!boundSymbols.has(symbol),'target symbol is ambiguously bound to multiple paths inside '+row.responsibility_id+': '+symbol);
        boundSymbols.add(symbol);
        fail(targetText.includes(symbol),'target symbol '+symbol+' not found in owning path '+binding.path+' for '+row.responsibility_id);
        const key=binding.path+'#'+symbol;
        const priors=targetSymbolOwners.get(key)||[];
        for (const prior of priors) {
          if (prior.responsibility_id===row.responsibility_id) continue;
          fail(
            prior.explicit===true
              && typeof prior.scope==='string'
              && prior.scope.length>0
              && prior.scope!==binding.responsibility_scope,
            'target path symbol has multiple responsibility owners without explicit distinct responsibility_scope split: '+key
          );
        }
        priors.push({responsibility_id:row.responsibility_id,scope:binding.responsibility_scope,explicit:true});
        targetSymbolOwners.set(key,priors);
      }
    }
    fail(boundPaths.size===row.fabushi_target_paths.length,'not every target path has an exact binding for '+row.responsibility_id);
    fail(boundSymbols.size===row.fabushi_target_symbols.length,'not every target symbol has an exact owning-path binding for '+row.responsibility_id);
  } else {
    const combinedTarget=[...targetTexts.values()].join('\n');
    for (const symbol of row.fabushi_target_symbols) {
      fail(combinedTarget.includes(symbol),'target symbol '+symbol+' not found for '+row.responsibility_id);
      const key=row.fabushi_target_paths.join(',')+'#'+symbol;
      const priors=targetSymbolOwners.get(key)||[];
      for (const prior of priors) {
        if (prior.responsibility_id!==row.responsibility_id) fail(false,'target symbol has multiple responsibility owners without explicit split: '+key);
      }
      priors.push({responsibility_id:row.responsibility_id,scope:null,explicit:false});
      targetSymbolOwners.set(key,priors);
    }
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
fail(JSON.stringify(schema).includes('"fabushi_target_bindings"'),'schema missing fabushi_target_bindings');
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
const targetCommit=process.env.TDRP_TARGET_SHA||process.env.GITHUB_SHA||null;
const report={
  project_id:'TDRP-001',spec_revision:9,target_commit:targetCommit,
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
  fail(ledger.target_commit===(targetCommit||ledger.target_commit),'G-EVIDENCE ledger target_commit must equal exact tested GitHub Actions HEAD');
  fail(ledger.coverage.unknown===0,'G-INVENTORY unknown must be 0');
  fail(ledger.coverage.unread===0,'G-INVENTORY unread must be 0');
  fail(ledger.coverage.omitted===0,'G-INVENTORY omitted must be 0');
  fail(ledger.coverage.unmapped_responsibilities===0,'G-TRACEABILITY unmapped responsibilities must be 0');
  fail(ledger.rows.length>0,'G-FILE/G-TRACEABILITY ledger cannot be empty at closure');
}
console.log(JSON.stringify(report,null,2));
