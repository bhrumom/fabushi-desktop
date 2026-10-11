import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

const source = fs.readFileSync(new URL("./human-call-media.tsx", import.meta.url), "utf8");

test("Human call controls reuse canonical Button semantics", () => {
  assert.ok(source.includes('import { SandButton } from "../recovered/ui/sand-kit-primitives";'));
  assert.equal(source.includes("<button "), false);
  assert.ok(source.includes('<SandButton aria-label="Start voice call"'));
  assert.ok(source.includes('<SandButton aria-label="Accept voice call"'));
  assert.ok(source.includes('<SandButton aria-label="Decline call"'));
  assert.ok(source.includes('<SandButton aria-label="End call"'));
  assert.ok(source.includes("aria-pressed={muted}"));
  assert.ok(source.includes("aria-pressed={cameraEnabled}"));
  assert.ok(source.includes("aria-pressed={screenSharing}"));
});

test("mute command owns duplicate refusal, pending projection, rollback and teardown", () => {
  assert.ok(source.includes("createInFlightCommandFence"));
  assert.ok(source.includes('callCommandFence.acquire("toggle-mute")'));
  assert.ok(source.includes("if (commandLease == null) return;"));
  assert.ok(source.includes("pending={mutePending}"));
  assert.ok(source.includes("track.enabled = !muted"));
  assert.ok(source.includes("setMuted(muted);"));
  assert.ok(source.includes("commandLease.release();"));
  assert.ok(source.includes("callCommandFence.dispose();"));
});

test("camera and screen commands share video-media ownership with pending and rollback", () => {
  assert.equal((source.match(/callCommandFence\.acquire\("video-media"\)/g) ?? []).length >= 3, true);
  assert.ok(source.includes("pending={cameraPending}"));
  assert.ok(source.includes("pending={screenPending}"));
  assert.ok(source.includes("existing.enabled = previousEnabled;"));
  assert.ok(source.includes("stream.removeTrack(track);"));
  assert.ok(source.includes("const screenStreamRef = useRef<MediaStream | null>(null);"));
  assert.ok(source.includes("await sender.replaceTrack(camera);"));
  assert.ok(source.includes("await sender.replaceTrack(screenTrack).catch(() => undefined);"));
  assert.ok(source.includes("stopStream(screenStreamRef.current);"));
});

test("screen sharing uses canonical selection UI and fences stale or vanished sources", () => {
  assert.ok(source.includes('import { OverlayDialog } from "../recovered/ui/overlay-primitives";'));
  assert.ok(source.includes('import { SandMenuItem } from "../recovered/ui/sand-menu-item";'));
  assert.equal(source.includes("const source = sources[0]"), false);
  assert.ok(source.includes("displayPickerGenerationRef.current"));
  assert.ok(source.includes("generation !== displayPickerGenerationRef.current"));
  assert.ok(source.includes("refreshed.find((candidate) => candidate.id === sourceId)"));
  assert.ok(source.includes("That display source is no longer available. Choose another source."));
  assert.ok(source.includes('label="Choose a display source"'));
  assert.ok(source.includes('aria-label="Choose another display source"'));
  assert.ok(source.includes("screenStreamRef.current !== capture"));
});

test("camera capture lifecycle maps pause, one-shot fatal recovery, and teardown into the canonical media owner", () => {
  assert.ok(source.includes("const [cameraCapturePaused, setCameraCapturePaused] = useState(false);"));
  assert.ok(source.includes("const cameraCaptureGenerationRef = useRef(0);"));
  assert.ok(source.includes("const cameraRecoveryInFlightRef = useRef(false);"));
  assert.ok(source.includes("const bindCameraTrackLifecycle = useCallback"));
  assert.ok(source.includes("cameraCaptureGenerationRef.current === generation"));
  assert.ok(source.includes("track.onmute = () => projectActivity(true);"));
  assert.ok(source.includes("track.onunmute = () => projectActivity(false);"));
  assert.ok(source.includes("track.onended = () =>"));
  assert.ok(source.includes("cameraRecoveryInFlightRef.current = true;"));
  assert.ok(source.includes("getPreferredUserMedia(bridge, { audio: false, video: true })"));
  assert.ok(source.includes("await sender.replaceTrack(replacement);"));
  assert.ok(source.includes("bindCameraTrackLifecycleRef.current(stream, replacement);"));
  assert.ok(source.includes("Camera capture stopped after one recovery attempt"));
  assert.ok(source.includes("existing != null && existing.readyState === \"live\""));
  assert.ok(source.includes("video: false, videoPaused: false, videoActive: false"));
  assert.ok(source.includes("Camera paused"));
  assert.equal((source.match(/invalidateCameraCapture\(\)/g) ?? []).length >= 6, true);
});

