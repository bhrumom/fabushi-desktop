import { useCallback, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { ForwardMessageDialog, type ForwardRecipient, type ForwardSettlement } from "./ForwardMessageDialog";
import { useForwardAccountScopeUnmountFence } from "./forward-account-scope";
import { projectHumanConversationTranscript } from "./model";
import { createRuntimeThemeInstaller, type ThemeDocument } from "../recovered/features/runtime-theme-token-installer";
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

type SourceMessage = {
  readonly id: string;
  readonly content: string;
  readonly attachments?: readonly { path: string; name: string; mimeType?: string; sizeBytes?: number }[];
};

type Controls = {
  readonly generation: number;
  readonly scope: { current: number };
  readonly switchAccount: (account: string | null) => void;
  readonly reopen: () => void;
  readonly close: () => void;
  readonly replaceSource: (messageId: string) => void;
  readonly setSource: (source: SourceMessage) => void;
  readonly refreshMessage: () => void;
  readonly refreshCredential: () => void;
};
let controls: Controls | null = null;
const themeInstaller = createRuntimeThemeInstaller(document as unknown as ThemeDocument, "light");
const root = createRoot(document.getElementById("root")!);
function Fixture() {
  const [account, setAccount] = useState<string | null>("A");
  const [generation, setGeneration] = useState(1);
  const scope = useRef(1);
  const [open, setOpen] = useState(true);
  const [source, setSource] = useState<SourceMessage>({ id: "message-1", content: "Forward this message" });
  const [messageRefresh, setMessageRefresh] = useState(0);
  const [credentialRefresh, setCredentialRefresh] = useState(0);
  const accountRef = useRef<string | null>(account);
  useForwardAccountScopeUnmountFence(scope);
  const isCurrent = useCallback((value: number) =>
    value === scope.current && accountRef.current != null, []);
  const message = useMemo<TranscriptMessage>(() => {
    // Exercise the same Host row -> TranscriptMessage projection used by
    // ProductionRenderer. Caption availability is never a fixture-only flag.
    const projected = projectHumanConversationTranscript([{
      ...source, kind: "message", role: "user", authorKind: "human", authorId: "1",
      delivery: "sent", timestampMs: 1 + messageRefresh,
    }], "Human conversation", "human:source")[0];
    if (projected?.kind !== "message") throw new Error("forward fixture requires a projected Human message");
    return projected;
  }, [source, messageRefresh]);
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
    replaceSource: (nextId) => setSource((current) => ({ ...current, id: nextId })),
    setSource,
    refreshMessage: () => setMessageRefresh((value) => value + 1),
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
    resolve: async (id: string, value: unknown) => {
      const item = pending.get(id);
      if (item == null) throw new Error(`unrecognized controlled request: ${id}`);
      pending.delete(id);
      item.resolve(value);
      // Let consumers render before a negative assertion reads the new scope.
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    },
    reject: async (id: string, reason: string) => {
      const item = pending.get(id);
      if (item == null) throw new Error(`unrecognized controlled request: ${id}`);
      pending.delete(id);
      item.reject(new Error(reason));
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    },
    switchAccount: (account: string | null) => controls?.switchAccount(account),
    reopen: () => controls?.reopen(),
    close: () => controls?.close(),
    replaceSource: (id: string) => controls?.replaceSource(id),
    setSource: async (source: SourceMessage) => {
      controls?.setSource(source);
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    },
    refreshMessage: () => controls?.refreshMessage(),
    refreshCredential: () => controls?.refreshCredential(),
    setTheme: (mode: "light" | "dark") => themeInstaller.update(mode),
    unmount: () => {
      const before = controls?.scope.current ?? -1;
      root.unmount();
      themeInstaller.dispose();
      return { before, after: controls?.scope.current ?? -1 };
    },
  },
});
root.render(<Fixture />);
