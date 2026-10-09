#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const OLD='863cf10d9f34fb0b1b35b35da1bda75acfc58d2e';
const OLD_TREE='5030985204963cbbd362ced7412d231b04ebd0cc';
const NEW='cf478d37c8f57df831cdedb5621e1cf2ef069a0f';
const NEW_TREE='fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad';
const STYLE='Telegram/SourceFiles/info/channel_statistics/earn/channel_earn.style';
const STYLE_OLD='393f0661beb32b1bd909c3dffa40b80f1b25b8a9';
const STYLE_NEW='dc6ae7766af8d145ed9f3125762b2bec7cafb415';
const LIB_OLD='24310a3196c6632f58101a3ec1c553c4565bc9be';
const LIB_NEW='91ff4463899f23f4227bc4cbbdbe7696d479b9bd';
const DESKTOP='3bc92400826cc4ca7ac665b467708e22261edc61';
const FABUSHI='c4fd9ca7aa1346fe8e4a07e2461a70f4b9f16b52';
const PREV_HEAD='b4d9f7f8cf580eefd553c5ce105f1d3e682de87f';
const START=5774, END=5793;
const assert=(x,m)=>{if(!x)throw new Error(m)};
const read=p=>fs.readFileSync(p,'utf8');
const readj=p=>JSON.parse(read(p));
const writej=(p,v)=>fs.writeFileSync(p,JSON.stringify(v,null,2)+'\n');
const headers={'Accept':'application/vnd.github+json','User-Agent':'fabushi-tdrp-rebaseline'};
if(process.env.GITHUB_TOKEN)headers.Authorization='Bearer '+process.env.GITHUB_TOKEN;
async function get(u){const r=await fetch(u,{headers});if(!r.ok)throw new Error('GET '+u+' '+r.status+' '+(await r.text()).slice(0,300));return r.json()}
const api=p=>get('https://api.github.com'+p);
const ref=async(r,b)=>(await api('/repos/'+r+'/git/ref/heads/'+encodeURIComponent(b))).object.sha;
const commit=(r,s)=>api('/repos/'+r+'/git/commits/'+s);
const tree=(r,s)=>api('/repos/'+r+'/git/trees/'+s+'?recursive=1');
const compare=(r,a,b)=>api('/repos/'+r+'/compare/'+a+'...'+b);

assert(await ref('bhrumom/fabushi-desktop','main')===DESKTOP,'desktop main drift');
assert(await ref('bhrumom/fabushi','main')===FABUSHI,'fabushi main drift');
assert(await ref('telegramdesktop/tdesktop','dev')===NEW,'tdesktop dev drift');

const nc=await commit('telegramdesktop/tdesktop',NEW);
assert(nc.tree.sha===NEW_TREE,'new root tree drift');
assert(nc.parents.length===1,'new authority parent shape drift');
const delta=await compare('telegramdesktop/tdesktop',OLD,NEW);
assert(delta.ahead_by===7&&delta.behind_by===0,'unexpected upstream compare');
assert(delta.files.length===22,'unexpected changed path count');
assert(delta.files.some(x=>x.filename===STYLE&&x.status==='modified'),'style delta missing');
assert(delta.files.some(x=>x.filename==='Telegram/lib_ui'&&x.status==='modified'),'lib_ui delta missing');

