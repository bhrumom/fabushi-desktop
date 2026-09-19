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
Current immutable package source gate Run `35425432746` on `98ac56f037cb1b0eaa66781087da986334754908` passed **109/109 runtime tests, 0 fail**, exact pinned renderer/source parity, pinned reference runner builds, CommonJS syntax, pinned reference `source:typecheck`, and the production renderer build. PR source gate Run `35425434873` also passed on the same product SHA.

Coverage includes real Marketplace install/uninstall materializing MCP + private `SKILL.md`, local default Marketplace install/uninstall, MCP persistence and executable-catalog filtering, OAuth/account/tool-toggle contracts, workflow publication/listener seams, reference coordinator/desktop bridge closure, exact AnysphereAgent model/tool continuation, and local Computer/Browser execution.

macOS delivery Run `35425432753` packaged and published version `2.0.0-alpha.4` as prerelease `grok-parity-mac-126`; artifact `10579089056` has digest `sha256:6c91562e2d8ede5641c6c4bb9010a1cd3ebad1c0d0744a5cb8cc48b9bd1850d1`.

## External dependency
The pinned reference's production Marketplace catalog service is not contained in the Git repository. Fabushi keeps an executable provider seam and functional local Marketplace instead of fabricating reference service contents.

## Acceptance
Automated backend/runtime scope: PASS. Packaged visual Marketplace/OAuth/workflow UX: GBR-007 human acceptance pending. The hosted packaged renderer probe produced no UI report and recorded `manualValidationRequired:true`, so no automated packaged-UI PASS is claimed.
