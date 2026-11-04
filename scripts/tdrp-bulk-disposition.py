#!/usr/bin/env python3
"""FMEP-BULK-01: Bulk subtree disposition generator for non-product dependencies.

Reads the fixed recursive inventory and outputs deterministic source_reviews
records for third-party gitlink submodules under contract/deep review depth
per FMEP-001 §15 and FMEP-SOURCE-01.

Usage:
    python3 scripts/tdrp-bulk-disposition.py [--inventory <path>] [--output <path>] [--summary] [--verify-schema]
"""

from __future__ import annotations

import argparse
from collections import defaultdict
import json
from pathlib import Path
import sys

DEFAULT_INVENTORY = Path("/Users/bhrum/Documents/ChatGPT/.tdrp-artifacts/13118749668/x/upstream-recursive-inventory.json")
DEFAULT_SCHEMA = Path(__file__).resolve().parent.parent / "projects/telegram-desktop-rust/contracts/parity-ledger.schema.json"
DEFAULT_EVIDENCE_ANCHOR = "docs/specs/fmep-bulk-subtrees-disposition.md#submodule-contract-disposition"

# High-risk Telegram proprietary submodules (status: pending, risk: high, review_depth: deep)
HIGH_RISK_PENDING = {
    "desktop-app/lib_storage": (
        "Telegram 专属加密存储格式与本地密钥派生，可能包含关键本地持久化产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，标记为 pending 并升级 deep review。"
    ),
    "TelegramMessenger/tgcalls": (
        "Telegram 专属音视频通话引擎与信令状态机，可能包含端到端 VoIP 通话关键语义；"
        "Fabushi 不构建、不链接、不分发此代码，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/lib_tl": (
        "Telegram 专属 Type Language (TL) schema 生成与序列化，可能包含通信协议结构产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，标记为 pending 并升级 deep review。"
    ),
}

# Medium-risk Telegram proprietary submodules (status: pending, risk: medium, review_depth: deep)
MEDIUM_RISK_PENDING = {
    "desktop-app/lib_ui": (
        "Telegram 专属 UI 交互与视图绘制层，可能包含聊天/消息 UI 行为产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/lib_base": (
        "Telegram 专属基础框架，可能包含核心基础设施产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/codegen": (
        "Telegram 专属代码生成工具，可能包含协议/样式生成产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/lib_crl": (
        "Telegram 专属异步调度库，可能包含并发产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/lib_webrtc": (
        "Telegram 专属 WebRTC 包装层，可能包含媒体传输产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/lib_spellcheck": (
        "Telegram 专属拼写检查封装，可能包含文本处理产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/lib_lottie": (
        "Telegram 专属 Lottie 动画渲染层，可能包含动态表情产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/lib_rpl": (
        "Telegram 专属响应式编程库，可能包含状态响应产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/lib_webview": (
        "Telegram 专属 WebView 容器封装，可能包含小程序/网页产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/libprisma": (
        "Telegram 专属主题解析层，可能包含主题颜色产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/lib_translate": (
        "Telegram 专属翻译提供层，可能包含翻译服务产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
    "desktop-app/lib_qr": (
        "Telegram 专属二维码展示层，可能包含登录扫码产品语义；"
        "Fabushi 不构建、不链接、不分发此代码，待评估产品语义，标记为 pending 并升级 deep review。"
    ),
}

# Generic third-party submodules with medium risk (security/crypto/auth, status: complete, risk: medium, review_depth: contract)
MEDIUM_RISK_COMPLETE = {
    "Yubico/libfido2",
    "PJK/libcbor",
}

# Generic third-party submodule with verified repository replacement
VERIFIED_REPLACEMENTS = {
    "desktop-app/cmark-gfm": (
        "Fabushi 不构建、不链接、不分发此第三方子模块（构建可达性：否；无运行时许可证分发义务）；"
        "Markdown 解析由 `pulldown-cmark`（`source/mahayana/mahayana-rs/Cargo.lock`）承接。"
    ),
}

GENERIC_THIRD_PARTY_FALLBACK = (
    "Fabushi 不构建、不链接、不分发此第三方子模块（构建可达性：否；无运行时许可证分发义务）。"
    "Fabushi 未提供等价实现；产品适用性另行评估。"
)

# License annotations for summary and reporting
SUBMODULE_LICENSES: dict[str, str] = {
    "ericniebler/range-v3": "BSL-1.0 (LICENSE.txt)",
    "hunspell/hunspell": "LGPL-2.1/GPL-2.0/MPL (COPYING*)",
    "hime-ime/hime": "GPL-2.0-or-later (COPYING*)",
    "desktop-app/lib_ui": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
    "flatpak/xdg-desktop-portal": "LGPL-2.1 (COPYING)",
    "KDE/kcoreaddons": "LGPL-2.0-or-later/等 (LICENSES/*)",
    "KDE/kimageformats": "LGPL-2.0-or-later/等 (LICENSES/*)",
    "hamonikr/nimf": "LGPL-3.0 (COPYING)",
    "TelegramMessenger/tgcalls": "GNU LGPL v3 (LICENSE)",
    "desktop-app/MicroTeX": "MIT (LICENSE)",
    "Yubico/libfido2": "BSD-2-Clause (LICENSE)",
    "desktop-app/lib_base": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
    "fcitx/fcitx5-qt": "LGPL-2.1-or-later/BSD-3-Clause (LICENSES/*)",
    "lz4/lz4": "BSD-2-Clause (lib/LICENSE) / GPL-2.0",
    "PJK/libcbor": "MIT (LICENSE.md)",
    "desktop-app/cmark-gfm": "BSD-2-Clause (COPYING)",
    "tzcnt/TooManyCooks": "BSL-1.0 (LICENSE)",
    "gitlab.com/mnauw/cppgir": "MIT (LICENSE)",
    "desktop-app/cmake_helpers": "unknown (无独立 LICENSE 文件)",
    "Cyan4973/xxHash": "BSD-2-Clause (LICENSE) / GPL-2.0",
    "google/cld3": "Apache-2.0 (LICENSE)",
    "desktop-app/codegen": "unknown (无独立 LICENSE 文件)",
    "nayuki/QR-Code-generator": "MIT (Readme.markdown 声明)",
    "desktop-app/lib_crl": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
    "Microsoft/GSL": "MIT (LICENSE)",
    "martinmoene/expected-lite": "BSL-1.0 (LICENSE.txt)",
    "desktop-app/lib_webrtc": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
    "desktop-app/lib_spellcheck": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
    "desktop-app/lib_lottie": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
    "desktop-app/lib_rpl": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
    "desktop-app/lib_webview": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
    "desktop-app/lib_storage": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
    "TartanLlama/expected": "CC0-1.0 (COPYING)",
    "desktop-app/libprisma": "MIT (LICENSE)",
    "tsyrogit/zxcvbn-c": "MIT (LICENSE.txt)",
    "desktop-app/lib_tl": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
    "nnen/doxygen-theme": "GPL-3.0 (LICENSE)",
    "desktop-app/lib_translate": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
    "desktop-app/lib_qr": "Telegram GPL-3.0 (代码仓无独立 LICENSE)",
}


def sanitize_row_id(repository: str) -> str:
    """Generate a valid responsibility ID from a repository slug."""
    clean = repository.replace("/", "-").replace(".", "-").replace("_", "-").upper()
    return f"TDRP-DEP-{clean}-001"


def get_submodule_metadata(repo: str) -> tuple[str, str, str, str]:
    """Return (status, risk, review_depth, rationale) for a given submodule."""
    if repo in HIGH_RISK_PENDING:
        return "pending", "high", "deep", HIGH_RISK_PENDING[repo]
    if repo in MEDIUM_RISK_PENDING:
        return "pending", "medium", "deep", MEDIUM_RISK_PENDING[repo]
    if repo in MEDIUM_RISK_COMPLETE:
        rationale = VERIFIED_REPLACEMENTS.get(repo, GENERIC_THIRD_PARTY_FALLBACK)
        return "complete", "medium", "contract", rationale
    # Generic third-party complete low-risk
    rationale = VERIFIED_REPLACEMENTS.get(repo, GENERIC_THIRD_PARTY_FALLBACK)
    return "complete", "low", "contract", rationale


def generate_bulk_disposition(
    inventory_entries: list[dict],
    evidence_anchor: str = DEFAULT_EVIDENCE_ANCHOR,
    scheme: str = "empty",
) -> list[dict]:
    """Generate deterministic source_reviews records for non-root submodule entries."""
    reviews = []
    for entry in inventory_entries:
        repo = entry["repository"]
        if repo == "telegramdesktop/tdesktop":
            continue

        status, risk, review_depth, rationale = get_submodule_metadata(repo)

        responsibility_ids = []
        if scheme == "row":
            responsibility_ids = [sanitize_row_id(repo)]

        review = {
            "repository": repo,
            "commit": entry["commit"],
            "path": entry["path"],
            "object": entry["object"],
            "category": "dependency",
            "risk": risk,
            "review_depth": review_depth,
            "status": status,
            "rationale": rationale,
            "evidence": [evidence_anchor],
            "responsibility_ids": responsibility_ids,
        }
        reviews.append(review)

    # Sort deterministically by (repository, path, commit, object)
    reviews.sort(key=lambda r: (r["repository"], r["path"], r["commit"], r["object"]))
    return reviews


def summarize_submodules(inventory_entries: list[dict]) -> str:
    """Produce a Markdown statistics table for non-root submodule repositories."""
    submodules: dict[str, dict] = defaultdict(lambda: {"blobs": 0, "commits": 0, "count": 0, "commit": None})
    for entry in inventory_entries:
        repo = entry["repository"]
        if repo == "telegramdesktop/tdesktop":
            continue
        submodules[repo]["count"] += 1
        if entry.get("type") == "commit":
            submodules[repo]["commits"] += 1
        else:
            submodules[repo]["blobs"] += 1
        if submodules[repo]["commit"] is None:
            submodules[repo]["commit"] = entry.get("commit")

    lines = [
        f"| 序号 | 子模块 (Repository) | 类别与风险 | 状态与深度 | 许可证类型 | Blob 数 | Commit 数 | 总对象数 |",
        f"| ---: | --- | --- | --- | --- | ---: | ---: | ---: |",
    ]
    total_blobs = 0
    total_commits = 0
    total_count = 0
    for idx, (repo, stats) in enumerate(sorted(submodules.items(), key=lambda x: -x[1]["count"]), 1):
        total_blobs += stats["blobs"]
        total_commits += stats["commits"]
        total_count += stats["count"]
        status, risk, review_depth, _ = get_submodule_metadata(repo)
        license_type = SUBMODULE_LICENSES.get(repo, "unknown")
        lines.append(
            f"| {idx} | `{repo}` | {risk} | {status} / {review_depth} | {license_type} | "
            f"{stats['blobs']} | {stats['commits']} | {stats['count']} |"
        )

    lines.append(
        f"| **合计** | **{len(submodules)} 个第三方子模块** | — | — | — | "
        f"**{total_blobs}** | **{total_commits}** | **{total_count}** |"
    )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", type=Path, default=DEFAULT_INVENTORY, help="Path to recursive inventory JSON")
    parser.add_argument("--output", type=Path, default=None, help="Output path for generated source_reviews JSON")
    parser.add_argument("--scheme", choices=["empty", "row"], default="empty", help="Mounting scheme (default: empty)")
    parser.add_argument("--evidence-anchor", default=DEFAULT_EVIDENCE_ANCHOR, help="Evidence anchor string")
    parser.add_argument("--summary", action="store_true", help="Print submodule summary table")
    parser.add_argument("--verify-schema", action="store_true", help="Validate output against parity-ledger.schema.json")
    args = parser.parse_args()

    if not args.inventory.is_file():
        print(f"Error: Inventory file not found: {args.inventory}", file=sys.stderr)
        return 1

    with args.inventory.open("r", encoding="utf-8") as f:
        inv = json.load(f)

    entries = inv.get("entries", [])
    if not entries:
        print("Error: No entries found in inventory", file=sys.stderr)
        return 1

    if args.summary:
        print(summarize_submodules(entries))
        if not args.output and not args.verify_schema:
            return 0

    reviews = generate_bulk_disposition(entries, evidence_anchor=args.evidence_anchor, scheme=args.scheme)
    complete_count = sum(1 for r in reviews if r["status"] == "complete")
    pending_count = sum(1 for r in reviews if r["status"] == "pending")
    print(
        f"Generated {len(reviews)} bulk disposition source_reviews across submodules "
        f"(complete={complete_count}, pending={pending_count}, scheme={args.scheme})."
    )

    if args.verify_schema:
        try:
            from jsonschema import Draft202012Validator
        except ImportError:
            print("Error: jsonschema package is required for --verify-schema", file=sys.stderr)
            return 1

        if not DEFAULT_SCHEMA.is_file():
            print(f"Error: Schema file not found: {DEFAULT_SCHEMA}", file=sys.stderr)
            return 1

        schema_raw = json.loads(DEFAULT_SCHEMA.read_text(encoding="utf-8"))
        validator = Draft202012Validator({
            "$schema": schema_raw["$schema"],
            "$defs": schema_raw["$defs"],
            "allOf": [schema_raw["properties"]["source_reviews"]],
        })
        errors = list(validator.iter_errors(reviews))
        if errors:
            print(f"Schema verification failed with {len(errors)} error(s):", file=sys.stderr)
            for err in errors[:10]:
                print(f"  {err.json_path}: {err.message}", file=sys.stderr)
            return 1
        print("Schema verification passed (0 errors against Draft202012Validator).")

    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with args.output.open("w", encoding="utf-8") as f:
            json.dump(reviews, f, indent=2, ensure_ascii=False)
            f.write("\n")
        print(f"Wrote deterministic output to {args.output}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