const nt=await tree('telegramdesktop/tdesktop',NEW_TREE);
const ot=await tree('telegramdesktop/tdesktop',OLD_TREE);
assert(nt.truncated===false&&ot.truncated===false,'root tree truncated');
const nr=nt.tree.filter(x=>x.type!=='tree');
const or=ot.tree.filter(x=>x.type!=='tree');
assert(nr.length===6653&&or.length===6651,'root non-directory count drift');
assert(nt.tree.length===6942,'root entry count drift');
assert(nt.tree.filter(x=>x.type==='blob').length===6617,'root blob count drift');
assert(nt.tree.filter(x=>x.type==='commit').length===36,'root gitlink count drift');
for(let i=0;i<END;i++)assert(nr[i].path===or[i].path,'deterministic path order changed inside accepted/read candidate prefix at '+(i+1));
const orderOf=p=>nr.findIndex(x=>x.path===p)+1;
const changedPrefixPaths=delta.files.map(x=>x.filename).filter(p=>{const o=orderOf(p);return o>0&&o<=END;}).sort();
const expectedChangedPrefix=['.agents/shared/test-loop.md','.agents/skills/process-inbox/scripts/workspace.py','.agents/skills/process-inbox/scripts/workspace_test.py','Telegram/CMakeLists.txt',STYLE,'Telegram/SourceFiles/test/README.md','Telegram/SourceFiles/test/test_animation_clock.cpp','Telegram/SourceFiles/test/test_animation_clock.h'].sort();
assert(JSON.stringify(changedPrefixPaths)===JSON.stringify(expectedChangedPrefix),'changed accepted-prefix path set drift '+JSON.stringify(changedPrefixPaths));
assert(orderOf(STYLE)===4599&&nr[4598].sha===STYLE_NEW,'order 4599 diff drift');
assert(orderOf('Telegram/lib_ui')===6589&&nr[6588].sha===LIB_NEW,'lib_ui order/pin drift');
const diffs=delta.files.map(f=>({path:f.filename,status:f.status,order:orderOf(f.filename),sha:(nr.find(x=>x.path===f.filename)||{}).sha}));

const lc=await commit('desktop-app/lib_ui',LIB_NEW);
const lt=await tree('desktop-app/lib_ui',lc.tree.sha);
const ld=await compare('desktop-app/lib_ui',LIB_OLD,LIB_NEW);
assert(ld.ahead_by===1&&ld.behind_by===0,'lib_ui ancestry drift');
assert(lt.tree.filter(x=>x.type!=='tree').length===432,'lib_ui non-directory count drift');
assert(lt.tree.filter(x=>x.type==='commit').length===0,'lib_ui nested gitlink drift');
const lf=ld.files.map(x=>x.filename).sort();
assert(JSON.stringify(lf)===JSON.stringify(['ui/widgets/fields/input_field.cpp','ui/widgets/fields/masked_input_field.cpp','ui/widgets/widgets.style']),'lib_ui changed path set drift');

for(let o=START;o<=END;o++)assert(nr[o-1].path===or[o-1].path&&nr[o-1].sha===or[o-1].sha,'pre-read identity/order changed at '+o);

const lockp='projects/telegram-desktop-rust/upstream.lock.json';
const idxp='projects/telegram-desktop-rust/inventory/index.json';
const dispidxp='projects/telegram-desktop-rust/inventory/source-dispositions.json';
const ledgerp='projects/telegram-desktop-rust/parity-ledger.json';
const acqp='projects/telegram-desktop-rust/inventory/build-time-acquisitions.json';
const lock=readj(lockp), idx=readj(idxp), di=readj(dispidxp), ledger=readj(ledgerp), acq=readj(acqp);
assert(lock.upstream.commit===OLD&&lock.upstream.tree===OLD_TREE,'lock predecessor mismatch');
assert(di.upstream.commit===OLD&&di.upstream.tree===OLD_TREE,'disposition predecessor mismatch');

const shardDir='projects/telegram-desktop-rust/inventory/source-dispositions';
const shardNames=fs.readdirSync(shardDir).filter(x=>/^\d+-\d+\.json$/.test(x)).sort((a,b)=>+a.split('-')[0]-+b.split('-')[0]);
let rows=[],styleRow=null;
for(const n of shardNames){
  const p=path.join(shardDir,n), s=readj(p);
  assert(s.upstream.commit===OLD&&s.upstream.tree===OLD_TREE,'shard authority drift '+n);
  for(const r of s.rows){
    if(r.recursive_order===4599)styleRow=r;
    rows.push(r);
  }
}
assert(styleRow&&styleRow.source_path===STYLE&&styleRow.source_blob_sha===STYLE_OLD,'style disposition row drift');
const unknownClosedBefore=rows.filter(r=>r.unknown_closed===true).length;
const omittedBefore=rows.filter(r=>r.omitted===true).length;
let prefixClosed=0;for(const r of rows){if(r.recursive_order===prefixClosed+1&&r.unknown_closed===true)prefixClosed++;else break}
const devCandidates=rows.filter(r=>/development-only.*non-applicable|non-applicable.*development-only/i.test([r.disposition,r.fabushi_target,r.ui_disposition,r.notes].filter(Boolean).join(' ')));
const byDisposition=Object.entries(rows.reduce((a,r)=>(a[r.disposition]=(a[r.disposition]||0)+1,a),{})).sort((a,b)=>a[0].localeCompare(b[0]));
console.log('RECOMPUTE prefixClosed='+prefixClosed+' unknownClosed='+unknownClosedBefore+' omitted='+omittedBefore+' developmentCandidate='+devCandidates.length);
console.log('DEVELOPMENT_DISPOSITIONS '+JSON.stringify(byDisposition.filter(([k])=>/development-only|non-applicable/i.test(k))));
assert(devCandidates.length===66,'development_only_non_applicable recompute drift '+devCandidates.length);
assert(unknownClosedBefore===279,'unknown_closed recompute drift');
assert(omittedBefore===0,'omitted recompute drift');
assert(prefixClosed===123,'deterministic_prefix_closed recompute drift');

