# Telegram Capability Research / Provider — Status

Date: 2026-10-02  
Spec: TDRP-001 revision 3  
Parent: FBCP-001  
Complete: false

| Area | Status | Note |
| --- | --- | --- |
| Role under FBCP | specified | Telegram is provider/capability source, not product root |
| Fixed upstream baseline | recorded | 33261535a0e747f125e0ed25486f01e556330677 |
| Recursive source closure | blocked | P0 pending |
| Telegram capability graph | blocked | P0 pending |
| C++ production logic inventory | blocked | P0 pending |
| FBCP destination mapping | blocked | P0 pending |
| Research dossiers | blocked | P0 pending |
| Telegram Provider | blocked | not implemented |
| C++ Rust replacement | blocked | not started |
| Provider interoperability | blocked | no evidence |
| Packaged FBCP acceptance | blocked | no artifact |

## Completion metrics

Report separately:

- research coverage
- provider design
- C++ Rust replacement
- Telegram behavior verified
- FBCP production wiring
- packaged provider acceptance

Do not report this project as a standalone Telegram product completion percentage.

## First vertical slice

Fabushi shell → Telegram sign-in → provider sync → unified Inbox → private human chat → explicit Ask Agent via FBCP InteractionGateway → Agent result → controlled publish back → restart recovery.
