# Bot-owned Mini App creation, edits and GitHub delivery

Status: implementation, 2026-10-21. User asks Fabushi Bot to generate and modify
Mini Apps and update their source on GitHub end to end. The earlier requirement
still applies: Mini App business code arrives with the installed package.

## Existing owners and design

Extend Runner's existing turn tools and Box resource execution; reuse Shell's
review, cancellation and remote-Box availability. GitHub discovery, authorization,
repository/branch/commit/PR operations remain the existing GitHub MCP connector.
No new credential owner, server, agent runtime, marketplace or conversation UI.
The hardcoded Dandelion0.2.1 publisher remains its historical bounded owner; it is
not widened into a signing bypass or a generic GitHub token proxy.

Expose a generic `MiniApp` authoring tool only when the current Bot has an isolated
Box Shell. Its create/edit/inspect/export operations execute an included Python
SDK through the existing reviewed Shell bridge. Per-project immutable revisions
live only in that agent's /workspace. Bot supplies its generated HTML/CSS/JS via
bounded edits, not host-bundled application business logic. Export produces a
source tree with a standalone packager and GitHub Actions workflow. GitHub MCP
publishes those files through ordinary visible tool calls and its existing grants.
This intentionally composes existing tools rather than hiding GitHub mutations
inside another tool and bypassing their approval/audit policy.

## Requirements

- BA-01: create a working offline project with bounded valid identity/version,
  accessible HTML/CSS/JS, no remote network, unsigned catalog preparation and CI.
  Include editable behavior tests under tests/ and execute them with pinned Node
  on Actions before packaging; the starter test exercises its actual script and
  click handlers against a bounded DOM seam. This is unit behavior evidence, not
  a claim of browser/visual acceptance for arbitrary generated applications.
- BA-02: edits require an exact expected revision and per-file expected digest
  (null for a new file). Validate the whole batch before an atomic current-pointer
  switch. Retain immutable previous revisions. Reject traversal, symlinks,
  unknown fields, oversized source, stale edits and accidental overwrites.
- BA-03: inspect/export return exact revision and per-file SHA256 with bounded
  UTF-8 source. Export is a GitHub-compatible file list including config, packager
  and Actions workflow. A source update never implies installed-code mutation.
  Export also writes an immutable, content-addressed JSON handoff in the agent's
  workspace. Its exportId covers the complete source tree including SDK build
  tooling; project revision alone covers user config/source/tests. SDK upgrades
  therefore do not overwrite an older export or masquerade as identical bytes.
- BA-04: include a deterministic dependency-free packager. Actions derives the
  exact GITHUB_SHA, validates resources, and emits manifest.unsigned.json, an
  unsigned ZIP and SHA256 evidence. Never call unsigned bytes installable .fabapp,
  create a publisher key or grant native execution. Existing signed Registry
  approval/install remains separate. Native projects retain their existing
  repository build/review workflow; Bot must inspect it before editing.
- BA-05: Bot prompt completes create/edit/export -> discover GitHub tools ->
  verify chosen repo/base SHA -> create branch/push exact files -> open PR ->
  observe actual CI result. No force push, silent repository visibility change,
  automatic merge/review bypass, credential in source, or retry of uncertain
  writes. Missing GitHub grant/Box/runner is an explicit unresolved dependency.
- BA-06: real tool composition preserves Shell review and cancellation. A denied
  Shell request creates no project. Shared-room/subagent restrictions stay owned
  by existing toolset filters, never restored through this tool.
  Invoke Python in isolated mode so an untrusted workspace module or PYTHONPATH
  cannot replace the SDK's standard-library imports. Pin that negative control
  through the actual Actions Bot tool execution.

## Failure, verification and delivery

Single writer lock and immutable snapshots bound each project transaction. Disk
reserve is max(1GiB,5% capacity) before writes. Source limits:64files,128KiB/file,
512KiB total across src/ and tests/; package assets currently UTF-8 web resources.
Only src/ enters the installed resources; tests/ stays developer source.
Unknown native/network
capabilities fail closed; existing native repositories use their own exact-source
pipeline instead of being converted into a web scaffold.

Actions-only tests must exercise actual SDK creation, optimistic edits, unchanged
history, denied/conflicting edits, export and generated-project packaging; actual
Runner tool exposure/review forwarding; and a disposable GitHub branch roundtrip
of exported source with exact remote contents/digests and cleanup. This last test
uses only the job's repo-scoped token, never production user credentials. Separate
real-model installed Bot acceptance is required before claiming live end-to-end
completion. Preserve current Mini App lifecycle/permission regressions.
The opt-in signed macOS gate asks the production Bot to create/edit/export and
copy the immutable JSON through CopyFromBox, then use its already-authorized
GitHub MCP to open a disposable draft PR. Independently check source digests,
counter behavior, exact remote blobs, candidate ancestry and draft PR ownership.
The verifier never creates or pushes source in place of the Bot. Only after
verification does it close that exact draft and remove its unchanged branch.
Missing Box, file transfer or GitHub grant fails the gate; job tokens are used
only by the independent verifier, never supplied to the Bot or its generated code.

Roll out through reviewed source PR, exact-head CI, signed macOS package and Bot
acceptance. Rollback removes authoring exposure without deleting user projects,
GitHub commits or installed Mini Apps. No completion claim from a prompt or
scaffold alone. Track source, CI, GitHub, package and live-Bot proof separately.
