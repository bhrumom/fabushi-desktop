export class AppAgentSurfaceUnavailableError extends Error {
  readonly code: "app_surface_unavailable";
}

export type AppAgentSurfaceOperation = "status" | "snapshot" | "find" | "action" | "wait" | "assert";

export type AppAgentSurfaceClient = Readonly<{
  discoveryPath: string;
  discovery(): Promise<{ origin: string; token: string; appId: "fabushi.desktop"; pid: number }>;
  call<T = unknown>(
    operation: AppAgentSurfaceOperation,
    input?: Record<string, unknown>,
    options?: { signal?: AbortSignal },
  ): Promise<T>;
  status<T = Record<string, unknown>>(): Promise<T>;
}>;

export function createAppAgentSurfaceClient(options?: {
  discoveryPath?: string;
  fetchImpl?: typeof fetch;
  timeoutMs?: number;
}): AppAgentSurfaceClient;
