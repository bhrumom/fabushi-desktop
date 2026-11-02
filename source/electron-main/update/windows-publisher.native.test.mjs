import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { randomUUID } from "node:crypto";
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test, { after, before } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { promisify } from "node:util";

// EUS-WIN-02/04/06: cases 1–10 exercise the shipping verifier's default
// powershell.exe process, Windows Authenticode API and signed PE bytes in
// Actions. Case 11 runs its emitted script in a real native process after a
// controlled CP437 prelude; it never substitutes a signature API result.
// Every installer launch is captured by a counting port; no fixture EXE runs.
// This ephemeral CI identity is not an authorized Fabushi release publisher.
// API contracts:
// https://learn.microsoft.com/powershell/module/pki/new-selfsignedcertificate?view=windowsserver2025-ps
// https://learn.microsoft.com/powershell/module/microsoft.powershell.utility/add-type?view=powershell-5.1
// https://learn.microsoft.com/powershell/module/microsoft.powershell.security/set-authenticodesignature?view=powershell-5.1
// https://learn.microsoft.com/powershell/module/microsoft.powershell.management/remove-item?view=powershell-5.1

const execFileAsync = promisify(execFile);
const testOptions = { concurrency: false, timeout: 45_000 };
let fixtureRoot;
let fixture;
let SandWindowsInstaller;
let SandInstallerSignatureError;
let resolveWindowsUpdatePublisherPolicy;

// The SHA-1 thumbprint is used only to address the certificate that this test
// created in the Windows store. Publisher authorization always uses RawData's
// SHA-256 plus the exact subject, independently checked during cleanup too.
const certificateFunctions = String.raw`
function Get-FixtureCertificateSha256($Certificate) {
  $hasher = [System.Security.Cryptography.SHA256]::Create()
  try {
    return [System.BitConverter]::ToString($hasher.ComputeHash($Certificate.RawData)).Replace('-', '').ToLowerInvariant()
  } finally {
    $hasher.Dispose()
  }
}
function Remove-FixtureCertificate([string]$Thumbprint, [string]$ExpectedSubject, [string]$ExpectedSha256) {
  if ($Thumbprint -notmatch '^[A-Fa-f0-9]{40}$' -or $ExpectedSha256 -notmatch '^[a-f0-9]{64}$' -or -not $ExpectedSubject.StartsWith('CN=Fabushi CI ONLY Inert Updater Fixture ')) {
    throw 'Invalid CI fixture cleanup identity'
  }
  $removed = @()
  foreach ($storeName in @('TrustedPublisher', 'Root', 'My')) {
    $location = if ($storeName -eq 'My') { 'CurrentUser' } else { 'LocalMachine' }
    $certificatePath = 'Cert:\' + $location + '\' + $storeName + '\' + $Thumbprint
    if (Test-Path -LiteralPath $certificatePath) {
      $owned = Get-Item -LiteralPath $certificatePath -ErrorAction Stop
      if ($owned.Subject -cne $ExpectedSubject -or (Get-FixtureCertificateSha256 $owned) -cne $ExpectedSha256) {
        throw 'Refusing to remove a certificate outside the exact CI fixture identity'
      }
      if ($storeName -eq 'My') {
        Remove-Item -LiteralPath $certificatePath -DeleteKey -Force -ErrorAction Stop
      } else {
        Remove-Item -LiteralPath $certificatePath -Force -ErrorAction Stop
      }
      if (Test-Path -LiteralPath $certificatePath) {
        throw 'CI fixture certificate cleanup did not remove the owned certificate'
      }
      $removed += $storeName
    }
  }
  return $removed
}
`;

