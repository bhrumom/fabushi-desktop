import { execFile, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { closeSync, createReadStream, fstatSync, openSync, readSync } from "node:fs";
import { promisify } from "node:util";

export const POWERSHELL_TIMEOUT_MS = 30_000;
export class SandInstallerSignatureError extends Error {}
export type WindowsPublisherDisabledReason = "windows-publisher-unconfigured" | "windows-publisher-invalid";
export interface WindowsUpdatePublisher {
  readonly subject: string;
  readonly certificateSha256: string;
}
export interface WindowsUpdatePublisherPolicy {
  readonly publishers: readonly WindowsUpdatePublisher[];
  readonly disabledReason: WindowsPublisherDisabledReason | null;
}

const SHA256 = /^[a-f0-9]{64}$/i;
const NO_PUBLISHERS: readonly WindowsUpdatePublisher[] = Object.freeze([]);

// The release owner supplies approved certificate identities. Product names,
// common names, downloaded metadata and historical publishers confer no trust.
export function resolveWindowsUpdatePublisherPolicy(raw?: string): WindowsUpdatePublisherPolicy {
  const rejected = (disabledReason: WindowsPublisherDisabledReason) => Object.freeze({ publishers: NO_PUBLISHERS, disabledReason });
  if (raw === undefined || typeof raw === "string" && raw.trim() === "") return rejected("windows-publisher-unconfigured");
  if (typeof raw !== "string" || raw.length > 32_768) return rejected("windows-publisher-invalid");
  let entries: unknown;
  try { entries = JSON.parse(raw); } catch { return rejected("windows-publisher-invalid"); }
  if (!Array.isArray(entries) || entries.length === 0 || entries.length > 16) return rejected("windows-publisher-invalid");
  const publishers: WindowsUpdatePublisher[] = [];
  const pins = new Set<string>();
  for (const entry of entries) {
    if (typeof entry !== "object" || entry === null || Array.isArray(entry)) return rejected("windows-publisher-invalid");
    const value = entry as Record<string, unknown>;
    if (Object.keys(value).length !== 2 || typeof value.subject !== "string" || typeof value.certificateSha256 !== "string") return rejected("windows-publisher-invalid");
    if (value.subject.length === 0 || value.subject.length > 1024 || value.subject !== value.subject.trim() || /[\u0000-\u001f\u007f-\u009f]/.test(value.subject) || !SHA256.test(value.certificateSha256)) return rejected("windows-publisher-invalid");
    const certificateSha256 = value.certificateSha256.toLowerCase();
    if (pins.has(certificateSha256)) return rejected("windows-publisher-invalid");
    pins.add(certificateSha256);
    publishers.push(Object.freeze({ subject: value.subject, certificateSha256 }));
  }
  return Object.freeze({ publishers: Object.freeze(publishers), disabledReason: null });
}

const execFileAsync = promisify(execFile);
async function runPowershellDefault(args: readonly string[]): Promise<{ stdout: string }> {
  return await execFileAsync("powershell.exe", [...args], { timeout: POWERSHELL_TIMEOUT_MS, windowsHide: true });
}
function spawnDetachedDefault(command: string, args: readonly string[], onError?: (error: Error) => void): void {
  const child = spawn(command, [...args], { detached: true, stdio: "ignore" });
  child.on("error", (error) => onError?.(error));
  child.unref();
}
async function installerHash(path: string): Promise<string> {
  const hash = createHash("sha256");
  for await (const bytes of createReadStream(path)) hash.update(bytes);
  return hash.digest("hex");
}
// Quit-time launch remains synchronous so the existing Electron quit/marker
// lifecycle does not exit while asynchronous verification is still pending.
function installerHashAtLaunch(path: string): string {
  const fd = openSync(path, "r");
  try {
    if (!fstatSync(fd).isFile()) throw new SandInstallerSignatureError("installer is not a regular file");
    const hash = createHash("sha256");
    const buffer = Buffer.allocUnsafe(64 * 1024);
    for (;;) {
      const count = readSync(fd, buffer, 0, buffer.length, null);
      if (count === 0) return hash.digest("hex");
      hash.update(buffer.subarray(0, count));
    }
  } finally { closeSync(fd); }
}

export class SandWindowsInstaller {
  private readonly policy: WindowsUpdatePublisherPolicy;
  private readonly verified = new Map<string, string>();
  private readonly probes = new Map<string, symbol>();

  constructor(private readonly options: {
    readonly quit: () => void;
    readonly log?: (message: string) => void;
    readonly publisherPolicyJson?: string;
    readonly runPowershell?: (args: readonly string[]) => Promise<{ stdout: string }>;
    readonly spawnDetached?: (command: string, args: readonly string[], onError?: (error: Error) => void) => void;
  }) { this.policy = resolveWindowsUpdatePublisherPolicy(options.publisherPolicyJson); }

  getDisabledReason(): WindowsPublisherDisabledReason | null { return this.policy.disabledReason; }

  async verifySignature(installerPath: string): Promise<void> {
    this.verified.delete(installerPath);
    if (this.policy.disabledReason !== null) throw new SandInstallerSignatureError("Windows update publisher authorization is unavailable");
    const probeId = Symbol();
    this.probes.set(installerPath, probeId);
    try {
      const before = await installerHash(installerPath);
      const escapedPath = installerPath.replace(/'/g, "''");
      const script = [
        "$ErrorActionPreference = 'Stop'",
        "[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)",
        `$sig = Get-AuthenticodeSignature -LiteralPath '${escapedPath}'`,
        "$subject = ''; $certificateSha256 = ''",
        "if ($sig.SignerCertificate) { $subject = $sig.SignerCertificate.Subject; $hasher = [System.Security.Cryptography.SHA256]::Create(); try { $certificateSha256 = [System.BitConverter]::ToString($hasher.ComputeHash($sig.SignerCertificate.RawData)).Replace('-', '').ToLowerInvariant() } finally { $hasher.Dispose() } }",
        "[pscustomobject]@{ status = $sig.Status.ToString(); signatureType = $sig.SignatureType.ToString(); signerSubject = $subject; signerCertificateSha256 = $certificateSha256 } | ConvertTo-Json -Compress",
      ].join("; ");
      const { stdout } = await (this.options.runPowershell ?? runPowershellDefault)(["-NoProfile", "-NonInteractive", "-Command", script]);
      let probe: unknown;
      try { probe = JSON.parse(stdout.trim()); } catch { throw new SandInstallerSignatureError("unreadable signature probe output"); }
      if (typeof probe !== "object" || probe === null || Array.isArray(probe)) throw new SandInstallerSignatureError("unreadable signature probe output");
      const value = probe as Record<string, unknown>;
      if (typeof value.status !== "string" || typeof value.signatureType !== "string" || typeof value.signerSubject !== "string" || typeof value.signerCertificateSha256 !== "string" || !SHA256.test(value.signerCertificateSha256)) throw new SandInstallerSignatureError("unreadable signature probe output");
      if (value.status !== "Valid" || value.signatureType !== "Authenticode") throw new SandInstallerSignatureError("installer requires a valid embedded Authenticode signature");
      const certificateSha256 = value.signerCertificateSha256.toLowerCase();
      if (!this.policy.publishers.some((publisher) => publisher.subject === value.signerSubject && publisher.certificateSha256 === certificateSha256)) throw new SandInstallerSignatureError("installer publisher is not authorized");
      const after = await installerHash(installerPath);
      if (before !== after) throw new SandInstallerSignatureError("installer changed during signature verification");
      if (this.probes.get(installerPath) !== probeId) throw new SandInstallerSignatureError("installer verification was superseded");
      this.verified.set(installerPath, after);
    } catch (error) {
      if (error instanceof SandInstallerSignatureError) throw error;
      throw new SandInstallerSignatureError("installer signature probe or file read failed");
    } finally {
      if (this.probes.get(installerPath) === probeId) this.probes.delete(installerPath);
    }
  }

  installOnQuit(installerPath: string, options: { readonly forceRun: boolean; readonly onSpawnError?: (error: unknown) => void }): void {
    const expectedHash = this.verified.get(installerPath);
    this.verified.delete(installerPath);
    try {
      if (this.policy.disabledReason !== null || expectedHash === undefined || this.probes.has(installerPath)) throw new SandInstallerSignatureError("installer has no current publisher verification");
      if (installerHashAtLaunch(installerPath) !== expectedHash) throw new SandInstallerSignatureError("installer changed after signature verification");
    } catch (error) {
      const failure = error instanceof SandInstallerSignatureError ? error : new SandInstallerSignatureError("verified installer is unavailable");
      this.options.log?.("Verified Windows installer launch refused");
      options.onSpawnError?.(failure);
      return;
    }
    this.runInstaller(installerPath, options.forceRun ? ["--updated", "/S", "--force-run"] : ["--updated", "/S"], options.onSpawnError);
  }

  quit(): void { this.options.quit(); }
  private runInstaller(installerPath: string, args: readonly string[], onSpawnError?: (error: unknown) => void): void {
    this.options.log?.("Launching verified Windows installer");
    try { (this.options.spawnDetached ?? spawnDetachedDefault)(installerPath, args, onSpawnError); }
    catch (error) { this.options.log?.("Verified Windows installer spawn failed"); onSpawnError?.(error); }
  }
}

