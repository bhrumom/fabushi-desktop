export interface OverlayBackdropPointerEvent {
  preventDefault(): void;
  stopPropagation(): void;
  stopImmediatePropagation(): void;
}

export function consumeOverlayBackdropPointer(event: OverlayBackdropPointerEvent): void {
  event.preventDefault();
  event.stopPropagation();
  event.stopImmediatePropagation();
}

export function scheduleDeferredOverlayDismiss(onClose: () => void): () => void {
  let active = true;
  queueMicrotask(() => {
    if (!active) return;
    active = false;
    onClose();
  });
  return () => {
    active = false;
  };
}

export function topmostOverlayLayerIndex(zIndexes: readonly number[]): number {
  if (zIndexes.length === 0) return -1;
  let topmost = 0;
  for (let index = 1; index < zIndexes.length; index += 1) {
    if (zIndexes[index] >= zIndexes[topmost]) topmost = index;
  }
  return topmost;
}
