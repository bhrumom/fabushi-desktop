/**
 * Pure side-effect-free projection helpers for Telegram-fusion forum topics,
 * pinned message banners, and community composer permissions.
 */

export interface CommunityComposerGuardInput {
  readonly conversationKind?: string;
  readonly memberStatus?: string;
  readonly canPostMessages?: boolean;
  readonly canSendMessages?: boolean;
  readonly slowModeSeconds?: number;
  readonly lastSentAtMs?: number;
  readonly nowMs?: number;
  readonly activeTopicClosed?: boolean;
  readonly canManageTopics?: boolean;
}

export type CommunityComposerGuardBannerKind =
  | 'none'
  | 'channel-readonly'
  | 'topic-closed'
  | 'member-restricted'
  | 'slow-mode-cooldown';

export interface CommunityComposerGuardResult {
  readonly canSend: boolean;
  readonly bannerKind: CommunityComposerGuardBannerKind;
  readonly remainingCooldownSeconds: number;
  readonly message: string | null;
}

export interface PinnedMessageEntry {
  readonly id: string;
  readonly content?: string;
  readonly text?: string;
  readonly topicId?: string;
}

export interface PinnedMessageBannerResult {
  readonly activePinId: string | null;
  readonly count: number;
  readonly previewText: string | null;
}

export interface ForumTopicItem {
  readonly id: string;
  readonly title: string;
  readonly pinned?: boolean;
  readonly closed?: boolean;
  readonly hidden?: boolean;
  readonly unreadCount?: number;
}

function isAdministrativeMember(status: string | undefined): boolean {
  return status === 'creator' || status === 'administrator' || status === 'owner';
}

/**
 * Pure resolver evaluating whether the active user may send messages in the current
 * community conversation, channel, or topic, returning the appropriate banner status.
 */
export function resolveCommunityComposerGuard(
  input: CommunityComposerGuardInput,
): CommunityComposerGuardResult {
  const {
    conversationKind,
    memberStatus,
    canPostMessages,
    canSendMessages,
    slowModeSeconds,
    lastSentAtMs,
    nowMs,
    activeTopicClosed,
    canManageTopics,
  } = input;

  const isAdmin = isAdministrativeMember(memberStatus);

  // 1. Channel Read-only Enforcement
  if (conversationKind === 'channel') {
    if (canPostMessages === false) {
      return {
        canSend: false,
        bannerKind: 'channel-readonly',
        remainingCooldownSeconds: 0,
        message: 'This channel is read-only.',
      };
    }
    if (canPostMessages !== true && !isAdmin) {
      return {
        canSend: false,
        bannerKind: 'channel-readonly',
        remainingCooldownSeconds: 0,
        message: 'This channel is read-only.',
      };
    }
  }

  // 2. Member Restrictions Enforcement
  if (memberStatus === 'restricted' || memberStatus === 'kicked' || memberStatus === 'left') {
    return {
      canSend: false,
      bannerKind: 'member-restricted',
      remainingCooldownSeconds: 0,
      message: 'You are restricted from posting in this conversation.',
    };
  }

  if (canSendMessages === false && !isAdmin) {
    return {
      canSend: false,
      bannerKind: 'member-restricted',
      remainingCooldownSeconds: 0,
      message: 'You are restricted from sending messages.',
    };
  }

  // 3. Topic Closed Interception & Administrative Exemption
  if (activeTopicClosed === true) {
    const isTopicManager = canManageTopics === true || isAdmin;
    if (!isTopicManager) {
      return {
        canSend: false,
        bannerKind: 'topic-closed',
        remainingCooldownSeconds: 0,
        message: 'This topic is closed.',
      };
    }
  }

  // 4. Slow Mode Cooldown Calculation
  if (typeof slowModeSeconds === 'number' && slowModeSeconds > 0 && !isAdmin) {
    if (typeof lastSentAtMs === 'number' && lastSentAtMs > 0) {
      const currentNow = typeof nowMs === 'number' ? nowMs : Date.now();
      const elapsedMs = currentNow - lastSentAtMs;
      const cooldownMs = slowModeSeconds * 1000;
      if (elapsedMs < cooldownMs) {
        const remainingCooldownSeconds = Math.max(1, Math.ceil((cooldownMs - elapsedMs) / 1000));
        return {
          canSend: false,
          bannerKind: 'slow-mode-cooldown',
          remainingCooldownSeconds,
          message: `Slow mode is active. Please wait ${remainingCooldownSeconds}s before sending another message.`,
        };
      }
    }
  }

  // 5. Default: No restrictions
  return {
    canSend: true,
    bannerKind: 'none',
    remainingCooldownSeconds: 0,
    message: null,
  };
}

/**
 * Pure resolver for pinned message banner isolation between main view and specific topic.
 */
export function resolvePinnedMessageBanner(
  pins: readonly string[],
  entries: readonly PinnedMessageEntry[],
  activeTopicId?: string | null,
): PinnedMessageBannerResult {
  if (!pins || pins.length === 0) {
    return { activePinId: null, count: 0, previewText: null };
  }

  const entryMap = new Map<string, PinnedMessageEntry>();
  for (const entry of entries) {
    entryMap.set(entry.id, entry);
  }

  const isMainScope = activeTopicId == null || activeTopicId === '';
  const scopedPinIds: string[] = [];
  const seenPinIds = new Set<string>();

  for (const pinId of pins) {
    if (seenPinIds.has(pinId)) continue;
    seenPinIds.add(pinId);

    const entry = entryMap.get(pinId);
    if (!entry) continue;

    const belongsToScope = isMainScope
      ? entry.topicId == null || entry.topicId === ''
      : entry.topicId === activeTopicId;

    if (belongsToScope) {
      scopedPinIds.push(pinId);
    }
  }

  if (scopedPinIds.length === 0) {
    return { activePinId: null, count: 0, previewText: null };
  }

  const activePinId = scopedPinIds[0];
  const activeEntry = entryMap.get(activePinId);
  const rawText = activeEntry?.content ?? activeEntry?.text ?? null;
  const previewText = rawText != null && rawText.trim().length > 0 ? rawText.trim() : null;

  return {
    activePinId,
    count: scopedPinIds.length,
    previewText,
  };
}

/**
 * Selects visible forum topics for the current viewer:
 * - Non-admins cannot see hidden topics.
 * - Pinned topics are sorted first, preserving relative creation/insertion order.
 */
export function selectVisibleForumTopics<T extends ForumTopicItem>(
  topics: readonly T[],
  canManageTopics?: boolean,
): T[] {
  if (!topics || topics.length === 0) {
    return [];
  }

  const visible = canManageTopics === true
    ? [...topics]
    : topics.filter((t) => t.hidden !== true);

  return visible.sort((a, b) => {
    const aPinned = a.pinned === true ? 1 : 0;
    const bPinned = b.pinned === true ? 1 : 0;
    return bPinned - aPinned;
  });
}