async function powershell(script, extraEnv = {}) {
  // Use Windows PowerShell 5.1, matching runPowershellDefault. PowerShell 7.1+
  // does not support Add-Type -OutputType ConsoleApplication.
  return await execFileAsync("powershell.exe", [
    "-NoProfile", "-NonInteractive", "-EncodedCommand",
    Buffer.from(script, "utf16le").toString("base64"),
  ], {
    env: { ...process.env, ...extraEnv },
    windowsHide: true,
    // Bound a stuck native trust-store import before the global hook expires,
    // so its exact last stage remains available as diagnostic evidence.
    timeout: 60_000,
    maxBuffer: 1024 * 1024,
  });
}

const setupScript = String.raw`
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
` + certificateFunctions + String.raw`
$fixtureDirectory = $env:FABUSHI_CI_NATIVE_FIXTURE_DIR
$subject = $env:FABUSHI_CI_NATIVE_FIXTURE_SUBJECT
$manifestPath = Join-Path $fixtureDirectory 'fixture.json'
$unsignedPath = Join-Path $fixtureDirectory 'unsigned-fixture.exe'
$signedPath = Join-Path $fixtureDirectory 'signed-fixture.exe'
$stagePath = Join-Path $fixtureDirectory 'setup-stage.txt'
[System.IO.File]::WriteAllText($stagePath, 'creating', [System.Text.UTF8Encoding]::new($false))
$cert = $null
try {
  $certificateParameters = @{
    Type = 'CodeSigningCert'
    Subject = $subject
    FriendlyName = $subject
    CertStoreLocation = 'Cert:\CurrentUser\My'
    KeyAlgorithm = 'RSA'
    KeyLength = 2048
    KeyUsage = 'DigitalSignature'
    KeyExportPolicy = 'NonExportable'
    HashAlgorithm = 'SHA256'
    NotBefore = (Get-Date).AddMinutes(-5)
    NotAfter = (Get-Date).AddHours(2)
  }
  $cert = New-SelfSignedCertificate @certificateParameters
  [System.IO.File]::WriteAllText($stagePath, 'created', [System.Text.UTF8Encoding]::new($false))
  $manifest = [ordered]@{
    subject = $cert.Subject
    certificateSha256 = (Get-FixtureCertificateSha256 $cert)
    thumbprint = $cert.Thumbprint
    notAfterUtc = $cert.NotAfter.ToUniversalTime().ToString('o')
    unsignedPath = $unsignedPath
    signedPath = $signedPath
    scope = 'CI-only inert fixture; no production publisher authorization or installation acceptance'
  }
  # Persist only nonsecret cleanup identity immediately after creation. No
  # private key, PFX, certificate bytes or trust-store export is written.
  [System.IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Compress), [System.Text.UTF8Encoding]::new($false))
  # CurrentUser Root imports trigger an interactive confirmation on hosted
  # Windows, even with -NonInteractive. Only on an ephemeral administrative
  # GitHub-hosted runner, install the PUBLIC certificate via X509Store into
  # LocalMachine Root/TrustedPublisher; keep private key in CurrentUser My.
  $publicCertificate = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new([byte[]]$cert.RawData)
  try {
    foreach ($storeName in @('Root', 'TrustedPublisher')) {
      [System.IO.File]::WriteAllText($stagePath, ('import:' + $storeName), [System.Text.UTF8Encoding]::new($false))
      $certificatePath = 'Cert:\LocalMachine\' + $storeName + '\' + $cert.Thumbprint
      if (Test-Path -LiteralPath $certificatePath) {
        throw 'CI fixture certificate already exists in the machine trust store'
      }
      $store = [System.Security.Cryptography.X509Certificates.X509Store]::new(
        $storeName, [System.Security.Cryptography.X509Certificates.StoreLocation]::LocalMachine
      )
      try {
        $store.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
        $store.Add($publicCertificate)
      } finally {
        $store.Close()
      }
      $imported = Get-Item -LiteralPath $certificatePath -ErrorAction Stop
      if ($imported.Subject -cne $manifest.subject -or (Get-FixtureCertificateSha256 $imported) -cne $manifest.certificateSha256) {
        throw 'Machine trust-store import did not preserve the exact CI-only public certificate'
      }
    }
  } finally {
    $publicCertificate.Dispose()
  }
  [System.IO.File]::WriteAllText($stagePath, 'trusted', [System.Text.UTF8Encoding]::new($false))
  $source = 'internal static class FabushiCiInertFixture { public static int Main() { return 0; } }'
  Add-Type -TypeDefinition $source -Language CSharp -OutputAssembly $unsignedPath -OutputType ConsoleApplication -ErrorAction Stop | Out-Null
  Copy-Item -LiteralPath $unsignedPath -Destination $signedPath -ErrorAction Stop
  [System.IO.File]::WriteAllText($stagePath, 'signing', [System.Text.UTF8Encoding]::new($false))
  $signed = Set-AuthenticodeSignature -LiteralPath $signedPath -Certificate $cert -HashAlgorithm SHA256 -IncludeChain Signer -ErrorAction Stop
  if ($signed.Status.ToString() -ne 'Valid') {
    throw ('Signing the owned CI fixture failed: ' + $signed.Status.ToString())
  }
  $probe = Get-AuthenticodeSignature -LiteralPath $signedPath -ErrorAction Stop
  if ($probe.Status.ToString() -ne 'Valid' -or $probe.SignatureType.ToString() -ne 'Authenticode' -or $probe.SignerCertificate.Subject -cne $manifest.subject -or (Get-FixtureCertificateSha256 $probe.SignerCertificate) -cne $manifest.certificateSha256) {
    throw 'The actual Windows API did not verify the exact CI fixture certificate'
  }
  [System.IO.File]::WriteAllText($stagePath, 'verified', [System.Text.UTF8Encoding]::new($false))
  $manifest['nativeStatus'] = $probe.Status.ToString()
  $manifest['signatureType'] = $probe.SignatureType.ToString()
  [System.IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Compress), [System.Text.UTF8Encoding]::new($false))
} catch {
  $setupError = $_
  if ($null -ne $cert) {
    Remove-FixtureCertificate $cert.Thumbprint $cert.Subject (Get-FixtureCertificateSha256 $cert) | Out-Null
  }
  throw $setupError
}
`;

