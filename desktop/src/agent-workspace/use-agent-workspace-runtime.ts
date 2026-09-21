import { useCallback, useEffect, useRef, useState } from 'react';
import type { ApprovalResolution, AttachmentContext, ComputerStatus } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  createAgentSubmissionQueue,
  type AgentSubmission,
  type AgentSubmissionQueue,
} from '../fabu-runtime/submission-queue';
import {
  clearLegacyAgentWorkspaceDrafts,
  normalizePersistedAgentDrafts,
  readLegacyAgentWorkspaceDrafts,
} from './agent-draft-store';
import { AgentCoordinatorClient, type AgentAttachmentUpload } from './coordinator-client';
import { AgentRuntimeCoordinator } from './agent-runtime-coordinator';
import {
  AgentTranscriptStore,
  type AgentTranscriptTerminalStatus,
} from './agent-transcript-store';
import { AgentWorkspaceController } from './agent-workspace-controller';

export interface UseAgentWorkspaceRuntimeOptions {
  readonly hostReady: boolean;
  readonly coordinatorClient: AgentCoordinatorClient;
  onError?(peerKey: string, message: string): void;
  onComputerStatus?(status: ComputerStatus): void;
  onOperationStarted?(peerKey: string, operationId: string): void;
  onOperationTerminal?(
    peerKey: string,
    operationId: string,
    status: AgentTranscriptTerminalStatus,
    message?: string,
  ): void;
}

export type AgentWorkspaceSubmissionInput = Omit<
  AgentSubmission,
  'nonce' | 'messageId' | 'createdAtMs'
> & {
  readonly messageId?: string;
  readonly createdAtMs?: number;
};

export interface AgentWorkspaceRuntimeFacade {
  readonly controller: AgentWorkspaceController;
  readonly transcriptStore: AgentTranscriptStore;
  readonly coordinator: AgentRuntimeCoordinator;
  readonly revision: number;
  submit(input: AgentWorkspaceSubmissionInput): ReturnType<AgentSubmissionQueue['submit']>;
  openConversation(peerKey: string, conversationId: string): Promise<void>;
  uploadAttachment(peerKey: string, input: Omit<AgentAttachmentUpload, 'requestId'>): Promise<AttachmentContext>;
  resolveApproval(resolution: ApprovalResolution): Promise<void>;
  interrupt(peerKey: string | null | undefined): Promise<void>;
  notify(): void;
}

const AGENT_WORKSPACE_DURABLE_DRAFT_KEY = 'agent-workspace:drafts:v2';
const AGENT_WORKSPACE_DRAFT_WRITE_DEBOUNCE_MS = 250;

let runtimeSubmissionSequence = 0;

