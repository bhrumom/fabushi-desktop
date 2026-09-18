# GBR-002 — TypeScript local agent runtime

Status: baseline-implemented / deeper-reference-parity-pending

## Actual result
The production path is now:

renderer -> context-isolated preload -> Electron IPC -> persistent local agent runtime -> inference boundary.

It supports agent create/list/rename/delete, send/stop, status transitions, transcript persistence, and renderer events. It does not invoke Mahayana CLI.

The runtime explicitly targets the installed Mac rather than a provided cloud computer. Inference can be configured with an OpenAI-compatible endpoint through `FABUSHI_AGENT_API_URL`, `FABUSHI_AGENT_API_KEY`, and `FABUSHI_AGENT_MODEL`.

## Evidence
Implementation commit: 1bce9f070ce789c082e47c1edb07462588d0dbc3

## Remaining parity
The reference repository contains a substantially larger host/coordinator/extension graph. This baseline establishes the required non-CLI architecture but does not yet constitute line-by-line backend parity.
