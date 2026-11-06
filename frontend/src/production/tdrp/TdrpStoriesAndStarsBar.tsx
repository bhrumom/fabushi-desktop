import React, { useState, useEffect, useCallback } from "react";
import { Sparkles, Plus, X } from "lucide-react";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import { TdrpStarsModal, type GiftItem } from "./TdrpStarsModal";
import { TdrpStoryPostModal, type StoryPublishPayload } from "./TdrpStoryPostModal";

export interface StoryItem {
  readonly id: string;
  readonly authorName: string;
  readonly avatarText: string;
  readonly content: string;
  readonly timestamp: string;
  readonly isSelf?: boolean;
  readonly privacy?: "all" | "contacts" | "closeFriends" | "selected";
  readonly expirePeriod?: number;
  readonly pinnedToTop?: boolean;
  readonly noforwards?: boolean;
}

export interface TdrpStoriesAndStarsBarProps {
  readonly client?: ProductionCoordinatorClient | null;
}

const INITIAL_STORIES: readonly StoryItem[] = [
  { id: "self", authorName: "我的动态", avatarText: "我", content: "分享生活与思考...", timestamp: "刚刚", isSelf: true },
  { id: "s1", authorName: "智能体头条", avatarText: "智", content: "原生架构升级完成，自动化无缝对接！", timestamp: "10分钟前" },
  { id: "s2", authorName: "全球热点", avatarText: "全", content: "去中心化网络技术融合进入新阶段。", timestamp: "1小时前" },
  { id: "s3", authorName: "法布施动态", avatarText: "法", content: "全球法布施 1.2.78 正式就绪！", timestamp: "2小时前" },
];

