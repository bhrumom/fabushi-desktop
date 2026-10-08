# TDRP Revision 9 complete read: art resources 213-224

Authority: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

This dossier records complete blob acquisition plus actual QRC/shipping-consumer reads for deterministic recursive orders 213-224. Asset identity is never treated as the product owner: each row is mapped to the code/state owner that consumes it. All twelve entries remain production-evidence-open in Fabushi, so this batch reduces unread only; it does not reduce unknown.

## Shared packaging and rights finding

All twelve assets are embedded by `Telegram/Resources/qrc/telegram/telegram.qrc`. Upstream `LICENSE@d70f4d6a87eece19c4ec184de36a8aba72b4a639` is GPLv3 and `LEGAL@09c9e383db122fe0694e73c48d6224676afe60f6` identifies Telegram Desktop copyright/GPL terms. Exact-name and resource-tree searches found no per-file license, source-art, or derivation notice for these twelve assets. Therefore Fabushi may preserve the behavioral/product responsibility but must not assume that Telegram branding or artwork can be copied verbatim; any reused bytes require explicit asset-rights/provenance closure.

## 213 — affiliate_logo.png

- Exact blob: `d5f9b5087d556480526a3fab1823e00ccede4469`; full binary read; PNG 320x320, 79,980 bytes.
- Packaging: QRC alias `art/affiliate_logo.png`.
- Shipping consumer: `Telegram/SourceFiles/ui/effects/premium_top_bar.cpp@282cba14c22d03ad872fd2754c6322efbc63e46c`. On palette changes, the premium top bar chooses this image only for descriptor logo `affiliate`, scales it into the top-bar logo slot, and applies descriptor gradient stops to mini-stars.
- Responsibility: affiliate/promotion top-bar visual variant bound to the existing premium/offer surface. The image owns no entitlement, affiliate identity, commerce state, navigation, or persistence.
- Fabushi disposition: applicable visual/product-semantic mapping; reuse an existing canonical promotional/header surface if/when the corresponding commerce/affiliate capability is present. Do not introduce a Telegram-named component or copy the branded bitmap without provenance.
- Evidence still required: shipping owner, canonical UI composition, theme/a11y states, commerce-state binding, packaged visual acceptance.

## 214 — background.tgv

- Exact blob: `a106f042d62c833e8f096d2877c490eb156d1169`; full binary read; gzip payload, 183,832 bytes.
- Packaging: QRC alias `art/background.tgv`.
- Shipping consumers:
  - `window_theme.cpp@2ead4b28873908361728df3f622318ed073cdee2`: `ReadDefaultImage()` decodes it with gzip-SVG enabled.
  - `window_themes_embedded.cpp@3874c727e77b771bbca48d22d11d7819fae79abe`: light default background combines the asset with canonical `Data::DefaultWallPaper()` colors, pattern opacity, gradient rotation and generated gradient.
  - `window_theme_preview.cpp@df6755bfa0515414fef2e091f4a4622aade42a79`: preview fallback applies the same wallpaper colors/rotation/opacity.
- Responsibility: default chat background pattern projection and preview, parameterized by the real Theme/Wallpaper owner.
- Fabushi disposition: applicable. Preserve canonical Theme/Appearance state, light/dark behavior, preview, pattern/gradient parameters, accessibility and persistence ownership rather than importing Telegram wallpaper state.

## 215 — bg_initial.jpg

- Exact blob: `602b52834120e737490094b20df3be350bfbb9de`; full binary read; JPEG 480x750, 139,091 bytes.
- Packaging: QRC alias `art/bg_initial.jpg`.
- Shipping consumers:
  - `data_session.cpp@d93703bd1ef76acc05cd0742abd3e042e235e27c`: inserted as the local thumbnail for `Legacy1DefaultWallPaper`.
  - `window_theme.cpp@2ead4b28873908361728df3f622318ed073cdee2`: loaded only when the canonical wallpaper identity is Legacy1, then scaled by UI/device scale.
- Responsibility: legacy-default wallpaper compatibility/display fallback; canonical wallpaper identity and migration remain in Data/Theme/LocalStorage owners.
- Fabushi disposition: applicable only as compatibility semantics where legacy/default-background migration is required; do not create a second wallpaper store or treat the JPEG as identity truth.

## 216 — bg_thumbnail.png

