import assert from "node:assert/strict";
import test from "node:test";
import {
  SandClientPersistenceStore,
  CLIENT_PERSISTENCE_MIGRATION_MARKER_FILENAME,
  encodeClientPersistenceFileName,
  type ClientPersistenceFiles,
} from "./client-persistence-store.ts";

// NMR-DRAFT-01: use the actual process-owned persistence class. Only the
// external file operations are controlled; no persistence queue is copied.
const DIRECTORY = "/test-owned/client-persistence";
const KEY = "sand.client.slice.account.local.composer-drafts";
const OTHER_KEY = "sand.client.slice.account.local.other-drafts";
const TIMEOUT = { timeout: 10_000 };

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((accept, fail) => { resolve = accept; reject = fail; });
  return { promise, resolve, reject };
}

function errno(code: "ENOENT" | "EIO" | "EACCES") {
  return Object.assign(new Error("Controlled file boundary " + code), { code });
}

type Operation = "list" | "read" | "write" | "rename" | "remove" | "size";
interface FileEvent { operation: Operation | "ensure"; path: string; value?: string }

function fileBoundary() {
  const content = new Map<string, string>();
  const nonRegular = new Set<string>();
  const events: FileEvent[] = [];
  const failures: { operation: Operation; path: string; error: Error; before?: () => void }[] = [];
  const writeGates: {
    value: string;
    entered: ReturnType<typeof deferred<void>>;
    release: ReturnType<typeof deferred<void>>;
  }[] = [];
  let directoryExists = false;
  const pathFor = (key: string) => DIRECTORY + "/" + encodeClientPersistenceFileName(key);
  const visit = (operation: Operation, path: string, value?: string) => {
    events.push({ operation, path, ...(value === undefined ? {} : { value }) });
    const index = failures.findIndex((failure) => failure.operation === operation && failure.path === path);
    if (index >= 0) {
      const [failure] = failures.splice(index, 1);
      failure.before?.();
      throw failure.error;
    }
  };
  const files: ClientPersistenceFiles = {
    joinPath: (directory, name) => directory + "/" + name,
    async ensureDir(directory) {
      assert.equal(directory, DIRECTORY);
      events.push({ operation: "ensure", path: directory });
      directoryExists = true;
    },
    async listFiles(directory) {
      assert.equal(directory, DIRECTORY);
      visit("list", directory);
      if (!directoryExists) throw errno("ENOENT");
      return [...new Set([...content.keys(), ...nonRegular])].filter((path) => path.startsWith(directory + "/"))
        .map((path) => path.slice(directory.length + 1));
    },
    async readTextFile(path) {
      visit("read", path);
      const value = content.get(path);
      if (value === undefined) throw errno("ENOENT");
      return value;
    },
    async writeTextFile(path, value) {
      visit("write", path, value);
      const index = writeGates.findIndex((gate) => gate.value === value);
      if (index >= 0) {
        const [gate] = writeGates.splice(index, 1);
        gate.entered.resolve();
        await gate.release.promise;
      }
      content.set(path, value);
    },
    async rename(from, to) {
      visit("rename", from, content.get(from));
      const value = content.get(from);
      if (value === undefined) throw errno("ENOENT");
      content.set(to, value);
      content.delete(from);
    },
    async removeFile(path) {
      visit("remove", path);
      content.delete(path);
    },
    async fileSize(path) {
      visit("size", path);
      if (nonRegular.has(path)) return null;
      const value = content.get(path);
      if (value === undefined) throw errno("ENOENT");
      return new TextEncoder().encode(value).length;
    },
  };
  return {
    files, events, pathFor,
    owner: (caps: { readonly maxValueBytes?: number; readonly maxTotalBytes?: number } = {}) => new SandClientPersistenceStore(DIRECTORY, files, caps),
    snapshot: () => Object.fromEntries([...content.entries()].sort(([left], [right]) => left.localeCompare(right))),
    dropFile(path: string) { content.delete(path); nonRegular.delete(path); },
    putFile(path: string, value: string) {
      directoryExists = true;
      nonRegular.delete(path);
      content.set(path, value);
    },
    replaceWithNonRegular(path: string) {
      directoryExists = true;
      content.delete(path);
      nonRegular.add(path);
    },
    nonRegularPaths: () => [...nonRegular].sort(),
    failNext(operation: Operation, path: string, error: Error, before?: () => void) {
      failures.push({ operation, path, error, ...(before === undefined ? {} : { before }) });
    },
    holdWrite(value: string) {
      const entered = deferred<void>(), release = deferred<void>();
      writeGates.push({ value, entered, release });
      return { entered: entered.promise, release: () => release.resolve(), fail: (error: Error) => release.reject(error) };
    },
  };
}

