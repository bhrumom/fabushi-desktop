import { execFile, spawn } from "node:child_process";
import { promisify } from "node:util";

export const WINDOWS_INSTALLER_SIGNER_ALLOWLIST = ["Anysphere, Inc.", "Anysphere"] as const;
export const POWERSHELL_TIMEOUT_MS = 30_000;
export const WINDOWS_PARENT_EXIT_GRACE_MS = 5_000;
export class SandInstallerSignatureError extends Error {}
const execFileAsync = promisify(execFile);

async function runPowershellDefault(args: readonly string[]): Promise<{ stdout: string }> {
  return await execFileAsync("powershell.exe", [...args], {
    timeout: POWERSHELL_TIMEOUT_MS,
    windowsHide: true,
  });
}

function spawnDetachedDefault(
  command: string,
  args: readonly string[],
  onError?: (error: Error) => void,
): void {
  const child = spawn(command, [...args], { detached: true, stdio: "ignore" });
  child.on("error", (error) => onError?.(error));
  child.unref();
}

function powershellSingleQuoted(value: string): string {
  return `'${value.replace(/'/g, "''")}'`;
}

export function buildWindowsInstallerParentHandoffScript(input: {
  readonly installerPath: string;
  readonly installerArgs: readonly string[];
  readonly parentProcessId: number;
  readonly graceMs?: number;
}): string {
  const installer = powershellSingleQuoted(input.installerPath);
  const args = input.installerArgs.map(powershellSingleQuoted).join(", ");
  const graceMs = input.graceMs ?? WINDOWS_PARENT_EXIT_GRACE_MS;
  return [
    "$ErrorActionPreference = 'Continue'",
    `$parentId = ${Math.trunc(input.parentProcessId)}`,
    `$graceMs = ${Math.max(0, Math.trunc(graceMs))}`,
    "$parentCreated = $null",
    "try { $parent = Get-Process -Id $parentId -ErrorAction Stop; $parentCreated = $parent.StartTime.ToUniversalTime().Ticks } catch {}",
    "if ($null -ne $parentCreated) {",
    "  try { $null = $parent.WaitForExit($graceMs) } catch {}",
    "  try {",
    "    $current = Get-Process -Id $parentId -ErrorAction Stop",
    "    $currentCreated = $current.StartTime.ToUniversalTime().Ticks",
    "    if ($currentCreated -eq $parentCreated) { Stop-Process -Id $parentId -Force -ErrorAction Stop }",
    "  } catch {}",
    "}",
    `Start-Process -FilePath ${installer} -ArgumentList @(${args})`,
  ].join("; ");
}

export class SandWindowsInstaller {
  constructor(private readonly options: {
    readonly quit: () => void;
    readonly log?: (message: string) => void;
    readonly signerAllowlist?: readonly string[];
    readonly runPowershell?: (args: readonly string[]) => Promise<{ stdout: string }>;
    readonly spawnDetached?: (
      command: string,
      args: readonly string[],
      onError?: (error: Error) => void,
    ) => void;
    readonly parentProcessId?: number;
  }) {}

  async verifySignature(installerPath: string): Promise<void> {
    const escapedPath = installerPath.replace(/'/g, "''");
    const script = [
      "$ErrorActionPreference = 'Stop'",
      `$sig = Get-AuthenticodeSignature -LiteralPath '${escapedPath}'`,
      "$cn = if ($sig.SignerCertificate) { $sig.SignerCertificate.GetNameInfo('SimpleName', $false) } else { '' }",
      "[pscustomobject]@{ status = $sig.Status.ToString(); signerCommonName = $cn } | ConvertTo-Json -Compress",
    ].join("; ");
    const { stdout } = await (this.options.runPowershell ?? runPowershellDefault)([
      "-NoProfile",
      "-NonInteractive",
      "-Command",
      script,
    ]);
    let probe: unknown;
    try {
      probe = JSON.parse(stdout.trim());
    } catch {
      throw new SandInstallerSignatureError("unreadable signature probe output");
    }
    if (
      typeof probe !== "object"
      || probe === null
      || typeof (probe as any).status !== "string"
      || typeof (probe as any).signerCommonName !== "string"
    ) {
      throw new SandInstallerSignatureError("unreadable signature probe output");
    }
    if ((probe as any).status !== "Valid") {
      throw new SandInstallerSignatureError(`signature status ${(probe as any).status}`);
    }
    if (!(this.options.signerAllowlist ?? WINDOWS_INSTALLER_SIGNER_ALLOWLIST).includes(
      (probe as any).signerCommonName,
    )) {
      throw new SandInstallerSignatureError(
        `unexpected signer "${(probe as any).signerCommonName}"`,
      );
    }
  }

  installOnQuit(
    installerPath: string,
    options: {
      readonly forceRun: boolean;
      readonly onSpawnError?: (error: unknown) => void;
    },
  ): void {
    const installerArgs = options.forceRun
      ? ["--updated", "/S", "--force-run"]
      : ["--updated", "/S"];
    const parentProcessId = this.options.parentProcessId ?? process.pid;
    const script = buildWindowsInstallerParentHandoffScript({
      installerPath,
      installerArgs,
      parentProcessId,
    });
    this.options.log?.(
      `Scheduling installer after parent process ${parentProcessId} exits: ${installerPath}`,
    );
    try {
      (this.options.spawnDetached ?? spawnDetachedDefault)(
        "powershell.exe",
        ["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", script],
        options.onSpawnError as ((error: Error) => void) | undefined,
      );
    } catch (error) {
      this.options.log?.(
        `Installer handoff spawn failed: ${error instanceof Error ? error.message : String(error)}`,
      );
      options.onSpawnError?.(error);
    }
  }

  quit(): void {
    this.options.quit();
  }
}
