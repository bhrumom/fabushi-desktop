import React, { useState } from "react";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import { SandIcon, SandButton } from "../../recovered/ui/sand-kit-primitives";
import {
  formatTdrpInvoiceMessage,
  formatTdrpPollMessage,
  formatTdrpTodoMessage,
  parseTdrpInteractivePayload,
  type TdrpInvoiceData,
  type TdrpPollData,
  type TdrpTodoData,
} from "./tdrp-interactive-models";

export * from "./tdrp-interactive-models";

export interface TdrpInteractiveMediaCardProps {
  readonly text: string;
  readonly client?: ProductionCoordinatorClient | null;
  readonly userId?: string;
  readonly onUpdateText?: (updatedText: string) => void;
}

export function TdrpInteractiveMediaCard({
  text,
  client,
  userId = "me",
  onUpdateText,
}: TdrpInteractiveMediaCardProps): React.JSX.Element | null {
  const payload = parseTdrpInteractivePayload(text);
  if (!payload) return null;

  const activeClient = client ?? null;

  if (payload.type === "tdrp-poll") {
    return <TdrpPollCard client={activeClient} data={payload} onUpdateText={onUpdateText} userId={userId} />;
  }
  if (payload.type === "tdrp-todo") {
    return <TdrpTodoCard client={activeClient} data={payload} onUpdateText={onUpdateText} userId={userId} />;
  }
  if (payload.type === "tdrp-invoice") {
    return <TdrpInvoiceCard client={activeClient} data={payload} onUpdateText={onUpdateText} />;
  }
  return null;
}

const cardStyle: React.CSSProperties = {
  background: "var(--sand-color-surface-card, rgba(128, 128, 128, 0.05))",
  border: "1px solid var(--sand-color-border-subtle, rgba(128, 128, 128, 0.2))",
  borderRadius: 12,
  padding: "12px 14px",
  minWidth: 260,
  maxWidth: 380,
  fontSize: 13,
  color: "inherit",
};

