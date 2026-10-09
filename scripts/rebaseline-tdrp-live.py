#!/usr/bin/env python3
import json, os, pathlib, subprocess, urllib.request
ROOT=pathlib.Path(__file__).resolve().parents[1]
OLD="cf478d37c8f57df831cdedb5621e1cf2ef069a0f"
OLD_TREE="fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad"
NEW="811b83a1cc5f61bd4238ab3ebfcbee6302078014"
NEW_TREE="4ddb5182fc1ae238e4dc62a396614614b5b35f97"
BRANCH="work/story-slider-view-coverage-20261008"
EXPECTED_CHANGED=["Telegram/SourceFiles/ui/chat/chat_theme_readability.cpp"]

def api(path):
    req=urllib.request.Request("https://api.github.com"+path,headers={"User-Agent":"fabushi-tdrp-rebaseline"})
    with urllib.request.urlopen(req) as r:
        return json.load(r)

def load(rel):
    return json.loads((ROOT/rel).read_text())
def dump(rel,obj):
    p=ROOT/rel; p.parent.mkdir(parents=True,exist_ok=True)
    p.write_text(json.dumps(obj,ensure_ascii=False,indent=2)+"\n")
def replace_exact(obj):
    if isinstance(obj,dict):
        return {k:replace_exact(v) for k,v in obj.items()}
    if isinstance(obj,list):
        return [replace_exact(v) for v in obj]
    if isinstance(obj,str):
        return obj.replace(OLD,NEW).replace(OLD_TREE,NEW_TREE)
    return obj

live=subprocess.check_output(["git","ls-remote","https://github.com/telegramdesktop/tdesktop.git","refs/heads/dev"],text=True).split()[0]
if live!=NEW:
    raise SystemExit(f"live dev moved: expected {NEW}, got {live}")
commit=api(f"/repos/telegramdesktop/tdesktop/git/commits/{NEW}")
if commit["tree"]["sha"]!=NEW_TREE:
    raise SystemExit("unexpected root tree")
cmp=api(f"/repos/telegramdesktop/tdesktop/compare/{OLD}...{NEW}")
changed=[x["filename"] for x in cmp["files"]]
if cmp["ahead_by"]!=1 or changed!=EXPECTED_CHANGED:
    raise SystemExit(f"unexpected upstream delta: ahead={cmp['ahead_by']} files={changed}")
tree=api(f"/repos/telegramdesktop/tdesktop/git/trees/{NEW_TREE}?recursive=1")
if tree.get("truncated"):
    raise SystemExit("root tree truncated")
non=[x for x in tree["tree"] if x["type"]!="tree"]
if len(non)!=6653:
    raise SystemExit(f"root non-directory changed: {len(non)}")
by_path={x["path"]:(i+1,x) for i,x in enumerate(non)}
expected_prefix={
"Telegram/SourceFiles/test/test_log.cpp":(5794,"17c72a8ae1bc672e7c380b855887b859b78fabec"),
"Telegram/SourceFiles/test/test_log.h":(5795,"b792c7df04e5bbabf0cee9fe43a591ffe9c337a1"),
"Telegram/SourceFiles/test/test_log_lines.cpp":(5796,"ab66b2b19ef81ce64bb6b1287de4ccea9237268e"),
"Telegram/SourceFiles/test/test_log_lines.h":(5797,"c20e337c4bb58c27fd8d1e4f3666d10891a602a1"),
"Telegram/SourceFiles/test/test_mapped_target.cpp":(5798,"e346082f2b851fe72eec44c7d1b765e57944dcb7"),
"Telegram/SourceFiles/test/test_mapped_target.h":(5799,"b75145d6a5692651199df507d85c560d0cb7766a"),
"Telegram/SourceFiles/test/test_marking_read.cpp":(5800,"fa683bfd70682191f82e49d7e6901a6a2a3a73b4"),
"Telegram/SourceFiles/test/test_marking_read.h":(5801,"67513a698292f68310c384914b9444ae6e7d43f7"),
"Telegram/SourceFiles/test/test_menu.cpp":(5802,"93cdaf3adf82725adb07e90be83e560d40a7b72d"),
}
for path,(order,sha) in expected_prefix.items():
    got_order,e=by_path[path]
    if got_order!=order or e["sha"]!=sha:
        raise SystemExit(f"prefix identity drift {path}: {got_order} {e['sha']}")