test("NMR-DRAFT-01 persistence preserves ordinary read write remove and actual ENOENT absence", TIMEOUT, async () => {
  const boundary = fileBoundary();
  const owner = boundary.owner();
  assert.equal(await owner.read(KEY), null, "A missing directory is an absent store");
  assert.deepEqual(await owner.listKeys("sand."), []);
  const original = JSON.stringify({ draftId: "original", prompt: "Kept content" });
  await owner.write(KEY, original);
  assert.equal(await owner.read(KEY), original);
  assert.deepEqual(await owner.listKeys("sand."), [KEY]);
  assert.equal(boundary.snapshot()[boundary.pathFor(KEY)], original);
  assert.equal(Object.keys(boundary.snapshot()).some((path) => path.endsWith(".tmp")), false);

  // A file can genuinely disappear after its name entered the cached index.
  boundary.dropFile(boundary.pathFor(KEY));
  assert.equal(await owner.read(KEY), null);
  assert.deepEqual(await owner.listKeys("sand."), []);
  await owner.write(KEY, original);
  await owner.remove(KEY);
  assert.equal(await owner.read(KEY), null);
  assert.deepEqual(await owner.listKeys("sand."), []);

  await owner.write(KEY, original);
  await owner.write(OTHER_KEY, "A file removed between listing and stat");
  const vanished = boundary.pathFor(OTHER_KEY);
  boundary.failNext("size", vanished, errno("ENOENT"), () => boundary.dropFile(vanished));
  const reopened = boundary.owner();
  assert.equal(await reopened.read(KEY), original);
  assert.equal(await reopened.read(OTHER_KEY), null);
  assert.deepEqual(await reopened.listKeys("sand."), [KEY]);
});

test("NMR-DRAFT-01 persistence propagates read I/O failures without hiding retained content", TIMEOUT, async () => {
  for (const code of ["EIO", "EACCES"] as const) {
    const boundary = fileBoundary();
    const owner = boundary.owner();
    const original = JSON.stringify({ draftId: "before-" + code, prompt: "Durable original" });
    await owner.write(KEY, original);
    const snapshot = boundary.snapshot();
    const underlying = errno(code);
    const fault = code === "EACCES" ? new Error("Wrapped file access failure", { cause: underlying }) : underlying;
    boundary.failNext("read", boundary.pathFor(KEY), fault);
    await assert.rejects(owner.read(KEY), (error) => error === fault);
    assert.deepEqual(boundary.snapshot(), snapshot);
    assert.deepEqual(await owner.listKeys("sand."), [KEY]);
    assert.equal(await owner.read(KEY), original, "Retry must still read the retained blob");

    const replacement = JSON.stringify({ draftId: "after-" + code, prompt: "Later explicit edit" });
    await owner.write(KEY, replacement);
    assert.equal(await owner.read(KEY), replacement);
  }
});

test("NMR-DRAFT-01 persistence never caches an empty or partial index after directory or stat failure", TIMEOUT, async () => {
  for (const operation of ["list", "size"] as const) for (const code of ["EIO", "EACCES"] as const) {
    const boundary = fileBoundary();
    const writer = boundary.owner();
    await writer.write(KEY, "First retained conversation");
    await writer.write(OTHER_KEY, "Second retained conversation");
    const snapshot = boundary.snapshot();
    const reopened = boundary.owner();
    const fault = errno(code);
    boundary.failNext(operation, operation === "list" ? DIRECTORY : boundary.pathFor(OTHER_KEY), fault);
    await assert.rejects(reopened.read(KEY), (error) => error === fault);
    assert.deepEqual(boundary.snapshot(), snapshot);
    assert.equal(await reopened.read(OTHER_KEY), "Second retained conversation");
    assert.equal(await reopened.read(KEY), "First retained conversation");
    assert.deepEqual((await reopened.listKeys("sand.")).sort(), [KEY, OTHER_KEY].sort());
    assert.deepEqual(boundary.snapshot(), snapshot);
  }
});

