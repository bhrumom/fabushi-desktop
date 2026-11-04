import React, { useState } from "react";
import { getSharedCoordinatorClient, type ProductionCoordinatorClient } from "../coordinator-client";
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

  const activeClient = client ?? getSharedCoordinatorClient();

  if (payload.type === "tdrp-poll") {
    return <TdrpPollCard client={activeClient} data={payload} onUpdateText={onUpdateText} userId={userId} />;
  }
  if (payload.type === "tdrp-todo") {
    return <TdrpTodoCard client={activeClient} data={payload} onUpdateText={onUpdateText} />;
  }
  if (payload.type === "tdrp-invoice") {
    return <TdrpInvoiceCard client={activeClient} data={payload} onUpdateText={onUpdateText} />;
  }
  return null;
}

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
    const updatedPoll: TdrpPollData = {
      ...poll,
      options: updatedOptions,
      totalVoters: newTotal,
      selectedOptionIds: nextSelected,
    };

    setPoll(updatedPoll);
    onUpdateText?.(formatTdrpPollMessage(updatedPoll));

    try {
      if (client) {
        await client.call("polls.vote", { pollId: poll.pollId, voterId: userId, optionIds: nextSelected });
      }
    } catch {
      // Optimistic UI state preserved
    } finally {
      setSubmitting(false);
    }
  };

  const handleRetract = async () => {
    if (poll.isClosed || submitting || !hasVoted) return;
    setSubmitting(true);
    const prevSelected = poll.selectedOptionIds ?? [];
    const updatedOptions = poll.options.map((opt) => ({
      ...opt,
      voterCount: prevSelected.includes(opt.id) ? Math.max(0, opt.voterCount - 1) : opt.voterCount,
    }));
    const newTotal = updatedOptions.reduce((acc, o) => acc + o.voterCount, 0);
    const updatedPoll: TdrpPollData = {
      ...poll,
      options: updatedOptions,
      totalVoters: newTotal,
      selectedOptionIds: [],
    };
    setPoll(updatedPoll);
    onUpdateText?.(formatTdrpPollMessage(updatedPoll));

    try {
      if (client) {
        await client.call("polls.retractVote", { pollId: poll.pollId, voterId: userId });
      }
    } catch {
      // Keep optimistic
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div style={{ background: "var(--fabushi-surface-2, rgba(255, 255, 255, 0.05))", border: "1px solid var(--fabushi-border, rgba(255, 255, 255, 0.12))", borderRadius: 12, padding: "12px 14px", minWidth: 260, maxWidth: 380, fontSize: 13 }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 8, gap: 8 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <SandIcon name="chart-bars" size="sm" />
          <span style={{ fontWeight: 600, fontSize: 14 }}>{poll.question}</span>
        </div>
        <span style={{ fontSize: 10, padding: "2px 6px", borderRadius: 6, background: isQuiz ? "rgba(234, 179, 8, 0.2)" : "rgba(59, 130, 246, 0.2)", color: isQuiz ? "#facc15" : "#60a5fa", fontWeight: 500 }}>
          {isQuiz ? "Quiz" : poll.isClosed ? "Closed" : "Poll"}
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
                border: isSelected ? "1px solid #3b82f6" : isCorrectAnswer ? "1px solid #22c55e" : isWrongAnswer ? "1px solid #ef4444" : "1px solid rgba(255, 255, 255, 0.08)",
                borderRadius: 8,
                padding: "8px 10px",
                background: "rgba(0, 0, 0, 0.15)",
                cursor: poll.isClosed ? "default" : "pointer",
                textAlign: "left",
                color: "inherit",
              }}
              type="button"
            >
              <div style={{ position: "absolute", left: 0, top: 0, bottom: 0, width: `${pct}%`, background: isCorrectAnswer ? "rgba(34, 197, 94, 0.2)" : isSelected ? "rgba(59, 130, 246, 0.2)" : "rgba(255, 255, 255, 0.06)", transition: "width 0.3s ease", zIndex: 0 }} />
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
        <div style={{ fontSize: 11, color: "rgba(255, 255, 255, 0.7)", marginBottom: 8 }}>{quizKind.explanation}</div>
      )}

      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: 11, opacity: 0.7 }}>
        <span>{poll.totalVoters} votes</span>
        {hasVoted && !poll.isClosed && (
          <button onClick={() => void handleRetract()} style={{ background: "none", border: "none", color: "#60a5fa", cursor: "pointer", fontSize: 11, padding: 0 }} type="button">
            Retract Vote
          </button>
        )}
      </div>
    </div>
  );
}