function nextRuntimeSubmissionId(prefix: string): string {
  runtimeSubmissionSequence += 1;
  return `${prefix}:${Date.now().toString(36)}:${runtimeSubmissionSequence.toString(36)}`;
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

/**
 * Owns the long-lived Agent workspace runtime boundary for the renderer.
 *
 * Messenger compatibility code may resolve a peer and provide a transport
 * callback, but request/operation ownership, transcript reduction, queued
 * submissions and restart-safe drafts live here. This keeps React views from
 * recreating or coordinating those Agent lifecycles themselves.
 */
export function useAgentWorkspaceRuntime(
  options: UseAgentWorkspaceRuntimeOptions,
): AgentWorkspaceRuntimeFacade {
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const [revision, setRevision] = useState(0);
  const notify = useCallback(() => {
    setRevision((value) => value + 1);
  }, []);

  const durableDraftsHydratedRef = useRef(false);
  const durableDraftReadRequestedRef = useRef(false);
  const controllerRef = useRef<AgentWorkspaceController | null>(null);
  if (!controllerRef.current) {
    controllerRef.current = new AgentWorkspaceController(readLegacyAgentWorkspaceDrafts());
  }
  const controller = controllerRef.current;

  const transcriptStoreRef = useRef<AgentTranscriptStore | null>(null);
  if (!transcriptStoreRef.current) transcriptStoreRef.current = new AgentTranscriptStore();
  const transcriptStore = transcriptStoreRef.current;

  // Declared before the queue because queue sends execute later, after the
  // coordinator has been created below. This avoids routing operation
  // ownership back through the React renderer.
  const coordinatorRef = useRef<AgentRuntimeCoordinator | null>(null);
  const submissionQueueRef = useRef<AgentSubmissionQueue | null>(null);
  if (!submissionQueueRef.current) {
    submissionQueueRef.current = createAgentSubmissionQueue({
      isBlocked: (input) => !optionsRef.current.hostReady || controller.isBusy(input.peerKey),
      send: async (input) => {
        const coordinator = coordinatorRef.current;
        if (!coordinator) throw new Error('Agent runtime coordinator is unavailable.');
        const requestId = nextRuntimeSubmissionId('chat-send');
        coordinator.beginLocalTurn({
          peerKey: input.peerKey,
          requestId,
          messageId: input.messageId,
          text: input.prompt,
          richText: input.richText,
          createdAtMs: input.createdAtMs,
          attachments: input.attachments,
        });
        try {
          const accepted = await optionsRef.current.coordinatorClient.send({
            requestId,
            text: input.prompt,
            conversationId: input.conversationId,
            agentId: input.agentId,
            clientMessageId: input.messageId,
            retryOfMessageId: input.retryOfMessageId,
            attachments: input.attachments,
            replyTo: input.replyTo,
            references: input.references,
          });
          if (!accepted) {
            coordinator.cancelLocalTurn(input.peerKey, requestId, input.messageId);
            throw new Error('Agent prompt was not accepted by Mahayana.');
          }
          const operationId = accepted.operationId ?? requestId;
          if (controller.isOperationFinished(operationId)) {
            controller.cancelRequest(requestId);
            notify();
            return;
          }
          coordinator.adoptOperation(requestId, operationId, input.peerKey);
          coordinator.claimOperation(operationId, input.peerKey);
        } catch (cause) {
          if (controller.peerForRequest(requestId) || controller.peerForRuntimeId(requestId)) {
            coordinator.cancelLocalTurn(input.peerKey, requestId, input.messageId);
          }
          throw cause;
        }
      },
      onPhase: (submission) => {
        if (submission.phase === 'queued') {
          transcriptStore.appendUserMessage(submission.peerKey, {
            id: submission.messageId,
            text: submission.prompt,
            richText: submission.richText,
            createdAtMs: submission.createdAtMs,
            optimistic: true,
            queued: true,
            attachments: submission.attachments,
          });
          notify();
          return;
        }
        transcriptStore.removeQueuedUserMessage(submission.peerKey, submission.messageId);
        notify();
      },
      onFailure: (submission, cause) => {
        transcriptStore.removeQueuedUserMessage(submission.peerKey, submission.messageId);
        controller.restoreDraft(submission.peerKey, {
          text: submission.prompt,
          richText: submission.richText,
          attachments: submission.attachments ? [...submission.attachments] : [],
          references: submission.references ? [...submission.references] : [],
          replyTo: submission.replyTo,
        });
        notify();
        optionsRef.current.onError?.(submission.peerKey, errorMessage(cause));
      },
    });
  }
  const submissionQueue = submissionQueueRef.current;

  if (!coordinatorRef.current) {
    coordinatorRef.current = new AgentRuntimeCoordinator(
      controller,
      transcriptStore,
      {
        onTranscriptChanged: () => notify(),
        onOperationChanged: () => notify(),
        onComputerStatus: (status) => optionsRef.current.onComputerStatus?.(status),
        onWorkspaceState: (key, value) => {
          if (key !== AGENT_WORKSPACE_DURABLE_DRAFT_KEY) return;
          const runtimeDrafts = normalizePersistedAgentDrafts(value);
          const localDrafts = controller.persistedDraftSnapshot();
          // A renderer edit that happened while the Host was connecting wins
          // over the older durable snapshot. Everything is then written back
          // through Rust so localStorage is migration-only.
          controller.hydratePersistedDrafts({
            ...runtimeDrafts,
            ...localDrafts,
          });
          durableDraftsHydratedRef.current = true;
          notify();
        },
        onOperationStarted: (peerKey, operationId) => {
          optionsRef.current.onOperationStarted?.(peerKey, operationId);
        },
        onRequestFailed: (peerKey, _requestId, message) => {
          optionsRef.current.onError?.(peerKey, message);
        },
        onOperationTerminal: (peerKey, operationId, status, message) => {
          submissionQueue.flush();
          optionsRef.current.onOperationTerminal?.(
            peerKey,
            operationId,
            status,
            message,
          );
        },
      },
    );
  }
  const coordinator = coordinatorRef.current;

  const submit = useCallback((input: AgentWorkspaceSubmissionInput) => {
    return submissionQueue.submit({
      ...input,
      nonce: nextRuntimeSubmissionId('agent-submission'),
      messageId: input.messageId ?? nextRuntimeSubmissionId('agent-message'),
      createdAtMs: input.createdAtMs ?? Date.now(),
    });
  }, [submissionQueue]);

  const openConversation = useCallback(async (peerKey: string, conversationId: string) => {
    try {
      await optionsRef.current.coordinatorClient.openConversation(
        nextRuntimeSubmissionId('conversation-open'),
        conversationId,
      );
    } catch (cause) {
      optionsRef.current.onError?.(peerKey, errorMessage(cause));
      throw cause;
    }
  }, []);

  const uploadAttachment = useCallback(async (
    peerKey: string,
    input: Omit<AgentAttachmentUpload, 'requestId'>,
  ): Promise<AttachmentContext> => {
    try {
      return await optionsRef.current.coordinatorClient.uploadAttachment({
        ...input,
        requestId: nextRuntimeSubmissionId('agent-attachment'),
      });
    } catch (cause) {
      optionsRef.current.onError?.(peerKey, errorMessage(cause));
      throw cause;
    }
  }, []);

  const resolveApproval = useCallback((resolution: ApprovalResolution) => {
    return optionsRef.current.coordinatorClient.resolveApproval(resolution);
  }, []);

  const interrupt = useCallback(async (peerKey: string | null | undefined) => {
    const operationId = controller.operationForPeer(peerKey);
    if (!operationId) return;
    await optionsRef.current.coordinatorClient.interrupt(operationId);
  }, [controller]);

  useEffect(() => {
    if (!options.hostReady || durableDraftReadRequestedRef.current) return;
    durableDraftReadRequestedRef.current = true;
    void optionsRef.current.coordinatorClient.readWorkspaceState(
      nextRuntimeSubmissionId('agent-workspace-state-read'),
      AGENT_WORKSPACE_DURABLE_DRAFT_KEY,
    ).catch((cause) => {
      durableDraftReadRequestedRef.current = false;
      optionsRef.current.onError?.('agent-workspace', errorMessage(cause));
    });
  }, [options.hostReady]);

  useEffect(() => {
    if (!options.hostReady || !durableDraftsHydratedRef.current) return;
    const timer = globalThis.setTimeout(() => {
      const snapshot = controller.persistedDraftSnapshot();
      void optionsRef.current.coordinatorClient.writeWorkspaceState(
        nextRuntimeSubmissionId('agent-workspace-state-write'),
        AGENT_WORKSPACE_DURABLE_DRAFT_KEY,
        snapshot,
      ).then(() => {
        clearLegacyAgentWorkspaceDrafts();
      }).catch((cause) => {
        optionsRef.current.onError?.('agent-workspace', errorMessage(cause));
      });
    }, AGENT_WORKSPACE_DRAFT_WRITE_DEBOUNCE_MS);
    return () => globalThis.clearTimeout(timer);
  }, [controller, options.hostReady, revision]);

  useEffect(() => {
    if (options.hostReady) submissionQueue.flush();
  }, [options.hostReady, submissionQueue]);

  useEffect(() => () => {
    submissionQueue.dispose();
    coordinator.dispose();
  }, [coordinator, submissionQueue]);

  return {
    controller,
    transcriptStore,
    coordinator,
    revision,
    submit,
    openConversation,
    uploadAttachment,
    resolveApproval,
    interrupt,
    notify,
  };
}