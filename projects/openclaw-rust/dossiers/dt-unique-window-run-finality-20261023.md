# OCRM follow-up — unique historical publisher windows, corrected Sand execution owner path, and terminal-finality risk

Task 3dd43a37-9f36-4b7a-adac-dfcc87584523. Audit-only, distinct evidence added after the committed Round 41 dossier (commit 84161293ab7b7b793ae439d766163b576e338cd6). This dossier is **not** an independent approval, successful publisher-hash reproduction, shipping code change, or OCRM completion receipt.

## Frozen subject and current boundaries

- Implementation subject: bhrumom/fabushi-desktop PR #169, **open / unmerged**, HEAD b5db35fa272c83833685d70a730b5f8285fa1176, tree d01637ba8d83cd6c723077555ccf725887f7156e.
- Canonical main independently read: 7394f6dc7687f9b986ee5589d40aa680ce23b90a, tree fbeb0bb32d944d0c1053880c471fe5c52279f052. PR base is historical; neither PR nor audit carrier equals live main.
- OCRM frozen OpenClaw: ad8d12f126d29b0db204187adf5092db80294e70, tree 04ccce62f1d74dc8e0c88290ae8ded027b7b0f16 (earlier locked and re-used, not rebaselined). New live OpenClaw main was read at 313b84fd5518a707b8101409fc18e4a2739f51b1, tree 78ed14aa495cd91b82e09e4c5b9e4d69962dbf7e; its drift is **not** accepted as a new source denominator.
- Audit-only carrier lease before this edit: 84161293ab7b7b793ae439d766163b576e338cd6, tree b8d4843feb78ae986d20e98a2b6fcde4380b24fb. Subject and carrier evidence remain separate.

## New independent timing observation: exactly one master-ancestry commit in each post-precursor window

First-party Microsoft Actions job logs were re-read for predecessor and publisher runs. Fresh GitHub REST commit-history queries against DefinitelyTyped/DefinitelyTyped's master ancestry returned the following **single** commit within each bounded post-precursor / pre-publisher-GET interval (committer-date filter, <=100 per page, returned count=1).

