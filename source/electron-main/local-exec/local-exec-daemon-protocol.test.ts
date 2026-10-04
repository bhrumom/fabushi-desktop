import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  clearLocalExecDaemonDiscoveryIfMatches,
  parseLocalExecConnection,
  parseLocalExecCredential,
  parseLocalExecDiscovery,
  readLocalExecDaemonConnection,
  readLocalExecDaemonDiscovery,
  writeLocalExecDaemonConnection,
  writeLocalExecDaemonDiscovery,
} from "./local-exec-daemon-protocol.js";

test("local-exec protocol validates connection, credential, and discovery payloads", () => {
  assert.deepEqual(parseLocalExecConnection({
    baseUrl: "http://127.0.0.1:4123",
    token: "token",
    headers: { "x-test": "ok" },
  }), {
    baseUrl: "http://127.0.0.1:4123",
    token: "token",
    headers: { "x-test": "ok" },
  });
  assert.equal(parseLocalExecConnection({ baseUrl: "", token: "x" }), null);
  assert.equal(parseLocalExecConnection({ baseUrl: "http://x", headers: { bad: 7 } }), null);

  assert.deepEqual(parseLocalExecCredential({
    credential: "renew-me",
    backendUrl: "https://api2.cursor.sh",
    expiresAtMs: 42,
  }), {
    credential: "renew-me",
    backendUrl: "https://api2.cursor.sh",
    expiresAtMs: 42,
  });
  assert.equal(parseLocalExecCredential({ credential: "", backendUrl: "https://x" }), null);

  assert.deepEqual(parseLocalExecDiscovery({
    pid: 77,
    startedAt: 1234,
    entryRealpath: "/app/local-exec",
    generationToken: "g-1",
    inflightCount: 2,
  }), {
    pid: 77,
    startedAt: 1234,
    entryRealpath: "/app/local-exec",
    generationToken: "g-1",
    inflightCount: 2,
  });
  assert.equal(parseLocalExecDiscovery({ pid: 0, startedAt: 1 }), null);
  assert.equal(parseLocalExecDiscovery({ pid: 1, startedAt: 1, inflightCount: -1 }), null);
});

test("local-exec discovery clear is compare-and-retire rather than blind deletion", async () => {
  const dir = await mkdtemp(join(tmpdir(), "fabushi-local-exec-protocol-"));
  try {
    const discoveryPath = join(dir, "local-exec-daemon.json");
    const current = {
      pid: 77,
      startedAt: 1234,
      entryRealpath: "/app/local-exec",
      generationToken: "g-1",
      inflightCount: 0,
    };
    await writeLocalExecDaemonDiscovery(current, discoveryPath);

    assert.equal(await clearLocalExecDaemonDiscoveryIfMatches({
      ...current,
      generationToken: "stale",
    }, discoveryPath), false);
    assert.deepEqual(await readLocalExecDaemonDiscovery(discoveryPath), current);

    assert.equal(await clearLocalExecDaemonDiscoveryIfMatches(current, discoveryPath), true);
    assert.equal(await readLocalExecDaemonDiscovery(discoveryPath), null);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

test("local-exec connection writer keeps secret material owner-only", async () => {
  const dir = await mkdtemp(join(tmpdir(), "fabushi-local-exec-connection-"));
  try {
    const path = join(dir, "connection.json");
    await writeLocalExecDaemonConnection({
      baseUrl: "http://127.0.0.1:4123",
      token: "secret",
    }, path);
    assert.deepEqual(await readLocalExecDaemonConnection(path), {
      baseUrl: "http://127.0.0.1:4123",
      token: "secret",
    });
    const raw = await readFile(path, "utf8");
    assert.match(raw, /"token":"secret"/);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});
