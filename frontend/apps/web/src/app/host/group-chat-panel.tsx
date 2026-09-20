"use client";

import React, { useMemo, useState, type FormEvent } from "react";
import type { BotSummary, GroupSummary } from "../../lib/mahayana-host/contracts";
import { parseGroupMentions } from "../../lib/fabushi-runtime/group-mentions";
import { BotMark, type BotMarkColor, type BotMarkShape } from "./bot-mark";
import styles from "./host.module.css";

type GroupDeltaPreview = {
  memberId: string;
  memberName: string;
  text: string;
};

export function GroupAvatarStack({
  group,
  bots,
  size = 32,
}: {
  group: GroupSummary;
  bots: BotSummary[];
  size?: number;
}) {
  const members = group.memberIds
    .map((id) => bots.find((bot) => bot.id === id))
    .filter((bot): bot is BotSummary => Boolean(bot))
    .slice(0, 3);
  return (
    <span className={styles.groupAvatarStack} style={{ "--group-avatar-size": `${size}px` } as React.CSSProperties}>
      {members.map((bot, index) => (
        <span key={bot.id} style={{ zIndex: members.length - index }}>
          <BotMark
            botId={bot.id}
            state="idle"
            size={size}
            shape={bot.avatarShape as BotMarkShape | undefined}
            color={bot.avatarColor as BotMarkColor | undefined}
            label={bot.name}
          />
        </span>
      ))}
    </span>
  );
}

export function GroupChatPanel({
  group,
  bots,
  previews,
  disabled,
  onSend,
  onEdit,
  onDelete,
}: {
  group: GroupSummary;
  bots: BotSummary[];
  previews: Record<string, GroupDeltaPreview>;
  disabled?: boolean;
  onSend: (text: string) => Promise<void> | void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const [input, setInput] = useState("");
  const members = useMemo(
    () => group.memberIds.map((id) => bots.find((bot) => bot.id === id)).filter((bot): bot is BotSummary => Boolean(bot)),
    [bots, group.memberIds],
  );
  const mentionTargets = useMemo(
    () => parseGroupMentions(input, members.map((member) => ({ id: member.id, name: member.name }))),
    [input, members],
  );
  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const text = input.trim();
    if (!text || disabled) return;
    setInput("");
    await onSend(text);
  };
  return (
    <>
      <header className={styles.groupChatHeader}>
        <div className={styles.groupHeaderIdentity}>
          <GroupAvatarStack group={group} bots={bots} size={28} />
          <div>
            <h1>{group.name}</h1>
            <p>{group.description || `${members.length} 个 Bot`}</p>
          </div>
        </div>
        <div className={styles.groupHeaderActions}>
          <button type="button" onClick={onEdit}>编辑</button>
          <button type="button" onClick={onDelete}>删除</button>
        </div>
      </header>
      <div className={styles.groupConversation}>
        <div className={styles.groupWelcome}>
          <GroupAvatarStack group={group} bots={bots} size={48} />
          <h2>{group.name}</h2>
          <p>{members.map((member) => member.name).join("、")}</p>
          <small>@名字 只叫指定 Bot，@everyone 或 @all 让所有成员参与。</small>
        </div>
        <div className={styles.groupMessages} aria-live="polite">
          {group.messages.map((message) => {
            const speaker = message.speaker;
            const member = speaker.kind === "member"
              ? bots.find((bot) => bot.id === speaker.id)
              : undefined;
            return (
              <article key={message.id} data-speaker={message.speaker.kind}>
                {member ? (
                  <BotMark
                    botId={member.id}
                    state="idle"
                    size={28}
                    shape={member.avatarShape as BotMarkShape | undefined}
                    color={member.avatarColor as BotMarkColor | undefined}
                    label={member.name}
                  />
                ) : <span className={styles.groupUserAvatar}>你</span>}
                <div>
                  <strong>{message.speaker.kind === "member" ? message.speaker.name : message.speaker.name || "你"}</strong>
                  <p>{message.content}</p>
                </div>
              </article>
            );
          })}
          {Object.entries(previews).map(([operationId, preview]) => {
            const member = bots.find((bot) => bot.id === preview.memberId);
            return (
              <article key={operationId} className={styles.groupPreview} data-speaker="member">
                {member ? (
                  <BotMark
                    botId={member.id}
                    state="thinking"
                    size={28}
                    shape={member.avatarShape as BotMarkShape | undefined}
                    color={member.avatarColor as BotMarkColor | undefined}
                    label={member.name}
                  />
                ) : null}
                <div><strong>{preview.memberName}</strong><p>{preview.text || "正在思考…"}</p></div>
              </article>
            );
          })}
        </div>
        <form
          className={styles.groupComposer}
          onSubmit={(event) => void submit(event)}
          data-mention-everyone={mentionTargets.isEveryone ? "true" : "false"}
          data-mentioned-members={mentionTargets.memberIds.join(",")}
        >
          <div className={styles.groupMentionBar}>
            <button type="button" onClick={() => setInput((current) => `${current}@everyone `)}>@everyone</button>
            {members.map((member) => (
              <button key={member.id} type="button" onClick={() => setInput((current) => `${current}@${member.name} `)}>@{member.name}</button>
            ))}
          </div>
          <textarea
            value={input}
            rows={1}
            onChange={(event) => setInput(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter" && !event.shiftKey) {
                event.preventDefault();
                event.currentTarget.form?.requestSubmit();
              }
            }}
            placeholder={`发消息到 ${group.name}…`}
            disabled={disabled}
          />
          <button type="submit" disabled={disabled || !input.trim()} aria-label="发送群消息">↑</button>
        </form>
      </div>
    </>
  );
}