function TdrpTodoCard({
  data,
  client,
  onUpdateText,
}: {
  data: TdrpTodoData;
  client?: ProductionCoordinatorClient | null;
  onUpdateText?: (newText: string) => void;
}) {
  const [todo, setTodo] = useState<TdrpTodoData>(data);
  const completedCount = todo.items.filter((it) => it.completed).length;
  const pct = todo.items.length > 0 ? Math.round((completedCount / todo.items.length) * 100) : 0;

  const handleToggle = async (itemId: string) => {
    const nextItems = todo.items.map((it) => (it.itemId === itemId ? { ...it, completed: !it.completed } : it));
    const updatedTodo: TdrpTodoData = { ...todo, items: nextItems };
    setTodo(updatedTodo);
    onUpdateText?.(formatTdrpTodoMessage(updatedTodo));

    const toggledItem = nextItems.find((it) => it.itemId === itemId);
    if (client && toggledItem) {
      try {
        await client.call("todo.toggleItem", { listId: todo.listId, itemId, completed: toggledItem.completed });
      } catch {
        // optimistic preserved
      }
    }
  };

  return (
    <div style={{ background: "var(--fabushi-surface-2, rgba(255, 255, 255, 0.05))", border: "1px solid var(--fabushi-border, rgba(255, 255, 255, 0.12))", borderRadius: 12, padding: "12px 14px", minWidth: 260, maxWidth: 380, fontSize: 13 }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 6 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <SandIcon name="check-square" size="sm" />
          <span style={{ fontWeight: 600 }}>{todo.title}</span>
        </div>
        <span style={{ fontSize: 11, opacity: 0.8 }}>{completedCount}/{todo.items.length} ({pct}%)</span>
      </div>

      <div style={{ height: 4, borderRadius: 2, background: "rgba(255, 255, 255, 0.1)", overflow: "hidden", marginBottom: 10 }}>
        <div style={{ width: `${pct}%`, height: "100%", background: "#22c55e", transition: "width 0.3s ease" }} />
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
        {todo.items.map((item) => (
          <label key={item.itemId} style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer", textDecoration: item.completed ? "line-through" : "none", opacity: item.completed ? 0.6 : 1 }}>
            <input checked={item.completed} onChange={() => void handleToggle(item.itemId)} style={{ cursor: "pointer" }} type="checkbox" />
            <span>{item.text}</span>
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
      } catch {
        // optimistic
      } finally {
        setPaying(false);
      }
    } else {
      setPaying(false);
    }
  };

  return (
    <div style={{ background: "var(--fabushi-surface-2, rgba(255, 255, 255, 0.05))", border: "1px solid var(--fabushi-border, rgba(255, 255, 255, 0.12))", borderRadius: 12, padding: "14px 16px", minWidth: 260, maxWidth: 380, fontSize: 13 }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 6 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <SandIcon name="credit-card" size="sm" />
          <span style={{ fontWeight: 600 }}>{invoice.title}</span>
        </div>
        <span style={{ fontWeight: 700, color: "#38bdf8", fontSize: 14 }}>{priceFormatted}</span>
      </div>

      {invoice.description && <p style={{ margin: "4px 0 10px 0", fontSize: 12, opacity: 0.8 }}>{invoice.description}</p>}

      {invoice.isPaid ? (
        <div style={{ display: "inline-flex", alignItems: "center", gap: 6, background: "rgba(34, 197, 94, 0.15)", border: "1px solid rgba(34, 197, 94, 0.3)", color: "#4ade80", padding: "4px 10px", borderRadius: 6, fontSize: 11, fontWeight: 600 }}>
          <SandIcon name="check-circle" size="sm" />
          <span>Settled · {invoice.chargeId ?? "Success"}</span>
        </div>
      ) : (
        <SandButton
          disabled={paying}
          onClick={() => void handlePay()}
          sentiment="accent"
          size="sm"
          style={{ width: "100%", justifyContent: "center" }}
        >
          {paying ? "Settling..." : `Pay ${priceFormatted}`}
        </SandButton>
      )}
    </div>
  );
}
