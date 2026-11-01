export interface ProductionAccountTransitionCleanupDeps {
  /** Sets the root-local departure latch before any account-scoped cleanup. */
  readonly setAccountDeparting: () => void;
  /** Abandons in-flight host-settings persistence and clears its local mirror. */
  readonly onAccountDeparted: () => void;
  /** Reports a best-effort local onboarding-mirror clear failure. */
  readonly reportOnboardingMirrorClearFailure: (error: unknown) => void;
  /** Fences account-scoped connector work before the host-settings clear. */
  readonly noteAccountDeparted: () => void;
  /** Writes the immutable empty account-scoped host-settings snapshot. */
  readonly syncHostSettingsToBox: (
    settings: Readonly<Record<string, unknown>>,
  ) => Promise<unknown>;
  /** Reports a best-effort host-settings clear failure; the local cleanup continues. */
  readonly reportHostSettingsClearFailure: (error: unknown) => void;
  readonly clearAccountScope: () => void;
  readonly clearGatewayDescriptor: () => Promise<void>;
  readonly resetMcpManager: () => Promise<void>;
}

export interface ProductionAccountTransitionCleanup {
  prepareAccountTransition(transition: {
    readonly previousSlot: string | null;
    readonly nextSlot: string | null;
  }): Promise<void>;
}

/**
 * Owns the account-departure cleanup that sits between coordinator account
 * authorization and the next account session.  It deliberately does not
 * create or register coordinator ports: the root supplies the already-owned
 * settings, descriptor-store, MCP, and lifecycle callbacks.
 */
export function createProductionAccountTransitionCleanup(
  deps: ProductionAccountTransitionCleanupDeps,
): ProductionAccountTransitionCleanup {
  const reportBestEffort = (
    reporter: (error: unknown) => void,
    error: unknown,
  ): void => {
    try {
      reporter(error);
    } catch {
      // Failure reporting cannot be allowed to skip account cleanup.
    }
  };
  return {
    async prepareAccountTransition({ previousSlot }): Promise<void> {
      if (previousSlot == null) return;

      deps.setAccountDeparting();
      try {
        deps.onAccountDeparted();
      } catch (error) {
        reportBestEffort(deps.reportOnboardingMirrorClearFailure, error);
      }
      deps.noteAccountDeparted();
      try {
        await deps.syncHostSettingsToBox({
          mcpCustomInstructionsAccountScope: null,
          mcpCustomInstructions: {},
          mcpCustomInstructionsByServerId: {},
          mcpDisabledToolsByServerId: {},
        });
      } catch (error) {
        reportBestEffort(deps.reportHostSettingsClearFailure, error);
      } finally {
        deps.clearAccountScope();
        await deps.clearGatewayDescriptor();
        await deps.resetMcpManager();
      }
    },
  };
}