test("screen capture pause and active lifecycle stay fenced to the selected capture", () => {
  assert.ok(source.includes("const [screenCapturePaused, setScreenCapturePaused] = useState(false);"));
  assert.ok(source.includes("track.onmute = () => projectCaptureActivity(true);"));
  assert.ok(source.includes("track.onunmute = () => projectCaptureActivity(false);"));
  assert.ok(source.includes("screenStreamRef.current !== capture"));
  assert.ok(source.includes("screenSharePaused: paused"));
  assert.ok(source.includes("screenShareActive: !paused"));
  assert.ok(source.includes("screenSharePaused: false, screenShareActive: false"));
  assert.ok(source.includes("Screen sharing paused"));
  assert.equal((source.match(/setScreenCapturePaused\(false\)/g) ?? []).length >= 5, true);
});

test("call diagnostics sample WebRTC stats with generation and teardown fencing", () => {
  assert.ok(source.includes("pc.getStats()"));
  assert.ok(source.includes("projectCallDiagnostics"));
  assert.ok(source.includes("diagnosticsGenerationRef.current !== samplingGeneration"));
  assert.ok(source.includes("callRef.current?.generation !== callGeneration"));
  assert.ok(source.includes("window.setInterval"));
  assert.ok(source.includes("window.clearInterval(diagnosticsTimerRef.current)"));
  assert.ok(source.includes('record.type === "candidate-pair"'));
  assert.ok(source.includes('statNumber(selectedPair, "currentRoundTripTime")'));
  assert.ok(source.includes('statNumber(record, "packetsLost")'));
  assert.ok(source.includes('statNumber(record, "jitter")'));
  assert.ok(source.includes('aria-label="Call diagnostics"'));
  assert.ok(source.includes("startDiagnostics(pc, nextCall.generation)"));
  assert.ok(source.includes("stopDiagnostics();"));
});


test("call-tone feedback owns sample data, rate, finite loops, offset, lifecycle, retry, and failure projection", () => {
  assert.ok(source.includes("CALL_TONE_SAMPLE_RATE = 48_000"));
  assert.ok(source.includes("CALL_TONE_LOOP_COUNT = 60"));
  assert.ok(source.includes("CALL_TONE_INITIAL_OFFSET_SAMPLES = 0"));
  assert.ok(source.includes("samples: Int16Array"));
  assert.ok(source.includes("loopCount: number"));
  assert.ok(source.includes("offsetSamples: number"));
  assert.ok(source.includes("advanceCallToneOffset"));
  assert.ok(source.includes("new AudioContext({ sampleRate: CALL_TONE_SAMPLE_RATE })"));
  assert.ok(source.includes("source.start(0, initialOffset)"));
  assert.ok(source.includes("source.stop(this.context.currentTime"));
  assert.ok(source.includes("callToneFailedKeyRef.current === key"));
  assert.ok(source.includes("stopCallTone(false, true);"));
  assert.ok(source.includes("void player.dispose();"));
  assert.ok(source.includes("Call audio feedback is unavailable. You can still answer or end the call."));
  assert.ok(source.includes('aria-label="Enable call sound"'));
  assert.ok(source.includes("data-call-tone-sample-rate={CALL_TONE_SAMPLE_RATE}"));
  assert.ok(source.includes("data-call-tone-loop-count={CALL_TONE_LOOP_COUNT}"));
  assert.ok(source.includes("data-call-tone-offset-samples={CALL_TONE_INITIAL_OFFSET_SAMPLES}"));
});


test("SFU session gateway stays privileged and renderer-safe", () => {
  const nativeMessaging = fs.readFileSync(
    new URL("../../../source/host/src/extensions/session/native_messaging.rs", import.meta.url),
    "utf8"
  );
  const production = fs.readFileSync(
    new URL("../../../source/host/src/extensions/session/production.rs", import.meta.url),
    "utf8"
  );
  const gateway = fs.readFileSync(
    new URL("../../../source/host/src/extensions/session/gateway.rs", import.meta.url),
    "utf8"
  );
  assert.ok(nativeMessaging.includes("list_human_call_media_sessions"));
  assert.ok(nativeMessaging.includes("create_human_call_media_session"));
  assert.ok(nativeMessaging.includes("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE_ENV"));
  assert.ok(nativeMessaging.includes("/media/sessions"));
  assert.ok(production.includes("list_call_media_sessions"));
  assert.ok(production.includes("create_call_media_session"));
  assert.ok(production.includes("canonical CallSession/device fence"));
  assert.ok(gateway.includes('"listCallMediaSessions"'));
  assert.ok(gateway.includes('"createCallMediaSession"'));
  assert.equal(source.includes("FABUSHI_REALTIME_SFU_APP_SECRET"), false);
  assert.equal(source.includes("provider_session_id"), false);
  assert.equal(source.includes("providerSessionId"), false);
  assert.equal(source.includes("FABUSHI_HOST_ACCESS_CREDENTIAL_FILE"), false);
});
