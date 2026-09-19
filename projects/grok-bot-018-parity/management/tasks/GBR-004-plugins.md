# GBR-004 — Grok-style plugins / MCP / OAuth / private skills / workflows

Status: automated runtime complete / packaged human UI acceptance delegated to GBR-007

## Objective
Recover the Grok-style plugin architecture so every visible installable/provider path has an executable backend. Static placeholder catalog rows are not accepted.

## Actual result
- Default local Marketplace is executable without a remote catalog: Custom MCP Server and Private Skill entries materialize real backend state.
- Remote provider seam remains available for Grok-compatible Marketplace service deployment.
- MCP stdio and Streamable HTTP providers support initialize, tools/list and tools/call.
- Server add/remove/enable, custom instructions and per-tool enable/disable are persisted.
- OAuth supports metadata discovery, PKCE loopback callback, refresh, secure token storage and multi-account selection.
- Private skills are file-backed `SKILL.md` records and enter the Agent workflow rather than being display-only.
- Workflows support CRUD, enable-for-agent, publication/listener seams and invocation from the reference Agent runtime.
- Secure secret requests go to safeStorage and do not enter the model transcript.
- Fake GitHub/Memory placeholder rows remain removed.

## Verification
Final package source gate Run `35419150457` on `1f27c239aa8931b3266b9b83c84ff234332785d9` passed 105/105 runtime tests. Coverage includes real Marketplace install/uninstall materializing MCP + private `SKILL.md`, local default Marketplace install/uninstall, MCP persistence and executable-catalog filtering, OAuth/account/tool-toggle contracts, workflow publication/listener seams, and reference coordinator/desktop bridge closure.

## External dependency
The pinned reference's production Marketplace catalog service is not contained in the Git repository. Fabushi keeps an executable provider seam and functional local Marketplace instead of fabricating reference service contents.

## Acceptance
Automated backend/runtime scope: PASS. Packaged visual Marketplace/OAuth/workflow UX: GBR-007 human acceptance pending.
