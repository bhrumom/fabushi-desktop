import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import {
  mergeAuthoritativeTranscriptMessage,
  reconcileAuthoritativeTranscriptBaseline,
  reconcileLateInitialTranscriptPage,
  reconcileTranscriptFeedBaseline,
  reconcileTranscriptLiveAppend,
  reconcileTranscriptLiveUpdate,
} from "./transcript-reconciliation.ts";
import type { ConversationTranscriptEntry, TranscriptMessage } from "./model.ts";

const PROPERTY_SEED = 0x52454302;
const NONCE = "nonce-a";
const ATTACHMENT = {
  path: "/tmp/report.pdf",
  name: "report.pdf",
  size: 4096,
  mimeType: "application/pdf",
};

function message(id: string, overrides: Partial<TranscriptMessage> = {}): TranscriptMessage {
  return {
    kind: "message",
    id,
    role: "assistant",
    author: "Agent",
    text: "",
    timestampMs: 0,
    ...overrides,
  };
}

type AttachmentEcho = TranscriptMessage & { sourceKind: "user-attachment" };

const history = message("history-1", {
  role: "assistant",
  author: "Agent",
  text: "Earlier context",
  timestampMs: 1,
});
const pendingUser = message("pending-" + NONCE, {
  role: "user",
  author: "You",
  text: "Uploading report.pdf",
  clientNonce: NONCE,
  delivery: "dispatching",
  attachments: [ATTACHMENT],
  timestampMs: 2,
});
const liveAssistant = message("answer-1", {
  role: "assistant",
  author: "Agent",
  text: "Final answer",
  clientNonce: "answer-nonce",
  isStreaming: true,
  timestampMs: 3,
});
const authoritativeUser = message("user-1", {
  role: "user",
  author: "You",
  text: "Uploaded report.pdf",
  clientNonce: NONCE,
  delivery: "sent",
  attachments: [ATTACHMENT],
  timestampMs: 2,
});
const staleStreamingBaseline = message("answer-1", {
  role: "assistant",
  author: "Agent",
  text: "Final",
  clientNonce: "answer-nonce",
  isStreaming: true,
  timestampMs: 3,
});
const terminalAssistant = message("answer-1", {
  role: "assistant",
  author: "Agent",
  text: "Final answer",
  clientNonce: "answer-nonce",
  isStreaming: false,
  timestampMs: 4,
});
const staleStreamingUpdate = message("answer-1", {
  role: "assistant",
  author: "Agent",
  text: "Final",
  clientNonce: "answer-nonce",
  isStreaming: true,
  timestampMs: 3,
});
const attachmentEcho: AttachmentEcho = {
  ...message("attachment-echo", {
    role: "user",
    author: "You",
    text: "",
    clientNonce: NONCE,
    delivery: "sent",
    attachments: [{ path: ATTACHMENT.path, name: ATTACHMENT.name }],
    timestampMs: 2,
  }),
  sourceKind: "user-attachment",
};

function semanticEntries(entries: readonly ConversationTranscriptEntry[]): unknown[] {
  return entries.map((entry) => {
    if (entry.kind !== "message") return { kind: entry.kind, id: entry.id };
    return {
      kind: entry.kind,
      id: entry.id,
      role: entry.role,
      text: entry.text,
      clientNonce: entry.clientNonce ?? null,
      streaming: entry.isStreaming === true,
      delivery: entry.delivery ?? null,
      attachments: (entry.attachments ?? []).map((attachment) => ({
        path: attachment.path,
        name: attachment.name,
        size: attachment.size ?? null,
        mimeType: attachment.mimeType ?? null,
      })),
    };
  });
}

function seededRandom(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    return state / 0x100000000;
  };
}

function shuffle<T>(items: readonly T[], random: () => number): T[] {
  const result = [...items];
  for (let index = result.length - 1; index > 0; index -= 1) {
    const swapIndex = Math.floor(random() * (index + 1));
    [result[index], result[swapIndex]] = [result[swapIndex]!, result[index]!];
  }
  return result;
}

