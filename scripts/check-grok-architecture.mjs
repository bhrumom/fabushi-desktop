#!/usr/bin/env node
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const manifestPath = path.join(root, 'projects/grok-fabu-parity/architecture-manifest.json');
const strict = process.argv.includes('--strict') || process.env.GROK_ARCH_STRICT === '1';

function fail(message) {
  console.error(`[grok-architecture] ERROR: ${message}`);
  process.exitCode = 1;
}

function gitBlobSha(content) {
  const bytes = Buffer.isBuffer(content) ? content : Buffer.from(content);
  return crypto
    .createHash('sha1')
    .update(Buffer.from(`blob ${bytes.length}\0`))
    .update(bytes)
    .digest('hex');
}

const manifestBase = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
const semanticAdaptationsPath = path.join(root, 'projects/grok-fabu-parity/architecture-semantic-adaptations.json');
const semanticAdaptations = fs.existsSync(semanticAdaptationsPath)
  ? JSON.parse(fs.readFileSync(semanticAdaptationsPath, 'utf8'))
  : { schemaVersion: 1, adaptations: [] };

if (semanticAdaptations.schemaVersion !== 1 || !Array.isArray(semanticAdaptations.adaptations)) {
  fail('invalid architecture semantic-adaptations registry');
}

const adaptationRefs = new Set();
const baseRowsByReference = new Map((manifestBase.modules ?? []).map((row) => [row.referencePath, row]));
for (const adaptation of semanticAdaptations.adaptations) {
  if (
    typeof adaptation.referencePath !== 'string'
    || typeof adaptation.expectedBaseParityMode !== 'string'
    || typeof adaptation.parityMode !== 'string'
    || typeof adaptation.targetBlobSha !== 'string'
    || typeof adaptation.rationale !== 'string'
    || !Array.isArray(adaptation.behavioralEvidence)
    || !Array.isArray(adaptation.testEvidence)
  ) {
    fail(`incomplete semantic adaptation: ${JSON.stringify(adaptation)}`);
    continue;
  }
  if (adaptationRefs.has(adaptation.referencePath)) {
    fail(`duplicate semantic adaptation for ${adaptation.referencePath}`);
    continue;
  }
  adaptationRefs.add(adaptation.referencePath);
  const baseRow = baseRowsByReference.get(adaptation.referencePath);
  if (baseRow == null) {
    fail(`semantic adaptation has no manifest row: ${adaptation.referencePath}`);
    continue;
  }
  if (baseRow.status !== 'implemented') {
    fail(`semantic adaptation cannot finalize non-implemented row: ${adaptation.referencePath} is ${baseRow.status}`);
  }
  if (baseRow.parityMode !== adaptation.expectedBaseParityMode) {
    fail(`semantic adaptation base mode changed for ${adaptation.referencePath}: expected ${adaptation.expectedBaseParityMode}, found ${baseRow.parityMode}`);
  }
  if (!['typescript-port', 'rust-adaptation'].includes(adaptation.parityMode)) {
    fail(`unsupported semantic adaptation mode ${adaptation.parityMode} for ${adaptation.referencePath}`);
  }
  if (!/^[0-9a-f]{40}$/.test(adaptation.targetBlobSha)) {
    fail(`semantic adaptation requires targetBlobSha for ${adaptation.referencePath}`);
  }
  if (adaptation.rationale.trim() === '') {
    fail(`semantic adaptation requires rationale for ${adaptation.referencePath}`);
  }
  if (adaptation.behavioralEvidence.length === 0 || adaptation.testEvidence.length === 0) {
    fail(`semantic adaptation requires behavioral and test evidence for ${adaptation.referencePath}`);
  }
  const target = path.resolve(root, baseRow.targetPath);
  if (!target.startsWith(root + path.sep) || !fs.existsSync(target)) {
    fail(`semantic adaptation target missing: ${baseRow.targetPath}`);
  } else {
    const actualTargetBlobSha = gitBlobSha(fs.readFileSync(target));
    if (actualTargetBlobSha !== adaptation.targetBlobSha) {
      fail(`semantic adaptation target drifted: ${baseRow.targetPath} expected ${adaptation.targetBlobSha}, found ${actualTargetBlobSha}`);
    }
  }
}

