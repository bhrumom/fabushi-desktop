import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

export function readDesktopProcessCommand(pid: number): string | null {
  try {
    const value = readFileSync(`/proc/${pid}/cmdline`, "utf8");
    if (value.length > 0) return value.replace(/\0/g, " ").trim();
  } catch {}
  try {
    return execFileSync("ps", ["-p", String(pid), "-o", "command="], {
      encoding: "utf8",
      timeout: 2_000,
    }).trim();
  } catch {
    return null;
  }
}

export function isSandHostProcess(pid: number): boolean {
  const command = readDesktopProcessCommand(pid);
  if (command == null) return false;
  return command.includes("host-main")
    || command.includes("mahayana-host")
    || command.includes("mahayana_host");
}
