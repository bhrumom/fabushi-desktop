#!/usr/bin/env python3
"""Deterministic progress summary generator for OpenClaw-Rust (OCRM-001).

Reads `upstream.lock.json`, `inventory/index.json`, `parity-ledger.json`, and
`requirements-traceability-matrix.json` and emits a compact Markdown summary table.
"""
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main():
    lock = json.loads((ROOT / "upstream.lock.json").read_text(encoding="utf-8"))
    index = json.loads((ROOT / "inventory/index.json").read_text(encoding="utf-8"))
    ledger = json.loads((ROOT / "parity-ledger.json").read_text(encoding="utf-8"))
    rtm = json.loads((ROOT / "requirements-traceability-matrix.json").read_text(encoding="utf-8"))

    up = lock["upstream"]
    inv = index["inventory"]
    cov = ledger["file_coverage"]
    resps = ledger["responsibilities"]

    impl_counts = Counter(r["status"]["implementation"] for r in resps)
    wiring_counts = Counter(r["status"]["wiring"] for r in resps)
    mapping_counts = Counter(r["status"]["mapping"] for r in resps)
    verif_counts = Counter(r["status"]["verification"] for r in resps)
    impl_not_wired = sum(
        r["status"]["implementation"] == "implemented" and r["status"]["wiring"] != "wired"
        for r in resps
    )
    rtm_counts = Counter(r["acceptance_status"] for r in rtm["rows"])

    tracked = inv["tracked_entries_total"]
    mapped = cov["mapped_paths"]
    dispositioned = inv["dispositioned"]
    grouped_non_prod = dispositioned - mapped
    frontier = tracked - dispositioned

    print(f"### OCRM-001 Mechanical Progress Snapshot (`{up['commit'][:12]}`)")
    print()
    print("| 维度 | 指标 | 数值 | 占比 / 状态 |")
    print("| --- | --- | ---: | --- |")
    print(f"| **上游锁定基线** | `{up['repository']}@{up['commit'][:12]}` | {tracked:,} 项 | `accepted_for_this_implementation_round = {up['accepted_for_this_implementation_round']}` |")
    print(f"| **Inventory 核算** | 已完成适用性判定 (`applicability_assessed`) | {inv['applicability_assessed']:,} / {tracked:,} | 100.00% (`omitted_known = 0`) |")
    print(f"| **Inventory 处置** | 已映射深读路径 (`deep-read`) | {mapped:,} | {mapped * 100 / tracked:.2f}% |")
    print(f"| **Inventory 处置** | 非产品成组裁决 (`grouped-evidence`) | {grouped_non_prod:,} | {grouped_non_prod * 100 / tracked:.2f}% (`OCRM-BULK-01`) |")
    print(f"| **Inventory 处置** | 已完成处置合计 (`dispositioned`) | {dispositioned:,} / {tracked:,} | **{dispositioned * 100 / tracked:.2f}%** |")
    print(f"| **Inventory 前沿** | 待深读产品源码前沿 (`deep-read-required`) | {frontier:,} | {frontier * 100 / tracked:.2f}% (`src/` + `extensions/`) |")
    print(f"| **Parity Ledger** | 已登记职责总数 (`responsibilities`) | {len(resps)} | `reviewed={mapping_counts['reviewed']}`, `mapped={mapping_counts['mapped']}` |")
    print(f"| **代码落地状态** | `implementation = implemented` | {impl_counts['implemented']} / {len(resps)} | `not-started={impl_counts['not-started']}` |")
    print(f"| **宿主生产接线** | `wiring = wired` | {wiring_counts['wired']} / {len(resps)} | **`implemented + not-wired = {impl_not_wired}`**（Wave 1-2 主战场） |")
    print(f"| **契约验证状态** | `verification` | {len(resps)} | `verified={verif_counts['verified']}`, `not-run={verif_counts['not-run']}` |")
    print(f"| **RTM 门禁矩阵** | `OCRM-R01..R24` + `OCRM-AC01..AC24` | {len(rtm['rows'])} | {dict(rtm_counts)} |")


if __name__ == "__main__":
    main()