theme_order,theme=by_path[EXPECTED_CHANGED[0]]
if theme_order!=5951 or theme["sha"]!="11c736768ea553afdcebfb88df106a9eb1fde1db":
    raise SystemExit("theme readability identity/order drift")

master=load("projects/telegram-desktop-rust/inventory/source-dispositions.json")
master["upstream"]["commit"]=NEW; master["upstream"]["tree"]=NEW_TREE
master["deterministic_recursive_prefix"]["last_order"]=5801
master["deterministic_recursive_prefix"]["entries"]=5801
shard_rel="projects/telegram-desktop-rust/inventory/source-dispositions/5794-5801.json"
if not any(x["path"]==shard_rel for x in master["shards"]):
    master["shards"].append({"path":shard_rel,"first_order":5794,"last_order":5801,"entries":8})
for ref in master["shards"]:
    p=ROOT/ref["path"]
    if not p.exists():
        if ref["path"]!=shard_rel:
            raise SystemExit(f"missing shard {ref['path']}")
        continue
    j=replace_exact(json.loads(p.read_text()))
    j["upstream"]["commit"]=NEW; j["upstream"]["tree"]=NEW_TREE
    dump(ref["path"],j)

defs=[
(5794,"Telegram/SourceFiles/test/test_log.cpp","17c72a8ae1bc672e7c380b855887b859b78fabec",6217,"test_log","Crash-surviving evidence-log writer: one call stays one physical line, Python splitlines separators are visibly escaped, PASS/FAIL/N/A stay distinct, and payload text cannot forge TEST_COMPLETE."),
(5795,"Telegram/SourceFiles/test/test_log.h","b792c7df04e5bbabf0cee9fe43a591ffe9c337a1",9183,"test_log","Public contract for one-line evidence grammar, verdict semantics, one-way secrecy-safe helper-number formatting, geometry/near formatting, and terminal completion."),
(5796,"Telegram/SourceFiles/test/test_log_lines.cpp","ab66b2b19ef81ce64bb6b1287de4ccea9237268e",8059,"test_log_lines","Independent raw-byte self-test for all Python splitlines separators, CRLF, trailing/separator-only payloads and embedded TEST_COMPLETE forgery attempts."),
(5797,"Telegram/SourceFiles/test/test_log_lines.h","c20e337c4bb58c27fd8d1e4f3666d10891a602a1",5194,"test_log_lines","Public anti-log-injection and anti-fake-completion oracle contract deliberately independent from the writer separator table."),
(5798,"Telegram/SourceFiles/test/test_mapped_target.cpp","e346082f2b851fe72eec44c7d1b765e57944dcb7",16552,"test_mapped_target","Visual-capture self-test proving complete mapped-target readiness across ElasticScroll and ScrollArea; overlap-only, empty, hidden, outside and nonpainting roots fail closed."),
(5799,"Telegram/SourceFiles/test/test_mapped_target.h","b75145d6a5692651199df507d85c560d0cb7766a",2213,"test_mapped_target","Public mapped-target capture contract requiring the whole target inside the relevant viewport and a real painting owner without shrinking to overlap."),
(5800,"Telegram/SourceFiles/test/test_marking_read.cpp","fa683bfd70682191f82e49d7e6901a6a2a3a73b4",14648,"test_marking_read","Professional harness control that temporarily minimizes without hiding the real main window to stop marking messages read, proves reversibility, captures, and restores."),
(5801,"Telegram/SourceFiles/test/test_marking_read.h","67513a698292f68310c384914b9444ae6e7d43f7",3245,"test_marking_read","Public bounded reversible not-marking-read test lever contract, including exact predicate, 3s refusal bound, exposure support and mandatory restoration."),
]
rows=[]
for order,path,sha,size,symbol,responsibility in defs:
    rows.append({
      "recursive_order":order,"source_path":path,"source_blob_sha":sha,"read_complete":True,"responsibility_decomposition_complete":True,
      "responsibility":responsibility,"upstream_owner":f"tdesktop task-test harness / {symbol}",
      "state_machine":"bounded professional-test precondition/control -> exact observation/assertion -> explicit refusal or pass -> teardown/restore",
      "side_effects":"TEST/EVIDENCE ONLY. May write professional evidence, exercise bounded real UI state, or capture the shipping surface inside the harness; must not create a second product state owner or production fallback.",
      "failure_cases":"Refusal, stale owner, injected/malformed evidence, incomplete viewport visibility, failed bounded state transition, teardown leak, or unverifiable state must fail closed and remain distinct from product success.",
      "ui_disposition":"TEST/EVIDENCE ONLY. Validate shipping canonical Fabushi UI/components and evidence contracts; introduce no Telegram-named product root, Search, Conversation, Participant or route owner.",
      "fabushi_canonical_owner":"Canonical source-neutral professional GitHub Actions test/evidence harness",
      "fabushi_target":"MAPPED OPEN: bind to the existing canonical professional test/evidence harness and obtain exact-head cross-platform evidence without weakening gates.",
      "platform_differences":"Qt/C++ mechanics are source-specific; preserve observable evidence/refusal/cleanup semantics in GitHub Actions using canonical Fabushi owners.",
      "production_evidence":"Exact live 811b83a1 blob identity/order reconciled; source read does not verify shipping product.",
      "test_evidence":"Current descendant HEAD must execute the relevant professional harness plus normal Rust desktop runtime and Desktop Chat Parity CI.",
      "disposition":"mapped-open-test-harness","unknown_closed":False,"omitted":False,"consumer_evidence":[],
      "reachability_status":"accepted-tree exact blob responsibility proven; professional test-harness closure open",
      "license_and_provenance":"tdesktop provenance retained; responsibility is mapped/reimplemented rather than source-copied.",
      "notes":"Read and decomposed under live 811b83a1 deterministic recursive authority. Reading closes no unknown and grants no production/release credit.",
      "consumer_symbol":symbol,
      "source_binary_evidence":{"evidence_status":"live-github-tree-bound-current-head-actions-required","accepted_upstream":NEW,"verified_blob_sha":sha,"byte_size":size,"file_type":"blob","recursive_order":order}
    })