- Exact blob: `529b87c55416800e2e095ac3d0616a489a216cef`; full binary read; PNG 320x480, 68,720 bytes.
- Packaging: QRC alias `art/bg_thumbnail.png`.
- Shipping consumer: `data_session.cpp@d93703bd1ef76acc05cd0742abd3e042e235e27c` appends canonical `Data::DefaultWallPaper()` when absent from the server list and assigns this local image only as its thumbnail.
- Responsibility: default-wallpaper picker/list thumbnail projection, not the full wallpaper or server state.
- Fabushi disposition: applicable through canonical Appearance/Theme picker projection if that capability is present; state remains with the single theme/settings owner.

## 217 — business_logo.png

- Exact blob: `25c357e50b1bbc32bf61df5498738adeba3a4a12`; full binary read; PNG 252x252, 48,042 bytes.
- Packaging: QRC alias `art/business_logo.png`.
- Shipping consumer: `premium_top_bar.cpp@282cba14c22d03ad872fd2754c6322efbc63e46c`. Descriptor logo `dollar` loads/scales this image and applies the premium foreground gradient to mini-stars.
- Responsibility: commerce/business top-bar visual variant; no business account, payment, entitlement, or promotion truth is owned by the bitmap.
- Fabushi disposition: applicable semantic mapping to canonical commerce/promotion header if supported; branded art reuse is rights-gated.

## 218 — cocoon.webp

- Exact blob: `060239730be9f1075cc42cdbb2e5a1835f2c46ce`; full binary read; WebP VP8X 396x396, 69,456 bytes.
- Packaging: QRC alias `art/cocoon.webp`.
- Shipping consumer: `ui/boxes/about_cocoon_box.cpp@4aab8e79a2fbbb0cf7feffffcb6cacf246e0275b` loads and device-pixel-ratio scales it into the gradient cover for `AboutCocoonBox`.
- Reachability: the information box is opened from AI compose/create info buttons, history summary UI, and translation UI; those entry points provide accessible naming where applicable.
- Responsibility: explanatory/privacy-information surface for upstream Cocoon-backed AI features, including a reachable info affordance. The image is presentation only.
- Fabushi disposition: the AI transparency/privacy/info responsibility is applicable to Fabushi AI surfaces, but the Telegram/Cocoon brand asset is not a canonical Fabushi owner and must not be copied without explicit rights/provenance. Reuse canonical Dialog/DetailPanel/Settings surfaces.

## 219-224 — dice1.svg ... dice6.svg

Exact blobs:
- 219 `dice1.svg@4ab6abbe6156b818721a2239f4ebe57a3f6d6a44`
- 220 `dice2.svg@1e9e0c85340392513459f2a8faaf4601e5ca6b47`
- 221 `dice3.svg@84ff3fcc13488970efdfddb8f33ac98ebea33ed2`
- 222 `dice4.svg@dc4af3c88f651a6284f132b2e5cd36b4af84f65f`
- 223 `dice5.svg@cdd1efd1db262f2bec428092fcd35c50de6a44e2`
- 224 `dice6.svg@f024e88b055fe11939f58413e875f1a95e50b456`

Each SVG was read in full. Each declares 128x128 artwork and is registered under QRC aliases `dice/dice1.svg` through `dice/dice6.svg`.

Shipping consumer: `Telegram/SourceFiles/ui/boxes/emoji_stake_box.cpp@86a13a1c2c08fabbe457da673d495ae87129dba7`.
- `MakeEmojiFrame(index,size)` dynamically resolves `:/gui/dice/dice%1.svg`, renders through `QSvgRenderer`, and respects device-pixel ratio.
- `MakeTable` constructs all six frames, binds faces 1-6 to `EmojiGameStakeArgs.milliRewards[i]`, and renders a separate jackpot cell using repeated face-six imagery.
- The stake/about title also embeds face six as a custom image.
- Therefore these are not generic reaction/dice assets: they are presentation for the emoji stake reward table. Reward amounts, eligibility, stake/payment mutation and settlement are owned elsewhere.

Fabushi disposition: the product responsibility is the stake/reward-result presentation integrated with the canonical payment/game owner if that capability is accepted for Fabushi. The six SVGs do not justify a second game/payment state owner, Telegram-prefixed component, or direct asset copy. Required closure includes payment/service policy, server contract, deterministic reward mapping, error/retry/settlement, accessible non-color-only result presentation, light/dark/responsive states, and packaged acceptance.

## Accounting effect

This batch proves full read + consumer decomposition for orders 213-224, so deterministic read prefix becomes 224 and unread decreases from 15,908 to 15,896. None of these twelve rows has Fabushi exact-head production evidence yet; therefore `unknown` remains 16,033 and `omitted` remains 0.
