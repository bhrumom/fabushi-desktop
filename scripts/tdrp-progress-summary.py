#!/usr/bin/env python3
"""Generate the FMEP-REPORT-01 progress overview table from the TDRP ledger.

Read-only. Usage:
    python3 scripts/tdrp-progress-summary.py [path/to/parity-ledger.json]

Prints a Markdown table to stdout for the top of both STATUS files. Categories
follow FMEP-001 Revision 3 §14; counts are derived, never hand-edited. Running
this locally is FMEP-FAST-09 local-feedback reporting, not validation evidence.
"""
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

DEFAULT = Path(__file__).resolve().parent.parent / "projects/telegram-desktop-rust/parity-ledger.json"
STATUSES = ["unreviewed", "understood", "mapped", "implemented", "verified"]
CATEGORY_LABEL = {
    "product": "产品职责（Fabushi owner）",
    "domain-only": "领域库实现、桌面未接线（FMEP-STATE-02）",
    "replaced": "旧栈职责→Fabushi 替代方案（E4，仅设计级 mapped）",
    "legacy": "旧 Telegram 栈（FMEP-EXIT-01，不计产品）",
    "governance": "构建/仓库治理支撑",
    "tool": "上游开发工具（FMEP-TOOL-01，已处置）",
}


def strip_repo(p: str) -> str:
    return p.split(":", 1)[1] if p.startswith("bhrumom/") and ":" in p else p


def is_gov(p: str) -> bool:
    return (
        p.endswith(".md")
        or p.startswith(("projects/", "docs/", "AGENTS", "scripts/", ".github/"))
        or p.endswith("parity-ledger.json")
    )


def is_legacy(p: str) -> bool:
    return "providers/telegram-" in p or "mahayana-telegram" in p or p.startswith("native/telegram-")


def category(row) -> str:
    notes = row.get("notes") or ""
    if "[FMEP-TOOL-01" in notes:
        return "tool"
    if "[FMEP-EXIT-01 E4 remapped]" in notes:
        return "replaced"
    if "[FMEP-EXIT-01" in notes:
        return "legacy"
    if "[FMEP-STATE-02" in notes:
        return "domain-only"
    paths = [strip_repo(p) for p in row.get("fabushi_target_paths") or []]
    product = [p for p in paths if not is_gov(p)]
    if not product:
        return "governance"
    if all(is_legacy(p) for p in product):
        return "legacy"
    return "product"


def main() -> int:
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT
    ledger = json.loads(path.read_text(encoding="utf-8"))
    rows = ledger["rows"]
    table = defaultdict(Counter)
    gates = defaultdict(Counter)
    for row in rows:
        cat = category(row)
        table[cat][row["implementation_status"]] += 1
        gates[cat][row.get("release_gate_status")] += 1

    cov = ledger.get("coverage", {})
    reviews = Counter(r.get("status") for r in ledger.get("source_reviews", []))
    print(f"_自动生成：`scripts/tdrp-progress-summary.py`，ledger U=`{ledger.get('upstream_commit', '')[:12]}`_")
    print()
    print("| 类别 | 行数 | understood | mapped | implemented | verified | release gate passed |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for cat in ["product", "domain-only", "replaced", "legacy", "governance", "tool"]:
        c = table.get(cat, Counter())
        total = sum(c.values())
        print(
            f"| {CATEGORY_LABEL[cat]} | {total} | {c['understood']} | {c['mapped']} | "
            f"{c['implemented']} | {c['verified']} | {gates[cat]['passed']} |"
        )
    print(f"| **合计** | {len(rows)} | | | | | |")
    print()
    def shown(value):
        return "未统计" if value is None else value

    print(
        f"来源身份 {cov.get('source_entries_total')}；unknown={shown(cov.get('unknown'))}，"
        f"unread={shown(cov.get('unread'))}，omitted={shown(cov.get('omitted'))}；"
        f"未映射职责 {cov.get('unmapped_responsibilities')}；"
        f"source_reviews {sum(reviews.values())}（complete {reviews.get('complete', 0)} / pending {reviews.get('pending', 0)}）。"
    )
    print("分母（全部适用产品职责）尚未完整分解，按 FMEP-STATE-01 不报告总百分比。")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
