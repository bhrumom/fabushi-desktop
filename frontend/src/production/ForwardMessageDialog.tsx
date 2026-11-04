import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent } from "react";
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

export interface ForwardDraftSource {
  readonly message: TranscriptMessage;
  readonly clientNonce: string;
}

export interface ForwardIntentSnapshot {
  readonly destinationConversationIds: readonly string[];
  readonly dropSenderNames: boolean;
  readonly dropCaptions: boolean;
  readonly failedBySource: Readonly<Record<string, readonly string[]>>;
}

export interface ForwardMessageDialogProps {
  readonly open: boolean;
  readonly accountScopeGeneration: number;
  readonly isAccountScopeCurrent: (generation: number) => boolean;
  readonly messages: readonly ForwardDraftSource[];
  readonly initialSnapshot?: ForwardIntentSnapshot | null;
  readonly sourceConversationId: string | null;
  readonly onClose: () => void;
  readonly onAddMore?: () => void;
  readonly onDiscard?: () => void;
  readonly onComplete?: () => void;
  readonly onRemoveSource?: (sourceEntryId: string) => void;
  readonly onSnapshot?: (snapshot: ForwardIntentSnapshot) => void;
  readonly searchRecipients: (input: {
    sourceConversationId: string;
    sourceEntryIds: readonly string[];
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
  }, isCurrent: () => boolean) => Promise<readonly ForwardSettlement[]>;
  readonly onSettled?: (settlement: readonly ForwardSettlement[], accountScopeGeneration: number) => void;
}

interface ForwardDialogLifetime {
  readonly accountScopeGeneration: number;
  readonly sourceConversationId: string;
}

// Match the existing Session request bound before freezing a retry intent.
// The Host still rechecks the bound and each destination's authorization.
const MAX_FORWARD_DESTINATIONS = 32;

