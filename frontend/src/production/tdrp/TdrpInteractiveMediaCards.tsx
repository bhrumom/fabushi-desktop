/**
 * Interactive TDRP media cards for Polls, Todo Lists, and Invoice/Gift Checkout.
 *
 * Upstream & Open PR Alignment:
 * - Telegram Desktop `Telegram/SourceFiles/data/data_poll.cpp` & `api/api_polls.cpp`
 *   (Poll voting, multi-choice toggle, quiz answer reveal, vote retraction).
 * - Telegram Desktop `Telegram/SourceFiles/data/data_todo_list.h`
 *   (Collaborative Todo item completion tracking).
 * - Telegram Desktop `Telegram/SourceFiles/api/api_credits.cpp` (`CreditsStatus`) &
 *   `Telegram/SourceFiles/payments/payments_form.cpp` (`InvoiceCredits`, `InvoiceStarGift`):
 *   Real Stars balance validation and idempotency-protected payment settlement.
 * - Host Rust `crates/fabushi-Telegram/src/stories_gifts_stars.rs` (`stars.status`,
 *   `stars.sendGift`) & `export_payments_bots.rs` (`payments.settlePayment`).
 * - Blueprint 03 (`docs/specs/fusion-blueprints/03-unified-composer-and-typed-actions.md`) &
 *   Blueprint 13 (`docs/specs/fusion-blueprints/13-checkout-stars-ledger-and-economy.md`).
 */

import React, { useEffect, useRef, useState } from "react";
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

export function createCheckoutIdempotencyKey(scope: string, itemId: string): string {
  const randomPart =
    typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
      ? crypto.randomUUID()
      : `${Date.now()}_${Math.random().toString(16).slice(2)}`;
  return `idemp_${scope}_${itemId}_${randomPart}`;
}

export function validateStarsCheckoutBalance(
  availableBalance: number | null | undefined,
  requiredStars: number
): { readonly valid: boolean; readonly errorMessage: string | null } {
  if (requiredStars <= 0) {
    return { valid: true, errorMessage: null };
  }
  if (availableBalance == null || !Number.isFinite(availableBalance)) {
    return { valid: true, errorMessage: null };
  }
  if (availableBalance < requiredStars) {
    return {
      valid: false,
      errorMessage: `Stars 余额不足：当前可用 ${availableBalance} ⭐，本次需要 ${requiredStars} ⭐`,
    };
  }
  return { valid: true, errorMessage: null };
}

export interface TdrpInteractiveMediaCardProps {
  readonly text: string;
  readonly client?: ProductionCoordinatorClient | null;
  readonly userId?: string;
  readonly onUpdateText?: (updatedText: string) => void;
  /** Non-destructive direct message callback for card follow-up actions. */
  readonly onSubmitDirectMessage?: (payload: string) => Promise<void> | void;
  /** Real Stars balance from `stars.status` (never hardcoded). */
  readonly starsBalance?: number | null;
  /** Notifies parent when a Stars/Gift checkout debits the user's Stars balance. */
  readonly onStarsBalanceChange?: (nextBalance: number) => void;
}