function TdrpPollCard({
  data,
  client,
  userId,
  onUpdateText,
}: {
  data: TdrpPollData;
  client?: ProductionCoordinatorClient | null;
  userId: string;
  onUpdateText?: (newText: string) => void;
}) {
  const [poll, setPoll] = useState<TdrpPollData>(data);
  const [submitting, setSubmitting] = useState(false);

  const quizKind = poll.kind.type === "Quiz" ? poll.kind : null;
  const isQuiz = quizKind !== null;
  const correctId = quizKind ? quizKind.correct_option_id : null;
  const hasVoted = (poll.selectedOptionIds ?? []).length > 0;
  const canRetract = !poll.isClosed && !isQuiz && !poll.revotingDisabled && hasVoted;

  const handleVote = async (optionId: number) => {
    if (poll.isClosed || submitting) return;
    setSubmitting(true);

    const prevSelected = poll.selectedOptionIds ?? [];
    const isMultiple = poll.kind.type === "Regular" && poll.kind.multiple_answers;
    const nextSelected = isMultiple
      ? prevSelected.includes(optionId)
        ? prevSelected.filter((id) => id !== optionId)
        : [...prevSelected, optionId]
      : [optionId];

    const updatedOptions = poll.options.map((opt) => {
      const wasSelected = prevSelected.includes(opt.id);
      const isNowSelected = nextSelected.includes(opt.id);
      let count = opt.voterCount;
      if (!wasSelected && isNowSelected) count += 1;
      if (wasSelected && !isNowSelected) count = Math.max(0, count - 1);
      return { ...opt, voterCount: count };
    });

    const newTotal = updatedOptions.reduce((acc, o) => acc + o.voterCount, 0);
    const updatedPoll: TdrpPollData = { ...poll, options: updatedOptions, totalVoters: newTotal, selectedOptionIds: nextSelected };
    setPoll(updatedPoll);
    onUpdateText?.(formatTdrpPollMessage(updatedPoll));

    try {
      if (client) {
        await client.call("polls.vote", { pollId: poll.pollId, voterId: userId, optionIds: nextSelected });
      }
    } catch { /* Optimistic UI */ } finally {
      setSubmitting(false);
    }
  };

  const handleRetract = async () => {
    if (!canRetract || submitting) return;
    setSubmitting(true);
    const prevSelected = poll.selectedOptionIds ?? [];
    const updatedOptions = poll.options.map((opt) => ({
      ...opt,
      voterCount: prevSelected.includes(opt.id) ? Math.max(0, opt.voterCount - 1) : opt.voterCount,
    }));
    const newTotal = updatedOptions.reduce((acc, o) => acc + o.voterCount, 0);
    const updatedPoll: TdrpPollData = { ...poll, options: updatedOptions, totalVoters: newTotal, selectedOptionIds: [] };
    setPoll(updatedPoll);
    onUpdateText?.(formatTdrpPollMessage(updatedPoll));

    try {
      if (client) {
        await client.call("polls.retractVote", { pollId: poll.pollId, voterId: userId });
      }
    } catch { /* Optimistic UI */ } finally {
      setSubmitting(false);
    }
  };

  return (
    <div style={cardStyle}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 8, gap: 8 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <SandIcon name="chart-bars" size="sm" />
          <span style={{ fontWeight: 600, fontSize: 14 }}>{poll.question}</span>
        </div>
        <span style={{ fontSize: 10, padding: "2px 6px", borderRadius: 6, background: isQuiz ? "rgba(234, 179, 8, 0.18)" : "rgba(37, 99, 235, 0.15)", color: isQuiz ? "#b45309" : "#2563eb", fontWeight: 600 }}>
          {isQuiz ? "测验" : poll.isClosed ? "已截止" : poll.publicVotes ? "公开投票" : "匿名投票"}
        </span>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 6, marginBottom: 10 }}>
        {poll.options.map((option) => {
          const isSelected = (poll.selectedOptionIds ?? []).includes(option.id);
          const pct = poll.totalVoters > 0 ? Math.round((option.voterCount / poll.totalVoters) * 100) : 0;
          const isCorrectAnswer = isQuiz && hasVoted && option.id === correctId;
          const isWrongAnswer = isQuiz && hasVoted && isSelected && option.id !== correctId;

          return (
            <button
              disabled={poll.isClosed || submitting}
              key={option.id}
              onClick={() => void handleVote(option.id)}
              style={{
                position: "relative",
                overflow: "hidden",
                border: isSelected ? "1.5px solid #2563eb" : isCorrectAnswer ? "1.5px solid #16a34a" : isWrongAnswer ? "1.5px solid #dc2626" : "1px solid rgba(128, 128, 128, 0.2)",
                borderRadius: 8,
                padding: "8px 10px",
                background: "var(--sand-color-surface-base, rgba(128, 128, 128, 0.04))",
                cursor: poll.isClosed ? "default" : "pointer",
                textAlign: "left",
                color: "inherit",
              }}
              type="button"
            >
              <div style={{ position: "absolute", left: 0, top: 0, bottom: 0, width: `${pct}%`, background: isCorrectAnswer ? "rgba(22, 163, 74, 0.18)" : isSelected ? "rgba(37, 99, 235, 0.18)" : "rgba(128, 128, 128, 0.08)", transition: "width 0.3s ease", zIndex: 0 }} />
              <div style={{ position: "relative", zIndex: 1, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <span style={{ display: "flex", alignItems: "center", gap: 6 }}>
                  {isSelected ? <SandIcon name="check" size="sm" /> : null}
                  {option.text}
                </span>
                <span style={{ opacity: 0.8, fontSize: 11, marginLeft: 8 }}>{pct}% ({option.voterCount})</span>
              </div>
            </button>
          );
        })}
      </div>

      {quizKind?.explanation && hasVoted && (
        <div style={{ fontSize: 12, padding: "6px 8px", borderRadius: 6, background: "rgba(234, 179, 8, 0.1)", border: "1px solid rgba(234, 179, 8, 0.25)", marginBottom: 8, opacity: 0.9 }}>
          解析：{quizKind.explanation}
        </div>
      )}

      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: 11, opacity: 0.75 }}>
        <span>{poll.totalVoters} 人参与</span>
        {canRetract && (
          <button onClick={() => void handleRetract()} style={{ background: "none", border: "none", color: "#2563eb", cursor: "pointer", fontSize: 11, padding: 0 }} type="button">
            撤回投票
          </button>
        )}
      </div>
    </div>
  );
}

