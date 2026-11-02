import { useCallback, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { ForwardMessageDialog, type ForwardRecipient, type ForwardSettlement } from "./ForwardMessageDialog";
import { useForwardAccountScopeUnmountFence } from "./forward-account-scope";
import type { TranscriptMessage } from "../recovered/features/conversation/workspace/model";
import "./production.css";

type RequestKind = "search" | "forward";
type RequestRecord = { readonly id: string; readonly kind: RequestKind; readonly args: unknown };
type PendingRequest = { readonly resolve: (value: unknown) => void; readonly reject: (error: Error) => void };
const calls: RequestRecord[] = [];
const pending = new Map<string, PendingRequest>();
const events: unknown[] = [];
let nextRequest = 0;

function controlled<T>(kind: RequestKind, args: unknown): Promise<T> {
  const id = `${kind}-${++nextRequest}`;
  calls.push({ id, kind, args });
  return new Promise<T>((resolve, reject) => {
    pending.set(id, { resolve: (value) => resolve(value as T), reject });
  });
}
const searchRecipients = (input: { sourceConversationId: string; sourceEntryId: string; query: string; limit: number }) =>
  controlled<readonly ForwardRecipient[]>("search", input);
const forwardMessage = (input: {
  sourceConversationId: string;
  sourceEntryId: string;
  destinationConversationIds: readonly string[];
  clientNonce: string;
  dropSenderNames: boolean;
  dropCaptions: boolean;
}) => controlled<readonly ForwardSettlement[]>("forward", input);

type Controls = {
  readonly generation: number;
  readonly scope: { current: number };
  readonly switchAccount: (account: string | null) => void;
  readonly reopen: () => void;
  readonly close: () => void;
  readonly replaceSource: (messageId: string) => void;
  readonly refreshCredential: () => void;
};
let controls: Controls | null = null;
const root = createRoot(document.getElementById("root")!);
function Fixture() {
  const [account, setAccount] = useState<string | null>("A");
  const [generation, setGeneration] = useState(1);
  const scope = useRef(1);
  const [open, setOpen] = useState(true);
  const [messageId, setMessageId] = useState("message-1");
  const [credentialRefresh, setCredentialRefresh] = useState(0);
  const accountRef = useRef<string | null>(account);
  useForwardAccountScopeUnmountFence(scope);
  const isCurrent = useCallback((value: number) =>
    value === scope.current && accountRef.current != null, []);
  const message = useMemo<TranscriptMessage>(() => ({
    id: messageId, kind: "message", role: "user", author: "You",
    text: "Forward this message", timestampMs: 1,
  }), [messageId]);
  controls = {
    generation,
    scope,
    switchAccount: (nextAccount) => {
      accountRef.current = nextAccount;
      scope.current += 1;
      setGeneration(scope.current);
      setAccount(nextAccount);
      setOpen(false);
    },
    reopen: () => { if (accountRef.current != null) setOpen(true); },
    close: () => setOpen(false),
    replaceSource: (nextId) => setMessageId(nextId),
    refreshCredential: () => setCredentialRefresh((value) => value + 1),
  };
  return <div data-account={account ?? "signed-out"} data-credential-refresh={credentialRefresh} data-generation={generation}>
    <ForwardMessageDialog
      accountScopeGeneration={generation}
      forwardMessage={forwardMessage}
      isAccountScopeCurrent={isCurrent}
      message={message}
      onClose={() => { events.push({ kind: "closed", account }); setOpen(false); }}
      onSettled={(settlement, scopedGeneration) => events.push({ kind: "settled", account, generation: scopedGeneration, settlement })}
      open={open}
      searchRecipients={searchRecipients}
      sourceConversationId="human:source"
    />
  </div>;
}
Object.assign(window, {
  __forwardHarness: {
    requests: () => [...calls],
    events: () => [...events],
    pending: () => [...pending.keys()],
    resolve: (id: string, value: unknown) => {
      const item = pending.get(id);
      if (item == null) throw new Error(`unrecognized controlled request: ${id}`);
      pending.delete(id);
      item.resolve(value);
    },
    reject: (id: string, reason: string) => {
      const item = pending.get(id);
      if (item == null) throw new Error(`unrecognized controlled request: ${id}`);
      pending.delete(id);
      item.reject(new Error(reason));
    },
    switchAccount: (account: string | null) => controls?.switchAccount(account),
    reopen: () => controls?.reopen(),
    close: () => controls?.close(),
    replaceSource: (id: string) => controls?.replaceSource(id),
    refreshCredential: () => controls?.refreshCredential(),
    unmount: () => {
      const before = controls?.scope.current ?? -1;
      root.unmount();
      return { before, after: controls?.scope.current ?? -1 };
    },
  },
});
root.render(<Fixture />);
