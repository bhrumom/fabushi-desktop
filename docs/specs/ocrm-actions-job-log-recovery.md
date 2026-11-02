# OCRM exact-job log recovery carrier

Status: specified before workflow edit; recovery execution pending.  
Implementation base: `ad9f5c9c925bfe0909558bdd35a4d345f1cc02ce`.  
Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`.

The existing completed Runtime run `41477395549`, attempt 1, tested subject
`18ef48cf1b72c045141f56b227be79226056113b`. Host job `125387737100` failed
in recovery after the compile, Settings, app and Native Runtime steps succeeded.
Two decoded-log connector requests returned Transport closed without a body.
That retrieval error does not classify the product failure. Its actual error
and nonzero test results still require the original job log.

Use the existing OCRM P0 source workflow as a read-only evidence carrier. A
writer-branch push whose commit message contains `[ocrm-recover-18ef-log]`
enables one additional job, with only contents/read and actions/read permission.
The existing source checks, frozen U, source gates and inventory writer retain
their current behavior. No product command or failed job is rerun by the carrier.

Before downloading, read the specified run attempt and job through the normal
GitHub Actions API. Require the exact repository, run, attempt, source SHA,
job ID/name, completed state and original failure conclusion. A mismatch fails
closed and cannot produce a successful recovered-log receipt.

Fetch only that completed job's existing GitHub-processed log. API credentials
must go only to api.github.com. Follow HTTPS log-download redirects only to
the approved GitHub Actions/Azure storage hosts without forwarding credentials;
reject userinfo, non-HTTPS, unknown hosts, excessive redirects or bodies over
100 MiB. Do not print signed URLs, request headers or tokens. No annotation API
or other unsupported repository endpoint is needed.

Save the source run/job metadata, exact log bytes, SHA-256, byte/line counts and
a receipt identifying both source subject and carrier SHA/run/attempt. Mark
explicitly that the carrier executed no product validation and that the original
job failed. Upload these files even on collection failure; a safe failure record
may describe an HTTP status/type but must not expose a signed URL or token.
Only a successful validated download may create the success receipt.

The source workflow's success at a newer carrier SHA is not a Runtime pass for
either SHA. Subsequent failure classification and test counts must come from
the recovered original log; preserve the original failed verdict and retrieval
history. The artifact establishes transport and identity, not independent
acceptance, full source closure, G01–G12 or AC01–AC24 completion.

## 2026-11-02: original 563 command evidence with no local executor

The preceding section records the historical 18ef recovery contract. This
extension uses the same collector for source H
`56386a66a1f2944545cee35b58d10cb085e3c292`, tree
`86f8fded50a54fc60b8f3a5d69af3af735e6c12a`, Runtime run `41498228438`,
attempt 1, Host job `125459002378`. Both original run and job are completed
with failure; the Recovery step failed, but its current raw signature remains
unclassified. The unavailable local executor prevents materializing the existing
command ZIPs. This is an evidence-instrument gap, not an environment diagnosis
of that Runtime failure. Earlier 233e outcomes remain their own evidence.

- REC-01: bind this exact repository, source SHA/tree, Runtime workflow, run,
  attempt, Host job/name and original failure conclusions before downloading.
  The carrier must preserve the source verdict and identify its separate commit,
  run and attempt. It executes no product validation.
- REC-02: retain the existing bounded HTTPS downloader and token-free approved
  redirects. Save the complete original GitHub-processed Host log and its hash,
  bytes/lines and source metadata. No signed URL, header, token or unsafe
  exception text is printed.
- REC-03: read only source artifacts `13167684148` (Recovery) and `13169190332`
  (Native), requiring their exact names, run/head metadata, unexpired state,
  recorded byte lengths and SHA-256 digests. The Recovery digest is
  `ee0c70b19203bd452d25510da83f4b7dd686968afd3b468cc13061e155d8c583`;
  the Native digest is
  `6fe5f43e03285d5516089e214bbc76510c914f7cad9b84f570e1913db7ae19f9`.
  Check their subject receipts against source H/tree/U/run/attempt, and check
  each declared input digest against the same exact-H Git file. Require the
  original job's matching upload steps to have succeeded. An artifact's own
  JSON cannot establish authority by itself.
- REC-04: inspect bounded ZIP members in memory; never extract or execute the
  archive helper. Reject duplicate or unsafe paths, symlinks, encryption,
  unsupported compression and excessive member/expanded sizes. Preserve the
  original archives. Emit recorded command/exit pairs, all Rust harness target,
  named outcome and summary lines, and bounded GitHub-processed failure context
  as JSON-escaped records with exact source filename/line numbers. Keep emitted
  output below 2 MiB/12,000 records; overflow fails rather than silently omitting
  evidence. Keep child summaries distinct; the collector does not sum tests or
  issue a Runtime verdict. Missing command/exit/log members fail collection.
- REC-05: run collection controls in this Actions job for wrong subject/attempt
  and unsafe/duplicate ZIP member refusal. These are instrument controls, never
  product tests. A success receipt requires all requested sources, hashes,
  input identities and bounded output to have been collected successfully.
- REC-06: the single writer push marker `[ocrm-recover-563-command-evidence]`
  activates this collection. Preserve `source-authority` unchanged; exclude
  only that marked push from `persist-verified-inventory` so the read-only
  carrier cannot create a derived commit or dispatch product checks. Normal
  events retain their current writer behavior. Verify the final two-path scope
  does not match Runtime's push paths and recheck for an open writer PR before
  publication. No source gate, runtime selector or global completion flag is
  removed. The only edited paths are this spec and the existing OCRM source
  workflow; both require immutable independent review before the leased push.

Keep collected raw command artifacts at their existing seven-day retention,
including the carrier copy. A newer green collector proves evidence transport
only. Actual failure classification, named counts and subsequent repair approval
must use the original records. No current 61dc result is inferred from this
563 collection; its own exact Runtime evidence remains separately required.
