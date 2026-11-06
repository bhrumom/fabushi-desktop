import React from "react";

export interface PinnedMessageBannerState {
  readonly count: number;
  readonly activePinId: string | null;
  readonly previewText?: string | null;
}

export interface ConversationPinnedBannerProps {
  readonly pinnedBanner: PinnedMessageBannerState;
  readonly onClickPin: (pinId: string) => void;
}

export function ConversationPinnedBanner({
  pinnedBanner,
  onClickPin,
}: ConversationPinnedBannerProps): React.JSX.Element | null {
  if (pinnedBanner.count <= 0 || pinnedBanner.activePinId == null) return null;
  return (
    <div
      className="sand-conversation-pinned-banner"
      data-testid="conversation-pinned-banner"
      role="button"
      tabIndex={0}
      onClick={() => {
        if (pinnedBanner.activePinId) onClickPin(pinnedBanner.activePinId);
      }}
    >
      <span className="sand-pinned-title">
        Pinned Message{pinnedBanner.count > 1 ? ` (${pinnedBanner.count})` : ""}
      </span>
      {pinnedBanner.previewText ? (
        <span className="sand-pinned-preview">{pinnedBanner.previewText}</span>
      ) : null}
    </div>
  );
}
