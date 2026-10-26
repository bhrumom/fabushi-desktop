# TDRP Revision 9 — live upstream rebaseline discovery (2026-10-27)

Status: **candidate-only; NOT accepted**. This is a separately pinned discovery checkpoint, **not** a reconstructed complete recursive inventory, new accepted lock, verified ledger, or migration completion claim.

- Fabushi starting main: `3f8e7ae768d71897958198dc4924e10c77098a5c` (PR #177 at `939b9f11041735b76da5ab61f0687e3722d31370` still subject to exact-head Actions at observation).
- Prior Revision-9 upstream lock: `telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, tree `5151db79560ed40bf3802482fcfb74b950be6c40`; `accepted=false`.
- Live `dev` via GitHub branches API: `f985723b552b30fe1c17d377d57f44e61a15a8c4`, tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`.
- GitHub compare old...new: `status=ahead`, `ahead_by=245`, `behind_by=0`. This is not a per-file semantic review.
- GitHub recursive root tree at exact new tree: `truncated=false`, **7,194** entries: **299** trees, **6,859** blobs and **36** direct gitlinks. Thus root non-directory entries = **6,895**; previous root count was **6,717**. Root `commit` entries are dependency gitlinks, not source blobs.
- Changed direct gitlink pins seen in the new exact root vs prior lock: `Telegram/lib_base` `ad56a40232ba542ceb796af7c6b4f474c64df851` → `95a543e426e35938dd632139d0bb828b268c3b4c`; `Telegram/lib_ui` `af1132e87d75f80926dbce1017a96312af3d0bbe` → `79c846fdf0327ff99d3efdc1cc4e11ef50bedc6d`; `Telegram/lib_webview` `48c72924706c37addb5f0bcf70b2df8b227f1313` → `ef03bbfd696c7527d7cb57ac20c3a5d30feb98fa`. All 36 direct gitlinks and their nested chains still require new exact-pinned validation.

## Fail-closed next gates

1. Rebuild all exact-root source entries (repository/commit/path/blob) from the GitHub recursive tree, including the 178 net additional root non-directory entries and changed blobs; independently recurse the 36 gitlinks, nested gitlinks, GitLab subtree and build-time external acquisitions. Never infer recursive total as previous count plus root delta because direct gitlink pins have changed.
2. Regenerate `upstream.lock.json`, `inventory/index.json`, `parity-ledger.json`, and affected dossiers under a single exact candidate; preserve old historical evidence and re-check changed source symbols/owners. Retain `upstream.accepted=false`, `baseline_ready=false`, and `acceptance.accepted=false`.
3. GitHub Actions must verify the exact candidate and root/recursive inventory, gitlink cardinality, source graph, build acquisitions, ledger/dossier bindings, and exact-head test evidence before acceptance. No old run validates this new tree.

This checkpoint is truthful discovery evidence only. Unknown/unread/omitted counts for the new recursive candidate are **not computed** and must not be copied from the old lock or declared zero.
