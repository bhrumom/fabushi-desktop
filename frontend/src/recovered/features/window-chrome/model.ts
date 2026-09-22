export type WindowPlatform = "aix" | "android" | "darwin" | "freebsd" | "haiku" | "linux" | "openbsd" | "sunos" | "win32" | "cygwin" | "netbsd";

export const WINDOW_CHROME_METRICS = Object.freeze({
  controlsInset: 140,
  hiddenControlsBlock: 51,
  controlsBlock: 52,
  iconSize: 12,
});

export function scaledWindowChromeDimension(pixels: number): string {
  return `calc(${pixels}px / var(--sand-zoom-factor, 1))`;
}

export function windowChromeBlock(platform: WindowPlatform): string {
  return platform === "win32"
    ? scaledWindowChromeDimension(WINDOW_CHROME_METRICS.hiddenControlsBlock)
    : `${WINDOW_CHROME_METRICS.controlsBlock}px`;
}

export function setWindowChromeVariables(
  platform: WindowPlatform,
  isFullscreen: boolean,
): (() => void) | undefined {
  if (typeof document === "undefined" || platform === "darwin" || isFullscreen) {
    return undefined;
  }

  const style = document.documentElement.style;
  style.setProperty(
    "--sand-window-controls-inset",
    scaledWindowChromeDimension(WINDOW_CHROME_METRICS.controlsInset),
  );
  style.setProperty("--sand-window-controls-block", windowChromeBlock(platform));

  return () => {
    style.removeProperty("--sand-window-controls-inset");
    style.removeProperty("--sand-window-controls-block");
  };
}

export function applyRootShellTheme(resolved: "light" | "dark"): void {
  if (typeof document === "undefined") return;
  document.documentElement.dataset.theme =
    resolved === "dark" ? "cursor-dark" : "cursor-light";
  document.documentElement.style.colorScheme = resolved;
}

export function applyRootShellZoomFactor(factor: number): void {
  if (typeof document === "undefined") return;
  document.documentElement.style.setProperty(
    "--sand-zoom-factor",
    String(Number.isFinite(factor) ? factor : 1),
  );
}

export function shouldRefreshRootShellOnFocus(
  transport: "browser" | "connecting" | "connected" | "down",
  visibility: "hidden" | "visible",
): boolean {
  return transport === "connected" && visibility === "visible";
}