export function GroupEditor({
  group,
  bots,
  onSave,
  onClose,
}: {
  group?: GroupSummary;
  bots: BotSummary[];
  onSave: (draft: { name: string; description: string; memberIds: string[] }) => Promise<void> | void;
  onClose: () => void;
}) {
  const [name, setName] = useState(group?.name ?? "");
  const [description, setDescription] = useState(group?.description ?? "");
  const [memberIds, setMemberIds] = useState<string[]>(group?.memberIds ?? []);
  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!name.trim() || memberIds.length === 0) return;
    await onSave({ name: name.trim(), description: description.trim(), memberIds });
  };
  return (
    <div className={styles.backdrop} onMouseDown={onClose}>
      <form className={styles.groupEditorDialog} onSubmit={(event) => void submit(event)} onMouseDown={(event) => event.stopPropagation()}>
        <header>
          <div><small>GROUP CHAT</small><h2>{group ? "编辑群聊" : "新建群聊"}</h2></div>
          <button type="button" onClick={onClose}>×</button>
        </header>
        <label><span>名称</span><input required maxLength={72} value={name} onChange={(event) => setName(event.target.value)} /></label>
        <label><span>描述</span><textarea rows={3} maxLength={2000} value={description} onChange={(event) => setDescription(event.target.value)} /></label>
        <fieldset>
          <legend>成员</legend>
          <div className={styles.groupMemberPicker}>
            {bots.filter((bot) => !bot.hidden).map((bot) => {
              const checked = memberIds.includes(bot.id);
              return (
                <label key={bot.id} data-selected={checked}>
                  <BotMark
                    botId={bot.id}
                    state={checked ? "happy" : "idle"}
                    size={34}
                    shape={bot.avatarShape as BotMarkShape | undefined}
                    color={bot.avatarColor as BotMarkColor | undefined}
                    label={bot.name}
                  />
                  <span><strong>{bot.name}</strong><small>{bot.title || bot.description || bot.id}</small></span>
                  <input
                    type="checkbox"
                    checked={checked}
                    onChange={(event) => setMemberIds((current) => event.target.checked ? [...current, bot.id] : current.filter((id) => id !== bot.id))}
                  />
                </label>
              );
            })}
          </div>
        </fieldset>
        <footer><button type="button" onClick={onClose}>取消</button><button type="submit" disabled={!name.trim() || memberIds.length === 0}>保存</button></footer>
      </form>
    </div>
  );
}