| Historical path | Predecessor immutable publisher job GET | Candidate Git commit and committer time | Later publishing job GET | GitHub master-history result in specified interval |
| --- | --- | --- | --- | --- |
| pg | run 32061431389/job 95483367518; 2026-08-17 19:39:37.031Z request, 19:39:46.095Z received | 97ba786e647c0899a2dd6d1b6807896762e725fa, 19:46:36Z, tree 022e9d61402858052bb76a773c093249f50435a3 | run 32063758027/job 95490787861; 20:04:47.512Z request, 20:04:55.314Z received | Query 19:46:36Z..20:04:55Z returned only 97ba786e... (merge PR #75378; changes pg declaration) |
| node | run 37970622331/job 113955996269; 2026-10-09 18:04:30.306Z request, 18:04:38.261Z received | a92efe8ec26d5931fbefff5e6daa5d6b2cc1fd06, 18:38:54Z, tree 6d321f29b7d4e42f314d21d28d81b18855546177 | run 37974941076/job 113970739568; 18:41:56.430Z request, 18:42:04.696Z received | Query 18:38:54Z..18:42:05Z returned only a92efe8... (merge PR #75653; its tip is the candidate node input tree) |

GitHub source references:
- https://api.github.com/repos/DefinitelyTyped/DefinitelyTyped/commits?sha=master&since=2026-08-17T19%3A46%3A36Z&until=2026-08-17T20%3A04%3A55Z&per_page=100
- https://api.github.com/repos/DefinitelyTyped/DefinitelyTyped/commits?sha=master&since=2026-10-09T18%3A38%3A54Z&until=2026-10-09T18%3A42%3A05Z&per_page=100
- https://github.com/microsoft/DefinitelyTyped-tools/actions/runs/32061431389 ; https://github.com/microsoft/DefinitelyTyped-tools/actions/runs/32063758027
- https://github.com/microsoft/DefinitelyTyped-tools/actions/runs/37970622331 ; https://github.com/microsoft/DefinitelyTyped-tools/actions/runs/37974941076

The **new** fact beyond earlier precursor comparisons is the zero-intervening-commit count for each master-history time slice, from the official bounded collections. This materially narrows the candidate-window ambiguity. It is retrospective commit ancestry and timestamp evidence, **not** immutable witness of mutable codeload tar.gz/master HTTP response identity, nor of absence of a historical force-ref change. Both publisher GETs independently downloaded master again **after** their predecessor caches were restored. Their logs provide no source response commit/tree, ETag or digest. Actions cache-list API was denied as unsupported by this connector; predecessor and follower Actions artifacts were empty. Do **not** elevate candidate identity or cache key to historical tarball provenance.

The historical getContentHash inputs remain candidate pg 4 declarations + package.json, node 89 declarations + package.json, from the individually pinned blobs in prior dossiers. Earlier 1/4 pg and 1/89 node declaration-input deltas are unaffected; no SHA256 reproduction was executed in this environment or in newly authorized Actions. Expected npm package contentHashes remain @types/pg@8.23.1 4c3006364fd06c962ce8e915a3dcf8d0d87bfec39f509aa95b270ad620cad104 and @types/node@26.6.5 3196021f34db9bcb68a729bd29ff60434d3b4c5f49619270db25e7dbd6dc651c. **Publisher hash reproduced 0/2; historical immutable HTTP source receipts 0/2.**

## New path correction and additional Sand terminal-finality source audit

**Correction to Round 40 dossier:** the claimed path source/host/src/automation_run_path.rs is not present under PR #169's subject tree. Recursive GitHub tree inspection of source/host/src/extensions (tree 09fd994276c96a1a55eb38e509c4516af8c0d646, 395 entries, untruncated), followed by exact ref fetch, establishes the actual source path **source/host/src/extensions/transcript/automation_run_path.rs** and the same asserted immutable Blob 7bedae399879f7269cfded2c5bf629ad9204a0b1. Correct the path when reproducing and reviewing prior source claims; this file has *not* been modified.

New source-level observations binding actual producer/store/consumer:
- Exact subject source/host/src/extensions/transcript/automation_run_path.rs@7bedae... lines 149-171 holds an in-memory in-flight key (agent_id, automation id) for non-Event triggers only; Event triggers skip this admission suppression. Lines 193-209 call record_run **before** begin_run, and lines 230-231 execute the provided callback after begin_run even if it returned a preexisting run with the same ID. A redelivery can therefore advance last_run_at even when its runUuid already exists; an in-memory admission guard cannot alone close process-death recovery.
- Exact subject source/host/src/automations/automation_store.rs@8ffd2175c7afb65611f65a1d56f3368b79881220 lines 42-46: record_run updates last_run_at; begin_run returns an existing run unchanged, including running; finish_run_with rewrites finished_at/status/detail for matching run IDs without an explicit already-terminal or generation guard. This suggests an additional **terminal-result-overwrite** race under duplicate/later callbacks; no failing execution or actual duplicated side effect has been observed.
- Exact subject source/host/src/extensions/automations/fire_delivery.rs@04e3a192e7fa9b1d3bda7ea1757084153679bd79 lines 137-160 returns no cached completion for a prior run whose persisted status is running (returns one only for terminal states). This can feed a restart redelivery back into the above path.
- Exact subject source/host/src/extensions/automations/sand_automation_fire_consumer.rs@b38b1614a95a34bbfe34acff39b8c3cdfaae943e lines 135-147 initializes pending fire states in a fresh process-local BTreeMap. Lines 187-207 and 336-355 retry non-404/409 completion failures, treating 404 or 409 as Reported. Lines 267-310 ACK reported IDs and dispatch absent IDs; recovery needs an authoritative server contract.
- Exact subject source/host/src/extensions/automations/backend_transport.rs@6dc4477c72a38179d71ab33013c9b7e2c3b89991 lines 65-100 re-read the current bearer for each POST and JSON-decode all 2xx. An empty 204 can thus become a Decode error; a late A-account callback could be attempted under B-account current bearer without a proved account-generation fence. This is a falsifiable source risk, not evidence of a production compromise.

Add a **proposed** (not approved or executed) AUTO-RUN-FINALITY-01 oracle to existing AUTO-RUN-AUTH-LATE-01/AUTO-RUN-CRASH-01: redeliver an already-terminal runUuid concurrently/after restart with conflicting subsequent execution outcomes; verify no second external mutation, no last_run_at advancement attributable to stale fire, and no overwrite of terminal result without an explicit authorized reconciliation transition. Retain source-count oracle plus 200 JSON / 204 empty / 401 revoke / 404 scoped-missing / 409 duplicate-or-conflict, per-account generation, missed response, ACK and crash replay. Do not implement tests/validators before scoped G01/G02, owner/oracle and independent review positive admission.

This audit does **not** assert that test semantics or the external deployed Sand service contract has already been established. GitHub code search of available bhrumom/fabushi-backend default branch still did not show the actual /sand/automation-runs/complete service route or authoritative deployed revision. The existence of backend CI workflows is not a Sand deployment receipt; Cursor default API host is not Fabushi server ownership proof.

## Official npm acquisition contract and unresolved bytes

New first-party npm registry specification read: npm/registry docs/responses/package-metadata.md Blob af4f3c77d913d45e68e36c755c1c572e8954ddc9. It defines registry package metadata GET, version-keyed metadata, and dist.tarball/dist.shasum/dist.integrity; dist.integrity is a package-tarball SRI field, **not** the DefinitelyTyped publisher's SHA256 getContentHash. npm's official provenance documentation also distinguishes signed provenance and publish attestations (https://docs.npmjs.com/generating-provenance-statements/). These define appropriate future verification sources but provide **zero actual bytes** for the pinned 14 published packages.

The 14 frozen PostgreSQL exact-version packages remain the enumerated set in Round 39 (pg, pg-cloudflare, pg-connection-string, pg-protocol, pg-pool, pg-int8, pg-types, pgpass, postgres-array, postgres-bytea, postgres-date, postgres-interval, split2, @types/pg). Official registry exact-version URLs were unavailable to the current web reader, and the GitHub connector rejects npm-host URLs. Actual Registry dist.integrity 0/14; independently rehashed published tarballs 0/14; actual packed LICENSE/NOTICE 0/14; package-specific signed provenance unverified. Do not invent expected SHA512 or use lock SRI as an archived-package receipt.

## Pending decisions, protection and executable frontier

- OCRM-DTPUBLISH-ARCHIVE-01: immutable historical codeload response provenance 0/2, independently executed publisher hash 0/2, reviewed Actions-only scoped gate not admitted.
- OCRM-NPM-PG14-DIST-01: 14 package integrity/archive/LICENSE/NOTICE/provenance unresolved.
- OCRM-SAND-OWNER-01, AUTO-RUN-AUTH-LATE-01, AUTO-RUN-CRASH-01, proposed AUTO-RUN-FINALITY-01: actual deployed owner and HTTP/account/replay authority absent; no production test or positive independent oracle.
- The exact subject's prior Actions source-authority run 39770034071 proved tracked-tree and **structural** G01/G02 for 58,750 rows/58 shards (artifact 12420959002), not dependency/Sand/full semantic closure and not a receipt for this audit carrier.
- Existing support requests were found by read-only mail search and not resent. Full Gmail thread reads were denied by connector; do not infer a reply or lack of reply from the inability to open threads. No private mailbox contents/identifiers are published here.
- No local build/test/hash/generator/schema/validation and no new Actions run. No product, pairing writer, Rust source, test, validator, workflow, source lock, inventory, parity-ledger, RTM, flag, release or deployment changes. No independent agent/reviewer invoked. This one new audit-only file is the sole permitted mutation.

Next allowed evidence action: obtain an authenticated official cache-object/HTTP response identity channel *if newly authorized*, else independently inspect exact-source account-generation and terminal-result call sites or a separate source-authorized P0 unit; seek approved Actions-only scoped G01/G02/owner-oracle adjudication before any executable hash gate. Do not silently rebaseline the upstream.