export function TdrpInteractiveMediaCard({
  text,
  client,
  userId = "me",
  onUpdateText,
  onSubmitDirectMessage,
  starsBalance,
  onStarsBalanceChange,
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
    return (
      <TdrpInvoiceCard
        client={activeClient}
        data={payload}
        onStarsBalanceChange={onStarsBalanceChange}
        onSubmitDirectMessage={onSubmitDirectMessage}
        onUpdateText={onUpdateText}
        starsBalance={starsBalance}
        userId={userId}
      />
    );
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
  userId = "me",
  starsBalance,
  onStarsBalanceChange,
  onSubmitDirectMessage,
  onUpdateText,
}: {
  data: TdrpInvoiceData;
  client?: ProductionCoordinatorClient | null;
  userId?: string;
  starsBalance?: number | null;
  onStarsBalanceChange?: (nextBalance: number) => void;
  onSubmitDirectMessage?: (payload: string) => Promise<void> | void;
  onUpdateText?: (newText: string) => void;
}) {
  const [invoice, setInvoice] = useState<TdrpInvoiceData>(data);
  const [paying, setPaying] = useState(false);
  const [liveStarsBalance, setLiveStarsBalance] = useState<number | null>(
    typeof starsBalance === "number" ? starsBalance : null
  );
  const [checkoutError, setCheckoutError] = useState<string | null>(null);
  const idempotencyKeyRef = useRef<string>(
    createCheckoutIdempotencyKey("invoice", data.invoiceId)
  );

  const isStarsCurrency = invoice.currency === "STARS" || invoice.currency === "XTR";

  useEffect(() => {
    if (typeof starsBalance === "number") {
      setLiveStarsBalance(starsBalance);
      return;
    }
    if (!isStarsCurrency || !client || invoice.isPaid) return;
    let cancelled = false;
    void client
      .call("stars.status", { peerId: userId })
      .then((res) => {
        if (cancelled) return;
        const status = res as { status?: { balance?: number }; balance?: number } | null;
        const resolved =
          typeof status?.status?.balance === "number"
            ? status.status.balance
            : typeof status?.balance === "number"
              ? status.balance
              : null;
        if (resolved != null) {
          setLiveStarsBalance(resolved);
        }
      })
      .catch(() => {
        /* Host balance query optional when offline */
      });
    return () => {
      cancelled = true;
    };
  }, [client, invoice.isPaid, isStarsCurrency, starsBalance, userId]);

  const priceFormatted = isStarsCurrency
    ? `${invoice.totalAmountCents} Stars`
    : `${(invoice.totalAmountCents / 100).toFixed(2)} ${invoice.currency}`;

  const handlePay = async () => {
    if (invoice.isPaid || paying) return;
    setCheckoutError(null);
    setPaying(true);

    try {
      let currentBalance = liveStarsBalance;
      if (isStarsCurrency && currentBalance == null && client) {
        try {
          const res = (await client.call("stars.status", { peerId: userId })) as {
            status?: { balance?: number };
            balance?: number;
          } | null;
          const fetched =
            typeof res?.status?.balance === "number"
              ? res.status.balance
              : typeof res?.balance === "number"
                ? res.balance
                : null;
          if (fetched != null) {
            currentBalance = fetched;
            setLiveStarsBalance(fetched);
          }
        } catch {
          /* Proceed if stars.status RPC is unavailable in standalone mode */
        }
      }

      if (isStarsCurrency) {
        const check = validateStarsCheckoutBalance(currentBalance, invoice.totalAmountCents);
        if (!check.valid) {
          setCheckoutError(check.errorMessage);
          setPaying(false);
          return;
        }
      }

      const idempotencyKey = idempotencyKeyRef.current;
      let chargeId = `ch_${idempotencyKey}`;

      if (client) {
        const settled = (await client.call("payments.settlePayment", {
          invoiceId: invoice.invoiceId,
          chargeId,
          idempotencyKey,
          currency: invoice.currency,
          totalAmountCents: invoice.totalAmountCents,
        })) as {
          ok?: boolean;
          receipt?: { charge_id?: string };
          error?: string;
        } | null;

        if (settled?.ok === false) {
          setCheckoutError(settled.error ?? "支付结算未通过校验，请重试。");
          setPaying(false);
          return;
        }
        if (typeof settled?.receipt?.charge_id === "string" && settled.receipt.charge_id.length > 0) {
          chargeId = settled.receipt.charge_id;
        }
      }

      if (isStarsCurrency && typeof currentBalance === "number") {
        const nextBalance = Math.max(0, currentBalance - invoice.totalAmountCents);
        setLiveStarsBalance(nextBalance);
        onStarsBalanceChange?.(nextBalance);
      }

      const updatedInvoice: TdrpInvoiceData = { ...invoice, isPaid: true, chargeId };
      setInvoice(updatedInvoice);
      const formatted = formatTdrpInvoiceMessage(updatedInvoice);
      onUpdateText?.(formatted);
      if (onSubmitDirectMessage != null && onUpdateText == null) {
        await onSubmitDirectMessage(formatted);
      }
    } catch (error) {
      setCheckoutError(error instanceof Error ? error.message : "支付失败，请稍后重试。");
    } finally {
      setPaying(false);
    }
  };

  return (
    <div style={cardStyle} data-idempotency-key={idempotencyKeyRef.current}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 6 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <SandIcon name="credit-card" size="sm" />
          <span style={{ fontWeight: 600 }}>{invoice.title}</span>
        </div>
        <span style={{ fontWeight: 700, color: "var(--sand-color-primary, #2563eb)", fontSize: 14 }}>{priceFormatted}</span>
      </div>

      {invoice.description && <p style={{ margin: "4px 0 10px 0", fontSize: 12, opacity: 0.8 }}>{invoice.description}</p>}

      {isStarsCurrency && liveStarsBalance != null && (
        <div
          data-testid="tdrp-invoice-stars-balance"
          style={{ marginBottom: 8, fontSize: 11, opacity: 0.8, display: "flex", justifyContent: "space-between" }}
        >
          <span>当前可用 Stars 余额</span>
          <strong>{liveStarsBalance} ⭐</strong>
        </div>
      )}

      {invoice.prices && invoice.prices.length > 0 && (
        <div style={{ margin: "6px 0 10px 0", padding: "6px 8px", borderRadius: 6, background: "rgba(128, 128, 128, 0.06)", fontSize: 11 }}>
          {invoice.prices.map((p, idx) => (
            <div key={idx} style={{ display: "flex", justifyContent: "space-between", opacity: 0.85 }}>
              <span>{p.label}</span>
              <span>
                {isStarsCurrency
                  ? `${p.amountCents} Stars`
                  : `${(p.amountCents / 100).toFixed(2)} ${invoice.currency}`}
              </span>
            </div>
          ))}
        </div>
      )}

      {checkoutError != null && (
        <div
          role="alert"
          data-testid="tdrp-invoice-checkout-error"
          style={{
            marginBottom: 8,
            padding: "6px 8px",
            borderRadius: 6,
            background: "rgba(220, 38, 38, 0.1)",
            border: "1px solid rgba(220, 38, 38, 0.3)",
            color: "#dc2626",
            fontSize: 11,
          }}
        >
          {checkoutError}
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