const manifest = {
  ...manifestBase,
  modules: (manifestBase.modules ?? []).map((row) => {
    const adaptation = semanticAdaptations.adaptations.find((candidate) => candidate.referencePath === row.referencePath);
    if (adaptation == null) return row;
    return {
      ...row,
      parityMode: adaptation.parityMode,
      behavioralEvidence: adaptation.behavioralEvidence,
      testEvidence: adaptation.testEvidence,
      notes: adaptation.rationale,
    };
  }),
};
if (manifest.schemaVersion !== 3) fail(`unsupported schemaVersion ${manifest.schemaVersion}`);
if (manifest.frozenReference?.commit !== 'a9f633e09d49a85829b8236331b9e21f7e612634') {
  fail('frozen Grok reference SHA changed');
}
if (!Array.isArray(manifest.modules) || manifest.modules.length !== manifest.moduleCount) {
  fail('moduleCount does not match modules length');
}

const refs = new Set();
const targets = new Map();
const allowedDevelopment = new Set(['planned', 'existing-needs-parity', 'implemented', 'not-applicable-noncode', 'removed-extra']);
const finalAllowed = new Set(['implemented', 'not-applicable-noncode', 'removed-extra']);
const requiredDomains = new Map([
  ['frontend/', 0],
  ['source/electron-main/', 0],
  ['source/electron-preload/', 0],
  ['source/node-agent-coordinator/', 0],
  ['source/host/', 0],
  ['source/shared/', 0],
  ['source/packages/', 0],
]);

for (const row of manifest.modules ?? []) {
  if (
    !row.referencePath
    || !/^[0-9a-f]{40}$/.test(row.referenceBlobSha || '')
    || !row.architecturalRole
    || !row.targetPath
    || !row.targetLanguage
    || !row.owner
    || !row.status
  ) {
    fail(`incomplete manifest row: ${JSON.stringify(row)}`);
    continue;
  }
  if (refs.has(row.referencePath)) fail(`duplicate referencePath ${row.referencePath}`);
  refs.add(row.referencePath);
  if (!allowedDevelopment.has(row.status)) fail(`invalid status ${row.status} for ${row.referencePath}`);
  if (typeof row.processOrPackage !== 'string' || row.processOrPackage.trim() === '') {
    fail(`missing processOrPackage for ${row.referencePath}`);
  }
  if (strict && !finalAllowed.has(row.status)) fail(`final gate: ${row.referencePath} is ${row.status}`);
  if (row.status === 'removed-extra') {
    fail(`reference module cannot be removed-extra; Fabushi-only extras must be inventoried separately: ${row.referencePath}`);
  }
  if (row.status === 'not-applicable-noncode') {
    if (row.referenceBlobSha) {
      fail(`source-bearing reference module cannot be not-applicable-noncode: ${row.referencePath}`);
    }
    if (!Array.isArray(row.testEvidence) || row.testEvidence.length === 0) {
      fail(`not-applicable-noncode row requires evidence: ${row.referencePath}`);
    }
  }
  if (row.status === 'implemented') {
    const target = path.resolve(root, row.targetPath);
    if (!target.startsWith(root + path.sep) || !fs.existsSync(target)) {
      fail(`implemented target missing: ${row.targetPath} for ${row.referencePath}`);
    }
    if (row.parityMode === 'reference-copy') {
      if (!['typescript', 'react-typescript'].includes(row.targetLanguage)) {
        fail(`reference-copy parity is only allowed for TypeScript/React modules: ${row.referencePath}`);
      }
      const actualBlobSha = gitBlobSha(fs.readFileSync(target));
      if (actualBlobSha !== row.referenceBlobSha) {
        fail(`reference-copy target diverged: ${row.targetPath} expected ${row.referenceBlobSha}, found ${actualBlobSha}`);
      }
    }
    for (const [label, evidencePaths] of [
      ['behavioralEvidence', row.behavioralEvidence],
      ['testEvidence', row.testEvidence],
    ]) {
      if (!Array.isArray(evidencePaths) || evidencePaths.length === 0) {
        fail(`implemented row requires non-empty ${label}: ${row.referencePath}`);
        continue;
      }
      for (const evidencePath of evidencePaths) {
        const evidence = path.resolve(root, evidencePath);
        if (!evidence.startsWith(root + path.sep) || !fs.existsSync(evidence)) {
          fail(`implemented ${label} missing: ${evidencePath} for ${row.referencePath}`);
        }
      }
    }
  }
  const prior = targets.get(row.targetPath);
  if (prior && prior !== row.referencePath) {
    fail(`targetPath collision: ${row.targetPath} maps both ${prior} and ${row.referencePath}`);
  } else {
    targets.set(row.targetPath, row.referencePath);
  }
  for (const domain of requiredDomains.keys()) {
    if (row.referencePath.startsWith(domain)) requiredDomains.set(domain, requiredDomains.get(domain) + 1);
  }
}
for (const [domain, count] of requiredDomains) {
  if (count === 0) fail(`frozen inventory has no entries for required domain ${domain}`);
}

