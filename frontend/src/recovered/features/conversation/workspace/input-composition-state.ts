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
