import { useCallback, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { ForwardMessageDialog, type ForwardRecipient, type ForwardSettlement } from "./ForwardMessageDialog";
import { useForwardAccountScopeUnmountFence } from "./forward-account-scope";
import { createCoordinatorClient, type ProductionCoordinatorClient } from "./coordinator-client";
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
let coordinator: ProductionCoordinatorClient | null = null;
let receiveCoordinator: ((data: unknown) => void) | null = null;
let coordinatorAttempts = 0;
let coordinatorCompletions = 0;

// Only the transferred port is controlled. Admission and pending/reply handling
// execute in the same production Coordinator client used by the renderer.
function enableCoordinator() {
  if (coordinator != null) throw new Error("Coordinator fixture is already enabled");
  coordinator = createCoordinatorClient({
    claim(consumer) {
      let supplied = false;
      return {
        request() {
          if (supplied) return;
          supplied = true;
          consumer.onPort({
            postMessage(frame) {
              const value = frame as { kind: string; requestId: string; method: string; args: unknown };
              if (value.kind !== "request") return;
              if (value.method !== "forwardHumanMessage") throw new Error(`unexpected Coordinator method ${value.method}`);
              const id = `coordinator-${value.requestId}`;
              calls.push({ id, kind: "forward", args: value.args });
              pending.set(id, {
                resolve: (destinations) => receiveCoordinator?.({ kind: "reply", requestId: value.requestId,
                  outcome: { status: "ok", value: { destinations } } }),
                reject: (error) => receiveCoordinator?.({ kind: "reply", requestId: value.requestId,
                  outcome: { status: "failed", failure: { code: "fixture", message: error.message } } }),
              });
            },
            start() {},
            close() {},
            addEventListener(type, listener) {
              if (type === "message") receiveCoordinator = (data) =>
                (listener as (event: { data: unknown }) => void)({ data });
            },
          });
        },
        release() {},
      };
    },
  });
  if (coordinator == null) throw new Error("Coordinator fixture could not claim its port");
}

function controlled<T>(kind: RequestKind, args: unknown): Promise<T> {
  const id = `${kind}-${++nextRequest}`;
  calls.push({ id, kind, args });
  return new Promise<T>((resolve, reject) => {
    pending.set(id, { resolve: (value) => resolve(value as T), reject });
  });
}
const searchRecipients = (input: { sourceConversationId: string; sourceEntryId: string; query: string; limit: number }) =>
  controlled<readonly ForwardRecipient[]>("search", input);
const forwardMessage = async (input: {
  sourceConversationId: string;
  sourceEntryId: string;
  destinationConversationIds: readonly string[];
  clientNonce: string;
  dropSenderNames: boolean;
  dropCaptions: boolean;
}, isCurrent: () => boolean): Promise<readonly ForwardSettlement[]> => {
  if (coordinator == null) return controlled<readonly ForwardSettlement[]>("forward", input);
  coordinatorAttempts += 1;
  try {
    const result = await coordinator.call("forwardHumanMessage", input, isCurrent) as { destinations: readonly ForwardSettlement[] };
    return result.destinations;
  } finally {
    coordinatorCompletions += 1;
  }
};

type Controls = {
  readonly generation: number;
  readonly scope: { current: number };
  readonly switchAccount: (account: string | null) => void;
  readonly reopen: () => void;
  readonly close: () => void;
  readonly replaceSource: (messageId: string) => void;
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
  const [messageId, setMessageId] = useState("message-1");
  const [messageRefresh, setMessageRefresh] = useState(0);
  const [credentialRefresh, setCredentialRefresh] = useState(0);
  const accountRef = useRef<string | null>(account);
  useForwardAccountScopeUnmountFence(scope);
  const isCurrent = useCallback((value: number) =>
    value === scope.current && accountRef.current != null, []);
  const message = useMemo<TranscriptMessage>(() => ({
    id: messageId, kind: "message", role: "user", author: "You",
    text: "Forward this message", timestampMs: 1 + messageRefresh,
  }), [messageId, messageRefresh]);
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
    enableCoordinator,
    coordinatorState: () => ({ attempts: coordinatorAttempts, completions: coordinatorCompletions }),
    readyCoordinator: async () => {
      if (receiveCoordinator == null) throw new Error("Coordinator fixture has no port");
      receiveCoordinator({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    },
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