before(async () => {
  assert.equal(process.platform, "win32", "native publisher contracts require a Windows Actions runner; do not skip");
  assert.equal(process.env.GITHUB_ACTIONS, "true", "native fixture execution is restricted to GitHub Actions");
  assert.equal(process.env.RUNNER_OS, "Windows", "native fixture requires the Windows runner boundary");
  assert.equal(process.env.RUNNER_ENVIRONMENT, "github-hosted", "native fixture must never alter a self-hosted machine trust store");
  fixtureRoot = await mkdtemp(join(tmpdir(), "fabushi-ci-only-authenticode-"));
  const desktopRequire = createRequire(new URL("../../../desktop/package.json", import.meta.url));
  const { build } = desktopRequire("esbuild");
  const bundlePath = join(fixtureRoot, "win32-installer.mjs");
  await build({
    entryPoints: [join(dirname(fileURLToPath(import.meta.url)), "win32-installer.ts")],
    outfile: bundlePath,
    bundle: true,
    platform: "node",
    format: "esm",
    target: "node22",
    packages: "external",
  });
  ({ SandWindowsInstaller, SandInstallerSignatureError, resolveWindowsUpdatePublisherPolicy } = await import(pathToFileURL(bundlePath).href));
  try {
    await powershell(setupScript, {
      FABUSHI_CI_NATIVE_FIXTURE_DIR: fixtureRoot,
      FABUSHI_CI_NATIVE_FIXTURE_SUBJECT: `CN=Fabushi CI ONLY Inert Updater Fixture 测试签名 ${randomUUID().replace(/-/g, "").slice(0, 16)}`,
    });
  } catch (error) {
    const lastStage = await readFile(join(fixtureRoot, "setup-stage.txt"), "utf8")
      .catch(() => "no-stage-recorded");
    console.error("FABUSHI_WINDOWS_NATIVE_SETUP_FAILURE " + JSON.stringify({
      lastStage, code: error.code ?? null, signal: error.signal ?? null,
      killed: Boolean(error.killed),
    }));
    throw error;
  }
  fixture = JSON.parse(await readFile(join(fixtureRoot, "fixture.json"), "utf8"));
  assert.match(fixture.subject, /^CN=Fabushi CI ONLY Inert Updater Fixture /);
  assert.match(fixture.certificateSha256, /^[a-f0-9]{64}$/);
  assert.equal(fixture.nativeStatus, "Valid");
  assert.equal(fixture.signatureType, "Authenticode");
  assert.equal(resolveWindowsUpdatePublisherPolicy(authorizedPolicy()).disabledReason, null);
  console.log(`FABUSHI_WINDOWS_NATIVE_FIXTURE ${JSON.stringify({
    sourceSha: process.env.GITHUB_SHA,
    subject: fixture.subject,
    certificateSha256: fixture.certificateSha256,
    status: fixture.nativeStatus,
    signatureType: fixture.signatureType,
    scope: fixture.scope,
  })}`);
}, { timeout: 90_000 });

