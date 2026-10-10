# Exact-source dossier: orders 5043-5056 — audio edit, decode, cache and track lifecycles

- Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`
- Root tree: `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`
- Accounting after batch: read-through 5,056; unread 11,064; unknown 15,841; unknown-closed 279; omitted 0.
- Read method: complete direct semantic reading of all fourteen exact blobs.

## 5043-5044 — encoded audio edit

Trim validates range and audio stream, remuxes only packets intersecting the requested range, rebases PTS/DTS to zero and forces monotonic timestamps before writing an Opus container. Concat independently validates both inputs, rescales the second stream to the output timebase, offsets it after the first and enforces monotonic PTS/DTS. Both reject empty/no-packet/error results and regenerate waveform/duration from the actual output.

Canonical owner: **Media audio edit/transformation**. Exact timestamps, invalid-input fail-closed behavior and regenerated derived metadata are required; the FFmpeg implementation is replaceable.

## 5045-5048 — decoder/custom IO/resample/speed

AudioPlayerLoader abstracts file/data/byte access, file-access leases, decoded-sample handoff and explicit Retry/RetryNotQueued/Wait/EOF/Error outcomes. AbstractFFMpegLoader provides bounded custom IO/seek over all three source forms and opens the selected audio stream. AbstractAudioFFMpegLoader normalizes sample format/rate/channel layout, owns resampler/filter resources, queues decoded frames to support speed changes, and uses atempo when available. FFMpegLoader owns codec open/seek/read/drain and never treats malformed decode as valid PCM.

Canonical owner: **Media audio decode/streaming**. Important contracts are bounded custom IO, seek, normalized PCM format, explicit wait/retry/EOF semantics, speed-change frame continuity, and deterministic cleanup.

## 5049-5050 — typed loader orchestration

The loader worker has separate voice/song/video current IDs and loader instances. Every load revalidates that the track ID, loading state and underlying source still match before mutating buffers. It accumulates decoded samples to a device buffer, waits when external streaming has not arrived, supports forced buffering, sets precise start/error/end state and reuses queued normal-speed frames when speed changes. External packet ingress is mutex-protected and coalesced to the loader thread.

Canonical owner: **Media playback/audio loader**. Stale-id fencing and speed-transition position continuity are mandatory.

## 5051-5052 — bounded local derived sound cache

Original bytes are decoded and converted to PCM S16 WAV at 44.1 kHz, mono/stereo, capped at three seconds. Successful derived bytes are cached by document id, with an explicit fallback-to-default path. Disk cache materializes deterministic private WAV files beneath its configured folder.

Canonical owner: **Media derived-audio cache**. Conversion must remain bounded and cache/disk lifetime subordinate to the media owner.

## 5053-5054 — short sound tracks and device idle lifetime

Small sounds are bounded to 10 MiB input, decoded once, optionally sampled for normalized peak animation, and played one-shot or looped at notification/default or override volume. Device detach preserves sample offset, device change reattaches active tracks, and 500 ms idle detachment releases the device when no track is active. Instance destruction requires all registered tracks gone.

Canonical owner: **Media notification/short-sound playback**.

## 5055-5056 — child/external streaming audio

The child decoder receives an already-open codec plus initial frame from a parent streaming pipeline, then consumes a packet queue. Empty queue is an explicit Wait until EOF is signaled; EOF drains the codec; invalid packets may be skipped only on the known invalid-data path; forced buffering coordinates with the parent loader.

Canonical owner: **Media streaming audio bridge**. Packet ingress, EOF/drain, buffer pressure and stale audio identity remain mandatory.

## Acceptance

Orders 5043-5056 are exact-tree/path/blob bound and read-complete only. They do not imply implementation, production wiring or release verification. Current exact-head Source authority must attest the shard and manifest.
