import assert from "node:assert/strict";
import test from "node:test";
import {
  SandClientPersistenceStore,
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
      return [...content.keys()].filter((path) => path.startsWith(directory + "/"))
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
      const value = content.get(path);
      if (value === undefined) throw errno("ENOENT");
      return new TextEncoder().encode(value).length;
    },
  };
  return {
    files, events, pathFor,
    owner: () => new SandClientPersistenceStore(DIRECTORY, files),
    snapshot: () => Object.fromEntries([...content.entries()].sort(([left], [right]) => left.localeCompare(right))),
    dropFile(path: string) { content.delete(path); },
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