const defs=[
['test_corner_patch','Rounded-corner patch visual oracle with exact containment/chrome exclusion and DPR device-pixel fail-closed measurement.','Canonical visual regression/capture harness owner'],
['test_custom_emoji','Debug/test-agent-only deterministic custom-emoji fixture for registered ids; never a production fake emoji fallback.','Canonical protected test-fixture + emoji rendering harness owner'],
['test_gated_stage','Explicit N/A stage gate that skips run/until/then in the same turn and never silently passes product failure.','Canonical professional test scenario/stage owner'],
['test_gram_reconcile','Protected wallet fixture reconciliation using local reveal/restore, public-key+address eligibility, no server export/reset/mint, and no recovery words in logs.','Canonical protected wallet test-fixture owner'],
['test_history_fixtures','Bounded service-message history fixtures with conservative rejection of network/index/read-state mutation and deterministic teardown.','Canonical messaging history test-fixture owner'],
['test_hover','Synthetic Click/Drag must emit Leave to prevent hover latch; same-frame DPR/theme-aware evidence and teardown.','Canonical desktop input + visual test harness owner'],
['test_ink','Visual ink measurement keeps refusal distinct from true zero and never relaxes thresholds to manufacture green.','Canonical visual measurement/evidence harness owner'],
['test_lang_pack','Full language-pack snapshot/override restore, Lang::Updated(), and nested LIFO fixture unwind.','Canonical localization test-fixture owner'],
['test_launch_fuse','Test-agent fuse blocks every OS launch; undeclared launch fails, declared expectation is one-shot, launch still remains blocked.','Canonical external-launch safety test harness owner'],
['test_layer_root','Resolve the real painting root; null/stray/misframed roots fail closed and capture must contain box pixels plus title shell.','Canonical dialog/layer visual-capture harness owner']
];
const defFor=p=>defs.find(d=>p.includes(d[0]));
const newRows=[], attEntries=[];
for(let o=START;o<=END;o++){
 const e=nr[o-1], d=defFor(e.path); assert(e.type==='blob'&&d,'unexpected source at '+o+' '+e.path);
 const sym=path.basename(e.path).replace(/\.[^.]+$/,'');
 newRows.push({
  recursive_order:o,source_path:e.path,source_blob_sha:e.sha,read_complete:true,responsibility_decomposition_complete:true,
  responsibility:d[1],upstream_owner:'tdesktop task-test harness / '+d[0],
  state_machine:'test precondition -> bounded deterministic action/fixture/capture -> exact assertion -> explicit teardown/diagnostics',
  side_effects:'TEST/EVIDENCE ONLY. May synthesize bounded input/fixtures/capture inside the harness; must not create a second product state owner or production fallback.',
  failure_cases:'Refusal, stale owner, invalid fixture/input, teardown leak, or unverifiable visual state must fail closed and remain distinct from product success.',
  ui_disposition:'TEST/EVIDENCE ONLY. Validate shipping canonical Fabushi UI/components; introduce no Telegram-named product root, Search, Conversation, Participant or route owner.',
  fabushi_canonical_owner:d[2],fabushi_target:'MAPPED OPEN: bind to the existing canonical professional test/evidence harness and obtain exact-head cross-platform evidence.',
  platform_differences:'Qt/C++ mechanics are source-specific; preserve observable evidence semantics in GitHub Actions without weakening thresholds or product behavior.',
  production_evidence:'Exact live cf478d37 blob identity/order reconciled; source read does not verify shipping product.',
  test_evidence:'Current descendant HEAD must execute the relevant professional harness plus normal Rust desktop runtime and Desktop Chat Parity CI.',
  disposition:'mapped-open-test-harness',unknown_closed:false,omitted:false,consumer_evidence:[],
  reachability_status:'accepted-tree exact blob responsibility proven; professional test-harness closure open',
  license_and_provenance:'tdesktop provenance retained; responsibility is mapped/reimplemented rather than source-copied.',
  notes:'Identity and deterministic order are unchanged from predecessor authority and were revalidated under cf478d37. Reading closes no unknown.',
  consumer_symbol:sym,
  source_binary_evidence:{evidence_status:'live-github-tree-bound-current-head-actions-required',accepted_upstream:NEW,verified_blob_sha:e.sha,byte_size:e.size??null,file_type:'blob',recursive_order:o}
 });
 attEntries.push({order:o,path:e.path,blob:e.sha,consumer_symbol:sym});
}

