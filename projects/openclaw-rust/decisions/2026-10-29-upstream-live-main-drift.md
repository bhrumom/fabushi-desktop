# Upstream live-main drift and source-baseline decision

Status: active decision record; does not accept source closure  
Date: 2026-10-29 (Asia/Shanghai)  
Project: OCRM-001

## Observed authoritative state

The repository lock currently pins:

- `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`
- tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`

The live upstream default branch currently resolves to:

- commit `70bf5121704c7a7173722c2bd7e360362e5da165`
- tree `3732f45aa4d3a924ac88b4b684f21fbab13b3c79`

GitHub compare from the locked commit reports the live branch ahead by 4,851 commits. This is source drift, not a harmless metadata update. The compare response did not include a usable file list for this very large diff, so it is not evidence that files are unchanged. Direct tree inspection at the current live tree records these node-host blobs: `crates/openclaw-node-host/src/node.rs` `f2ea70719e114f4201b245ba8c9b89710f38dc12`, `crates/openclaw-node-host/src/runtime.rs` `d29d1bd468436cdc7c636427b3ec2e07aa9624e4`, `src/node-host/config-shared.ts` `3844b92c31a174df10721b1a299a63e2e78744fc`, `src/node-host/connection.ts` `da661abb04c1541fcc58b53a9d25d2aa5d565981`, `src/node-host/invoke.ts` `dd97dd4eed05eb6878bed0024f1a6c9a8b1c56dc`, and `src/node-host/runtime-manifest.ts` `856e0f728d07421741a8d3f16aade54b12e284cb`. These live identities must be compared to the locked source during an explicit rebaseline; they do not rewrite the current inventory.

## Decision

1. Do not silently rewrite `upstream.lock.json`, inventory shards, parity ledger, or prior dossiers to live main.
2. Treat all existing source reads and receipts as bound to the locked commit/tree only.
3. Continue reading and mapping the locked implementation round where its source identity is explicit, while separately auditing changed live-main files before any final rebaseline.
4. Keep `baseline_identity_accepted`, `source_closure_ready`, and all downstream completion flags false until a GitHub Actions authority job enumerates and validates the selected exact source/dependency closure.
5. Before final OCRM acceptance, choose one explicit current baseline (the locked implementation round or the live default branch), update the lock and regenerate inventory/RTM/ledger provenance atomically through the required Actions authority path, then repeat any affected source/owner/oracle review.

## Consequences

The node-host dossier remains valid only as pinned-source evidence. It cannot be cited as proof of live-main parity. The current inventory remains intentionally unread/unclassified for the audited files because the canonical rows are still bound to the locked source and no exact-head authority artifact has accepted the new live tree. This is a controlled rebaseline boundary, not a global blocker: unchanged pinned responsibilities and independent source audits may continue.

## References

- `projects/openclaw-rust/upstream.lock.json`
- `projects/openclaw-rust/inventory/index.json`
- `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md`
- live GitHub refs queried on 2026-10-29: `openclaw/openclaw@70bf5121704c7a7173722c2bd7e360362e5da165`, tree `3732f45aa4d3a924ac88b4b684f21fbab13b3c79`
- prior live-main observation superseded by this refresh: `bf6a97a1761ed7f752544f4934aa708b9d896bd3`, tree `3fa25384ab9647485cf8e4293121bbe87e1548f4`
