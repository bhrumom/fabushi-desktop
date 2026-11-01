export const ACCESS_BLOCKED_FAILURE_CODE = "sand-access-blocked" as const;
export const ACCESS_ONBOARDING_URL = "https://fabushi.ombhrum.com/" as const;

export type SandAccessState =
  | "granted"
  | "unavailable"
  | "paymentRequired"
  | "unknown";
export type SandAccessBlockReason =
  | "none"
  | "teamPrivacyMode"
  | "teamSetupRequired"
  | "teamAccessRequired"
  | "notOffered"
  | "freeTrialAvailable"
  | "paywallIndividual"
  | "paywallTeamMember"
  | "paywallTeamAdmin"
  | "unspecified";

export interface SandAccess {
  readonly state: SandAccessState | "checking";
  readonly reason: SandAccessBlockReason;
}

export const SAND_ACCESS_CHECKING: SandAccess = Object.freeze({
  state: "checking",
  reason: "unspecified",
});
export const SAND_ACCESS_UNKNOWN: SandAccess = Object.freeze({
  state: "unknown",
  reason: "unspecified",
});

const STATES = new Set(["checking", "granted", "unavailable", "paymentRequired", "unknown"]);
const REASONS = new Set([
  "none",
  "teamPrivacyMode",
  "teamSetupRequired",
  "teamAccessRequired",
  "notOffered",
  "freeTrialAvailable",
  "paywallIndividual",
  "paywallTeamMember",
  "paywallTeamAdmin",
  "unspecified",
]);

export function isSandAccess(value: unknown): value is SandAccess {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return false;
  const candidate = value as Record<string, unknown>;
  return typeof candidate.state === "string"
    && typeof candidate.reason === "string"
    && STATES.has(candidate.state)
    && REASONS.has(candidate.reason);
}

export function projectSandAccess(value: unknown): SandAccess {
  return isSandAccess(value) ? value : SAND_ACCESS_UNKNOWN;
}

export interface AccessCoverCopy {
  readonly title: string;
  readonly body: string;
  readonly action: string | null;
}

export function accessNoticeCopy(access: SandAccess): AccessCoverCopy | null {
  if (access.state === "checking" || access.state === "unknown" || access.state === "granted") {
    return null;
  }

  switch (access.reason) {
    case "teamPrivacyMode":
      return {
        title: "Your team's privacy mode blocks Fabushi",
        body: "Fabushi cannot run under the team's legacy privacy mode. Ask a team admin to change that policy.",
        action: "See Details",
      };
    case "teamSetupRequired":
      return {
        title: "Your team has not set up Fabushi yet",
        body: "A team admin must finish setup before members can send messages.",
        action: "See Details",
      };
    case "teamAccessRequired":
      return {
        title: "Your team has not granted this account Fabushi access",
        body: "A team admin can grant access from the team's settings.",
        action: "Request Access",
      };
    case "notOffered":
      return {
        title: "Fabushi is not available for this account",
        body: "There is no setup or purchase path available for this account.",
        action: null,
      };
    case "freeTrialAvailable":
      return {
        title: "Start a Fabushi trial to send messages",
        body: "This account can start a trial now.",
        action: "Start Trial",
      };
    case "paywallIndividual":
      return {
        title: "Fabushi requires an eligible plan",
        body: "Upgrade this account before sending messages.",
        action: "Upgrade",
      };
    case "paywallTeamMember":
      return {
        title: "Fabushi requires an eligible team seat",
        body: "Ask a team admin to move this account to an eligible seat.",
        action: "Request Access",
      };
    case "paywallTeamAdmin":
      return {
        title: "Fabushi requires an eligible team seat",
        body: "Move this account to an eligible seat before sending messages.",
        action: "Manage Seats",
      };
    default:
      break;
  }

  if (access.state === "unavailable") {
    return {
      title: "Fabushi is not available for this account",
      body: "Sending stays disabled until this account is granted access.",
      action: "Check Access",
    };
  }
  if (access.state === "paymentRequired") {
    return {
      title: "Fabushi is not included in this plan",
      body: "Sending stays disabled until the account has access.",
      action: "Check Access",
    };
  }
  return null;
}

export function accessCoverCopy(access: SandAccess): AccessCoverCopy {
  return accessNoticeCopy(access) ?? {
    title: "Fabushi is not available on this account yet",
    body: "Check what this account needs on the web.",
    action: "Check Access",
  };
}

export interface AccessCoverGateInput {
  readonly rosterFailureCode: string | null | undefined;
  readonly hasReachedBox: boolean;
  readonly isShowingRestoredRoster: boolean;
  readonly isComputerRebuildLocked: boolean;
}

export function shouldShowAccessCover(input: AccessCoverGateInput): boolean {
  return input.rosterFailureCode === ACCESS_BLOCKED_FAILURE_CODE
    && !input.hasReachedBox
    && !input.isShowingRestoredRoster
    && !input.isComputerRebuildLocked;
}

export interface SandAccessReader {
  getSandAccessFresh(): Promise<unknown>;
}

export async function readFreshSandAccess(bridge: SandAccessReader): Promise<SandAccess> {
  return projectSandAccess(await bridge.getSandAccessFresh());
}

export interface ExternalUrlBridge {
  openExternal(url: string): Promise<void>;
}

export function openAccessOnboarding(bridge: ExternalUrlBridge): Promise<void> {
  return bridge.openExternal(ACCESS_ONBOARDING_URL);
}
