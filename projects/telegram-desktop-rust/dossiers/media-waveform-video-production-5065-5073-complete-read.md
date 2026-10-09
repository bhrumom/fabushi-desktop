# Exact-source dossier: orders 5065-5073 — waveform, media contracts, canvas, encode and frame extraction

- Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`
- Root tree: `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`
- Accounting after batch: read-through 5,073; unread 11,047; unknown 15,841; unknown-closed 279; omitted 0.
- Read method: complete direct exact-blob semantic reading of all nine sources. This batch advances source understanding only.

## 5065-5066 — waveform derivation

The decoder accepts file or in-memory audio, finds the best audio stream, decodes and resamples to mono S16, and checks the shared atomic cancellation flag during the decode loop. Fine peaks are initially sampled at roughly 50 spans/second but are capped at 100,000 entries; on pressure the array is pairwise max-compacted and the span size doubles. Requested bars are max-reduced across the fine spans and normalized against an average-derived peak with a hard minimum/uint16 maximum. Cancellation or insufficient valid decoded audio returns an empty waveform.

Canonical owner: **Media waveform/derived-audio**.

## 5067 — shared media value contracts

Defines repeat mode, order mode, video quality identity, 0.5–2.5 playback speed bounds, tenth-step speed equality and strict positive/area-bounded frame validation. These are source-neutral state contracts and must not split into Telegram-specific stores.

## 5068-5069 — video/still canvas composition

A small smooth sample derives top/bottom colors from the image center and adapts HSV value/saturation for a usable canvas gradient. Placement preserves aspect ratio and changes the fit rule for tall media. Fitting paints the gradient and smooth image into a premultiplied canvas; empty/null inputs remain fail-safe.

Canonical owner: **Media editor/video composition**.

## 5070-5071 — media production / transcode pipeline

The source contract covers still or video input, exact/short-side sizing, crop, rotation, flip, trim range, MP4 versus alpha WebM sticker mode, CRF, audio removal/silence, FPS cap, cover capture, overlay layers, attached stickers and additional music tracks.

Key production responsibilities from the implementation:

- hard source-size ceiling of 1,000 MiB and decode area ceiling of 4096²;
- H.264 or VP9 encoders with bounded bitrate/CRF and even output dimensions;
- lossless video remux only when geometry/trim/FPS permit it, otherwise decode/compose/re-encode;
- crop/rotation/flip/display-matrix semantics are planned explicitly, with geometry baked only when needed;
- compatible AAC/MP3 audio may be remuxed; otherwise audio is decoded, resampled to 48 kHz float planar and re-encoded AAC;
- music tracks support placement, trim, volume, looping and silence padding; mixing clamps samples to [-1,1];
- silent audio is generated only under a bounded duration ceiling so broken timestamps cannot create unbounded output;
- still images can become timed H.264 video with static/animated overlays and optional music;
- animated overlays support Lottie/WebM, transform/flip/rotation and cutout compositing;
- frame timestamps are rebased and forced monotonic; copied-video DTS restart is detected and triggers a retry ignoring the broken edit-list timeline;
- source probing/size estimation avoids needless quality loss and estimates container/audio/video overhead;
- MP4 output uses fast-start/moov relocation; WebM sticker encoding retries a stricter CRF ladder until the result fits the caller's byte budget;
- progress callbacks are cancellation gates;
- transcode temp files live in a user-only directory and stale files older than 24h are cleared asynchronously.

Canonical owner: **Media send/edit video production**. These are required product/media-service semantics, not a request to copy the FFmpeg implementation.

## 5072-5073 — video frame extraction

The extractor accepts file or bytes, rejects attached-picture streams as video, reads rotation/duration, caps decode area at 4096² and reuses sequential decode when requested positions are near. Seeking occurs when moving backwards or more than three seconds ahead; a single request decodes at most a ten-second span before returning the best retained frame. Render preserves rotation, aspect fit or cover-crop and alpha premultiplication.

Canonical owner: **Media video frame/thumbnail extraction**.

## Acceptance

Orders 5065-5073 are exact-tree/path/blob bound and `read_complete=true` only. Production implementation, UI integration, service behavior, tests, artifact provenance and release acceptance remain open. Current exact-head Source authority must attest this shard and manifest.
