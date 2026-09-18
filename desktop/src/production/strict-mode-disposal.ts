export interface StrictModeDisposable {
  dispose(): void;
}

export function createStrictModeDisposalGuard() {
  let current: StrictModeDisposable | null = null;
  let generation = 0;
  return {
    attach(resource: StrictModeDisposable | null | undefined): () => void {
      if (current != null && current !== resource) current.dispose();
      current = resource ?? null;
      const attachedGeneration = ++generation;
      return () => {
        queueMicrotask(() => {
          if (generation !== attachedGeneration || current !== resource) return;
          resource?.dispose();
          current = null;
        });
      };
    }
  };
}
