# IV Markdown view-widget complete read — d346 baseline

Status: read-complete / mapped-open / bounded Highlight + transcript/viewer pointer lifecycle slices implemented / not verified
Spec: TDRP-001 Revision 9 / FBCP-001 Revision 7  
Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`  
Root tree: `5db05afa460bb030ac36316beb6496df03742c59`

## Source identity and read method

`Telegram/SourceFiles/iv/markdown/iv_markdown_view_widget.cpp@431b61464f8e0b89d1615366813d3fc8bd176196`

Read completely in bounded ranges 1–430, 431–860, 861–1290 and 1291–1634. No truncated output is counted as a complete read.

## Responsibility recovered — TDRP-R9-IV-VIEW-WIDGET-LIFECYCLE-001

The Qt widget itself is presentation-specific, but the file carries mature product behavior that remains applicable:

- replacing/removing an article clears active/pressed click handlers, detaches text/media callbacks from the old article, fences repaint callbacks with weak widget lifetime, resets selection/caches and relayout state;
- palette/raster changes invalidate the correct derived article caches rather than mutating canonical content;
- search-match projection, anchor lookup, scroll-anchor restore, details expansion/toggle and blockquote toggle all preserve one article state owner;
- copy/context menu chooses between selected rich text, typed prepared-link copy semantics and inline-button URL copy; no generic URL behavior is synthesized for a link kind that does not expose it;
- code-block copy sanitizes the click-handler context so viewer-copy feedback cannot retain an owning session-window capability;
- Control-wheel zoom is accumulated in deterministic steps; horizontal wheel/touch/pointer scrolling uses one directional-lock and begin/update/end lifecycle;
- touch cancellation, focus loss, pointer leave, article replacement/content change and drag settlement all release pressed/ripple/tooltip state;
- selection differentiates letters/words/paragraphs, tracks endpoints across segments, and prevents an active selection/drag from accidentally activating a link/toggle/media action;
- media, prepared-link and generic click-handler activation are arbitrated in one order, with left/middle button policy and selection conflicts checked before dispatch;
- visible range is projected into article coordinates and drives request-derived loading coverage plus formatted-date refresh scheduling;
- formatted-date refresh is suspended while hidden/minimized and bounded to a one-day timer horizon;
- missing-media relayout is retried once rather than entering an unbounded relayout loop.

## Existing-owner audit

The applicable behavior belongs in existing Fabushi owners:

- `frontend/src/recovered/features/conversation/workspace/transcript.tsx`: canonical ConversationTranscript/rich-text interaction surface.
- `frontend/src/recovered/features/conversation/workspace/find-in-chat-controller.ts`: current transcript search/navigation lifecycle.
- `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx`: current media interaction/resource projection.
- `frontend/src/recovered/features/conversation/cards/transcript-card/url-card.ts`: current typed URL normalization/external-open owner.
- `frontend/src/production/ProductionRenderer.tsx`: only shipping ConversationWorkspace composition and message-copy/action wiring.

Rejected: a MarkdownDocument application root, Telegram/IV navigation owner, second transcript store, second URL owner, or second media owner.

## Oracle / invariants

- **ORA-TDRP-IV-VIEW-WIDGET-001:** all pointer/keyboard/touch/context-menu interactions must resolve against the current canonical content/scope and cannot settle into a replaced/disposed surface.
- **INV-TDRP-IV-VIEW-REPLACE-001:** replacing content releases old active/pressed/media callbacks and stale repaint/action paths.
- **INV-TDRP-IV-SELECTION-ACTION-001:** non-empty selection/drag prevents accidental link/toggle/media activation.
- **INV-TDRP-IV-COPY-TYPED-001:** selected-rich-text, prepared-link, inline-button URL and code-block copy keep distinct semantic/security paths.
- **INV-TDRP-IV-SCROLL-LIFECYCLE-001:** horizontal wheel/touch/pointer scroll has balanced begin/update/end/cancel semantics.
- **INV-TDRP-IV-VISIBLE-WORK-001:** timers/loading/repaint work is visibility/lifetime bounded and cannot continue indefinitely after hidden/disposed/replaced state.
- **INV-TDRP-IV-MEDIA-ARBITRATION-001:** media/link/generic action dispatch has one deterministic priority and no duplicate activation.

## Target composition

`ConversationWorkspace -> ConversationTranscript -> canonical rich-text/search/media/url action owners`

Qt QWidget painting, cursors and platform event classes are replaced by web/desktop canonical interaction primitives. State-machine, cancellation, selection, activation, visibility and security semantics remain in scope.

## Future evidence required

GitHub Actions must eventually prove:
- current-head execution of `UNIT-TDRP-IV-VIEW-HIGHLIGHT-RUNTIME-001` and `CONTRACT-TDRP-IV-VIEW-HIGHLIGHT-DISPOSAL-001`, covering supersede, transcript replacement, unmount cancellation and late-settlement fencing;
- transcript pointer intent across content replacement for links, attachment/media buttons, disclosure toggles and other button actions (implemented by `CONTRACT-TDRP-IV-VIEW-POINTER-ACTIVATION-FENCE-001`; exact-head execution pending); canonical MediaViewer media replacement/unmount now releases active pointer capture, implicit lost-pointer-capture retires the matching owner, and interactive controls are excluded from pan capture (`CONTRACT-TDRP-IV-VIEW-MEDIA-POINTER-RELEASE-001`; exact-head execution pending); broader gesture settlement remains open;
- focus-chain exit/window blur/page hiding, pointer leave, touch/pointer cancellation and drag cancellation for transcript action intent (implemented by `CONTRACT-TDRP-IV-VIEW-POINTER-ACTIVATION-FENCE-001`; exact-head execution pending); broader touch-scroll settlement remains open;
- selection versus link/toggle/media activation conflicts (pointer-originated canonical transcript links/buttons are now suppressed while the browser exposes a non-empty selection; keyboard activation remains independent; exact-head execution pending);
- typed external-link copy context is handled by the canonical message menu; code-block copy now crosses the capability-minimal `copyTranscriptCodeText(code, clipboard)` boundary, which accepts no session/window/controller context and fails closed on missing/denied clipboard access (`CONTRACT-TDRP-IV-VIEW-CODE-COPY-SANITIZED-001`; exact-head execution pending);
- wheel/touch/pointer horizontal scroll begin/end balance;
- zoom/reflow/anchor continuity;
- hidden/minimized timer quiescence and reactivation;
- missing-media bounded retry;
- shipping keyboard/a11y/temporal/reload evidence.

Normative identifiers: `TDRP-MOD-01`, `TDRP-MOD-02`, `TDRP-OWN-01`, `TDRP-COMP-01`, `G-FILE`, `G-MODULE`, `G-PRODUCTION`, `G-COMPOSITION`, `G-UI-FUNCTIONAL`, `G-TEMPORAL`, `G-PERF-SOAK`, `G-SECURITY-PRIVACY`, `G-EVIDENCE`.

## Current verdict

Read-complete and mapped-open. The current PR implements bounded lifecycle slices inside existing canonical owners. Under `FindInChatBar`, only the latest scheduled Highlight refresh may settle, transcript replacement invalidates pending work, callbacks read the live controller/container, and StrictMode-safe disposal cancels late work before it can retain detached DOM in the global CSS Highlight registry. In `ConversationTranscript`, primary pointerdown on a transcript action target (`a[href]` or `button`) captures the current target plus entries generation; entries replacement, pointer cancel (including touch cancellation), pointer leave, movement beyond the bounded drag threshold, focus-chain exit, window blur, or page hiding retire that intent. A later pointer click is stopped in capture unless generation and target identity still match, and pointer-originated canonical transcript links/buttons are suppressed while the browser exposes a non-empty selection. Keyboard activation remains independent and the existing typed URL-owner checks continue to arbitrate downstream dispatch. `UNIT-TDRP-IV-VIEW-HIGHLIGHT-RUNTIME-001`, `CONTRACT-TDRP-IV-VIEW-HIGHLIGHT-DISPOSAL-001`, `CONTRACT-TDRP-IV-VIEW-POINTER-ACTIVATION-FENCE-001`, and `CONTRACT-TDRP-IV-VIEW-MEDIA-POINTER-RELEASE-001` trace these bounded slices. MediaViewer replacement/unmount now releases active capture before pointer ownership is reset, implicit capture loss clears the matching owner, and zoomed panning does not steal canonical interactive controls. Code-block copy now delegates to a capability-minimal clipboard helper that receives only code text plus the clipboard writer and cannot retain a session/window/controller capability. MediaViewer pan/zoom gesture breadth beyond pointer-owner retirement, touch/scroll/zoom equivalence, general hidden-window timers, and packaged temporal/a11y/performance evidence remain open; no verified claim is made.
