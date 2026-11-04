import React, { useState, useEffect, useCallback } from "react";
import { Sparkles, Gift, Plus, X } from "lucide-react";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import { TdrpStarsModal, type GiftItem } from "./TdrpStarsModal";

export interface StoryItem {
  readonly id: string;
  readonly authorName: string;
  readonly avatarText: string;
  readonly content: string;
  readonly timestamp: string;
  readonly isSelf?: boolean;
}

export interface TdrpStoriesAndStarsBarProps {
  readonly client?: ProductionCoordinatorClient | null;
}

const INITIAL_STORIES: readonly StoryItem[] = [
  { id: "self", authorName: "我的动态", avatarText: "我", content: "分享今天的生活与思考...", timestamp: "刚刚", isSelf: true },
  { id: "s1", authorName: "智能体头条", avatarText: "智", content: "原生架构升级完成，全场景自动化无缝对接！", timestamp: "10分钟前" },
  { id: "s2", authorName: "全球热点", avatarText: "全", content: "去中心化网络技术融合进入新阶段。", timestamp: "1小时前" },
  { id: "s3", authorName: "法布施动态", avatarText: "法", content: "全球法布施 1.2.78 正式就绪，开启智能协作新篇章！", timestamp: "2小时前" },
];

export const TdrpStoriesAndStarsBar: React.FC<TdrpStoriesAndStarsBarProps> = ({ client }) => {
  const [stories, setStories] = useState<StoryItem[]>(() => [...INITIAL_STORIES]);
  const [activeStory, setActiveStory] = useState<StoryItem | null>(null);
  const [showPostModal, setShowPostModal] = useState(false);
  const [newStoryContent, setNewStoryContent] = useState("");

  // Stars & Gifts state
  const [starsBalance, setStarsBalance] = useState(1280);
  const [showStarsModal, setShowStarsModal] = useState(false);
  const [selectedGift, setSelectedGift] = useState<GiftItem>({
    name: "咖啡",
    cost: 100,
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

  const handlePublishStory = useCallback(() => {
    const text = newStoryContent.trim();
    if (!text) return;
    const newStory: StoryItem = {
      id: `story-${Date.now()}`,
      authorName: "我",
      avatarText: "我",
      content: text,
      timestamp: "刚刚",
      isSelf: true,
    };
    setStories((prev) => [prev[0], newStory, ...prev.slice(1)]);
    setNewStoryContent("");
    setShowPostModal(false);

    if (client != null) {
      client.call("stories.publish", { content: text }).catch(() => {});
    }
  }, [client, newStoryContent]);

  const handleSendGift = useCallback(() => {
    if (starsBalance < selectedGift.cost) {
      setTransferStatus("Stars 余额不足");
      return;
    }
    const nextBalance = starsBalance - selectedGift.cost;
    setStarsBalance(nextBalance);
    setTransferStatus(`已成功送出 ${selectedGift.name}！`);

    if (client != null) {
      client.call("stars.transfer", { amount: selectedGift.cost, gift: selectedGift.name }).catch(() => {});
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
        padding: "8px 12px",
        background: "var(--cursor-bg-chrome, #1a1a1f)",
        borderBottom: "1px solid var(--sand-border-weak, rgba(255, 255, 255, 0.08))",
      }}
    >
      {/* Stories carousel */}
      <div style={{ display: "flex", alignItems: "center", gap: "10px", overflowX: "auto", scrollbarWidth: "none" }}>
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
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              cursor: "pointer",
            }}
          >
            <div
              style={{
                width: "36px",
                height: "36px",
                borderRadius: "50%",
                padding: "2px",
                background: story.isSelf
                  ? "var(--sand-border-weak, #444)"
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
                  fontSize: "13px",
                  fontWeight: 600,
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
                  bottom: "0px",
                  right: "-2px",
                  width: "14px",
                  height: "14px",
                  borderRadius: "50%",
                  background: "var(--sand-fill-accent, #1084fe)",
                  color: "#fff",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  fontSize: "10px",
                  fontWeight: "bold",
                  border: "1.5px solid var(--cursor-bg-chrome, #1a1a1f)",
                }}
              >
                <Plus size={10} strokeWidth={3} />
              </span>
            )}
            <span
              style={{
                fontSize: "10px",
                marginTop: "3px",
                color: "var(--sand-text-muted, #aaa)",
                maxWidth: "42px",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}
            >
              {story.authorName}
            </span>
          </div>
        ))}
      </div>

      {/* Stars Balance trigger */}
      <div style={{ display: "flex", alignItems: "center", gap: "8px", marginLeft: "12px", flexShrink: 0 }}>
        <button
          type="button"
          data-testid="tdrp-stars-balance-button"
          onClick={() => setShowStarsModal(true)}
          style={{
            display: "flex",
            alignItems: "center",
            gap: "5px",
            padding: "4px 8px",
            borderRadius: "14px",
            border: "1px solid rgba(255, 193, 7, 0.4)",
            background: "rgba(255, 193, 7, 0.12)",
            color: "#ffc107",
            cursor: "pointer",
            fontSize: "11px",
            fontWeight: 600,
          }}
        >
          <Sparkles size={13} color="#ffc107" />
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
                  width: "36px",
                  height: "36px",
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

      {/* Publish Story Modal */}
      {showPostModal && (
        <div
          role="dialog"
          aria-modal="true"
          data-testid="tdrp-post-story-modal"
          onClick={() => setShowPostModal(false)}
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: "rgba(0, 0, 0, 0.65)",
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
              padding: "20px",
              borderRadius: "14px",
              width: "300px",
              color: "#fff",
            }}
          >
            <h4 style={{ margin: "0 0 10px 0", fontSize: "14px", fontWeight: 600 }}>发布我的动态</h4>
            <textarea
              data-testid="tdrp-post-story-input"
              rows={3}
              placeholder="分享此时此刻的想法..."
              value={newStoryContent}
              onChange={(e) => setNewStoryContent(e.target.value)}
              style={{
                width: "100%",
                padding: "8px",
                borderRadius: "8px",
                border: "1px solid var(--sand-border-weak, #444)",
                background: "var(--cursor-bg-chrome, #18181c)",
                color: "#fff",
                fontSize: "12px",
                resize: "none",
                marginBottom: "12px",
              }}
            />
            <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
              <button
                type="button"
                onClick={() => setShowPostModal(false)}
                style={{ padding: "5px 12px", borderRadius: "6px", border: "1px solid #555", background: "transparent", color: "#ccc", cursor: "pointer", fontSize: "12px" }}
              >
                取消
              </button>
              <button
                type="button"
                data-testid="tdrp-post-story-confirm"
                onClick={handlePublishStory}
                style={{ padding: "5px 14px", borderRadius: "6px", border: 0, background: "var(--sand-fill-accent, #1084fe)", color: "#fff", fontWeight: 600, cursor: "pointer", fontSize: "12px" }}
              >
                发布
              </button>
            </div>
          </div>
        </div>
      )}

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
