import { useEffect, useLayoutEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
import type { TranscriptMessage } from "../recovered/features/conversation/workspace/model";
import { OverlayDialog } from "../recovered/ui/overlay-primitives";
import { SandButton } from "../recovered/ui/sand-kit-primitives";
import {
  isForwardRecipientNavigationKey,
  isForwardSubmitShortcut,
  isForwardToggleShortcut,
  nextForwardRecipientIndex,
} from "./forward-recipient-navigation";

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
  readonly accountScopeGeneration: number;
  readonly isAccountScopeCurrent: (generation: number) => boolean;
  readonly message: TranscriptMessage | null;
  readonly sourceConversationId: string | null;
  readonly onClose: () => void;
  readonly searchRecipients: (input: {
    sourceConversationId: string;
    sourceEntryId: string;
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
  readonly onSettled?: (settlement: readonly ForwardSettlement[], accountScopeGeneration: number) => void;
}

interface ForwardDialogLifetime {
  readonly accountScopeGeneration: number;
  readonly sourceConversationId: string;
  readonly sourceEntryId: string;
  readonly requestId: string;
}

function createForwardRequestId(sourceConversationId: string, sourceEntryId: string): string {
  const suffix = typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
    ? crypto.randomUUID()
    : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  return `forward:${sourceConversationId}:${sourceEntryId}:${suffix}`;
}

export function ForwardMessageDialog({
  open,
  accountScopeGeneration,
  isAccountScopeCurrent,
  message,
  sourceConversationId,
  onClose,
  searchRecipients,
  forwardMessage,
  onSettled,
}: ForwardMessageDialogProps) {
  const searchRef = useRef<HTMLInputElement | null>(null);
  const recipientListRef = useRef<HTMLDivElement | null>(null);
  const searchGenerationRef = useRef(0);
  const lifetimeRef = useRef<ForwardDialogLifetime | null>(null);
  const submittingLifetimeRef = useRef<ForwardDialogLifetime | null>(null);
  const [query, setQuery] = useState("");
  const [recipients, setRecipients] = useState<readonly ForwardRecipient[]>([]);
  const [selected, setSelected] = useState<ReadonlySet<string>>(() => new Set());
  const [dropSenderNames, setDropSenderNames] = useState(false);
  const [dropCaptions, setDropCaptions] = useState(false);
  const [searching, setSearching] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  const [activeRecipientIndex, setActiveRecipientIndex] = useState(0);

  const sourceEntryId = message?.id ?? null;
  const scopeKey = open && sourceEntryId != null && sourceConversationId != null
    ? JSON.stringify([accountScopeGeneration, sourceConversationId, sourceEntryId])
    : null;

  const isCurrent = (lifetime: ForwardDialogLifetime): boolean =>
    lifetimeRef.current === lifetime && isAccountScopeCurrent(lifetime.accountScopeGeneration);
  const canInteract = (): boolean => {
    const lifetime = lifetimeRef.current;
    return lifetime != null && isCurrent(lifetime) && submittingLifetimeRef.current !== lifetime;
  };

  // A committed close/source/account change invalidates both asynchronous
  // lanes before paint. Object-only message refreshes preserve the same intent.
  useLayoutEffect(() => {
    lifetimeRef.current = null;
    submittingLifetimeRef.current = null;
    searchGenerationRef.current += 1;
    setQuery("");
    setRecipients([]);
    setSelected(new Set());
    setDropSenderNames(false);
    setDropCaptions(false);
    setSearching(false);
    setSubmitting(false);
    setFailure(null);
    setActiveRecipientIndex(0);
    if (scopeKey == null || sourceEntryId == null || sourceConversationId == null) return;
    const lifetime: ForwardDialogLifetime = {
      accountScopeGeneration,
      sourceConversationId,
      sourceEntryId,
      requestId: createForwardRequestId(sourceConversationId, sourceEntryId),
    };
    lifetimeRef.current = lifetime;
    return () => {
      if (lifetimeRef.current === lifetime) lifetimeRef.current = null;
      if (submittingLifetimeRef.current === lifetime) submittingLifetimeRef.current = null;
      searchGenerationRef.current += 1;
    };
  }, [accountScopeGeneration, scopeKey, sourceConversationId, sourceEntryId]);

  useEffect(() => {
    const lifetime = lifetimeRef.current;
    if (lifetime == null || !isCurrent(lifetime)) return;
    const generation = ++searchGenerationRef.current;
    const isSearchCurrent = () => isCurrent(lifetime) && generation === searchGenerationRef.current;
    const timer = window.setTimeout(() => {
      if (!isSearchCurrent()) return;
      setSearching(true);
      void searchRecipients({
        sourceConversationId: lifetime.sourceConversationId,
        sourceEntryId: lifetime.sourceEntryId,
        query,
        limit: 50,
      })
        .then((next) => {
          if (!isSearchCurrent()) return;
          setRecipients(next);
          setActiveRecipientIndex((current) => next.length === 0 ? 0 : Math.min(current, next.length - 1));
          setFailure(null);
        })
        .catch((error: unknown) => {
          if (!isSearchCurrent()) return;
          setRecipients([]);
          setFailure(error instanceof Error ? error.message : String(error));
        })
        .finally(() => {
          if (isSearchCurrent()) setSearching(false);
        });
    }, query.length === 0 ? 0 : 150);
    return () => {
      window.clearTimeout(timer);
      if (generation === searchGenerationRef.current) searchGenerationRef.current += 1;
    };
  }, [query, scopeKey, searchRecipients]);

  const selectedRecipients = useMemo(
    () => recipients.filter((recipient) => selected.has(recipient.id)),
    [recipients, selected],
  );

  const focusActiveRecipient = (index: number) => {
    const lifetime = lifetimeRef.current;
    window.requestAnimationFrame(() => {
      if (lifetime == null || !isCurrent(lifetime)) return;
      recipientListRef.current
        ?.querySelector<HTMLElement>(`[data-forward-recipient-index="${index}"]`)
        ?.scrollIntoView({ block: "nearest" });
    });
  };

  const handleRecipientKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (!canInteract() || recipients.length === 0) return;
    if (isForwardSubmitShortcut(event)) {
      event.preventDefault();
      void submit();
      return;
    }
    if (isForwardRecipientNavigationKey(event.key)) {
      event.preventDefault();
      const next = nextForwardRecipientIndex(activeRecipientIndex, recipients.length, event.key);
      if (next != null) {
        setActiveRecipientIndex(next);
        focusActiveRecipient(next);
      }
      return;
    }
    if (isForwardToggleShortcut(event)) {
      event.preventDefault();
      const recipient = recipients[activeRecipientIndex];
      if (recipient != null) toggleRecipient(recipient.id);
    }
  };

  const toggleRecipient = (id: string) => {
    if (!canInteract()) return;
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const submit = async () => {
    const lifetime = lifetimeRef.current;
    if (
      lifetime == null
      || !isCurrent(lifetime)
      || submittingLifetimeRef.current === lifetime
      || selected.size === 0
    ) return;
    // React may batch two submit events before repainting `submitting`.
    submittingLifetimeRef.current = lifetime;
    setSubmitting(true);
    setFailure(null);
    try {
      const settlement = await forwardMessage({
        sourceConversationId: lifetime.sourceConversationId,
        sourceEntryId: lifetime.sourceEntryId,
        destinationConversationIds: [...selected],
        clientNonce: lifetime.requestId,
        dropSenderNames,
        dropCaptions,
      });
      if (!isCurrent(lifetime)) return;
      onSettled?.(settlement, lifetime.accountScopeGeneration);
      if (!isCurrent(lifetime)) return;
      const failed = settlement.filter((item) => item.status === "failed");
      if (failed.length === 0) {
        closeLifetime(lifetime);
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
      if (isCurrent(lifetime)) setFailure(error instanceof Error ? error.message : String(error));
    } finally {
      if (isCurrent(lifetime) && submittingLifetimeRef.current === lifetime) {
        submittingLifetimeRef.current = null;
        setSubmitting(false);
      }
    }
  };

  const closeLifetime = (lifetime: ForwardDialogLifetime) => {
    if (!isCurrent(lifetime)) return;
    lifetimeRef.current = null;
    submittingLifetimeRef.current = null;
    searchGenerationRef.current += 1;
    onClose();
  };
  const close = () => {
    const lifetime = lifetimeRef.current;
    if (lifetime != null && submittingLifetimeRef.current !== lifetime) closeLifetime(lifetime);
  };

  return <OverlayDialog
    className="fabushi-forward-dialog"
    initialFocusRef={searchRef}
    label="Forward message"
    onClose={close}
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
            onChange={(event) => {
              const lifetime = lifetimeRef.current;
              if (lifetime != null && isCurrent(lifetime)) setQuery(event.currentTarget.value);
            }}
            placeholder="Search conversations"
            ref={searchRef}
            type="search"
            value={query}
          />
        </label>
        <div
          aria-activedescendant={recipients.length === 0 ? undefined : `fabushi-forward-recipient-${activeRecipientIndex}`}
          aria-busy={searching || undefined}
          aria-label="Forward recipients"
          aria-multiselectable="true"
          className="fabushi-forward-dialog__recipients"
          onKeyDown={handleRecipientKeyDown}
          ref={recipientListRef}
          role="listbox"
          tabIndex={0}
        >
          {recipients.length === 0 && !searching
            ? <p role="status">{query.length === 0 ? "No eligible conversations." : "No eligible conversations match this search."}</p>
            : recipients.map((recipient, index) => <label
              aria-selected={selected.has(recipient.id)}
              className="fabushi-forward-dialog__recipient"
              data-active={activeRecipientIndex === index ? "true" : undefined}
              data-forward-recipient-index={index}
              id={`fabushi-forward-recipient-${index}`}
              key={recipient.id}
              onMouseEnter={() => { if (canInteract()) setActiveRecipientIndex(index); }}
              role="option"
            >
              <input
                checked={selected.has(recipient.id)}
                disabled={submitting}
                onChange={() => toggleRecipient(recipient.id)}
                tabIndex={-1}
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
              onChange={(event) => { if (canInteract()) setDropSenderNames(event.currentTarget.checked); }}
              type="checkbox"
            />
            <span>Hide sender name</span>
          </label>
          <label>
            <input
              checked={dropCaptions}
              disabled={submitting}
              onChange={(event) => {
                if (!canInteract()) return;
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
        <SandButton disabled={submitting} onClick={close} size="sm" variant="secondary">Cancel</SandButton>
        <SandButton disabled={selected.size === 0} pending={submitting} size="sm" type="submit">
          {submitting ? "Forwarding…" : "Forward"}
        </SandButton>
      </footer>
    </form>
  </OverlayDialog>;
}