dump(shard_rel,{"format_version":2,"project_id":"TDRP-001","spec_revision":9,"upstream":{"repository":"telegramdesktop/tdesktop","commit":NEW,"tree":NEW_TREE},"range":{"first_order":5794,"last_order":5801,"entries":8},"rows":rows})
dump("projects/telegram-desktop-rust/inventory/source-dispositions.json",master)

att_dir=ROOT/"projects/telegram-desktop-rust/inventory/source-attestation-manifests"
for p in att_dir.glob("*.json"):
    j=replace_exact(json.loads(p.read_text()))
    if "accepted_upstream" in j: j["accepted_upstream"]=NEW
    if "accepted_tree" in j: j["accepted_tree"]=NEW_TREE
    p.write_text(json.dumps(j,ensure_ascii=False,indent=2)+"\n")
att={"accepted_upstream":NEW,"accepted_tree":NEW_TREE,"deterministic_order_basis":"non-directory/blob rank in accepted recursive tree","start":5794,"end":5801,"entries":[{"order":r["recursive_order"],"path":r["source_path"],"blob":r["source_blob_sha"],"consumer_symbol":r["consumer_symbol"]} for r in rows]}
dump("projects/telegram-desktop-rust/inventory/source-attestation-manifests/5794-5801.json",att)

