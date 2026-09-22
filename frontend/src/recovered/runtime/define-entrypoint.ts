import type { ComponentType } from "react";

export type EntrypointAvailability =
  | { kind: "available" }
  | { kind: "retained"; reason: string }
  | { kind: "unavailable"; reason: string };

export interface EntrypointContext {
  gates: ReadonlySet<string>;
  hasAgents: boolean;
}

export interface RecoveredFeatureEntrypoint<Params = unknown> {
  availability(context: EntrypointContext): EntrypointAvailability;
  loadView(): Promise<{ default: ComponentType<{ params: Params }> }>;
}

export function defineEntrypoint<Params>(
  entrypoint: RecoveredFeatureEntrypoint<Params>,
): Readonly<RecoveredFeatureEntrypoint<Params>> {
  return Object.freeze(entrypoint);
}

export function alwaysAvailable(): EntrypointAvailability {
  return { kind: "available" };
}