test("NMR-DRAFT-01 persistence retains the indexed blob after failed removal and allows retry", TIMEOUT, async () => {
  for (const code of ["EIO", "EACCES"] as const) {
    const boundary = fileBoundary();
    const owner = boundary.owner();
    await owner.write(KEY, "Must remain visible after failed remove");
    const snapshot = boundary.snapshot();
    const fault = errno(code);
    boundary.failNext("remove", boundary.pathFor(KEY), fault);
    await assert.rejects(owner.remove(KEY), (error) => error === fault);
    assert.deepEqual(boundary.snapshot(), snapshot);
    assert.deepEqual(await owner.listKeys("sand."), [KEY]);
    assert.equal(await owner.read(KEY), "Must remain visible after failed remove");

    await owner.remove(KEY);
    assert.equal(await owner.read(KEY), null);
    assert.deepEqual(await owner.listKeys("sand."), []);
    await owner.write(KEY, "Positive operation after recovery");
    assert.equal(await owner.read(KEY), "Positive operation after recovery");
  }
});

test("NMR-DRAFT-01 the main persistence owner orders an admitted old write before replacement restore and B", TIMEOUT, async () => {
  for (const oldWriteFails of [false, true]) {
    const boundary = fileBoundary();
    const owner = boundary.owner();
    const prior = JSON.stringify({ draftId: "before-old-write", prompt: "Previous durable state" });
    const original = JSON.stringify({ draftId: "old-lifetime-A", prompt: "A" });
    const newer = JSON.stringify({ draftId: "new-lifetime-B", prompt: "B" });
    await owner.write(KEY, prior);
    const held = boundary.holdWrite(original);
    const oldWrite = owner.write(KEY, original);
    const fault = errno("EIO");
    const oldCompletion = oldWriteFails
      ? assert.rejects(oldWrite, (error) => error === fault)
      : oldWrite;
    await held.entered;
    // These are calls to the same production owner admitted by main IPC.
    // Renderer disposal does not cancel the already-entered file operation.
    let replacementRead: string | null | undefined;
    const replacement = owner.read(KEY).then(async (raw) => {
      replacementRead = raw;
      await owner.write(KEY, newer);
    });
    if (oldWriteFails) held.fail(fault);
    else held.release();
    await Promise.all([oldCompletion, replacement]);
    assert.equal(replacementRead, oldWriteFails ? prior : original);
    assert.equal(await owner.read(KEY), newer);
    assert.equal(boundary.snapshot()[boundary.pathFor(KEY)], newer);
    assert.equal(Object.keys(boundary.snapshot()).some((path) => path.endsWith(".tmp")), false);
    const committedValues = boundary.events.filter((event) => event.operation === "rename").map((event) => event.value);
    assert.deepEqual(committedValues, oldWriteFails ? [prior, newer] : [prior, original, newer]);
  }
});


