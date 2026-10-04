export interface ProductionBroadcastWindow {
  readonly webContents: {
    send(channel: string, payload: unknown): void;
  };
}

export interface ProductionBroadcastBrowserWindowSource {
  getAllWindows(): readonly ProductionBroadcastWindow[];
}

export type ProductionBroadcastWindowEligibility = (
  window: ProductionBroadcastWindow,
) => boolean;

/**
 * Electron-main broadcast owner.
 *
 * The frozen Grok behavior remains the default when no eligibility predicate is
 * supplied. Shipping Fabushi production wiring supplies a predicate so
 * renderer-scoped events are delivered only to the trusted main renderer.
 */
export function createProductionWindowBroadcaster(
  browserWindow: ProductionBroadcastBrowserWindowSource,
  isEligible: ProductionBroadcastWindowEligibility = () => true,
): (channel: string, payload: unknown) => void {
  return (channel, payload) => {
    for (const window of browserWindow.getAllWindows()) {
      if (!isEligible(window)) continue;
      window.webContents.send(channel, payload);
    }
  };
}
