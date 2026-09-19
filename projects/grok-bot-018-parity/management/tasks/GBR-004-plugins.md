# GBR-004 — Grok-style plugins / MCP / OAuth / private skills / workflows

Status: **runtime complete / human UI acceptance pending**

## Objective

Recover the Grok-style plugin architecture so every visible install/provider path has an executable backend rather than placeholder catalog rows.

## Actual result

- Default local Marketplace works without an external provider.
- Custom MCP Server and Private Skill entries install executable MCP/SKILL.md payloads and uninstall cleanly.
- Optional remote Marketplace provider accepts real HTTP install metadata.
- MCP supports stdio and Streamable HTTP, initialize/tools/list/tools/call, server/tool configuration and synchronization.
- OAuth uses PKCE loopback with secure token storage; account/tool-toggle paths are executable.
- Private skills/workflows are file-backed, reloadable and injected into Agent behavior.
- Workflow publication is persistent/reversible; URL skill import and listener coordinator seams are executable.
- External catalog/subscription/account services fail closed when not configured.

## Verification

Final source gate Run `35417698506` at shipped SHA `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd` passed 103/103 tests, including Marketplace install/uninstall, default local Marketplace closure, MCP OAuth, remote HTTP MCP, workflow publication/listeners and SKILL.md persistence.

Visual Marketplace/OAuth/account interaction remains GBR-007.
