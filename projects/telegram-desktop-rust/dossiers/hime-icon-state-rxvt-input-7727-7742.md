# Hime icon-state source and rxvt input integration — orders 7,727–7,742

Accepted recursive authority: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941` → `Telegram/ThirdParty/hime` → `hime-ime/hime@9b3e6f9ab59d1fe4d9de73d3bf0fed7789f921c5`.

Exact-head source-review artifact `11679359489` (sha256:674f0e23e3393b806ef183ff38aa937b16faeca9261c9c2a7611c2052893284f) materialized all 16 files. Recomputed Git blob SHA-1, byte length, mode/type and recursive path all matched the accepted tree.

## 7,727–7,741 — layered icon source and exported status variants

Order 7,727 is a 30×30 GIMP XCF with 20 inspected layers: BG, half, pink/blue/gray/dark base layers, white/full and half variants, and full/half/grayscale effect layers. Orders 7,728–7,741 are 30×30 PNG exports for black, blue, dark, gray and pink base/full/half states.

Hime artwork, palette and branding are replacement-only. The applicable responsibility is typed input-status distinctions plus deterministic source/resource provenance. Those semantics map to the existing Fabushi design-system asset pipeline, Resource owner and platform-input status owner. No Hime asset registry or second input-state owner is introduced.

## 7,742 — rxvt-unicode input-method patch

The patch adds key press/release dispatch with handled/not-handled fallback, focus in/out lifecycle, input-context creation/destruction and preedit spot/cursor geometry. rxvt-unicode/Hime/XIM mechanics are replacement-only. Applicable semantics map to Fabushi's existing platform input + canonical Composer/RichText editor path, which already owns IME composition/key suppression and cross-conversation scope fencing.

This range is read and responsibility-decomposed but remains `mapped-open`. Reading closes no unknown and does not establish shipping equivalence. Focused production tests and descendant exact-head evidence remain required.

Next deterministic unread: **7,743** `Telegram/ThirdParty/hime::distro/dev-tools/qt5-test/Makefile`.