const changedPrefixSemantic={
  '.agents/shared/test-loop.md':{
    responsibility:'Professional task-test loop authority for background macOS launch, bounded evidence collection, occlusion/exposure handling, crash/death diagnostics and console-input safeguards.',
    owner:'Canonical development-only professional test orchestration owner',
    disposition:'development-only-non-applicable-with-canonical-test-replacement'
  },
  '.agents/skills/process-inbox/scripts/workspace.py':{
    responsibility:'Development-only test workspace runner: background LaunchServices client start, activation suppression/opt-in, path-scoped cleanup, crash/result accounting and telemetry-number formatting.',
    owner:'Canonical development tooling/test-runner owner',
    disposition:'development-only-non-applicable-with-canonical-test-replacement'
  },
  '.agents/skills/process-inbox/scripts/workspace_test.py':{
    responsibility:'Self-tests for the development-only workspace runner background-launch, activation, cleanup, environment refusal and report contracts.',
    owner:'Canonical development tooling self-test owner',
    disposition:'development-only-non-applicable-with-canonical-test-replacement'
  },
  'Telegram/CMakeLists.txt':{
    responsibility:'Build graph reachability for the professional test harness, including the newly added window-exposure implementation/header; production composition remains source-neutral.',
    owner:'Canonical desktop build/test composition owner',
    disposition:'mapped-open-test-build-composition'
  },
  'Telegram/SourceFiles/test/README.md':{
    responsibility:'Task-test harness authority expanded for non-activating background launch, explicit window exposure, activation refusal, crash/death evidence and telemetry-safe helper numbers.',
    owner:'Canonical professional cross-platform test/evidence harness owner',
    disposition:'mapped-open-test-harness'
  },
  'Telegram/SourceFiles/test/test_animation_clock.cpp':{
    responsibility:'Deterministic clocked-frame runner now routes helper numbers and gate/refusal diagnostics through the shared telemetry-safe NumberFormat contract while preserving exact timing/refusal semantics.',
    owner:'Canonical visual regression/timing harness owner',
    disposition:'mapped-open-test-harness'
  },
  'Telegram/SourceFiles/test/test_animation_clock.h':{
    responsibility:'Clocked animation request/frame/evidence interface with shared NumberFormat-aware helper text contracts for telemetry-safe numeric diagnostics.',
    owner:'Canonical visual regression/timing harness owner',
    disposition:'mapped-open-test-harness'
  }
};
for(const n of shardNames){
 const p=path.join(shardDir,n), s=readj(p);s.upstream.commit=NEW;s.upstream.tree=NEW_TREE;
 for(const r of s.rows){
  if(r.source_binary_evidence?.accepted_upstream===OLD)r.source_binary_evidence.accepted_upstream=NEW;
  const sem=changedPrefixSemantic[r.source_path];
  if(sem){
    const e=nr.find(x=>x.path===r.source_path);assert(e,'changed-prefix path disappeared '+r.source_path);
    r.source_blob_sha=e.sha;r.read_complete=true;r.responsibility_decomposition_complete=true;
    r.responsibility=sem.responsibility;r.upstream_owner=sem.owner;r.fabushi_canonical_owner=sem.owner;r.disposition=sem.disposition;
    r.production_evidence='Exact cf478d37 changed blob re-read and rebound; source/tooling read does not itself verify the shipping product.';
    r.test_evidence='Fresh exact-descendant GitHub Actions evidence required; no threshold reduction, fake product fallback, or source-specific product owner is allowed.';
    r.notes='Changed accepted-prefix blob explicitly re-read under cf478d37. Existing applicability/unknown/omitted classification is retained only where the responsibility class remains the same.';
    r.source_binary_evidence={...(r.source_binary_evidence||{}),evidence_status:'live-github-tree-bound-current-head-actions-required',accepted_upstream:NEW,verified_blob_sha:e.sha,byte_size:e.size??null,file_type:'blob',recursive_order:r.recursive_order};
  }
  if(r.recursive_order===4599){
   r.source_blob_sha=STYLE_NEW;r.read_complete=true;r.responsibility_decomposition_complete=true;
   r.responsibility='Channel earnings withdrawal field spacing: replace permanent negative placeholder margins with a floating-state horizontal placeholder shift so the Stars icon remains clear without shifting resting text.';
   r.upstream_owner='tdesktop channel earnings style + desktop-app/lib_ui InputField floating-placeholder contract';
   r.state_machine='resting placeholder uses canonical text/icon margins -> floating transition interpolates placeholderShiftLeft 0 to -23px with vertical shift -> RTL mirroring after translation';
   r.side_effects='Visual/layout only; no payment, Stars balance, recipient, withdrawal, or account truth lives in this style.';
   r.failure_cases='icon overlap; shift applied while resting; InputField/MaskedInputField divergence; RTL translation order drift; source-specific field component duplication';
   r.ui_disposition='Reuse canonical Fabushi TextField/InputField with a leading-icon/floating-label variation; no Telegram/channel-specific field root.';
   r.fabushi_canonical_owner='Canonical TextField/InputField design-system owner';
   r.fabushi_target='MAPPED OPEN: prove floating-label/leading-icon clearance, RTL, theme, responsive and accessibility behavior in the existing canonical TextField.';
   r.platform_differences='Telegram/lib_ui performs horizontal interpolation in Qt paintEvent; Fabushi must express the same responsibility in its source-neutral TextField owner.';
   r.production_evidence='Exact cf478d37 style blob plus lib_ui 91ff446 three-blob change read; current-head canonical TextField visual evidence remains required.';
   r.test_evidence='Focused resting/floating leading-icon visual+a11y coverage in LTR/RTL, theme/DPR, reload and canonical component reuse.';
   r.disposition='mapped-open-canonical-textfield-placeholder-icon-clearance';r.unknown_closed=false;r.omitted=false;
   r.consumer_symbol='botEarnInputField / InputField::paintEvent / MaskedInputField::paintEvent';
   r.consumer_evidence=['cf478d37 replaces placeholderMargins(-23px,...) with placeholderShiftLeft:-23px.','lib_ui 91ff446 adds placeholderShiftLeft style property and interpolated horizontal translation in InputField and MaskedInputField.'];
   r.reachability_status='live direct style consumer plus pinned lib_ui implementation proven; canonical Fabushi UI verification remains open';
   r.license_and_provenance='telegramdesktop/tdesktop and desktop-app/lib_ui provenance retained; behavior reimplemented source-neutrally.';
   r.notes='Changed blob was explicitly re-read under cf478d37; unknown remains open until product evidence.';
   r.source_binary_evidence={...(r.source_binary_evidence||{}),evidence_status:'live-github-tree-bound-current-head-actions-required',accepted_upstream:NEW,verified_blob_sha:STYLE_NEW,byte_size:nr[4598].size??null,file_type:'blob',recursive_order:4599};
  }
 }
 writej(p,s);
}
writej(path.join(shardDir,START+'-'+END+'.json'),{format_version:2,project_id:'TDRP-001',spec_revision:9,upstream:{repository:'telegramdesktop/tdesktop',commit:NEW,tree:NEW_TREE},range:{first_order:START,last_order:END,entries:newRows.length},rows:newRows});
di.upstream.commit=NEW;di.upstream.tree=NEW_TREE;di.deterministic_recursive_prefix.last_order=END;di.deterministic_recursive_prefix.entries=END;
di.shards.push({path:'projects/telegram-desktop-rust/inventory/source-dispositions/'+START+'-'+END+'.json',first_order:START,last_order:END,entries:newRows.length});
writej(dispidxp,di);

