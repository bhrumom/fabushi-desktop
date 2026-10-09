# Telegram source read: deterministic orders 2401–2500

Authority: accepted `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`, with binary/generated evidence from run `37866391182`, job `113614324572`, artifact `11588621238`.

This range covers Media Player PiP/play/repeat/settings/shuffle/speed/volume, playlist download/play/pause/cancel, discrete audio-speed options, Poll creation/configuration/result state and media-upload/hidden-results feedback, then Premium feature-promo particles. Premium particle names are dynamically resolved in `premium_promo_particles.cpp` through QRC-backed `icons/premium/%1.svg`; they are applicable even where simple literal resource grep would undercount them. Poll animations are likewise QRC-backed with direct C++ callers.

Only `playlist_shadow` (orders 2449–2451) is explicitly unreachable: generated accepted-tree reachability has no consumer and exact resource audit found no product caller. Its three density files do not create a Fabushi feature. All applicable rows remain mapped/open; source reading grants no implemented or verified credit.
