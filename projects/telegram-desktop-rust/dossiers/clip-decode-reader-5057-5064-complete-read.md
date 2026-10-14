# Exact-source dossier: orders 5057-5064 — media clip streamability, decode and reader lifecycle

- Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`
- Root tree: `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`
- Accounting after batch: read-through 5,064; unread 11,056; unknown 15,841; unknown-closed 279; omitted 0.
- Read method: complete direct exact-blob semantic reading of all eight sources. These records close source understanding only.

## 5057-5058 — fast-start / streamability probe

`CheckStreamingSupport` accepts file or memory bytes, rejects undersized/unopenable input, walks MP4 atom headers including extended 64-bit sizes, and returns true only when `moov` appears before 128 KiB. Malformed lengths, truncated extended sizes, atoms running past the source or late/missing `moov` fail closed.

Canonical owner: **Media attachment/streaming inspection**. This is a derived capability probe, not a media/message owner.

## 5059-5062 — bounded video/animated decoder

The FFmpeg reader opens file/memory through custom IO, chooses the video stream, reads rotation metadata and in Inspecting mode detects an audio stream. Decode area is hard bounded: 1280×720 for inline playback and 3840×2160 for sending/inspection, enforced both through decoder max-pixels and per-frame validation.

Read uses a packet queue and explicit Success/Error/EOF outcomes. Up to a bounded number of invalid packets may be skipped; EOF before any frame is an error. Normal EOF seeks back to start and flushes the codec for animation looping. Real media time and presentation time are tracked separately so frame delays can be corrected without falsifying media position.

Rendering preserves/identifies alpha, premultiplies when needed, resizes through a reusable scaler and applies 90/180/270-degree rotation. Inspecting classification treats small no-audio H.264 as gifv and small no-audio VP9 as WebM sticker. Codec/format/custom-IO resources are freed deterministically.

Canonical owner: **Media video/animated clip decode**.

## 5063-5064 — reader scheduler, visibility and lifetime fencing

Reader construction assigns one of up to eight workers, preferring the least loaded after all workers exist. Display uses an atomic triple-buffer protocol with explicit waiting-for-dimensions, waiting-for-request and waiting-for-first-frame states before the cyclic show/write states.

Frame requests carry DPR-adjusted frame/outer size, corner/radius, alpha policy and colorization. Prepared frames reuse caches while preserving alpha/background/rounding semantics.

The worker-side private object owns the file access lease and decoder. Promotion from the public Reader pointer map into the worker reader map occurs under one mutex. Stop first removes the public pointer entry; an unpromoted private can then be deleted because no worker route can reach it. Pointer lookup also verifies that the public Reader still points to the same private, protecting against address reuse. Main-thread callbacks re-check that the manager still carries the Reader.

When the current frame is not displayed for a bounded period, GIF decoding auto-pauses; displaying again resumes it. Manual video pause shifts animation clocks so resume does not jump. The scheduler sleeps until the next presentation deadline, and reader destruction/manager clear release file leases, decoder state and load accounting. Started load accounting is upgraded from the placeholder estimate to actual pixel area even if the owning Reader disappeared, preventing permanent worker-load drift.

`PrepareForSending` runs Inspecting mode, derives duration/audio/gifv/WebM-sticker flags, renders the first frame as thumbnail (making alpha opaque except WebM stickers), and combines the fast-start probe into `supportsStreaming`.

Canonical owner: **Media preview/animated-video reader** plus the existing attachment inspection path. No separate Telegram/GIF runtime is introduced.

## Acceptance

Orders 5057-5064 are exact-tree/path/blob bound and read-complete. They remain mapped-open: current Fabushi production wiring, responsive/a11y UI behavior, integration/E2E and release verification are still required. Current exact-head Source authority must attest the shard and manifest.