const attDir='projects/telegram-desktop-rust/inventory/source-attestation-manifests';
for(const n of fs.readdirSync(attDir).filter(x=>/^\d+-\d+\.json$/.test(x))){
 const p=path.join(attDir,n),m=readj(p);assert(m.accepted_upstream===OLD&&m.accepted_tree===OLD_TREE,'attestation authority drift '+n);
 m.accepted_upstream=NEW;m.accepted_tree=NEW_TREE;
 for(const e of m.entries||[]){
   if(changedPrefixPaths.includes(e.path)){
     const liveEntry=nr.find(x=>x.path===e.path);assert(liveEntry,'attestation changed path missing '+e.path);e.blob=liveEntry.sha;
     if(e.path===STYLE)e.consumer_symbol='botEarnInputField / InputField::paintEvent / MaskedInputField::paintEvent';
   }
  }
 writej(p,m);
}
writej(path.join(attDir,START+'-'+END+'.json'),{accepted_upstream:NEW,accepted_tree:NEW_TREE,deterministic_order_basis:'non-directory/blob rank in accepted recursive tree',start:START,end:END,entries:attEntries});

rows=[...rows,...newRows];
const unknownClosed=rows.filter(r=>r.unknown_closed===true).length;
const omitted=rows.filter(r=>r.omitted===true).length;
let deterministicPrefixClosed=0;for(const r of rows){if(r.recursive_order===deterministicPrefixClosed+1&&r.unknown_closed===true)deterministicPrefixClosed++;else break}
assert(unknownClosed===279&&omitted===0&&deterministicPrefixClosed===123,'post-update closure recompute drift');

