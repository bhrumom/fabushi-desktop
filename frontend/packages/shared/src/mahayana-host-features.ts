import journeyContract from "../../../../contracts/automation/cross-platform-journeys.json";

export type MahayanaHostJourneyStep =
  | { action: "login"; username: string; password: string }
  | { action: "oauthLogin"; provider: "google" | "apple" | "microsoft" | "github" }
  | { action: "expectReady" }
  | { action: "sendChat"; text: string; expectedReply: string }
  | { action: "installMiniApp"; miniAppId: string }
  | { action: "openMiniApp"; miniAppId: string }
  | {
      action: "approveCapability";
      miniAppId: string;
      capability: string;
      decision: "allow-once" | "deny";
    }
  | { action: "interruptOperation"; label: string }
  | { action: "clearSession" };

export interface MahayanaHostFeature {
  id: MahayanaHostFeatureId;
  label: string;
  steps: ReadonlyArray<MahayanaHostJourneyStep>;
}

interface MahayanaHostJourneyContract {
  schemaVersion: 1;
  features: ReadonlyArray<MahayanaHostFeature>;
}

export const mahayanaHostJourneyContract =
  journeyContract as MahayanaHostJourneyContract;

export const mahayanaHostFeatures = mahayanaHostJourneyContract.features;

export type MahayanaHostFeatureId =
  | "auth.login"
  | "runtime.boot"
  | "chat.send"
  | "marketplace.install"
  | "miniapp.open"
  | "capability.approval"
  | "operation.interrupt"
  | "session.clear";

export type MahayanaHostFeatureState = "pending" | "passed" | "failed";