acq=replace_exact(load("projects/telegram-desktop-rust/inventory/build-time-acquisitions.json"))
acq["upstream_commit"]=NEW
for key in ("github_action_ref_resolutions","github_action_immutable_authority","github_clone_ref_resolutions","github_clone_immutable_authority"):
    if isinstance(acq.get(key),dict) and "accepted_upstream" in acq[key]: acq[key]["accepted_upstream"]=NEW
dump("projects/telegram-desktop-rust/inventory/build-time-acquisitions.json",acq)

lock=load("projects/telegram-desktop-rust/upstream.lock.json")
lock["upstream"]["commit"]=NEW; lock["upstream"]["tree"]=NEW_TREE; lock["upstream"]["committed_at"]=api(f"/repos/telegramdesktop/tdesktop/commits/{NEW}")["commit"]["committer"]["date"]
lock["coverage"]["root_source_entries_total"]=6653; lock["coverage"]["recursive_source_entries_total"]=16125
lock["coverage"]["unknown_minimum"]=15846; lock["coverage"]["unread_minimum"]=10324; lock["coverage"]["omitted_known"]=0
lock["coverage"]["note"]="Accepted 811b83a1 tree 4ddb5182; read-through 5,801; unknown=15,846, unread=10,324, omitted=0."
lock["source_disposition_evidence"]["upstream_commit"]=NEW; lock["source_disposition_evidence"]["read_through"]=5801
lock["acceptance"]["note"]="Live upstream 811b83a1; exact blob read-through 5801/16125; unknown/unread closure remains open; fresh descendant exact-head production/release evidence required."
dump("projects/telegram-desktop-rust/upstream.lock.json",lock)

index=load("projects/telegram-desktop-rust/inventory/index.json")
index["upstream"]["commit"]=NEW; index["upstream"]["tree"]=NEW_TREE
index["inventory"].update({"root_non_directory_entries":6653,"recursive_non_directory_entries":16125,"unknown_minimum":15846,"unread_minimum":10324,"omitted_known":0})
index["inventory"]["note"]="Live 811b83a1 recursive census 16,125; accepted-tree blob order exact through 5,801; unknown 15,846, unread 10,324, omitted 0."
index["note"]="Live authority exact through deterministic blob order 5,801; unread_minimum is 10,324; unknown remains 15,846; omitted remains 0."
index["rebaseline"]={"from_commit":OLD,"to_commit":NEW,"ahead_by":1,"changed_paths":1,"changed_authority_paths":EXPECTED_CHANGED,"current_head_source_authority_evidence":"Fresh exact-head Source authority evidence required after 811b83a1; predecessor runs/artifacts are historical only.","current_source_read_through":5801,"first_unread_order":5802,"first_unread_path":"Telegram/SourceFiles/test/test_menu.cpp"}
index["source_dispositions"]["upstream_commit"]=NEW; index["source_dispositions"]["read_through"]=5801
dump("projects/telegram-desktop-rust/inventory/index.json",index)

ledger=load("projects/telegram-desktop-rust/parity-ledger.json")
ledger["upstream_commit"]=NEW; ledger["coverage"]["source_entries_total"]=16125; ledger["coverage"]["unknown"]=15846; ledger["coverage"]["unread"]=10324; ledger["coverage"]["omitted"]=0
ledger["source_dispositions"]["upstream_commit"]=NEW; ledger["source_dispositions"]["read_through"]=5801
ledger["rebaseline"]={"from_commit":OLD,"to_commit":NEW,"ahead_by":1,"changed_paths":1,"changed_authority_paths":EXPECTED_CHANGED,
"source_read_reconciliation":"Accepted-tree blob order exact through 5,801/16,125; unknown 15,846, unread 10,324, omitted 0. Orders 5,794-5,801 retain exact identity under 811b83a1; first unread is 5,802 test/test_menu.cpp. The new chat_theme_readability.cpp blob is order 5,951 and remains unread/unknown.",
"source_authority_evidence":"Deterministic ordering is derived from non-directory/blob rank in accepted recursive tree 4ddb5182fc1ae238e4dc62a396614614b5b35f97. cf478d37 evidence is historical only.",
"current_source_read_through":5801,"first_unread_order":5802,"first_unread_path":"Telegram/SourceFiles/test/test_menu.cpp"}
dump("projects/telegram-desktop-rust/parity-ledger.json",ledger)

