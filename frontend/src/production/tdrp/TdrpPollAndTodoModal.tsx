/**
 * Modal for creating Polls, Todo Lists, Invoices, and Bot inline queries.
 *
 * Upstream & Open PR Alignment:
 * - Telegram Desktop `Telegram/SourceFiles/data/data_poll.cpp` & `api/api_polls.cpp`
 *   (Poll flags, Quiz vs Regular mode, 32-option bound).
 * - Telegram Desktop `Telegram/SourceFiles/data/data_todo_list.h`
 *   (`TodoListData::Flag::OthersCanAppend | OthersCanComplete`).
 * - Telegram Desktop `Telegram/SourceFiles/payments/payments_form.cpp` & `api/api_credits.cpp`
 *   (`LabeledPrice`, `InvoiceCredits` with idempotency key).
 * - Telegram Desktop `Telegram/SourceFiles/api/api_sending.cpp` (`SendSimpleMedia`:
 *   `action.clearDraft = false` — typed actions dispatch directly without overwriting
 *   the user's active composer text draft).
 * - Blueprint 03 (`docs/specs/fusion-blueprints/03-unified-composer-and-typed-actions.md`) &
 *   Blueprint 13 (`docs/specs/fusion-blueprints/13-checkout-stars-ledger-and-economy.md`).
 */

import React, { useState } from "react";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import type { DirectTypedActionPayload } from "../../recovered/features/conversation/workspace/submission";
import { SandIconButton, SandButton } from "../../recovered/ui/sand-kit-primitives";
import {
  formatTdrpInvoiceMessage,
  formatTdrpPollMessage,
  formatTdrpTodoMessage,
  type TdrpLabeledPrice,
  type TdrpPollKind,
} from "./tdrp-interactive-models";

export function createTypedActionIdempotencyKey(prefix: string): string {
  const suffix =
    typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
      ? crypto.randomUUID()
      : `${Date.now()}_${Math.random().toString(16).slice(2)}`;
  return `${prefix}_${suffix}`;
}

export interface TdrpPollAndTodoModalProps {
  readonly isOpen: boolean;
  readonly initialMode?: "poll" | "todo" | "invoice" | "bot";
  readonly conversationId: string;
  readonly client?: ProductionCoordinatorClient | null;
  readonly onClose: () => void;
  readonly onInsertMessage?: (formattedPayload: string) => void;
  /**
   * Direct non-destructive message submission callback. Dispatches the formatted
   * card payload directly through the conversation submission queue without
   * mutating `composerDraftStore` or relying on `setTimeout(..., 50)`.
   */
  readonly onSubmitDirectMessage?: (
    formattedPayload: string,
    typedPayload?: DirectTypedActionPayload
  ) => Promise<void> | void;
  /**
   * Structured typed-action direct submission callback (Blueprint 03 Section 2.4).
   */
  readonly onDirectSubmit?: (
    payload: DirectTypedActionPayload,
    formattedPayload: string
  ) => Promise<void> | void;
}

