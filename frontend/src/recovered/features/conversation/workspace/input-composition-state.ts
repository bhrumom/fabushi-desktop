export interface PromptEditorKeyboardEventLike {
  readonly key: string;
  readonly shiftKey: boolean;
  readonly isComposing: boolean;
}

export type PromptEditorKeyAction = "escape" | "submit" | null;

export function promptEditorKeyAction(
  event: PromptEditorKeyboardEventLike,
  compositionActive: boolean,
): PromptEditorKeyAction {
  if (event.isComposing || compositionActive) return null;
  if (event.key === "Escape") return "escape";
  if (event.key === "Enter" && !event.shiftKey) return "submit";
  return null;
}

export function shouldHoldScopeFenceDuringComposition(compositionScopeKey: string | null): boolean {
  return compositionScopeKey != null;
}


export interface PlatformInputStatusValue {
  readonly kind: "known" | "unknown";
  readonly value?: string;
}

export type PlatformInputFocus = "focused" | "blurred" | "unknown";
export type PlatformInputLifecycle =
  | "idle"
  | "focus"
  | "blur"
  | "composition-start"
  | "composition-update"
  | "composition-end";

export interface NativePlatformInputStatusSnapshot {
  readonly method: PlatformInputStatusValue;
  readonly language: PlatformInputStatusValue;
  readonly width: PlatformInputStatusValue;
  readonly scriptVariant: PlatformInputStatusValue;
  readonly preedit: PlatformInputStatusValue;
  readonly candidate: PlatformInputStatusValue;
  readonly focus: PlatformInputFocus;
  readonly lifecycle: PlatformInputLifecycle | "unknown";
  readonly sampledAt: number;
  readonly source: "native-platform";
}

export interface PlatformInputStatusSnapshot {
  readonly composition: "active" | "idle";
  readonly method: PlatformInputStatusValue;
  readonly language: PlatformInputStatusValue;
  readonly width: PlatformInputStatusValue;
  readonly scriptVariant: PlatformInputStatusValue;
  /** Native IME preedit presence delivered by Chromium composition events. */
  readonly preedit: PlatformInputStatusValue;
  /** Candidate UI/content is deliberately unknown until a native adapter reports it. */
  readonly candidate: PlatformInputStatusValue;
  readonly focus: PlatformInputFocus;
  readonly lifecycle: PlatformInputLifecycle;
  readonly generation: number;
  readonly source: "dom-ime" | "dom-ime+native";
}

const UNKNOWN_PLATFORM_INPUT_VALUE: PlatformInputStatusValue = Object.freeze({ kind: "unknown" });

export interface CompositionInputStatusDetails {
  readonly focus?: PlatformInputFocus;
  readonly lifecycle?: PlatformInputLifecycle;
  readonly generation?: number;
  /** undefined means Chromium has not supplied a preedit signal. */
  readonly preeditPresent?: boolean;
}

export function compositionOnlyInputStatus(
  active: boolean,
  details: CompositionInputStatusDetails = {},
): PlatformInputStatusSnapshot {
  return {
    composition: active ? "active" : "idle",
    method: UNKNOWN_PLATFORM_INPUT_VALUE,
    language: UNKNOWN_PLATFORM_INPUT_VALUE,
    width: UNKNOWN_PLATFORM_INPUT_VALUE,
    scriptVariant: UNKNOWN_PLATFORM_INPUT_VALUE,
    preedit: details.preeditPresent === undefined
      ? UNKNOWN_PLATFORM_INPUT_VALUE
      : { kind: "known", value: details.preeditPresent ? "active" : "empty" },
    candidate: UNKNOWN_PLATFORM_INPUT_VALUE,
    focus: details.focus ?? "unknown",
    lifecycle: details.lifecycle ?? (active ? "composition-start" : "idle"),
    generation: details.generation ?? 0,
    source: "dom-ime",
  };
}

export function mergeNativePlatformInputStatus(
  dom: PlatformInputStatusSnapshot,
  native: NativePlatformInputStatusSnapshot,
): PlatformInputStatusSnapshot {
  const preferNative = (nativeValue: PlatformInputStatusValue, domValue: PlatformInputStatusValue) =>
    nativeValue.kind === "known" ? nativeValue : domValue;
  return {
    ...dom,
    method: preferNative(native.method, dom.method),
    language: preferNative(native.language, dom.language),
    width: preferNative(native.width, dom.width),
    scriptVariant: preferNative(native.scriptVariant, dom.scriptVariant),
    // Chromium owns the editable's live preedit/focus/lifecycle truth. Native
    // probes can add capability state, but cannot overwrite a newer DOM event.
    preedit: dom.preedit,
    candidate: preferNative(native.candidate, dom.candidate),
    focus: dom.focus,
    lifecycle: dom.lifecycle,
    generation: dom.generation,
    source: "dom-ime+native",
  };
}

export interface PlatformInputSampleFence {
  readonly scopeKey: string;
  readonly generation: number;
  readonly requestSerial: number;
}

export function isCurrentPlatformInputSample(
  request: PlatformInputSampleFence,
  current: PlatformInputSampleFence,
): boolean {
  return request.scopeKey === current.scopeKey
    && request.generation === current.generation
    && request.requestSerial === current.requestSerial;
}

export function platformInputStatusText(status: PlatformInputStatusSnapshot): string | null {
  if (status.composition !== "active") return null;
  const unknown = [status.method, status.language, status.width, status.scriptVariant]
    .every((value) => value.kind === "unknown");
  if (unknown) {
    const preedit = status.preedit.kind === "known" ? ` Preedit signal: ${status.preedit.value ?? "present"}.` : "";
    return `Composing text.${preedit} Input method, language, width mode, script variant, and candidate state are not reported by the native platform adapter.`;
  }
  const value = (label: string, state: PlatformInputStatusValue) =>
    state.kind === "known" && state.value ? `${label}: ${state.value}` : `${label}: unknown`;
  return [
    "Composing text.",
    value("Input method", status.method),
    value("Language", status.language),
    value("Width", status.width),
    value("Script variant", status.scriptVariant),
  ].join(" ");
}
