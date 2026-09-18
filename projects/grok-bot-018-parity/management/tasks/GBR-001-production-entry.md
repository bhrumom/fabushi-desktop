# GBR-001 — Production parity entry

Status: in-progress

## Deliverable

Replace the current layered Fabushi desktop entry (Messenger + parity patch + Mahayana workbench + credential vault + MiniApp bridges) with a single reference-aligned production shell. Keep legacy modules unreachable in production during migration.

## Acceptance

- main.tsx mounts one parity application root.
- no contacts/groups/Telegram/payment/MiniApp/Mahayana workbench/credential-vault surface is mounted by the production entry.
- production source declares Grok reference commit and intentional local-computer adaptation.
- build remains packageable on macOS.

## Evidence

Pending implementation commit and CI.
