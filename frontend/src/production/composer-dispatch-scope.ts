import type { ComposerSubmission } from "../recovered/features/conversation/workspace/submission";
import { CoordinatorCallError, type ProductionCoordinatorClient } from "./coordinator-client";

export interface HumanDispatchConversation {
  readonly id: string;
  readonly name: string;
}

/** A local snapshot of the existing account owner, never a gateway argument. */
export type ComposerDispatchOrigin = Readonly<{
  accountScopeGeneration: number;
  accountSlot: string | null;
  isCurrent(): boolean;
}> & (
  | Readonly<{ kind: "human"; conversation: HumanDispatchConversation }>
  | Readonly<{ kind: "agent" }>
);

export interface ScopedComposerSubmission extends ComposerSubmission {
  readonly origin: ComposerDispatchOrigin;
}

export function createComposerDispatchOrigin(input: {
  readonly accountScopeGeneration: number;
  readonly accountSlot: string | null;
  readonly humanConversation?: HumanDispatchConversation | null;
  readonly isAccountScopeCurrent: (generation: number) => boolean;
}): ComposerDispatchOrigin {
  const { accountScopeGeneration, accountSlot, isAccountScopeCurrent } = input;
  const scope = {
    accountScopeGeneration,
    accountSlot,
    isCurrent: () => accountSlot != null && isAccountScopeCurrent(accountScopeGeneration),
  };
  return input.humanConversation == null
    ? Object.freeze({ ...scope, kind: "agent" as const })
    : Object.freeze({
      ...scope,
      kind: "human" as const,
      conversation: Object.freeze({ id: input.humanConversation.id, name: input.humanConversation.name }),
    });
}

type DispatchClient = Pick<ProductionCoordinatorClient, "call">;
type ComposerAttachments = ComposerSubmission["attachments"];

function requireCurrent(origin: ComposerDispatchOrigin, method: string): void {
  if (!origin.isCurrent()) throw new CoordinatorCallError("request-retired", `${method} request is no longer current`);
}

/** Used by the production queue; ownership must have been captured before enqueue. */
export async function sendScopedComposerSubmission(input: {
  readonly client: DispatchClient;
  readonly submission: ScopedComposerSubmission;
  readonly commitAttachments: (attachments: ComposerAttachments) => Promise<ComposerAttachments>;
  readonly onAttachmentsCommitted?: (submission: ScopedComposerSubmission, attachments: ComposerAttachments) => void;
  readonly onHumanResult?: (submission: ScopedComposerSubmission, result: unknown, conversation: HumanDispatchConversation) => void;
  readonly onHumanRoster?: (submission: ScopedComposerSubmission, value: unknown) => void;
}): Promise<void> {
  const { client, submission } = input;
  const { origin } = submission;
  const method = origin.kind === "human" ? "sendHumanMessage" : "sendPrompt";
  requireCurrent(origin, method);
  if (origin.kind === "human" && origin.conversation.id !== submission.agentId) {
    throw new Error("Human submission does not match its original conversation");
  }
  const attachments = await input.commitAttachments(submission.attachments);
  requireCurrent(origin, method);
  input.onAttachmentsCommitted?.(submission, attachments);
  requireCurrent(origin, method);
  const options = { isDispatchCurrent: origin.isCurrent };

  if (origin.kind === "human") {
    const result = await client.call("sendHumanMessage", {
      conversationId: submission.agentId,
      text: submission.prompt,
      clientNonce: submission.nonce,
      composedAtMs: submission.createdAtMs,
      attachments,
      ...(submission.replyToId == null ? {} : { replyToId: submission.replyToId }),
    }, options);
    // A posted request belongs to Host. Retirement only suppresses its UI work.
    if (!origin.isCurrent()) return;
    input.onHumanResult?.(submission, result, origin.conversation);
    if (!origin.isCurrent()) return;
    const roster = await client.call("listHumanConversations", {}, options);
    if (origin.isCurrent()) input.onHumanRoster?.(submission, roster);
    return;
  }

  await client.call("sendPrompt", {
    agentId: submission.agentId,
    prompt: submission.prompt,
    directAddressedAcceptance: true,
    attachmentPaths: attachments.map((attachment) => attachment.path),
    attachmentNames: attachments.map((attachment) => attachment.name),
    clientNonce: submission.nonce,
    enterEpochMs: submission.createdAtMs,
    composedAtMs: submission.createdAtMs,
    ...(submission.richText == null ? {} : { richText: submission.richText }),
    ...(submission.replyToId == null ? {} : { replyToId: submission.replyToId }),
    ...(submission.isFork === undefined ? {} : { isFork: submission.isFork }),
  }, options);
}

/** Human picker and pill actions share the same rendered-account owner. */
export async function reactScopedHumanMessage(input: {
  readonly client: DispatchClient;
  readonly origin: ComposerDispatchOrigin;
  readonly input: Readonly<{ agentId: string; entryId: string; emoji: string }>;
  readonly onResult: (value: unknown, conversation: HumanDispatchConversation) => void;
}): Promise<void> {
  const { origin } = input;
  requireCurrent(origin, "reactHumanMessage");
  if (origin.kind !== "human" || origin.conversation.id !== input.input.agentId) {
    throw new Error("Human reaction does not match its original conversation");
  }
  const value = await input.client.call("reactHumanMessage", {
    conversationId: input.input.agentId,
    entryId: input.input.entryId,
    emoji: input.input.emoji,
  }, { isDispatchCurrent: origin.isCurrent });
  if (origin.isCurrent()) input.onResult(value, origin.conversation);
}
