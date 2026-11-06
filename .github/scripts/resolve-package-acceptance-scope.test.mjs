import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { resolvePackageAcceptanceScope } from "./resolve-package-acceptance-scope.mjs";

const scriptUrl = new URL("./resolve-package-acceptance-scope.mjs", import.meta.url);
const script = fileURLToPath(scriptUrl);
const booleanInputs = [
  "PACKAGING_ARTIFACT_ONLY",
  "PACKAGING_PUBLISH_RELEASE",
  "PACKAGING_PROMOTE_EXISTING_CANDIDATE",
  "PACKAGING_ITERATE_EXISTING_MINIAPP_CANDIDATE",
];
const full = { scope: "full", artifactOnly: false };
const artifactOnly = { scope: "artifact-only", artifactOnly: true };
const dispatch = { PACKAGING_EVENT_NAME: "workflow_dispatch" };

function ownedDirectory(t) {
  const directory = mkdtempSync(join(tmpdir(), "fabushi-package-scope-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  return directory;
}

function childEnvironment(inputs, output) {
  const environment = { ...process.env };
  for (const name of ["PACKAGING_EVENT_NAME", ...booleanInputs, "GITHUB_OUTPUT"]) {
    delete environment[name];
  }
  Object.assign(environment, inputs);
  if (output !== undefined) environment.GITHUB_OUTPUT = output;
  return environment;
}

function runCli(directory, inputs, output) {
  return spawnSync(process.execPath, [script], {
    cwd: directory,
    env: childEnvironment(inputs, output),
    encoding: "utf8",
    timeout: 10_000,
  });
}

function fileSnapshot(directory) {
  return readdirSync(directory).sort().map((name) => [
    name,
    readFileSync(join(directory, name), "utf8"),
  ]);
}

test("push defaults and canonical false inputs retain full acceptance scope", () => {
  for (const value of [undefined, "", "false"]) {
    const inputs = { PACKAGING_EVENT_NAME: "push" };
    if (value !== undefined) {
      for (const name of booleanInputs) inputs[name] = value;
    }
    assert.deepEqual(resolvePackageAcceptanceScope(inputs), full);
  }
});

test("full dispatch leaves every existing release and candidate route combination to its owner", () => {
  for (let mask = 0; mask < 8; mask += 1) {
    const inputs = { ...dispatch, PACKAGING_ARTIFACT_ONLY: "false" };
    for (let index = 0; index < 3; index += 1) {
      inputs[booleanInputs[index + 1]] = mask & (1 << index) ? "true" : "false";
    }
    assert.deepEqual(resolvePackageAcceptanceScope(inputs), full);
  }
  assert.deepEqual(resolvePackageAcceptanceScope(dispatch), full);
});

test("explicit dispatch artifact-only accepts only disabled existing routes", () => {
  for (const value of [undefined, "", "false"]) {
    const inputs = { ...dispatch, PACKAGING_ARTIFACT_ONLY: "true" };
    if (value !== undefined) {
      for (const name of booleanInputs.slice(1)) inputs[name] = value;
    }
    assert.deepEqual(resolvePackageAcceptanceScope(inputs), artifactOnly);
  }
});

test("every boolean rejects noncanonical raw strings and nonstring values", () => {
  const invalid = [
    "True", "False", "TRUE", "FALSE", "1", "0", " true", "false ", "true\n",
    "null", "undefined", true, false, 0, 1, null, [], {},
  ];
  for (const name of booleanInputs) {
    for (const value of invalid) {
      assert.throws(
        () => resolvePackageAcceptanceScope({ ...dispatch, [name]: value }),
        { message: `${name} must be empty, "true", or "false"` },
      );
    }
  }
});

test("missing, unknown and noncanonical events cannot select a scope", () => {
  for (const event of [undefined, "", "workflow_call", "pull_request", "PUSH", "push ", null]) {
    for (const requested of ["false", "true"]) {
      assert.throws(
        () => resolvePackageAcceptanceScope({
          PACKAGING_EVENT_NAME: event,
          PACKAGING_ARTIFACT_ONLY: requested,
        }),
        { message: 'PACKAGING_EVENT_NAME must be "push" or "workflow_dispatch"' },
      );
    }
  }
});

test("push cannot opt out of full acceptance", () => {
  assert.throws(
    () => resolvePackageAcceptanceScope({
      PACKAGING_EVENT_NAME: "push",
      PACKAGING_ARTIFACT_ONLY: "true",
    }),
    { message: "PACKAGING_ARTIFACT_ONLY requires workflow_dispatch" },
  );
});

test("artifact-only rejects each conflicting route and their combined request", () => {
  for (const name of booleanInputs.slice(1)) {
    assert.throws(
      () => resolvePackageAcceptanceScope({
        ...dispatch,
        PACKAGING_ARTIFACT_ONLY: "true",
        [name]: "true",
      }),
      { message: `PACKAGING_ARTIFACT_ONLY cannot be combined with ${name}` },
    );
  }
  assert.throws(
    () => resolvePackageAcceptanceScope({
      ...dispatch,
      ...Object.fromEntries(booleanInputs.map((name) => [name, "true"])),
    }),
    { message: `PACKAGING_ARTIFACT_ONLY cannot be combined with ${booleanInputs.slice(1).join(", ")}` },
  );
});

test("actual CLI appends exactly the canonical outputs for full and artifact-only scopes", (t) => {
  const cases = [
    { inputs: { PACKAGING_EVENT_NAME: "push" }, expected: "scope=full\nartifact_only=false\n" },
    {
      inputs: {
        ...dispatch,
        PACKAGING_PUBLISH_RELEASE: "true",
        PACKAGING_PROMOTE_EXISTING_CANDIDATE: "true",
      },
      expected: "scope=full\nartifact_only=false\n",
    },
    {
      inputs: { ...dispatch, PACKAGING_ARTIFACT_ONLY: "true" },
      expected: "scope=artifact-only\nartifact_only=true\n",
    },
  ];
  for (const [index, entry] of cases.entries()) {
    const directory = ownedDirectory(t);
    const output = join(directory, "github output.txt");
    const prefix = index === 2 ? "" : "unrelated=preserved\n";
    if (prefix) writeFileSync(output, prefix);
    const result = runCli(directory, entry.inputs, output);
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, "");
    assert.equal(result.stderr, "");
    assert.equal(readFileSync(output, "utf8"), prefix + entry.expected);
    assert.deepEqual(readdirSync(directory), ["github output.txt"]);
  }
});

test("actual CLI failures do not create or alter output or unrelated files", (t) => {
  const cases = [
    {
      inputs: { ...dispatch, PACKAGING_ARTIFACT_ONLY: "TRUE" },
      message: /PACKAGING_ARTIFACT_ONLY must be empty, "true", or "false"/,
    },
    {
      inputs: { PACKAGING_EVENT_NAME: "workflow_call" },
      message: /PACKAGING_EVENT_NAME must be "push" or "workflow_dispatch"/,
    },
    {
      inputs: {},
      message: /PACKAGING_EVENT_NAME must be "push" or "workflow_dispatch"/,
    },
    {
      inputs: { PACKAGING_EVENT_NAME: "push", PACKAGING_ARTIFACT_ONLY: "true" },
      message: /PACKAGING_ARTIFACT_ONLY requires workflow_dispatch/,
    },
    ...booleanInputs.slice(1).map((name) => ({
      inputs: { ...dispatch, PACKAGING_ARTIFACT_ONLY: "true", [name]: "true" },
      message: new RegExp(`PACKAGING_ARTIFACT_ONLY cannot be combined with ${name}`),
    })),
    {
      inputs: { PACKAGING_EVENT_NAME: "push" },
      omitOutput: true,
      message: /GITHUB_OUTPUT must name the workflow output file/,
    },
    {
      inputs: { PACKAGING_EVENT_NAME: "push" },
      emptyOutput: true,
      message: /GITHUB_OUTPUT must name the workflow output file/,
    },
  ];
  for (const [index, entry] of cases.entries()) {
    const directory = ownedDirectory(t);
    const output = join(directory, "github-output.txt");
    writeFileSync(join(directory, "unrelated.txt"), "retained fixture");
    if (index % 2 === 0) writeFileSync(output, "existing=unchanged\n");
    const before = fileSnapshot(directory);
    const outputArgument = entry.omitOutput ? undefined : entry.emptyOutput ? "" : output;
    const result = runCli(directory, entry.inputs, outputArgument);
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 1);
    assert.equal(result.stdout, "");
    assert.match(result.stderr, /^package acceptance scope: /);
    assert.match(result.stderr, entry.message);
    assert.deepEqual(fileSnapshot(directory), before);
    if (index % 2 !== 0) assert.equal(existsSync(output), false);
  }
});

test("importing the helper never reads CLI policy or writes workflow output", (t) => {
  const directory = ownedDirectory(t);
  const output = join(directory, "github-output.txt");
  writeFileSync(output, "existing=unchanged\n");
  const before = fileSnapshot(directory);
  const result = spawnSync(process.execPath, [
    "--input-type=module",
    "--eval",
    `await import(${JSON.stringify(scriptUrl.href)});`,
  ], {
    cwd: directory,
    env: childEnvironment({
      PACKAGING_EVENT_NAME: "unknown",
      PACKAGING_ARTIFACT_ONLY: "invalid",
    }, output),
    encoding: "utf8",
    timeout: 10_000,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "");
  assert.equal(result.stderr, "");
  assert.deepEqual(fileSnapshot(directory), before);
});
