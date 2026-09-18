# GBR-004 — Grok-style plugins

Status: functional-baseline / deeper-reference-parity-pending

## Actual result
The production renderer exposes a Grok-style Plugins overlay with browse, search, install, remove, enable, and disable operations. Plugin state is persisted in the desktop runtime.

Initial catalog: Files, Terminal, Browser, GitHub, Memory.

## Evidence
Implementation commit: 1bce9f070ce789c082e47c1edb07462588d0dbc3

## Remaining parity
Provider-specific OAuth/MCP setup, accounts, server-tool configuration, private skills, workflows, and full marketplace metadata from the reference are not yet reproduced.