export const TdrpStoriesAndStarsBar: React.FC<TdrpStoriesAndStarsBarProps> = ({ client }) => {
  const [stories, setStories] = useState<StoryItem[]>(() => [...INITIAL_STORIES]);
  const [activeStory, setActiveStory] = useState<StoryItem | null>(null);
  const [showPostModal, setShowPostModal] = useState(false);

  // Stars & Gifts state
  const [starsBalance, setStarsBalance] = useState(1280);
  const [showStarsModal, setShowStarsModal] = useState(false);
  const [selectedGift, setSelectedGift] = useState<GiftItem>({
    name: "咖啡",
    cost: 50,
  });
  const [transferStatus, setTransferStatus] = useState<string | null>(null);

  // Sync stories and stars with coordinator
  useEffect(() => {
    if (client == null) return;
    client
      .call("stories.list", {})
      .then((res: unknown) => {
        if (res && typeof res === "object" && "stories" in res && Array.isArray((res as { stories: unknown[] }).stories)) {
          const fetched = (res as { stories: Array<{ id: string; authorName?: string; content?: string }> }).stories.map(
            (s, idx) => ({
              id: s.id || `story-${idx}`,
              authorName: s.authorName || "用户",
              avatarText: (s.authorName || "用")[0],
              content: s.content || "精彩瞬间",
              timestamp: "最近",
            })
          );
          if (fetched.length > 0) {
            setStories([INITIAL_STORIES[0], ...fetched]);
          }
        }
      })
      .catch(() => {});

    client
      .call("stars.status", {})
      .then((res: unknown) => {
        if (res && typeof res === "object" && "balance" in res && typeof (res as { balance: number }).balance === "number") {
          setStarsBalance((res as { balance: number }).balance);
        }
      })
      .catch(() => {});
  }, [client]);

  const handlePublishStory = useCallback((payload: StoryPublishPayload) => {
    const newStory: StoryItem = {
      id: `story-${Date.now()}`,
      authorName: "我",
      avatarText: "我",
      content: payload.content,
      timestamp: "刚刚",
      isSelf: true,
      privacy: payload.privacy,
      expirePeriod: payload.expirePeriod,
      pinnedToTop: payload.pinnedToTop,
      noforwards: payload.noforwards,
    };
    setStories((prev) => [prev[0], newStory, ...prev.slice(1)]);

    if (client != null) {
      client.call("stories.publish", payload).catch(() => {});
    }
  }, [client]);

  const handleSendGift = useCallback((options: { hideMyName: boolean; convertStars: boolean }) => {
    if (starsBalance < selectedGift.cost) {
      setTransferStatus("Stars 余额不足");
      return;
    }
    const nextBalance = starsBalance - selectedGift.cost;
    setStarsBalance(nextBalance);
    setTransferStatus(`已成功送出 ${selectedGift.name}！`);

    if (client != null) {
      client.call("stars.transfer", {
        amount: selectedGift.cost,
        gift: selectedGift.name,
        hideMyName: options.hideMyName,
        convertStars: options.convertStars,
      }).catch(() => {});
    }

    setTimeout(() => {
      setTransferStatus(null);
    }, 2500);
  }, [client, selectedGift, starsBalance]);

  const handleRechargeStars = useCallback(
    (amount: number) => {
      const nextBalance = starsBalance + amount;
      setStarsBalance(nextBalance);
      setTransferStatus(`充值成功，增加 Stars ${amount}！`);
      setTimeout(() => setTransferStatus(null), 2000);
    },
    [starsBalance]
  );

  return (
    <div
      className="tdrp-stories-stars-bar"
      data-testid="tdrp-stories-stars-bar"
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        height: "36px",
        minHeight: "36px",
        padding: "0 8px",
        background: "var(--sand-surface-backdrop, var(--cursor-bg-chrome, #18181c))",
        borderBottom: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.12))",
        userSelect: "none",
        boxSizing: "border-box",
      }}
    >
      {/* Compact Mini Story Rings */}
      <div style={{ display: "flex", alignItems: "center", gap: "6px", overflowX: "auto", scrollbarWidth: "none" }}>
        {stories.map((story) => (
          <div
            key={story.id}
            data-testid={`tdrp-story-avatar-${story.id}`}
            onClick={() => {
              if (story.isSelf) {
                setShowPostModal(true);
              } else {
                setActiveStory(story);
              }
            }}
            title={story.authorName}
            style={{
              position: "relative",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
            }}
          >
            <div
              style={{
                width: "24px",
                height: "24px",
                borderRadius: "50%",
                padding: "1.5px",
                background: story.isSelf
                  ? "var(--sand-border-weak, #666)"
                  : "linear-gradient(135deg, #1084fe, #00c972)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <div
                style={{
                  width: "100%",
                  height: "100%",
                  borderRadius: "50%",
                  background: "var(--cursor-bg-editor, #242429)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontSize: "11px",
                  fontWeight: 700,
                  color: "#fff",
                }}
              >
                {story.avatarText}
              </div>
            </div>
            {story.isSelf && (
              <span
                style={{
                  position: "absolute",
                  bottom: "-1px",
                  right: "-2px",
                  width: "10px",
                  height: "10px",
                  borderRadius: "50%",
                  background: "#1084fe",
                  color: "#fff",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  border: "1px solid var(--sand-surface-backdrop, #18181c)",
                }}
              >
                <Plus size={8} strokeWidth={3} />
              </span>
            )}
          </div>
        ))}
      </div>

      {/* Mini Stars balance trigger */}
      <div style={{ display: "flex", alignItems: "center", gap: "6px", flexShrink: 0 }}>
        <button
          type="button"
          data-testid="tdrp-stars-balance-button"
          onClick={() => setShowStarsModal(true)}
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: "4px",
            padding: "2px 7px",
            borderRadius: "10px",
            border: "1px solid rgba(255, 193, 7, 0.4)",
            background: "rgba(255, 193, 7, 0.12)",
            color: "#ffc107",
            cursor: "pointer",
            fontSize: "11px",
            fontWeight: 700,
          }}
        >
          <Sparkles size={11} color="#ffc107" />
          <span>{starsBalance.toLocaleString()}</span>
        </button>
      </div>

      {/* View Story Modal */}
      {activeStory && (
        <div
          role="dialog"
          aria-modal="true"
          data-testid="tdrp-story-viewer-modal"
          onClick={() => setActiveStory(null)}
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: "rgba(0, 0, 0, 0.75)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 9999,
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              background: "var(--cursor-bg-editor, #242429)",
              padding: "20px 24px",
              borderRadius: "16px",
              width: "320px",
              color: "#fff",
              boxShadow: "0 10px 30px rgba(0,0,0,0.5)",
            }}
          >
            <div style={{ display: "flex", alignItems: "center", gap: "10px", marginBottom: "12px" }}>
              <div
                style={{
                  width: "32px",
                  height: "32px",
                  borderRadius: "50%",
                  background: "linear-gradient(135deg, #1084fe, #00c972)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontWeight: 600,
                }}
              >
                {activeStory.avatarText}
              </div>
              <div>
                <div style={{ fontSize: "13px", fontWeight: 600 }}>{activeStory.authorName}</div>
                <div style={{ fontSize: "10px", color: "#aaa" }}>{activeStory.timestamp}</div>
              </div>
              <button
                type="button"
                onClick={() => setActiveStory(null)}
                style={{ marginLeft: "auto", background: "none", border: 0, color: "#aaa", cursor: "pointer" }}
              >
                <X size={16} />
              </button>
            </div>
            <p style={{ fontSize: "13px", lineHeight: "1.5", margin: "14px 0" }}>{activeStory.content}</p>
          </div>
        </div>
      )}

      {/* Story Publisher Modal */}
      <TdrpStoryPostModal
        isOpen={showPostModal}
        onClose={() => setShowPostModal(false)}
        onPublish={handlePublishStory}
      />

      {/* Stars & Gifts Modal */}
      <TdrpStarsModal
        isOpen={showStarsModal}
        onClose={() => setShowStarsModal(false)}
        starsBalance={starsBalance}
        selectedGift={selectedGift}
        onSelectGift={setSelectedGift}
        onRechargeStars={handleRechargeStars}
        onSendGift={handleSendGift}
        transferStatus={transferStatus}
      />
    </div>
  );
};
