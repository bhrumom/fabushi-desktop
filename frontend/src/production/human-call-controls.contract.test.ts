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
