# Exact-source dossier: orders 5038-5042 — audio playback and capture

- Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`
- Root tree: `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`
- Accounting after batch: read-through 5,042; unread 11,078; unknown 15,841; unknown-closed 279; omitted 0.
- Read method: complete direct exact-blob semantic reading. OpenAL/FFmpeg/Qt types are implementation reference, not a requested source copy.

## 5038-5039: playback mixer/device lifecycle

The source owns one process audio mixer with typed Voice/Song/Video tracks. It opens and tears down the playback device/context under one mutex, detects device disconnection/change and reattaches active buffered tracks. Each track has explicit stopped/start/play/pause/resume/stop/error/end states, speed-dependent and independent positions, three queued buffers, external-stream sync timestamps, volume and fade state.

The fader thread drives bounded start/stop/pause/resume fades, song suppression while voice plays, global suppression, volume changes, position updates and preload requests. Buffer starvation becomes waiting-for-data rather than false completion; OpenAL failures force a typed error terminal state. Device detach retains enough buffered state to reattach. Destructor clears tracks, closes the device, quits and joins fader/loader threads.

The same source also reads audio metadata/cover art for prepared sends and computes bounded 100-sample waveforms as derived media information.

**Fabushi owner:** current canonical Media playback/audio owner. Required parity includes device change/disconnect recovery, id-fenced track settlement, preload/buffer starvation, speed/volume, fade/suppression, external synchronization, error state, teardown and derived metadata/waveform. No Telegram/OpenAL-named product owner is created.

## 5040-5042: microphone capture/encoding lifecycle

Capture is a singleton resource owner with a worker thread. Start resolves the selected/default capture device, opens mono 48 kHz capture, and either streams timestamped PCM chunks to an external processor or initializes Opus/FFmpeg custom-IO encoding. Samples are polled on a timer; progress/level is emitted at bounded cadence.

The first 400 ms are skipped and the following 300 ms faded in; stop applies a matching tail fade and rejects too-short/unaligned recordings instead of returning corrupt media. Encoding resamples to the codec format, writes/drains packets, finalizes the container, derives waveform/duration and then frees codec/resampler/IO/device state. Pause can return the current bounded recording result; stop during an active processing pass is treated as an error-safe path. Destruction queues Inner destruction to its own thread, quits and joins.

**Fabushi owner:** current canonical Composer voice/audio capture owner plus thin microphone/codec platform adapters. Required parity includes availability, start/started state, progress/level, pause/stop, cancellation/error fencing, waveform/duration result, device/codec cleanup and no stale callback into a disposed Composer.

## Acceptance

All five rows are exact-path/blob bound and `read_complete=true`; implementation and verification remain open. A current exact-head Source authority job must attest the shard/manifest before these records are accepted as current-head source evidence.
