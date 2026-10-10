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