authority=f"Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@{NEW} (root tree {NEW_TREE}). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,801; first unread=5,802 Telegram/SourceFiles/test/test_menu.cpp@93cdaf3adf82725adb07e90be83e560d40a7b72d; unread=10,324; unknown=15,846; omitted=0. Upstream delta cf478d37→811b83a1 modifies order 5,951 Telegram/SourceFiles/ui/chat/chat_theme_readability.cpp only; that product responsibility remains unread/unknown. Runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 remain historical-only for 863cf10d+b4d9f7f8."
docs=[
"docs/specs/telegram-desktop-rust-equivalence-migration.md",
"projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md",
"projects/fabushi-communication-platform/quality/requirements-traceability-matrix.md",
"projects/fabushi-communication-platform/management/tasks/P0-product-domain-and-telegram-absorption.md",
"projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md",
"projects/telegram-desktop-rust/management/tasks/P0-source-closure-and-ledger.md",
"projects/telegram-desktop-rust/STATUS.md",
]
note="\n\n### Revision 9 live read-through note: 5794-5801\n\nOrders 5,794-5,797 preserve professional evidence-log one-line/completion-forgery integrity plus an independent raw-byte oracle. Orders 5,798-5,799 preserve complete mapped-target/viewport capture readiness. Orders 5,800-5,801 preserve the bounded reversible not-marking-read evidence lever. These are test/evidence responsibilities only, create no second product owner, and close no unknown. The new 811b83a1 readability delta is order 5,951 and remains unread/unknown.\n"
for rel in docs:
    p=ROOT/rel; body=p.read_text()
    lines=body.splitlines(True)
    if lines and lines[0].startswith("Live Revision 9 authority"): lines[0]=authority+"\n"
    else: lines.insert(0,authority+"\n\n")
    body="".join(lines)
    if "Revision 9 live read-through note: 5794-5801" not in body: body+=note
    p.write_text(body)

dossier=ROOT/"projects/telegram-desktop-rust/dossiers/test-professional-harness-5794-5801-complete-read.md"
dossier.write_text(f"""# Revision 9 exact-blob read dossier — orders 5794–5801

Accepted upstream: `{NEW}`
Accepted tree: `{NEW_TREE}`
Disposition: read-complete / responsibility-decomposed / mapped-open professional test harness
Unknown closed: 0
Omitted: 0

- 5794–5795: one-call/one-physical-line evidence writer, splitlines escaping, distinct PASS/FAIL/N/A and terminal-completion integrity; telemetry-number mode is one-way and must never hide fixture-secret data.
- 5796–5797: independent raw-byte line grammar oracle, including CRLF, all separators, trailing/separator-only payloads and attempted TEST_COMPLETE forgery.
- 5798–5799: complete mapped-target/viewport visual readiness, preserving full requested frame; overlap-only, hidden, empty, outside and nonpainting targets fail closed.
- 5800–5801: bounded reversible test-only minimize lever that prevents marking messages read only after a positive control, proves activation undoes it, reapplies if needed, and restores the real window/exposure.

No Telegram-named product owner, fake fallback, softened threshold, unknown closure or release credit is introduced.
""")
print(json.dumps({"upstream":NEW,"tree":NEW_TREE,"read_through":5801,"first_unread":5802,"theme_order":theme_order}))
