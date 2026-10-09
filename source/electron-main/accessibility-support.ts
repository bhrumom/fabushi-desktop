export interface DesktopAccessibilityState {
  readonly screenReader: boolean;
}

let screenReader = false;

export function readDesktopAccessibilityState(): DesktopAccessibilityState {
  return { screenReader };
}

export function setDesktopAccessibilitySupportEnabled(enabled: boolean): DesktopAccessibilityState {
  screenReader = enabled === true;
  return readDesktopAccessibilityState();
}
