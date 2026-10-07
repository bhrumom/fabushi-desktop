# IV Markdown button-row complete read — d346 baseline

Status: read-complete / mapped-open / not implemented / not verified  
Spec: TDRP-001 Revision 9 / FBCP-001 Revision 7  
Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`  
Root tree: `5db05afa460bb030ac36316beb6496df03742c59`

## Source identities and read method

Both accepted source entries were read completely at the accepted exact commit. The implementation was read in bounded ranges 1-520 and 521-1002; no truncated connector rendering is credited as a full read.

- `Telegram/SourceFiles/iv/markdown/iv_markdown_button_row.h@0b9a011680bac413aecefe8b1aa2750b30ab62a0`
- `Telegram/SourceFiles/iv/markdown/iv_markdown_button_row.cpp@cf02fad9c6b76a2b2d9f3f7404b2e763cab41b64`

## Responsibilities recovered

### TDRP-R9-IV-BUTTON-ROW-CONTRACT-001

The public/runtime contract carries product behavior beyond painting:

- runtime button identity is projected through weak/shared ownership so disposed/replaced rows cannot accept stale activation;
- each active button has a typed button kind, color, handler, loading key and optional elided-label context;
- URL/Auth/CopyText kinds expose distinct URL/copy semantics and context-menu labels;
- disabled buttons deliberately have no activation handler;
- hit testing is against the current laid-out row, not a stale source rectangle;
- loading state is keyed to the authoritative message/button request identity rather than a renderer-only boolean;
- ripple/loading state is row-local derived interaction state and must not become a second durable message truth.

### TDRP-R9-IV-BUTTON-ROW-LIFECYCLE-001

The implementation further defines lifecycle/error/settlement behavior:

- `RichPageButtonClickHandler` re-resolves the current runtime button on every click/tooltip/copy request; expired runtime state yields no action;
- left-click activation delegates to the existing bot/rich-page action contract; other pointer buttons do not execute the action;
- URL/Auth copy yields the encoded URL data, while CopyText yields text and a different context-menu label;
- label elision changes tooltip disclosure but not action identity;
- loading is active only while the authoritative rich-page button record has a live request id;
- loading animation quiesces when the host no longer paints the covered band, and reduced/disabled animation uses a bounded timer path instead of an unbounded visual loop;
- row refresh preserves handler identity only while the corresponding runtime slot remains valid, clears disabled handlers, and drops ripple ownership when the row shrinks;
- primary/success/danger/default visual variants are presentation detail, but disabled/pending/activation state and action semantics must survive the Fabushi UI replacement.

## Existing-owner audit

Current canonical Fabushi already has suitable owners:

- `frontend/src/recovered/features/conversation/cards/transcript-card/widget-interactions.ts` owns transcript-card action pending/settled state, scope generation fencing and stale-result rejection through `createWidgetInteractionAdapter`.
- `frontend/src/recovered/features/conversation/cards/transcript-card/url-card.ts` owns normalized HTTP URL projection and the native external-open boundary through `normalizeLinkUrl` / `createUrlCardProvider`.
- `frontend/src/recovered/ui/sand-kit-primitives.tsx` contains the existing canonical button primitive family, including pending/disabled accessibility semantics.
- `frontend/src/recovered/features/conversation/workspace/transcript.tsx` owns the unique `ConversationTranscript` product composition.

A source-specific Telegram/Markdown button runtime, second message store, second URL owner, or second transcript root is therefore rejected. The target is to extend the existing transcript-card action/URL/button composition when implementation is scheduled.

## Behavior oracle / invariants

- **ORA-TDRP-IV-BUTTON-ROW-001:** a visible rich-message button can execute only the action represented by the current canonical transcript entry and current scope; replaced/disposed entries cannot settle into the new scope.
- **INV-TDRP-IV-BUTTON-STALE-001:** a stale runtime/action callback must not execute or mutate a replacement entry.
- **INV-TDRP-IV-BUTTON-DISABLED-001:** disabled controls have no activation path.
- **INV-TDRP-IV-BUTTON-COPY-001:** URL/Auth and CopyText preserve distinct clipboard/context semantics without granting a broader external-open/auth capability.
- **INV-TDRP-IV-BUTTON-LOADING-001:** pending/loading projection is derived from the authoritative action request lifecycle and quiesces after settlement/disposal/coverage loss.

These identifiers are planning/mapping oracles in this dossier. They are not execution evidence.

## Target composition

`ConversationTranscript -> TranscriptCard action/URL projection -> canonical Button primitive -> typed desktop/native action boundary`

The source UI geometry, Qt painter, gradients and ripple implementation are not ported literally. Product semantics, lifecycle, accessibility and action/security boundaries remain in scope.

## Test basis / required future evidence

Before either row can become implemented/verified, GitHub Actions must cover at least:

- scope replacement/dispose while an action is pending;
- disabled-button non-activation;
- URL/Auth/CopyText tooltip/copy/context distinctions;
- duplicate click / pending request fencing;
- rejected/failed action rollback without stale settlement;
- loading quiescence after settlement and lifecycle disposal;
- keyboard/pointer equivalence and accessible pending/disabled state in the shipping transcript;
- packaged temporal/reload/restart behavior where the action has durable effects.

Relevant normative gates: `TDRP-MOD-01`, `TDRP-MOD-02`, `TDRP-OWN-01`, `TDRP-COMP-01`, `G-FILE`, `G-MODULE`, `G-PRODUCTION`, `G-COMPOSITION`, `G-UI-FUNCTIONAL`, `G-TEMPORAL`, `G-EVIDENCE`.

## Current verdict

Read-complete and mapped-open only. No shipping implementation or current-head execution evidence is claimed by this dossier.
