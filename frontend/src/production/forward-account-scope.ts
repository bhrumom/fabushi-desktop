import { useLayoutEffect, type MutableRefObject } from "react";

// A root teardown must invalidate an outstanding forward/roster read during
// layout unmount, not in a later passive effect that can race with a response.
// The identity owner stays in Electron; this is projection-lifetime fencing.
export function useForwardAccountScopeUnmountFence(
  generationRef: MutableRefObject<number>,
): void {
  useLayoutEffect(() => () => {
    generationRef.current += 1;
  }, [generationRef]);
}
