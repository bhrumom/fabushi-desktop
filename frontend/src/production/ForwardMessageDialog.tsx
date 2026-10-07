import { useEffect, useMemo, useRef, useState } from "react";
import type { TranscriptMessage } from "../recovered/features/conversation/workspace/model";
import { OverlayDialog } from "../recovered/ui/overlay-primitives";
import { SandButton } from "../recovered/ui/sand-kit-primitives";

export interface ForwardRecipient {
  readonly id: string;
  readonly name: string;
}

export interface ForwardSettlement {
  readonly conversationId: string;
  readonly status: "sent" | "failed";
  readonly error?: string;
}

export interface ForwardMessageDialogProps {
  readonly open: boolean;
  readonly message: TranscriptMessage | null;
  readonly sourceConversationId: string | null;
  readonly onClose: () => void;
  readonly searchRecipients: (input: {
    sourceConversationId: string;
    query: string;
    limit: number;
  }) => Promise<readonly ForwardRecipient[]>;
  readonly forwardMessage: (input: {
    sourceConversationId: string;
    sourceEntryId: string;
    destinationConversationIds: readonly string[];
    clientNonce: string;
    dropSenderNames: boolean;
    dropCaptions: boolean;
  }) => Promise<readonly ForwardSettlement[]>;
  readonly onSettled?: (settlement: readonly ForwardSettlement[]) => void;
}

function createForwardRequestId(sourceConversationId: string, sourceEntryId: string): string {
  const suffix = typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  return `forward:${sourceConversationId}:${sourceEntryId}:${suffix}`;
}

