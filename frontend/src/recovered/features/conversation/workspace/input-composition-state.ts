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

export interface PlatformInputStatusSnapshot {
  readonly composition: "active" | "idle";
  readonly method: PlatformInputStatusValue;
  readonly language: PlatformInputStatusValue;
  readonly width: PlatformInputStatusValue;
  readonly scriptVariant: PlatformInputStatusValue;
  readonly source: "composition-only" | "native";
}

const UNKNOWN_PLATFORM_INPUT_VALUE: PlatformInputStatusValue = Object.freeze({ kind: "unknown" });

export function compositionOnlyInputStatus(active: boolean): PlatformInputStatusSnapshot {
  return {
    composition: active ? "active" : "idle",
    method: UNKNOWN_PLATFORM_INPUT_VALUE,
    language: UNKNOWN_PLATFORM_INPUT_VALUE,
    width: UNKNOWN_PLATFORM_INPUT_VALUE,
    scriptVariant: UNKNOWN_PLATFORM_INPUT_VALUE,
    source: "composition-only",
  };
}

export function platformInputStatusText(status: PlatformInputStatusSnapshot): string | null {
  if (status.composition !== "active") return null;
  const unknown = [status.method, status.language, status.width, status.scriptVariant]
    .every((value) => value.kind === "unknown");
  if (unknown) {
    return "Composing text. Input method, language, width mode, and script variant are not reported by the platform.";
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
