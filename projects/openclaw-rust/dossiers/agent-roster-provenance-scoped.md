# OCRM Technical Dossier — Unit 15: Agent Roster, Provenance & Workspace Transitions (`OCRM-UNIT-ROSTER-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M08 (Agent Roster, Configuration Provenance, Workspace Transitions & Security Audit)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The locked-source audit also recorded the complete 49-line
`src/security/audit-agent-roster.ts` file (blob
`97e144954c8138c45af11ef859975217c2f2cdda`) in the canonical inventory,
mapped to `OCRM-RESP-AGENT-ROSTER-AUDIT-001`. This records source behavior and
the existing Rust owner candidate only; it does not upgrade runtime wiring or
verification.

The following 6 core agent roster representation, include provenance, workspace transition, implicit defaults, security audit, and doctor staging files (totaling 572 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shards:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0021` | `src/agents/agent-roster.ts` | `89b4c78e017ee7715ddef2052bf6ace0863fd209` | 173 | 1..173 (full-file) | `OCRM-RESP-AGENT-ROSTER-REPRESENTATION-001` |
| `part-0031` | `src/config/agent-roster-provenance.ts` | `2ada1a408e176df45e8571b597777e85cb049227` | 114 | 1..114 (full-file) | `OCRM-RESP-AGENT-ROSTER-PROVENANCE-001` |
| `part-0031` | `src/config/agent-workspace-roster-transition.ts` | `e8e480275ad9d5dbf302e8acd9908e625c0df43f` | 101 | 1..101 (full-file) | `OCRM-RESP-AGENT-WORKSPACE-TRANSITION-001` |
| `part-0031` | `src/config/implicit-agent-roster.ts` | `7037173dd7e41fd8abe29b111c2bfeb27fa37f20` | 19 | 1..19 (full-file) | `OCRM-RESP-AGENT-IMPLICIT-ROSTER-001` |
| `part-0045` | `src/security/audit-agent-roster.ts` | `97e144954c8138c45af11ef859975217c2f2cdda` | 49 | 1..49 (full-file) | `OCRM-RESP-AGENT-ROSTER-AUDIT-001` |
| `part-0030` | `src/commands/doctor/shared/roster-include-write.ts` | `985beab31531c6aeaf49aa9c7465a3de23d4c497` | 116 | 1..116 (full-file) | `OCRM-RESP-DOCTOR-ROSTER-INCLUDE-WRITE-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-AGENT-ROSTER-REPRESENTATION-001`**:
   - **Behavior**: Agent roster representation decoding (`entries` map vs `list` array), normalization, sole-agent vs marked default resolution, ACP subagent target aggregation, and Gateway system agent enumeration.
   - **Rust Owner**: `source/host/src/agents/agent_roster.rs::list_agent_ids, try_resolve_default_agent_id`.
   - **Production Call Chain**: Producer `HostRosterBookkeeping::apply_config` → Owner `try_resolve_default_agent_id` → Consumer `HostRosterBookkeeping::apply`.
   - **Oracles**: `OCRM-ROSTER-O01`, `OCRM-ROSTER-O02`, `OCRM-ROSTER-O03`, `OCRM-ROSTER-O09`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-AGENT-ROSTER-UPSTREAM-CALLER-001"]`: Host agent workspace configuration & doctor inspection pipelines do not yet invoke roster collapse, provenance, audit, and doctor include inspection helpers in real production flows).

2. **`OCRM-RESP-AGENT-ROSTER-PROVENANCE-001`**:
   - **Behavior**: Roster include provenance evaluation, determining whether `$include` contributes membership or default metadata at `agents.entries`, `agents.list`, `id`, `default`, or `bindings` boundaries.
   - **Rust Owner**: `source/host/src/agents/agent_roster.rs::config_include_owns_agent_roster_values`.
   - **Production Call Chain**: Producer `can_write_doctor_include` → Owner `include_contribution_owns_agent_roster` → Consumer `can_write_doctor_include`.
   - **Oracle**: `OCRM-ROSTER-O04`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-AGENT-ROSTER-UPSTREAM-CALLER-001"]`: Host agent workspace configuration & doctor inspection pipelines do not yet invoke roster collapse, provenance, audit, and doctor include inspection helpers in real production flows).

3. **`OCRM-RESP-AGENT-WORKSPACE-TRANSITION-001`**:
   - **Behavior**: Workspace transition and survivor workspace pinning on multi-agent to sole-agent collapse, preserving agent workspace isolation across roster mutations.
   - **Rust Owner**: `source/host/src/agents/agent_roster.rs::pin_survivor_workspace_for_roster_collapse`.
   - **Production Call Chain**: Producer `pin_survivor_workspace_for_roster_collapse` → Consumer `source/host/src/agents/settings_file.rs::write_sand_settings_file`.
   - **Oracle**: `OCRM-ROSTER-O07`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-AGENT-ROSTER-UPSTREAM-CALLER-001"]`: Host agent workspace configuration & doctor inspection pipelines do not yet invoke roster collapse, provenance, audit, and doctor include inspection helpers in real production flows).

4. **`OCRM-RESP-AGENT-IMPLICIT-ROSTER-001`**:
   - **Behavior**: Materializes default implicit agent roster (`{ entries: { main: {} } }`) on fresh unauthored configurations.
   - **Rust Owner**: `source/host/src/agents/agent_roster.rs::apply_implicit_agent_roster_defaults`.
   - **Production Call Chain**: Producer `HostRosterBookkeeping::apply_config` → Owner `apply_implicit_agent_roster_defaults` → Consumer `list_agent_ids`.
   - **Oracle**: `OCRM-ROSTER-O05`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-AGENT-ROSTER-UPSTREAM-CALLER-001"]`: Host agent workspace configuration & doctor inspection pipelines do not yet invoke roster collapse, provenance, audit, and doctor include inspection helpers in real production flows).

