export interface McpDiscoveryFailedReport {
  readonly errorClass: string;
  readonly elapsedMs: number;
  readonly servedStale: boolean;
}

export interface McpDiscoverySuccessSettlement<T> {
  readonly resolution: T;
}

export interface McpDiscoveryFailureSettlement<T> {
  readonly report: McpDiscoveryFailedReport;
  readonly clearCache: boolean;
  readonly staleTools?: readonly T[];
}

export function settleMcpDiscoverySuccess<T>(
  resolution: T,
): McpDiscoverySuccessSettlement<T> {
  return { resolution };
}

export function settleMcpDiscoveryFailure<T>(
  error: unknown,
  elapsedMs: number,
  staleTools: readonly T[] | undefined,
): McpDiscoveryFailureSettlement<T> {
  const errorClass =
    error instanceof Error
      ? error.name.length > 0
        ? error.name
        : "Error"
      : typeof error;
  return {
    report: {
      errorClass,
      elapsedMs,
      servedStale: staleTools !== undefined,
    },
    clearCache: staleTools === undefined,
    ...(staleTools === undefined ? {} : { staleTools }),
  };
}