export function ForwardMessageDialog({
  open,
  accountScopeGeneration,
  isAccountScopeCurrent,
  messages,
  initialSnapshot = null,
  sourceConversationId,
  onClose,
  onAddMore,
  onDiscard,
  onComplete,
  onRemoveSource,
  onSnapshot,
  searchRecipients,
  forwardMessage,
  onSettled,
}: ForwardMessageDialogProps) {
  const searchRef = useRef<HTMLInputElement | null>(null);
  const recipientListRef = useRef<HTMLDivElement | null>(null);
  const searchGenerationRef = useRef(0);
  const lifetimeRef = useRef<ForwardDialogLifetime | null>(null);
  const submittingLifetimeRef = useRef<ForwardDialogLifetime | null>(null);
  const snapshotRef = useRef<ForwardIntentSnapshot | null>(null);
  const initialSnapshotRef = useRef<ForwardIntentSnapshot | null>(initialSnapshot);
  initialSnapshotRef.current = initialSnapshot;
  const selectedRef = useRef<ReadonlySet<string>>(new Set());
  const [query, setQuery] = useState("");
  const [recipients, setRecipients] = useState<readonly ForwardRecipient[]>([]);
  const [selected, setSelected] = useState<ReadonlySet<string>>(() => new Set());
  const [dropSenderNames, setDropSenderNames] = useState(false);
  const [dropCaptions, setDropCaptions] = useState(false);
  const [searching, setSearching] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [snapshotLocked, setSnapshotLocked] = useState(false);
  const [searchFailure, setSearchFailure] = useState<string | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const [activeRecipientIndex, setActiveRecipientIndex] = useState(0);

  const sourceEntryIds = messages.map((source) => source.message.id);
  const sourceEntryIdsRef = useRef(sourceEntryIds);
  sourceEntryIdsRef.current = sourceEntryIds;
  const sourceEntryIdsKey = sourceEntryIds.join("\u0000");
  const sourceIdsValid = sourceEntryIds.length > 0
    && new Set(sourceEntryIds).size === sourceEntryIds.length
    && messages.every((source) => source.clientNonce.length > 0 && source.clientNonce.length <= 200);
  const scopeKey = open && sourceIdsValid && sourceConversationId != null
    ? JSON.stringify([accountScopeGeneration, sourceConversationId, sourceEntryIds])
    : null;

  const isCurrent = (lifetime: ForwardDialogLifetime): boolean =>
    lifetimeRef.current === lifetime && isAccountScopeCurrent(lifetime.accountScopeGeneration);
  const canInteract = (): boolean => {
    const lifetime = lifetimeRef.current;
    return lifetime != null && isCurrent(lifetime) && submittingLifetimeRef.current !== lifetime;
  };
  const canEditIntent = (): boolean => canInteract() && snapshotRef.current == null;
  const updateSelected = (next: ReadonlySet<string>) => {
    selectedRef.current = next;
    setSelected(next);
  };

  // A committed close/source/account change invalidates both asynchronous
  // lanes before paint. Object-only message refreshes preserve the same intent.
  useLayoutEffect(() => {
    lifetimeRef.current = null;
    submittingLifetimeRef.current = null;
    snapshotRef.current = null;
    searchGenerationRef.current += 1;
    setQuery("");
    setRecipients([]);
    updateSelected(new Set());
    setDropSenderNames(false);
    setDropCaptions(false);
    setSearching(false);
    setSubmitting(false);
    setSnapshotLocked(false);
    setSearchFailure(null);
    setFailure(null);
    setActiveRecipientIndex(0);
    if (scopeKey == null || sourceConversationId == null) return;
    const lifetime: ForwardDialogLifetime = {
      accountScopeGeneration,
      sourceConversationId,
    };
    lifetimeRef.current = lifetime;
    const initial = initialSnapshotRef.current;
    if (initial != null) {
      snapshotRef.current = initial;
      setDropSenderNames(initial.dropSenderNames);
      setDropCaptions(initial.dropCaptions);
      updateSelected(new Set(Object.values(initial.failedBySource).flat()));
      setSnapshotLocked(true);
    }
    return () => {
      if (lifetimeRef.current === lifetime) lifetimeRef.current = null;
      if (submittingLifetimeRef.current === lifetime) submittingLifetimeRef.current = null;
      searchGenerationRef.current += 1;
    };
  }, [accountScopeGeneration, scopeKey, sourceConversationId]);

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
        sourceEntryIds: sourceEntryIdsRef.current,
        query,
        limit: 50,
      })
        .then((next) => {
          if (!isSearchCurrent()) return;
          setRecipients(next);
          setActiveRecipientIndex((current) => next.length === 0 ? 0 : Math.min(current, next.length - 1));
          setSearchFailure(null);
        })
        .catch((error: unknown) => {
          if (!isSearchCurrent()) return;
          setRecipients([]);
          setSearchFailure(error instanceof Error ? error.message : String(error));
        })
        .finally(() => {
          if (isSearchCurrent()) setSearching(false);
        });
    }, query.length === 0 ? 0 : 150);
    return () => {
      window.clearTimeout(timer);
      if (generation === searchGenerationRef.current) searchGenerationRef.current += 1;
    };
  }, [query, scopeKey, searchRecipients, sourceEntryIdsKey]);

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
    if (!canEditIntent()) return;
    const next = new Set(selectedRef.current);
    if (next.has(id)) next.delete(id);
    else if (next.size < MAX_FORWARD_DESTINATIONS) next.add(id);
    else {
      setFailure(`Select up to ${MAX_FORWARD_DESTINATIONS} conversations. Remove one before adding another.`);
      return;
    }
    updateSelected(next);
    setFailure(null);
  };

  const submit = async () => {
    const lifetime = lifetimeRef.current;
    if (
      lifetime == null
      || !isCurrent(lifetime)
      || submittingLifetimeRef.current === lifetime
      || selectedRef.current.size === 0
      || !sourceIdsValid
    ) return;
    if (selectedRef.current.size > MAX_FORWARD_DESTINATIONS) {
      setFailure(`Select up to ${MAX_FORWARD_DESTINATIONS} conversations before forwarding.`);
      return;
    }
    const snapshot: ForwardIntentSnapshot = snapshotRef.current ?? {
      destinationConversationIds: [...selectedRef.current],
      dropSenderNames: dropSenderNames || dropCaptions,
      dropCaptions,
      failedBySource: Object.fromEntries(messages.map((source) => [
        source.message.id,
        [...selectedRef.current],
      ])),
    };
    snapshotRef.current = snapshot;
    submittingLifetimeRef.current = lifetime;
    setSnapshotLocked(true);
    setSubmitting(true);
    setFailure(null);
    try {
      const failedBySource: Record<string, readonly string[]> = {};
      const blockedDestinations = new Set<string>();
      let firstFailure: string | null = null;
      // Keep source selection order for each destination; later messages wait
      // behind an earlier failure and are retried after it settles.
      for (const source of messages) {
        if (!isCurrent(lifetime)) return;
        const requested = snapshot.failedBySource[source.message.id] ?? snapshot.destinationConversationIds;
        const blocked = requested.filter((id) => blockedDestinations.has(id));
        const attempt = requested.filter((id) => !blockedDestinations.has(id));
        const failed = new Set<string>(blocked);
        if (attempt.length > 0) {
          try {
            const settlement = await forwardMessage({
              sourceConversationId: lifetime.sourceConversationId,
              sourceEntryId: source.message.id,
              destinationConversationIds: attempt,
              clientNonce: source.clientNonce,
              dropSenderNames: snapshot.dropSenderNames,
              dropCaptions: snapshot.dropCaptions,
            }, () => isCurrent(lifetime));
            if (!isCurrent(lifetime)) return;
            const unexpected = settlement.some((item) => !attempt.includes(item.conversationId));
            for (const conversationId of attempt) {
              const results = settlement.filter((item) => item.conversationId === conversationId);
              if (unexpected || results.length !== 1 || results[0]?.status !== "sent") {
                failed.add(conversationId);
                if (firstFailure == null) firstFailure = results[0]?.error ?? "Delivery was not confirmed.";
              }
            }
          } catch (error) {
            if (!isCurrent(lifetime)) return;
            for (const conversationId of attempt) failed.add(conversationId);
            if (firstFailure == null) firstFailure = error instanceof Error ? error.message : String(error);
          }
        }
        const remaining = [...failed];
        failedBySource[source.message.id] = remaining;
        for (const conversationId of remaining) blockedDestinations.add(conversationId);
      }
      if (!isCurrent(lifetime)) return;
      const nextSnapshot: ForwardIntentSnapshot = { ...snapshot, failedBySource };
      snapshotRef.current = nextSnapshot;
      onSnapshot?.(nextSnapshot);
      const failedDestinations = new Set(Object.values(failedBySource).flat());
      updateSelected(failedDestinations);
      const aggregate = snapshot.destinationConversationIds.map((conversationId): ForwardSettlement => ({
        conversationId,
        status: failedDestinations.has(conversationId) ? "failed" : "sent",
        ...(failedDestinations.has(conversationId)
          ? { error: "One or more selected messages still need attention." }
          : {}),
      }));
      onSettled?.(aggregate, lifetime.accountScopeGeneration);
      if (!isCurrent(lifetime)) return;
      if (failedDestinations.size === 0) {
        onComplete?.();
        closeLifetime(lifetime);
        return;
      }
      const failedCount = Object.values(failedBySource).reduce((count, ids) => count + ids.length, 0);
      setFailure(firstFailure ?? `${failedCount} message deliveries still need attention. Retry sends only the unconfirmed message/destination pairs.`);
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
    // Closing retires presentation callbacks; an already accepted Host intent
    // remains durable and may settle after this dialog has gone away.
    if (lifetime != null) closeLifetime(lifetime);
  };
  const addMore = () => {
    if (snapshotRef.current != null || submitting) return;
    close();
    onAddMore?.();
  };
  const discard = () => {
    onDiscard?.();
    close();
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
        <h2>Forward ${messages.length === 1 ? "message" : "messages"}</h2>
        <p>Select one or more conversations. Every source message is checked independently; only recipients eligible for all selected messages are shown.</p>
      </header>
      <div className="fabushi-forward-dialog__body">
        <section aria-label="Messages to forward" style={{ display: "grid", gap: 6 }}>
          {messages.map((source) => <div key={source.message.id} style={{ alignItems: "center", display: "flex", gap: 8, minWidth: 0 }}>
            <span style={{ flex: 1, minWidth: 0, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {source.message.text.trim().length > 0
                ? source.message.text.trim()
                : `${source.message.attachments?.length ?? 0} attachment(s)`}
            </span>
            <SandButton
              aria-label={`Remove message ${source.message.id}`}
              disabled={snapshotLocked || submitting}
              onClick={() => onRemoveSource?.(source.message.id)}
              size="sm"
              variant="secondary"
            >Remove</SandButton>
          </div>)}
        </section>
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
                disabled={snapshotLocked}
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
              disabled={snapshotLocked}
              onChange={(event) => {
                if (!canEditIntent()) return;
                const checked = event.currentTarget.checked;
                setDropSenderNames(checked);
                if (!checked) setDropCaptions(false);
              }}
              type="checkbox"
            />
            <span>Hide sender name</span>
          </label>
          <label>
            <input
              checked={dropCaptions}
              disabled={snapshotLocked}
              onChange={(event) => {
                if (!canEditIntent()) return;
                const checked = event.currentTarget.checked;
                setDropCaptions(checked);
                if (checked) setDropSenderNames(true);
              }}
              type="checkbox"
            />
            <span>Hide media caption</span>
          </label>
        </fieldset>
        {failure == null && searchFailure == null ? null : <p className="fabushi-forward-dialog__error" role="alert">{failure ?? searchFailure}</p>}
        {snapshotLocked ? <p>Retry keeps your forwarding choices and skips confirmed recipients. Messages already being sent may still arrive after you close.</p> : null}
      </div>
      <footer>
        <span aria-live="polite">{selected.size === 0 ? "No recipients selected" : `${selected.size} selected`}</span>
        {onAddMore != null && !snapshotLocked ? <SandButton onClick={addMore} size="sm" variant="secondary">Add another message</SandButton> : null}
        <SandButton onClick={snapshotLocked ? close : discard} size="sm" variant="secondary">{snapshotLocked ? "Close" : "Cancel"}</SandButton>
        <SandButton disabled={selected.size === 0 || messages.length === 0} pending={submitting} size="sm" type="submit">
          {submitting ? "Forwarding…" : snapshotLocked ? "Retry" : "Forward"}
        </SandButton>
      </footer>
    </form>
  </OverlayDialog>;
}