test("NMR-DRAFT-01 one-shot migration rejects marker I/O errors without resurrecting retired drafts", TIMEOUT, async () => {
  const marker = DIRECTORY + "/" + CLIENT_PERSISTENCE_MIGRATION_MARKER_FILENAME;
  const staleTemporary = DIRECTORY + "/controlled-stale.tmp";
  const legacy = Object.freeze([Object.freeze({ key: KEY, value: "Original legacy draft A" })]);
  const originalLegacy = JSON.stringify(legacy);
  for (const lifetime of ["same", "reopened"] as const) for (const failure of ["EIO", "wrapped-EACCES", "non-regular"] as const) {
    const boundary = fileBoundary();
    const first = boundary.owner();
    assert.equal(await first.migrateFromLocalStorage(legacy), true);
    const completedMarker = boundary.snapshot()[marker];
    assert.equal(new Date(completedMarker).toISOString(), completedMarker);
    await first.remove(KEY);
    assert.equal(await first.read(KEY), null);
    // A cold ledger would clean this temporary. A failed marker probe must
    // reject before even that mutation, as well as before any stale import.
    boundary.putFile(staleTemporary, "Retain until a permitted ledger operation");
    if (failure === "non-regular") boundary.replaceWithNonRegular(marker);
    const retained = boundary.snapshot();
    const retainedTypes = boundary.nonRegularPaths();
    const owner = lifetime === "same" ? first : boundary.owner();
    const fault = failure === "wrapped-EACCES"
      ? new Error("Controlled wrapped marker access failure", { cause: errno("EACCES") })
      : errno("EIO");
    if (failure !== "non-regular") boundary.failNext("size", marker, fault);
    const firstEvent = boundary.events.length;
    if (failure === "non-regular") {
      await assert.rejects(owner.migrateFromLocalStorage(legacy), /migration marker is not a regular file/);
    } else {
      await assert.rejects(owner.migrateFromLocalStorage(legacy), (error) => error === fault);
    }
    assert.deepEqual(boundary.events.slice(firstEvent), [{ operation: "size", path: marker }],
      "Unknown marker state must stop before ledger, ensure, write, rename or remove");
    assert.deepEqual(boundary.snapshot(), retained);
    assert.deepEqual(boundary.nonRegularPaths(), retainedTypes);
    assert.equal(boundary.snapshot()[boundary.pathFor(KEY)], undefined);
    assert.equal(JSON.stringify(legacy), originalLegacy);

    // Only the controlled external boundary recovers its original regular file;
    // production does not delete, backfill or reinterpret an old marker.
    if (failure === "non-regular") boundary.putFile(marker, completedMarker);
    const recovered = boundary.snapshot();
    const recoveryEvent = boundary.events.length;
    assert.equal(await owner.hasCompletedOneShotMigration(), true);
    assert.equal(await owner.migrateFromLocalStorage(legacy), true);
    assert.deepEqual(boundary.events.slice(recoveryEvent), [
      { operation: "size", path: marker }, { operation: "size", path: marker },
    ]);
    assert.deepEqual(boundary.snapshot(), recovered);
    assert.equal(boundary.snapshot()[boundary.pathFor(KEY)], undefined);
    assert.equal(await owner.read(KEY), null, "A stale snapshot cannot resurrect the ordinarily removed draft");
  }

  for (const wrapped of [false, true]) {
    const boundary = fileBoundary();
    const owner = boundary.owner();
    if (wrapped) boundary.failNext("size", marker, new Error("Wrapped genuine marker absence", { cause: errno("ENOENT") }));
    assert.equal(await owner.migrateFromLocalStorage(legacy), true);
    assert.equal(await owner.read(KEY), legacy[0].value);
    const completed = boundary.snapshot()[marker];
    assert.equal(new Date(completed).toISOString(), completed);
    assert.equal(boundary.events[0].operation, "size");
    assert.equal(boundary.events[0].path, marker);
    assert.equal(JSON.stringify(legacy), originalLegacy);
  }

  const historical = fileBoundary();
  const owner = historical.owner();
  await owner.write(KEY, "Current canonical draft B");
  historical.putFile(marker, "");
  const retained = historical.snapshot();
  const firstEvent = historical.events.length;
  assert.equal(await owner.hasCompletedOneShotMigration(), true, "A historical regular zero-byte marker remains completed");
  assert.equal(await owner.migrateFromLocalStorage([...legacy, { key: OTHER_KEY, value: "Must not import" }]), true);
  assert.deepEqual(historical.events.slice(firstEvent), [
    { operation: "size", path: marker }, { operation: "size", path: marker },
  ]);
  assert.deepEqual(historical.snapshot(), retained, "Compatibility does not backfill, rewrite or delete the marker");
  assert.equal(await owner.read(KEY), "Current canonical draft B");
  assert.equal(await owner.read(OTHER_KEY), null);
});

