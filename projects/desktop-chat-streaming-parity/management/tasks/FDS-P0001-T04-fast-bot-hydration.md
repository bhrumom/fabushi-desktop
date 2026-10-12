# FDS-P0001-T04 — Fast account Bot hydration

Objective: load account Bot membership immediately after Host readiness rather than after self-hosted sync/reconciliation.

Implementation: lightweight account Bot and Mini App identity reads start as soon as the Host is ready; the full account sync remains the later authority/fallback.

Acceptance: installed account Bots such as 全球法布施 can appear without waiting for the slower self-hosted bootstrap.

Status: implemented; CI/integration pending.
