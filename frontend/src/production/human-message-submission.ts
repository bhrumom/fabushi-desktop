import type { DesktopBridge } from "../recovered/contracts/desktop-bridge";
import { commitComposerAttachments } from "../recovered/features/conversation/workspace/desktop";
import type { ComposerSubmission } from "../recovered/features/conversation/workspace/submission";
import type { ProductionCoordinatorClient } from "./coordinator-client";

export interface HumanComposerSubmissionOptions {
  readonly client: Pick<ProductionCoordinatorClient, "call">;
  readonly bridge: Pick<DesktopBridge, "commitStagedAttachments"> | null;
  readonly isCurrent: () => boolean;
  readonly onCommitted?: (originalPath: string) => void;
}

/**
 * Continue the original Composer queue input until the Host may own its intent.
 * Commit progress stays on that input, not in a second queue or message store.
 * Once a request may have been posted, only the Host can reconstruct its body.
 */
export async function sendHumanComposerSubmission(
  submission: ComposerSubmission,
  options: HumanComposerSubmissionOptions,
): Promise<unknown> {
  const requireCurrent = () => {
    if (!options.isCurrent()) throw new Error("Human Composer submission is outside its current scope");
  };
  requireCurrent();
  const state = submission.humanDispatch ??= { requestPosted: false, committedAttachments: [] };
  if (state.requestPosted) {
    const result = await options.client.call("retryHumanMessage", {
      conversationId: submission.agentId,
      clientNonce: submission.nonce,
    }, { isCurrent: options.isCurrent });
    requireCurrent();
    return result;
  }

  if (state.committedAttachments.length > submission.attachments.length
    || state.committedAttachments.some((committed, index) => committed.name !== submission.attachments[index]?.name
      || typeof committed.path !== "string" || committed.path.length === 0)) {
    throw new Error("Human Composer retained attachment progress conflicts with its original submission");
  }
  // The existing desktop commit boundary can succeed for one file before a
  // later file fails. Retain each acknowledged result before continuing.
  for (let index = state.committedAttachments.length; index < submission.attachments.length; index += 1) {
    requireCurrent();
    const original = submission.attachments[index];
    const committed = options.bridge == null
      ? { path: original.path, name: original.name }
      : (await commitComposerAttachments(options.bridge, [{ path: original.path, name: original.name }]))[0];
    if (committed == null || typeof committed.path !== "string" || committed.path.length === 0) {
      throw new Error("The desktop bridge did not return a committed Human attachment");
    }
    // The commit already happened even if account/selection changed while its
    // reply was in flight. Keep that result, but suppress all obsolete UI work.
    state.committedAttachments.push({ path: committed.path, name: original.name });
    requireCurrent();
    options.onCommitted?.(original.path);
  }

  requireCurrent();
  const result = await options.client.call("sendHumanMessage", {
    conversationId: submission.agentId,
    text: submission.prompt,
    clientNonce: submission.nonce,
    composedAtMs: submission.createdAtMs,
    attachments: state.committedAttachments.map((attachment) => ({ ...attachment })),
    ...(submission.replyToId == null ? {} : { replyToId: submission.replyToId }),
  }, {
    isCurrent: options.isCurrent,
    // This runs at the actual frame boundary. A throwing post or lost response
    // is still possibly admitted and must never replay renderer payload.
    onRequestWillPost: () => { state.requestPosted = true; },
  });
  requireCurrent();
  return result;
}
