export interface CanonicalButtonInteractionInput {
  readonly disabled?: boolean;
  readonly pending?: boolean;
}

export interface CanonicalButtonInteractionState {
  readonly ariaBusy: true | undefined;
  readonly disabled: boolean;
  readonly pending: boolean;
}

export function resolveButtonInteractionState(
  input: CanonicalButtonInteractionInput,
): CanonicalButtonInteractionState {
  const pending = input.pending === true;
  return {
    ariaBusy: pending ? true : undefined,
    disabled: input.disabled === true || pending,
    pending,
  };
}
