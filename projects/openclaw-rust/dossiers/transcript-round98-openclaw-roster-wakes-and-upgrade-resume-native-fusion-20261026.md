# OCRM Technical Dossier: Round 98 — OpenClaw Roster Search, Background Wakes & Upgrade Recreate Resume Native Fusion

## 1. Executive Summary

This dossier records the full-source native Rust equivalence and fusion for:
1. **`OCRM-M20`**: Roster Content Search, Message & Media Indexing, Snippet Framing & Result Bounds (`source/host/src/extensions/transcript/roster_search.rs`, `source/host/src/extensions/content_search/`)
2. **`OCRM-M21`**: Background Wakes, Inbound/Outbound Channel Cue Formatting & Delivery Failure Redrive (`source/host/src/extensions/transcript/background_wakes.rs`)
3. **`OCRM-M22`**: Upgrade Recreate Resume, Atomic Quiesce Lifecycle & Source-Specific Resume Prompts (`source/host/src/extensions/transcript/upgrade_recreate_resume.rs`)

All upstream capabilities from `openclaw/openclaw` across search indexing, wake formatting, and quiesce/resume lifecycles are natively realized in canonical Fabushi Host Rust architecture under `source/host/src/extensions/transcript/`.

---

## 2. Upstream Source to Canonical Fabushi Rust Mapping

### 2.1 Roster & Content Search (`OCRM-M20`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/memory/search.ts` | Transcript message and attachment content search | `search_agents`, `search_media`, `RosterContentSearch` | Host Transcript Extension |
| `src/memory/snippets.ts` | Sliding window snippet builder with whitespace normalization | `build_content_snippet` | Host Transcript Extension |
| `src/memory/limits.ts` | Per-agent match caps & total search result limits | `AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT` (5), `AGENT_CONTENT_SEARCH_MAX_RESULTS` (50) | Host Transcript Extension |
| `src/memory/filters.ts` | Hidden outbound agent peer message filtering | `find_agent_content_matches` (`hiddenOutboundAgentPeerMessage` fence) | Host Transcript Extension |

### 2.2 Background Wakes & External Channels (`OCRM-M21`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/channels/wakes.ts` | Wake cues for timeline events, inbounds, and delivery failures | `TIMELINE_EVENT_WAKE_CUE`, `CHANNEL_INBOUND_WAKE_CUE`, `CHANNEL_DELIVERY_FAILED_WAKE_CUE` | Host Transcript Extension |
| `src/channels/address.ts` | Address formatting and normalization across strings & objects | `format_channel_address_value`, `distinct_inbound_channel_addresses` | Host Transcript Extension |
| `src/channels/prompts.ts` | Prompt framing for incoming messages & emoji reactions | `build_channel_inbound_wake_prompt` | Host Transcript Extension |
| `src/channels/failures.ts` | Humanized error messages and delivery failure disclosure | `humanize_channel_delivery_failure`, `build_channel_delivery_failure_wake_prompt` | Host Transcript Extension |

### 2.3 Upgrade Recreate Resume Lifecycle (`OCRM-M22`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/snapshot/quiesce.ts` | Atomic quiesce flag and running turns accounting | `UpgradeRecreateResume::quiesce_for_upgrade` | Host Transcript Extension |
| `src/snapshot/status.ts` | Quiescing state inspection across threads | `UpgradeRecreateResume::is_quiescing_for_upgrade` | Host Transcript Extension |
| `src/snapshot/resume.ts` | Post-upgrade resume toggle and turn continuation | `UpgradeRecreateResume::resume_after_recreate` | Host Transcript Extension |
| `src/snapshot/prompt.ts` | Source-specific prompt framing (turn, automation, background) | `build_upgrade_resume_prompt` | Host Transcript Extension |

---

## 3. Falsifiable Test Oracles

1. **`OCRM-ROSTER-O01` (Search Constant Bounds)**: Validates `AGENT_CONTENT_SEARCH_MAX_MATCHES_PER_AGENT == 5` and `AGENT_CONTENT_SEARCH_MAX_RESULTS == 50`.
2. **`OCRM-ROSTER-O02` (Serialization Integrity)**: Validates camelCase serialization of `TranscriptSearchResult` and `RosterMediaSearchResult`.
3. **`OCRM-ROSTER-O03` (Content Snippet Builder)**: Validates whitespace flattening, centering of keyword, and ellipsis boundaries.
4. **`OCRM-ROSTER-O04` (Matches & Hidden Filter)**: Validates reverse chronological scan and filtering of `hiddenOutboundAgentPeerMessage`.
5. **`OCRM-WAKE-O01` (Wake Cue Constants)**: Validates string values of `[event]`, `[inbound]`, and `[channel-delivery-failed]`.
6. **`OCRM-WAKE-O02` (Channel Address Normalization)**: Validates string and structured JSON object address parsing.
7. **`OCRM-WAKE-O03` (Distinct Addresses)**: Validates deduplication and ordering of inbound addresses.
8. **`OCRM-WAKE-O04` (Inbound Wake Prompt)**: Validates message/reaction pluralization and external origin notice.
9. **`OCRM-WAKE-O05` (Humanized Delivery Failure Copy)**: Validates error translation across Slack, Discord, and invalid addresses.
10. **`OCRM-WAKE-O06` (Delivery Failure Wake Prompt)**: Validates in-app disclosure instructions without silent retry.
11. **`OCRM-UPGRADE-O01` (Atomic Quiesce Lifecycle)**: Validates `quiesce_for_upgrade`, `is_quiescing_for_upgrade`, and `resume_after_recreate`.
12. **`OCRM-UPGRADE-O02` (Resume Prompt Semantics)**: Validates silence semantics for automation and background revival sources.
