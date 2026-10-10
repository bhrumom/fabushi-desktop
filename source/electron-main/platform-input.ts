import { execFile } from "node:child_process";

export interface DesktopPlatformInputValue {
  readonly kind: "known" | "unknown";
  readonly value?: string;
}

export type DesktopPlatformInputFocus = "focused" | "blurred" | "unknown";
export type DesktopPlatformInputLifecycle =
  | "idle"
  | "focus"
  | "blur"
  | "composition-start"
  | "composition-update"
  | "composition-end"
  | "unknown";

export interface DesktopPlatformInputState {
  readonly method: DesktopPlatformInputValue;
  readonly language: DesktopPlatformInputValue;
  readonly width: DesktopPlatformInputValue;
  readonly scriptVariant: DesktopPlatformInputValue;
  readonly preedit: DesktopPlatformInputValue;
  readonly candidate: DesktopPlatformInputValue;
  readonly focus: DesktopPlatformInputFocus;
  readonly lifecycle: DesktopPlatformInputLifecycle;
  readonly sampledAt: number;
  readonly source: "native-platform";
}

export type PlatformInputCommandRunner = (
  command: string,
  args: readonly string[],
  timeoutMs: number,
) => Promise<string>;

const UNKNOWN_VALUE: DesktopPlatformInputValue = Object.freeze({ kind: "unknown" });
const MAX_PLATFORM_INPUT_VALUE_LENGTH = 512;
const PLATFORM_INPUT_PROBE_TIMEOUT_MS = 750;

function unknownState(sampledAt: number): DesktopPlatformInputState {
  return {
    method: UNKNOWN_VALUE,
    language: UNKNOWN_VALUE,
    width: UNKNOWN_VALUE,
    scriptVariant: UNKNOWN_VALUE,
    preedit: UNKNOWN_VALUE,
    candidate: UNKNOWN_VALUE,
    focus: "unknown",
    lifecycle: "unknown",
    sampledAt,
    source: "native-platform",
  };
}

function normalizeProbeValue(value: string): DesktopPlatformInputValue {
  const normalized = value.trim();
  if (
    normalized.length === 0
    || normalized.length > MAX_PLATFORM_INPUT_VALUE_LENGTH
    || normalized.includes("\u0000")
    || normalized.includes("\n")
    || normalized.includes("\r")
  ) return UNKNOWN_VALUE;
  return { kind: "known", value: normalized };
}

export const runPlatformInputCommand: PlatformInputCommandRunner = (
  command,
  args,
  timeoutMs,
) => new Promise((resolve, reject) => {
  execFile(
    command,
    [...args],
    {
      encoding: "utf8",
      timeout: timeoutMs,
      windowsHide: true,
      maxBuffer: 4096,
    },
    (error, stdout) => {
      if (error != null) {
        reject(error);
        return;
      }
      resolve(stdout);
    },
  );
});

async function readFirstKnownMethod(
  probes: readonly { readonly command: string; readonly args: readonly string[] }[],
  runCommand: PlatformInputCommandRunner,
): Promise<DesktopPlatformInputValue> {
  for (const probe of probes) {
    try {
      const value = normalizeProbeValue(
        await runCommand(probe.command, probe.args, PLATFORM_INPUT_PROBE_TIMEOUT_MS),
      );
      if (value.kind === "known") return value;
    } catch {
      // A missing/blocked platform provider is an unknown value, never evidence
      // for a guessed locale, keyboard layout, or alternate input runtime.
    }
  }
  return UNKNOWN_VALUE;
}

export interface ReadDesktopPlatformInputOptions {
  readonly platform?: NodeJS.Platform;
  readonly runCommand?: PlatformInputCommandRunner;
  readonly now?: () => number;
}

export async function readDesktopPlatformInputState(
  options: ReadDesktopPlatformInputOptions = {},
): Promise<DesktopPlatformInputState> {
  const platform = options.platform ?? process.platform;
  const runCommand = options.runCommand ?? runPlatformInputCommand;
  const sampledAt = (options.now ?? Date.now)();
  const state = unknownState(sampledAt);

  let method = UNKNOWN_VALUE;
  if (platform === "darwin") {
    // AppleCurrentKeyboardLayoutInputSourceID is OS-owned current input-source
    // state. Keep the returned identifier opaque; do not infer language,
    // full/half width, script variant, or candidate visibility from its name.
    method = await readFirstKnownMethod([
      {
        command: "/usr/bin/defaults",
        args: ["read", "com.apple.HIToolbox", "AppleCurrentKeyboardLayoutInputSourceID"],
      },
    ], runCommand);
  } else if (platform === "linux") {
    // Query only installed native input frameworks. Absolute paths avoid PATH
    // substitution. Fcitx5 is tried before IBus because both may be installed,
    // while the active one is the only probe expected to return a concrete id.
    method = await readFirstKnownMethod([
      { command: "/usr/bin/fcitx5-remote", args: ["-n"] },
      { command: "/usr/bin/ibus", args: ["engine"] },
    ], runCommand);
  }

  return method.kind === "known" ? { ...state, method } : state;
}
