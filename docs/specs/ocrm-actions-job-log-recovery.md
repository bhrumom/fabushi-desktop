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
