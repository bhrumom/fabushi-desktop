# TDRP Revision 9 — professional scenario/secrecy evidence complete read (orders 5822–5824)

Authority: `telegramdesktop/tdesktop@b0d1fe5e08c963402b8b71648c70a2bf68e6fab2` (root tree `e279373aec283ebfcc6e3bb7385e51205acd84a2`).

This range is exact-blob read-complete and responsibility-decomposed. It is test/security evidence only and grants no shipping or release credit.

- **5822 `test_scenario.cpp`**: the checked-in `SetupScenario` is deliberately a no-op debug overlay slot; automation replaces this entire file per task. The canonical responsibility is to preserve a test-only scenario injection seam without allowing product behavior or secret state to leak into the repository copy.
- **5823–5824 `test_secrecy_scan.*`**: a fail-closed secrecy oracle. It selects this launch's evidence by file identity/day/banner, never mtime; treats client plain/Send as deciding and Recv as report-only; distinguishes bounded short secrets from embedded coincidences; tracks phrase runs and long tokens; withholds any public site that could itself reveal secret material; allows only exact source-proven computed-field values to be non-deciding; requires accepted files, planted/named controls, Send+Recv headers for MTP, and matcher/dump canaries; returns undecided on missing/foreign/unreadable/control/canary/input failures; and rescans its own appended evidence rows for leakage.

Self-tests cover clean/dirty/undecided trees, identity slicing, foreign parts, computed-field traps, public-site redaction, telemetry number formatting, shared helper formatting and deliberate controls.

After this shard the deterministic read-through is **5,824 / 16,125**, unread is **10,301**, unknown remains **15,846**, omitted remains **0**, and first unread is order **5,825** `Telegram/SourceFiles/test/test_style.cpp@9fe31e6e846bb3546ee5d1a8c9a4afeab8c26bdd`.