after(async () => {
  if (fixtureRoot === undefined) return;
  let cleanupSucceeded = false;
  try {
    const { stdout } = await powershell(String.raw`
$ErrorActionPreference = 'Stop'
` + certificateFunctions + String.raw`
$manifestPath = Join-Path $env:FABUSHI_CI_NATIVE_FIXTURE_DIR 'fixture.json'
if (Test-Path -LiteralPath $manifestPath) {
  $identity = Get-Content -LiteralPath $manifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
  $removed = @(Remove-FixtureCertificate $identity.thumbprint $identity.subject $identity.certificateSha256)
  [pscustomobject]@{ fixtureCreated = $true; remainingOwnedCertificates = 0; removedStores = $removed; privateKeyDeletionRequested = ($removed -contains 'My') } | ConvertTo-Json -Compress
} else {
  [pscustomobject]@{ fixtureCreated = $false; manifestAbsent = $true } | ConvertTo-Json -Compress
}
`, { FABUSHI_CI_NATIVE_FIXTURE_DIR: fixtureRoot });
    const cleanup = JSON.parse(stdout.trim());
    if (fixture !== undefined) {
      assert.equal(cleanup.fixtureCreated, true);
      assert.equal(cleanup.remainingOwnedCertificates, 0);
      assert.deepEqual([...cleanup.removedStores].sort(), ["My", "Root", "TrustedPublisher"]);
      assert.equal(cleanup.privateKeyDeletionRequested, true);
    }
    console.log(`FABUSHI_WINDOWS_NATIVE_CLEANUP ${JSON.stringify(cleanup)}`);
    cleanupSucceeded = true;
  } finally {
    // This is the exact temporary directory owned by this test file. Generated
    // signed EXEs (which contain the public test certificate) are not uploaded.
    // Preserve the nonsecret exact cleanup identity if native cleanup fails;
    // the hook failure must fail the gate rather than claim trust was removed.
    if (cleanupSucceeded) await rm(fixtureRoot, { recursive: true, force: true });
    else console.error(`CI fixture cleanup failed; exact owned identity remains at ${join(fixtureRoot, "fixture.json")}`);
  }
}, { timeout: 70_000 });

function authorizedPolicy(overrides = {}) {
  return JSON.stringify([{
    subject: fixture.subject,
    certificateSha256: fixture.certificateSha256,
    ...overrides,
  }]);
}

