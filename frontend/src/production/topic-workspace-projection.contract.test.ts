import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  resolveCommunityComposerGuard,
  resolvePinnedMessageBanner,
  selectVisibleForumTopics,
  type ForumTopicItem,
  type PinnedMessageEntry,
} from './topic-workspace-projection.ts';

test('FMEP-SLICE-01 S2/S3 - Channel read-only guard intercepts regular members and permits administrators/posters', () => {
  // Regular member without post permission in a channel is intercepted
  const memberInChannel = resolveCommunityComposerGuard({
    conversationKind: 'channel',
    memberStatus: 'member',
    canPostMessages: false,
  });
  assert.equal(memberInChannel.canSend, false);
  assert.equal(memberInChannel.bannerKind, 'channel-readonly');
  assert.equal(memberInChannel.remainingCooldownSeconds, 0);
  assert.match(memberInChannel.message ?? '', /read-only/i);

  // Default unspecified member in channel without explicit post permission is also read-only
  const defaultMember = resolveCommunityComposerGuard({
    conversationKind: 'channel',
  });
  assert.equal(defaultMember.canSend, false);
  assert.equal(defaultMember.bannerKind, 'channel-readonly');

  // Administrator in channel is exempt and can post
  const adminInChannel = resolveCommunityComposerGuard({
    conversationKind: 'channel',
    memberStatus: 'administrator',
  });
  assert.equal(adminInChannel.canSend, true);
  assert.equal(adminInChannel.bannerKind, 'none');
  assert.equal(adminInChannel.message, null);

  // Creator / Owner in channel can post
  const creatorInChannel = resolveCommunityComposerGuard({
    conversationKind: 'channel',
    memberStatus: 'creator',
  });
  assert.equal(creatorInChannel.canSend, true);
  assert.equal(creatorInChannel.bannerKind, 'none');

  // Member with explicit canPostMessages permission can post
  const memberWithPostRight = resolveCommunityComposerGuard({
    conversationKind: 'channel',
    memberStatus: 'member',
    canPostMessages: true,
  });
  assert.equal(memberWithPostRight.canSend, true);
  assert.equal(memberWithPostRight.bannerKind, 'none');

  // Non-channel conversation (e.g., standard group) does not trigger channel-readonly
  const normalGroup = resolveCommunityComposerGuard({
    conversationKind: 'group',
    canPostMessages: false,
  });
  assert.equal(normalGroup.canSend, true);
  assert.equal(normalGroup.bannerKind, 'none');
});

test('FMEP-SLICE-01 S2/S3 - Member restrictions guard blocks restricted users and respects admin exemption', () => {
  // Member with restricted status
  const restrictedMember = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'restricted',
  });
  assert.equal(restrictedMember.canSend, false);
  assert.equal(restrictedMember.bannerKind, 'member-restricted');

  // Member with canSendMessages = false
  const blockedSend = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'member',
    canSendMessages: false,
  });
  assert.equal(blockedSend.canSend, false);
  assert.equal(blockedSend.bannerKind, 'member-restricted');

  // Administrator is not blocked by default group canSendMessages=false
  const adminWithGroupRestriction = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'administrator',
    canSendMessages: false,
  });
  assert.equal(adminWithGroupRestriction.canSend, true);
  assert.equal(adminWithGroupRestriction.bannerKind, 'none');
});

test('FMEP-SLICE-01 S2/S3 - Closed Topic intercepts ordinary members while exempting administrators and topic managers', () => {
  // Ordinary member attempting to send in closed topic
  const ordinaryInClosedTopic = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'member',
    activeTopicClosed: true,
    canManageTopics: false,
  });
  assert.equal(ordinaryInClosedTopic.canSend, false);
  assert.equal(ordinaryInClosedTopic.bannerKind, 'topic-closed');
  assert.equal(ordinaryInClosedTopic.remainingCooldownSeconds, 0);
  assert.match(ordinaryInClosedTopic.message ?? '', /closed/i);

  // Administrator is exempt in closed topic
  const adminInClosedTopic = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'administrator',
    activeTopicClosed: true,
    canManageTopics: false,
  });
  assert.equal(adminInClosedTopic.canSend, true);
  assert.equal(adminInClosedTopic.bannerKind, 'none');

  // Owner / Creator is exempt in closed topic
  const ownerInClosedTopic = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'creator',
    activeTopicClosed: true,
  });
  assert.equal(ownerInClosedTopic.canSend, true);
  assert.equal(ownerInClosedTopic.bannerKind, 'none');

  // User with canManageTopics = true is exempt even if regular member
  const managerInClosedTopic = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'member',
    activeTopicClosed: true,
    canManageTopics: true,
  });
  assert.equal(managerInClosedTopic.canSend, true);
  assert.equal(managerInClosedTopic.bannerKind, 'none');

  // Open topic allows all members to send
  const openTopic = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'member',
    activeTopicClosed: false,
  });
  assert.equal(openTopic.canSend, true);
  assert.equal(openTopic.bannerKind, 'none');
});

