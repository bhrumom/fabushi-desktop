export type DesktopCallMediaPermission = "granted" | "denied" | "prompt" | "not-requested";

export interface DesktopCallDisplaySource {
  readonly id: string;
  readonly name: string;
  readonly displayId?: string;
}

export interface DesktopCallMediaPort {
  requestPermissions(input: { readonly audio: boolean; readonly video: boolean }): Promise<{
    readonly microphone: DesktopCallMediaPermission;
    readonly camera: DesktopCallMediaPermission;
  }>;
  listDisplaySources(): Promise<readonly DesktopCallDisplaySource[]>;
}

export interface DesktopCallMediaNative {
  readonly desktopCapturer?: {
    getSources(options: {
      readonly types: readonly ("screen" | "window")[];
      readonly thumbnailSize?: { readonly width: number; readonly height: number };
      readonly fetchWindowIcons?: boolean;
    }): Promise<readonly {
      readonly id?: unknown;
      readonly name?: unknown;
      readonly display_id?: unknown;
    }[]>;
  };
  readonly systemPreferences?: {
    askForMediaAccess?(mediaType: "microphone" | "camera"): Promise<boolean>;
    getMediaAccessStatus?(mediaType: "microphone" | "camera"): string;
  };
}

function normalizedStatus(value: unknown): DesktopCallMediaPermission {
  if (value === "granted") return "granted";
  if (value === "denied" || value === "restricted") return "denied";
  return "prompt";
}

async function requestOne(
  native: DesktopCallMediaNative,
  platform: NodeJS.Platform,
  kind: "microphone" | "camera",
  requested: boolean,
): Promise<DesktopCallMediaPermission> {
  if (!requested) return "not-requested";
  const existing = normalizedStatus(native.systemPreferences?.getMediaAccessStatus?.(kind));
  if (existing === "granted" || existing === "denied") return existing;
  if (platform === "darwin" && native.systemPreferences?.askForMediaAccess != null) {
    return await native.systemPreferences.askForMediaAccess(kind) ? "granted" : "denied";
  }
  // Linux and Windows use Chromium's getUserMedia permission prompt. Main
  // still returns an explicit prompt state so the renderer never treats the
  // absence of the macOS-only API as a denial.
  return "prompt";
}

export function createDesktopCallMediaPort(
  native: DesktopCallMediaNative,
  platform: NodeJS.Platform = process.platform,
): DesktopCallMediaPort {
  return {
    async requestPermissions(input) {
      const [microphone, camera] = await Promise.all([
        requestOne(native, platform, "microphone", input.audio),
        requestOne(native, platform, "camera", input.video),
      ]);
      return { microphone, camera };
    },
    async listDisplaySources() {
      if (native.desktopCapturer?.getSources == null) {
        throw new Error("Electron desktopCapturer is unavailable for screen sharing.");
      }
      const sources = await native.desktopCapturer.getSources({
        types: ["screen", "window"],
        thumbnailSize: { width: 0, height: 0 },
        fetchWindowIcons: false,
      });
      const seen = new Set<string>();
      return sources.flatMap((source) => {
        const id = typeof source.id === "string" ? source.id.trim() : "";
        const name = typeof source.name === "string" ? source.name.trim() : "";
        if (id.length === 0 || name.length === 0 || seen.has(id)) return [];
        seen.add(id);
        const displayId = typeof source.display_id === "string" && source.display_id.trim().length > 0
          ? source.display_id.trim()
          : undefined;
        return [{ id, name, ...(displayId == null ? {} : { displayId }) }];
      });
    },
  };
}