function installerHarness(publisherPolicyJson) {
  const calls = { spawns: [], errors: [], quits: 0 };
  const installer = new SandWindowsInstaller({
    publisherPolicyJson,
    quit: () => { calls.quits += 1; },
    // Deliberately omit runPowershell: every verifier call uses the production
    // default native process implementation. Never execute the installer EXE.
    spawnDetached: (command, args) => { calls.spawns.push({ command, args: [...args] }); },
  });
  const install = (candidate, forceRun = false) => installer.installOnQuit(candidate, {
    forceRun,
    onSpawnError: (error) => { calls.errors.push(error); },
  });
  return { installer, calls, install };
}

async function candidate(name, signed = true) {
  const target = join(fixtureRoot, "cases", name);
  await mkdir(dirname(target), { recursive: true });
  await copyFile(signed ? fixture.signedPath : fixture.unsignedPath, target);
  return target;
}

async function signatureStatus(candidatePath) {
  const { stdout } = await powershell(String.raw`
$ErrorActionPreference = 'Stop'
$signature = Get-AuthenticodeSignature -LiteralPath $env:FABUSHI_CI_NATIVE_PROBE_PATH
[pscustomobject]@{ status = $signature.Status.ToString(); signatureType = $signature.SignatureType.ToString() } | ConvertTo-Json -Compress
`, { FABUSHI_CI_NATIVE_PROBE_PATH: candidatePath });
  return JSON.parse(stdout.trim());
}

function assertNoLaunch(harness, expectedErrors = 1) {
  assert.equal(harness.calls.spawns.length, 0);
  assert.equal(harness.calls.quits, 0);
  assert.equal(harness.calls.errors.length, expectedErrors);
  for (const error of harness.calls.errors) assert.ok(error instanceof SandInstallerSignatureError);
}

test("native Windows: exact CI certificate subject and SHA256 authorize an embedded signed candidate", testOptions, async () => {
  const path = await candidate("authorized.exe");
  const h = installerHarness(authorizedPolicy());
  await h.installer.verifySignature(path);
  h.install(path);
  assert.deepEqual(h.calls.spawns, [{ command: path, args: ["--updated", "/S"] }]);
  assert.equal(h.calls.errors.length, 0);
  assert.equal(h.calls.quits, 0);
});

test("native Windows: an actually trusted signed file cannot supply missing publisher authorization", testOptions, async () => {
  const path = await candidate("unconfigured.exe");
  const h = installerHarness(undefined);
  assert.equal(h.installer.getDisabledReason(), "windows-publisher-unconfigured");
  await assert.rejects(h.installer.verifySignature(path), /publisher authorization is unavailable/);
  h.install(path);
  assertNoLaunch(h);
});

test("native Windows: a valid certificate hash cannot authorize a different subject", testOptions, async () => {
  const path = await candidate("wrong-subject.exe");
  const h = installerHarness(authorizedPolicy({ subject: "CN=Fabushi CI ONLY Wrong Subject" }));
  await assert.rejects(h.installer.verifySignature(path), /publisher is not authorized/);
  h.install(path);
  assertNoLaunch(h);
});

test("native Windows: the exact subject cannot authorize a different certificate SHA256", testOptions, async () => {
  const path = await candidate("wrong-certificate.exe");
  const wrongHash = (fixture.certificateSha256[0] === "0" ? "1" : "0") + fixture.certificateSha256.slice(1);
  const h = installerHarness(authorizedPolicy({ certificateSha256: wrongHash }));
  await assert.rejects(h.installer.verifySignature(path), /publisher is not authorized/);
  h.install(path);
  assertNoLaunch(h);
});

test("native Windows: a real unsigned PE is refused and cannot receive launch permission", testOptions, async () => {
  const path = await candidate("unsigned.exe", false);
  const probe = await signatureStatus(path);
  assert.equal(probe.status, "NotSigned");
  const h = installerHarness(authorizedPolicy());
  await assert.rejects(h.installer.verifySignature(path), SandInstallerSignatureError);
  h.install(path);
  assertNoLaunch(h);
});

