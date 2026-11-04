#!/usr/bin/env python3
"""Deterministic progress summary generator for OpenClaw-Rust (OCRM-001).

Reads `upstream.lock.json`, `inventory/index.json`, `parity-ledger.json`, and
`requirements-traceability-matrix.json` and emits a compact Markdown summary table,
including OCRM-MODULAR-01 modularity and anti-stub-compression metrics.
"""
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REPO_ROOT = ROOT.parent.parent


def get_module_line_count(target_file: Path) -> int:
    if not target_file.exists() or not target_file.is_file():
        return 0
    total = len(target_file.read_text(encoding="utf-8", errors="replace").splitlines())
    if target_file.name == "mod.rs":
        for sibling in target_file.parent.glob("*.rs"):
            if sibling != target_file:
                total += len(sibling.read_text(encoding="utf-8", errors="replace").splitlines())
    sub_dir = target_file.parent / target_file.stem
    if sub_dir.is_dir():
        for sub_file in sub_dir.glob("**/*.rs"):
            total += len(sub_file.read_text(encoding="utf-8", errors="replace").splitlines())
    return total


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

    # OCRM-MODULAR-01 metrics
    target_stats = {}
    for r in resps:
        target = r.get("target") or {}
        t_path = target.get("path")
        if not t_path:
            continue
        if t_path not in target_stats:
            target_stats[t_path] = {"source_refs": []}
        for s in r.get("source_refs", []):
            if isinstance(s, dict) and s.get("path"):
                target_stats[t_path]["source_refs"].append(s["path"])

    god_files = []
    compressed_stubs = []
    for t_path, stats in target_stats.items():
        if "/tests/" in t_path or t_path.startswith("tests/") or t_path.startswith("source/host/tests/"):
            continue
        t_file = REPO_ROOT / t_path
        if not t_file.exists() or not t_file.is_file():
            continue
        single_lines = len(t_file.read_text(encoding="utf-8", errors="replace").splitlines())
        module_lines = get_module_line_count(t_file)
        total_sources = len(stats["source_refs"])
        unique_sources = len(set(stats["source_refs"]))

        if single_lines > 1200:
            god_files.append((t_path, single_lines))
        if total_sources >= 15 and module_lines < 150:
            compressed_stubs.append((t_path, total_sources, unique_sources, module_lines))

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
    print(f"| **模块解耦红线** | `OCRM-MODULAR-01` 巨石文件 (`> 1,200` 行) | {len(god_files)} 处 | 待解耦目录化模块（Wave 1 目标降至 0） |")
    print(f"| **模块解耦红线** | `OCRM-MODULAR-01` 多合一空壳压缩 (`>= 15` 源映射但 `< 150` 行) | {len(compressed_stubs)} 处 | 待拆分子模块补齐状态机（Wave 1 目标降至 0） |")

    if god_files or compressed_stubs:
        print()
        print("#### OCRM-MODULAR-01 待整改清单（Wave 1 迭代清零）")
        if god_files:
            print("\n- **巨石文件（> 1,200 行，待目录化拆分）：**")
            for gf, lines in sorted(god_files, key=lambda x: x[1], reverse=True):
                print(f"  - `{gf}`: {lines:,} 行")
        if compressed_stubs:
            print("\n- **多合一空壳压缩（>= 15 上游映射且 < 150 行，待补齐子模块状态机）：**")
            for cs, total_s, uniq_s, lines in sorted(compressed_stubs, key=lambda x: x[1], reverse=True):
                print(f"  - `{cs}`: {total_s} 源引用 ({uniq_s} 独立路径) 映射入 {lines} 行")


if __name__ == "__main__":
    main()
