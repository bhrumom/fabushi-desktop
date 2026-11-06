/**
 * Contract & behavioral verification for Human silent (`silent: true`) and
 * scheduled (`scheduledAtMs` / `scheduleDate` / `scheduledAt`) message delivery.
 *
 * Upstream & Open PR Alignment:
 * - Telegram Desktop `Telegram/SourceFiles/api/api_sending.cpp` (`InnerFillMessagePostFlags`
 *   silent/scheduled delivery flags).
 * - PR #111 (`origin/tdrp/human-silent-scheduled-current-main-20261018`):
 *   Preserves `silent` and `scheduledAtMs` across `ComposerSubmissionQueue`,
 *   `sendScopedComposerSubmission`, and `ConversationComposer` send options menu.
 * - Blueprint 03 (`docs/specs/fusion-blueprints/03-unified-composer-and-typed-actions.md`).
 */

// @vitest-environment jsdom
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";
import {
  createComposerSubmissionQueue,
  type ComposerSubmission,
} from "../recovered/features/conversation/workspace/submission";
import { ConversationComposer } from "../recovered/features/conversation/workspace/composer";
import {
  bindComposerSubmissionScope,
  sendScopedComposerSubmission,
  type ComposerDispatchScope,
} from "./composer-dispatch-scope";

const REPO_ROOT = resolve(import.meta.dirname, "../../..");

function readRepoFile(relativePath: string): string {
  return readFileSync(resolve(REPO_ROOT, relativePath), "utf8");
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("Human silent & scheduled message delivery contract", () => {
  it("preserves silent and scheduled delivery options in submission.ts and composer.tsx", () => {
    const submissionSource = readRepoFile("frontend/src/recovered/features/conversation/workspace/submission.ts");
    const composerSource = readRepoFile("frontend/src/recovered/features/conversation/workspace/composer.tsx");

    expect(submissionSource).toContain("silent?: boolean;");
    expect(submissionSource).toContain("scheduledAtMs?: number;");
    expect(submissionSource).toContain("scheduleDate?: number;");
    expect(submissionSource).toContain("scheduledAt?: string;");
    expect(submissionSource).toContain("directTypedPayload?: DirectTypedActionPayload;");

    expect(composerSource).toContain("onSendSilently?(): void | Promise<void>;");
    expect(composerSource).toContain("onScheduleSend?(scheduledAtMs: number): void | Promise<void>;");
    expect(composerSource).toContain("Send Without Sound");
    expect(composerSource).toContain("Schedule Message");
    expect(composerSource).toContain("Choose a future date and time.");
  });

  it("preserves silent, scheduledAtMs, scheduleDate, scheduledAt, and directTypedPayload through offline queueing and scoped Human dispatch", async () => {
    const scope: ComposerDispatchScope = { accountId: "acct-human", generation: 1 };
    let transportDown = true;
    const rpcCalls: { method: string; params: Record<string, unknown> }[] = [];

    const queue = createComposerSubmissionQueue({
      isTransportDown: () => transportDown,
      send: (submission) =>
        sendScopedComposerSubmission({
          client: {
            async call<T = unknown>(method: string, params: Record<string, unknown>): Promise<T> {
              rpcCalls.push({ method, params });
              return { message_id: "msg-scheduled-1" } as T;
            },
          },
          submission: bindComposerSubmissionScope(submission, () => scope),
          isHumanConversation: true,
        }),
    });

    const scheduledMs = 1_900_000_000_000;
    const baseSubmission: ComposerSubmission = {
      nonce: "nonce-sched-1",
      agentId: "peer-human-1",
      prompt: "Quiet reminder",
      attachments: [],
      createdAtMs: 1_800_000_000_000,
      silent: true,
      scheduledAtMs: scheduledMs,
      scheduleDate: Math.floor(scheduledMs / 1000),
      scheduledAt: new Date(scheduledMs).toISOString(),
    };

    const handle = queue.submit(baseSubmission);
    expect(queue.snapshot()[0]?.phase).toBe("queued");
    expect(queue.snapshot()[0]?.silent).toBe(true);
    expect(queue.snapshot()[0]?.scheduledAtMs).toBe(scheduledMs);

    transportDown = false;
    queue.flush();

    await expect(handle.completion).resolves.toBe("sent");
    expect(rpcCalls).toEqual([
      {
        method: "messages.send",
        params: {
          peer_id: "peer-human-1",
          text: "Quiet reminder",
          client_nonce: "nonce-sched-1",
          reply_to_id: undefined,
          attachments: [],
          silent: true,
          scheduled_at_ms: scheduledMs,
          schedule_date: Math.floor(scheduledMs / 1000),
          scheduled_at: new Date(scheduledMs).toISOString(),
        },
      },
    ]);
  });

  it("renders Send options in ConversationComposer and dispatches silent and future-scheduled callbacks", async () => {
    (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    let silentCalls = 0;
    const scheduledTimestamps: number[] = [];

    await act(async () => {
      root.render(
        <ConversationComposer
          draft={{ prompt: "Deploy quietly", attachments: [] }}
          onChange={() => undefined}
          onScheduleSend={(ms) => {
            scheduledTimestamps.push(ms);
          }}
          onSendSilently={() => {
            silentCalls += 1;
          }}
          onStageFiles={() => undefined}
          onSubmit={() => undefined}
          transcribeAudio={async () => ({ text: "" })}
        />
      );
    });

    const optionsTrigger = container.querySelector<HTMLButtonElement>('button[aria-label="Send options"]');
    expect(optionsTrigger).not.toBeNull();

    await act(async () => {
      optionsTrigger?.click();
    });

    const silentItem = Array.from(document.body.querySelectorAll<HTMLElement>('[role="menuitem"]')).find(
      (node) => node.textContent?.includes("Send Without Sound")
    );
    expect(silentItem).toBeDefined();

    await act(async () => {
      silentItem?.click();
    });
    expect(silentCalls).toBe(1);

    await act(async () => {
      optionsTrigger?.click();
    });

    const scheduleItem = Array.from(document.body.querySelectorAll<HTMLElement>('[role="menuitem"]')).find(
      (node) => node.textContent?.includes("Schedule Message")
    );
    expect(scheduleItem).toBeDefined();

    await act(async () => {
      scheduleItem?.click();
    });

    const confirmButton = container.querySelector<HTMLButtonElement>(".sand-prompt-schedule-confirm");
    expect(confirmButton).not.toBeNull();

    await act(async () => {
      confirmButton?.click();
    });
    expect(scheduledTimestamps).toHaveLength(1);
    expect(scheduledTimestamps[0]).toBeGreaterThan(Date.now());

    await act(async () => {
      root.unmount();
    });
  });
});
