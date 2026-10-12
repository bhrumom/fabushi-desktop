import type { ReactionActionScope } from "./reaction-actions.ts";

export type ReactionPickerContent =
  | string
  | ((props: unknown) => unknown);

export type ReactionPickerContentRef =
  | ((value: unknown) => void)
  | { current: unknown };

export interface ReactionPickerExpansionInputs {
  readonly entryId: string;
  readonly myReactions: ReadonlySet<string>;
  readonly onReacted: () => void;
}

export interface ReactionPickerExpansionContentOwner {
  readonly Content: ReactionPickerContent;
  readonly contentRef: ReactionPickerContentRef;
}

export interface ReactionPickerExpansionHandoff
  extends ReactionPickerExpansionInputs, ReactionPickerExpansionContentOwner {
  readonly scope: ReactionActionScope;
  readonly onExpandPicker: () => void;
  readonly onOpenChange: (open: boolean) => void;
}

export type ReactionPickerExpansionHandoffInput = ReactionPickerExpansionHandoff;

function hasContentOwner(
  input: ReactionPickerExpansionContentOwner,
): boolean {
  return (
    typeof input.Content === "function"
    || typeof input.Content === "string"
  ) && (
    typeof input.contentRef === "function"
    || (
      typeof input.contentRef === "object"
      && input.contentRef !== null
    )
  );
}

export function projectReactionPickerExpansionHandoff(
  input: ReactionPickerExpansionHandoffInput,
): ReactionPickerExpansionHandoff | null {
  if (
    input.scope.agentId == null
    || input.entryId.length === 0
    || typeof input.onReacted !== "function"
    || typeof input.onExpandPicker !== "function"
    || typeof input.onOpenChange !== "function"
    || typeof input.myReactions?.has !== "function"
    || !hasContentOwner(input)
  ) {
    return null;
  }

  return {
    scope: {
      accountSlot: input.scope.accountSlot,
      agentId: input.scope.agentId,
    },
    entryId: input.entryId,
    myReactions: input.myReactions,
    onReacted: input.onReacted,
    Content: input.Content,
    contentRef: input.contentRef,
    onExpandPicker: input.onExpandPicker,
    onOpenChange: input.onOpenChange,
  };
}