const expectedCounts = {
  'frontend/': 280,
  'source/electron-main/': 184,
  'source/electron-preload/': 16,
  'source/node-agent-coordinator/': 24,
  'source/host/': 471,
  'source/shared/': 165,
  'source/packages/': 852,
};
for (const [domain, expected] of Object.entries(expectedCounts)) {
  const actual = requiredDomains.get(domain);
  if (actual !== expected) fail(`${domain} expected ${expected} frozen modules, found ${actual}`);
}

const coordinatorMainPath = path.join(root, 'source/node-agent-coordinator/src/main.rs');
if (fs.existsSync(coordinatorMainPath)) {
  const coordinatorMain = fs.readFileSync(coordinatorMainPath, 'utf8');
  if (coordinatorMain.includes('env::var("MAHAYANA_API_BASE_URL")')) {
    fail('product API must not be reused as the Host gateway in Mahayana Coordinator');
  }
  for (const [label, pattern] of [
    ['Host stdin business serialization', /serde_json::to_writer\s*\(\s*&mut\s+active\.stdin/],
    ['legacy stdin business fallback marker', /Compatibility lane while the Host gateway/],
    ['Host stdout business reply settlement', /HOST_REQUEST_FAILED/],
  ]) {
    if (pattern.test(coordinatorMain)) {
      fail(`shipping Coordinator must use the Host gateway exclusively; found ${label}`);
    }
  }
  for (const required of [
    'read_gateway_discovery',
    'dispatch_http_json',
    'stream_http_events',
    'gateway_discovery_path',
  ]) {
    if (!coordinatorMain.includes(required)) {
      fail('shipping Coordinator must retain Grok Host gateway transport: missing ' + required);
    }
  }
}
const coordinatorMainRow = (manifest.modules ?? []).find(
  (row) => row.referencePath === 'source/node-agent-coordinator/main.ts',
);
if (coordinatorMainRow?.status === 'implemented' && fs.existsSync(coordinatorMainPath)) {
  const coordinatorMain = fs.readFileSync(coordinatorMainPath, 'utf8');
  for (const [label, patterns] of [
    ['LocalExec supervisor', [/LocalExecDaemonRuntime/, /local_exec/]],
    ['MCP OAuth forwarder', [/McpOAuthForwarder/, /mcp_oauth_forwarder/]],
    ['WebAuthn provider', [/WebAuthnProvider/, /webauthn/]],
    ['Inference router', [/InferenceRouter|CoordinatorInferenceRouter/, /inference_router/]],
    ['client-side tool relay', [/ClientSideToolV2Relay/, /client_side_tool_v2_relay/]],
  ]) {
    if (!patterns.some((pattern) => pattern.test(coordinatorMain))) {
      fail(`Coordinator main cannot be implemented before production composition wires ${label}`);
    }
  }
}

const sourceHostMainPath = path.join(root, 'source/host/app/src/main.rs');
if (!fs.existsSync(sourceHostMainPath)) {
  fail('shipping Host binary must be owned by source/host/app/src/main.rs');
}

const desktopPackagePath = path.join(root, 'desktop/package.json');
if (fs.existsSync(desktopPackagePath)) {
  const desktopPackage = JSON.parse(fs.readFileSync(desktopPackagePath, 'utf8'));
  if (strict) {
    const shippingMain = String(desktopPackage.main ?? '');
    if (/^electron\//.test(shippingMain)) {
      fail(`final gate: desktop package main still points at legacy CJS Electron root: ${shippingMain}`);
    }
    const shippedFiles = Array.isArray(desktopPackage.build?.files)
      ? desktopPackage.build.files.map(String)
      : [];
    if (shippedFiles.some((entry) => entry === 'electron/**' || entry.startsWith('electron/'))) {
      fail('final gate: desktop package still ships the legacy desktop/electron runtime');
    }
  }

  for (const scriptName of ['build:host', 'build:host:ci']) {
    const script = String(desktopPackage.scripts?.[scriptName] ?? '');
    if (!script.includes('../source/host/app/Cargo.toml') || !script.includes('--bin mahayana-app-host')) {
      fail(`${scriptName} must compile the shipping Host from source/host`);
    }
    if (script.includes('mahayana-app-host-desktop')) {
      fail(`${scriptName} must not compile the legacy third_party desktop Host binary directly`);
    }
  }
}

const stageHostPath = path.join(root, 'desktop/scripts/stage-host.mjs');
if (fs.existsSync(stageHostPath)) {
  const stageHost = fs.readFileSync(stageHostPath, 'utf8');
  if (!stageHost.includes("'source',\n  'host',\n  'app',\n  'target'")) {
    fail('desktop staging must copy mahayana-app-host from source/host/app/target');
  }
  if (/hostExecutable[\s\S]{0,260}'third_party'[\s\S]{0,260}'mahayana-rs'[\s\S]{0,260}'target'/.test(stageHost)) {
    fail('desktop staging must not copy the legacy third_party desktop Host binary');
  }
}

const rendererWorkspaceRuntimePath = path.join(root, 'desktop/src/agent-workspace/use-agent-workspace-runtime.ts');
if (fs.existsSync(rendererWorkspaceRuntimePath)) {
  const rendererWorkspaceRuntime = fs.readFileSync(rendererWorkspaceRuntimePath, 'utf8');
  const forbiddenAcceptancePatterns = [
    ['request id used as operation-id fallback', /accepted\.operationId\s*\?\?\s*requestId/],
    ['request id accepted as canonical operation id', /operationId\s*===\s*requestId[\s\S]{0,220}(?:adoptOperation|claimOperation)/],
  ];
  for (const [label, pattern] of forbiddenAcceptancePatterns) {
    if (pattern.test(rendererWorkspaceRuntime)) {
      fail(`renderer must wait for a distinct canonical Coordinator operation id; found ${label}`);
    }
  }
}

const rendererShellPath = path.join(root, 'desktop/src/agent-workspace/agent-root-shell.tsx');
if (fs.existsSync(rendererShellPath)) {
  const rendererShell = fs.readFileSync(rendererShellPath, 'utf8');
  if (/activeBusy\s*=\s*Boolean\([^\n]*(?:requestSnapshot|requestForPeer)/.test(rendererShell)) {
    fail('renderer must not project a pending request as an active/running operation');
  }
  if (/projectAgentSidebarItems\([^\n]*requestSnapshot/.test(rendererShell)) {
    fail('sidebar activity must be driven by canonical operations, not pending request ids');
  }
}

const rendererRuntimePath = path.join(root, 'desktop/src/agent-workspace/agent-runtime-coordinator.ts');
if (fs.existsSync(rendererRuntimePath)) {
  const rendererRuntime = fs.readFileSync(rendererRuntimePath, 'utf8');
  const forbiddenCorrelationPatterns = [
    ['unambiguous runtime fallback helper', /\bunambiguousRuntimeId\b/],
    ['runtime-id inventory fallback helper', /\bknownRuntimeIds\b/],
    ['event operation-id nullish fallback', /event\.operationId\s*\?\?/],
    ['operation id reinterpreted as request id', /peerForRequest\(operationId\)/],
  ];
  for (const [label, pattern] of forbiddenCorrelationPatterns) {
    if (pattern.test(rendererRuntime)) {
      fail(`renderer runtime correlation must fail closed; found ${label}`);
    }
  }
}

if (strict) {
  const forbiddenParallelRoots = [
    'desktop/src',
    'desktop/electron',
    'frontend/apps/web',
    'third_party/mahayana',
  ];
  for (const legacyRoot of forbiddenParallelRoots) {
    if (fs.existsSync(path.join(root, legacyRoot))) {
      fail(`final gate: legacy parallel runtime root still exists: ${legacyRoot}`);
    }
  }
}

if (process.exitCode) process.exit(process.exitCode);
console.log(`[grok-architecture] PASS modules=${manifest.moduleCount} strict=${strict}`);