export function ForwardMessageDialog({
  open,
  message,
  sourceConversationId,
  onClose,
  searchRecipients,
  forwardMessage,
  onSettled,
}: ForwardMessageDialogProps) {
  const searchRef = useRef<HTMLInputElement | null>(null);
  const generationRef = useRef(0);
  const [query, setQuery] = useState("");
  const [recipients, setRecipients] = useState<readonly ForwardRecipient[]>([]);
  const [selected, setSelected] = useState<ReadonlySet<string>>(() => new Set());
  const [dropSenderNames, setDropSenderNames] = useState(false);
  const [dropCaptions, setDropCaptions] = useState(false);
  const [searching, setSearching] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  const [requestId, setRequestId] = useState<string | null>(null);

  const scopeKey = open && message != null && sourceConversationId != null
    ? `${sourceConversationId}:${message.id}`
    : null;

  useEffect(() => {
    if (scopeKey == null || message == null || sourceConversationId == null) {
      generationRef.current += 1;
      setQuery("");
      setRecipients([]);
      setSelected(new Set());
      setDropSenderNames(false);
      setDropCaptions(false);
      setFailure(null);
      setRequestId(null);
      return;
    }
    generationRef.current += 1;
    setQuery("");
    setRecipients([]);
    setSelected(new Set());
    setDropSenderNames(false);
    setDropCaptions(false);
    setFailure(null);
    setRequestId(createForwardRequestId(sourceConversationId, message.id));
  }, [scopeKey, message, sourceConversationId]);

  useEffect(() => {
    if (scopeKey == null || sourceConversationId == null) return;
    const generation = ++generationRef.current;
    const timer = window.setTimeout(() => {
      setSearching(true);
      void searchRecipients({ sourceConversationId, query, limit: 50 })
        .then((next) => {
          if (generation !== generationRef.current) return;
          setRecipients(next);
          setFailure(null);
        })
        .catch((error: unknown) => {
          if (generation !== generationRef.current) return;
          setRecipients([]);
          setFailure(error instanceof Error ? error.message : String(error));
        })
        .finally(() => {
          if (generation === generationRef.current) setSearching(false);
        });
    }, query.length === 0 ? 0 : 150);
    return () => window.clearTimeout(timer);
  }, [query, scopeKey, searchRecipients, sourceConversationId]);

  const selectedRecipients = useMemo(
    () => recipients.filter((recipient) => selected.has(recipient.id)),
    [recipients, selected],
  );

  const toggleRecipient = (id: string) => {
    if (submitting) return;
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const submit = async () => {
    if (
      submitting
      || message == null
      || sourceConversationId == null
      || requestId == null
      || selected.size === 0
    ) return;
    setSubmitting(true);
    setFailure(null);
    try {
      const settlement = await forwardMessage({
        sourceConversationId,
        sourceEntryId: message.id,
        destinationConversationIds: [...selected],
        clientNonce: requestId,
        dropSenderNames,
        dropCaptions,
      });
      onSettled?.(settlement);
      const failed = settlement.filter((item) => item.status === "failed");
      if (failed.length === 0) {
        onClose();
        return;
      }
      const succeeded = new Set(
        settlement.filter((item) => item.status === "sent").map((item) => item.conversationId),
      );
      setSelected((current) => new Set([...current].filter((id) => !succeeded.has(id))));
      setFailure(
        failed.length === 1
          ? (failed[0]?.error ?? "One destination could not be forwarded.")
          : `${failed.length} destinations could not be forwarded. Retry will reuse the same request identity.`,
      );
    } catch (error) {
      setFailure(error instanceof Error ? error.message : String(error));
    } finally {
      setSubmitting(false);
    }
  };

  return <OverlayDialog
    className="fabushi-forward-dialog"
    initialFocusRef={searchRef}
    label="Forward message"
    onClose={() => { if (!submitting) onClose(); }}
    open={open}
  >
    <form onSubmit={(event) => { event.preventDefault(); void submit(); }}>
      <header>
        <h2>Forward message</h2>
        <p>Select one or more conversations. Send eligibility is checked before results are exposed and again at send time.</p>
      </header>
      <div className="fabushi-forward-dialog__body">
        <label className="fabushi-forward-dialog__search">
          <span>Recipients</span>
          <input
            aria-busy={searching || undefined}
            autoComplete="off"
            onChange={(event) => setQuery(event.currentTarget.value)}
            placeholder="Search conversations"
            ref={searchRef}
            type="search"
            value={query}
          />
        </label>
        <div aria-busy={searching || undefined} aria-label="Forward recipients" className="fabushi-forward-dialog__recipients" role="group">
          {recipients.length === 0 && !searching
            ? <p role="status">{query.length === 0 ? "No eligible conversations." : "No eligible conversations match this search."}</p>
            : recipients.map((recipient) => <label className="fabushi-forward-dialog__recipient" key={recipient.id}>
              <input
                checked={selected.has(recipient.id)}
                disabled={submitting}
                onChange={() => toggleRecipient(recipient.id)}
                type="checkbox"
              />
              <span>{recipient.name}</span>
            </label>)}
        </div>
        <fieldset className="fabushi-forward-dialog__privacy">
          <legend>Forward options</legend>
          <label>
            <input
              checked={dropSenderNames}
              disabled={submitting || dropCaptions}
              onChange={(event) => setDropSenderNames(event.currentTarget.checked)}
              type="checkbox"
            />
            <span>Hide sender name</span>
          </label>
          <label>
            <input
              checked={dropCaptions}
              disabled={submitting}
              onChange={(event) => {
                const checked = event.currentTarget.checked;
                setDropCaptions(checked);
                if (checked) setDropSenderNames(true);
              }}
              type="checkbox"
            />
            <span>Hide media caption</span>
          </label>
        </fieldset>
        {failure == null ? null : <p className="fabushi-forward-dialog__error" role="alert">{failure}</p>}
      </div>
      <footer>
        <span aria-live="polite">{selectedRecipients.length === 0 ? "No recipients selected" : `${selectedRecipients.length} selected`}</span>
        <SandButton disabled={submitting} onClick={onClose} size="sm" variant="secondary">Cancel</SandButton>
        <SandButton disabled={selected.size === 0} pending={submitting} size="sm" type="submit">
          {submitting ? "Forwarding…" : "Forward"}
        </SandButton>
      </footer>
    </form>
  </OverlayDialog>;
}
