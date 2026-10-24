# User-agent updater workflow complete read — strict index 95

## Authority
- Upstream: telegramdesktop/tdesktop@83f9f56347cc3421c1e3ef537c7bb6dd06ce7cd0, tree bb0159bfedb38dd96e579af4ffe6360b6beb7332.
- Source: .github/workflows/user_agent_updater.yml@1e3dae8182aefc4e4c76df065f547a43a1e50437, 17 line slots / 331 ASCII bytes; complete read and SHA revalidated.
- Provider: desktop-app/action_code_updater@master -> 0a8d3ebb437e5c77cf5d066d4a54cc93d34b534a, tree 5f51a9339cccc975911e3dcff02027395f2dee61.
- Provider root: action.yml@5a9c7dff415a31f18a614119809f2d396b0e47c7, index.mjs@322bfa9e3e874299a09c35650611c2cd753cca3f, check_unused_styles.py@6a061912121f49ffa386ff9692b989378b6bdd2b; no committed package manifest, package lock, or LICENSE in this audited tree.
- action.yml runs npm init and unversioned runtime installs of simple-git, isbinaryfile, @actions/core, and @actions/github.
- Consumer definition: Telegram/SourceFiles/mtproto/details/mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9 lines 64-70 defines DnsUserAgent() with Chrome/151.0.0.0.
- Consumer call: Telegram/SourceFiles/mtproto/special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88 line 338 sets User-Agent to DnsUserAgent() for Mozilla, Google, RemoteConfig, Realtime, and FireStore special-config HTTPS attempts.
- Parent exact-head 63c3e018e4299dc2bd2d7aecabd3417956fffbb6/tree 522c7aa20a9343329e8f6d4e9486db80d72bc1a6 completed TDRP run 39893740985 with all 30 returned jobs successful (authority 12476225945 sha256:3a53a48a775f8f6ba926e6a212ba4599830a8c0aa380f76a2dda2891854729de; closure 12475679271 sha256:6ddaab2d6b8944246f1a691931045b7410cc1890e31c374de29ca6e8ca42e0a2; reachability 12476490692 sha256:6b9ed74b703c1891f5c322dd8f0b72bf44fa395cdacd51c545cbfddca9f4381e) and Rust desktop runtime run 39893740983 at 6/6 success (artifact 12476097221 sha256:3ffab22ed376341be1bed979aad8131ef7097e9b230dc27dadb4d14ed441165c).

## Complete-read behavior
The workflow runs at 00:00 UTC on day one of each month and on its named repository_dispatch event. It has no workflow_dispatch entry. The provider selects the UserAgent updater for type user-agent.

The provider executes google-chrome --version, applies a four-number regular expression whose dot separators are not escaped, extracts the first decimal major, and reduces the replacement to major.0.0.0. It recursively scans the cloned repository, excludes .git paths and binary files, and replaces every matching Chrome/<four numeric fields> string; the mutation is not syntactically scoped to DnsUserAgent(). Frozen exact-source search has one Chrome/151.0.0.0 hit, the DnsUserAgent() literal above.

No modified files is converted to successful exit 0. With changes, the provider proposes chrome_<version>, rejects an already-visible matching branch, commits/pushes, then lists/creates a PR. PR API exceptions are caught and logged without rethrowing, so green can coexist with a pushed branch whose PR was not created. No concurrency group serializes overlapping runs.

## Existing-owner-first and consumer counterexample
The workflow responsibility is repository maintenance/proposal provenance and maps to Fabushi's existing contribution, review, quality and exact-head evidence lifecycle. It is not a shipping HTTP owner.

The name-adjacent shipping source/host/src/extensions/browser_ua/extension.rs@57bdfdf9105a1ab6a63d9bcfb7c1b3e0b37fd0d9 plus host_production_extensions.rs@6294645234707fef8472aff85737c2550252fd0b was inspected. That BrowserUa owner wires Auth/Experiments to an owner-stamp file and token kill-switch marker; source/host/tests/extensions_browser_ua_contract.rs@9c912f03defd4ee5e57c51b50c218f3641acef46 verifies that marker lifecycle. It does not set an outbound HTTP User-Agent header and is therefore rejected as an equivalent DnsUserAgent() owner.

The direct DnsUserAgent()/SpecialConfigRequest networking behavior remains an explicit follow-on source responsibility. When that consumer is audited, current HTTP/network request owners must be checked first; a new minimal source-neutral compatibility-header owner is allowed only if exact-head owner-absence evidence requires it. This workflow row does not create one.

## Fail-closed conclusions
- Provider master, runtime npm packages, and runner Chrome are mutable provenance.
- Missing committed provider LICENSE/package-lock is incomplete provenance, not a legal conclusion.
- Wildcard dot separators make the version matcher broader than literal dotted versions; future exact diffs must be reviewed.
- Runner Chrome major is an observation, not proof of remote-service compatibility.
- A successful no-op does not prove every compatibility header is current.
- Existing branch, overlapping runs, cancellation, push/PR split settlement, and swallowed PR errors are falsifiable outcomes.
- github.token is repository administration authority only, never a product credential or acceptance authority.
- Green cannot prove merge, shipping HTTP behavior, source closure, release readiness, or migration acceptance.

## Exact traceability IDs
- Requirement: TDRP-R9-USER-AGENT-MAINTENANCE-PROVENANCE-001
- Oracles: ORA-TDRP-USER-AGENT-EXACT-SOURCE-001, ORA-TDRP-USER-AGENT-CONSUMER-TRACE-001, ORA-TDRP-USER-AGENT-NO-PUBLICATION-AUTHORITY-001
- Invariants: INV-TDRP-USER-AGENT-NO-PARALLEL-RUNTIME-001, INV-TDRP-USER-AGENT-NOOP-NOT-SETTLEMENT-001, INV-TDRP-USER-AGENT-PR-GREEN-NOT-SETTLEMENT-001

## Disposition
Strict index 95 closes only the workflow/provenance responsibility. The direct network consumer is traced but not falsely marked implemented. baseline_identity_accepted=false and source_closure_ready=false remain unchanged.
