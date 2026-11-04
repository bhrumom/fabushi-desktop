import React, { useState } from "react";
import { getSharedCoordinatorClient, type ProductionCoordinatorClient } from "../coordinator-client";
import { SandIconButton, SandButton } from "../../recovered/ui/sand-kit-primitives";
import {
  formatTdrpInvoiceMessage,
  formatTdrpPollMessage,
  formatTdrpTodoMessage,
  type TdrpPollKind,
} from "./tdrp-interactive-models";

export { TdrpComposerToolbar, type TdrpComposerToolbarProps } from "./TdrpComposerToolbar";

export interface TdrpPollAndTodoModalProps {
  readonly isOpen: boolean;
  readonly initialMode?: "poll" | "todo" | "invoice" | "bot";
  readonly conversationId: string;
  readonly client?: ProductionCoordinatorClient | null;
  readonly onClose: () => void;
  readonly onInsertMessage: (formattedPayload: string) => void;
}

export function TdrpPollAndTodoModal({
  isOpen,
  initialMode = "poll",
  conversationId,
  client,
  onClose,
  onInsertMessage,
}: TdrpPollAndTodoModalProps): React.JSX.Element | null {
  const [mode, setMode] = useState<"poll" | "todo" | "invoice" | "bot">(initialMode);
  const activeClient = client ?? getSharedCoordinatorClient();

  // Poll state
  const [pollQuestion, setPollQuestion] = useState("");
  const [pollOptions, setPollOptions] = useState<string[]>(["", ""]);
  const [isQuiz, setIsQuiz] = useState(false);
  const [quizCorrectIndex, setQuizCorrectIndex] = useState(0);
  const [quizExplanation, setQuizExplanation] = useState("");
  const [multipleAnswers, setMultipleAnswers] = useState(false);

  // Todo state
  const [todoTitle, setTodoTitle] = useState("");
  const [todoItems, setTodoItems] = useState<string[]>([""]);

  // Invoice state
  const [invoiceTitle, setInvoiceTitle] = useState("");
  const [invoiceDesc, setInvoiceDesc] = useState("");
  const [invoiceCurrency, setInvoiceCurrency] = useState("USD");
  const [invoiceAmount, setInvoiceAmount] = useState("10.00");

  // Bot state
  const [botQuery, setBotQuery] = useState("");
  const [botResults, setBotResults] = useState<{ id: string; title: string; text: string }[]>([]);

  if (!isOpen) return null;

  const handleCreatePoll = async () => {
    const validOptions = pollOptions.map((o) => o.trim()).filter(Boolean);
    if (!pollQuestion.trim() || validOptions.length < 2) return;

    const kind: TdrpPollKind = isQuiz
      ? { type: "Quiz", correct_option_id: quizCorrectIndex, explanation: quizExplanation.trim() || undefined }
      : { type: "Regular", multiple_answers: multipleAnswers };

    let pollId = `poll_${Date.now()}`;
    if (activeClient) {
      try {
        const res = (await activeClient.call("polls.create", {
          question: pollQuestion.trim(),
          options: validOptions,
          kind,
        })) as { poll?: { poll_id?: string } };
        if (res?.poll?.poll_id) pollId = res.poll.poll_id;
      } catch {
        // Fallback to local generated ID
      }
    }

    const payload = formatTdrpPollMessage({
      pollId,
      question: pollQuestion.trim(),
      options: validOptions.map((text, idx) => ({ id: idx, text, voterCount: 0 })),
      isClosed: false,
      totalVoters: 0,
      kind,
      selectedOptionIds: [],
    });

    onInsertMessage(payload);
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
        })) as { todoList?: { list_id?: string } };
        if (res?.todoList?.list_id) listId = res.todoList.list_id;
      } catch {
        // Fallback
      }
    }

    const payload = formatTdrpTodoMessage({
      listId,
      title: todoTitle.trim(),
      items: validItems.map((text, idx) => ({ itemId: `item_${idx + 1}`, text, completed: false })),
    });

    onInsertMessage(payload);
    onClose();
  };

  const handleCreateInvoice = async () => {
    if (!invoiceTitle.trim()) return;
    const amountVal = parseFloat(invoiceAmount) || 0;
    const totalAmountCents = invoiceCurrency === "STARS" ? Math.round(amountVal) : Math.round(amountVal * 100);

    let invoiceId = `inv_${Date.now()}`;
    if (activeClient) {
      try {
        const res = (await activeClient.call("payments.createInvoice", {
          title: invoiceTitle.trim(),
          description: invoiceDesc.trim(),
          currency: invoiceCurrency,
          totalAmountCents,
        })) as { invoice?: { invoice_id?: string } };
        if (res?.invoice?.invoice_id) invoiceId = res.invoice.invoice_id;
      } catch {
        // Fallback
      }
    }

    const payload = formatTdrpInvoiceMessage({
      invoiceId,
      title: invoiceTitle.trim(),
      description: invoiceDesc.trim(),
      currency: invoiceCurrency,
      totalAmountCents,
      isPaid: false,
    });

    onInsertMessage(payload);
    onClose();
  };

  const handleSearchBot = async (query: string) => {
    setBotQuery(query);
    if (!query.trim() || !activeClient) {
      setBotResults([]);
      return;
    }
    try {
      const res = (await activeClient.call("bots.queryInline", {
        botId: "@calc",
        query: query.trim(),
      })) as { results?: { id: string; title: string; send_message_text: string }[] };
      if (res?.results) {
        setBotResults(res.results.map((r) => ({ id: r.id, title: r.title, text: r.send_message_text })));
      }
    } catch {
      setBotResults([]);
    }
  };

  return (
    <div style={{ position: "fixed", inset: 0, zIndex: 1000, background: "rgba(0, 0, 0, 0.65)", display: "flex", alignItems: "center", justifyContent: "center" }}>
      <div style={{ background: "var(--fabushi-surface-1, #1e1e24)", border: "1px solid var(--fabushi-border, rgba(255, 255, 255, 0.15))", borderRadius: 16, width: 440, maxHeight: "85vh", display: "flex", flexDirection: "column", boxShadow: "0 20px 40px rgba(0,0,0,0.5)", color: "inherit", fontSize: 13, overflow: "hidden" }}>
        {/* Header */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "12px 16px", borderBottom: "1px solid rgba(255, 255, 255, 0.08)" }}>
          <div style={{ display: "flex", gap: 8 }}>
            {(["poll", "todo", "invoice", "bot"] as const).map((t) => (
              <button
                key={t}
                onClick={() => setMode(t)}
                style={{
                  background: mode === t ? "var(--fabushi-primary, #3b82f6)" : "transparent",
                  color: mode === t ? "#fff" : "rgba(255, 255, 255, 0.6)",
                  border: "none",
                  borderRadius: 6,
                  padding: "4px 10px",
                  fontSize: 12,
                  fontWeight: 600,
                  cursor: "pointer",
                }}
                type="button"
              >
                {t === "poll" ? "Poll" : t === "todo" ? "Todo" : t === "invoice" ? "Invoice" : "Bot"}
              </button>
            ))}
          </div>
          <SandIconButton aria-label="Close" icon="close" label="Close" onClick={onClose} size="sm" />
        </div>

        {/* Body */}
        <div style={{ padding: 16, overflowY: "auto", display: "flex", flexDirection: "column", gap: 12 }}>
          {mode === "poll" && (
            <>
              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>Question</label>
                <input
                  onChange={(e) => setPollQuestion(e.target.value)}
                  placeholder="Ask a question..."
                  style={{ width: "100%", padding: "8px 10px", borderRadius: 8, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit" }}
                  type="text"
                  value={pollQuestion}
                />
              </div>

              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>Options</label>
                <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                  {pollOptions.map((opt, idx) => (
                    <div key={idx} style={{ display: "flex", gap: 6, alignItems: "center" }}>
                      {isQuiz && (
                        <input
                          checked={quizCorrectIndex === idx}
                          name="quiz_correct"
                          onChange={() => setQuizCorrectIndex(idx)}
                          title="Mark as correct answer"
                          type="radio"
                        />
                      )}
                      <input
                        onChange={(e) => {
                          const next = [...pollOptions];
                          next[idx] = e.target.value;
                          setPollOptions(next);
                        }}
                        placeholder={`Option ${idx + 1}`}
                        style={{ flex: 1, padding: "6px 8px", borderRadius: 6, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit" }}
                        type="text"
                        value={opt}
                      />
                      {pollOptions.length > 2 && (
                        <SandIconButton aria-label="Remove" icon="close" label="Remove" onClick={() => setPollOptions(pollOptions.filter((_, i) => i !== idx))} size="sm" />
                      )}
                    </div>
                  ))}
                  {pollOptions.length < 10 && (
                    <SandButton onClick={() => setPollOptions([...pollOptions, ""])} sentiment="neutral" size="sm" type="button">
                      + Add Option
                    </SandButton>
                  )}
                </div>
              </div>

              <div style={{ display: "flex", gap: 16, marginTop: 4 }}>
                <label style={{ display: "flex", alignItems: "center", gap: 6, cursor: "pointer", fontSize: 12 }}>
                  <input checked={isQuiz} onChange={(e) => setIsQuiz(e.target.checked)} type="checkbox" />
                  <span>Quiz Mode</span>
                </label>
                {!isQuiz && (
                  <label style={{ display: "flex", alignItems: "center", gap: 6, cursor: "pointer", fontSize: 12 }}>
                    <input checked={multipleAnswers} onChange={(e) => setMultipleAnswers(e.target.checked)} type="checkbox" />
                    <span>Multiple Answers</span>
                  </label>
                )}
              </div>

              {isQuiz && (
                <div>
                  <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>Explanation (Optional)</label>
                  <input
                    onChange={(e) => setQuizExplanation(e.target.value)}
                    placeholder="Shown when users select an answer"
                    style={{ width: "100%", padding: "6px 8px", borderRadius: 6, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit" }}
                    type="text"
                    value={quizExplanation}
                  />
                </div>
              )}
            </>
          )}

          {mode === "todo" && (
            <>
              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>Checklist Title</label>
                <input
                  onChange={(e) => setTodoTitle(e.target.value)}
                  placeholder="e.g. Release Sprint Tasks"
                  style={{ width: "100%", padding: "8px 10px", borderRadius: 8, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit" }}
                  type="text"
                  value={todoTitle}
                />
              </div>

              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>Items</label>
                <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                  {todoItems.map((item, idx) => (
                    <div key={idx} style={{ display: "flex", gap: 6, alignItems: "center" }}>
                      <input
                        onChange={(e) => {
                          const next = [...todoItems];
                          next[idx] = e.target.value;
                          setTodoItems(next);
                        }}
                        placeholder={`Task ${idx + 1}`}
                        style={{ flex: 1, padding: "6px 8px", borderRadius: 6, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit" }}
                        type="text"
                        value={item}
                      />
                      {todoItems.length > 1 && (
                        <SandIconButton aria-label="Remove" icon="close" label="Remove" onClick={() => setTodoItems(todoItems.filter((_, i) => i !== idx))} size="sm" />
                      )}
                    </div>
                  ))}
                  <SandButton onClick={() => setTodoItems([...todoItems, ""])} sentiment="neutral" size="sm" type="button">
                    + Add Task
                  </SandButton>
                </div>
              </div>
            </>
          )}

          {mode === "invoice" && (
            <>
              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>Title</label>
                <input
                  onChange={(e) => setInvoiceTitle(e.target.value)}
                  placeholder="e.g. Pro Subscription"
                  style={{ width: "100%", padding: "8px 10px", borderRadius: 8, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit" }}
                  type="text"
                  value={invoiceTitle}
                />
              </div>
              <div>
                <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>Description</label>
                <input
                  onChange={(e) => setInvoiceDesc(e.target.value)}
                  placeholder="e.g. 1 Year access to premium cloud bots"
                  style={{ width: "100%", padding: "8px 10px", borderRadius: 8, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit" }}
                  type="text"
                  value={invoiceDesc}
                />
              </div>
              <div style={{ display: "flex", gap: 10 }}>
                <div style={{ flex: 1 }}>
                  <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>Currency</label>
                  <select
                    onChange={(e) => setInvoiceCurrency(e.target.value)}
                    style={{ width: "100%", padding: "8px 10px", borderRadius: 8, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit" }}
                    value={invoiceCurrency}
                  >
                    <option value="USD">USD ($)</option>
                    <option value="STARS">Stars</option>
                    <option value="TON">TON</option>
                    <option value="EUR">EUR (€)</option>
                    <option value="CNY">CNY (¥)</option>
                  </select>
                </div>
                <div style={{ flex: 1 }}>
                  <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>Amount</label>
                  <input
                    onChange={(e) => setInvoiceAmount(e.target.value)}
                    placeholder="10.00"
                    style={{ width: "100%", padding: "8px 10px", borderRadius: 8, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit" }}
                    type="number"
                    value={invoiceAmount}
                  />
                </div>
              </div>
            </>
          )}

          {mode === "bot" && (
            <div>
              <label style={{ display: "block", fontSize: 11, fontWeight: 600, marginBottom: 4, opacity: 0.8 }}>Inline Query</label>
              <input
                onChange={(e) => void handleSearchBot(e.target.value)}
                placeholder="Type bot query, e.g. @calc 42 * 10..."
                style={{ width: "100%", padding: "8px 10px", borderRadius: 8, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit" }}
                type="text"
                value={botQuery}
              />
              <div style={{ display: "flex", flexDirection: "column", gap: 6, marginTop: 10 }}>
                {botResults.map((r) => (
                  <button
                    key={r.id}
                    onClick={() => {
                      onInsertMessage(r.text);
                      onClose();
                    }}
                    style={{
                      background: "rgba(255,255,255,0.06)",
                      border: "1px solid rgba(255,255,255,0.1)",
                      borderRadius: 8,
                      padding: "8px 10px",
                      textAlign: "left",
                      color: "inherit",
                      cursor: "pointer",
                    }}
                    type="button"
                  >
                    <div style={{ fontWeight: 600 }}>{r.title}</div>
                    <div style={{ fontSize: 11, opacity: 0.7 }}>{r.text}</div>
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, padding: "12px 16px", borderTop: "1px solid rgba(255, 255, 255, 0.08)" }}>
          <SandButton onClick={onClose} sentiment="neutral" size="sm" type="button">Cancel</SandButton>
          {mode === "poll" && (
            <SandButton onClick={() => void handleCreatePoll()} sentiment="accent" size="sm" type="button">Create Poll</SandButton>
          )}
          {mode === "todo" && (
            <SandButton onClick={() => void handleCreateTodo()} sentiment="accent" size="sm" type="button">Create Checklist</SandButton>
          )}
          {mode === "invoice" && (
            <SandButton onClick={() => void handleCreateInvoice()} sentiment="accent" size="sm" type="button">Send Invoice</SandButton>
          )}
        </div>
      </div>
    </div>
  );
}
