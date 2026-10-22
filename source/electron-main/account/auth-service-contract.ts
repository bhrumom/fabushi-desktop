export type SandAuthStatus =
  | { readonly kind: "logging-in" }
  | { readonly kind: "logged-out"; readonly errorMessage?: string }
  | { readonly kind: "logged-in"; readonly authId?: string; readonly email?: string; readonly expiresAt?: number; readonly displayName?: string; readonly profilePictureUrl?: string; readonly isAnysphereUser?: boolean };

export interface FabushiDeviceAgentSession {
  readonly accessToken: string;
  readonly accessTokenExpiresAt: number;
  readonly deviceId: string;
  readonly sessionId: string;
  readonly username: string;
}

export interface AuthServicePort {
  subscribe(listener: (status: SandAuthStatus) => void): () => void;
  getStatus(): Promise<SandAuthStatus>;
  getValidAccessToken(options?: { readonly backendUrl?: string }): Promise<string>;
  peekAccessToken?(): Promise<string | null>;
  deviceAgentSession?(): Promise<FabushiDeviceAgentSession | null>;
  revokeForAccountRefusal(): Promise<{ readonly kind: "completed"; readonly status: SandAuthStatus } | { readonly kind: "failed"; readonly status: SandAuthStatus; readonly error: unknown }>;
  login(): Promise<SandAuthStatus>;
  cancelLogin(): Promise<SandAuthStatus>;
  logout(): Promise<SandAuthStatus>;
  updateDisplayName(name: string): Promise<SandAuthStatus>;
  devLogin?(options: { readonly tier?: string; readonly email?: string }): Promise<SandAuthStatus>;
}
