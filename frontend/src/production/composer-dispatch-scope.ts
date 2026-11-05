/**
 * Account-scoped composer dispatch fencing around Host IPC (`media.upload`,
 * `messages.send`, and `sendPrompt`).
 *
 * Upstream & Open PR Alignment:
 * - PR #190 (`origin/checkpoint/tdrp-nmr50-recovery-20bf9475-r1`): Binds every
 *   composer submission to a `ComposerDispatchOrigin` (`accountId` + `generation`)
 *   at enqueue time so an account switch retires in-flight uploads and queued
 *   dispatches before any cross-account Host IPC call goes on the wire.
 * - PR #111 (`origin/tdrp/human-silent-scheduled-current-main-20261018`) &
 *   Telegram Desktop `Telegram/SourceFiles/api/api_sending.cpp` (`InnerFillMessagePostFlags`):
 *   Preserves `silent`, `scheduledAtMs`, `scheduleDate`, `scheduledAt`, and
 *   `directTypedPayload` across the scoped Human and Agent dispatch boundaries.
 * - Blueprint 03 (`docs/specs/fusion-blueprints/03-unified-composer-and-typed-actions.md`):
 *   Section 2.2 Account-Scoped Dispatch Fencing (`composer-dispatch-scope.ts`).
 */

import { CoordinatorCallError, type CoordinatorCallAdmission, type CoordinatorClient } from "./coordinator-client";
import type { ComposerSubmission } from "../recovered/features/conversation/workspace/submission";

export interface ComposerDispatchScope {
  readonly accountId: string | null;
  readonly generation: number;
}

export interface ComposerDispatchOrigin extends ComposerDispatchScope {
  isCurrent(): boolean;
}

export type ScopedComposerSubmission = ComposerSubmission & {
  readonly dispatchOrigin: ComposerDispatchOrigin;
};

type DispatchCallClient = Pick<CoordinatorClient, "call">;

export function captureComposerDispatchOrigin(
  readCurrentScope: () => ComposerDispatchScope
): ComposerDispatchOrigin {
  const captured = readCurrentScope();
  return {
    accountId: captured.accountId,
    generation: captured.generation,
    isCurrent() {
      const current = readCurrentScope();
      return current.generation === captured.generation && current.accountId === captured.accountId;
    },
  };
}

export function bindComposerSubmissionScope(
  submission: ComposerSubmission,
  readCurrentScope: () => ComposerDispatchScope
): ScopedComposerSubmission {
  return {
    ...submission,
    dispatchOrigin: captureComposerDispatchOrigin(readCurrentScope),
  };
}

function retiredDispatchError(method: string): CoordinatorCallError {
  return new CoordinatorCallError("request-retired", `Composer dispatch retired before ${method}.`);
}

function requireCurrent(origin: ComposerDispatchOrigin, method: string): void {
  if (!origin.isCurrent()) {
    throw retiredDispatchError(method);
  }
}

export function guessComposerAttachmentMime(name: string): string {
  const lower = name.toLowerCase();
  if (lower.endsWith(".png")) return "image/png";
  if (lower.endsWith(".jpg") || lower.endsWith(".jpeg")) return "image/jpeg";
  if (lower.endsWith(".gif")) return "image/gif";
  if (lower.endsWith(".webp")) return "image/webp";
  if (lower.endsWith(".svg")) return "image/svg+xml";
  if (lower.endsWith(".mp3")) return "audio/mpeg";
  if (lower.endsWith(".wav")) return "audio/wav";
  if (lower.endsWith(".ogg") || lower.endsWith(".oga")) return "audio/ogg";
  if (lower.endsWith(".m4a")) return "audio/mp4";
  if (lower.endsWith(".mp4")) return "video/mp4";
  if (lower.endsWith(".webm")) return "video/webm";
  if (lower.endsWith(".mov")) return "video/quicktime";
  if (lower.endsWith(".pdf")) return "application/pdf";
  return "application/octet-stream";
}

export async function sendScopedComposerSubmission(input: {
  client: DispatchCallClient;
  submission: ScopedComposerSubmission;
  isHumanConversation: boolean;
  onHumanSendResult?: (result: unknown) => void;
}): Promise<void> {
  const { origin } = { origin: input.submission.dispatchOrigin };
  const callScoped = <T = unknown>(method: string, params: Record<string, unknown>): Promise<T> => {
    requireCurrent(origin, method);
    const admission: CoordinatorCallAdmission & { readonly isDispatchCurrent: () => boolean } = {
      isCurrent: origin.isCurrent,
      isDispatchCurrent: origin.isCurrent,
    };
    return input.client.call<T>(method, params, admission);
  };

  if (input.isHumanConversation) {
    requireCurrent(origin, input.submission.attachments.length > 0 ? "media.upload" : "messages.send");
    const uploadedAttachments: { path: string; name: string }[] = [];
    for (const attachment of input.submission.attachments) {
      const uploaded = await callScoped<{ id?: string; file_path?: string; file_name?: string }>("media.upload", {
        path: attachment.path,
        mime_type: guessComposerAttachmentMime(attachment.name),
      });
      uploadedAttachments.push({
        path: typeof uploaded?.file_path === "string" && uploaded.file_path.length > 0 ? uploaded.file_path : attachment.path,
        name: typeof uploaded?.file_name === "string" && uploaded.file_name.length > 0 ? uploaded.file_name : attachment.name,
      });
    }
    const result = await callScoped("messages.send", {
      peer_id: input.submission.agentId,
      text: input.submission.prompt,
      client_nonce: input.submission.nonce,
      reply_to_id: input.submission.replyToId,
      attachments: uploadedAttachments,
      ...(input.submission.silent !== undefined ? { silent: input.submission.silent } : {}),
      ...(input.submission.scheduledAtMs !== undefined ? { scheduled_at_ms: input.submission.scheduledAtMs } : {}),
      ...(input.submission.scheduleDate !== undefined ? { schedule_date: input.submission.scheduleDate } : {}),
      ...(input.submission.scheduledAt !== undefined ? { scheduled_at: input.submission.scheduledAt } : {}),
      ...(input.submission.directTypedPayload !== undefined
        ? { direct_typed_payload: input.submission.directTypedPayload }
        : {}),
    });
    if (origin.isCurrent()) {
      input.onHumanSendResult?.(result);
    }
    return;
  }

  await callScoped("sendPrompt", {
    nonce: input.submission.nonce,
    agentId: input.submission.agentId,
    prompt: input.submission.prompt,
    richText: input.submission.richText,
    attachments: input.submission.attachments,
    replyToId: input.submission.replyToId,
    isFork: input.submission.isFork,
    ...(input.submission.silent !== undefined ? { silent: input.submission.silent } : {}),
    ...(input.submission.scheduledAtMs !== undefined ? { scheduledAtMs: input.submission.scheduledAtMs } : {}),
    ...(input.submission.scheduleDate !== undefined ? { scheduleDate: input.submission.scheduleDate } : {}),
    ...(input.submission.scheduledAt !== undefined ? { scheduledAt: input.submission.scheduledAt } : {}),
    ...(input.submission.directTypedPayload !== undefined
      ? { directTypedPayload: input.submission.directTypedPayload }
      : {}),
  });
}