test("native Windows: modifying a signed PE section produces HashMismatch and is refused", testOptions, async () => {
  const path = await candidate("tampered.exe");
  const bytes = await readFile(path);
  assert.equal(bytes.readUInt16LE(0), 0x5a4d);
  const pe = bytes.readUInt32LE(0x3c);
  assert.equal(bytes.readUInt32LE(pe), 0x00004550);
  const sectionCount = bytes.readUInt16LE(pe + 6);
  const sectionTable = pe + 24 + bytes.readUInt16LE(pe + 20);
  let changed = false;
  for (let index = 0; index < sectionCount; index += 1) {
    const section = sectionTable + index * 40;
    const size = bytes.readUInt32LE(section + 16);
    const offset = bytes.readUInt32LE(section + 20);
    if (size > 32 && offset > sectionTable && offset + size <= bytes.length) {
      // Modify authenticated section content, never the excluded PE checksum
      // or certificate table/overlay. The fixture remains inert and unlaunched.
      bytes[offset + 16] ^= 1;
      changed = true;
      break;
    }
  }
  assert.equal(changed, true, "generated PE has no signed section available for the negative control");
  await writeFile(path, bytes);
  const probe = await signatureStatus(path);
  assert.equal(probe.status, "HashMismatch");
  const h = installerHarness(authorizedPolicy());
  await assert.rejects(h.installer.verifySignature(path), SandInstallerSignatureError);
  h.install(path);
  assertNoLaunch(h);
});

test("native Windows: replacing the file after native verification prevents the spawn callback", testOptions, async () => {
  const path = await candidate("replaced-after-verification.exe");
  const h = installerHarness(authorizedPolicy());
  await h.installer.verifySignature(path);
  await copyFile(fixture.unsignedPath, path);
  h.install(path, true);
  assertNoLaunch(h);
  assert.match(h.calls.errors[0].message, /changed after signature verification/);
});

test("native Windows: a native verification permits exactly one captured launch", testOptions, async () => {
  const path = await candidate("one-use.exe");
  const h = installerHarness(authorizedPolicy());
  await h.installer.verifySignature(path);
  h.install(path, true);
  h.install(path, true);
  assert.deepEqual(h.calls.spawns, [{ command: path, args: ["--updated", "/S", "--force-run"] }]);
  assert.equal(h.calls.errors.length, 1);
  assert.ok(h.calls.errors[0] instanceof SandInstallerSignatureError);
  assert.equal(h.calls.quits, 0);
});

test("native Windows: failed native reverification revokes the prior receipt even when signed bytes return", testOptions, async () => {
  const path = await candidate("failed-reverification.exe");
  const h = installerHarness(authorizedPolicy());
  await h.installer.verifySignature(path);
  await copyFile(fixture.unsignedPath, path);
  await assert.rejects(h.installer.verifySignature(path), SandInstallerSignatureError);
  await copyFile(fixture.signedPath, path);
  h.install(path);
  assertNoLaunch(h);
  assert.match(h.calls.errors[0].message, /no current publisher verification/);
});

test("native Windows: actual PowerShell literal paths retain spaces, apostrophes and wildcard characters", testOptions, async () => {
  const path = await candidate("space and ' quote [literal]/signed fixture.exe");
  const h = installerHarness(authorizedPolicy());
  await h.installer.verifySignature(path);
  h.install(path);
  assert.deepEqual(h.calls.spawns, [{ command: path, args: ["--updated", "/S"] }]);
  assert.equal(h.calls.errors.length, 0);
  assert.equal(h.calls.quits, 0);
});