test("REC-I02 seeded event reorders converge across baseline/live/update/replay permutations", () => {
  const baseline: readonly ConversationTranscriptEntry[] = [
    history,
    authoritativeUser,
    staleStreamingBaseline,
  ];
  const latePage: readonly ConversationTranscriptEntry[] = [
    message("history-0", { text: "Oldest context", timestampMs: 0 }),
    history,
  ];
  const eventKinds = [
    "baseline",
    "attachment-echo",
    "terminal-update",
    "attachment-replay",
    "stale-update",
  ] as const;
  const random = seededRandom(PROPERTY_SEED);
  let canonical: unknown[] | null = null;

  for (let trial = 0; trial < 96; trial += 1) {
    let entries: ConversationTranscriptEntry[] = [history, pendingUser, liveAssistant];
    let baselineSeen = false;
    const preBaselineIds = new Set<string>();
    const order = shuffle(eventKinds, random);

    for (const event of order) {
      if (event === "baseline") {
        entries = reconcileTranscriptFeedBaseline(entries, baseline, new Set(preBaselineIds));
        baselineSeen = true;
        preBaselineIds.clear();
      } else if (event === "attachment-echo" || event === "attachment-replay") {
        if (!baselineSeen) preBaselineIds.add(attachmentEcho.id);
        entries = [...reconcileTranscriptLiveAppend(entries, attachmentEcho, { resolvedNonce: NONCE })];
      } else if (event === "terminal-update") {
        if (!baselineSeen) preBaselineIds.add(terminalAssistant.id);
        entries = [...reconcileTranscriptLiveUpdate(entries, terminalAssistant, { allowInsert: true })];
      } else {
        if (!baselineSeen) preBaselineIds.add(staleStreamingUpdate.id);
        entries = [...reconcileTranscriptLiveUpdate(entries, staleStreamingUpdate, { allowInsert: true })];
      }
    }

    entries = reconcileLateInitialTranscriptPage(entries, latePage);
    const semantic = semanticEntries(entries);
    const diagnostic = "seed=0x" + PROPERTY_SEED.toString(16) + " trial=" + trial + " order=" + order.join(",");
    if (canonical == null) canonical = semantic;
    else assert.deepEqual(semantic, canonical, diagnostic);

    assert.deepEqual(entries.map((entry) => entry.id), ["history-0", "history-1", "user-1", "answer-1"], diagnostic);
    const user = entries.find((entry): entry is TranscriptMessage => entry.kind === "message" && entry.id === "user-1");
    const assistant = entries.find((entry): entry is TranscriptMessage => entry.kind === "message" && entry.id === "answer-1");
    assert.ok(user, diagnostic);
    assert.ok(assistant, diagnostic);
    assert.equal(user.text, "Uploaded report.pdf", diagnostic);
    assert.equal(user.delivery, "sent", diagnostic);
    assert.equal(user.attachments?.length, 1, diagnostic);
    assert.equal(assistant.text, "Final answer", diagnostic);
    assert.equal(assistant.isStreaming, false, diagnostic);
  }
});

test("REC-I03 terminal baseline race survives live-before-baseline and update-before-baseline", () => {
  const baselineFirst = reconcileTranscriptFeedBaseline([], [staleStreamingBaseline], new Set());
  const baselineThenLive = reconcileTranscriptLiveUpdate(baselineFirst, terminalAssistant, { allowInsert: true });
  const baselineThenReplay = reconcileTranscriptLiveUpdate(baselineThenLive, staleStreamingUpdate, { allowInsert: true });

  const liveFirst = reconcileTranscriptLiveUpdate([], terminalAssistant, { allowInsert: true });
  const liveThenBaseline = reconcileTranscriptFeedBaseline(liveFirst, [staleStreamingBaseline], new Set(["answer-1"]));
  const updateBeforeBaseline = reconcileTranscriptLiveUpdate([], terminalAssistant, { allowInsert: true });
  const updateThenBaseline = reconcileTranscriptFeedBaseline(updateBeforeBaseline, [staleStreamingBaseline], new Set(["answer-1"]));
  const ordinaryBaselineMerge = reconcileAuthoritativeTranscriptBaseline([terminalAssistant], [staleStreamingBaseline]);

  for (const result of [baselineThenReplay, liveThenBaseline, updateThenBaseline, ordinaryBaselineMerge]) {
    const terminal = result.find((entry): entry is TranscriptMessage => entry.kind === "message" && entry.id === "answer-1");
    assert.ok(terminal);
    assert.equal(terminal.text, "Final answer");
    assert.equal(terminal.isStreaming, false);
    assert.equal(terminal.clientNonce, "answer-nonce");
  }
});

