export interface TdrpPollOption {
  id: number;
  text: string;
  voterCount: number;
}

export type TdrpPollKind =
  | { type: "Regular"; multiple_answers: boolean }
  | { type: "Quiz"; correct_option_id: number; explanation?: string };

export interface TdrpPollData {
  type: "tdrp-poll";
  pollId: string;
  question: string;
  options: TdrpPollOption[];
  isClosed: boolean;
  totalVoters: number;
  kind: TdrpPollKind;
  publicVotes?: boolean;
  shuffleAnswers?: boolean;
  revotingDisabled?: boolean;
  closePeriodSecs?: number;
  closeDateMs?: number;
  selectedOptionIds?: number[];
}

export interface TdrpTodoItem {
  itemId: string;
  text: string;
  completed: boolean;
  completedBy?: string;
  completionDateMs?: number;
  assigneeId?: string;
  dueDateMs?: number;
}

export interface TdrpTodoData {
  type: "tdrp-todo";
  listId: string;
  title: string;
  othersCanAppend?: boolean;
  othersCanComplete?: boolean;
  items: TdrpTodoItem[];
}

export interface TdrpLabeledPrice {
  label: string;
  amountCents: number;
}

export interface TdrpPaymentReceipt {
  receiptId: string;
  paidAtMs: number;
  totalAmountCents: number;
  currency: string;
  providerName: string;
}

export interface TdrpInvoiceData {
  type: "tdrp-invoice";
  invoiceId: string;
  title: string;
  description: string;
  currency: string;
  totalAmountCents: number;
  isPaid: boolean;
  chargeId?: string;
  prices?: TdrpLabeledPrice[];
  suggestedTipAmounts?: number[];
  maxTipAmount?: number;
  isRecurring?: boolean;
  needName?: boolean;
  needPhone?: boolean;
  needEmail?: boolean;
  needShippingAddress?: boolean;
  receipt?: TdrpPaymentReceipt;
}

export type TdrpInteractivePayload = TdrpPollData | TdrpTodoData | TdrpInvoiceData;

export function formatTdrpPollMessage(poll: Omit<TdrpPollData, "type">): string {
  return `\`\`\`tdrp-poll\n${JSON.stringify({ type: "tdrp-poll", ...poll }, null, 2)}\n\`\`\``;
}

export function formatTdrpTodoMessage(todo: Omit<TdrpTodoData, "type">): string {
  return `\`\`\`tdrp-todo\n${JSON.stringify({ type: "tdrp-todo", ...todo }, null, 2)}\n\`\`\``;
}

export function formatTdrpInvoiceMessage(invoice: Omit<TdrpInvoiceData, "type">): string {
  return `\`\`\`tdrp-invoice\n${JSON.stringify({ type: "tdrp-invoice", ...invoice }, null, 2)}\n\`\`\``;
}

export function parseTdrpInteractivePayload(text: string): TdrpInteractivePayload | null {
  if (!text || typeof text !== "string") return null;
  const trimmed = text.trim();

  const codeFenceMatch = trimmed.match(/```(?:tdrp-(poll|todo|invoice)|json)\s*([\s\S]*?)```/);
  const jsonCandidate = codeFenceMatch ? codeFenceMatch[2].trim() : trimmed;

  if (jsonCandidate.startsWith("{") && jsonCandidate.endsWith("}")) {
    try {
      const parsed = JSON.parse(jsonCandidate) as Record<string, unknown>;
      if (parsed.type === "tdrp-poll" || (parsed.pollId && parsed.question && Array.isArray(parsed.options))) {
        return {
          type: "tdrp-poll",
          pollId: String(parsed.pollId || ""),
          question: String(parsed.question || ""),
          options: (parsed.options as TdrpPollOption[]).map((o, idx) => ({
            id: typeof o.id === "number" ? o.id : idx,
            text: String(o.text || ""),
            voterCount: Number(o.voterCount || 0),
          })),
          isClosed: Boolean(parsed.isClosed),
          totalVoters: Number(parsed.totalVoters || 0),
          kind: (parsed.kind as TdrpPollKind) || { type: "Regular", multiple_answers: false },
          publicVotes: parsed.publicVotes !== undefined ? Boolean(parsed.publicVotes) : undefined,
          shuffleAnswers: parsed.shuffleAnswers !== undefined ? Boolean(parsed.shuffleAnswers) : undefined,
          revotingDisabled: parsed.revotingDisabled !== undefined ? Boolean(parsed.revotingDisabled) : undefined,
          closePeriodSecs: typeof parsed.closePeriodSecs === "number" ? parsed.closePeriodSecs : undefined,
          closeDateMs: typeof parsed.closeDateMs === "number" ? parsed.closeDateMs : undefined,
          selectedOptionIds: Array.isArray(parsed.selectedOptionIds) ? (parsed.selectedOptionIds as number[]) : [],
        };
      }
      if (parsed.type === "tdrp-todo" || (parsed.listId && parsed.title && Array.isArray(parsed.items))) {
        return {
          type: "tdrp-todo",
          listId: String(parsed.listId || ""),
          title: String(parsed.title || ""),
          othersCanAppend: parsed.othersCanAppend !== undefined ? Boolean(parsed.othersCanAppend) : true,
          othersCanComplete: parsed.othersCanComplete !== undefined ? Boolean(parsed.othersCanComplete) : true,
          items: (parsed.items as TdrpTodoItem[]).map((it, idx) => ({
            itemId: String(it.itemId || `item_${idx}`),
            text: String(it.text || ""),
            completed: Boolean(it.completed),
            completedBy: it.completedBy ? String(it.completedBy) : undefined,
            completionDateMs: typeof it.completionDateMs === "number" ? it.completionDateMs : undefined,
            assigneeId: it.assigneeId ? String(it.assigneeId) : undefined,
            dueDateMs: typeof it.dueDateMs === "number" ? it.dueDateMs : undefined,
          })),
        };
      }
      if (parsed.type === "tdrp-invoice" || (parsed.invoiceId && parsed.totalAmountCents !== undefined)) {
        return {
          type: "tdrp-invoice",
          invoiceId: String(parsed.invoiceId || ""),
          title: String(parsed.title || "Fabushi Invoice"),
          description: String(parsed.description || ""),
          currency: String(parsed.currency || "USD").toUpperCase(),
          totalAmountCents: Number(parsed.totalAmountCents || 0),
          isPaid: Boolean(parsed.isPaid),
          chargeId: parsed.chargeId ? String(parsed.chargeId) : undefined,
          prices: Array.isArray(parsed.prices) ? (parsed.prices as TdrpLabeledPrice[]) : undefined,
          suggestedTipAmounts: Array.isArray(parsed.suggestedTipAmounts) ? (parsed.suggestedTipAmounts as number[]) : undefined,
          maxTipAmount: typeof parsed.maxTipAmount === "number" ? parsed.maxTipAmount : undefined,
          isRecurring: parsed.isRecurring !== undefined ? Boolean(parsed.isRecurring) : undefined,
          receipt: parsed.receipt ? (parsed.receipt as TdrpPaymentReceipt) : undefined,
        };
      }
    } catch {
      // Ignore parse failure
    }
  }
  return null;
}