test("NMR-DRAFT-01 one-shot migration commits its marker atomically and recovers after write or rename failure", TIMEOUT, async () => {
  const marker = DIRECTORY + "/" + CLIENT_PERSISTENCE_MIGRATION_MARKER_FILENAME;
  const temporary = marker + ".tmp";
  const originalPath = DIRECTORY + "/retained-legacy-original.json";
  const initial = Object.freeze([Object.freeze({ key: KEY, value: "Legacy draft A" })]);
  const retryInput = Object.freeze([
    Object.freeze({ key: KEY, value: "Legacy draft A" }),
    Object.freeze({ key: OTHER_KEY, value: "An eligible missing draft" }),
  ]);
  const originals = JSON.stringify({ initial, retryInput });
  for (const lifetime of ["same", "reopened"] as const) for (const operation of ["write", "rename"] as const) {
    const boundary = fileBoundary();
    boundary.putFile(originalPath, originals);
    const first = boundary.owner();
    const fault = operation === "write"
      ? errno("EIO")
      : new Error("Controlled wrapped marker rename failure", { cause: errno("EACCES") });
    boundary.failNext(operation, temporary, fault,
      operation === "write" ? () => boundary.putFile(temporary, "partial marker bytes") : undefined);
    await assert.rejects(first.migrateFromLocalStorage(initial), (error) => error === fault);
    const afterFailure = boundary.snapshot();
    assert.equal(afterFailure[marker], undefined, "A failed temporary write or rename cannot create a completed marker");
    assert.equal(afterFailure[boundary.pathFor(KEY)], initial[0].value, "Previously committed data survives marker failure");
    assert.equal(afterFailure[originalPath], originals);
    assert.equal(typeof afterFailure[temporary], "string");
    if (operation === "write") assert.equal(afterFailure[temporary], "partial marker bytes");
    else assert.equal(new Date(afterFailure[temporary]).toISOString(), afterFailure[temporary]);
    assert.equal(await first.hasCompletedOneShotMigration(), false);
    assert.equal(await first.read(KEY), initial[0].value);
    assert.equal(JSON.stringify({ initial, retryInput }), originals);

    const owner = lifetime === "same" ? first : boundary.owner();
    const beforeReplacement = boundary.events.length;
    await owner.write(KEY, "Newer canonical draft B");
    assert.equal(await owner.read(KEY), "Newer canonical draft B");
    const removed = boundary.events.slice(beforeReplacement).filter((event) => event.operation === "remove");
    assert.deepEqual(removed.map((event) => event.path), lifetime === "reopened" ? [temporary] : [],
      "A replacement owner cleans only the abandoned temporary, never the canonical or original file");
    const retryEvent = boundary.events.length;
    assert.equal(await owner.migrateFromLocalStorage(retryInput), true);
    const retryEvents = boundary.events.slice(retryEvent);
    assert.deepEqual(retryEvents.filter((event) => event.operation === "rename").map((event) => event.path), [
      boundary.pathFor(OTHER_KEY) + ".tmp", temporary,
    ], "Retry commits only the missing eligible value and then one completion marker");
    assert.equal(retryEvents.some((event) => event.operation === "write" && event.path === boundary.pathFor(KEY) + ".tmp"), false);
    assert.equal(retryEvents.some((event) => event.operation === "write" && event.path === marker), false,
      "The final marker is never the write target");
    assert.equal(await owner.read(KEY), "Newer canonical draft B");
    assert.equal(await owner.read(OTHER_KEY), retryInput[1].value);
    const complete = boundary.snapshot();
    assert.equal(new Date(complete[marker]).toISOString(), complete[marker], "The existing ISO marker format is unchanged");
    assert.equal(complete[originalPath], originals);
    assert.equal(Object.keys(complete).some((path) => path.endsWith(".tmp")), false);
    assert.equal(JSON.stringify({ initial, retryInput }), originals);

    const replayEvent = boundary.events.length;
    assert.equal(await owner.migrateFromLocalStorage([
      ...retryInput, { key: "sand.client.slice.account.local.unseen-draft", value: "Must not import after completion" },
    ]), true);
    assert.deepEqual(boundary.events.slice(replayEvent), [{ operation: "size", path: marker }]);
    assert.deepEqual(boundary.snapshot(), complete);
  }

  // Distinguish marker acknowledgment failure above from a genuinely
  // interrupted import: both entries are present in the same frozen input,
  // and the second blob fails before that original input is fully imported.
  for (const lifetime of ["same", "reopened"] as const) for (const operation of ["write", "rename"] as const) {
    const boundary = fileBoundary();
    boundary.putFile(originalPath, originals);
    const first = boundary.owner();
    const otherTemporary = boundary.pathFor(OTHER_KEY) + ".tmp";
    const fault = operation === "write"
      ? errno("EIO")
      : new Error("Controlled wrapped entry rename failure", { cause: errno("EACCES") });
    boundary.failNext(operation, otherTemporary, fault,
      operation === "write" ? () => boundary.putFile(otherTemporary, "partial entry bytes") : undefined);
    assert.equal(await first.migrateFromLocalStorage(retryInput), false,
      "The existing per-entry failure contract is false, unlike marker commit rejection");
    const interrupted = boundary.snapshot();
    assert.equal(interrupted[boundary.pathFor(KEY)], retryInput[0].value);
    assert.equal(interrupted[boundary.pathFor(OTHER_KEY)], undefined);
    assert.equal(interrupted[marker], undefined);
    assert.equal(interrupted[temporary], undefined, "Marker commit cannot begin before the original input finishes");
    assert.equal(interrupted[originalPath], originals);
    assert.equal(interrupted[otherTemporary], operation === "write" ? "partial entry bytes" : retryInput[1].value);
    assert.equal(boundary.events.some((event) => event.operation === operation && event.path === otherTemporary), true);
    assert.equal(boundary.events.some((event) => (event.operation === "write" || event.operation === "rename")
      && (event.path === marker || event.path === temporary)), false);
    assert.equal(JSON.stringify({ initial, retryInput }), originals);

    const owner = lifetime === "same" ? first : boundary.owner();
    const replacementEvent = boundary.events.length;
    await owner.write(KEY, "Newer B after interrupted import");
    assert.deepEqual(boundary.events.slice(replacementEvent)
      .filter((event) => event.operation === "remove").map((event) => event.path),
    lifetime === "reopened" ? [otherTemporary] : []);
    const retryEvent = boundary.events.length;
    assert.equal(await owner.migrateFromLocalStorage(retryInput), true,
      "Retry resumes the exact original frozen input without replacing newer canonical values");
    const retryEvents = boundary.events.slice(retryEvent);
    assert.deepEqual(retryEvents.filter((event) => event.operation === "rename").map((event) => event.path),
      [otherTemporary, temporary]);
    assert.equal(retryEvents.some((event) => event.operation === "write"
      && event.path === boundary.pathFor(KEY) + ".tmp"), false);
    assert.equal(await owner.read(KEY), "Newer B after interrupted import");
    assert.equal(await owner.read(OTHER_KEY), retryInput[1].value);
    const complete = boundary.snapshot();
    assert.equal(new Date(complete[marker]).toISOString(), complete[marker]);
    assert.equal(complete[originalPath], originals);
    assert.equal(Object.keys(complete).some((path) => path.endsWith(".tmp")), false);
    assert.equal(JSON.stringify({ initial, retryInput }), originals);
    const replayEvent = boundary.events.length;
    assert.equal(await owner.migrateFromLocalStorage(retryInput), true);
    assert.deepEqual(boundary.events.slice(replayEvent), [{ operation: "size", path: marker }]);
    assert.deepEqual(boundary.snapshot(), complete);
  }

  // Namespace, existing-key priority and both caps remain the original policy:
  // invalid or over-budget entries are skipped while eligible values complete.
  const boundary = fileBoundary();
  const owner = boundary.owner({ maxValueBytes: 4, maxTotalBytes: 6 });
  const small = "sand.migration.small";
  const tooLarge = "sand.migration.per-key-cap";
  const totalCap = "sand.migration.total-cap";
  await owner.write(KEY, "B");
  const compatibility = Object.freeze([
    Object.freeze({ key: "outside.namespace", value: "x" }),
    Object.freeze({ key: "sand." + "x".repeat(200), value: "x" }),
    Object.freeze({ key: KEY, value: "oldA" }),
    Object.freeze({ key: tooLarge, value: "12345" }),
    Object.freeze({ key: small, value: "abc" }),
    Object.freeze({ key: totalCap, value: "xyz" }),
  ]);
  const supplied = JSON.stringify(compatibility);
  assert.equal(await owner.migrateFromLocalStorage(compatibility), true);
  assert.equal(await owner.read(KEY), "B");
  assert.equal(await owner.read(small), "abc");
  assert.equal(await owner.read(tooLarge), null);
  assert.equal(await owner.read(totalCap), null);
  assert.deepEqual((await owner.listKeys("sand.")).sort(), [KEY, small].sort());
  const complete = boundary.snapshot();
  assert.deepEqual(Object.keys(complete).sort(), [boundary.pathFor(KEY), boundary.pathFor(small), marker].sort());
  assert.equal(new Date(complete[marker]).toISOString(), complete[marker]);
  assert.equal(JSON.stringify(compatibility), supplied);
  assert.equal(await owner.hasCompletedOneShotMigration(), true);
});