test("REC-I05 empty and prefix-partial payloads preserve richer committed text and attachment metadata", () => {
  const committed = message("committed", {
    text: "Complete durable response",
    richText: "{\"text\":\"Complete durable response\"}",
    attachments: [ATTACHMENT],
  });
  const empty = message("committed", { text: "", attachments: [] });
  const partial = message("committed", {
    text: "Complete durable",
    richText: "",
    attachments: [{ path: ATTACHMENT.path, name: "", mimeType: "" }],
  });

  const afterEmpty = mergeAuthoritativeTranscriptMessage(committed, empty);
  const afterPartial = reconcileAuthoritativeTranscriptBaseline([committed], [partial])[0];
  assert.equal(afterEmpty.text, committed.text);
  assert.equal(afterPartial?.kind, "message");
  if (afterPartial?.kind === "message") {
    assert.equal(afterPartial.text, committed.text);
    assert.equal(afterPartial.richText, committed.richText);
    assert.deepEqual(afterPartial.attachments, [ATTACHMENT]);
  }
});

test("REC-I06 attachment echo replay deduplicates resources and preserves pending delivery until authoritative settlement", () => {
  const first = reconcileTranscriptLiveAppend([pendingUser], attachmentEcho, { resolvedNonce: NONCE });
  const replay = reconcileTranscriptLiveAppend(first, attachmentEcho, { resolvedNonce: NONCE });
  assert.equal(first.length, 1);
  assert.equal(replay.length, 1);
  const entry = replay[0];
  assert.ok(entry?.kind === "message");
  assert.equal(entry.id, pendingUser.id);
  assert.equal(entry.delivery, "dispatching");
  assert.equal(entry.text, pendingUser.text);
  assert.deepEqual(entry.attachments, [ATTACHMENT]);
});

test("REC-I08 account and conversation owner scopes fence identical ids/nonces from cross-scope updates", () => {
  const key = (account: string, conversation: string) => account + "\u0000" + conversation;
  const owners = new Map<string, readonly ConversationTranscriptEntry[]>([
    [key("account-a", "conversation-a"), [message("shared-id", { text: "A", clientNonce: "shared-nonce" })]],
    [key("account-a", "conversation-b"), [message("shared-id", { text: "B", clientNonce: "shared-nonce" })]],
    [key("account-b", "conversation-a"), [message("shared-id", { text: "C", clientNonce: "shared-nonce" })]],
  ]);
  const applyUpdate = (account: string, conversation: string, incoming: TranscriptMessage) => {
    const ownerKey = key(account, conversation);
    owners.set(ownerKey, reconcileTranscriptLiveUpdate(owners.get(ownerKey) ?? [], incoming, { allowInsert: true }));
  };

  applyUpdate("account-a", "conversation-a", message("shared-id", { text: "updated A", clientNonce: "shared-nonce" }));
  const a = owners.get(key("account-a", "conversation-a"))?.[0];
  const b = owners.get(key("account-a", "conversation-b"))?.[0];
  const c = owners.get(key("account-b", "conversation-a"))?.[0];
  assert.ok(a?.kind === "message");
  assert.ok(b?.kind === "message");
  assert.ok(c?.kind === "message");
  assert.equal(a.text, "updated A");
  assert.equal(b.text, "B");
  assert.equal(c.text, "C");

  const rendererSource = readFileSync(new URL("../../../../production/ProductionRenderer.tsx", import.meta.url), "utf8");
  assert.match(rendererSource, /accountScopeGenerationRef\.current === accountScopeGeneration/);
  assert.match(rendererSource, /onUpdated:[\s\S]*?reconcileTranscriptLiveUpdate\(existing, projected, \{ allowInsert: true \}\)/);
  assert.match(rendererSource, /onAppended:[\s\S]*?reconcileTranscriptLiveAppend\(existing, projected, \{ resolvedNonce \}\)/);
  assert.doesNotMatch(rendererSource, /function (?:mergeTranscriptAttachments|mergeAuthoritativeTranscriptMessage|transcriptEntriesCorrelate|reconcileAuthoritativeTranscriptBaseline|reconcileTranscriptFeedBaseline|reconcileLateInitialTranscriptPage)\s*\(/);
});