export function TdrpPollAndTodoModal({
  isOpen,
  initialMode = "poll",
  conversationId,
  client,
  onClose,
  onInsertMessage,
  onSubmitDirectMessage,
  onDirectSubmit,
}: TdrpPollAndTodoModalProps): React.JSX.Element | null {
  const [mode, setMode] = useState<"poll" | "todo" | "invoice" | "bot">(initialMode);
  const activeClient = client ?? null;

  const dispatchNonDestructively = async (
    formattedPayload: string,
    typedPayload: DirectTypedActionPayload
  ): Promise<void> => {
    if (onDirectSubmit != null) {
      await onDirectSubmit(typedPayload, formattedPayload);
    } else if (onSubmitDirectMessage != null) {
      await onSubmitDirectMessage(formattedPayload, typedPayload);
    } else {
      onInsertMessage?.(formattedPayload);
    }
  };

  // Poll state (upstream data_poll.h & create_poll_box.cpp)
  const [pollQuestion, setPollQuestion] = useState("");
  const [pollOptions, setPollOptions] = useState<string[]>(["", ""]);
  const [isQuiz, setIsQuiz] = useState(false);
  const [quizCorrectIndex, setQuizCorrectIndex] = useState(0);
  const [quizExplanation, setQuizExplanation] = useState("");
  const [multipleAnswers, setMultipleAnswers] = useState(false);
  const [publicVotes, setPublicVotes] = useState(false);
  const [closePeriodSecs, setClosePeriodSecs] = useState<number | undefined>(undefined);

  // Todo state (upstream data_todo_list.h)
  const [todoTitle, setTodoTitle] = useState("");
  const [todoItems, setTodoItems] = useState<string[]>([""]);
  const [othersCanAppend, setOthersCanAppend] = useState(true);
  const [othersCanComplete, setOthersCanComplete] = useState(true);

  // Invoice state (upstream payments_panel_data.h)
  const [invoiceTitle, setInvoiceTitle] = useState("");
  const [invoiceDesc, setInvoiceDesc] = useState("");
  const [invoiceCurrency, setInvoiceCurrency] = useState("USD");
  const [invoiceAmount, setInvoiceAmount] = useState("10.00");
  const [isRecurring, setIsRecurring] = useState(false);

  // Bot state
  const [botQuery, setBotQuery] = useState("");
  const [botResults, setBotResults] = useState<{ id: string; title: string; text: string }[]>([]);

  if (!isOpen) return null;

  const inputStyle: React.CSSProperties = {
    width: "100%",
    padding: "8px 10px",
    borderRadius: 8,
    background: "var(--sand-color-surface-base, rgba(128, 128, 128, 0.06))",
    border: "1px solid rgba(128, 128, 128, 0.22)",
    color: "inherit",
    fontSize: 13,
    boxSizing: "border-box",
  };

  const handleCreatePoll = async () => {
    const validOptions = pollOptions.map((o) => o.trim()).filter(Boolean);
    if (!pollQuestion.trim() || validOptions.length < 2) return;
    const explanation = quizExplanation.trim().slice(0, 200) || undefined;
    const kind: TdrpPollKind = isQuiz
      ? { type: "Quiz", correct_option_id: quizCorrectIndex, explanation }
      : { type: "Regular", multiple_answers: multipleAnswers };

    let pollId = `poll_${Date.now()}`;
    if (activeClient) {
      try {
        const res = (await activeClient.call("polls.create", {
          question: pollQuestion.trim(),
          options: validOptions,
          kind,
          publicVotes,
          closePeriodSecs,
        })) as { poll?: { poll_id?: string } };
        if (res?.poll?.poll_id) pollId = res.poll.poll_id;
      } catch {
        /* Fallback to local pollId */
      }
    }

    const formattedPayload = formatTdrpPollMessage({
      pollId,
      question: pollQuestion.trim(),
      options: validOptions.map((text, idx) => ({ id: idx, text, voterCount: 0 })),
      isClosed: false,
      totalVoters: 0,
      kind,
      publicVotes,
      closePeriodSecs,
      selectedOptionIds: [],
    });
    const typedPayload: DirectTypedActionPayload = {
      kind: "poll",
      question: pollQuestion.trim(),
      options: validOptions,
      isAnonymous: !publicVotes,
      allowsMultipleAnswers: !isQuiz && multipleAnswers,
      isQuiz,
      ...(isQuiz ? { correctOptionIndex: quizCorrectIndex } : {}),
      ...(isQuiz && explanation ? { explanation } : {}),
    };

    await dispatchNonDestructively(formattedPayload, typedPayload);
    onClose();
  };

  const handleCreateTodo = async () => {
    const validItems = todoItems.map((i) => i.trim()).filter(Boolean);
    if (!todoTitle.trim() || validItems.length === 0) return;

    let listId = `todo_${Date.now()}`;
    if (activeClient) {
      try {
        const res = (await activeClient.call("todo.createList", {
          conversationId,
          title: todoTitle.trim(),
          othersCanAppend,
          othersCanComplete,
        })) as { todoList?: { list_id?: string } };
        if (res?.todoList?.list_id) listId = res.todoList.list_id;
      } catch {
        /* Fallback to local listId */
      }
    }

    const items = validItems.map((text, idx) => ({
      itemId: `item_${idx + 1}`,
      text,
      completed: false,
    }));
    const formattedPayload = formatTdrpTodoMessage({
      listId,
      title: todoTitle.trim(),
      othersCanAppend,
      othersCanComplete,
      items,
    });
    const typedPayload: DirectTypedActionPayload = {
      kind: "todo",
      title: todoTitle.trim(),
      items: items.map((item) => ({ id: item.itemId, text: item.text, completed: item.completed })),
      othersCanAppend,
      othersCanComplete,
    };

    await dispatchNonDestructively(formattedPayload, typedPayload);
    onClose();
  };

  const handleCreateInvoice = async () => {
    if (!invoiceTitle.trim()) return;
    const amountVal = parseFloat(invoiceAmount) || 0;
    const totalAmountCents =
      invoiceCurrency === "STARS" || invoiceCurrency === "XTR"
        ? Math.round(amountVal)
        : Math.round(amountVal * 100);
    const prices: TdrpLabeledPrice[] = [{ label: invoiceTitle.trim(), amountCents: totalAmountCents }];
    const idempotencyKey = createTypedActionIdempotencyKey("inv");

    let invoiceId = `inv_${Date.now()}`;
    if (activeClient) {
      try {
        const res = (await activeClient.call("payments.createInvoice", {
          title: invoiceTitle.trim(),
          description: invoiceDesc.trim(),
          currency: invoiceCurrency,
          totalAmountCents,
          prices,
          isRecurring,
          idempotencyKey,
        })) as { invoice?: { invoice_id?: string } };
        if (res?.invoice?.invoice_id) invoiceId = res.invoice.invoice_id;
      } catch {
        /* Fallback to local invoiceId */
      }
    }

    const formattedPayload = formatTdrpInvoiceMessage({
      invoiceId,
      title: invoiceTitle.trim(),
      description: invoiceDesc.trim(),
      currency: invoiceCurrency,
      totalAmountCents,
      isPaid: false,
      prices,
      isRecurring,
    });
    const typedPayload: DirectTypedActionPayload = {
      kind: "invoice",
      title: invoiceTitle.trim(),
      description: invoiceDesc.trim(),
      currency: invoiceCurrency === "STARS" ? "XTR" : invoiceCurrency,
      amount: totalAmountCents,
      prices: prices.map((price) => ({ label: price.label, amount: price.amountCents })),
      idempotencyKey,
    };

    await dispatchNonDestructively(formattedPayload, typedPayload);
    onClose();
  };

  const handleSearchBot = async (query: string) => {
    setBotQuery(query);
    if (!query.trim() || !activeClient) { setBotResults([]); return; }
    try {
      const res = (await activeClient.call("bots.queryInline", { botId: "@calc", query: query.trim() })) as { results?: { id: string; title: string; send_message_text: string }[] };
      if (res?.results) setBotResults(res.results.map((r) => ({ id: r.id, title: r.title, text: r.send_message_text })));
    } catch { setBotResults([]); }
  };

  return (
    <div style={{ position: "fixed", inset: 0, zIndex: 1000, background: "rgba(0, 0, 0, 0.45)", backdropFilter: "blur(4px)", display: "flex", alignItems: "center", justifyContent: "center" }}>
      <div style={{ background: "var(--sand-color-surface-card, #ffffff)", border: "1px solid var(--sand-color-border-subtle, rgba(128, 128, 128, 0.2))", borderRadius: 16, width: 440, maxHeight: "85vh", display: "flex", flexDirection: "column", boxShadow: "0 16px 36px rgba(0,0,0,0.2)", color: "inherit", fontSize: 13, overflow: "hidden" }}>
        {/* Header Tabs */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "12px 16px", borderBottom: "1px solid rgba(128, 128, 128, 0.15)" }}>
          <div style={{ display: "flex", gap: 6 }}>
            {([
              { id: "poll", label: "投票" },
              { id: "todo", label: "待办清单" },
              { id: "invoice", label: "收款账单" },
              { id: "bot", label: "快捷指令" },
            ] as const).map((t) => (
              <button
                key={t.id}
                onClick={() => setMode(t.id)}
                style={{
                  background: mode === t.id ? "var(--sand-color-primary, #2563eb)" : "transparent",
                  color: mode === t.id ? "#ffffff" : "inherit",
                  border: mode === t.id ? "none" : "1px solid rgba(128, 128, 128, 0.2)",
                  borderRadius: 6,
                  padding: "4px 10px",
                  fontSize: 12,
                  fontWeight: 600,
                  cursor: "pointer",
                }}
                type="button"
              >
                {t.label}
              </button>
            ))}
          </div>
          <SandIconButton aria-label="Close" icon="close" label="关闭" onClick={onClose} size="sm" />
        </div>

        {/* Body */}
        <div style={{ padding: 16, overflowY: "auto", display: "flex", flexDirection: "column", gap: 12 }}>
          {mode === "poll" && (
            <>
              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>投票标题 / 问题</label>
                <input onChange={(e) => setPollQuestion(e.target.value)} placeholder="提出一个问题..." style={inputStyle} type="text" value={pollQuestion} />
              </div>

              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>选项列表 (最多 32 项)</label>
                <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                  {pollOptions.map((opt, idx) => (
                    <div key={idx} style={{ display: "flex", gap: 6, alignItems: "center" }}>
                      {isQuiz && (
                        <input
                          checked={quizCorrectIndex === idx}
                          name="quizCorrect"
                          onChange={() => setQuizCorrectIndex(idx)}
                          title="设为正确答案"
                          type="radio"
                        />
                      )}
                      <input
                        onChange={(e) => {
                          const next = [...pollOptions];
                          next[idx] = e.target.value;
                          setPollOptions(next);
                        }}
                        placeholder={`选项 ${idx + 1}`}
                        style={{ ...inputStyle, flex: 1 }}
                        type="text"
                        value={opt}
                      />
                      {pollOptions.length > 2 && (
                        <SandIconButton
                          aria-label="Delete option"
                          icon="close"
                          label="删除"
                          onClick={() => setPollOptions(pollOptions.filter((_, i) => i !== idx))}
                          size="sm"
                        />
                      )}
                    </div>
                  ))}
                </div>
                {pollOptions.length < 32 && (
                  <button
                    onClick={() => setPollOptions([...pollOptions, ""])}
                    style={{ marginTop: 6, background: "transparent", border: "none", color: "var(--sand-color-primary, #2563eb)", cursor: "pointer", fontSize: 12, padding: "2px 0" }}
                    type="button"
                  >
                    + 添加选项
                  </button>
                )}
              </div>

              {/* Upstream flags: PublicVotes, MultiChoice, Quiz, ClosePeriod */}
              <div style={{ display: "flex", flexDirection: "column", gap: 8, paddingTop: 4, borderTop: "1px solid rgba(128, 128, 128, 0.12)" }}>
                <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer", fontSize: 12 }}>
                  <input checked={publicVotes} onChange={(e) => setPublicVotes(e.target.checked)} type="checkbox" />
                  <span>公开投票者名单 (默认匿名)</span>
                </label>

                {!isQuiz && (
                  <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer", fontSize: 12 }}>
                    <input checked={multipleAnswers} onChange={(e) => setMultipleAnswers(e.target.checked)} type="checkbox" />
                    <span>允许多选</span>
                  </label>
                )}

                <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer", fontSize: 12 }}>
                  <input checked={isQuiz} onChange={(e) => { setIsQuiz(e.target.checked); if (e.target.checked) setMultipleAnswers(false); }} type="checkbox" />
                  <span>问答测验模式 (单选唯一正确答案)</span>
                </label>

                {isQuiz && (
                  <div>
                    <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>答案解析 (最多 200 字)</label>
                    <input maxLength={200} onChange={(e) => setQuizExplanation(e.target.value)} placeholder="答题后展示给用户的解释说明..." style={inputStyle} type="text" value={quizExplanation} />
                  </div>
                )}
              </div>
            </>
          )}

          {mode === "todo" && (
            <>
              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>待办清单标题</label>
                <input onChange={(e) => setTodoTitle(e.target.value)} placeholder="输入待办清单标题..." style={inputStyle} type="text" value={todoTitle} />
              </div>

              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>任务项列表</label>
                <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                  {todoItems.map((item, idx) => (
                    <div key={idx} style={{ display: "flex", gap: 6, alignItems: "center" }}>
                      <input
                        onChange={(e) => {
                          const next = [...todoItems];
                          next[idx] = e.target.value;
                          setTodoItems(next);
                        }}
                        placeholder={`任务 ${idx + 1}`}
                        style={{ ...inputStyle, flex: 1 }}
                        type="text"
                        value={item}
                      />
                      {todoItems.length > 1 && (
                        <SandIconButton
                          aria-label="Delete item"
                          icon="close"
                          label="删除"
                          onClick={() => setTodoItems(todoItems.filter((_, i) => i !== idx))}
                          size="sm"
                        />
                      )}
                    </div>
                  ))}
                </div>
                {todoItems.length < 32 && (
                  <button
                    onClick={() => setTodoItems([...todoItems, ""])}
                    style={{ marginTop: 6, background: "transparent", border: "none", color: "var(--sand-color-primary, #2563eb)", cursor: "pointer", fontSize: 12, padding: "2px 0" }}
                    type="button"
                  >
                    + 添加任务
                  </button>
                )}
              </div>

              {/* Upstream Todo flags: othersCanAppend, othersCanComplete */}
              <div style={{ display: "flex", flexDirection: "column", gap: 8, paddingTop: 4, borderTop: "1px solid rgba(128, 128, 128, 0.12)" }}>
                <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer", fontSize: 12 }}>
                  <input checked={othersCanAppend} onChange={(e) => setOthersCanAppend(e.target.checked)} type="checkbox" />
                  <span>允许群成员追加新任务</span>
                </label>
                <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer", fontSize: 12 }}>
                  <input checked={othersCanComplete} onChange={(e) => setOthersCanComplete(e.target.checked)} type="checkbox" />
                  <span>允许群成员勾选完成任务</span>
                </label>
              </div>
            </>
          )}

          {mode === "invoice" && (
            <>
              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>项目名称 / 账单标题</label>
                <input onChange={(e) => setInvoiceTitle(e.target.value)} placeholder="如：专业版订阅 1 年..." style={inputStyle} type="text" value={invoiceTitle} />
              </div>
              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>账单描述</label>
                <input onChange={(e) => setInvoiceDesc(e.target.value)} placeholder="服务详情及权益说明..." style={inputStyle} type="text" value={invoiceDesc} />
              </div>
              <div style={{ display: "flex", gap: 8 }}>
                <div style={{ flex: 1 }}>
                  <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>结算货币</label>
                  <select onChange={(e) => setInvoiceCurrency(e.target.value)} style={inputStyle} value={invoiceCurrency}>
                    <option value="USD">USD ($)</option>
                    <option value="EUR">EUR (€)</option>
                    <option value="CNY">CNY (¥)</option>
                    <option value="STARS">STARS (⭐)</option>
                  </select>
                </div>
                <div style={{ flex: 1 }}>
                  <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>金额</label>
                  <input min="0.01" onChange={(e) => setInvoiceAmount(e.target.value)} step="0.01" style={inputStyle} type="number" value={invoiceAmount} />
                </div>
              </div>
              <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer", fontSize: 12 }}>
                <input checked={isRecurring} onChange={(e) => setIsRecurring(e.target.checked)} type="checkbox" />
                <span>自动周期续费 (订阅模式)</span>
              </label>
            </>
          )}

          {mode === "bot" && (
            <>
              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>指令搜索 (@calc)</label>
                <input onChange={(e) => handleSearchBot(e.target.value)} placeholder="输入计算表达式，如 24 * 7..." style={inputStyle} type="text" value={botQuery} />
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 6, maxHeight: 180, overflowY: "auto" }}>
                {botResults.map((res) => (
                  <div
                    key={res.id}
                    onClick={() => {
                      void dispatchNonDestructively(res.text, {
                        kind: "formatted_card",
                        markdown: res.text,
                      }).then(() => {
                        onClose();
                      });
                    }}
                    style={{ padding: "8px 10px", borderRadius: 8, border: "1px solid rgba(128, 128, 128, 0.18)", cursor: "pointer", background: "rgba(128, 128, 128, 0.05)" }}
                  >
                    <div style={{ fontWeight: 600 }}>{res.title}</div>
                    <div style={{ fontSize: 12, opacity: 0.7 }}>{res.text}</div>
                  </div>
                ))}
                {botQuery && botResults.length === 0 && (
                  <div style={{ opacity: 0.6, textAlign: "center", padding: 12 }}>未找到相关指令结果</div>
                )}
              </div>
            </>
          )}
        </div>

        {/* Footer */}
        <div style={{ padding: "12px 16px", borderTop: "1px solid rgba(128, 128, 128, 0.15)", display: "flex", justifyContent: "flex-end", gap: 8 }}>
          <SandButton onClick={onClose} sentiment="neutral" size="sm" type="button">取消</SandButton>
          {mode !== "bot" && (
            <SandButton
              onClick={mode === "poll" ? handleCreatePoll : mode === "todo" ? handleCreateTodo : handleCreateInvoice}
              sentiment="accent"
              size="sm"
              type="button"
            >
              插入到对话
            </SandButton>
          )}
        </div>
      </div>
    </div>
  );
}
