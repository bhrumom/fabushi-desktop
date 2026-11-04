# Complete read — fixed-U repository user-agent updater workflow

Date: 2026-11-02  
Task round: 177  
Desktop base: `b23ea7be65bf659e8830cd0f92bf3de24c4df2fe`  
Platform-core authority: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, non-truncated, 7,194 entries  
Source: `.github/workflows/user_agent_updater.yml@1e3dae8182aefc4e4c76df065f547a43a1e50437`  
Tree index: 133 zero-based  
Responsibility: `TDRP-R9-REPOSITORY-USER-AGENT-UPDATER-WORKFLOW-001`

## Source workflow

The exact source is mode 100644, 331 ASCII bytes/code points, 16 LF, zero CR and a trailing newline. Admissions are exact repository dispatch `Restart user_agent_updater workflow.` and monthly cron `0 0 1 * *`; there is no manual workflow dispatch. The single job uses mutable `ubuntu-latest`, no explicit permissions/environment/concurrency/timeout/retry/checkout, and mutable `desktop-app/action_code_updater@master` with `type: "user-agent"`.

All 20 fixed-U workflow blobs were checked for `user_agent_updater` / the user-agent type contract; only this workflow contains it. Fixed-U therefore identifies scheduler admission plus an external repository-dispatch producer, not a reusable local workflow caller.

## Action lineage and runtime

Observed Action master remains `0a8d3ebb437e5c77cf5d066d4a54cc93d34b534a`, master unprotected, commit unsigned, tree `5f51a9339cccc975911e3dcff02027395f2dee61`. Files are `action.yml@5a9c7dff415a31f18a614119809f2d396b0e47c7`, `check_unused_styles.py@6a061912121f49ffa386ff9692b989378b6bdd2b`, and `index.mjs@322bfa9e3e874299a09c35650611c2cd753cca3f`; no LICENSE/package manifest/lock exists. The composite action runs `npm init -y`, installs four named packages without versions, and passes `github.token` on Node argv.

`UserAgent` runs `google-chrome --version`, parses a pattern based on `[0-9]+.[0-9]+.[0-9]+.[0-9]+` whose dots are regex wildcards, takes the first digit run as major, and derives `major.0.0.0`. It globally replaces `Chrome/<four-component-pattern>`. Missing parsed version exits 1.

## Discovery and mutation

For this mode clone options are empty, so the Action clones the repository's current default branch through a token-bearing URL, not fixed U/event SHA; current repository metadata says `dev`. It recursively enumerates files, drops every path containing substring `.git` (including `.github`), excludes files classified binary, decodes remaining bytes as Node UTF-8 strings, and rewrites every changed file. No modified files is handled as exit 0. Candidate evidence must therefore bind default-branch SHA, runner/Chrome/runtime package identities, traversed/excluded paths, binary decisions, input/output bytes and regex matches.

At fixed U the relevant product anchor is `Telegram/SourceFiles/mtproto/details/mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9`: `DnsUserAgent()` contains `Chrome/151.0.0.0` and DomainResolver sends it as `User-Agent`. `Telegram/SourceFiles/mtproto/special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` uses the same helper for special-config HTTP requests. This is a direct consumer link, not permission to duplicate the canonical networking owner in this repository-maintenance entry.

## Publication, security and recovery

Derived branch is `chrome_<version>`; commit and PR title are `Update User-Agent for DNS to Chrome <version>.`. The Action checks fresh-clone local branch state, creates a local branch, stages all changes, commits, pushes, then does an unpaginated PR list/create. A remote same-name branch may escape local enumeration. PR API exceptions are caught without rethrow, so pushed branch without PR is partial publication. There is no explicit concurrency, timeout, retry, rollback, transaction or merge.

Fail-closed boundaries: mutable Action/runner/Chrome/npm/default branch, implicit token scope, token in argv/remote URL, unescaped regex dots, broad repository scan, `.git` substring over-filter, invalid-UTF8 replacement, binary-classifier drift, remote-branch collision, swallowed PR errors, and source automation being mistaken for Fabushi networking authority.

## Fabushi disposition

Existing-owner-first maps this to root repository source-maintenance/process + repository security/provenance + shared quality/evidence. DNS/special-config runtime behavior stays with the canonical networking owner. No Telegram updater/runtime/provider, second networking truth, Product Tasks/Automations owner, shipping updater, or platform-core change is introduced.

Authored only and not run: 28 contract, 1 integration, 15 fault/recovery, 12 security/provenance. Status: `implemented / validation_status=not-run / accepted=false`.