const uiPin=lock.direct_gitlinks.find(x=>x.path==='Telegram/lib_ui');assert(uiPin.commit===LIB_OLD,'lib_ui lock pin drift');uiPin.commit=LIB_NEW;
const uiCount=lock.observed_recursive_inventory.direct_component_counts.find(x=>x.mount==='Telegram/lib_ui');assert(uiCount.commit===LIB_OLD&&uiCount.entries===432,'lib_ui component census drift');uiCount.commit=LIB_NEW;
const total=nr.length+lock.observed_recursive_inventory.direct_component_counts.reduce((s,x)=>s+x.entries,0)+lock.observed_recursive_inventory.github_nested_component_counts.reduce((s,x)=>s+x.entries,0)+lock.observed_recursive_inventory.gitlab_cppgir.non_directory_entries+lock.observed_recursive_inventory.cppgir_child.entries;
assert(total===16125,'recursive total recompute drift '+total);
const unread=total-END,unknown=total-unknownClosed;assert(unread===10332&&unknown===15846,'coverage recompute drift');

lock.upstream.commit=NEW;lock.upstream.tree=NEW_TREE;lock.upstream.committed_at='2026-10-09T16:50:46Z';
lock.previous_historical_baseline={commit:OLD,tree:OLD_TREE,evidence_status:'historical-only',reason:'Live dev advanced seven commits from 863cf10d through cf478d37 to cf478d37; predecessor exact-head evidence is historical for the new authority.'};
lock.observed_root_inventory={tree_truncated:false,entry_count:nt.tree.length,blob_count:nt.tree.filter(x=>x.type==='blob').length,direct_gitlink_count:nt.tree.filter(x=>x.type==='commit').length,sourcefiles_blob_count:nt.tree.filter(x=>x.type==='blob'&&x.path.startsWith('Telegram/SourceFiles/')).length,resource_blob_count:nt.tree.filter(x=>x.type==='blob'&&x.path.startsWith('Telegram/Resources/')).length,root_non_directory_count:nr.length};
lock.delta_from_previous_baseline={ahead_by:7,changed_paths:22,added_paths:2,modified_paths:20};
lock.coverage={...lock.coverage,root_source_entries_total:nr.length,recursive_source_entries_total:total,unknown_minimum:unknown,unread_minimum:unread,omitted_known:omitted,note:'Accepted cf478d37 tree 9a2b87e2; read-through 5,793; unknown=15,846, unread=10,332, omitted=0.'};
lock.acceptance.note='Live upstream cf478d37; exact blob read-through 5793/16123; unknown/unread closure remains open; fresh descendant exact-head production/release evidence required.';
if(lock.observed_recursive_inventory){lock.observed_recursive_inventory.current_upstream_commit=NEW;lock.observed_recursive_inventory.current_census_status='live-cf478d37-rebaseline-pending-current-head-actions';lock.observed_recursive_inventory.evidence_status='historical-only-superseded-by-upstream-cf478d37';lock.observed_recursive_inventory.superseded_by_upstream_commit=NEW}
if(lock.baseline_identity_evidence){lock.baseline_identity_evidence.status='historical-only';lock.baseline_identity_evidence.superseded_by_upstream_commit=NEW}
if(lock.recursive_inventory_evidence){lock.recursive_inventory_evidence.status='historical-only';lock.recursive_inventory_evidence.superseded_by_upstream_commit=NEW}
lock.source_disposition_evidence={path:dispidxp,upstream_commit:NEW,deterministic_prefix_closed:deterministicPrefixClosed,development_only_non_applicable:devCandidates.length,unknown_closed:unknownClosed,omitted,read_through:END};
lock.historical_exact_head_evidence={status:'historical-only',predecessor_upstream_commit:OLD,predecessor_target_head:PREV_HEAD,runs:[{id:37958691718,workflow:'Rust desktop runtime',conclusion:'success'},{id:37958689647,workflow:'Desktop Chat Parity CI',conclusion:'success'}],artifacts:[{id:11630066443,digest:'sha256:d4eb35fe0536f9545d0a80b1f79f9821fd530404d062caf5c8fd693f076b66b5'},{id:11631221092,digest:'sha256:d0f12d1dd0c81f4226f5a38d1e9dd147f431c71b3dc96b334b054b92c2800f43'},{id:11630211787,digest:'sha256:2de02ef0217a770851b883ee7c6798fc597200075c7f1bf13918b4d789a51b88'}],note:'These prove 863cf10d+b4d9f7f8 only and are not current cf478d37 evidence.'};
writej(lockp,lock);

