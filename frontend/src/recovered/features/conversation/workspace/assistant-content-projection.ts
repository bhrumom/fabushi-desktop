export type AssistantProjectionRevision =
  | { readonly mode: "text-prefix"; readonly value: string }
  | { readonly mode: "append-sequence"; readonly values: readonly string[] }
  | { readonly mode: "exact"; readonly value: string };

export interface AssistantProjectionCandidate {
  readonly path: string;
  readonly kind: string;
  readonly structuralIdentity: string;
  readonly revision: AssistantProjectionRevision;
}

export interface AssistantProjectionEntry extends AssistantProjectionCandidate {
  readonly key: string;
}

export interface AssistantProjectionState {
  readonly ownerId: string;
  readonly generation: number;
  readonly streaming: boolean;
  readonly mode: "initial" | "patch" | "replace";
  readonly entries: readonly AssistantProjectionEntry[];
}

export interface ReconcileAssistantProjectionInput {
  readonly ownerId: string;
  readonly streaming: boolean;
  readonly candidates: readonly AssistantProjectionCandidate[];
}

function revisionsEqual(previous: AssistantProjectionRevision, next: AssistantProjectionRevision): boolean {
  if (previous.mode !== next.mode) return false;
  if (previous.mode === "append-sequence" && next.mode === "append-sequence") {
    return previous.values.length === next.values.length
      && previous.values.every((value, index) => value === next.values[index]);
  }
  if (previous.mode === "append-sequence" || next.mode === "append-sequence") return false;
  return previous.value === next.value;
}

function revisionCanGrow(previous: AssistantProjectionRevision, next: AssistantProjectionRevision): boolean {
  if (previous.mode !== next.mode) return false;
  if (previous.mode === "text-prefix" && next.mode === "text-prefix") {
    return next.value.startsWith(previous.value);
  }
  if (previous.mode === "append-sequence" && next.mode === "append-sequence") {
    if (next.values.length < previous.values.length) return false;
    if (previous.values.length === 0) return true;
    const lastPreviousIndex = previous.values.length - 1;
    for (let index = 0; index < lastPreviousIndex; index += 1) {
      if (previous.values[index] !== next.values[index]) return false;
    }
    return (next.values[lastPreviousIndex] ?? "").startsWith(previous.values[lastPreviousIndex] ?? "");
  }
  return false;
}

export function areAssistantProjectionCandidatesCompatible(
  previous: AssistantProjectionCandidate,
  next: AssistantProjectionCandidate,
  allowGrowth: boolean,
): boolean {
  if (previous.path !== next.path
    || previous.kind !== next.kind
    || previous.structuralIdentity !== next.structuralIdentity) return false;
  return revisionsEqual(previous.revision, next.revision)
    || (allowGrowth && revisionCanGrow(previous.revision, next.revision));
}

function createProjectionEntries(
  ownerId: string,
  generation: number,
  candidates: readonly AssistantProjectionCandidate[],
): AssistantProjectionEntry[] {
  return candidates.map((candidate, index) => ({
    ...candidate,
    key: `${ownerId}:projection:${generation}:${candidate.path}:${index}`,
  }));
}

export function reconcileAssistantContentProjection(
  previous: AssistantProjectionState | null,
  input: ReconcileAssistantProjectionInput,
): AssistantProjectionState {
  if (previous == null) {
    return {
      ownerId: input.ownerId,
      generation: 0,
      streaming: input.streaming,
      mode: "initial",
      entries: createProjectionEntries(input.ownerId, 0, input.candidates),
    };
  }

  const allowGrowth = previous.streaming || input.streaming;
  const lengthIsCompatible = input.candidates.length === previous.entries.length
    || (allowGrowth && input.candidates.length > previous.entries.length);
  const canPatch = previous.ownerId === input.ownerId
    && lengthIsCompatible
    && previous.entries.every((entry, index) => {
      const candidate = input.candidates[index];
      return candidate != null && areAssistantProjectionCandidatesCompatible(entry, candidate, allowGrowth);
    });

  if (!canPatch) {
    const generation = previous.generation + 1;
    return {
      ownerId: input.ownerId,
      generation,
      streaming: input.streaming,
      mode: "replace",
      entries: createProjectionEntries(input.ownerId, generation, input.candidates),
    };
  }

  const retainedEntries = previous.entries.map((entry, index) => ({
    ...input.candidates[index]!,
    key: entry.key,
  }));
  const appendedEntries = input.candidates.slice(previous.entries.length).map((candidate, offset) => {
    const index = previous.entries.length + offset;
    return {
      ...candidate,
      key: `${input.ownerId}:projection:${previous.generation}:${candidate.path}:${index}`,
    };
  });
  return {
    ownerId: input.ownerId,
    generation: previous.generation,
    streaming: input.streaming,
    mode: "patch",
    entries: [...retainedEntries, ...appendedEntries],
  };
}
