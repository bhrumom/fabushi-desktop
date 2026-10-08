# Media-view detached-window chrome 1496-1537 - responsibility closure dossier

Authority: telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db / tree 94ae09469c816b350f60dc9ada1ff049323be8e7.

## Exact source boundary

Orders 1496-1513 are the macOS media-view title-button raster families (title_button_close_mac, generic traffic-light background/shadow, minimize, maximize and shrink/restore). Orders 1514-1537 are the cross-platform viewer title-button and shadow families for close, minimize, maximize and restore.

The accepted upstream Telegram/SourceFiles/media/view/media_view.style composes those resources into one detached media-view WindowTitle responsibility: mediaviewTitleMinimize, mediaviewTitleMaximize, mediaviewTitleRestore, mediaviewTitleClose; the WindowTitle minimize/maximize/restore/close slots; and the corresponding macOS title-button family. The style also declares detached-viewer minimum/default window geometry. This is one responsibility boundary, not forty-two independent icon capabilities.

Binary/type and consumer evidence for orders 1496-1500 is already retained in the accepted 1401-1500 source-authority evidence. Orders 1501-1537 are proven by parent-head run 37856862858, source-authority job 113583254143, artifact 11584661832, digest sha256:b282792a51f3fb2f6d66cd00bad76c575f07147a1cdbfeffcf669fd9cf4bfc50, members source-binary-evidence-1501-1600.txt and source-consumer-reachability-1501-1600.txt.

## Fabushi platform adaptation

Fabushi does not create a second native OS window for media preview. The current canonical owner is frontend/src/recovered/features/conversation/workspace/media-viewer.tsx, composed inside the shipping Conversation workspace. It renders exactly one in-app modal role=dialog with aria-modal=true.

The upstream close/minimize/maximize/restore chrome is adapted as follows: close maps to the modal accessible close action and Escape; minimize is not applicable because no detached viewer window exists; maximize/restore are replaced by responsive full-viewport modal layout rather than independent OS-window state; OS title shadows and traffic-light artwork are not adopted.

The dialog locks background scrolling while active, restores the previous body overflow and the invoking control focus on teardown, and keeps media inside viewport-relative bounds. No BrowserWindow or renderer-side window.minimize/maximize/restore owner is introduced.

## Production and test evidence

Production owner: frontend/src/recovered/features/conversation/workspace/media-viewer.tsx and frontend/src/recovered/features/conversation/workspace/view.css.

Focused contract: CONTRACT-TDRP-MEDIAVIEW-WINDOW-CHROME-REPLACEMENT-001 in frontend/src/recovered/features/recovered-conversation-infra.contract.test.ts.

The contract requires modal semantics, close/Escape behavior, scroll lock plus focus restoration, viewport-responsive replacement, focus-visible close styling, and absence of a second BrowserWindow/minimize/maximize/restore path.

Verification is bound to implementation HEAD 930a4d4e8894c1589f184cfd5b9410bf104f53d2: GitHub Actions run 37860154455, renderer job 113594053563 completed successfully and executed the focused recovered-conversation contract, architecture gates, renderer typecheck and renderer build. Source authority job 113594053543 on that same HEAD produced artifact 11585542841 (sha256:fdaf778a364adfc3b9c71d4c511abad0842e7130bfecdaf63d122a019de05716) with exact accepted-tree binary/consumer evidence. Orders 1496-1537 are therefore atomically closed as a verified platform replacement and deterministic read-through advances to 1537. This does not close the broader IV/media-view lifecycle, remote service dependencies, packaged release acceptance, or overall source closure.