function TdrpTodoCard({
  data,
  client,
  userId,
  onUpdateText,
}: {
  data: TdrpTodoData;
  client?: ProductionCoordinatorClient | null;
  userId?: string;
  onUpdateText?: (newText: string) => void;
}) {
  const [todo, setTodo] = useState<TdrpTodoData>(data);
  const completedCount = todo.items.filter((it) => it.completed).length;
  const pct = todo.items.length > 0 ? Math.round((completedCount / todo.items.length) * 100) : 0;

  const handleToggle = async (itemId: string) => {
    const nextItems = todo.items.map((it) => (it.itemId === itemId ? { ...it, completed: !it.completed, completedBy: !it.completed ? userId : undefined, completionDateMs: !it.completed ? Date.now() : undefined } : it));
    const updatedTodo: TdrpTodoData = { ...todo, items: nextItems };
    setTodo(updatedTodo);
    onUpdateText?.(formatTdrpTodoMessage(updatedTodo));

    const toggledItem = nextItems.find((it) => it.itemId === itemId);
    if (client && toggledItem) {
      try {
        await client.call("todo.toggleItem", { listId: todo.listId, itemId, completed: toggledItem.completed, completedBy: userId });
      } catch { /* Optimistic UI */ }
    }
  };

  return (
    <div style={cardStyle}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 6 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <SandIcon name="check-square" size="sm" />
          <span style={{ fontWeight: 600 }}>{todo.title}</span>
        </div>
        <span style={{ fontSize: 11, opacity: 0.8 }}>{completedCount}/{todo.items.length} ({pct}%)</span>
      </div>

      <div style={{ height: 4, borderRadius: 2, background: "rgba(128, 128, 128, 0.15)", overflow: "hidden", marginBottom: 10 }}>
        <div style={{ width: `${pct}%`, height: "100%", background: "#16a34a", transition: "width 0.3s ease" }} />
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
        {todo.items.map((item) => (
          <label key={item.itemId} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 8, cursor: "pointer", textDecoration: item.completed ? "line-through" : "none", opacity: item.completed ? 0.6 : 1 }}>
            <span style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <input checked={item.completed} onChange={() => void handleToggle(item.itemId)} style={{ cursor: "pointer" }} type="checkbox" />
              <span>{item.text}</span>
            </span>
            {item.completed && item.completedBy && (
              <span style={{ fontSize: 10, opacity: 0.7 }}>已由 {item.completedBy} 勾选</span>
            )}
          </label>
        ))}
      </div>
    </div>
  );
}

function TdrpInvoiceCard({
  data,
  client,
  onUpdateText,
}: {
  data: TdrpInvoiceData;
  client?: ProductionCoordinatorClient | null;
  onUpdateText?: (newText: string) => void;
}) {
  const [invoice, setInvoice] = useState<TdrpInvoiceData>(data);
  const [paying, setPaying] = useState(false);

  const priceFormatted =
    invoice.currency === "STARS"
      ? `${invoice.totalAmountCents} Stars`
      : `${(invoice.totalAmountCents / 100).toFixed(2)} ${invoice.currency}`;

  const handlePay = async () => {
    if (invoice.isPaid || paying) return;
    setPaying(true);
    const chargeId = `ch_sim_${Date.now()}`;
    const updatedInvoice: TdrpInvoiceData = { ...invoice, isPaid: true, chargeId };
    setInvoice(updatedInvoice);
    onUpdateText?.(formatTdrpInvoiceMessage(updatedInvoice));

    if (client) {
      try {
        await client.call("payments.settlePayment", { invoiceId: invoice.invoiceId, chargeId });
      } catch { /* Optimistic UI */ } finally {
        setPaying(false);
      }
    } else {
      setPaying(false);
    }
  };

  return (
    <div style={cardStyle}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 6 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <SandIcon name="credit-card" size="sm" />
          <span style={{ fontWeight: 600 }}>{invoice.title}</span>
        </div>
        <span style={{ fontWeight: 700, color: "var(--sand-color-primary, #2563eb)", fontSize: 14 }}>{priceFormatted}</span>
      </div>

      {invoice.description && <p style={{ margin: "4px 0 10px 0", fontSize: 12, opacity: 0.8 }}>{invoice.description}</p>}

      {invoice.prices && invoice.prices.length > 0 && (
        <div style={{ margin: "6px 0 10px 0", padding: "6px 8px", borderRadius: 6, background: "rgba(128, 128, 128, 0.06)", fontSize: 11 }}>
          {invoice.prices.map((p, idx) => (
            <div key={idx} style={{ display: "flex", justifyContent: "space-between", opacity: 0.85 }}>
              <span>{p.label}</span>
              <span>{(p.amountCents / 100).toFixed(2)} {invoice.currency}</span>
            </div>
          ))}
        </div>
      )}

      {invoice.isPaid ? (
        <div style={{ display: "inline-flex", alignItems: "center", gap: 6, background: "rgba(22, 163, 74, 0.12)", border: "1px solid rgba(22, 163, 74, 0.3)", color: "#16a34a", padding: "4px 10px", borderRadius: 6, fontSize: 11, fontWeight: 600 }}>
          <SandIcon name="check-circle" size="sm" />
          <span>已结算 · {invoice.chargeId ?? "成功"}</span>
        </div>
      ) : (
        <SandButton
          disabled={paying}
          onClick={() => void handlePay()}
          sentiment="accent"
          size="sm"
          style={{ width: "100%", justifyContent: "center" }}
        >
          {paying ? "结算中..." : `立即支付 ${priceFormatted}`}
        </SandButton>
      )}
    </div>
  );
}