5. **`OCRM-RESP-AGENT-ROSTER-AUDIT-001`**:
   - **Behavior**: Security audit rule `config.agent_roster.invalid_default_count`, detecting invalid default selection counts in explicit vs compatibility fleets.
   - **Rust Owner**: `source/host/src/agents/agent_roster.rs::collect_agent_roster_findings`.
   - **Production Call Chain**: Producer `collect_agent_roster_findings` → Consumer `security_diagnostics`.
   - **Oracle**: `OCRM-ROSTER-O06`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-AGENT-ROSTER-UPSTREAM-CALLER-001"]`: Host agent workspace configuration & doctor inspection pipelines do not yet invoke roster collapse, provenance, audit, and doctor include inspection helpers in real production flows).

6. **`OCRM-RESP-DOCTOR-ROSTER-INCLUDE-WRITE-001`**:
   - **Behavior**: Doctor guarded config write staging and include boundary determination, protecting include-owned rosters from destructive in-place writes.
   - **Rust Owner**: `source/host/src/agents/agent_roster.rs::can_write_doctor_include`.
   - **Production Call Chain**: Producer `can_write_doctor_include` → Consumer `source/host/src/agents/settings_file.rs::write_sand_settings_file`.
   - **Oracle**: `OCRM-ROSTER-O08`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-AGENT-ROSTER-UPSTREAM-CALLER-001"]`: Host agent workspace configuration & doctor inspection pipelines do not yet invoke roster collapse, provenance, audit, and doctor include inspection helpers in real production flows).

---

## 3. Falsifiable Test Oracles (`source/host/tests/openclaw_agent_roster_contract.rs`)

1. **`OCRM-ROSTER-O01`**: `test_ocrm_roster_o01_entries_and_list_representations` (verifies `entries` map and `list` array representation, and unauthored implicit `"main"` fallback).
2. **`OCRM-ROSTER-O02`**: `test_ocrm_roster_o02_default_and_sole_resolution` (verifies sole agent default, explicit fleet exclusion, and legacy single default marker).
3. **`OCRM-ROSTER-O03`**: `test_ocrm_roster_o03_acp_and_gateway_subagent_targets` (verifies ACP subagent target aggregation and system agent IDs `system`, `gateway`).
4. **`OCRM-ROSTER-O04`**: `test_ocrm_roster_o04_include_provenance` (verifies include boundary identification at `agents.entries`, `list`, `default`, and `bindings`).
5. **`OCRM-ROSTER-O05`**: `test_ocrm_roster_o05_implicit_defaults_materialization` (verifies implicit defaults injection without mutating explicit fleets).
6. **`OCRM-ROSTER-O06`**: `test_ocrm_roster_o06_security_audit_invalid_default_count` (verifies detection of multiple default markers).
7. **`OCRM-ROSTER-O07`**: `test_ocrm_roster_o07_survivor_workspace_pinning` (verifies survivor workspace directory pinning on multi-agent collapse).
8. **`OCRM-ROSTER-O08`**: `test_ocrm_roster_o08_doctor_include_write_staging` (verifies doctor write rejection on include-owned rosters).
9. **`OCRM-ROSTER-O09`**: `test_ocrm_roster_o09_bookkeeping_apply_config_wiring` (verifies `HostRosterBookkeeping::apply_config` applying active agent fallback).

---

## 4. Invariant and Metric Ledger

- **Inventory Tracking**:
  - `mapped_paths`: 148
  - `unclassified_paths`: 58,602
  - `tracked_paths`: 58,750 (strictly conserved: $148 + 58,602 = 58,750$)
  - `omitted_known`: 0
- **Responsibilities**:
  - Total: 90
  - Active blockers: `BL-CURSOR-BACKEND-IMAGE-RESOLUTION-001` (Cursor AiService proto schema lacks resolution parameter), `BL-AGENT-ROSTER-UPSTREAM-CALLER-001` (Host agent workspace configuration & doctor inspection pipelines do not yet invoke roster collapse, provenance, audit, and doctor include inspection helpers in real production flows).
- **Global Completion Flags**:
  - All 8 flags remain `false`.

## 5. Exact-head production reachability re-audit (2026-10-29)

The exact Fabushi target under review is
`2225400ea73392523586a110301edeb87c986001`. A bounded source search of the
production Host confirms that `agent_roster.rs` is currently a library/test
surface, not a connected configuration owner:

- `HostRosterBookkeeping::apply_config` has no production call site;
- `HostRosterBookkeeping` and `HostRosterPorts` are constructed only by
  contract tests;
- `apply_implicit_agent_roster_defaults`,
  `collect_agent_roster_findings`, `can_write_doctor_include`, and
  `pin_survivor_workspace_for_roster_collapse` have no production consumers;
- the production `SettingsService` owns Fabushi `settings.json` fields, but
  does not own or expose an OpenClaw `agents.entries`/`agents.list` resolved
  configuration document.

The dossier's earlier “Production Call Chain” entries therefore remain
planned ownership contracts, not evidence of production wiring. No ledger
status is upgraded. The blocker `BL-AGENT-ROSTER-UPSTREAM-CALLER-001` remains
valid until a spec-approved configuration/doctor owner supplies the resolved
document and invokes the helpers through the existing Host lifecycle.
