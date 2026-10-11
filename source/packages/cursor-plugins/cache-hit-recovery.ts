export interface SingleCacheHitRecoveryOptions<T> {
  readonly wasCacheHit: boolean;
  readonly load: () => Promise<T>;
  readonly invalidate: () => Promise<void>;
  readonly reinstall: () => Promise<void>;
  readonly onRecoveryAttempt?: ((error: unknown) => void | Promise<void>) | undefined;
}

/**
 * A completed marketplace cache is derived state. If a cache hit can no longer
 * be loaded, discard that derived state and reinstall it exactly once. Cache
 * misses and second-load failures are propagated without another retry.
 */
export async function loadWithSingleCacheHitRecovery<T>(
  options: SingleCacheHitRecoveryOptions<T>,
): Promise<T> {
  try {
    return await options.load();
  } catch (error) {
    if (!options.wasCacheHit) throw error;
    await options.onRecoveryAttempt?.(error);
    await options.invalidate();
    await options.reinstall();
    return options.load();
  }
}
