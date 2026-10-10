# UI effects 6085-6104 complete read

Orders **6,085-6,104** are exact-blob read and responsibility-decomposed against accepted Telegram `65e23ba7137ea4129b6bc1b2616104a1f59495ef` / root tree `6b616494f3465324e749a04dcd1c9d508657a998`. Reading alone does not reduce `unknown`.

- **Particles and celebration (6,085-6,090, 6,098-6,099):** drifting/fireworks/Stars particles preserve bounded frame deltas, randomized but bounded placement/motion, clipping, pause clock compensation, lifetime completion and theme/DPR cache behavior. Reduced-motion remains a canonical product policy rather than a source widget concern.
- **Reaction fly and glare/loading (6,087-6,094):** transient custom-emoji/reaction overlays fence target visibility and dirty repaint; glare refuses when animation is disabled; LoadingState skeletons preserve semantic row geometry, RTL mirroring and responsive width.
- **Message send transition (6,095-6,097):** pending local-id origin is reconciled to the authoritative Transcript item, cleaned on id change/removal/clear, type-checked for text/sticker/GIF, gated by reduced-motion/power saving, destroyed if the target disappears and repainted when resource download state changes. This is presentation over canonical Message/Transcript truth, not a second message state owner.
- **Story/status outline (6,100-6,101):** bounded segmented circular outlines and unread gradients map to canonical Avatar/Status/Badge presentation.
- **Premium/commerce presentation (6,102-6,104):** shared component styling plus optional GPU 3D cover lifecycle maps to canonical commerce/credits/gifts owners and semantic design-system components. GPU support is capability-gated; pause freezes current output and compensates timers; light/dark opacity and pointer gestures remain lifecycle inputs, not commerce truth.

Accounting after this read: **6,104 / 16,125**, **10,021 unread**, **15,846 unknown**, **0 omitted**. First unread: **6,105** `Telegram/SourceFiles/ui/effects/premium_3d_mesh.cpp@b8541b310714f1449d7fd2544e562c16453a00cf`.