test('FMEP-SLICE-01 S2/S3 - Slow Mode accurately calculates remaining cooldown seconds and exempts administrators', () => {
  const nowMs = 1_700_000_000_000;
  const lastSentAtMs = nowMs - 4_000; // Sent 4 seconds ago
  const slowModeSeconds = 10; // 10-second cooldown -> 6 seconds remaining

  // Regular member in cooldown
  const memberInCooldown = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'member',
    slowModeSeconds,
    lastSentAtMs,
    nowMs,
  });
  assert.equal(memberInCooldown.canSend, false);
  assert.equal(memberInCooldown.bannerKind, 'slow-mode-cooldown');
  assert.equal(memberInCooldown.remainingCooldownSeconds, 6);
  assert.match(memberInCooldown.message ?? '', /6s/);

  // Cooldown expired (sent 11 seconds ago with 10s cooldown)
  const expiredCooldown = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'member',
    slowModeSeconds: 10,
    lastSentAtMs: nowMs - 11_000,
    nowMs,
  });
  assert.equal(expiredCooldown.canSend, true);
  assert.equal(expiredCooldown.bannerKind, 'none');
  assert.equal(expiredCooldown.remainingCooldownSeconds, 0);

  // Administrator is exempt from slow mode cooldown
  const adminExemptFromSlowMode = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'administrator',
    slowModeSeconds: 60,
    lastSentAtMs: nowMs - 1_000,
    nowMs,
  });
  assert.equal(adminExemptFromSlowMode.canSend, true);
  assert.equal(adminExemptFromSlowMode.bannerKind, 'none');
  assert.equal(adminExemptFromSlowMode.remainingCooldownSeconds, 0);

  // Creator is exempt from slow mode cooldown
  const creatorExemptFromSlowMode = resolveCommunityComposerGuard({
    conversationKind: 'group',
    memberStatus: 'creator',
    slowModeSeconds: 60,
    lastSentAtMs: nowMs - 1_000,
    nowMs,
  });
  assert.equal(creatorExemptFromSlowMode.canSend, true);
  assert.equal(creatorExemptFromSlowMode.bannerKind, 'none');
  assert.equal(creatorExemptFromSlowMode.remainingCooldownSeconds, 0);
});

test('FMEP-SLICE-01 S2/S3 - Pinned message banner isolates main conversation from topic child scopes', () => {
  const entries: PinnedMessageEntry[] = [
    { id: 'msg-main-pin-1', content: 'Main Room Rules', topicId: undefined },
    { id: 'msg-main-pin-2', content: 'Second Main Announcement', topicId: '' },
    { id: 'msg-topic-dev-1', content: 'Dev Topic Guidelines', topicId: 'topic-dev' },
    { id: 'msg-topic-ops-1', content: 'Ops Incident Response', topicId: 'topic-ops' },
  ];

  const pins = ['msg-main-pin-1', 'msg-topic-dev-1', 'msg-topic-ops-1', 'msg-main-pin-2'];

  // 1. Scoped to Main Conversation View (activeTopicId = null | undefined | "")
  const mainBanner = resolvePinnedMessageBanner(pins, entries, null);
  assert.equal(mainBanner.activePinId, 'msg-main-pin-1');
  assert.equal(mainBanner.count, 2);
  assert.equal(mainBanner.previewText, 'Main Room Rules');

  const mainBannerUndefined = resolvePinnedMessageBanner(pins, entries, undefined);
  assert.equal(mainBannerUndefined.activePinId, 'msg-main-pin-1');
  assert.equal(mainBannerUndefined.count, 2);
  assert.equal(mainBannerUndefined.previewText, 'Main Room Rules');

  // 2. Scoped to Dev Topic View (activeTopicId = "topic-dev")
  const devBanner = resolvePinnedMessageBanner(pins, entries, 'topic-dev');
  assert.equal(devBanner.activePinId, 'msg-topic-dev-1');
  assert.equal(devBanner.count, 1);
  assert.equal(devBanner.previewText, 'Dev Topic Guidelines');

  // 3. Scoped to Ops Topic View (activeTopicId = "topic-ops")
  const opsBanner = resolvePinnedMessageBanner(pins, entries, 'topic-ops');
  assert.equal(opsBanner.activePinId, 'msg-topic-ops-1');
  assert.equal(opsBanner.count, 1);
  assert.equal(opsBanner.previewText, 'Ops Incident Response');

  // 4. Topic without any pins
  const emptyTopicBanner = resolvePinnedMessageBanner(pins, entries, 'topic-empty');
  assert.equal(emptyTopicBanner.activePinId, null);
  assert.equal(emptyTopicBanner.count, 0);
  assert.equal(emptyTopicBanner.previewText, null);

  // 5. Empty pins array
  const noPinsBanner = resolvePinnedMessageBanner([], entries, null);
  assert.equal(noPinsBanner.activePinId, null);
  assert.equal(noPinsBanner.count, 0);
  assert.equal(noPinsBanner.previewText, null);
});

test('FMEP-SLICE-01 S2/S3 - Visible forum topics filters hidden topics for non-admins and prioritizes pinned topics', () => {
  const topics: ForumTopicItem[] = [
    { id: 't-general', title: 'General', pinned: false, closed: false, hidden: false },
    { id: 't-rules', title: 'Rules & Guidelines', pinned: true, closed: true, hidden: false },
    { id: 't-admin-private', title: 'Admin Discussion', pinned: false, closed: false, hidden: true },
    { id: 't-admin-pinned', title: 'Admin Pinned Notice', pinned: true, closed: false, hidden: true },
  ];

  // Non-admin view: hidden topics filtered out, pinned sorted first
  const nonAdminVisible = selectVisibleForumTopics(topics, false);
  assert.deepEqual(
    nonAdminVisible.map((t) => t.id),
    ['t-rules', 't-general'],
  );
  assert.equal(nonAdminVisible.some((t) => t.hidden === true), false);

  // Admin view (canManageTopics = true): hidden topics visible, pinned sorted first
  const adminVisible = selectVisibleForumTopics(topics, true);
  assert.deepEqual(
    adminVisible.map((t) => t.id),
    ['t-rules', 't-admin-pinned', 't-general', 't-admin-private'],
  );
  assert.equal(adminVisible.length, 4);
});