test("native Windows: Unicode publisher identity survives a controlled CP437 native prelude and matches strictly", { ...testOptions, timeout: 90_000 }, async () => {
  assert.ok(fixture.subject.includes("测试签名"));
  const path = await candidate("unicode-subject.exe");
  const codePagePath = join(fixtureRoot, "unicode-starting-codepages.txt");
  const nativeProbes = [];
  let spawns = 0;
  let quits = 0;
  const errors = [];
  // Cases 1–10 omit runPowershell and use the default native launcher. This
  // case controls only the real child process's starting encoding: execute
  // the production-generated script verbatim after a CP437 prelude, then
  // return the actual native stdout unchanged. No signature result is faked.
  // Setting an inherited parent console is insufficient: libuv may create a
  // separate hidden console when windowsHide:true redirects all stdio pipes.
  const runPowershell = async (args) => {
    assert.deepEqual(args.slice(0, -1), ["-NoProfile", "-NonInteractive", "-Command"]);
    assert.equal(args.length, 4);
    const prelude = [
      "$ErrorActionPreference = 'Stop'",
      "[Console]::OutputEncoding = [System.Text.Encoding]::GetEncoding(437)",
      "$startingCodePage = [Console]::OutputEncoding.CodePage",
      "if ($startingCodePage -ne 437) { throw 'Could not establish the CP437 native precondition' }",
      "[System.IO.File]::AppendAllText($env:FABUSHI_CI_UNICODE_CODEPAGE_LOG, ($startingCodePage.ToString() + [Environment]::NewLine), [System.Text.UTF8Encoding]::new($false))",
    ].join("; ");
    const result = await execFileAsync("powershell.exe", [
      ...args.slice(0, -1), prelude + "; " + args.at(-1),
    ], {
      env: { ...process.env, FABUSHI_CI_UNICODE_CODEPAGE_LOG: codePagePath },
      windowsHide: true,
      timeout: 30_000,
      maxBuffer: 1024 * 1024,
    });
    const nativeProbe = JSON.parse(result.stdout.trim());
    assert.equal(nativeProbe.status, "Valid");
    assert.equal(nativeProbe.signatureType, "Authenticode");
    assert.equal(nativeProbe.signerSubject, fixture.subject);
    assert.equal(nativeProbe.signerCertificateSha256, fixture.certificateSha256);
    nativeProbes.push(nativeProbe);
    return result;
  };
  const makeInstaller = (subject) => new SandWindowsInstaller({
    publisherPolicyJson: authorizedPolicy({ subject }),
    runPowershell,
    quit: () => { quits += 1; },
    spawnDetached: () => { spawns += 1; },
  });
  const approved = makeInstaller(fixture.subject);
  await approved.verifySignature(path);
  approved.installOnQuit(path, { forceRun: false, onSpawnError: (error) => errors.push(error) });
  assert.equal(spawns, 1);
  assert.equal(errors.length, 0);
  const differentSubject = fixture.subject.replace("测试签名", "测试签明");
  assert.notEqual(differentSubject, fixture.subject);
  const denied = makeInstaller(differentSubject);
  await assert.rejects(denied.verifySignature(path), /publisher is not authorized/);
  denied.installOnQuit(path, { forceRun: false, onSpawnError: (error) => errors.push(error) });
  assert.equal(spawns, 1);
  assert.equal(errors.length, 1);
  assert.ok(errors[0] instanceof SandInstallerSignatureError);
  assert.equal(quits, 0);
  assert.equal(nativeProbes.length, 2);
  const startingCodePages = (await readFile(codePagePath, "utf8")).trim().split(/\r?\n/).map(Number);
  assert.deepEqual(startingCodePages, [437, 437]);
  // Emitted only after both actual CP437 starts, round trips, strict policy
  // comparison and captured-launch assertions pass. Child-local encoding and
  // environment die with each awaited process; the parent is never mutated.
  console.log(`FABUSHI_WINDOWS_NATIVE_UNICODE ${JSON.stringify({
    execution: "real-powershell-with-controlled-encoding-prelude",
    defaultNativeLauncherCases: 10,
    startingCodePages,
    subject: nativeProbes[0].signerSubject,
    certificateSha256: nativeProbes[0].signerCertificateSha256,
    exactUnicodeIdentityAllowed: true,
    differentUnicodeSubjectDenied: true,
    capturedSpawns: spawns,
    executedFixtureExecutables: 0,
  })}`);
});
