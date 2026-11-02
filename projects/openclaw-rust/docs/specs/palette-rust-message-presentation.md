# Palette presentation of current Rust message results — PMS-v1

Status: implementation contract; exact-H Actions and complete UI acceptance pending.
Discovery H: `8e5545b61ad916bb9781539dfe0f28543ffc68dd`.
Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`.
This closes the existing Fabushi consumer defect RSU-FUNCTIONAL-CONTEXT-01;
no new OpenClaw reading or whole-search equivalence credit is implied.

## Actual chain and decision

`searchAgents` already returns accepted, ordered message results from the Rust
Session/content-search owners. `ProductionRenderer` supplies its existing message
provider to `CommandPalette`; the provider validates the reply and fences late
responses by requestId. The model currently applies its own per-code-point
lowercase/token/fuzzy filter to original snippets. Query `ος` can therefore lose
Rust's hit `ΟΣ`; combining-mark or emoji queries can also be removed by the
empty-token All-tab early return. Fuzzy sorting additionally reorders Host results.

**PMS-01 — FIX / IMPL_BUG**: keep message matching and ordering in the existing
Rust owner. The renderer consumes current accepted results; it does not add a
second Unicode matching rule. Other palette categories keep their current policy.

## Contract

| ID | Required behavior |
| --- | --- |
| PMS-01 | Only a successful reply for the provider's still-current request can authorize display as a Host message result. Bind local provenance to the exact parsed ready-array identity, actual trimmed query and existing request generation/availability/disposal state. Never accept provenance fields from transport JSON, parser-only output, clones or manually constructed objects. |
| PMS-02 | Loading, same-query retry, failure, cancellation, reset, loss of availability, empty query and dispose revoke old provenance immediately. A late reply that ignores abort cannot publish or regain authority, including A→B→A. Headers or a query string alone are not provenance. |
| PMS-03 | The existing model accepts messages only with ready status and matching current provenance. Keep every accepted original snippet, ID, agent ID, role and timestamp, with Host order preserved. Do not normalize, token-match or fuzzy-sort messages in JavaScript. |
| PMS-04 | Messages tab displays that Host-ordered block. All tab keeps the current non-message results and their relative order, inserts the Host-ordered message block after visible/non-message results and before hidden participants, and retains it for nonempty queries whose local token list is empty. Empty query retains the existing default view. Other tabs do not gain messages. |
| PMS-05 | `CommandPalette` passes its existing messageStatus to the model and includes it in the memo dependency list. Reused previous arrays must not keep old entries visible after a status transition. ProductionRenderer's existing value/status/provider/account-reset composition remains the caller. |
| PMS-06 | Keep existing non-message normalization, fuzzy budget, deduplication, tab/filter/virtualization/navigation and request debounce. No new store, permission owner, search engine, transport protocol or persistent format is introduced. Currentness bookkeeping is private presentation metadata in the existing provider. |

The provider may use a private WeakMap for ready-array provenance plus a read-only
currentness query. It must reuse the current provider lifecycle and not introduce
a second scheduler/epoch owner. An absent status/provenance grants no message
admission. Preserve all old non-message tests and behavior. A direct `.ts` runtime
import for the model's existing link-display helper is permitted for the existing
Node 24 test path; do not create a custom loader/framework.

## Scope and causal evidence

Production paths are the existing `command-palette-message-provider.ts`,
`command-palette-model.ts`, and `CommandPalette.tsx`. No Rust search matching
change is required after RSU-v1. Add a Rust integration contract in
`source/host/tests/palette_message_gateway_contract.rs` that persists actual
transcripts through ProductionSessionWorkers, calls the real Session Gateway,
independently asserts IDs/snippets/order, and optionally exports the exact returned
JSON when `OCRM_PALETTE_FIXTURE_PATH` is supplied by the Actions command.
The export uses fixture-owned data only. Absence of the export variable must not
skip any Rust behavior assertion.

`frontend/src/production/command-palette-message-search.contract.test.ts` must
require that real Rust export for positive consumer cases, then pass its original
results through the existing provider and actual model. Controlled transport may
hold or fail replies but must not implement search or manufacture positive search
results. Pin lowercase-final-sigma, expansion/contraction, partial expansion,
emoji/empty-token All results, Host order versus old fuzzy order, and original
navigation identity. Test malformed/forged provenance, retained arrays, same-query
retry, late ignored-abort replies, reentry/reset/unavailable/dispose and unchanged
other-category ordering. Check actual component status/memo and production caller
wiring; typecheck the changed production graph with the existing desktop TypeScript
configuration.

Root integration must run this export→consumer chain on one exact H in GitHub
Actions, bind U/tree, commands/exits, required test names, input hashes and exported
JSON/log hashes, and save receipts with the existing owner evidence. Reuse the
Linux Host build cache and Node 24/dependency conventions; preserve all MGS/MGA,
RSU, media, preview, Automation and config gates. Source-only assertions, a
handwritten JSON fixture, a skipped test or a TDRP result cannot substitute for
this OCRM evidence. No local or htch-runtime build/test/lint/schema execution.

No durable migration or new restart work is needed. Cancel/replacement ownership
stays with the existing provider and Rust execution owners. Full rendered-browser
interaction, accessibility/visual acceptance, actual remote transport/indexed
backends, platforms/packages and all applicable final OCRM gates remain open.