idx.upstream.commit=NEW;idx.upstream.tree=NEW_TREE;
idx.inventory={...idx.inventory,root_non_directory_entries:nr.length,recursive_non_directory_entries:total,unknown_minimum:unknown,unread_minimum:unread,omitted_known:omitted,note:'Live cf478d37 recursive census 16,125; accepted-tree blob order exact through 5,793; unknown 15,846, unread 10,332, omitted 0.'};
idx.rebaseline={from_commit:OLD,to_commit:NEW,ahead_by:1,changed_paths:2,changed_authority_paths:[STYLE,'Telegram/lib_ui'],current_head_source_authority_evidence:'Fresh exact-head Source authority evidence required after cf478d37; predecessor runs/artifacts are historical only.',current_source_read_through:END,first_unread_order:5794,first_unread_path:nr[5793].path};
idx.source_dispositions={...lock.source_disposition_evidence};
idx.note='Live authority exact through deterministic blob order 5,793; unread_minimum is 10,332; unknown remains 15,846; omitted remains 0.';
if(idx.latest_recursive_evidence){idx.latest_recursive_evidence.evidence_status='historical-only-superseded-by-upstream-cf478d37';idx.latest_recursive_evidence.note='Predecessor recursive evidence does not prove cf478d37; fresh same-head Source authority evidence required.'}
writej(idxp,idx);

ledger.upstream_commit=NEW;ledger.coverage.source_entries_total=total;ledger.coverage.unknown=unknown;ledger.coverage.unread=unread;ledger.coverage.omitted=omitted;
for(const r of ledger.rows)if(r.source_repository==='telegramdesktop/tdesktop'&&r.source_commit===OLD){const e=nr.find(x=>x.path===r.source_path);assert(e&&e.sha===r.source_blob_sha,'ledger source changed/disappeared '+r.responsibility_id+' '+r.source_path);r.source_commit=NEW}
ledger.source_dispositions={...lock.source_disposition_evidence};ledger.historical_exact_head_evidence=lock.historical_exact_head_evidence;writej(ledgerp,ledger);

function rebindAuthority(v){if(Array.isArray(v)){v.forEach(rebindAuthority);return}if(!v||typeof v!=='object')return;for(const [k,x] of Object.entries(v)){if(['upstream_commit','accepted_upstream','accepted_upstream_commit'].includes(k)&&x===OLD)v[k]=NEW;else rebindAuthority(x)}}
assert(acq.upstream_commit===OLD,'build-time acquisition predecessor drift');rebindAuthority(acq);assert(acq.upstream_commit===NEW,'build-time acquisition rebind failed');writej(acqp,acq);

