# Upstream live-main drift and source-baseline decision

Status: active decision record; does not accept source closure  
Date: 2026-10-29 (Asia/Shanghai)  
Project: OCRM-001

## Observed authoritative state

The repository lock currently pins:

- `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`
- tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`

The live upstream default branch currently resolves to:

- commit `c0f1ba6ba4439a79d2cb91dde9666038fff7ad69`
- tree `f80fc84244c576760b0d36555d9b6849978b6baf`

GitHub compare from the locked commit reports the live branch ahead by 4,854 commits, with 300 files in the returned compare list. This is source drift, not a harmless metadata update. The current live tree must be queried directly for any newly audited file identities; prior live identities do not prove the current tree unchanged. These live identities must be compared to the locked source during an explicit rebaseline; they do not rewrite the current inventory.

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
- live GitHub refs queried on 2026-10-29: `openclaw/openclaw@c0f1ba6ba4439a79d2cb91dde9666038fff7ad69`, tree `f80fc84244c576760b0d36555d9b6849978b6baf`
- prior live-main observations superseded by this refresh: `29ebbe869cc4b4d085be6f7ef79f0edc3c204ec9`/`d2f7823097e89fb26dd27ad3ef6fe42d1451c8a2`, `70bf5121704c7a7173722cbd7e360362e5da165`/`3732f45aa4d3a924ac88b4b684f21fbab13b3c79`, and `bf6a97a1761ed7f752544f4934aa708b9d896bd3`/`3fa25384ab9647485cf8e4293121bbe87e1548f4`
