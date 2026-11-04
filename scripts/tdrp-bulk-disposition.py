#!/usr/bin/env python3
"""FMEP-BULK-01: Bulk subtree disposition generator for non-product dependencies.

Reads the fixed recursive inventory and outputs deterministic source_reviews
records for third-party gitlink submodules under contract review depth.

Usage:
    python3 scripts/tdrp-bulk-disposition.py [--inventory <path>] [--output <path>] [--summary] [--verify-schema]
"""

from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import sys

DEFAULT_INVENTORY = Path("/Users/bhrum/Documents/ChatGPT/.tdrp-artifacts/13118749668/x/upstream-recursive-inventory.json")
DEFAULT_SCHEMA = Path(__file__).resolve().parent.parent / "projects/telegram-desktop-rust/contracts/parity-ledger.schema.json"
DEFAULT_EVIDENCE_ANCHOR = "docs/specs/fmep-bulk-subtrees-disposition.md#submodule-contract-disposition"

# Semantic rationales mapped by submodule repository
SUBMODULE_RATIONALES: dict[str, str] = {
    "ericniebler/range-v3": "C++ ranges library replaced by Rust standard iterator and itertools idioms.",
    "hunspell/hunspell": "C++ spellchecker replaced by Chromium native spellcheck and Web platform APIs.",
    "hime-ime/hime": "Linux input method framework binding replaced by Chromium native IME input pipeline.",
    "desktop-app/lib_ui": "Qt desktop UI library replaced by React/TypeScript desktop renderer and design system.",
    "flatpak/xdg-desktop-portal": "Desktop portal client bindings replaced by Electron platform integration APIs.",
    "KDE/kcoreaddons": "KDE platform utilities replaced by Electron native integration and Rust std filesystem APIs.",
    "KDE/kimageformats": "Image plugin codecs replaced by Chromium native image decoders (WebP, AVIF, PNG, JPEG).",
    "hamonikr/nimf": "Korean input method framework replaced by Chromium/Electron IME integration.",
    "TelegramMessenger/tgcalls": "C++ WebRTC calling engine replaced by Chromium WebRTC media stack and native audio pipeline.",
    "desktop-app/MicroTeX": "LaTeX rendering library replaced by Web KaTeX and MathJax DOM rendering.",
    "Yubico/libfido2": "Hardware token FIDO2/WebAuthn library replaced by Chromium WebAuthn platform APIs.",
    "desktop-app/lib_base": "Qt base utility framework replaced by Rust standard library and Electron runtime.",
    "fcitx/fcitx5-qt": "Fcitx5 Qt input method frontend replaced by Chromium native IME bridge.",
    "lz4/lz4": "LZ4 compression implementation replaced by Rust lz4_flex crate.",
    "PJK/libcbor": "CBOR encoder/decoder replaced by Rust serde_cbor/ciborium crates.",
    "desktop-app/cmark-gfm": "GitHub Flavored Markdown parser replaced by Rust pulldown-cmark crate.",
    "tzcnt/TooManyCooks": "C++ build helper utility replaced by standard Cargo build orchestration.",
    "gitlab.com/mnauw/cppgir": "GObject introspection C++ wrapper generator replaced by Rust gio/glib ecosystem.",
    "desktop-app/cmake_helpers": "CMake build scripts replaced by Cargo workspace dependencies and build.rs.",
    "Cyan4973/xxHash": "Fast hash algorithm library replaced by Rust xxhash-rust crate.",
    "google/cld3": "Compact Language Detector v3 replaced by Rust whatlang crate and Web language APIs.",
    "desktop-app/codegen": "Qt code generation tool replaced by Rust macro codegen and TypeScript definitions.",
    "nayuki/QR-Code-generator": "QR code generator replaced by Rust qrcode crate and canvas rendering.",
    "desktop-app/lib_crl": "C++ concurrency and dispatch library replaced by Tokio async runtime and crossbeam channels.",
    "Microsoft/GSL": "Guidelines Support Library replaced by Rust ownership, borrowing, and slice types.",
    "martinmoene/expected-lite": "Expected<T, E> utility replaced by Rust standard Result<T, E> enum.",
    "desktop-app/lib_webrtc": "WebRTC platform wrapper replaced by Chromium native WebRTC engine.",
    "desktop-app/lib_spellcheck": "Platform spellchecker adapter replaced by Chromium native spellcheck.",
    "desktop-app/lib_lottie": "Lottie animation player replaced by Lottie-web and WebGL/Canvas player.",
    "desktop-app/lib_rpl": "Reactive programming library replaced by Rust Tokio/Futures streams and React hooks.",
    "desktop-app/lib_webview": "Qt WebView container replaced by Electron native BrowserWindow and WebContents.",
    "desktop-app/lib_storage": "Client storage encryption wrapper replaced by SQLite (SandAgentDb) and OS keychain.",
    "TartanLlama/expected": "C++11/14/17 expected<T, E> replaced by Rust standard Result<T, E>.",
    "desktop-app/libprisma": "Color scheme and theme parsing replaced by Fabushi theme manager and CSS variables.",
    "tsyrogit/zxcvbn-c": "Password strength estimator replaced by Rust zxcvbn crate.",
    "desktop-app/lib_tl": "TL schema binary serializer replaced by stateless HTTPS/TLS framing under FMEP-EXIT-01.",
    "nnen/doxygen-theme": "Documentation theme replaced by standard Rustdoc and Markdown specifications.",
    "desktop-app/lib_translate": "In-app translation provider replaced by Cloudflare Worker AI and Fabushi translation APIs.",
    "desktop-app/lib_qr": "Qt QR display widget replaced by React SVG/Canvas QR component.",
}


def sanitize_row_id(repository: str) -> str:
    """Generate a valid responsibility ID from a repository slug."""
    clean = repository.replace("/", "-").replace(".", "-").replace("_", "-").upper()
    return f"TDRP-DEP-{clean}-001"


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

        rationale = SUBMODULE_RATIONALES.get(
            repo,
            f"Third-party submodule '{repo}' replaced by Fabushi Cargo and Electron/Chromium architecture under FMEP-BULK-01.",
        )

        responsibility_ids = []
        if scheme == "row":
            responsibility_ids = [sanitize_row_id(repo)]

        review = {
            "repository": repo,
            "commit": entry["commit"],
            "path": entry["path"],
            "object": entry["object"],
            "category": "dependency",
            "risk": "low",
            "review_depth": "contract",
            "status": "complete",
            "rationale": rationale,
            "evidence": [evidence_anchor],
            "responsibility_ids": responsibility_ids,
        }
        reviews.append(review)

    # Sort deterministically
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
        f"| 序号 | 子模块 (Repository) | 固定 Commit (SHA-1) | Blob 数量 | Commit 数量 | 总对象数 |",
        f"| ---: | --- | --- | ---: | ---: | ---: |",
    ]
    total_blobs = 0
    total_commits = 0
    total_count = 0
    for idx, (repo, stats) in enumerate(sorted(submodules.items(), key=lambda x: -x[1]["count"]), 1):
        total_blobs += stats["blobs"]
        total_commits += stats["commits"]
        total_count += stats["count"]
        sha = stats["commit"][:12] if stats["commit"] else "unknown"
        lines.append(f"| {idx} | `{repo}` | `{sha}` | {stats['blobs']} | {stats['commits']} | {stats['count']} |")

    lines.append(f"| **合计** | **{len(submodules)} 个第三方子模块** | — | **{total_blobs}** | **{total_commits}** | **{total_count}** |")
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
    print(f"Generated {len(reviews)} bulk disposition source_reviews across submodules (scheme={args.scheme}).")

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
