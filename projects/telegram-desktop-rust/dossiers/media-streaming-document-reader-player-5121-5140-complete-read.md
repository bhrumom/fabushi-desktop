# Exact-source dossier: orders 5121-5140 — streaming Document, file worker, loaders, Player and Reader

Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`, tree `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`. Complete direct exact-blob reads. Accounting after batch: read-through 5,140; unread 10,980; unknown 15,841; unknown-closed 279; omitted 0.

## 5121-5122 — shared streaming Document

The Document owns a shared Player/Reader for document/photo consumers, registers instances, derives loader priority from active consumers, projects bounded waiting animation state, validates/saves a good thumbnail into the canonical cache, and reacts to measured speed to request a higher or lower quality under explicit preload/speed thresholds. Quality switching is a request to the surrounding canonical media owner, not a second player.

## 5123-5125 — file/decode worker boundary

The File creates a bounded FFmpeg IO context around Reader, caps a single read and queued packets, initializes exact audio/video streams, seeks only through the configured seekable contract, processes queued packets until the delegate asks it to sleep, supports wake/end/loop, reports cache completeness and owns interruption + thread join on stop/destruction. FileDelegate is the sole typed ready/error/wait/cache/packet/end boundary back to Player.

## 5126-5127 — shared consumer Instance

An Instance is a handle over one shared Document. Copying registers another consumer; destruction unregisters it. It forwards play/pause/resume/stop, speed/volume, frame and shown-mark operations, and owns per-consumer priority plus an explicit shared-player lock that must be balanced. This preserves one shared player truth across multiple views.

## 5128-5133 — part loaders

The Loader contract uses fixed 128-KiB parts, explicit failed-offset signaling, cancellation, priority epochs, speed estimates and optional streamed-downloader attachment. Local loaders validate file size and exact part shape, publish only on the main thread and reject remote-downloader semantics. The remote source loader de-duplicates offsets, reuses already downloaded parts, supports priority/reset/cancel/stop and estimates speed only across intervals with active requests, marking low-sample estimates unreliable. In Fabushi the remote mechanism maps to the existing media transport; MTProto itself is not a dependency target.

## 5134-5135 — macOS native frame adapter

The macOS adapter accepts only a valid CoreVideo bi-planar 420 pixel buffer, RAII-locks it read-only, validates planes/strides, allocates destination storage and converts NV12 to BGRA. Null buffers, unsupported formats, failed locks/planes/storage/conversion setup fail closed. This belongs only in the canonical macOS media adapter where native texture decoding is used.

## 5136-5137 — streaming Player

Player is the central streaming state machine: opens the file worker, creates audio/video tracks, merges valid start information, tracks received/played positions, schedules video frames, coordinates audio-video sync, pauses for user or lack of data separately, resumes when buffering thresholds are met, supports mark-as-shown gating, speed/volume, loop/end/failure, full-cache projection and shared instance locks. File-thread callbacks cross to main-thread state through guarded callbacks; session and playback lifetimes are explicit.

## 5138-5139 — Reader/cache

Reader divides remote content into bounded parts/slices, handles header-special caching, validates serialized cache entries, keeps a small used-slice working set, preloads ahead, shares data with the full-file downloader, communicates between streaming and main threads through queues/semaphores, supports sleep/wake/async stop, de-duplicates/cancels offsets and flushes surviving slices to cache on teardown. A streaming failure does not invalidate the separate downloader role.

## 5140 — round preview

RoundPreview wraps an in-memory clip reader for circular preview frames. It starts only when ready, marks decode errors bad, requests rounded frames and forwards repaint notifications through the current callback.

## Acceptance

All twenty entries are exact tree/path/blob bound and `read_complete=true` only. No streaming implementation or release credit is granted. Same-head Source authority plus concurrency/fault/performance/platform tests and shipping MediaViewer evidence remain required.
