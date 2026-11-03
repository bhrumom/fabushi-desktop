import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const panelSource = readFileSync(new URL("./device-pairing.tsx", import.meta.url), "utf8");
const settingsSource = readFileSync(new URL("./view.tsx", import.meta.url), "utf8");
const surfaceSource = readFileSync(new URL("./desktop-surface.tsx", import.meta.url), "utf8");

test("device pairing presentation stays on the canonical Host owner and fails closed", () => {
  assert.match(panelSource, /client\.call\("device\.pair\.setupCode", \{\}\)/);
  assert.match(panelSource, /typeof value\.setupCode !== "string"/);
  assert.match(panelSource, /typeof value\.qrTerminal === "string"/);
  assert.match(panelSource, /data:image\/png;base64,/);
  assert.match(panelSource, /The pairing service returned no QR presentation\./);
  assert.match(panelSource, /result\.qrTerminal != null \? <details/);
  assert.match(panelSource, /typeof value\.expiresAtMs !== "number"/);
  assert.match(panelSource, /result\.expiresAtMs <= nowMs/);
  assert.match(panelSource, /role="alert"/);
  assert.match(panelSource, /role="status"/);
  assert.match(panelSource, /navigator\.clipboard\.writeText\(result\.setupCode\)/);
  assert.doesNotMatch(panelSource, /localStorage|sessionStorage|console\.(log|info|warn|error)/);
});

test("device pairing is mounted in the existing Settings section registry", () => {
  assert.match(settingsSource, /id: "pairing", label: "Pairing"/);
  assert.match(settingsSource, /SettingsSectionId = .*"pairing"/);
  assert.match(surfaceSource, /import \{ DevicePairingPanel \} from "\.\/device-pairing"/);
  assert.match(surfaceSource, /section === "pairing"\) return <DevicePairingPanel client=\{coordinatorClient\} \/>/);
});
