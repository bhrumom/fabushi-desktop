import { useCallback, useMemo, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { ForwardMessageDialog, type ForwardRecipient, type ForwardSettlement } from "./ForwardMessageDialog";
import { createCoordinatorClient } from "./coordinator-client";
import { useForwardAccountScopeUnmountFence } from "./forward-account-scope";
import type { CoordinatorPortBridge, TransferredCoordinatorPort } from "../recovered/contracts/desktop-bridge";
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

type RecordedCoordinatorFrame = {
  readonly portId: string;
  readonly frame: unknown;
};
type CoordinatorForwardOutcome = {
  readonly id: string;
  readonly status: "resolved" | "rejected";
};
const coordinatorFrames: RecordedCoordinatorFrame[] = [];
const coordinatorOutcomes: CoordinatorForwardOutcome[] = [];
const coordinatorPorts = new Map<string, ControlledCoordinatorPort>();
let coordinatorConsumer: Parameters<CoordinatorPortBridge["claim"]>[0] | null = null;
let currentCoordinatorPort: ControlledCoordinatorPort | null = null;
let nextCoordinatorPort = 0;

// Use the shipping client, retaining only the transport endpoint as the test
// double. structuredClone enforces the same no-callback wire boundary as a
// real MessagePort; a local dispatch guard must never enter request args.
class ControlledCoordinatorPort implements TransferredCoordinatorPort {
  readonly id = `port-${++nextCoordinatorPort}`;
  private readonly messageListeners = new Set<(event: { data: unknown }) => void>();
  private readonly closeListeners = new Set<(event: Record<string, never>) => void>();
  postMessage(frame: unknown): void {
    coordinatorFrames.push({ portId: this.id, frame: structuredClone(frame) });
  }
  close(): void {}
  start(): void {}
  addEventListener(type: "message", listener: (event: { data: unknown }) => void): void;
  addEventListener(type: "close", listener: (event: Record<string, never>) => void): void;
  addEventListener(
    type: "message" | "close",
    listener: ((event: { data: unknown }) => void) | ((event: Record<string, never>) => void),
  ): void {
    if (type === "message") this.messageListeners.add(listener as (event: { data: unknown }) => void);
    else this.closeListeners.add(listener as (event: Record<string, never>) => void);
  }
  receive(frame: unknown): void {
    for (const listener of this.messageListeners) listener({ data: frame });
  }
  disconnect(): void {
    for (const listener of this.closeListeners) listener({});
  }
}

const useCoordinatorTransport = new URLSearchParams(window.location.search).get("transport") === "coordinator";
const coordinatorClient = useCoordinatorTransport ? createCoordinatorClient({
  claim: (consumer) => {
    coordinatorConsumer = consumer;
    // A request deliberately does not supply a port until the test releases
    // the readiness barrier. This is the real client's reconnect wait.
    return { request: () => {}, release: () => {} };
  },
}) : null;
const afterRender = () => new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));

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
}, options?: { readonly isDispatchCurrent?: () => boolean }): Promise<readonly ForwardSettlement[]> => {
  if (!useCoordinatorTransport) return controlled<readonly ForwardSettlement[]>("forward", input);
  if (coordinatorClient == null) throw new Error("controlled coordinator client is unavailable");
  const id = `forward-${++nextRequest}`;
  calls.push({ id, kind: "forward", args: input });
  return coordinatorClient.call("forwardHumanMessage", input, options).then((value) => {
    coordinatorOutcomes.push({ id, status: "resolved" });
    return (value as { destinations: readonly ForwardSettlement[] }).destinations;
  }, (error: unknown) => {
    coordinatorOutcomes.push({ id, status: "rejected" });
    throw error;
  });
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
    coordinatorFrames: () => [...coordinatorFrames],
    coordinatorOutcomes: () => [...coordinatorOutcomes],
    connectCoordinator: () => {
      if (coordinatorConsumer == null) throw new Error("fixture is not using the coordinator transport");
      const port = new ControlledCoordinatorPort();
      coordinatorPorts.set(port.id, port);
      currentCoordinatorPort = port;
      coordinatorConsumer.onPort(port);
      return port.id;
    },
    readyCoordinator: async () => {
      if (currentCoordinatorPort == null) throw new Error("controlled coordinator port is missing");
      currentCoordinatorPort.receive({ kind: "lifecycle", phase: "ready", protocolVersion: 1 });
      await afterRender();
    },
    disconnectCoordinator: async () => {
      if (currentCoordinatorPort == null) throw new Error("controlled coordinator port is missing");
      const port = currentCoordinatorPort;
      currentCoordinatorPort = null;
      port.disconnect();
      await afterRender();
    },
    replyCoordinator: async (requestId: string, settlement: readonly ForwardSettlement[]) => {
      const request = coordinatorFrames.find(({ frame }) => {
        const value = frame as { kind?: string; requestId?: string };
        return value.kind === "request" && value.requestId === requestId;
      });
      const port = request == null ? null : coordinatorPorts.get(request.portId);
      if (port == null) throw new Error(`unrecognized coordinator request: ${requestId}`);
      port.receive({ kind: "reply", requestId, outcome: { status: "ok", value: { destinations: settlement } } });
      await afterRender();
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
