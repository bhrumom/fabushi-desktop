# Existing roster search Unicode match projection — RSU-v1

Status: implementation contract; exact-H Actions and complete acceptance pending.
Discovery H: `42ee1165d216a08361e6a1c7f9fe29bd609c8b1c`.
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`.
This preserves the existing Fabushi search path under OCRM-R05/R24 and
AC13/AC24. It grants no new upstream reading, citation or equivalence credit.

## Production defect and existing owners

The command palette requests `searchAgents`; the registered Host method reaches
the existing TranscriptManager / Session gateway and `roster_search::search_agents`.
When the index is absent, not ready or returns no available result, the linear
path reads actual Session transcript entries and constructs snippets. The renderer
uses the returned snippet for both the result label and search matching.

The current roster snippet locates the query in `flat.to_lowercase()` but treats
that byte offset as an offset in the original flattened text. Lowercasing can
expand or contract UTF-8. For example, 90 `İ` characters followed by `Needle`
can produce only an ellipsis for query `needle`, losing the visible match.

Decision **RSU-01 — FIX / IMPL_BUG**: resolve the match back to the original text
before selecting context. Use the existing `content_search::agent_content_search`
snippet implementation as the shared pure helper owner. Keep Session storage,
Transcript roster dispatch, index readiness and renderer projection owners.
Do not copy a second independent normalization/mapping algorithm into roster.
The existing typed helper lowercases individual characters; blindly delegating
would lose the roster path's whole-string contextual lowercasing, such as the
Greek final sigma. That behavior must also be retained by the shared matcher.

## Contract and causal controls

| ID | Required behavior |
| --- | --- |
| RSU-01 | Lookup uses the existing whole-string Unicode lowercase semantics. Convert a found normalized span back to complete original UTF-8 characters before slicing; preserve the matched original casing, including matches within a lowercase expansion. |
| RSU-02 | Preserve whitespace collapsing, empty/no-match outcomes and ellipses. The roster path keeps its 30-byte leading / 60-byte trailing context at valid UTF-8 boundaries around the correctly mapped original span. The existing typed helper keeps its 30/60 character context. ASCII outputs remain exact. |
| RSU-03 | The actual Session gateway fallback returns readable snippets containing the original matched span for expanding, contracting, multibyte and contextual cases. Exercise absent, unready and unavailable index fallbacks; an available index still supplies its own results. |
| RSU-04 | Retain JSON shape, role/timestamp projection, newest-first ordering, hidden-peer exclusion, per-agent/global limits and stale indexed-agent filtering. A read must not mutate stored transcript rows; reopen the Session owner and repeat a meaningful search. |
| RSU-05 | The algorithm stays linear in input plus the underlying substring search; do not repeatedly lowercase growing prefixes or introduce a second index, store, task, callback or permission owner. |

Use real `ProductionSessionWorkers` and the existing registered Session gateway
contract seam in `source/host/tests/roster_search_contract.rs`. Establish actual
persisted input before search and retained input after search/reopen. Add direct
shared-helper controls in `agent_content_search_contract.rs` for Unicode casing,
whitespace, empty/no-match and preserved character windows. Expected snippets
must be independently specified, not obtained by calling another implementation
of the same function. Preserve all existing tests and indexed/media controls.

The existing Linux Host cache can execute both affected contract targets in the
current native read/recovery step; record required names exactly once, H/tree/U,
commands/exits and input/log hashes. Add both targets to the existing macOS
recovery command only where absent, with the same named evidence check.
All executable validation is restricted to GitHub Actions. No local or
htch-runtime compilation, tests, generator, schema or lint execution is allowed.

## Recovery and limits

This is a synchronous read projection with no accepted asynchronous job,
new persistent format or data migration. Restart/reopen uses the same Session
owner and transcript bytes. Existing errors and cancellation ownership remain.

This batch does not implement the separate Memory citation/provenance gap,
Unicode normalization beyond the existing lowercase contract, every indexed
search backend, full UI interaction, remote services or packaged/platform
acceptance. All applicable full-migration gates remain open. Historical dossiers
and read facts are retained; no global inventory/ledger/RTM flag is promoted.