const live='Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@'+NEW+' (root tree '+NEW_TREE+'). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,793; first unread=5,794 '+nr[5793].path+'@'+nr[5793].sha+'; unread=10,332; unknown=15,846; omitted=0. Changed accepted-prefix orders 11/30/31/92/4599/5759/5764/5765 were explicitly re-read; lib_ui is now order 6,589; runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 are historical-only for '+OLD+' + '+PREV_HEAD+'.';
const docs=['docs/specs/telegram-desktop-rust-equivalence-migration.md','projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md','projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md','projects/fabushi-communication-platform/management/tasks/P0-product-domain-and-telegram-absorption.md','projects/telegram-desktop-rust/management/tasks/P0-source-closure-and-ledger.md','projects/fabushi-communication-platform/quality/requirements-traceability-matrix.md','projects/telegram-desktop-rust/STATUS.md'];
for(const p of docs){let s=read(p);s=live+'\n\n'+s.replace(/^.*863cf10d9f34fb0b1b35b35da1bda75acfc58d2e.*5030985204963cbbd362ced7412d231b04ebd0cc.*$/gm,m=>/current live|当前|discovery HEAD|live authority/i.test(m)?'Historical predecessor authority: '+m:m);s+='\n\n### cf478d37 live-authority rebaseline\n\n'+live+'\n\nDirect delta: channel_earn.style replaces static negative placeholder margins with floating placeholderShiftLeft; lib_ui 91ff446 implements horizontal interpolation in InputField and MaskedInputField. Canonical TextField remains the sole product owner. Orders 5,774-5,793 were identity/order reconciled; reading alone closes no unknown.\n';fs.writeFileSync(p,s)}

fs.writeFileSync('projects/telegram-desktop-rust/dossiers/live-cf478d37-placeholder-libui-rebaseline.md','# TDRP Revision 9 live cf478d37 rebaseline\n\n'+live+'\n\n## Direct delta\n\n- channel_earn.style: botEarnInputField replaces permanent placeholderMargins(-23px,...) with placeholderShiftLeft:-23px.\n- Telegram/lib_ui '+LIB_OLD+' -> '+LIB_NEW+': exactly input_field.cpp, masked_input_field.cpp, widgets.style changed. Both field painters interpolate horizontal floating-placeholder movement; RTL mirroring remains after translation. New lib_ui has 432 non-directory blobs and zero nested gitlinks.\n\n## Deterministic reconciliation\n\nRoot paths retain identical order. Accepted-prefix changed blobs are re-read at orders 11/30/31/92/4599/5759/5764/5765. Two new test_window_exposure blobs appear at 5,847-5,848, after the read-through candidate. Orders 5,774-5,793 remain identity/order-stable and are formally recorded. Telegram/lib_ui moves to root order 6,589 because of the two new files. Full recursive denominator re-evaluates to 16,125; first unread is 5,794 '+nr[5793].path+'@'+nr[5793].sha+'.\n');
fs.writeFileSync('projects/telegram-desktop-rust/dossiers/test-professional-harness-5774-5793-complete-read.md','# Professional test harness 5774-5793 complete read\n\nAuthority '+NEW+' / '+NEW_TREE+'. All twenty blobs preserve predecessor identity/order.\n\n'+newRows.map(r=>'- '+r.recursive_order+' '+r.source_path+'@'+r.source_blob_sha+' — '+r.responsibility+' Canonical owner: '+r.fabushi_canonical_owner+'. Status mapped-open; unknown not closed.').join('\n')+'\n\nThese are test/evidence responsibilities, not production Telegram UI or fake production fallbacks.\n');

console.log(JSON.stringify({upstream:NEW,tree:NEW_TREE,root_non_directory:nr.length,recursive_non_directory:total,read_through:END,first_unread:{order:5794,path:nr[5793].path,blob:nr[5793].sha},unread,unknown,omitted,deterministic_prefix_closed:deterministicPrefixClosed,development_only_non_applicable:devCandidates.length,unknown_closed:unknownClosed,identity_diffs:diffs,lib_ui:{commit:LIB_NEW,non_directory:432,changed_blobs:lf}},null,2));
