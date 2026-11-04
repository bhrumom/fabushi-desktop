import React from "react";

export interface CommunityComposerGuardBannerProps {
  readonly composerGuard: {
    readonly bannerKind: string;
    readonly message: string | null;
  };
}

export function CommunityComposerGuardBanner({
  composerGuard,
}: CommunityComposerGuardBannerProps): React.JSX.Element | null {
  if (composerGuard.bannerKind === "none" || !composerGuard.message) return null;
  return (
    <div
      className="sand-community-composer-guard-banner"
      data-testid="community-composer-guard-banner"
      data-banner-kind={composerGuard.bannerKind}
      role="status"
    >
      <span>{composerGuard.message}</span>
    </div>
  );
}
